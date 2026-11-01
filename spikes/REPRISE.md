# Note de reprise (01/11/2026 : étape 01 close avec dettes, étape 02 en attente de validation des fiches)

À lire en premier après un `/clear`. Ne contient aucun secret.

## Où on en est

- **État au 01/11/2026** : étape 01 **close sur décision de Mimis** avec des dettes ouvertes (ligne 01 du tableau de `ROADMAP.md` §4, `ops/BILAN-ETAPE-01.md`). Étape 02 démarrée : fiches `specs/F01-comptes-organisation.md`, `F02-roles-permissions.md`, `F03-journal-audit.md` rédigées, **à valider par Mimis avant tout code** (procédure `ROADMAP.md` §8). Les points à trancher sont listés à la fin de chaque fiche.

- **Étape 00 terminée** : décisions DP-76 à DP-84 écrites, documents corrigés, résultats dans `spikes/*/NOTE.md` (section 8). Identité Git locale : `Hassan <Hassan@habek.cc>`. Les commits sont locaux (pas de dépôt distant).
- **Étape 01 en cours** (`ROADMAP.md` §4). Faits et testés localement : fondation (horloge, configuration, `OrgTx`, stockage de fichiers), migration de fondation, rôles SQL (`ops/sql/roles.sql`), 12 tests de base, serveur `/health` et `/ready`, journal JSON, contrat OpenAPI généré, **module de sauvegarde F39** (copie `pg_dump` par `immo_backup`, chiffrement `age` à clé publique, manifeste SHA-256, vérification, restauration chronométrée, alertes) et **commande `immo backup keygen|run|list|verify [--deep]|check|restore`**, éprouvée sur un scénario réel (copie, vérification, restauration, altération détectée, alerte de copie absente).
- **File de tâches faite** (`platform/jobs.rs`, `immo work`) : insertion dans la transaction métier, prise `SKIP LOCKED`, attente croissante, alerte au cinquième échec, reprise des tâches orphelines ; 5 tests de base et un scénario à deux travailleurs réels.
- **Front `web/` fait** (Vite, React, TypeScript strict, ESLint, Prettier, Vitest, size-limit, Playwright 390 et 1440 px, client d'API généré depuis `openapi.json`) : JS 59 Ko compressés sur 150 Ko de budget. `npm audit --omit=dev` : 0 faille ; 14 failles « modérées » dans les outils de développement seulement (minimatch), non livrées.
- **Déploiement, essayé en local** : `immo migrate` (base vide, rejeu idempotent, migration altérée refusée), `immo event` (écrit `platform_events` avec `immo_platform` ; action mal formée refusée), `immo pdf-worker` (rendu en processus enfant, environnement vide, délai, taille bornée ; 3 tests), `ops/ci/check-migrations.sh` et `ops/ci/scan-secrets.sh` (chacun avec son essai dans un dépôt jetable ; arbre et historique du dépôt propres).
- **Écrit mais JAMAIS exécuté** (ni plateforme de CI, ni Docker, aucun compilateur C sur le poste ; `cargo audit` et `cargo deny` ont été installés puis exécutés le 01/11/2026 : verts) : `.github/workflows/ci.yml` et `nightly.yml`, `Dockerfile` (cibles `app` et `ops`), `compose.yaml`, `ops/caddy/Caddyfile`, `ops/deploy/deploy.sh` et `promote.sh` (syntaxe seulement, `bash -n`). Les premières exécutions en CI les corrigeront.
- **Reste à faire** à l'étape 01 : environnements (essai, production), alertes et canal unique, première restauration complète chronométrée sur instance neuve, gouvernance (inventaire des accès). **Bloqué** par C-01 (comptes), C-02 (hébergement, second fournisseur en écriture seule), C-03 (domaine) et par l'absence de Docker.
- **Anomalie notée** : `task.ps1 start` a laissé la tâche `web-install4` à l'état « perdue (superviseur absent) » alors que la commande s'était terminée normalement ; à examiner (ne tue rien).

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


