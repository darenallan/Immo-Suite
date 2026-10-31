# Note de reprise (31/10/2026, étape 00 close, étape 01 en cours)

À lire en premier après un `/clear`. Ne contient aucun secret.

## Où on en est

- **Étape 00 terminée** : décisions DP-76 à DP-84 écrites, documents corrigés, résultats dans `spikes/*/NOTE.md` (section 8). Identité Git locale : `Hassan <Hassan@habek.cc>`. Les commits sont locaux (pas de dépôt distant).
- **Étape 01 en cours** (`ROADMAP.md` §4). Faits et testés localement : fondation (horloge, configuration, `OrgTx`, stockage de fichiers), migration de fondation, rôles SQL (`ops/sql/roles.sql`), 12 tests de base, serveur `/health` et `/ready`, journal JSON, contrat OpenAPI généré, **module de sauvegarde F39** (copie `pg_dump` par `immo_backup`, chiffrement `age` à clé publique, manifeste SHA-256, vérification, restauration chronométrée, alertes) et **commande `immo backup keygen|run|list|verify [--deep]|check|restore`**, éprouvée sur un scénario réel (copie, vérification, restauration, altération détectée, alerte de copie absente).
- **Reste à faire** à l'étape 01 : file de tâches, CI (rapide, intégration, nocturne), `deny.toml`, image Docker et Compose, scripts de déploiement, front `web/`, squelette du moteur PDF isolé. **Bloqué** par C-01 (comptes), C-02 (hébergement, second fournisseur en écriture seule), C-03 (domaine) et par l'absence de Docker.

## Ce que Mimis doit faire (reste ouvert)

1. Fournir C-01, C-02, C-03 (voir `ROADMAP.md` §6) ; choisir le second fournisseur de copies.
2. Générer et garder en **deux exemplaires** la clé privée de sauvegarde : `immo backup keygen <fichier>` (la clé publique s'affiche, la privée va dans le fichier, hors du dépôt et hors du serveur).
3. Révoquer dans la console Neon la clé d'API `napi_…` et les deux jetons AI Gateway collés dans le chat ; détruire ou archiver les projets Neon `immo-spikes` et `immo-spikes-restore`.
4. Mesure réseau à Ouagadougou (DP-75), quand il le pourra.
5. Relire les choix délégués de DP-83.

## Règles d'exploitation (à respecter)

- **Toute commande passe par `spikes/tools/task.ps1`** (délai, plafond de journal, code de sortie, journaux dans `spikes/out/tasks/`). Lire `spikes/tools/README.md` d'abord. Jamais `python -` ni `python` nu : un tel appel a produit un journal de 3,46 Go.
- **Ne jamais arrêter un processus sans avoir lu son rôle** (`task.ps1 audit`, puis `clean` en simulation). Deux incidents du 31/10/2026 sont décrits dans le README de l'outil : un conhost arrêté a rendu PostgreSQL local inutilisable, et un arrêt de « restes » par parenté de PID a tué 58 processus sans lien (dont une mesure Neon). L'arrêt automatique des restes est supprimé.
- Ne jamais toucher au PostgreSQL de Mimis (port 5432).

## État du poste

- Dossier de session : `C:\Immo Suite` ; **dépôt : `C:\Immo Suite\Immo Suite`**. Dans un nouveau shell PowerShell : `$env:Path = [Environment]::GetEnvironmentVariable('Path','User') + ';' + [Environment]::GetEnvironmentVariable('Path','Machine')`.
- Rust 1.99.1 GNU (sans MSVC), `sqlx-cli` 0.9, Typst 0.15.1, Node 24, Chrome 156, venv Python `spikes/.venv` (`pypdf`, `pymupdf`, `psutil`, `num2words`, `openpyxl`, `fonttools`). Docker et WSL absents.
- **PostgreSQL local jetable** (port 5433, `127.0.0.1`, utilisateur `postgres` sans mot de passe, données dans `%LOCALAPPDATA%\immo-spikes-pg\data`, bases `spike_b`, `spike_c`, `spike_d`) : démarrer par `task.ps1 start -Name pg-spikes -Daemonizes -ReadyPort 5433 -Exe pg_ctl.exe -ArgList -D,<données>,-l,<journal>,start`, arrêter par `pg_ctl stop -m fast` (via `task.ps1 run`). Ne redémarre pas seul.
- **Neon** : projet `immo-spikes` (`royal-lab-36803492`, organisation `org-misty-mountain-86987907`, `aws-eu-central-1`, Postgres 18.6, plan gratuit) avec les branches `production`, `spike-b`, `spike-c` (et `spike-d` si créée) ; projet `immo-spikes-restore` (`spring-resonance-05201307`). Profil du CLI : `--profile agent`. Les URL de connexion et mots de passe de rôles sont dans `%LOCALAPPDATA%\immo-spikes-neon\.env.local` (hors dépôt). Plan gratuit : 100 CU-heures par mois, 1 Go, veille après 5 minutes, fenêtre de restauration de 6 heures ; **CU-heures consommées non relevées** : à lire dans la console.
- Aller-retour PC–Neon mesuré : 138,5 ms.

## Pour reprendre

Dans une nouvelle conversation : « Lis `CLAUDE.md`, puis `spikes/REPRISE.md`, puis reprends l'étape 00 là où elle s'est arrêtée. »


