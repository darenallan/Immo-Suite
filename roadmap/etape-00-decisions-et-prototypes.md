# Étape 00 — Décisions techniques et prototypes

| Champ | Valeur |
|---|---|
| Statut | À faire |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | Avant T1 (fondations) |
| Effort | L, environ 13 jours-développeur (JD), dont 10 de prototypes (`ARCHITECTURE.md` §7) |
| Dépend de | Aucune étape. Contenus C-01 et C-02 (comptes d'organisation, budget et plan Neon) |
| Jalon | Contribue à J0 (fin de l'étape 01) |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis seulement les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Prouver, sur le PostgreSQL réel (Neon) et avec de vrais outils, les quatre points qui peuvent casser le pilote : isolation entre organisations avec pool de connexions, numérotation des quittances sans trou sous concurrence, rendu PDF de la quittance, import d'un Excel imparfait. Enregistrer les décisions bloquantes (`TESTING_QA.md` PP-32) avant d'écrire du code produit.
Valeur : une agence ne verra jamais un trou de numéro, une fuite entre agences ou un PDF illisible, parce que ces risques sont levés avant la première migration.

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections à lire (et seulement celles-là) |
|---|---|---|
| Prototypes (a) à (e) | `ARCHITECTURE.md` §7 | §7 en entier, §6 (PDF), §5.3 et §5.4 (OrgTx, tests d'étanchéité), §4.4 (numérotation), §4.5 (idempotence) |
| Neon, rôles, sauvegarde logique | DP-55, décisions n° 21 et 46 | `DATABASE_SCHEMA.md` §6.9 et §9.1 ; `DEPLOYMENT.md` §1.2 (grille éliminatoire) |
| Tests attendus | `TESTING_QA.md` | §2.3 (concurrence), §3 (isolation, numérotation), §6 (PDF), §7 (import) |
| Règles | R-004, R-050, R-053, R-056, R-080 ; E-10, E-11, E-20, E-21, E-27 | `DOMAINE.md` (règles et cas cités) |
| Décisions de départ | DP-45 à DP-49, DP-55, DP-56, DP-62 | `DECISIONS.md` |

Décisions à enregistrer à la fin de l'étape (une entrée `DP-xx` chacune, format de `DECISIONS.md`) :

| Sujet | N° | Statut attendu |
|---|---|---|
| Moteur PDF | 7 (puis 83) | Prise au prototype (a) |
| Sauvegarde logique sous RLS forcée (rôle `immo_backup`) | 46 (DP-55 conditionnelle) | Confirmée ou repli « RLS non forcée » |
| Plan Neon, région, historique de restauration | 5, 86 (partie base) | À trancher avec C-02 |
| Capacité réelle (deux développeurs ?) | 14, 96 | À trancher par Mimis avant de commencer |
| Pilote en deux temps | 97 | À trancher par Mimis (voir `ROADMAP.md` §3) |
| Dépôt à la racine ou dans un sous-dossier ; dossiers marketing exclus | `CODING_RULES.md` question 4 | À trancher |
| Observation des usages, erreurs du front, outils du front, outils de test | 28, 66, 71, 72, 78, 82 | Proposées par les documents, à confirmer en bloc |

## 3. Prérequis

- Contenus C-01 (comptes d'organisation, double authentification, gestionnaire de mots de passe) et C-02 (budget, plan Neon, hébergeur du serveur, du stockage objet et du second fournisseur).
- Poste : Rust stable, Docker, `sqlx-cli`, Node LTS (`CLAUDE.md`). Une imprimante laser noir et blanc pour le prototype (a).
- Décision de Mimis sur la capacité (deux développeurs ou DP-54) : elle change la règle de revue (`CODING_RULES.md` §8.3).

## 4. Fiches et notes à produire avant de coder

Pas de fiche `specs/F-xx` (aucune fonctionnalité). Une note d'une page par prototype, dans `spikes/<nom>/NOTE.md`, versée ensuite dans `DECISIONS.md` (`ARCHITECTURE.md` §7). Plan de chaque note : objectif, protocole, résultats chiffrés, critères atteints ou non, décision proposée, écarts constatés avec les documents. **S'arrêter pour validation de Mimis après la rédaction des cinq protocoles** (avant d'exécuter).

## 5. Tâches détaillées

### 5.1 Base de données (prototypes b et c, projet Neon séparé « immo-spikes »)

1. Créer un projet Neon dédié aux prototypes (jamais celui de l'essai ni de la production), version 16 ou plus, mise en veille désactivée, région candidate. Vérifier `CREATE EXTENSION btree_gist`.
2. Créer en SQL, sans passer par la console, les rôles `immo_owner`, `immo_app`, `immo_platform`, `immo_backup`. Consigner `SELECT rolname, rolbypassrls, rolsuper FROM pg_roles`. Constater (et noter) que les rôles créés par la console Neon ont `BYPASSRLS`.
3. Schéma minimal : `organizations`, `units` (ou table jouet) avec `org_id`, `UNIQUE (org_id, id)`, RLS activée et forcée, `current_org_id()` lisant `current_setting('app.org_id', true)`, politique `org_isolation`.
4. Prototype (c) : `receipt_counters (org_id, series, year, last_number)`, `payments` et `receipts` réduits, `INSERT … ON CONFLICT DO UPDATE … RETURNING` (`ARCHITECTURE.md` §4.4), contrainte `UNIQUE (org_id, series, year, sequence)`, verrou consultatif par bail, table `idempotency_keys` avec colonne `route`.
5. Restauration : tester `pg_dump` au format personnalisé avec `immo_backup` sous RLS forcée, puis `pg_restore` sur un autre projet Neon ; relever la fenêtre d'historique du plan et la restauration à un instant donné.

### 5.2 Noyau métier (`spikes/`, hors du code produit)

1. Prototype (b) : type `OrgTx` (`OrgTx::begin(&pool, org_id, user_id)` qui exécute `set_config('app.org_id', $1, true)`), pool de 2 connexions, 50 tâches alternant deux organisations, 10 000 itérations, zéro lecture croisée ; test négatif obligatoire : la variante `SET app.org_id` de session **doit** fuir ; mesure du surcoût par requête ; coût de `after_release` / `before_acquire`.
2. Prototype (c) : 20 clients × 500 validations concurrentes sur 3 organisations, 10 % d'annulations forcées après l'attribution du numéro ; vérifier zéro trou, zéro doublon, numéros consécutifs dans l'ordre de validation ; passage d'année (E-27) avec une `Clock` injectée ; même clé d'idempotence envoyée 3 fois en parallèle : un seul paiement ; p95 d'une validation.
3. Prototype (a) : fonction pure `amount_in_words` (R-053, DP-64 : orthographe traditionnelle, « francs CFA ») testée sur 0 à 2 000 et sur les frontières de `TESTING_QA.md` §6, plus 999 999 999 ; comparaison avec un analyseur de référence indépendant.
4. Prototype (d) : lecteur CSV/XLSX, correspondance de colonnes, validation, rapport, annulation, sur les fichiers de 5.7.

### 5.3 API
Un serveur HTTP minimal (Axum) par prototype (c) seulement, pour jouer la concurrence par le réseau : `POST /spike/payments` avec `Idempotency-Key`. Aucune route du contrat `API_CONTRACTS.md` n'est créée.

### 5.4 Interface
Gabarit HTML de la quittance (A4 et A5, CSS d'impression repris de `maquette/`) pour les moteurs qui le demandent.

### 5.5 Documents PDF (prototype a)
Comparer les trois options de `ARCHITECTURE.md` §6 (navigateur sans interface, tracé natif Rust, composition embarquée de type Typst) sur les 20 cas types de `UI_UX_DESIGN.md` : accents (Ouédraogo, Yaméogo, ç, œ), noms longs, société, montant à 7 chiffres, maximal 999 999 999, partielle, avance, tiers payeur, colocation, TVA incluse, rectificative, annulée, ESSAI, DÉMONSTRATION, balisage dans un nom, logo absent et maximal. Mesurer sur 100 rendus : médiane, p95, mémoire au pic, taille de l'image de déploiement ; une seule page en A4 et en A5 ; polices embarquées ; aucune métadonnée d'auteur ; aucune requête sortante. Imprimer sur une laser noir et blanc. Trancher aussi : impression depuis le PDF ou depuis un aperçu HTML (`API_CONTRACTS.md` question 7).

### 5.6 Audit et événements
Aucun. Le prototype (c) écrit une ligne `audit_log` jouet dans la même transaction pour vérifier le coût.

### 5.7 Tests et fixtures
- `spikes/import/fixtures/` : fichier de 200 lignes de la maquette (`importExemple` : 3 erreurs aux lignes 14, 87 et 152, 2 paires de doublons de téléphone, bloc fictif `+226 00 00 04 NN`), variantes sales de `TESTING_QA.md` §7 (CSV `;` Windows-1252, dates en nombre de série Excel, cellules fusionnées, téléphones avec ou sans +226, montants `75 000`, `75.000`, `75000 F`), fichiers piégés (archive à fort ratio, entité XML externe, `.xlsm`, formules).
- Prototype (e), **préparation seulement** : protocole de mesure d'aller-retour depuis Ouagadougou vers 2 ou 3 régions (3G et 4G de deux opérateurs) avec un front de démonstration de 150 Ko ; la mesure finale se fait en étape 01 quand l'essai existe. Qui mesure sur place ? [À valider].
- Prototype (d) : 2 000 lignes en moins de 5 secondes ; analyse à mémoire bornée ; aucune ligne créée avant confirmation (comptage).
- Initialiser le dépôt Git (`git init`) selon la décision de racine, avec `.gitignore` excluant `film/`, `flyer/`, `infographie/`, `site/`, `logo/` (copier le logo nécessaire en `templates/assets/`), tout `.venv`, `node_modules/`, `target/`, `maquette/_captures/`.

## 6. Critères d'acceptation vérifiables

1. `immo_app` : `rolbypassrls = false`, `rolsuper = false`, propriétaire d'aucune table (requête de contrôle jointe à la note).
2. 0 lecture croisée sur 10 000 itérations avec un pool de 2 connexions ; le test de la variante `SET` de session échoue comme prévu.
3. Sans contexte, `SELECT` renvoie 0 ligne sur toute table jouet et `INSERT` est refusé.
4. Numérotation : 0 trou, 0 doublon, numéros consécutifs sur au moins 10 000 validations (20 × 500) avec 10 % d'annulations ; E-10 et E-27 reproduits.
5. Idempotence : la même clé envoyée 3 fois en parallèle crée 1 paiement et rend 3 réponses identiques.
6. `pg_dump`/`pg_restore` sous RLS forcée avec `immo_backup` restituent les mêmes comptes de lignes par organisation, **ou** la note recommande « RLS non forcée » (n° 46) avec ses conséquences.
7. PDF : les 20 cas tiennent sur une page en A4 et en A5 ; p95 inférieur à 1 s ; extraction de texte égale à l'instantané ; impression papier lisible en noir et blanc ; moteur proposé pour la décision n° 7.
8. Import : sur le fichier de référence, rapport exact (197 créables, 3 erreurs aux lignes 14, 87 et 152, 2 doublons proposés à la fusion) ; après annulation, aucune trace (E-21) ; fichiers piégés refusés.
9. Chaque note contient les chiffres mesurés et une décision proposée ; chaque dépassement de durée prévue est noté comme résultat.

## 7. Sécurité et déploiement à vérifier

- Aucune donnée réelle, aucun secret dans le dépôt (chaînes de connexion en variables locales) ; détection de secrets installée dès le premier commit si l'outil est choisi.
- Projet Neon des prototypes détruit ou archivé à la fin ; clés et mots de passe de test jamais réutilisés ailleurs.
- Les prototypes vivent dans `spikes/` : hors build, jamais importés par le code produit (`CODING_RULES.md` §1.1).

## 8. Liste de démonstration (Mimis)

1. Lancer la commande de test du prototype (b) : voir « 0 fuite sur 10 000 » puis le test de la variante `SET` qui **échoue** (c'est la preuve du piège).
2. Lancer le prototype (c) : voir le tableau « 10 000 validations, 0 trou, 0 doublon, p95 = … ms ».
3. Ouvrir les PDF des 20 cas en A4 et A5 ; tenir en main l'impression laser d'une quittance ; lire le tableau de comparaison des moteurs.
4. Lancer l'import sur le fichier de 200 lignes : lire « 197 créables, 3 erreurs (lignes 14, 87, 152) » ; annuler ; vérifier « 0 ligne restante ».
5. Lire les notes et valider (ou refuser) chaque décision proposée.

## 9. Définition de terminé, risques, hors étape

Terminé : les cinq notes sont validées par Mimis, les entrées `DP-xx` correspondantes sont écrites et le tableau des décisions ouvertes est mis à jour ; le dépôt Git existe et `spikes/` est exclu du build ; les 44 tests du schéma (`DATABASE_SCHEMA.md` §9.4) sont inventoriés pour être rejoués en étape 01 ; `ROADMAP.md` est recalé avec la vitesse constatée.

| Risque | Mesure |
|---|---|
| Neon impose `BYPASSRLS` ou refuse `pg_dump` sous RLS forcée | Repli n° 46 : RLS activée mais non forcée, avec test de propriété supplémentaire ; décision écrite avant l'étape 01 |
| Aucun moteur PDF ne tient les 20 cas | Prototype étendu d'1 jour maximum ; sinon aperçu HTML imprimable en repli, PDF en tâche de fond |
| Fenêtre d'historique du plan Neon inférieure à 30 jours | Compenser par les 30 copies logiques quotidiennes (`ARCHITECTURE.md` §10.4) et noter l'écart |
| Prototype qui s'éternise | Durées maximales : b 2 j, c 1 j, a 3 j, d 3 j ; dépasser est un résultat à noter |

Hors étape : toute migration du schéma de production, toute route du contrat, toute interface réelle, l'intégration continue (étape 01).

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 00
- Dépend de : aucune
- Tranche : Fondations
- Jalon : J0 (avec 01)
- F : F39
- R : (prototypes seulement : R-004, R-050, R-053, R-080 sont implémentées aux étapes 02, 07, 12)
- E : (rejouées en spike : E-10, E-11, E-20, E-21, E-27)
- PP : PP-32
- Contenus : C-01, C-02, C-19
