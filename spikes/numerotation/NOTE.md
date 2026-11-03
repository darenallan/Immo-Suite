# Prototype (c) — Numérotation des quittances et idempotence sous concurrence

| Champ | Valeur |
|---|---|
| Statut | **Exécuté le 31/10/2026 (protocole validé par Mimis).** Résultats en section 8 ; décision proposée en section 6, **en attente de validation de Mimis** |
| Durée maximale | 1 jour (dépasser est un résultat à noter) |
| Décision attendue | Mécanisme de numérotation et d'idempotence ; confirmation de l'année d'émission (DP-61) |
| Documents | `ARCHITECTURE.md` §4.4, §4.5 ; `TESTING_QA.md` §2.3, §3 ; `DOMAINE.md` R-050, E-07, E-10, E-27 ; DP-15, DP-61, DP-69 |

## 1. Objectif

Prouver que le compteur transactionnel `INSERT … ON CONFLICT DO UPDATE … RETURNING` donne zéro trou, zéro doublon et des numéros consécutifs dans l'ordre de validation, y compris avec des annulations après attribution et un changement d'année ; que la même clé d'idempotence envoyée trois fois en parallèle ne crée qu'un paiement ; mesurer le p95 d'une validation.

## 2. Environnement

- Deux configurations, pour séparer le mécanisme du réseau : **L** = PostgreSQL local 5433 (tout sur le PC) ; **N** = serveur Axum sur le PC, base Neon `immo-spikes` (branche `spike-c`, Frankfurt, plan gratuit, DP-69).
- Rust 1.99.1 : `spikes/numerotation/` (serveur Axum minimal et générateur de charge dans le même crate). Une seule route, `POST /spike/payments` avec `Idempotency-Key`. Aucune route de `API_CONTRACTS.md`.
- Rôle `immo_app` du prototype (b). Budget de calcul : au plus 10 CU-heures pour l'ensemble des prototypes.

## 3. Protocole

1. **Schéma jouet** : `organizations`, `receipt_counters (org_id, series, year, last_number, PK (org_id, series, year))`, `payments`, `receipts` (avec `sequence`, `validated_at` pris par `clock_timestamp()` **après** la mise à jour du compteur, et `UNIQUE (org_id, series, year, sequence)`), `idempotency_keys (org_id, key, route, request_hash, response_status, response_body)` unique sur `(org_id, key)`, `audit_log` jouet écrit dans la même transaction. Verrou consultatif par bail (`pg_advisory_xact_lock(hashtextextended(lease_id::text, 0))`), **pris avant** le compteur, le compteur étant pris en dernier.
2. **Horloge injectée** : un trait `Clock` ; la route lit l'heure par lui, jamais par `now()`.
3. **Charge de référence** : 3 organisations, série `official`, 20 clients concurrents. Les clients continuent jusqu'à **10 000 quittances validées** (critère 4 lu littéralement). 10 % des tentatives, tirées avec une graine fixe, échouent volontairement **après** l'attribution du numéro (annulation forcée de la transaction). Un second passage « 20 × 500 tentatives » est joué pour lever l'ambiguïté du libellé et consigné à part.
4. **Vérifications après la charge**, par organisation, série et année : `min(sequence) = 1` ; `max(sequence) = count(*)` (zéro trou) ; aucun doublon (contrainte et requête) ; `validated_at` non décroissant quand la séquence croît (numéros dans l'ordre de validation) ; `last_number = max(sequence)` (les incréments annulés ne laissent aucune trace) ; nombre de lignes d'audit = nombre de quittances.
5. **E-10** : deux paiements successifs reçoivent 1 puis 2.
6. **E-27 (changement d'année)** : jouer 812 validations en 2026, puis une validation à 2027 avec la `Clock` : la séquence 2027 repart à 1 (`2027-000001`). Cas DP-61 : paiement daté du 31/12/2026 saisi le 02/01/2027 reçoit un numéro de 2027.
7. **Idempotence** : (i) même clé × 3 requêtes simultanées, répété 100 fois avec des clés différentes : 1 paiement, 3 réponses identiques octet pour octet, même numéro ; (ii) même clé, contenu différent : `422 idempotency.key_reused_with_different_body`, aucun effet ; (iii) réponse perdue (le client coupe après la validation) puis nouvel essai : réponse enregistrée, rien de créé (E-07) ; (iv) coupure avant validation (panne injectée) puis nouvel essai : le paiement est créé normalement.
8. **Deux paiements simultanés sur le même bail** : sérialisation observée dans les horodatages, aucun interblocage (compteur d'erreurs `40P01` = 0).
9. **Mesures de durée** : p50, p95 et p99 de la validation vue par le **client** et vue par le **serveur** (de `BEGIN` à `COMMIT`), temps de maintien du verrou du compteur, nombre d'allers-retours par validation, aller-retour de base du PC vers Neon.

## 4. Résultats chiffrés (à remplir)

| Mesure | L (local) | N (Neon) |
|---|---|---|
| Validations réussies / annulées forcées | | |
| Trous / doublons / violations d'ordre | | |
| E-10, E-27, cas DP-61 | | |
| Idempotence (i) à (iv) | | |
| Interblocages | | |
| p50 / p95 / p99 client (ms) | | |
| p50 / p95 / p99 serveur, de BEGIN à COMMIT (ms) | | |
| Maintien du verrou du compteur (p95, ms) | | |
| Allers-retours par validation ; aller-retour de base (ms) | | |
| Débit (validations par seconde) ; durée totale | | |
| CU-heures consommées | | |

## 5. Critères d'acceptation (étape 00 §6)

4. 0 trou, 0 doublon, numéros consécutifs sur au moins 10 000 validations avec 10 % d'annulations ; E-10 et E-27 reproduits.
5. La même clé envoyée 3 fois en parallèle crée 1 paiement et rend 3 réponses identiques.
Le p95 est une mesure à consigner (objectif de `TESTING_QA.md` §8.2 : enregistrement p95 ≤ 300 ms côté serveur), pas un critère de réussite de l'étape.

## 6. Décision proposée

À rédiger après mesure : confirmer le mécanisme de `ARCHITECTURE.md` §4.4 et §4.5, ou le corriger avec les chiffres.

## 7. Écarts et risques déjà connus

- La charge, le serveur et la base ne sont pas dans la même région : avec un aller-retour de l'ordre de 100 à 300 ms entre ce PC et Francfort et plusieurs allers-retours par validation, le verrou du compteur est tenu longtemps et le débit sera bas (3 organisations en parallèle). Le p95 de la configuration N **ne représente pas** un serveur hébergé près de la base ; la configuration L et le p95 côté serveur séparent le mécanisme du réseau. La durée totale peut atteindre plusieurs dizaines de minutes.
- Plan gratuit (DP-69) : calcul 0,25 à 2 CU, mise en veille après 5 minutes ; échauffement avant chaque série.
- `pg_xact_commit_timestamp` n'est pas supposé disponible : l'ordre se vérifie par `validated_at`, valable parce que le verrou du compteur est tenu jusqu'à la fin de la transaction.

## 8. Résultats (exécution du 31/10/2026)

Prototype : `spikes/numerotation/` (serveur Axum + générateur de charge + vérification). Journaux : `spikes/numerotation/out/local-run.txt` et `neon-run.txt` (hors Git). Configuration **L** : serveur et base sur le PC (PostgreSQL 18.6 local). Configuration **N** : serveur sur le PC, base Neon (branche `spike-c`, Frankfurt, plan gratuit), aller-retour PC–Neon **138,5 ms** mesuré.

### 8.1 Tableau des mesures

| Mesure | L (local) | N (Neon) |
|---|---|---|
| Charge principale (20 clients, 3 organisations, 10 % d'annulations forcées après le numéro) | 10 017 validées sur 11 186 tentatives, 7 s, 1 375 validations/s | 10 014 validées sur 11 174 tentatives, 2 129 s, 4,7 validations/s |
| Second passage « 20 × 500 » (série d'essai) | 8 968 validées sur 10 000, 8 s | 8 968 validées sur 10 000, 1 978 s |
| Réponses inattendues | 0 | 0 |
| Annulations forcées (rollback après attribution du numéro) | 2 201 | 2 192 |
| Quittances à la fin ; lignes d'audit ; paiements sans quittance | 19 113 ; 19 113 ; 0 | 19 110 ; 19 110 ; 0 |
| Trous / doublons / violations d'ordre / compteur ≠ maximum | **0 / 0 / 0 / 0** | **0 / 0 / 0 / 0** |
| E-10 | 1 puis 2 | 1 puis 2 |
| E-27 et cas DP-61 (paiement du 31/12/2026 saisi le 02/01/2027) | `2027-000001`, `2027-000002`, retour `2026-000001` | identique |
| Idempotence (i) : 100 clés × 3 requêtes simultanées | 100/100 réponses identiques, 1 seul paiement, même numéro | identique |
| (ii) même clé, contenu différent | 422 `idempotency.key_reused_with_different_body`, 1 paiement | identique |
| (iii) réponse perdue puis nouvel essai | 504 puis 201 rejoué, 1 paiement | identique |
| (iv) coupure avant validation puis nouvel essai | 500, 0 paiement, puis 201, 1 paiement | identique |
| 20 paiements simultanés sur un même bail | 20 validés en 0,1 s, numéros consécutifs | 20 validés en 19,6 s, consécutifs |
| Interblocages (`40P01`) ; autres erreurs de base | 0 ; 0 | 0 ; 0 |
| Client, p50 / p95 / p99 (ms) | 9 / 35 / 73 | 3 130 / 7 651 / 11 312 |
| Serveur, de BEGIN à COMMIT, p50 / p95 / p99 (ms) | 8,8 / 41,2 / 93,4 | 3 106 / 7 670 / 11 299 |
| Maintien du verrou du compteur, p50 / p95 (ms) | 1,6 / 6,1 | 552 / 749 |

Contrôle négatif de la vérification : une quittance supprimée à la main dans la base locale est bien signalée (« ÉCART DÉTECTÉ », compteur ≠ maximum).

### 8.2 Lecture des chiffres

- Le p95 de la configuration N **ne représente pas** un serveur hébergé près de la base : 20 clients se disputent trois verrous de compteur tenus ≈ 4 allers-retours (552 ms = 4 × 138 ms), d'où une attente en file de plusieurs secondes. En L (aller-retour négligeable), le p95 serveur est de 41 ms, sous l'objectif de 300 ms de `TESTING_QA.md` §8.2. Le p95 absolu devra être remesuré à l'étape 01 avec un serveur dans la région de la base.
- Après le compteur, la transaction exécute encore 4 instructions (quittance, audit, mise à jour de la clé d'idempotence, validation). Leur nombre est ce qui fixe la durée du verrou : en région, avec ≈ 1 ms d'aller-retour, environ 5 ms par validation et par organisation, soit de l'ordre de 200 validations/s par organisation [estimation, à vérifier à l'étape 01].

### 8.3 Écarts au protocole et limites

- **E-27** a été joué avec 2 validations en 2027 puis 1 en 2026 (retour à l'année précédente), et non sur « 812 validations en 2026 puis une en 2027 » : la charge principale produit déjà plus de 3 000 quittances par organisation en 2026, donc la remise à 1 et l'indépendance des années sont éprouvées sans cette répétition.
- **Idempotence** : la ligne de clé est insérée en premier (`ON CONFLICT DO NOTHING`) puis **mise à jour** à la fin avec la réponse. Cela exige un `UPDATE` sur `idempotency_keys` pour `immo_app`, en écart avec la règle « jamais d'`UPDATE` sur une écriture » (acceptable : ce n'est pas une table d'écriture seule ; l'alternative « insérer la clé à la fin » n'a pas été essayée). Même remarque pour `receipt_counters`.
- La durée de la configuration N (≈ 1 h 10) est un résultat, conforme à la prévision de la section 7.
- Mesures prises depuis un PC avec un lien WAN : valeurs comparatives, pas des valeurs de production.
- Les CU-heures consommées par la branche `spike-c` n'ont pas été relevées ; à lire dans la console Neon.

### 8.4 Décision proposée (à valider par Mimis)

Confirmer le mécanisme de `ARCHITECTURE.md` §4.4 (compteur `INSERT … ON CONFLICT DO UPDATE … RETURNING`, verrou consultatif par bail pris avant, compteur pris en dernier, rien d'autre que la numérotation dans le verrou) et de §4.5 (table de clés d'idempotence, ligne prise en premier), avec deux précisions : (1) accorder `UPDATE` sur `idempotency_keys` et `receipt_counters` à `immo_app` et le documenter dans `DATABASE_SCHEMA.md` ; (2) héberger le serveur dans la région de la base, la durée du verrou étant proportionnelle au nombre d'allers-retours après le compteur.

