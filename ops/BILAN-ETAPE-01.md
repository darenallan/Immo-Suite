# Bilan de l'étape 01 — Socle, CI, environnements, sauvegardes (01/11/2026)

**Verdict : le socle est terminé et éprouvé sur le poste de développement. L'étape ne peut PAS être déclarée « Terminée » au sens de `ROADMAP.md`** :
six des neuf critères d'acceptation exigent une plateforme de CI, un hébergeur, un second fournisseur de copies ou un domaine, qui n'existent pas
(C-01, C-02, C-03). Aucune de ces vérifications n'a été faite ni simulée comme si elle l'avait été. Clôture à décider par Mimis (procédure `ROADMAP.md` §8).

## Critères d'acceptation (`roadmap/etape-01-…md` §6)

| # | Critère | État | Preuve ou blocage |
|---|---|---|---|
| 1 | PR avec avertissement clippy, mauvais formatage, dépendance vulnérable ou secret → CI rouge | **non vérifié** | workflow `.github/workflows/ci.yml` écrit, jamais exécuté (C-01). Chaque contrôle passe séparément en local : clippy `-D warnings`, `cargo fmt --check`, `ops/ci/scan-secrets.sh` (essai négatif inclus) |
| 2 | Migration fusionnée puis modifiée → CI rouge | **vérifié en local** | `ops/ci/test-check-migrations.sh` (ajout accepté ; modification, suppression, renommage, antidatage refusés) ; `immo migrate` refuse une migration altérée ; test de somme de contrôle dans `tests/db/tests/foundation.rs` |
| 3 | Test de schéma : 0 table métier passe ; table jouet sans RLS échoue | **vérifié** | `tests/db/tests/foundation.rs` (audit_passe_sans_table_metier, audit_echoue_…) |
| 4 | Fusion sur `main` → déploiement sur l'essai en < 15 min, `/health` = ok, ligne `platform_events` | **non vérifié** | `Dockerfile`, `compose.yaml`, `Caddyfile`, `deploy.sh` écrits, jamais exécutés (pas de Docker, pas d'hébergeur). Vérifiés : `/health` (tests HTTP), `immo event` écrit bien la ligne `deploy.done` |
| 5 | Copie de la veille chez le second fournisseur, illisible sans clé, suppression refusée | **partiel** | vérifié en local avec un dossier : chiffrement `age` à clé publique, illisible avec une mauvaise clé, jamais réécrite. **Non vérifié** : second fournisseur réel, verrouillage d'objet, refus de suppression avec les identifiants du travail |
| 6 | Restauration complète sur instance neuve < 4 h, comptes et sommes identiques, compte rendu (PP-14) | **partiel (à blanc)** | `ops/restauration/COMPTE-RENDU-2026-11-01.md` : 3 organisations, 1 200 lignes, 1,2 s, comptes et sommes identiques. **Non vérifié** : instance neuve chez l'hébergeur, copie réelle, stockage objet, cinq requêtes de contrôle (pas de tables métier) |
| 7 | Sauvegarde retardée de 25 h en essai → alerte sur le canal (PP-15) | **partiel** | l'alerte `backup.missing` se déclenche à 25 h et pas à 23 h (test) ; la commande `immo backup check` sort en code 1 et journalise `alert=backup.missing`. **Aucun canal d'envoi** : décision n° 89 non prise, aucun essai |
| 8 | `cargo audit`, `cargo deny check`, `npm audit` verts ; secrets verts sur l'historique (PP-20) | **vérifié en local** | `cargo audit` (0.22.2) et `cargo deny check` (0.20.2) verts le 01/11/2026 après deux exceptions écrites et datées (`deny.toml`) : RUSTSEC-2026-0387 (crypto-bigint, via `age`, chemin P-256 que nous n'utilisons pas) et RUSTSEC-2026-0374 (basic-toml, compilation seulement) ; `npm audit --omit=dev` : 0 faille ; secrets : arbre et historique propres. Première exécution des outils : ils ont trouvé 3 réglages à corriger dans `deny.toml`, corrigés |
| 9 | Front de la page d'accueil dans les budgets `size-limit` | **vérifié** | JS 58,8 Ko, CSS 1,1 Ko, total 60,1 Ko compressés (budgets 150 / 30 / 200 Ko) |

## Livrables de la section 5 de la fiche d'étape

| § | Livrable | État |
|---|---|---|
| 5.1 | Migration de fondation, rôles SQL, test de schéma, somme de contrôle des migrations | **fait** |
| 5.2 | Espace de travail, `Fcfa`, `RateBp`, lints, tests d'or et propriété P16, file de tâches, `tracing` | **fait** |
| 5.3 | `/health`, `/ready`, OpenAPI généré et versionné, client TypeScript généré, outils notés (DP-84) | **fait** |
| 5.4 | Front Vite/React/TS strict, ESLint, Vitest, size-limit, Playwright 390/1440, tokens, textes | **fait** |
| 5.5 | Squelette `PdfRenderer` et processus isolé | **fait** (isolement système posé par le conteneur : non essayé, pas de Docker) |
| 5.6 | `platform_events` pour déploiement, sauvegarde, restauration | **fait** : `backup.done` / `backup.failed`, `deploy.done` / `deploy.failed` via `immo event`. La ligne « restauration » est à écrire par la procédure de restauration réelle |
| 5.7.1 | Dépôt protégé, hook local | hook **fait** (`ops/hooks/pre-commit`) ; dépôt distant et protection de `main` : **bloqué C-01** |
| 5.7.2 | CI (rapide, intégration, nocturne) | **écrite, jamais exécutée** |
| 5.7.3 | Image, Compose, Caddy, déploiement, promotion | **écrits, jamais exécutés** |
| 5.7.4 | Environnements essai et production, DNS | **non fait : bloqué C-02, C-03** |
| 5.7.5 | Sauvegardes (3 couches) | couche 2 **faite et éprouvée en local** ; couche 1 : Neon plan gratuit (6 h) ; couche 3 : **non faite** (pas de stockage objet) |
| 5.7.6 | Alertes et canal unique | alertes **émises dans le journal** ; canal **non choisi** (n° 89) |
| 5.7.7 | Première restauration complète | **à blanc seulement** (voir critère 6) |
| 5.7.8 | Gouvernance | `ops/INVENTAIRE-ACCES.md` **fait** (état : aucun compte n'existe) ; exercice trimestriel **à planifier** avec Mimis |
| 5.7.9 | Prototype (e) mesure réseau | **reporté** par Mimis (DP-75) |

## Ce qu'il faut de Mimis pour finir (par ordre d'effet)

1. **C-01** comptes d'organisation (plateforme de code et de CI, registre d'images) → débloque les critères 1, 2 (CI), 4 et le dépôt protégé.
2. **C-02** hébergeur (serveur, stockage objet) et **second fournisseur** (écriture seule, verrouillage d'objet) → critères 4, 5, 6 et la couche 3.
3. **C-03** domaine → environnements, DNS, certificats.
4. **Décision n° 89** : canal d'alerte (e-mail d'équipe, messagerie) → critère 7.
5. **Clé de sauvegarde** : `immo backup keygen <fichier>`, deux exemplaires.
6. **Révoquer** la clé d'API Neon `napi_…` et les deux jetons AI Gateway.
7. **Mesure réseau** (e), quand possible.
8. **Relire DP-83** (choix délégués).
