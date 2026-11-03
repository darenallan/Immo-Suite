# Prototype (a) : régénérer les entrées

Les dossiers `cases/` (20 instantanés), `assets/` (logos), `fonts/static/` (polices statiques) et `build/` ne sont pas versés. Depuis `spikes/pdf/`, avec le venv de `spikes/` :

1. `fonttools` : `python -m fontTools.varLib.instancer fonts/IBMPlexSans-Variable.ttf wght=400 -o fonts/static/IBMPlexSans-Regular.ttf` (puis `wght=700` pour Bold) ;
2. `cd words && cargo build --release` (fournit `amount.exe`), puis `python -I make_cases.py` ;
3. `python -I build_html.py`, puis `python -I harness.py render <moteur>`, `check <moteur>`, `bench <moteur>`.

Moteurs : `e1-proc`, `e1-open`, `e2`, `e3-cli`, `e3-lib`. Résultats : `NOTE.md` §8. Lancer les commandes par `spikes/tools/task.ps1`.
