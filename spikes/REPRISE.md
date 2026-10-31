# Étape 00 — Note de reprise (31/10/2026, fin de séance)

À lire en premier après un `/clear`. Ne contient aucun secret.

## Où on en est

Protocoles validés par Mimis le 31/10/2026 ; prototypes **exécutés**, notes rédigées dans `spikes/*/NOTE.md` (section 8 de chacune), **décisions proposées en attente de validation de Mimis**. Rien n'est encore écrit dans `DECISIONS.md` pour les prototypes (règle : valider d'abord). **Aucun commit** : dépôt Git initialisé, `spikes/` et les documents non suivis. Identité Git locale : `hassanhabek <Hassan@habek.cc>` (configurée dans le dépôt, pas en global).

| Prototype | État | Résultat principal |
|---|---|---|
| (b) Isolation avec pool | **fait** (local et Neon) | 0 lecture croisée sur 10 000 itérations (Neon, 4 467 s) ; T4 fuit comme prévu ; pg_dump sous RLS forcée : seul un rôle BYPASSRLS réussit (créable en SQL sur Neon) ; pg_restore vers un second projet : comptes identiques ; restauration à T0 : 100 lignes sur 150, branche créée en 1,9 s ; surcoût : l'aller-retour réseau domine (OrgTx = +4 allers-retours) |
| (c) Numérotation | local et Neon faits | 0 trou, 0 doublon, 0 interblocage sur 19 110 quittances ; idempotence 100/100 ; E-10 et E-27 reproduits |
| (a) PDF | mesuré, **impression papier à faire par Mimis** | trois moteurs tiennent les 20 cas ; Typst en bibliothèque proposé (3 ms, 17 Mo, reproductible) ; montant en lettres : 22 011 valeurs concordantes |
| (d) Import | **fait** (local et comptage sur Neon) | 31/31 fichiers conformes, 197 créables et 3 erreurs aux lignes 14, 87, 152, 8 fichiers piégés neutralisés ; constat : 197 locataires sur 36 lots (R-020) |
| (e) Réseau | **non fait** (DP-75) : Mimis mesure à Ouagadougou | — |

## Ce que Mimis doit faire (reste ouvert)

1. **Valider ou refuser** chaque décision proposée (section « Décision proposée » de chaque note), en particulier : moteur PDF (n° 7), rôle de sauvegarde avec `BYPASSRLS` (n° 46), règles d'import de la section 8.3 de la note (d), règles d'accord du montant en lettres (note a, 8.3).
2. **Imprimer** sur l'imprimante laser noir et blanc les PDF de `spikes/pdf/build/e3-cli/` (cas 01 accents, 06 maximum, 13 annulée, en A4 et A5), depuis le PDF puis depuis un aperçu HTML.
3. **Mesure réseau** à Ouagadougou (DP-75), deux vagues à valider.
4. **En fin d'étape** : révoquer dans la console Neon la clé d'API `napi_…` (collée dans le chat le 31/10/2026) et les deux jetons AI Gateway ; détruire ou archiver les projets Neon `immo-spikes` et `immo-spikes-restore`.

## Ce que Claude fait ensuite

Après validation : écrire les entrées `DP-xx` dans `DECISIONS.md` ; corriger les écarts listés dans DP-69 et DP-71 (`DEPLOYMENT.md` §4.1 et §10, `ARCHITECTURE.md` §3.10 et §10.4, `SECURITY.md` §5.3, `TESTING_QA.md` §6, `ROADMAP.md` J0) ; inventorier les 44 tests de `DATABASE_SCHEMA.md` §9.4 ; mettre à jour `ROADMAP.md` §4 et la section « commandes » de `CLAUDE.md` ; premier commit (un commit par intention, identifiants DP-xx) ; dérouler la liste de démonstration ; **ne pas commencer l'étape 01 sans demande**.

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


