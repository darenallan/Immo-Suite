"""Compare le rapport canonique de spike-import à chaque *.expected.json (sous-ensemble des clés présentes).

    check_import.py            tous les fichiers ; affiche un verdict par fichier et un total
    check_import.py <nom>      un seul
Une ligne n'est comparée que sur les clés présentes dans l'attendu ; une clé « rows » impose le même nombre de lignes.
"""
import json
import subprocess
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")
HERE = Path(__file__).parent
FIX = HERE / "fixtures"
EXE = HERE / "target" / "release" / "spike-import.exe"
CANARY = (FIX / "canary.txt").read_text(encoding="utf-8") if (FIX / "canary.txt").exists() else None


def source_of(name):
    for ext in ("xlsx", "csv", "xls", "xlsm"):
        if (FIX / f"{name}.{ext}").exists():
            return FIX / f"{name}.{ext}"
    raise FileNotFoundError(name)


def diff(exp, act):
    out = []
    for k, v in exp.items():
        if k == "rows":
            if len(act.get("rows", [])) != len(v):
                out.append(f"rows : {len(act.get('rows', []))} lignes au lieu de {len(v)}")
                continue
            for e, a in zip(v, act["rows"]):
                for kk, vv in e.items():
                    if a.get(kk) != vv:
                        out.append(f"ligne {e['line']} · {kk} : attendu {vv!r}, obtenu {a.get(kk)!r}")
        elif act.get(k) != v:
            out.append(f"{k} : attendu {v!r}, obtenu {act.get(k)!r}")
    return out


def main(only=None):
    names = sorted(p.name.removesuffix(".expected.json") for p in FIX.glob("*.expected.json") if not only or p.name.startswith(only))
    ok = rows_ok = rows_total = 0
    for n in names:
        exp = json.loads((FIX / f"{n}.expected.json").read_text(encoding="utf-8"))
        r = subprocess.run([str(EXE), "analyze", str(source_of(n))], capture_output=True, text=True, encoding="utf-8", timeout=120)
        if r.returncode != 0:
            print(f"ÉCHEC  {n:<28} code {r.returncode} {r.stderr.strip()[:150]}")
            continue
        act = json.loads(r.stdout)
        d = diff(exp, act)
        if CANARY and CANARY in r.stdout:
            d.append("LE FICHIER TÉMOIN A ÉTÉ LU (entité externe résolue)")
        n_rows = len(exp.get("rows", []))
        rows_total += n_rows
        rows_ok += n_rows - len({x.split(" · ")[0] for x in d if x.startswith("ligne")})
        if d:
            print(f"ÉCART  {n:<28} {len(d)} écart(s)")
            for x in d[:6]:
                print("         ", x)
        else:
            ok += 1
            print(f"OK     {n:<28} {act.get('file')}{' ' + act['refusal'] if act.get('refusal') else ''} · créables {act.get('creatable', '-')}/{act.get('total', '-')}")
    print(f"\n{ok}/{len(names)} fichiers conformes ; lignes détaillées conformes : {rows_ok}/{rows_total}")
    return 0 if ok == len(names) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1] if len(sys.argv) > 1 else None))
