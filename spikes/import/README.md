# Prototype (d) : régénérer les fixtures

Le dossier `fixtures/` (31 fichiers et leurs `*.expected.json`, dont une archive « bombe » et un fichier témoin) n'est pas versé. Depuis `spikes/import/`, avec le venv de `spikes/` :

1. `python -I make_fixtures.py` (a besoin de `reference-source.json`, extrait de `maquette/data/import.js`) ;
2. `cargo build --release`, puis `python -I check_import.py` (31 fichiers attendus conformes) et `python -I perf_import.py` ;
3. garanties de processus : démarrer le PostgreSQL local puis `target/release/spike-import.exe db fixtures/reference.xlsx local`.

Résultats : `NOTE.md` §8. Lancer les commandes par `spikes/tools/task.ps1`.
