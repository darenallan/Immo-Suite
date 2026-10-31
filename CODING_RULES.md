# Immo Suite — Règles de code

| Champ | Valeur |
|---|---|
| Statut | Propriétaire des règles de code, de revue, de dépôt et de travail avec Claude Code. Les règles n° 69 à 77 sont retenues par DP-56 ; les règles critiques de la section 7 découlent de décisions déjà prises |
| Date | 30/10/2026 |
| Dépend de | `DECISIONS.md` (état au 30/10/2026), dossier de cadrage v1.0 (§12, §19), `ARCHITECTURE.md` v0.2 (§2.2, §3, §4, §5), `DATABASE_SCHEMA.md` v0.2 (§1, §4, §6, §7), `SECURITY.md` v0.2 (§5.4, §7), `API_CONTRACTS.md` v0.2, `DOMAINE.md`, `CONTEXTE.md`, `UI_UX_DESIGN.md` (tokens, composants, microcopy, accessibilité, performance), `CLAUDE.md` |
| Alimente | `TESTING_QA.md`, `DEPLOYMENT.md`, `CLAUDE.md` |
| Version | 0.2 |

Légende des marqueurs ([PILOTE], [APRÈS], [À valider], [Hypothèse], [À VÉRIFIER AVEC UN JURISTE]) : voir `CLAUDE.md`.

Hiérarchie en cas de conflit : voir `CLAUDE.md` (hiérarchie par sujet). Ce document ne contredit aucun autre ; là où une règle manque, il le dit (« Questions ouvertes ») et ne la devine pas.

Principe directeur : **livrer vite sans jamais compromettre l'intégrité des données financières**. Les règles qui suivent se classent donc en deux familles :

- celles qui **ne se négocient pas** (section 7) : argent, isolation, immutabilité, numérotation, secrets, données de test. Chacune est vérifiée par une machine, pas par la vigilance de quelqu'un ;
- celles qui **accélèrent** (formatage automatique, petits changements, revue légère hors chemins sensibles, interrupteurs de fonctionnalités) : elles existent pour que la première famille ne ralentisse pas.

Indispensable avant le premier pilote :

- **[PILOTE]** Toutes les règles critiques de la section 7 vérifiées automatiquement en CI.
- **[PILOTE]** Formatage, analyse statique sans avertissement, vérification des requêtes SQL à la compilation, audit des dépendances, détection de secrets.
- **[PILOTE]** Revue par la seconde personne pour les chemins sensibles ; définition de « terminé » appliquée.
- **[APRÈS]** Test automatique de l'architecture entre modules, analyse des licences affinée, outils de couverture des tests, métriques de qualité de code.

## 1. Organisation du dépôt

### 1.1 Structure (proposition d'`ARCHITECTURE.md` §3.4, décision n° 23)

```text
Cargo.toml                espace de travail Rust ([workspace.lints] communs)
rust-toolchain.toml       version de Rust figée
rustfmt.toml, clippy.toml, deny.toml
crates/
  domain/                 règles pures, aucune E/S : Fcfa, Rate, dates, affectation, commission,
                          montant en lettres, numéro de quittance, statut et ancienneté d'une échéance
  app/                    modules (identity, property, lease, ledger, receipts, collections,
                          statements, expenses, imports, audit), use_cases/, platform/
  server/                 binaire « immo » : HTTP (Axum), CLI, travailleur de fond
migrations/               SQLx, en avant seulement
templates/                gabarits de quittance et de compte rendu
web/                      front TypeScript (Vite) : src/, tests/, package-lock.json
seeds/demo/               scénario de l'organisation de démonstration
tests/fixtures/           cas E-01 à E-48 (golden), fichiers d'import, cas PDF : données fictives uniquement
tests/contract/           exemples d'API validés contre OpenAPI
ops/                      Dockerfile, proxy, scripts de sauvegarde et de restauration
spikes/                   prototypes jetables (hors build, jamais importés)
maquette/                 référence visuelle, inchangée
*.md                      documents à la racine (DECISIONS.md, DOMAINE.md, …)
```

- **Dépendances entre crates** : `domain` ← `app` ← `server`. `domain` ne dépend ni de SQLx, ni de Tokio, ni de l'horloge : il se teste sans base, vite, par propriétés. Une dépendance qui remonte (par exemple `domain` vers `app`) échoue en CI (`cargo deny` ou vérification de l'arbre `cargo metadata`).
- **Dépendances entre modules** de `app` : graphe sans cycle d'`ARCHITECTURE.md` §2.2. Chaque module n'expose que sa façade `pub mod api` ; ses tables et requêtes sont privées. Vérifié en revue [PILOTE], par un test sur les `use` [APRÈS].
- **Les productions marketing** (`film/`, `flyer/`, `infographie/`, `logo/`, `site/`) et leurs environnements Python `.venv` ne font **pas** partie du dépôt de code (question ouverte d'`ARCHITECTURE.md` §3.4). Si le dépôt est créé à la racine actuelle, un `.gitignore` les exclut avant le premier commit ; sinon le dépôt est créé dans un sous-dossier propre. Aucun `.venv`, `node_modules/`, `target/` ni fichier généré volumineux n'est versionné.
- **Fichiers générés versionnés**, parce que la CI en vérifie la fraîcheur : `.sqlx/` (requêtes vérifiées hors connexion), `openapi.json`, `web/src/api/generated/`. Ils ne se modifient jamais à la main.

### 1.2 Documents

Les documents `.md` de la racine font partie du dépôt. Une règle du domaine se lit dans `DOMAINE.md` **avant** d'être codée ; une décision structurante s'écrit dans `DECISIONS.md` **avant** d'être appliquée (`CLAUDE.md`). Voir section 8.7.

## 2. Conventions de nommage

| Élément | Convention | Exemple |
|---|---|---|
| Langue | Identifiants de code en **anglais** (tables, colonnes, fonctions, routes, codes d'erreur, champs JSON, valeurs d'énumération) ; textes affichés, commentaires métier, documentation et messages de commit en **français**. Décision n° 18 à fermer dans ce sens, que `ARCHITECTURE.md`, `DATABASE_SCHEMA.md` et `API_CONTRACTS.md` suivent déjà | `record_payment`, « Paiement enregistré » |
| Crates, modules, fonctions, variables Rust | `snake_case` | `ledger`, `generate_installments` |
| Types, traits, énumérations Rust | `PascalCase` | `OrgTx`, `PaymentRecorded` |
| Constantes | `SCREAMING_SNAKE_CASE` | `MAX_AMOUNT` |
| Nouveaux types métier | type dédié, jamais un alias de primitive | `Fcfa(i64)`, `RateBp(u16)`, `LeaseId(Uuid)` |
| Tables, colonnes, index, contraintes, fonctions SQL | conventions de `DATABASE_SCHEMA.md` §1.1 (`snake_case`, tables au pluriel, `_at`, `_on`, `_date`, `_bp`, `<table>_by_<colonnes>`, `chk_*`, `*_guard`) | `payments_by_lease` |
| Fichiers de migration | `AAAAMMJJHHMM_description.sql`, un objectif par fichier | `202611021030_add_row_version.sql` |
| Routes et champs JSON | `API_CONTRACTS.md` §2 : `/api/` + ressources au pluriel, `snake_case`, énumérations anglaises | `GET /api/correction-requests` |
| Codes d'erreur | `domaine.motif` en `snake_case`, stables | `payment.external_ref_duplicate` |
| Variables d'environnement | `MAJUSCULES_AVEC_TIRETS_BAS`, préfixe du composant | `DATABASE_URL`, `S3_BUCKET` |
| Composants React | `PascalCase`, un composant par fichier, nommé comme le fichier | `PaymentForm.tsx` |
| Fichiers TypeScript | `kebab-case` hors composants ; dossiers d'écran nommés d'après le catalogue | `screens/s04-encaisser/` |
| Hooks, fonctions, variables TypeScript | `camelCase` ; hooks préfixés `use` | `usePaymentContext` |
| Clés de texte (interface) | `domaine.contexte.élément` | `payment.error.external_ref_duplicate` |
| Événements d'usage | `snake_case`, verbe au passé, liste blanche d'`ARCHITECTURE.md` §8.4 | `payment_form_opened` |
| Identifiants de traçabilité dans le code | `R-xxx`, `E-xx`, `DP-xx`, `F-xx` dans les commentaires et noms de tests | `// R-050 (DP-46) : numéro attribué en dernier` |

- Un commentaire dit **pourquoi** (règle, décision, piège), jamais ce que le code dit déjà. Tout code qui applique une règle d'argent cite son identifiant `R-xxx`.
- Un nom de test décrit le comportement et cite son cas : `partial_payment_leaves_remaining_balance_e05`.

## 3. Formatage, analyse statique, configuration, secrets

### 3.1 Rust

| Outil | Règle | Statut |
|---|---|---|
| `rustfmt` | `cargo fmt --check` bloquant ; configuration dans `rustfmt.toml` | [PILOTE] |
| `clippy` | `cargo clippy --all-targets --locked -- -D warnings` : **zéro avertissement** | [PILOTE] |
| Lints de l'espace de travail | `[workspace.lints]` : `unsafe_code = "forbid"` ; `clippy::unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `dbg_macro`, `print_stdout`, `print_stderr` en `deny` hors tests | [PILOTE] |
| Lints propres à l'argent | `clippy::float_arithmetic` en `deny` dans `domain` et dans les modules d'argent ; `clippy::arithmetic_side_effects` en `deny` dans `domain` (débordements) | [PILOTE] |
| Lints d'accès | `clippy.toml` : `disallowed-types` (`sqlx::PgPool`, `sqlx::Pool`, `sqlx::PgConnection`) et `disallowed-methods` (`sqlx::query`, `sqlx::query_as`, `sqlx::query_scalar` non vérifiées, `chrono::Utc::now`, `chrono::Local::now`, `std::time::SystemTime::now`) ; `#[allow]` seulement dans `platform`, avec un commentaire (décision n° 75) | [PILOTE] |
| Dépendances | `cargo audit` et `cargo deny check` bloquants (avis de gravité haute ou critique, licences, sources, doublons) ; `Cargo.lock` versionné ; `--locked` partout | [PILOTE] |
| Requêtes SQL | `cargo sqlx prepare --check` bloquant ; `.sqlx/` versionné | [PILOTE] |
| Temps de compilation | surveillé (crates séparés) ; seuil d'alerte à définir [Hypothèse] | [APRÈS] |

- La version de Rust est figée par `rust-toolchain.toml` ; une mise à jour est une PR dédiée.
- Un `#[allow(…)]` est exceptionnel : il porte un commentaire qui cite la raison et il est relu. Un test de CI compte les `#[allow]` et échoue au-delà d'un plafond fixé à la première mesure.

### 3.2 TypeScript

| Outil | Règle | Statut |
|---|---|---|
| `tsc` | mode strict : `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `noImplicitOverride`, `noFallthroughCasesInSwitch` ; `tsc --noEmit` bloquant | [PILOTE] |
| ESLint | `typescript-eslint` (règles « strict »), `eslint-plugin-jsx-a11y`, `react-hooks` ; `no-explicit-any`, `ban-ts-comment` (pas de `@ts-ignore`), `no-restricted-syntax` pour `dangerouslySetInnerHTML`, `eval`, `new Function` ; zéro avertissement | [PILOTE] |
| Prettier | formatage automatique, bloquant en CI | [PILOTE] |
| Poids | contrôle de taille compressée par `size-limit` : HTML+CSS+JS ≤ 200 Ko, JavaScript ≤ 150 Ko, CSS ≤ 30 Ko (`UI_UX_DESIGN.md`, Performance) ; bloquant | [PILOTE] |
| Dépendances | `package-lock.json` versionné, `npm ci`, `npm audit` bloquant au seuil « haute », scripts d'installation désactivés sauf liste blanche | [PILOTE] |

### 3.3 Configuration

- **Une seule structure typée** `Config`, remplie au démarrage depuis les variables d'environnement, validée (valeurs obligatoires, plages, URL). Une configuration invalide **arrête le démarrage** avec un message clair. Aucune lecture de variable d'environnement ailleurs.
- Les variables et leurs valeurs par environnement sont dans `DEPLOYMENT.md`. Le dépôt contient un `.env.example` **sans aucune valeur sensible**.
- Les paramètres qui varient par organisation (préfixe de quittance, seuil de confirmation, durée des liens, droits réglables) sont dans `org_settings`, pas dans la configuration ni dans le code (`ARCHITECTURE.md` §8.3). Aucun identifiant ni nom d'organisation dans le code (`if org_id == …` est interdit : règle d'or 12).

### 3.4 Secrets

- **Aucun secret dans le dépôt**, aucun fichier `.env` versionné (`.gitignore`). Détection de secrets sur tout l'historique en CI et en pré-validation locale [PILOTE] (outil : décision n° 76).
- Un secret est enveloppé dans un type qui **masque** son `Debug` et son `Display` (par exemple un type `Secret<T>` d'une bibliothèque reconnue) ; il n'est jamais journalisé, jamais renvoyé dans une erreur, jamais sérialisé dans l'API.
- Les postes de développement n'ont **aucun** secret de production ; les secrets de production ne sont pas accessibles aux branches de test (`SECURITY.md` §5.4, §7.6).
- Un secret exposé est révoqué et remplacé immédiatement (procédure de `DEPLOYMENT.md`).

## 4. Rust

### 4.1 Erreurs (décision n° 69)

- **Contexte** : code financier ; une erreur doit être traitée, journalisée et traduite en réponse, jamais faire tomber un processus au milieu d'une transaction.
- **Options** : (a) types d'erreur propres (`thiserror`) dans les bibliothèques, `anyhow` seulement dans le binaire et la CLI ; (b) `anyhow` partout ; (c) `Box<dyn Error>`.
- **Choix recommandé** : (a) [À valider, décision n° 69].
- **Raison** : (a) permet de faire correspondre chaque erreur à un code stable d'`API_CONTRACTS.md` et de la tester ; (b) cache la nature de l'erreur ; (c) est moins riche.
- **Règles** :
  - Chaque module expose une énumération d'erreurs métier (`LedgerError::ExternalRefDuplicate { … }`) ; la couche HTTP seule les traduit en `problem+json` (code, `params`, statut). Une erreur de base de données imprévue devient `server.unexpected` avec un identifiant de requête, **sans** le texte SQL ni les valeurs dans la réponse.
  - **`unwrap()`, `expect()`, `panic!`, `unreachable!` hors tests sont interdits en production** (lints). Les indexations qui peuvent échouer utilisent `get()`. Exceptions rarissimes (invariant prouvé par construction) : commentaire qui cite la preuve, relue.
  - Un `panic` imprévu est capturé par une couche du serveur qui répond `500`, journalise et alerte ; il ne tue ni le processus ni le travailleur de fond.
  - Les conversions numériques qui peuvent perdre de l'information (`as`) sont interdites pour l'argent ; on utilise `TryFrom`.
  - Les opérations qui peuvent échouer renvoient `Result` ; on ne l'ignore jamais (`#[must_use]`, lint `let_underscore_must_use` sur les chemins d'argent).

### 4.2 Argent

- **Type dédié** `Fcfa(i64)` (nouveau type, pas d'alias) dans `domain` : addition, soustraction, comparaison, négation, somme avec **débordement vérifié** ; **aucune** multiplication ni division par un flottant, aucune conversion implicite (R-001, DP-45). Construction par `Fcfa::new(i64) -> Result<Fcfa, AmountError>` qui applique les bornes de `DATABASE_SCHEMA.md` §1.2 (1 à 999 999 999 en valeur absolue pour un paiement, un loyer, un reversement ; 0 à 999 999 999 pour charges, dépôt, dépense).
- **Taux** : type `RateBp(u16)` (centièmes de point, 0 à 10 000, R-015). La seule opération qui arrondit est `RateBp::commission(base: Fcfa) -> Result<Fcfa, …>` : `(base × taux + 5 000) / 10 000` en division entière sur un entier intermédiaire plus large (`i128`), base négative **refusée** (règle non définie, `API_CONTRACTS.md` §6.3). Une seule opération d'arrondi dans tout le code (DP-14, E-26).
- **Montant en lettres** : fonction de `domain`, testée de 0 à 999 999 999 (R-053, `TESTING_QA.md`), jamais produite par le moteur PDF ni le front.
- **Statut, ancienneté et tranche d'une échéance** : **une seule fonction** de `domain`, appelée par toutes les lectures (tableau de bord, impayés, fiche, relevé) ; jamais recopiée en SQL ni en TypeScript (`ARCHITECTURE.md` §4.9, DP-51).
- **Affectation** : fonction pure de `domain` qui prend l'état des lignes ouvertes et le montant et renvoie la répartition (R-043, R-087, R-044). Les propriétés du livre (somme affectée ≤ paiement, contre-passation exacte) y sont testées par propriétés.
- **Pas d'argent dans les journaux, les événements d'usage ni les noms de fichier** au-delà de ce que `SECURITY.md` autorise (montants arrondis par tranche si utiles).
- Piège SQL : dans une requête, `sum(bigint)` renvoie un `numeric` ; toute commission calculée en SQL convertit en `bigint` avant de diviser (`DATABASE_SCHEMA.md` §4.6). La règle est dans `domain` ; le SQL ne renvoie que des montants.

### 4.3 Temps (décision n° 70)

- **Règle** : aucune lecture directe de l'horloge dans `app` ni `domain`. Le service `Clock` (injectable) fournit l'instant et la **date locale de l'organisation** ; en SQL, toute lecture passe par `app_now()` et `org_local_date()`, jamais par `CURRENT_DATE` ni `now()` direct (DP-50). Les lints de 3.1 interdisent `Utc::now`, `Local::now` et `SystemTime::now` hors `platform::clock`.
- Dates métier : type date sans fuseau ; instants : UTC. Le fuseau `Africa/Ouagadougou` est une propriété de l'organisation (R-002, R-003), même s'il vaut UTC+0.
- **Bibliothèque** : `chrono` avec `chrono-tz`, ou `jiff`. Recommandation : `chrono` + `chrono-tz` [À valider, décision n° 70] parce que SQLx le gère nativement ; `jiff` est une alternative plus moderne dont la prise en charge par la version de SQLx retenue est à vérifier.
- Les tests fixent l'horloge au 13/11/2026 09:15 (jeu de démonstration). Un test de parité vérifie que `Clock` et `app_now()` donnent la même date.

### 4.4 Base de données (SQLx)

- **Requêtes vérifiées à la compilation** : macros `query!`, `query_as!`, `query_scalar!` ; `.sqlx/` versionné ; `cargo sqlx prepare --check` bloquant. Les fonctions non vérifiées (`sqlx::query`) sont interdites sauf dans `platform` (lint).
- **Contexte d'organisation obligatoire** : toute fonction qui touche la base prend `&mut OrgTx` ; jamais `&PgPool` ni `&PgConnection` hors de `platform` (DP-48, `ARCHITECTURE.md` §5.3). `OrgTx` ne se construit que par `OrgTx::begin(&pool, org_id, user_id)`, qui exécute `set_config('app.org_id', …, true)` en première instruction. L'identité vient de la session vérifiée, jamais d'un paramètre de requête. Les tâches de fond construisent leur `OrgTx` depuis l'`org_id` de la tâche.
- **Chaque requête filtre aussi `org_id` explicitement** (lisibilité, plans d'exécution, index) : la sécurité au niveau des lignes est la seconde barrière, pas la première.
- **Pas de SQL construit par concaténation** (`format!` dans une requête est interdit) ; tri et filtres venant du client passent par des énumérations en liste blanche.
- **Pas de `SELECT *`** ; colonnes nommées ; pas de colonne de statut dérivé écrite en base (DP-51).
- **Transactions** : une action métier = une transaction ; niveau `READ COMMITTED` (le contrôle différé des affectations en dépend, `DATABASE_SCHEMA.md` §4.5) ; on ne le change jamais sans revoir ce point. Ordre d'une écriture d'argent : contrôles métier → verrou consultatif du bail (**même fonction de clé** que la base : `hashtextextended(lease_id::text, 0)`) → insertions → **compteur de quittance en dernier** → instantané → audit → insertion de la tâche de rendu PDF. Aucun appel au stockage de fichiers, au moteur PDF ni au réseau dans une transaction.
- **Audit dans la même transaction** que l'action (R-008) : si l'écriture d'audit échoue, l'action est annulée. Une action sensible sans ligne d'audit est une erreur de revue.
- **Écriture seule** : jamais `UPDATE` ni `DELETE` sur `payments`, `allocations`, `receipts`, `reminders`, `audit_log` et les autres tables d'écriture (liste de `DATABASE_SCHEMA.md` §4.2). Une correction est une nouvelle ligne.
- **Délais** : `statement_timeout` et `lock_timeout` posés par l'application ; taille du pool bornée ; arrêt propre des connexions.
- **Migrations** : en avant seulement, rétrocompatibles, un objectif par fichier, `SET lock_timeout = '5s'` en tête, jamais modifiées une fois appliquées ; règles de `DATABASE_SCHEMA.md` §7. Une migration qui touche une garde (déclencheur, contrainte d'écriture seule, politique d'isolation) est relue par **deux** personnes. Toute nouvelle table métier porte `org_id`, la sécurité au niveau des lignes forcée, une politique, une ligne dans `purge_order` ; le test de schéma l'impose.

### 4.5 Journalisation structurée

- `tracing` en JSON sur la sortie standard ; `#[instrument(skip_all, fields(…))]` avec une **liste blanche** de champs : identifiant de requête, route (le **modèle**, par exemple `/api/payments/{id}`, jamais l'adresse réelle), statut, durée, `org_id`, `user_id`, version (`SECURITY.md` §7.7).
- **Jamais** : nom, téléphone, corps de requête, texte de relance, mot de passe, jeton, cookie, valeur chiffrée, adresse contenant un jeton.
- Niveaux : `error` = un humain doit regarder (alerte) ; `warn` = anormal mais géré ; `info` = événement métier notable ; `debug` désactivé en production. Chaque erreur non prévue porte l'identifiant de requête affiché à l'utilisateur.
- Un test de CI cherche des numéros `+226` et des motifs de jetons dans les journaux d'un parcours complet.

### 4.6 Dépendances

- **Peu nombreuses et justifiées** : toute nouvelle dépendance est lue en revue (maintenance récente, taille de l'arbre transitif, usage de `unsafe`, licence) et notée dans la description de la PR. Préférer la bibliothèque standard à une dépendance pour un besoin de quelques lignes.
- Licences : `cargo deny` avec une liste d'autorisations explicite (décision n° 77) ; toute licence copyleft forte est refusée.
- Mises à jour hebdomadaires (outil automatique de PR) ; une faille connue se corrige en 72 heures (`SECURITY.md` §7.6).
- TLS en Rust pur (`rustls`) de préférence à une dépendance système, pour une image minimale.

### 4.7 `unsafe`

**Interdit** (`unsafe_code = "forbid"`). Une exception exige : une entrée `DECISIONS.md` qui la justifie, un module isolé, deux relecteurs, un test qui exerce le chemin, et le retrait du `forbid` pour ce seul crate. Aucune exception n'est prévue au pilote.

## 5. Front TypeScript

### 5.1 Structure (proposition)

```text
web/src/
  app/             routage, mise en page, gardes de rôle, bandeau de mise à jour
  screens/         un dossier par écran du catalogue (s03-accueil, s04-encaisser, s05-impayes, …)
  components/      primitives de la maquette : Button, Field, AmountField, Table/CardList,
                   Badge, Tabs, Dialog, Sheet, Toast, EmptyState, Skeleton, AsyncView
  api/             client généré (generated/) + enveloppe d'appel (en-têtes, jeton anti-falsification, clés d'idempotence)
  texts/           fr.ts : tous les textes affichés, par clé ; correspondance code d'erreur → texte
  format/          formatage d'affichage uniquement : formatFcfa, formatDate, formatPhone
  styles/          tokens.css (copie conforme des tokens de UI_UX_DESIGN.md), base.css
  tests/           tests de composants et de formatage
```

- **Découpage par écran** (`import()` dynamique) : l'écran Encaisser (S04) se charge seul au premier accès du caissier. Bibliothèques limitées et listées : routeur, primitives accessibles (dialogue, feuille, onglets), client généré. **Pas** de bibliothèque de composants complète, pas de CSS-in-JS, pas de bibliothèque de dates lourde, pas de police web, pas de `lodash` entier ni de `moment` (décisions n° 71 et 72).
- **État des données** : client généré + une enveloppe fine ; pas de bibliothèque de cache au pilote [À valider, décision n° 71]. Rien d'une réponse d'API n'est mis en cache durablement (ni service worker, ni stockage du navigateur) : `Cache-Control: no-store` et `ARCHITECTURE.md` §3.6.

### 5.2 Règles

| Sujet | Règle |
|---|---|
| **Aucun calcul d'argent** | Le front affiche ce que renvoie l'API : aucune addition, soustraction, multiplication, arrondi ni pourcentage sur un montant, y compris l'aperçu d'affectation, le reste dû et la commission (DP-45, `API_CONTRACTS.md` §1.1). Les montants sont typés `Fcfa` (type marqué) ; la seule fonction qui les formate est `formatFcfa`. Comparer un montant à zéro pour un affichage est permis |
| **Tokens** | Couleurs, espacements, rayons, ombres, tailles de texte : uniquement des variables de `tokens.css` (copie conforme de `UI_UX_DESIGN.md`, palette DP-44 : bleu nuit `#142B4A`, turquoise `#159D91` en remplissage ou bordure seulement, jamais en texte ; teintes dérivées pour les contrastes). Aucune valeur de couleur ni de dimension écrite en dur dans un composant ; un test compare `tokens.css` au bloc de tokens du document de design. Un statut financier n'utilise **jamais** le turquoise |
| **Formulaires** | Étiquette toujours visible (jamais le seul `placeholder`) ; « (obligatoire) » en texte ; validation à la sortie du champ et à l'envoi ; erreur sous le champ, liée par `aria-describedby`, avec résumé en tête ; montant en `inputmode="numeric"`, entiers seulement, séparateur de milliers automatique, suffixe « FCFA » ; téléphone avec préfixe « +226 » fixe ; date JJ/MM/AAAA ; mode « Espèces » et date du jour par défaut (date du jour fournie par le **serveur**) ; la validation du front est un **confort**, le serveur valide toujours |
| **Clé d'idempotence** | Créée à l'ouverture du formulaire d'écriture d'argent, conservée dans l'état de l'écran, réutilisée à chaque nouvel essai, régénérée pour une nouvelle saisie. Le bouton passe en « chargement » et bloque le double clic |
| **Brouillon** | Les données saisies survivent à une erreur ou une coupure (état de l'écran, éventuellement `sessionStorage` effacé à la déconnexion). Aucune donnée métier dans `localStorage` |
| **Textes centralisés** | Tout texte affiché est dans `texts/fr.ts`, par clé ; aucune chaîne française dans un composant, sauf une clé. Vouvoiement provisoire (décision n° 17). Mots interdits à l'écran : identifiant technique, « null », « erreur 500 », « token », « supprimer » pour un paiement ou une quittance. Chaque code d'erreur du catalogue d'`API_CONTRACTS.md` a un texte ; un test le vérifie |
| **Erreurs** | Dire quoi faire, jamais de code technique, `[Réessayer]` quand cela a un sens, code d'incident (`request_id`) affiché en petit. États : champ, bandeau de page, page d'erreur, quittance en échec (le paiement est enregistré), conflit |
| **Chargement** | Composant commun `AsyncView` : squelette aux dimensions du contenu (pas de rond plein écran), pulsation d'opacité supprimée si `prefers-reduced-motion`, message « La connexion est lente… » et [Réessayer] après 10 secondes. Chaque écran de données gère les quatre états : chargement, vide (avec action), erreur, succès |
| **HTML** | Jamais d'insertion de HTML brut ; texte libre (notes, noms) toujours traité comme du texte (`SECURITY.md` §7.2) |
| **Rôles** | Le front masque ce que le rôle n'a pas le droit de faire ; le serveur refuse de toute façon. Il se fonde sur `GET /api/me` (`permissions`, `features`), jamais sur un rôle recopié |
| **Appels réseau** | Uniquement par le client généré (`web/src/api/`) ; `fetch(` interdit ailleurs (test de CI) |
| **Énumérations** | Valeurs inconnues gérées par un repli neutre (« Autre ») : le serveur peut en ajouter (`API_CONTRACTS.md` §1.3) |

### 5.3 Accessibilité (WCAG 2.1 AA, `UI_UX_DESIGN.md`)

- Contraste de texte ≥ 4,5:1, composants ≥ 3:1, texte principal quasi noir sur fond clair (plein soleil), texte utile ≥ 14 px. **Couleur jamais seule** : icône + libellé + couleur sur chaque statut.
- Cibles tactiles ≥ 44 × 44 px, espacées de 8 px. Tout au clavier, ordre de tabulation logique, anneau de focus visible ; seule une modale piège le focus (focus initial sur [Annuler] pour une action irréversible, Échap ferme).
- `lang="fr"`, un seul `h1` par écran, repères de page, tableaux sémantiques sur ordinateur et listes de cartes sur mobile.
- Résultats dynamiques annoncés (`aria-live`) ; montant lu en toutes lettres par les lecteurs d'écran.
- Zoom jusqu'à 200 % sans défilement horizontal ; `prefers-reduced-motion` respecté ; aucune animation de plus de 200 ms.
- Vérification : règles `jsx-a11y` en lint, contrôle automatique des écrans clés en CI (`TESTING_QA.md`), passage manuel clavier et lecteur d'écran sur les cinq parcours.

### 5.4 Performance

Budgets de `UI_UX_DESIGN.md` : premier chargement ≤ 200 Ko compressés, JavaScript ≤ 150 Ko, CSS ≤ 30 Ko, images d'une page ≤ 100 Ko (logo ≤ 30 Ko), une feuille d'icônes SVG ≤ 15 Ko, **zéro** police web, LCP ≤ 3 s en « 4G lente » avec processeur ralenti 4×, 25 lignes par page de tableau, pas de défilement infini. Contrôle en CI (`size-limit`), mesure du LCP sur l'essai. Si 150 Ko sont dépassés malgré le découpage, Preact avec sa couche de compatibilité remplace React sans réécrire les écrans (`ARCHITECTURE.md` §3.5).

## 6. Les règles qui accélèrent

Pour que les règles critiques ne ralentissent pas la livraison :

- **Tout ce qui est mécanique est automatique** : formatage, lints, génération du client, vérification des requêtes. On ne discute jamais de style en revue.
- **Petits changements** : une PR = une règle ou une intention. Hors chemins sensibles (section 8.3), une PR de moins de 150 lignes qui passe la CI peut être fusionnée après une relecture rapide ; la cible générale est de 400 lignes (8.2).
- **Interrupteurs de fonctionnalités** (`ARCHITECTURE.md` §8.2) : une fonction se fusionne « éteinte », s'allume pour une agence, puis pour toutes. Au plus 10 interrupteurs actifs, chacun avec une date de retrait.
- **Migrations rétrocompatibles** : le retour arrière est un simple redéploiement de l'image précédente.
- **Tests de règles écrits d'abord** à partir des cas `E-xx` (section 9) : le temps passé à les écrire est rendu en confiance.

## 7. Interdits et règles critiques

Pour chaque règle : pourquoi, et **comment elle est vérifiée automatiquement**. Une règle sans vérification automatique n'est pas critique : elle est à compléter avant le pilote.

| # | Règle | Pourquoi | Vérification automatique | Statut |
|---|---|---|---|---|
| 1 | **Aucun nombre à virgule pour de l'argent** : pas de `f32`, `f64`, `NUMERIC`, `DECIMAL`, `number` décimal, `parseFloat`, `toFixed` | Un flottant ne représente pas exactement 0,1 ; une erreur d'un franc dans une quittance se constate devant le locataire (R-001, DP-45) | `clippy::float_arithmetic` en `deny` dans `domain` et modules d'argent ; recherche de `f32\|f64` dans les modules d'argent ; test de schéma : aucune colonne de montant de type `numeric`, `real` ou `double precision` ; front : recherche de `parseFloat\|toFixed\|Math.round` hors `format/` ; tests de propriétés : toute valeur de l'API est un entier JSON | [PILOTE] |
| 2 | **Aucune modification ni suppression d'un paiement** (ni d'une affectation, quittance, reversement, relance, version de compte rendu, ligne d'audit) | La confiance dans le livre vient de l'immutabilité ; une correction est une contre-passation liée et tracée (R-047, R-048, DP-47) | Recherche en CI de `UPDATE`/`DELETE` visant les 25 tables d'écriture dans `crates/` (`DATABASE_SCHEMA.md` §4.2) ; tests de base : `UPDATE`, `DELETE`, `TRUNCATE` refusés pour `immo_app` **et** pour le propriétaire du schéma (G-05, ISO-4) ; contrôle nocturne des déclencheurs actifs ; test d'API : `PUT/PATCH/DELETE` renvoient 405 `payment.immutable` | [PILOTE] |
| 3 | **Aucun accès à la base hors du contexte d'organisation** | Une requête sans contexte est une fuite entre agences, le risque n° 1 d'un produit multi-clients (dossier §12, §24, DP-48) | Lints `disallowed-types` sur `PgPool`/`PgConnection` et `disallowed-methods` sur `sqlx::query*` non vérifiées hors `platform` ; test d'étanchéité par route (404 + audit) et par table (`org_id`, RLS forcée, politique) ; test sans contexte (zéro ligne, insertion refusée) ; test de concurrence sur le pool (50 tâches, 2 connexions) ; rôle applicatif sans `BYPASSRLS` | [PILOTE] |
| 4 | **Jamais de secret dans le dépôt** | Un secret versionné est à considérer comme public pour toujours | Détection de secrets sur l'historique complet, en CI et en pré-validation ; `.env` dans `.gitignore` ; recherche de motifs `postgres://…:…@` | [PILOTE] |
| 5 | **Jamais de donnée réelle dans les tests, les exemples, les captures ni les conversations** | Téléphones et noms de locataires sont des données personnelles (loi n° 001-2021/AN à vérifier, décision n° 15) | Test de CI : aucun numéro hors du bloc fictif `+226 00 00 …` dans `tests/`, `seeds/`, `maquette/` ; adresses électroniques limitées à `example.test` ; fixtures modifiées = deuxième relecteur obligatoire ; données d'essai issues de `immo anonymize` avec contrôle | [PILOTE] |
| 6 | **Jamais de contournement de la numérotation des quittances** : pas de séquence PostgreSQL, pas d'`INSERT INTO receipts` hors du module `receipts`, pas de numéro choisi par l'appelant | Une quittance sans numéro ou avec un trou est une preuve contestable (R-050, DP-46) | Recherche de `CREATE SEQUENCE`/`nextval` dans les migrations et le code ; recherche de `INTO receipts` hors du module ; déclencheur de la base (le numéro doit égaler le compteur et suivre le précédent) ; prototype (c) rejoué en CI avec annulations forcées ; requête nocturne de recherche de trous | [PILOTE] |
| 7 | **Aucune lecture directe de l'heure** (`now()`, `CURRENT_DATE`, `Utc::now`) | « Aujourd'hui » détermine retard, antidatage et date future ; un paramètre de session modifiable permettrait d'antidater un paiement (DP-50) | Lints `disallowed-methods` ; recherche de `CURRENT_DATE`/`now()` dans les migrations et requêtes hors `app_now()` ; test de parité `Clock` / `app_now()` | [PILOTE] |
| 8 | **Aucune règle d'argent dans le front ni dupliquée** (statut d'échéance, ancienneté, commission, montant en lettres, affectation) | Deux implémentations divergent ; le dossier veut des totaux qui concordent par construction (`ARCHITECTURE.md` §4.9) | Recherche de ces calculs hors `domain` ; test de concordance des totaux (tableau de bord = somme des listes, contrôle n° 20 de la maquette) ; front : recherche d'opérateurs sur des montants | [PILOTE] |
| 9 | **Aucune action sensible sans ligne d'audit dans la même transaction** | Pas d'action sans trace (R-008) | Test par action sensible : l'action échoue si l'écriture d'audit échoue ; test qui compare la liste des codes d'audit de `SECURITY.md` §6.1 aux actions codées | [PILOTE] |
| 10 | **Aucune table métier sans `org_id`, RLS forcée et politique** ; **aucune migration appliquée modifiée** | Une table oubliée est un trou d'isolation ; une migration réécrite divergerait entre environnements | Test de schéma (`pg_class`) ; CI qui compare la somme de contrôle des migrations déjà fusionnées | [PILOTE] |
| 11 | **Aucun droit seulement masqué dans l'interface** | Le serveur est la seule barrière fiable (F02) | Test par couple rôle × action généré depuis la matrice ; test d'agent terrain avec lots assignés et non assignés | [PILOTE] |
| 12 | **Aucune insertion de HTML brut, aucune requête sortante construite depuis une saisie, aucune concaténation SQL** | Scripts intersites, SSRF, injection (`SECURITY.md` §7.2) | Lint `no-restricted-syntax` ; recherche de `format!` dans les requêtes ; rendu PDF sans réseau (test) | [PILOTE] |
| 13 | **Aucune donnée personnelle dans les journaux, les événements d'usage ni les signalements** | Minimisation (`SECURITY.md` §7.7, §8.1) | Test de CI qui cherche numéros et jetons dans les journaux d'un parcours complet ; liste blanche des événements et propriétés | [PILOTE] |
| 14 | **Aucun interrupteur de fonctionnalité ne change une règle d'argent pour une seule organisation** | Règle d'or 12 : règles d'argent jamais sur mesure | Revue ; recherche d'`org_features` dans `domain` | [PILOTE] |
| 15 | **Aucun `unsafe`, aucun `unwrap` en production** | Un arrêt brutal en pleine transaction est inacceptable | Lints en `deny` / `forbid` | [PILOTE] |

## 8. Processus

### 8.1 Branches et commits (décision n° 73)

- **Contexte** : un développeur (DP-54), livraison plusieurs fois par semaine, retour arrière par redéploiement de l'image précédente.
- **Options** : (a) développement sur tronc commun avec branches courtes et PR (« trunk-based ») ; (b) branches longues par fonctionnalité (« GitFlow »).
- **Décision** : (a) (DP-56, décision n° 73). Les interrupteurs de fonctionnalités permettent de fusionner du code inachevé éteint ; (b) crée des fusions pénibles pour une petite équipe.
- **Règles** :
  - `main` est protégée : fusion uniquement par PR, CI verte obligatoire, relecture obligatoire (8.3). Fusion par **squash** : un commit par PR sur `main`.
  - Branches courtes (une journée à trois jours) : `feat/F09-idempotence`, `fix/R-041-ref-doublon`, `refactor/…`, `docs/…`, `chore/…`, `spike/…` (prototype jetable, jamais fusionné dans le code produit).
  - **Message de commit** (français, mode impératif) : `type(portée): sujet [F09][R-042][DP-49]`, sujet de 72 caractères au plus, corps facultatif qui dit **pourquoi**. Types : `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `chore`, `sec`. L'identifiant `DP-xx` ou `F-xx` est obligatoire dès que le changement applique une décision ou une fonctionnalité (`CLAUDE.md`). Exemple : `feat(ledger): refuser un paiement daté du futur [F09][R-039]`.
  - Aucune fusion de code qui échoue à un test « parce que c'est urgent » : un correctif prioritaire passe par la même CI (qui est rapide).
  - Aucune force-push sur `main` ; aucun commit direct. Un secret commis est révoqué, pas seulement retiré de l'historique.

### 8.2 Taille des changements

- Cible : **moins de 400 lignes modifiées** hors fichiers générés ; au-delà de 800, la PR est découpée. Un changement de schéma, son code et l'écran correspondant restent dans la même PR (monorepo), mais une migration lourde est seule.
- Une PR qui touche `domain`, `ledger`, `receipts` ou `migrations/` reste **petite** : on ne mélange pas refactoring et changement de règle.

### 8.3 Revue de code (décision n° 74, DP-54)

- **Règle** (DP-54) : un seul développeur écrit le code. Toute PR est relue par son auteur après un **délai de 24 heures**, avec la liste de contrôle ci-dessous, la CI verte et la mention « auto-relu » dans la PR. Chemins **sensibles** (relecture complète et liste de contrôle obligatoires) : `crates/domain/`, modules `ledger`, `receipts`, `identity`, `statements`, `imports`, `platform` (fichiers, PDF, sessions), `migrations/`, `ops/`, `clippy.toml`, `deny.toml`, configuration de CI. Un **second avis externe** (personne à désigner) est demandé pour ces chemins et exigé pour une migration qui touche une garde, avant la mise en production suivante ; il est tracé dans la PR.
- **Échéance de relecture** : première réponse en une demi-journée ouvrée, pour que la revue ne devienne pas le goulot.
- Le relecteur lance la PR sur l'essai et vérifie un parcours à la main quand l'interface change.

**Liste de contrôle de revue** (à copier dans le modèle de PR) :

- [ ] La PR cite les identifiants `F-xx`, `R-xxx`, `DP-xx` concernés ; aucune règle métier inventée (sinon : question ouverte ajoutée à `DOMAINE.md`)
- [ ] Argent : entiers, type `Fcfa`, aucun flottant, aucune règle dupliquée dans le front
- [ ] Isolation : `OrgTx` partout, `org_id` filtré, test d'isolation de chaque nouvelle route, aucune table sans `org_id` ni RLS
- [ ] Immutabilité : aucun `UPDATE`/`DELETE` sur une table d'écriture ; contre-passation pour corriger
- [ ] Numérotation : aucun contournement du compteur
- [ ] Droits : l'action est déclarée dans `authorize` ; test rôle × action ; rien de seulement masqué dans l'interface
- [ ] Audit : action sensible tracée dans la même transaction
- [ ] Idempotence : clé obligatoire sur toute nouvelle écriture d'argent
- [ ] SQL : requêtes vérifiées, paramétrées, colonnes nommées, délais, aucune lecture directe de l'heure
- [ ] Erreurs : pas de `unwrap`, code d'erreur stable et texte du front ajouté
- [ ] Journaux : aucune donnée personnelle ni jeton
- [ ] Migration : rétrocompatible, `lock_timeout`, testée sur base vide et sur copie ; l'ancienne version du binaire fonctionne avec le nouveau schéma
- [ ] Contrat : différence `openapi.json` relue ; ajout seulement ou dépréciation datée
- [ ] Front : textes centralisés, tokens, quatre états, accessibilité, poids ; aucun HTML brut
- [ ] Tests : cas `E-xx` ajoutés ou mis à jour, test de non-régression pour un bogue corrigé
- [ ] Documentation à jour (8.6) ; décision structurante enregistrée **avant** le code

### 8.4 Définition de « terminé »

Une tâche est terminée quand **tout** ce qui suit est vrai :

1. Le code est fusionné sur `main` après revue (8.3), CI verte.
2. Les règles concernées sont couvertes par des tests citant leurs `R-xxx` / `E-xx` ; les tests de règles financières ont été écrits **avant** le code.
3. Chaque nouvelle route a son test d'isolation, son test de droits et son test de contrat.
4. L'action sensible est auditée ; l'événement d'usage de l'écran existe (`ARCHITECTURE.md` §8.4).
5. Les textes sont en français dans `texts/fr.ts` ; l'écran gère chargement, vide, erreur, succès ; les contrôles d'accessibilité automatiques passent ; les budgets de poids sont respectés.
6. La migration éventuelle est rétrocompatible et exercée sur l'essai ; le retour arrière reste possible.
7. La fonction est derrière un interrupteur si elle est livrée pendant le pilote.
8. Déployée sur l'essai, parcours vérifié à la main sur un téléphone d'entrée de gamme quand l'interface change.
9. Les documents concernés sont à jour (8.6).

### 8.5 Décisions d'architecture à consigner

- Le projet n'a **pas** de dossier d'ADR séparé : le journal de décisions est `DECISIONS.md` (entrées `DP-xx`). Une **décision structurante** — périmètre, règle d'argent, modèle de données, pile technique, prix, légal, hébergement — s'écrit dans `DECISIONS.md` **avant** d'être appliquée dans le code (`CLAUDE.md`). Le format est le modèle d'entrée de ce fichier : contexte, décision, alternatives écartées, conséquences, statut, liens.
- Un prototype jetable (`spikes/`) se termine par une note d'une page versée sous forme d'entrée `DP-xx` ; dépasser la durée prévue est en soi un résultat à noter (`ARCHITECTURE.md` §7).
- Une recommandation d'un document n'est **pas** une décision tant qu'elle n'a pas d'entrée `DP-xx`. En cas de conflit entre documents, on signale le conflit ; on ne tranche pas seul.
- Tout `// TODO` porte un numéro de décision ou de question ouverte ; il est interdit en production pour une règle d'argent.

### 8.6 Documentation à jour

| Si le changement touche… | Mettre à jour |
|---|---|
| une route, un champ, un code d'erreur | `API_CONTRACTS.md` (catalogue, journal des changements), `openapi.json`, textes du front |
| une table, une contrainte, une garde | `DATABASE_SCHEMA.md` et la migration |
| une règle métier, un statut, un exemple chiffré | `DOMAINE.md` (règle `R-xxx`, exemple `E-xx`) **après** décision enregistrée |
| un choix de pile, d'hébergement, de sécurité | `DECISIONS.md`, puis `ARCHITECTURE.md` ou `SECURITY.md` |
| une commande, une variable d'environnement, la structure du dépôt | `CLAUDE.md` (sous 100 lignes), `DEPLOYMENT.md` |
| un écran ou un texte | `UI_UX_DESIGN.md` (microcopy) |

La documentation périmée est un défaut de revue au même titre qu'un test manquant.

## 9. Travailler avec Claude Code dans ce dépôt

Ces règles s'adressent à Claude Code et à toute personne qui lui confie une tâche. Elles complètent `CLAUDE.md` (qui reste sous 100 lignes et pointe ici).

1. **Lire avant de toucher.** `CLAUDE.md` en début de tâche ; `DOMAINE.md` **avant** de toucher à un calcul, un statut, une règle d'argent ; `API_CONTRACTS.md` avant une route ; `DATABASE_SCHEMA.md` avant une migration ou une requête ; `SECURITY.md` avant l'authentification, les droits ou les données sensibles.
2. **Tests des règles financières d'abord.** Pour toute règle d'argent : écrire d'abord le test qui exprime le cas `E-xx` (montants exacts de `DOMAINE.md`), le voir échouer, puis écrire le code. Les trois cas d'ancrage doivent toujours passer : loyer de 75 000 avec paiement partiel de 50 000 (E-05) ; paiement de 100 000 avec avance de 25 000 (E-06) ; compte rendu à 400 000 encaissés, commission de 8 %, dépenses de 30 000, net de 338 000 (E-18).
3. **Petits changements.** Une règle ou une intention par PR (moins de 400 lignes). Pas de refactoring non demandé mélangé à un changement de comportement.
4. **Ne jamais inventer une règle métier.** Si `DOMAINE.md` ne tranche pas (exemples : jour d'échéance 31 en mois court, doublon de référence bloquant ou non, définition d'une promesse « tenue »), **s'arrêter**, poser la question à la personne qui pilote le projet, ou ajouter la question à la section « Questions ouvertes » du document concerné. Ne pas coder un comportement « raisonnable » par défaut sur de l'argent.
5. **Mettre à jour `DECISIONS.md` avant d'appliquer** une décision structurante, et citer `R-xxx`, `E-xx`, `DP-xx`, `F-xx` dans le code, les tests et les messages de commit.
6. **Ne jamais affaiblir une garde pour faire passer un test** : désactiver un lint, un test d'isolation, une contrainte ou un déclencheur, ou ajouter un `#[allow]`, demande l'accord explicite de la personne qui pilote le projet. Un test qui échoue révèle un défaut : on le corrige.
7. **Ne pas modifier une migration appliquée** ; en écrire une nouvelle. Ne jamais lancer de commande destructive sur une base qui n'est pas une base de test locale. Ne jamais utiliser le rôle propriétaire du schéma depuis l'application.
8. **Aucun secret, aucune donnée réelle** dans les prompts, les fichiers, les exemples ni les captures. Les exemples utilisent le bloc fictif `+226 00 00 …` et des noms de la maquette.
9. **Ne pas écraser un fichier existant sans demander** ; ne pas ajouter de dépendance, de fonction hors périmètre de `CONTEXTE.md`, ni de `unsafe` sans accord. Pas d'IA au MVP (règle d'or 11).
10. **Vérifier avant d'annoncer.** Exécuter `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo sqlx prepare --check`, `npm run lint` et `npm test` (commandes de `CLAUDE.md`) et rapporter le résultat réel ; ne dire « terminé » que si la définition de 8.4 est remplie. Si un test échoue, le dire avec la sortie.
11. **Ne valider (commit) que si on le demande** ; ne jamais pousser ni ouvrir de PR sans demande.
12. **Garder `CLAUDE.md` à jour** (statuts, commandes) et sous 100 lignes ; y mettre un pointeur, jamais le contenu d'un autre document.

Modèle de demande à Claude Code pour une règle d'argent :

```text
Implémente R-0xx (DP-xx, F-xx) dans crates/domain.
1. Lis DOMAINE.md (règle R-0xx, exemples E-xx) et les questions ouvertes liées.
2. Écris d'abord les tests des cas E-xx et un test de propriété ; montre-les en échec.
3. Implémente la fonction pure (aucune E/S, entiers seulement).
4. Ne code aucun comportement que DOMAINE.md ne définit pas : liste-les en questions ouvertes.
5. Lance cargo fmt --check, clippy -D warnings, cargo test -p domain, et rapporte les résultats.
```

## 10. Indispensable avant le pilote et ce qui peut venir pendant les essais

| Indispensable avant le premier pilote | Peut venir pendant les essais |
|---|---|
| Règles 1 à 15 de la section 7 vérifiées par la CI (sauf outils notés) | Test automatique des `use` entre modules (graphe sans cycle) |
| `rustfmt`, `clippy -D warnings`, `cargo sqlx prepare --check`, `cargo audit`, `cargo deny`, `npm audit`, détection de secrets | Mesure de couverture de tests, seuil de couverture |
| `unsafe` interdit, `unwrap` interdit, type `Fcfa`, `RateBp`, `Clock` | Lint maison (au lieu de lints génériques) pour l'accès à la base |
| `OrgTx` obligatoire et lints `disallowed-*` sur `PgPool` | Test de non-régression du temps de compilation |
| Front en mode strict, textes centralisés, tokens, client généré, budgets de poids en CI | Analyse dynamique des pages, tests visuels étendus |
| Revue obligatoire par la seconde personne, liste de contrôle, définition de « terminé » | Modèle de PR et étiquettes affinés, automatisation des mises à jour de dépendances |
| `DECISIONS.md` à jour avant le code ; `CLAUDE.md` à jour | Signature des images et des commits |

## Questions ouvertes

1. **Capacité (décision n° 14, H11)** : *tranchée par DP-54 : un développeur, environ 25 heures par semaine. La règle de revue de 8.3 en tient compte.*
2. **Langue des identifiants (n° 18)** : *tranchée par DP-67 : anglais.*
3. **Hébergeur du code et de la CI** (question 13 d'`ARCHITECTURE.md`) : non défini. Les règles de protection de branche, de revue obligatoire et de CI bloquante en dépendent (voir `DEPLOYMENT.md`).
4. **Dossiers marketing et `.venv` dans la racine actuelle** : sortir du dépôt de code, ou créer le dépôt dans un sous-dossier propre ?
5. **Lints sur les types primitifs** : `clippy::disallowed_types` ne cible peut-être pas les types primitifs `f32` et `f64` ; à vérifier au premier essai, sinon `float_arithmetic` et la recherche en CI suffisent.
6. **Outil de comparaison OpenAPI, bibliothèque de génération du contrat, générateur du client TypeScript** : à choisir au prototype de la tranche verticale (décision n° 61).
7. **Bibliothèque de dates (n° 70)** : prise en charge de `jiff` par la version de SQLx retenue.
8. **Plafond du nombre de `#[allow]`** et seuil d'alerte du temps de compilation : à fixer à la première mesure.
9. **Second avis externe sur les chemins sensibles et les migrations de garde** : qui est la personne à désigner (DP-54) ? Ouverte : à fixer par Mimis avant la tranche verticale.
10. **Modèle de PR et configuration des protections de branche** : dépendent de l'hébergeur du code.

## Décisions à enregistrer dans DECISIONS.md

Mise à jour du 30/10/2026 : les décisions n° 69 à 77 sont prises par DP-56. Aucune autre décision n'est présentée comme **prise** dans ce fichier. Les règles critiques de la section 7 sont des conséquences directes de règles déjà décidées (DP-45, DP-46, DP-47, DP-48, DP-49, DP-50, DP-51) ; leur outillage de vérification reste une proposition. Décisions **ouvertes** [À valider], à ajouter à la suite de la n° 68 :

| N° | Décision | Recommandation de ce document |
|---|---|---|
| 69 | Gestion des erreurs Rust : types d'erreur propres (`thiserror`) dans les bibliothèques, `anyhow` seulement dans le binaire et la CLI ; `unwrap`, `expect`, `panic` interdits hors tests ; `unsafe` interdit | Oui (4.1, 4.7) |
| 70 | Bibliothèque de dates : `chrono` + `chrono-tz` ou `jiff` | `chrono` + `chrono-tz` (compatibilité SQLx), à vérifier (4.3) |
| 71 | Données du front : client généré et enveloppe fine, sans bibliothèque de cache au pilote | Oui (5.1) |
| 72 | Outils du front : ESLint (`typescript-eslint`, `jsx-a11y`) + Prettier + `size-limit` | Oui (3.2) ; Biome écarté pour ses règles d'accessibilité moins complètes [Hypothèse] |
| 73 | Branches et commits : tronc commun, branches courtes, `main` protégée, squash, commits français avec identifiants `F`/`R`/`DP` | Oui (8.1) |
| 74 | Revue : par la personne qui n'a pas écrit, chemins sensibles et liste de contrôle, PR de moins de 400 lignes | Oui (8.2, 8.3) |
| 75 | Outillage de la règle « jamais `&PgPool` hors `platform` » : lints `disallowed-types` et `disallowed-methods` dès le pilote | Oui (3.1) ; avance l'outil de [APRÈS] à [PILOTE] (`ARCHITECTURE.md` §5.3) |
| 76 | Détection de secrets : un outil dédié en CI sur tout l'historique et en pré-validation locale | Un outil à binaire unique et sans service (choix du produit à faire au prototype) |
| 77 | Politique de licences de `cargo deny` : liste blanche (licences permissives courantes), copyleft fort refusé | Oui (4.6), liste exacte à établir |
