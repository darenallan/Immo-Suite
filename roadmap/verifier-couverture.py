#!/usr/bin/env python3
"""Contrôle de couverture de la roadmap (lecture seule).

Usage : python -I roadmap/verifier-couverture.py [racine]
Vérifie :
  1. toute fonctionnalité T1/T2 (F01-F14, F39, F40, F15 minimal) est dans une étape ;
  2. toute règle R-001 à R-110 est dans une étape ;
  3. chaque critère PP-01 à PP-38 est livré par une étape ;
  4. chaque contenu C-xx cité a un propriétaire et une échéance dans ROADMAP.md ;
  5. le graphe de dépendances est sans cycle et identique au graphe Mermaid.
"""
import re
import sys
from pathlib import Path

root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent.parent
steps = {}
for f in sorted((root / "roadmap").glob("etape-*.md")):
    n = f.name.split("-")[1]
    txt = f.read_text(encoding="utf-8")
    sec = txt.split("## 10. Couverture")[-1] if "## 10. Couverture" in txt else ""
    d = {}
    for m in re.finditer(r"^- ([^:\n]+?) : (.*)$", sec, re.M):
        d[m.group(1).strip()] = re.sub(r"\([^)]*\)", "", m.group(2))
    steps[n] = d

ok = True
def report(title, missing):
    global ok
    status = "OK" if not missing else "MANQUE " + ", ".join(missing)
    if missing:
        ok = False
    print(f"- {title} : {status}")

def collect(key, pattern):
    found = set()
    for d in steps.values():
        found |= set(re.findall(pattern, d.get(key, "")))
    return found

print(f"Étapes lues : {len(steps)}")
f_req = [f"F{i:02d}" for i in range(1, 15)] + ["F39", "F40", "F15", "F16", "F17", "F18", "F19", "F20", "F22", "F23", "F25"]
report("Fonctionnalités T1, T2 et T3 (F01-F14, F39, F40, F15-F20, F22, F23, F25)",
       [f for f in f_req if f not in collect("F", r"F\d\d")])
report("Règles R-001 à R-110", [f"R-{i:03d}" for i in range(1, 111) if f"R-{i:03d}" not in collect("R", r"R-\d{3}")])
report("Critères PP-01 à PP-38", [f"PP-{i:02d}" for i in range(1, 39) if f"PP-{i:02d}" not in collect("PP", r"PP-\d\d")])

road = (root / "ROADMAP.md").read_text(encoding="utf-8")
rows = {}
for m in re.finditer(r"^\| (C-\d\d) \|.*?\| ([^|]+) \| ([^|]+) \|$", road, re.M):
    rows[m.group(1)] = (m.group(2).strip(), m.group(3).strip())
cited = collect("Contenus", r"C-\d\d")
bad = [c for c in sorted(cited) if c not in rows or not rows[c][0] or not rows[c][1].lower().startswith(("avant", "liste", "volet"))]
report(f"Contenus cités ({len(cited)}) avec propriétaire et échéance", bad)
all_rows_ok = [c for c, (o, e) in rows.items() if not o or not e]
report(f"Contenus de ROADMAP.md ({len(rows)}) complets", all_rows_ok)

deps = {}
for n, d in steps.items():
    deps[n] = set(re.findall(r"\d\d", d.get("Dépend de", "")))
mer = set(re.findall(r"E(\d\d) --> E(\d\d)", road))
file_edges = {(a, b) for b, s in deps.items() for a in s}
report("Graphe Mermaid identique aux « Dépend de » des étapes",
       [f"{a}->{b}" for a, b in sorted(file_edges ^ mer)])

state = {}
def dfs(u, path):
    state[u] = 1
    for v in deps.get(u, ()):
        if state.get(v) == 1:
            return path + [v]
        if state.get(v) is None:
            r = dfs(v, path + [v])
            if r:
                return r
    state[u] = 2
    return None
cycle = None
for n in deps:
    if state.get(n) is None:
        cycle = dfs(n, [n])
        if cycle:
            break
report("Graphe de dépendances sans cycle", [" > ".join(cycle)] if cycle else [])
print("RÉSULTAT GLOBAL :", "VERT" if ok else "ROUGE")
sys.exit(0 if ok else 1)
