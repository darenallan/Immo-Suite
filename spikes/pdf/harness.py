"""Banc du prototype (a) : rend les 20 cas × A4/A5 avec un moteur, contrôle, mesure.

    harness.py render <moteur>      40 PDF dans build/<moteur>/
    harness.py check  <moteur>      contrôles automatiques (TESTING_QA.md §6)
    harness.py bench  <moteur>      100 rendus A4 : médiane, p95, mémoire au pic de l'arbre de processus
    harness.py repro  <moteur>      deux rendus du même instantané : même SHA-256 ?

Moteurs : e1-proc (Chrome, un processus par rendu), e1-open (Chrome maintenu ouvert, via node),
e2 (tracé natif Rust), e3-cli (Typst en ligne de commande), e3-lib (Typst en bibliothèque).
"""
import hashlib
import json
import sys

sys.stdout.reconfigure(encoding="utf-8")  # -I ignore PYTHONIOENCODING ; la console Windows est en cp1252
import re
import statistics
import subprocess
import sys
import threading
import time
from pathlib import Path

import psutil
import pymupdf

HERE = Path(__file__).parent
CHROME = r"C:\Program Files\Google\Chrome\Application\chrome.exe"
TYPST = "typst"
NATIVE = HERE / "native" / "target" / "release" / "spike-native.exe"
TYPST_LIB = HERE / "typst-lib" / "target" / "release" / "spike-typst-lib.exe"
NODE_OPEN = HERE / "chrome-open" / "render.mjs"
CASES = sorted((HERE / "cases").glob("*.json"))
FORMATS = ("a4", "a5")


def case_ids():
    return [json.loads(p.read_text(encoding="utf-8"))["id"] for p in CASES]


# ───────────── rendu ─────────────

def render_one(engine: str, cid: str, fmt: str, out: Path) -> None:
    out.parent.mkdir(parents=True, exist_ok=True)
    if engine == "e1-proc":
        html = HERE / "build" / "html" / f"{cid}-{fmt}.html"
        profile = HERE / "build" / "chrome-profile"
        cmd = [
            CHROME, "--headless=new", "--disable-gpu", "--no-first-run", "--no-pdf-header-footer", f"--user-data-dir={profile}",
            '--host-resolver-rules=MAP * ~NOTFOUND', f"--print-to-pdf={out}", html.as_uri(),
        ]
    elif engine == "e3-cli":
        cmd = [TYPST, "compile", "--root", str(HERE), "--font-path", str(HERE / "fonts" / "static"), "--input", f"case=cases/{cid}.json",
               "--input", f"fmt={fmt}", str(HERE / "quittance.typ"), str(out)]
    elif engine == "e2":
        cmd = [str(NATIVE), str(HERE / "cases" / f"{cid}.json"), fmt, str(out)]
    elif engine == "e3-lib":
        cmd = [str(TYPST_LIB), str(HERE / "cases" / f"{cid}.json"), fmt, str(out)]
    else:
        raise SystemExit(f"moteur inconnu ou à lancer autrement : {engine}")
    r = subprocess.run(cmd, capture_output=True, text=True, cwd=HERE)
    if r.returncode != 0 or not out.exists():
        raise RuntimeError(f"échec {engine} {cid} {fmt} : {r.stderr[:300]}")


def cmd_render(engine: str) -> None:
    if engine == "e1-open":
        subprocess.run(["node", str(NODE_OPEN), "render-all"], check=True, cwd=HERE / "chrome-open")
        return
    t0 = time.perf_counter()
    for cid in case_ids():
        for fmt in FORMATS:
            render_one(engine, cid, fmt, HERE / "build" / engine / f"{cid}-{fmt}.pdf")
    print(f"{engine} : 40 documents rendus en {time.perf_counter() - t0:.1f} s")


# ───────────── contrôles ─────────────

def norm(s: str) -> str:
    s = s.replace("\u00a0", " ").replace("\u202f", " ").replace("\u00ad", "")
    s = re.sub(r"-\s+", "-", s)  # coupure de ligne après un trait d'union : « quatre-/vingt »
    return re.sub(r"\s+", " ", s).strip()


def cmd_check(engine: str) -> int:
    expects = {json.loads(p.read_text(encoding="utf-8"))["id"]: json.loads(p.read_text(encoding="utf-8"))["expect"] for p in CASES}
    tot = dict(pages=0, text=0, fonts=0, meta=0, links=0, color=0, ink=0)
    failures = []
    producers = set()
    for cid in case_ids():
        for fmt in FORMATS:
            f = HERE / "build" / engine / f"{cid}-{fmt}.pdf"
            doc = pymupdf.open(f)
            ok_pages = len(doc) == 1
            tot["pages"] += ok_pages
            text = norm("".join(p.get_text() for p in doc))
            missing = [e for e in expects[cid] if norm(e) not in text]
            tot["text"] += not missing
            fonts = doc[0].get_fonts(full=True)
            # embarquée = programme de police (ttf) ou tracés de glyphes (Type3) ; une police non embarquée aurait ext « n/a » et un autre type
            embedded = all(fo[0] > 0 and (fo[1] != "n/a" or fo[2] == "Type3") for fo in fonts) and bool(fonts)
            type3 = any(fo[2] == "Type3" for fo in fonts)
            tot["type3"] = tot.get("type3", 0) + type3
            tot["fonts"] += embedded
            md = doc.metadata or {}
            producers.add((md.get("producer") or "", md.get("creator") or ""))
            neutral = not (md.get("author") or "").strip() and (md.get("title") or "") in ("Quittance", "")
            tot["meta"] += neutral
            raw = f.read_bytes()
            no_ext = not doc[0].get_links() and not re.search(rb"/(URI|Launch|GoToR|JavaScript|JS)\b", raw)
            tot["links"] += no_ext
            pix = doc[0].get_pixmap(dpi=150, colorspace=pymupdf.csRGB)
            samples = memoryview(pix.samples)
            # pixels colorés : écart entre canaux > 12 (le document doit rester achromatique)
            colored = sum(1 for i in range(0, len(samples), 3) if max(samples[i:i + 3]) - min(samples[i:i + 3]) > 12)
            gray = pix.tobytes("png")  # inutilisé ; la mesure d'encre se fait sur le tampon RVB
            darkest = min(samples[i] for i in range(0, len(samples), 3))
            tot["color"] += colored == 0
            tot["ink"] += darkest <= 40
            if not (ok_pages and not missing and embedded and neutral and no_ext and colored == 0 and darkest <= 40):
                failures.append((cid, fmt, dict(pages=len(doc), missing=missing[:3], embedded=embedded, meta=md.get("author"), links=no_ext, colored=colored, darkest=darkest)))
    n = len(case_ids()) * len(FORMATS)
    print(f"{engine} — sur {n} documents :")
    print(f"  une seule page            : {tot['pages']}/{n}")
    print(f"  texte égal à l'instantané : {tot['text']}/{n}")
    print(f"  polices toutes embarquées : {tot['fonts']}/{n}   (dont en Type3, tracés sans programme de police : {tot.get('type3', 0)}/{n})")
    print(f"  métadonnées neutres       : {tot['meta']}/{n}   (producteur/créateur : {sorted(producers)})")
    print(f"  aucune référence externe  : {tot['links']}/{n}")
    print(f"  achromatique (0 pixel coloré) : {tot['color']}/{n}   texte noir (pixel ≤ 40/255) : {tot['ink']}/{n}")
    for f in failures[:12]:
        print("  ÉCART", f)
    return 1 if failures else 0


# ───────────── performance ─────────────

def tree_rss(proc: psutil.Process) -> int:
    total = 0
    try:
        for p in [proc, *proc.children(recursive=True)]:
            try:
                total += p.memory_info().rss
            except psutil.Error:
                pass
    except psutil.Error:
        pass
    return total


def timed_render(engine: str, cid: str, fmt: str, out: Path) -> tuple[float, int]:
    """Durée (ms) et mémoire au pic (octets) de l'arbre de processus, relevée toutes les 20 ms."""
    peak = 0
    stop = threading.Event()
    holder = {}

    def sampler():
        nonlocal peak
        while not stop.is_set():
            p = holder.get("p")
            if p is not None:
                peak = max(peak, tree_rss(p))
            time.sleep(0.02)

    th = threading.Thread(target=sampler, daemon=True)
    th.start()
    t0 = time.perf_counter()
    # on rejoue la commande de render_one en gardant le processus pour l'échantillonnage
    import types

    real_run = subprocess.run

    def run_spy(cmd, **kw):
        pr = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, cwd=kw.get("cwd"))
        try:
            holder["p"] = psutil.Process(pr.pid)
        except psutil.NoSuchProcess:
            pass  # processus plus court que l'échantillonnage (moteur natif) : voir le mode --repeat
        out_, err_ = pr.communicate()
        return types.SimpleNamespace(returncode=pr.returncode, stderr=err_, stdout=out_)

    subprocess.run = run_spy
    try:
        render_one(engine, cid, fmt, out)
    finally:
        subprocess.run = real_run
    ms = (time.perf_counter() - t0) * 1000
    stop.set()
    th.join()
    return ms, peak


def cmd_bench(engine: str) -> None:
    if engine == "e1-open":
        # un seul processus node qui garde Chrome ouvert ; la mémoire est celle de tout l'arbre (node + Chrome), toutes les 20 ms
        pr = subprocess.Popen(["node", str(NODE_OPEN), "bench"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, cwd=HERE / "chrome-open")
        ps, peak = psutil.Process(pr.pid), 0
        while pr.poll() is None:
            peak = max(peak, tree_rss(ps))
            time.sleep(0.02)
        out, err = pr.communicate()
        times = sorted(json.loads(out))
        p95 = times[int(round((len(times) - 1) * 0.95))]
        print(f"e1-open : {len(times)} rendus A4 — médiane {statistics.median(times):.0f} ms, p95 {p95:.0f} ms, max {times[-1]:.0f} ms ; mémoire au pic de l'arbre {peak / 1e6:.0f} Mo\n  {err.strip()}")
        return
    ids = case_ids()
    out = HERE / "build" / "bench" / f"{engine}.pdf"
    for cid in ids[:3]:  # échauffement non compté
        timed_render(engine, cid, "a4", out)
    times, peaks = [], []
    for _ in range(5):
        for cid in ids:
            ms, peak = timed_render(engine, cid, "a4", out)
            times.append(ms)
            peaks.append(peak)
    times.sort()
    p95 = times[int(round((len(times) - 1) * 0.95))]
    print(f"{engine} : {len(times)} rendus A4 — médiane {statistics.median(times):.0f} ms, p95 {p95:.0f} ms, max {times[-1]:.0f} ms ; mémoire au pic {max(peaks) / 1e6:.0f} Mo (médiane des pics {statistics.median(peaks) / 1e6:.0f} Mo)")


def cmd_repro(engine: str) -> None:
    cid = "20-nominal"
    hashes = []
    for k in range(2):
        out = HERE / "build" / "repro" / f"{engine}-{k}.pdf"
        render_one(engine, cid, "a5", out)
        time.sleep(1.2)  # pour que toute date d'horloge diffère
        hashes.append(hashlib.sha256(out.read_bytes()).hexdigest())
    same = hashes[0] == hashes[1]
    print(f"{engine} : SHA-256 identiques sur deux rendus : {same}")
    if not same:
        a, b = (HERE / "build" / "repro" / f"{engine}-{k}.pdf" for k in range(2))
        da, db = a.read_bytes(), b.read_bytes()
        diffs = [i for i in range(min(len(da), len(db))) if da[i] != db[i]]
        print(f"  tailles {len(da)} / {len(db)} ; {len(diffs)} octets différents ; premier écart à l'octet {diffs[0] if diffs else '-'}")
        if diffs:
            s = max(0, diffs[0] - 40)
            print("  contexte A :", da[s:diffs[0] + 60])
            print("  contexte B :", db[s:diffs[0] + 60])


if __name__ == "__main__":
    cmd, engine = sys.argv[1], sys.argv[2]
    sys.exit({"render": lambda e: cmd_render(e), "check": cmd_check, "bench": lambda e: cmd_bench(e), "repro": lambda e: cmd_repro(e)}[cmd](engine) or 0)
