# Prototype (b) — Isolation entre organisations avec pool de connexions

| Champ | Valeur |
|---|---|
| Statut | **Exécuté le 31/10/2026 (protocole validé par Mimis).** Résultats en section 8 ; décision proposée en section 6, **en attente de validation de Mimis** |
| Durée maximale | 2 jours (dépasser est un résultat à noter) |
| Décision attendue | Mécanisme d'isolation définitif ; modèle de rôles ; compatibilité de Neon (n° 46, DP-55 conditionnelle) |
| Documents | `ARCHITECTURE.md` §5.3, §5.4, §7 ; `DATABASE_SCHEMA.md` §6.9, §9.1 ; `TESTING_QA.md` §3 ; DP-55, DP-69, DP-70 |

## 1. Objectif

Prouver sur le PostgreSQL réel (Neon) qu'une connexion de pool réutilisée par deux organisations ne fuit jamais de données de l'une vers l'autre quand `OrgTx` pose le contexte avec `set_config('app.org_id', …, true)`, que la RLS forcée tient avec des rôles créés en SQL sans `BYPASSRLS`, que `pg_dump` fonctionne sous RLS forcée, et mesurer le surcoût.

## 2. Environnement

- Cible principale : projet Neon `immo-spikes` (`aws-eu-central-1`, Postgres 18.6, plan gratuit, DP-69), une branche `spike-b` créée pour l'occasion. Cible de comparaison : PostgreSQL local 18.6 sur `127.0.0.1:5433`.
- Rust 1.99.1 (toolchain GNU), crate `spikes/isolation-pool/` avec son propre `Cargo.toml`, hors build du produit (`CODING_RULES.md` §1.1). Connexion **directe** à Neon (pas de pooler, DP-55).
- Les mots de passe de rôles sont générés localement et rangés dans `%LOCALAPPDATA%\immo-spikes-neon\.env.local` : jamais dans le dépôt.
- Budget de calcul gratuit : au plus 10 CU-heures pour l'ensemble des prototypes (quota de 100 par mois) ; la consommation est relevée en fin de séance.

## 3. Protocole

1. **Rôles par SQL** (en tant que `neondb_owner`) : `immo_owner`, `immo_app`, `immo_platform`, `immo_backup`. Consigner `SELECT rolname, rolbypassrls, rolsuper FROM pg_roles`. Constater et noter que `neondb_owner` (rôle de la console) a `BYPASSRLS` (déjà constaté le 31/10/2026). Essayer `CREATE ROLE immo_backup BYPASSRLS` et noter le message exact (succès ou refus).
2. **Schéma jouet** : `organizations`, `units` (`org_id`, `id`, `UNIQUE (org_id, id)`), `memberships` ; `ENABLE` et `FORCE ROW LEVEL SECURITY` ; `current_org_id()` lisant `current_setting('app.org_id', true)` ; politique `org_isolation` (`USING` et `WITH CHECK`) ; `immo_app` ne possède aucune table.
3. **Type `OrgTx`** : `OrgTx::begin(&pool, org_id, user_id)` exécute `SELECT set_config('app.org_id', $1, true), set_config('app.user_id', $2, true)` en première instruction. Pool de **2 connexions** (`max_connections = 2`).
4. **Tests d'étanchéité** (un test par ligne, résultat consigné) :
   - T1 sans contexte : `SELECT count(*)` renvoie 0 sur chaque table jouet ; `INSERT` refusé (code `42501`).
   - T2 deux organisations : A ne voit rien de B ; un `UPDATE` de A sur des lignes de B touche 0 ligne ; un `INSERT` de A avec l'`org_id` de B est refusé ; la clé composite interdit qu'une ligne de A vise un lot de B.
   - T3 concurrence : 50 tâches alternent les organisations A et B sur le pool de 2 connexions, **10 000 itérations** au total (200 par tâche) ; chaque itération lit ses lignes et compte toute ligne dont l'`org_id` n'est pas le sien. Attendu : 0.
   - T4 **test négatif obligatoire** : variante `SET app.org_id` de session, où 10 % des itérations « oublient » de poser le contexte. Attendu : au moins une lecture croisée (le test **doit échouer**, c'est la preuve du piège). Noter le nombre de fuites. Variante `SET LOCAL` hors transaction : noter le comportement.
   - T5 recherche d'adhésions à la connexion : une fonction `find_memberships(user)` (`SECURITY DEFINER`, rôle limité) permet de retrouver les organisations d'un utilisateur avant que le contexte existe, sans ouvrir `memberships` à `immo_app` sans contexte.
5. **Surcoût par requête** : 1 000 requêtes identiques pour chaque variante : (i) requête nue sur une table sans RLS, (ii) `OrgTx` (BEGIN, `set_config`, requête, COMMIT), (iii) `OrgTx` avec `after_release` (`RESET`) et `before_acquire` (vérification). Relever p50 et p95, le nombre d'allers-retours réseau par requête et l'aller-retour de base (`SELECT 1`) du PC vers Neon, sur le PostgreSQL local et sur Neon. La latence du PC vers Neon n'est pas celle d'un serveur hébergé dans la région : la comparer, ne pas l'extrapoler.
6. **Neon, sauvegarde logique sous RLS forcée** (critère 6), sur un jeu d'environ 10 000 lignes réparties sur 3 organisations :
   - V1 `immo_backup` avec `BYPASSRLS` (si Neon le permet) ; V2 `immo_backup` sans `BYPASSRLS`, avec une politique `FOR SELECT TO immo_backup USING (true)` (option **non prévue** par la décision n° 46, à évaluer) ; V3 sans rôle dédié (constat).
   - Pour chaque variante : commande exacte, code de sortie, comptes de lignes par organisation et par table dans le fichier (`pg_dump -Fc`).
   - `pg_restore` vers un **second projet Neon gratuit** `immo-spikes-restore`, puis comparaison des comptes par organisation.
7. **Historique et restauration à un instant donné** : relever la fenêtre du plan (6 heures, DP-69). Marquer un instant T0, ajouter des lignes, créer une branche à T0 par le CLI, comparer les comptes et chronométrer. Aucune mesure sur 30 jours (objectif suspendu, DP-69).
8. `CREATE EXTENSION btree_gist` : déjà vérifié le 31/10/2026 (version 1.8).

## 4. Résultats chiffrés (à remplir)

| Mesure | Local | Neon |
|---|---|---|
| Rôles : `rolbypassrls` / `rolsuper` de `immo_app` | | |
| `CREATE ROLE … BYPASSRLS` accepté par `neondb_owner` | | |
| T1 à T3 : lectures croisées sur 10 000 itérations | | |
| T4 : fuites observées (doit être > 0) | | |
| Surcoût `OrgTx` (p50 / p95, ms) | | |
| Surcoût avec `after_release` et `before_acquire` (p50 / p95) | | |
| Aller-retour `SELECT 1` du PC vers la base (ms) | | |
| `pg_dump` V1, V2, V3 : réussi ? comptes identiques ? | | |
| `pg_restore` : comptes par organisation identiques ? | | |
| Restauration à T0 : réussie ? durée | | |
| CU-heures consommées | | |

## 5. Critères d'acceptation (étape 00 §6)

1. `immo_app` : `rolbypassrls = false`, `rolsuper = false`, propriétaire d'aucune table (requête jointe).
2. 0 lecture croisée sur 10 000 itérations avec un pool de 2 connexions ; la variante `SET` de session échoue comme prévu.
3. Sans contexte, `SELECT` renvoie 0 ligne et `INSERT` est refusé.
6. `pg_dump` et `pg_restore` sous RLS forcée restituent les mêmes comptes, **ou** la note recommande « RLS non forcée » (n° 46) avec ses conséquences.

## 6. Décision proposée

À rédiger après mesure. Deux issues possibles : confirmer DP-55 (rôle de sauvegarde avec `BYPASSRLS`, ou variante V2 si elle fonctionne mieux) ; ou replier sur la décision n° 46 (RLS activée mais non forcée, avec un test de propriété supplémentaire).

## 7. Écarts et risques déjà connus

- Neon est en plan gratuit (DP-69) : calcul 0,25 à 2 CU, mise en veille après 5 minutes ; benchmarks précédés d'un échauffement.
- Mesures prises depuis un PC sur Windows et un lien WAN : surcoût et p95 sont comparatifs, pas des valeurs de production.
- L'inventaire des 44 tests de `DATABASE_SCHEMA.md` §9.4 est fait en fin d'étape, pas ici.

## 8. Résultats (exécution du 31/10/2026)

Prototype : `spikes/isolation-pool/` (`src/main.rs`, `sql/schema.sql`, `pitr.ps1`). Journaux : `out/local-*.txt`, `out/neon-*.txt` (hors Git). Cibles : PostgreSQL local 18.6 (port 5433) et Neon (branche `spike-b`, Postgres 18.6, connexion **directe**, plan gratuit, Frankfurt). Aller-retour du PC vers Neon, mesuré par `psql` : **138,5 ms**, stable.

### 8.1 Rôles et RLS

| Mesure | Local | Neon |
|---|---|---|
| `immo_app` : `rolbypassrls` / `rolsuper` / `rolcreaterole` | false / false / false | false / false / false |
| Objets du schéma `public` possédés par `immo_app` | 0 | 0 |
| `CREATE ROLE immo_backup … BYPASSRLS` par SQL | accepté | **accepté** (par `neondb_owner`) |
| Rôle de la console `neondb_owner` | (superutilisateur `postgres`) | `bypassrls=true`, `createrole=true`, non superutilisateur |
| `ENABLE` et `FORCE ROW LEVEL SECURITY` | `organizations`, `units`, `leases`, `memberships` : oui ; `plain_units` (témoin) : non | identique |

### 8.2 Étanchéité (T1 à T5)

| Test | Local | Neon |
|---|---|---|
| T1 sans contexte : `SELECT count(*)` sur chaque table ; `INSERT` | 0 ligne ; refusé `42501` | identique |
| T2 : A lit les lots de B ; `UPDATE` de A sur B ; `INSERT` de A avec l'`org_id` de B ; bail de A sur un lot de B | 0 ; 0 ligne touchée ; `42501` ; `23503` | identique |
| T3 : 50 tâches, pool de 2 connexions, 10 000 itérations avec `OrgTx` (`set_config(…, true)`) | **0 lecture croisée** sur 500 000 lignes lues, 2,4 s | **0 lecture croisée** sur 500 000 lignes lues, 4 467 s |
| T4 (test négatif) : `SET` de session avec 10 % d'oublis | **fuite** : 25 150 lignes étrangères lues dans 503 itérations | **fuite** : 25 100 lignes dans 502 itérations |
| T4b : `SET LOCAL` hors transaction | accepté sans effet (avertissement) ; la valeur de session précédente survit : **fuite** | identique |
| T5 : adhésions avant contexte | 0 sans contexte ; `find_memberships` rend 2 et 1 ; la politique `own_memberships` (avec `app.user_id` seul) rend les 2 de l'utilisateur | identique |

Incident de banc à noter : un premier T5 avait vu une adhésion « sans contexte » parce que T4 laissait `app.org_id` posé sur les connexions du pool partagé. T4 utilise désormais un pool jetable. C'est le piège lui-même, constaté involontairement.

### 8.3 Surcoût par requête (1 000 requêtes mesurées, 100 d'échauffement)

| Variante | Local p50 / p95 (ms) | Neon p50 / p95 (ms) |
|---|---|---|
| Aller-retour : `SELECT 1` par le pool (1 connexion) | 0,20 / 0,22 | 424,0 / 447,6 |
| (i) requête nue, table sans RLS | 0,40 / 0,44 | 370,2 / 374,7 |
| (ii) `OrgTx` : BEGIN, `set_config`, requête, COMMIT | 0,43 / 0,46 | 928,3 / 937,9 |
| (iii) `OrgTx` + `after_release` (RESET) + `before_acquire` (vérification) | 0,52 / 0,56 | 1 099,0 / 1 106,4 |

Lecture : en local, `OrgTx` ajoute 0,03 ms et les garde-fous 0,09 ms de plus. Sur Neon, le coût est entièrement de l'**aller-retour réseau** : `OrgTx` ajoute 558 ms à la requête nue, soit 4,0 allers-retours de 138,5 ms ; les garde-fous en ajoutent 171 ms (1,2). Le temps d'une itération de T3 (6,4 allers-retours) est cohérent avec la variante (ii) (6,7). Même un `SELECT 1` par le pool coûte 3,1 allers-retours (424 ms) : `sqlx` vérifie la connexion à la prise et prépare les instructions ; la décomposition exacte n'a pas été vérifiée. Ces valeurs viennent d'un PC à 138 ms de la base ; **un serveur hébergé dans la région de la base (1 à 2 ms) paiera quelques millisecondes**, pas des centaines. Le p95 absolu est à remesurer à l'étape 01.

### 8.4 Sauvegarde logique sous RLS forcée et restauration

| Variante (`pg_dump -Fc`) | Local | Neon |
|---|---|---|
| **V1** `immo_backup` avec `BYPASSRLS` | code 0 ; comptes du fichier identiques à la source, par table et par organisation | code 0 ; **identiques** (`units` 5 000 / 3 000 / 2 000, `leases` 1 488 / 884 / 576 pour A / B / C, plus `organizations`, `memberships`, `plain_units`) |
| **V2** rôle sans `BYPASSRLS` avec une politique `FOR SELECT … USING (true)` | échec : « query would be affected by row-level security policy » | échec, même message (après correction de mon banc : le droit de lecture doit être accordé par le propriétaire des tables) |
| **V3a** `immo_owner` (propriétaire, RLS forcée) | échec, même message | échec, même message |
| **V3b** `immo_app` | échec, même message | échec, même message |

`pg_dump` fixe `row_security = off` : une politique, même `USING (true)`, ne suffit pas ; **seul** un rôle avec `BYPASSRLS` (ou superutilisateur) peut sauvegarder une table à RLS forcée. `pg_restore` du fichier V1 :

| Mesure | Local | Neon (vers un second projet `immo-spikes-restore`) |
|---|---|---|
| Code de sortie ; durée | 0 ; 0,4 s | 0 ; 18,4 s (rôles compris) |
| Comptes par table et par organisation, source contre restauration | identiques | identiques |
| RLS forcée après restauration | oui (4 tables) | oui (4 tables) |

### 8.5 Restauration à un instant donné (Neon)

100 lignes écrites, instant T0 lu sur le serveur (`2026-10-31T16:17:15.793Z`), 8 s plus tard 50 lignes de plus (150 au total sur `spike-b`). Branche `spike-b-t0` créée à T0 : **100 lignes, dont 0 postérieure à T0**. Requête de création : **1,9 s** ; première requête sur la branche, réveil du calcul compris : **1,2 s**. Fenêtre du plan gratuit : 6 heures (DP-69) ; aucune mesure sur 30 jours (objectif suspendu).

Constat d'outil : le CLI `neon branches create --parent` accepte **soit** une branche **soit** un instant, jamais les deux (`nom@instant` répond « not found ») ; la création d'une branche à un instant d'une branche non par défaut passe par l'API (`neon api /projects/{id}/branches -X POST` avec `parent_id` et `parent_timestamp`). Le CLI ne sait aussi que restaurer une branche en place (`branches restore`), ce qui détruit l'état courant. À consigner dans `DEPLOYMENT.md`.

### 8.6 Écarts et limites

- Particularités de Neon rencontrées : (1) `neondb_owner` ne peut pas exécuter `ALTER ROLE … NOSUPERUSER`, même sans changement (erreur `42501`) ; (2) il ne peut pas accorder de droits sur des tables qu'il ne possède pas : tout `GRANT` passe par `immo_owner` ; (3) un rôle ne peut pas être supprimé tant que des privilèges en dépendent ; (4) les rôles créés par SQL n'ont pas `BYPASSRLS`, sauf demande explicite acceptée.
- Les mesures de durée viennent d'un PC et d'un lien WAN : comparatives, pas des valeurs de production.
- **CU-heures consommées non relevées** (à lire dans la console Neon ; budget de 10 CU-heures pour l'ensemble des prototypes). Plusieurs calculs ont tourné ensemble pendant plus d'une heure.
- `find_memberships` accepte n'importe quel identifiant d'utilisateur : l'appelant doit être la couche `identity`.
- L'inventaire des 44 tests de `DATABASE_SCHEMA.md` §9.4 est fait en fin d'étape, pas ici.

### 8.7 Décision proposée (à valider par Mimis)

1. **Isolation** : confirmer `OrgTx` avec `set_config('app.org_id', …, true)` comme **seul** mécanisme (0 fuite sur 10 000 itérations, deux bases) ; ajouter la règle de revue/lint contre `&PgPool` hors `platform`, `SET` de session et `SET LOCAL` hors transaction, trois formes qui fuient. Les garde-fous `after_release` / `before_acquire` ne sont pas nécessaires pour l'étanchéité mesurée ; ils coûtent 1,2 aller-retour par prise de connexion : à garder ou non après mesure sur le serveur d'essai.
2. **Décision n° 46 / DP-55** : **confirmée sans repli**. Rôle `immo_backup` avec `BYPASSRLS`, créé par SQL, utilisé seulement par la sauvegarde et la restauration ; **RLS forcée conservée**. La variante « politique de lecture totale sans `BYPASSRLS` » ne fonctionne pas.
3. **Latence** : héberger le serveur dans la région de la base ; envisager d'envoyer `BEGIN` et `set_config` en un seul message à l'étape 01.
4. **Restauration** : la branche à un instant donné fonctionne dans la fenêtre du plan ; consigner l'usage de l'API et la limite du CLI.
