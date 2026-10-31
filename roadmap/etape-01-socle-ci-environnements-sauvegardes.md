# Étape 01 — Socle : dépôt, intégration continue, environnements, sauvegardes

| Champ | Valeur |
|---|---|
| Statut | À faire |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | Fondations (F39 socle) |
| Effort | L, environ 14 JD |
| Dépend de | Étape 00 (notes validées, décisions n° 7 et 46 prises) ; contenus C-01, C-02, C-03 |
| Jalon | **J0 — Environnement prêt** |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Poser le chantier : dépôt protégé, intégration continue qui bloque les erreurs d'argent et d'isolation, trois environnements séparés, image unique déployée automatiquement sur l'essai, sauvegardes en trois couches et **première restauration complète chronométrée**.
Valeur : les sauvegardes avec test de restauration existent avant toute donnée réelle (règle de l'étape), et chaque étape suivante se livre en petits changements vérifiés.

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections |
|---|---|---|
| Pile, structure du dépôt | `ARCHITECTURE.md` §3.1 à §3.4, §3.9, §3.10 | idem |
| Règles de code, lints, revue | `CODING_RULES.md` | §1, §3, §4.1 à §4.3, §7, §8 |
| Intégration continue, tests | `TESTING_QA.md` | §1.3, §3.1, §10 |
| Exploitation | `DEPLOYMENT.md` | §1, §2, §3, §5 (en entier), §7.3, §10 |
| Sécurité | `SECURITY.md` | §5.3, §5.4, §7.6 |
| Schéma | `DATABASE_SCHEMA.md` | §1.2 (conventions), §6.2 à §6.5, §7.1 |
| Décisions | DP-46 à DP-51, DP-55, DP-56 ; n° 22, 24, 76, 77, 87, 88, 89, 90, 92, 95 | `DECISIONS.md` |

Fonctionnalité : **F39** (sauvegardes chiffrées, restauration testée, journalisation technique). Règles : R-001 (type `Fcfa` et lints), R-002 (`Clock`).

## 3. Prérequis

- Étape 00 terminée et notes validées.
- C-01 : comptes d'organisation (dépôt, CI, registrar, hébergeur, second fournisseur, gestionnaire de mots de passe), double authentification, deux administrateurs, codes de secours placés à deux endroits.
- C-02 : serveur applicatif, stockage objet et second fournisseur de sauvegardes choisis par la grille de `DEPLOYMENT.md` §1.2 ; plan Neon avec sa fenêtre d'historique ; budget mensuel accepté (`DEPLOYMENT.md` §1.3 : 42 à 111 € par mois).
- C-03 : domaine technique provisoire tant que le nom est provisoire (décision n° 13) [À valider, n° 100].
- Décisions prises ou proposées à confirmer : n° 24 (RPO 15 min, RTO 4 h, test mensuel), n° 88 (Compose et proxy à certificats automatiques), n° 92 (secrets), n° 95 (chiffrement des sauvegardes à clé publique), n° 76 et 77 (secrets, licences).

## 4. Fiche de spécification à produire : `specs/F39-sauvegardes.md`

Plan : **règles** (3 couches, RPO 15 min, RTO 4 h, 30 jours d'historique et 12 copies mensuelles, clé hors hébergeur en deux exemplaires, identifiants en écriture seule, aucune sauvegarde valide avant restauration) ; **cas limites** (clé perdue, copie corrompue ou plus petite de moitié, second fournisseur sans verrouillage d'objet, plan Neon avec moins de 30 jours, restauration d'une seule organisation) ; **écrans** (aucun, commandes `immo backup …`) ; **erreurs** (alertes S2 `DEPLOYMENT.md` §7.3) ; **cas chiffrés** (dernière copie de moins de 24 h ; durée totale inférieure à 4 h ; totaux par organisation identiques). **S'arrêter pour validation de Mimis après la fiche.**

## 5. Tâches détaillées

### 5.1 Base de données
1. `migrations/AAAAMMJJHHMM_foundation.sql` (en avant seulement, `SET lock_timeout = '5s'`) : `btree_gist` ; `app_now()` (en production `now()`, redéfinie en essai et en test par `immo_owner`) ; `current_org_id()` ; fonction `forbid_mutation()` ; tables globales `platform_events` (bigint) et `purge_order` ; table `jobs` (type, charge utile, `org_id`, état `queued|running|done|failed`, tentatives, prochain essai, clé d'unicité facultative).
2. `ops/sql/roles.sql` (exécuté par l'administrateur, hors migrations) : `immo_owner`, `immo_app`, `immo_platform`, `immo_backup` (selon la décision n° 46), droits de `DATABASE_SCHEMA.md` §6.3.
3. Test de schéma (`tests/db/schema_audit.rs`) : parcourt `pg_class` ; échoue si une table du schéma métier n'a pas `org_id`, RLS activée **et forcée** et politique ; liste blanche des tables globales ; test négatif (une table jouet sans RLS fait échouer le test).
4. CI : somme de contrôle des migrations déjà fusionnées ; une migration appliquée modifiée fait échouer le build (`CODING_RULES.md` §7 n° 10).

### 5.2 Noyau métier et plate-forme
1. Espace de travail Rust (`Cargo.toml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml`) ; crates `domain` (`Fcfa(i64)` avec addition, soustraction, comparaison, débordement vérifié, bornes 1 à 999 999 999 ; `RateBp(u16)`), `app` (`platform` : `Clock` injectable, `Config` typée et validée, `FileStore` avec disque pour le développement, interface `PdfRenderer`, `Mailer` vide), `server` (binaire `immo`).
2. Lints de l'espace de travail : `unsafe_code = "forbid"` ; `unwrap_used`, `expect_used`, `panic`, `todo`, `dbg_macro`, `print_*` en `deny` hors tests ; `float_arithmetic` et `arithmetic_side_effects` en `deny` dans `domain` ; `disallowed-types` (`PgPool`, `PgConnection`) et `disallowed-methods` (`sqlx::query*` non vérifiées, `Utc::now`, `Local::now`, `SystemTime::now`) avec `#[allow]` réservé à `platform`.
3. Tests d'or de départ (`crates/domain`) : `Fcfa` refuse les flottants et les débordements ; test de propriété P16 (toute valeur produite est un entier dans les bornes).
4. File de tâches : prise par `SELECT … FOR UPDATE SKIP LOCKED`, tentatives avec attente croissante, planificateur avec clé d'unicité ; tâche de démonstration idempotente ; alerte après cinq échecs.
5. `tracing` JSON avec liste blanche de champs (`request_id`, modèle de route, statut, durée, `org_id`, `user_id`, version) ; aucun corps de requête.

### 5.3 API
`GET /health` (corps `{"status":"ok"}`, aucune donnée) et `GET /ready` (réseau interne : base et stockage). Génération du contrat OpenAPI depuis le code, fichier `openapi.json` versionné, comparaison de rupture (C1 et C2), client TypeScript généré dans le même commit (C3). Choisir les outils (`CODING_RULES.md` question 6) et le noter dans `DECISIONS.md`.

### 5.4 Interface
`web/` : Vite, React, TypeScript strict (`strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`), ESLint (`typescript-eslint`, `jsx-a11y`, `react-hooks`), Prettier, Vitest, `size-limit` (HTML+CSS+JS ≤ 200 Ko, JS ≤ 150 Ko, CSS ≤ 30 Ko), Playwright (390 et 1440 px), tokens copiés de `maquette/css/tokens.css`, fichier de textes `texts/fr.ts`, page d'accueil technique affichant la version. Aucune police web.

### 5.5 Documents PDF
Squelette `PdfRenderer` et processus isolé (utilisateur sans droits, sans réseau, système de fichiers en lecture seule) selon le moteur décidé en étape 00. Pas de gabarit de quittance ici.

### 5.6 Audit et événements
Ligne `platform_events` pour chaque déploiement (empreinte de l'image, personne, heure), chaque sauvegarde et chaque restauration (`DEPLOYMENT.md` §2.2 et §5.3).

### 5.7 Infrastructure, sauvegardes et tests
1. **Dépôt et protection** : plateforme sur compte d'organisation, double authentification obligatoire, `main` protégée (CI verte, relecture), fusion par squash, modèle de PR avec la liste de contrôle de `CODING_RULES.md` §8.3, hook local (formatage, lint rapide, détection de secrets sur tout l'historique).
2. **CI** (`TESTING_QA.md` §10) : étape 1 « PR rapide » (≤ 10 min) et étape 2 « PR intégration » (≤ 15 min) avec PostgreSQL réel et les trois rôles de production (`#[sqlx::test]`, aucun nouvel essai automatique) ; étape 3 nocturne (squelette) ; `cargo audit`, `cargo deny` (liste blanche de licences à établir), `npm audit`, `cargo sqlx prepare --check`.
3. **Image et déploiement** : Dockerfile multi-étapes (front puis binaire `--locked --release`), image minimale, utilisateur sans droits, système de fichiers en lecture seule, étiquette = commit, identifiée par empreinte ; publication au registre ; déploiement **automatique sur l'essai** à chaque fusion ; script de promotion manuelle prenant l'empreinte en argument (idempotent) ; Compose et Caddy (certificats automatiques, journal d'accès sans jetons).
4. **Environnements** : développement (Docker local), essai et production **créés** avec secrets, clés, seaux, noms d'hôte et noms de cookies distincts (`DEPLOYMENT.md` §1.1, §2.4) ; DNS, CAA, HSTS sans « preload », SPF et DMARC « aucun expéditeur » (`DEPLOYMENT.md` §3).
5. **Sauvegardes** : couche 1 (continue, plan Neon) ; couche 2 (copie logique quotidienne `pg_dump` au format personnalisé avec `immo_backup`, **chiffrée avant envoi** à clé publique, clé privée hors ligne en deux exemplaires, envoi vers le second fournisseur en écriture seule avec verrouillage d'objet, manifeste SHA-256) ; couche 3 (réplication du seau de fichiers) ; commandes `immo backup run|verify` ; tâche de contrôle nocturne.
6. **Alertes** : sauvegarde absente depuis plus de 24 h, copie de moins de 50 % de la précédente, échec de chiffrement ou d'envoi, verrouillage désactivé, certificat à moins de 14 jours, disque à plus de 80 %, échec de livraison ; canal unique vers le développeur et le porteur du projet, sans donnée personnelle (n° 89).
7. **Première restauration complète chronométrée** sur une instance neuve, avec un jeu synthétique (au moins 3 organisations fictives et 1 000 lignes), selon `DEPLOYMENT.md` §5.4 ; compte rendu au modèle de §5.6 ; essai de suppression avec les identifiants du second fournisseur (doit être refusé).
8. **Gouvernance** : inventaire daté de tous les accès (`DEPLOYMENT.md` §3.3), codes de secours placés, exercice « personne indisponible » planifié au trimestre.
9. **Prototype (e)** : mesure d'aller-retour depuis Ouagadougou vers l'essai (3G et 4G, deux opérateurs) avec un front de 150 Ko ; consigner dans une note et proposer la région définitive.

## 6. Critères d'acceptation vérifiables

1. Une PR avec un avertissement `clippy`, un formatage faux, une dépendance vulnérable de gravité haute ou un secret fait échouer la CI.
2. Une migration déjà fusionnée puis modifiée fait échouer la CI.
3. Le test de schéma passe avec 0 table métier et échoue quand une table jouet sans RLS est ajoutée.
4. Fusionner sur `main` déploie l'image sur l'essai en moins de 15 minutes ; `GET https://<essai>/health` répond `ok` ; la ligne `platform_events` de déploiement existe.
5. Une copie chiffrée de la veille existe chez le second fournisseur ; sa lecture sans la clé privée est impossible ; sa suppression avec les identifiants du travail d'envoi est refusée.
6. Restauration complète sur instance neuve : durée totale inférieure à 4 heures, comptes de lignes et sommes par organisation identiques, compte rendu rempli (PP-14).
7. Une sauvegarde volontairement retardée de 25 heures en essai déclenche l'alerte sur le canal (PP-15).
8. `cargo audit`, `cargo deny check`, `npm audit` verts ; détection de secrets verte sur l'historique (PP-20).
9. Taille du front de la page d'accueil technique inférieure aux budgets de `size-limit`.

## 7. Sécurité et déploiement à vérifier

Secrets distincts par environnement ; `MIGRATION_DATABASE_URL` accessible à la seule étape de déploiement ; aucun secret de production sur un poste ; journaux du proxy sans jeton ni numéro ; TLS `verify-full` vers la base et le stockage ; rôle `immo_backup` jamais utilisé par l'application ; domaine, DNS et registrar au nom de l'organisation avec deux administrateurs.

## 8. Liste de démonstration (Mimis)

1. Ouvrir une PR dont le formatage est faux : voir la CI rouge ; corriger : voir la CI verte ; fusionner.
2. Ouvrir `https://essai.<domaine>/health` et le journal de déploiement (empreinte, heure, personne).
3. Dans la console du second fournisseur : voir la copie chiffrée de la nuit ; essayer de la supprimer avec les identifiants du travail : refus.
4. Lire le compte rendu de restauration chronométrée (« total : … h … min, cible 4 h ») et vérifier les totaux par organisation.
5. Provoquer l'alerte « sauvegarde manquante » en essai et la recevoir sur le canal.

## 9. Définition de terminé, risques, hors étape

Terminé : critères 1 à 9 verts avec preuves au dossier ; `CLAUDE.md` à jour (commandes réelles) ; entrées `DP-xx` pour les outils choisis ; mesure du prototype (e) consignée ; `ROADMAP.md` recalé (vitesse constatée sur les étapes 00 et 01).

| Risque | Mesure |
|---|---|
| Second fournisseur sans écriture seule ni verrouillage d'objet | Copie hors ligne mensuelle (`SECURITY.md` §5.3), noter l'écart, ou changer de fournisseur |
| CI trop lente (> 15 min) | Découpage et cache avant d'ajouter des tests |
| Plan Neon sans 30 jours d'historique | Les 30 copies logiques couvrent l'écart ; coût du plan supérieur à arbitrer par Mimis |
| Nom du produit changé après le domaine | Domaine technique provisoire, cookies sans `Domain` |

Hors étape : toute table métier, toute authentification, toute route autre que `/health` et `/ready`, tout écran.

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 01
- Dépend de : 00
- Tranche : Fondations
- Jalon : J0
- F : F39
- R : R-001, R-002
- E : aucun
- PP : PP-10, PP-13, PP-14, PP-15, PP-20, PP-33
- Contenus : C-01, C-02, C-03
