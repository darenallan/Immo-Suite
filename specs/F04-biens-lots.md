# F04 — Biens et lots (étape 05)

| Champ | Valeur |
|---|---|
| Statut | **Rédigée le 03/11/2026, validée par Mimis le 03/11/2026** (recommandations du §10 acceptées, DP-92) |
| Étape | 05, `roadmap/etape-05-biens-lots-locataires.md` §4 point 1 |
| Dépend de | dossier §16 F04 ; DP-03, DP-25, DP-41, DP-56 (n° 64) ; `DATABASE_SCHEMA.md` `properties`, `units`, `unit_assignments` ; `API_CONTRACTS.md` §2.5, §2.6, §5.2, §7.4, §7.17 ; `UI_UX_DESIGN.md` S06, S07, « Carte lot », « Tableau responsive », « Filtres », « Recherche unique » |
| Règles | R-005, R-010, R-011, R-012, R-081 (portée), R-094 ; R-006, R-013 et R-093 préparées ici, appliquées à l'étape 06 |
| Critères visés | critères 1, 2, 5, 6, 7 de l'étape 05 ; P16 |
| Hors fiche | baux, statut « occupé », geste « en travaux » et son audit (étape 06) ; mandats et lots couverts (étape 16) ; travaux et documents d'un lot (étapes 15 et 20) ; `import_batch_id` (étape 12) |

## 1. Objet

Représenter le patrimoine réel : un **bien** est un emplacement (immeuble, cour, site, maison) ; un **lot** est ce qu'on loue (maison, appartement, boutique, entrepôt, bureau). Une maison seule se crée en un geste. Rien ne se supprime.

## 2. Règles

| N° | Règle | Source |
|---|---|---|
| 1 | Un bien a un nom (1 à 160 caractères après suppression des espaces de bord), une adresse, un quartier et une ville facultatifs, un propriétaire par défaut facultatif (fiche `owners`, F05) | dossier F04, schéma |
| 2 | **Maison seule** : `POST /properties` avec `single_unit` crée, dans **une seule transaction**, 1 bien et 1 lot. Référence du lot : celle saisie, sinon `1` [proposition, question 4] ; usage `residential`, type `house` par défaut, modifiables dans le même formulaire | R-010 |
| 3 | La référence d'un lot (1 à 40 caractères) est **unique dans son bien, lots archivés compris**. Comparaison après suppression des espaces de bord et **sans tenir compte de la casse** (« a1 » et « A1 » sont le même lot) [proposition, question 5]. Doublon : `409 unit.reference_duplicate`. Deux « A1 » dans deux biens différents : acceptés | R-011, schéma |
| 4 | Usage : `residential`, `commercial`, `mixed`. Type : `house`, `apartment`, `villa`, `studio`, `room`, `shop`, `workshop`, `warehouse`, `office`, `premises`, `other` | R-094, schéma |
| 5 | **Champs selon l'usage** : « activité autorisée » n'existe que pour un lot `commercial` ou `mixed`. Envoyée pour un lot `residential` : `422 validation.failed`, champ `permitted_activity`, code `validation.not_applicable`. Passer un lot de `commercial` à `residential` exige d'effacer l'activité dans la même modification. L'**indexation** n'est pas sur le lot : elle est portée par le bail (DP-41) | R-012, DP-41 |
| 6 | Loyer de référence facultatif, **entier FCFA de 1 à 999 999 999** ; indicatif seulement (le loyer du bail fait foi). Surface facultative, strictement positive, au plus deux décimales, **sans flottant** (texte décimal sur le fil, `numeric` en base) | R-001, schéma, P16 |
| 7 | **Statut d'un lot** : `vacant` à la création. `occupied` est dérivé du bail en cours (étape 06) ; `under_works` est posé à la main (étape 06) ; `archived` par l'archivage. À l'étape 05, aucun bail n'existe : tout lot non archivé est `vacant` | R-013, R-093 |
| 8 | **Archivage**, jamais de suppression : `archived_at`, `archived_by`. Un bien ou un lot archivé reste lisible avec son historique ; il disparaît des listes par défaut (filtre `archived=true` pour le revoir). Archiver un bien archive ses lots non archivés **dans la même transaction**. Refus si un lot a un bail en cours (`409 unit.has_active_lease`) : la vérification est écrite à l'étape 06, quand les baux existent | R-005, R-006 |
| 9 | Un lot archivé ne se modifie plus (`409 unit.archived`, nouveau code) ; même règle pour un bien archivé (`409 property.archived`). Pas de désarchivage au pilote [proposition, question 6] | R-005 (déduit) |
| 10 | **Modification concurrente** : `version` et `ETag` sur chaque fiche ; `PATCH` exige `If-Match` (`428 concurrency.version_required` sinon, `412 concurrency.version_mismatch` si périmé, rien n'est écrit) | n° 64, §5.2 |
| 11 | **Portée de l'agent terrain** : il ne voit que les lots qui lui sont assignés (`unit_assignments` actifs) et les biens qui en contiennent au moins un. Un lot non assigné répond `404` (jamais `403`), y compris par la recherche | R-081 |
| 12 | Droits : lecture `catalog.read` (gérant, gestionnaire, propriétaire ; caissier, comptable et collaborateur en lecture ; agent terrain sur ses lots) ; écriture `catalog.write` (gérant, gestionnaire, propriétaire) | §4.6 |
| 13 | Audit dans la transaction de l'action : `property.created`, `property.updated`, `property.archived`, `unit.created`, `unit.updated`, `unit.archived` (détail : champs changés avant/après) | R-008, étape §5.2 |

## 3. Données (migration de l'étape)

1. `properties` et `units` **comme dans `DATABASE_SCHEMA.md`** (colonnes, contrôles, clés composites `(org_id, id)`, RLS forcée par `apply_org_policies`, `row_version`), sans `import_batch_id` (étape 12). Noms de colonnes du schéma : `district`, `surface_m2`, `permitted_activity` (le contrat d'API dit `neighborhood`, `surface`, `allowed_activity` : question 1).
2. Unicité de la référence insensible à la casse : index unique `(org_id, property_id, lower(btrim(reference)))` à la place de la contrainte `UNIQUE (org_id, property_id, reference)` du schéma (si la question 5 est acceptée).
3. **Colonne de recherche** : `search_key text GENERATED ALWAYS AS (app_fold(…)) STORED` sur `properties` (nom, quartier, ville) et `units` (référence), où `app_fold(text)` est une fonction SQL `IMMUTABLE` (minuscules et `translate()` des lettres accentuées du français, sans extension). Index `(org_id, search_key text_pattern_ops)`. `pg_trgm` reste [APRÈS] (étape §9 : budget de 150 ms mesuré sur 500 lots).
4. `unit_assignments` : ajout de la clé étrangère composite `(org_id, unit_id) → units(org_id, id)`, posée `NOT VALID` puis validée (la table de l'étape 03 est vide hors essais : DP-89). La règle « liste vide seulement » de l'étape 03 est levée : `PUT /users/{membership_id}/unit-assignments` accepte des lots existants, non archivés, de l'organisation.
5. Gardes de mise à jour : `row_version` reste modifiable (cas de test de `DATABASE_SCHEMA.md` §1.2) ; `org_id`, `id`, `property_id`, `created_at`, `created_by` ne le sont pas (droits `UPDATE` par colonne).
6. Droits de `immo_app` : SELECT, INSERT, UPDATE (colonnes modifiables) ; aucun DELETE. `immo_platform` : lecture sous accord support (politique `support_access` de F40, inchangée).

## 4. API (`API_CONTRACTS.md` §7.4)

| Route | Droit | Corps → réponse | Erreurs propres |
|---|---|---|---|
| `GET /properties` | `catalog.read` | filtres `q`, `district`, `owner_id`, `archived` ; tri `name`, `-created_at` → `{ items[], next_cursor }` (nom, quartier, propriétaire, nombre de lots, lots vacants) | `request.filter_unknown` |
| `POST /properties` | `catalog.write` | `{ name, address?, district?, city?, default_owner_id?, single_unit? { reference?, kind?, usage?, surface_m2?, reference_rent?, permitted_activity? } }` → `201` bien (et son lot) | `validation.*`, `owner.not_found` |
| `GET /properties/{id}` | `catalog.read` | fiche, lots (résumés), `version` | `404` |
| `PATCH /properties/{id}` | `catalog.write` | champs du bien ; `If-Match` | `concurrency.*`, `property.archived` |
| `POST /properties/{id}/archive` | `catalog.write` | → bien et ses lots archivés | `unit.has_active_lease` (étape 06) |
| `GET /units` | `catalog.read` | filtres `property_id`, `district`, `usage`, `kind`, `status`, `q`, `archived` → liste (référence, bien, statut, loyer de référence ; locataire et échéance du mois : `null` jusqu'à l'étape 06) | |
| `POST /units` | `catalog.write` | `{ property_id, reference, kind, usage, surface_m2?, reference_rent?, permitted_activity? }` → `201` lot | `unit.reference_duplicate`, `validation.not_applicable`, `property.archived` |
| `GET /units/{id}` | `catalog.read` | aperçu (bien, usage, type, statut, loyer de référence ; bail actif : `null`), `version` | `404` |
| `PATCH /units/{id}` | `catalog.write` | champs du lot ; `property_id` **non** modifiable (un lot ne change pas de bien) ; `If-Match` | `concurrency.*`, `unit.reference_duplicate`, `unit.archived` |
| `POST /units/{id}/archive` | `catalog.write` | → lot archivé | `unit.has_active_lease` (étape 06) |

- `POST /units/{id}/works` et `GET /units/{id}/leases|interventions|documents` : **non livrées** à l'étape 05 (étapes 06, 15, 20) ; les onglets de S07 affichent leur état vide sans appel.
- Pagination par curseur signé (25 par défaut, 100 au plus) ; champ inconnu dans un corps : `422 request.unknown_field` ; filtre inconnu : `422 request.filter_unknown`.
- Identifiant d'une autre organisation, ou lot hors portée de l'agent terrain : `404` (A4, A5).

### 4.1 Recherche unique (`GET /search`, `API_CONTRACTS.md` §2.6, commune à F04 et F06)

- Droit `search.use` (tous les rôles ; agent terrain limité à ses lots). Paramètre `q`, 2 à 80 caractères (sinon `422`), espaces de bord supprimés.
- **Texte** : `app_fold(q)` comparé par préfixe de mot et par sous-chaîne aux `search_key` des locataires (nom affiché), des lots (référence, et nom du bien pour « A1 Zogona ») et des biens (nom, quartier). Insensible aux accents et à la casse : « ouedraogo » trouve « Ouédraogo ».
- **Téléphone** : si `q`, après suppression des espaces, points, tirets et de `+226`/`00226`, compte au moins 4 chiffres, on cherche ces chiffres dans le téléphone des locataires (« 00000101 » et « +226 00 00 01 01 » trouvent `+22600000101`).
- Réponse : `{ tenants[≤8], units[≤8], receipts[] }` ; `receipts` reste vide jusqu'à l'étape 08 ; solde et lots des locataires : `null` et `[]` jusqu'aux étapes 06 et 07. Aucun résultat : `200` avec trois listes vides.
- Portée : l'agent terrain ne reçoit que ses lots et les locataires de ses lots (aucun à l'étape 05) ; le caissier ne reçoit ni contact de secours ni notes ; les fiches archivées et fusionnées sont exclues.
- Budget : 150 ms (p95 serveur) sur 500 lots et 1 000 locataires, mesuré par un test de charge local.
- Événement `search_performed` (nouveau, origine `client`, ajouté à `SPECS` et à `tests/contract/events-whitelist.json`) : `length` (entier 2 à 80), `group` (choix `tenants`, `units`, `receipts`, `none`), `empty` (choix `yes`, `no`) ; **jamais le texte tapé**.
- Front : un champ toujours accessible (en haut sur ordinateur, icône sur mobile, raccourci « / »), suggestions dès 2 caractères après 200 ms, groupes Locataires, Lots, Quittances ; aucun résultat : « Aucun résultat pour « {terme} ». Vérifiez l'orthographe ou cherchez par numéro de téléphone. » [Créer un locataire] (pour qui a `catalog.write`).

## 5. Interface

- **S06 Biens et lots** : liste des biens (nom, quartier, propriétaire, nombre de lots, occupation : « 3 lots, 3 vacants » à cette étape) ; cartes sous 600 px, tableau au-delà (colonnes de « Tableau responsive ») ; filtres quartier, propriétaire, usage, archivés (barre de puces sur ordinateur, feuille « Filtrer » sur mobile, mémorisés côté front) ; « Afficher plus » par 25.
- **Création** : formulaire bien et lots. Interrupteur « Maison seule » : un seul lot, ses champs dans le même formulaire. Sinon, lots ajoutés ligne par ligne (référence, usage, type, surface, loyer de référence) ; le champ « Activité autorisée » n'apparaît que pour un usage commercial ou mixte. Le propriétaire se choisit dans la liste des mandants (F05), avec « Créer un mandant » pour qui en a le droit.
- **S07 Fiche lot** : en-tête (référence, bien, quartier, usage en badge neutre, type, statut, loyer de référence) ; onglets Aperçu, Baux, Paiements, Travaux, Documents (l'onglet actif dans l'adresse) ; Baux, Paiements, Travaux et Documents montrent leur état vide ; bouton « Mettre en travaux » absent (étape 06) ; « Archiver » derrière une modale de confirmation.
- Erreurs : « Cette référence est déjà utilisée dans ce bien. » sous le champ ; conflit de version : bandeau ✱ de §5.2 avec [Recharger].
- États : vide (« Aucun bien pour le moment. Créez votre premier bien ou importez votre portefeuille. » [Créer un bien] ; [Importer] absent jusqu'à l'étape 12), chargement (squelette), erreur, succès (toast). Lot archivé : texte atténué et libellé « Archivé ». Cibles de 44 px, étiquettes liées, résumé des erreurs, clavier, 390 et 1440 px.
- Événements : `screen_viewed` (S06, S07), `validation_error` (champs ajoutés à la liste du front).

## 6. Cas limites

| Cas | Comportement |
|---|---|
| Deux lots « A1 » dans le bien « Zogona » | le second : `409 unit.reference_duplicate` |
| « A1 » dans « Zogona » et « A1 » dans « Gounghin » | acceptés |
| « A1 » puis « a1 » dans le même bien | refusé (question 5) |
| « A1 » archivé, puis nouveau « A1 » dans le même bien | refusé : la référence reste prise (schéma : « archivés compris ») |
| Maison seule « Villa Kaboré » | 1 bien et 1 lot `house`, `residential`, référence `1` ; si la création du lot échoue, aucun bien n'est créé |
| Lot `residential` avec activité « Restauration » | `422`, `validation.not_applicable` |
| Lot `mixed` avec activité | accepté (le bail choisira son type, R-094) |
| Loyer de référence `75000.5`, `0` ou `1 000 000 000` | `422` (`validation.amount_not_integer`, `validation.amount_positive`, `validation.amount_out_of_range`) |
| Recherche « zogona », « ZOGONA », « Zogóna » | même bien |
| Deux gestionnaires modifient le même lot | le second reçoit `412`, sa saisie reste dans le formulaire, rien n'est écrasé |
| Lot d'une autre organisation visé par `property_id` | `404` (clé composite : ISO-3) |
| Archivage d'un bien | ses lots sont archivés ; tout reste lisible ; l'audit contient une ligne par fiche archivée |
| Agent terrain assigné à 2 lots sur 5 | liste, fiche et recherche ne montrent que ces 2 lots ; les 3 autres : `404` |

## 7. Tests (écrits avant le code)

| Test | Contenu | Critère |
|---|---|---|
| Domaine | référence normalisée ; champs applicables selon l'usage (R-012) ; loyer de référence : propriété P16 (entier dans les bornes, refus du reste) | 1, 2 |
| Base | unicité par bien (casse comprise) ; clé composite lot → bien d'une autre organisation refusée (ISO-3) ; `row_version` modifiable, `org_id` non ; aucun `DELETE` ; `app_fold` | 1, 7 |
| Isolation | chaque route ajoutée : un identifiant de B vu de A répond `404` (couverture 100 % du registre) | §7 de l'étape |
| Droits (A3) | 10 routes × 7 rôles, générés depuis la matrice | |
| Portée (A5) | agent terrain : lots assignés visibles, non assignés introuvables (liste, fiche, recherche) | 5 |
| Concurrence | `PATCH` sans `If-Match` → 428 ; périmé → 412 sans écriture | 6 |
| Maison seule | 1 et 1 ; échec du lot ⇒ aucun bien | 1 |
| Archivage | bien et lots archivés, toujours lisibles, filtrés des listes par défaut, non modifiables | 7 |
| E2E 390 et 1440 px | créer « Zogona » avec « A1 » (habitation) et « B1 » (boutique, activité visible) ; maison seule ; doublon de référence | 1, 2 |

## 8. Ce qui sera vérifié sur le poste, et ce qui ne le sera pas

- Migrations exécutées sur l'essai : non (l'essai n'existe pas, C-01 à C-03) ; vérifiées sur la base jetable.
- Budget de 150 ms de la recherche : mesuré sur la base locale avec 500 lots.
- Modèle Excel de l'agence (C-09) : absent ; les champs suivent le dossier et le schéma, à confronter au premier fichier reçu.

## 9. Écarts aux documents propriétaires (à corriger avant la migration)

- Noms de champs du contrat d'API (question 1).
- `unit.archived`, `property.archived`, `owner.not_found`, `validation.not_applicable` : codes nouveaux, à ajouter au catalogue d'`API_CONTRACTS.md` et à son journal des changements (§9).
- Unicité insensible à la casse (question 5) : `DATABASE_SCHEMA.md` `units`.

## 10. Questions pour Mimis

1. **Noms des champs** : le contrat d'API et le schéma divergent (`neighborhood`/`district`, `surface`/`surface_m2`, `allowed_activity`/`permitted_activity`). *Recommandation : prendre partout les noms du schéma et corriger `API_CONTRACTS.md` §7.4* ; ils sont plus précis et aucun client ne les utilise encore.
2. **Propriétaire du bien et droits** : le formulaire de bien doit lister les mandants, mais `GET /owners` exige `statement.read` (voir F05, question 1).
3. **Bien sans lot** : un bien peut-il exister sans aucun lot (saisi en deux temps) ? *Recommandation : oui* ; il apparaît avec « 0 lot ».
4. **Référence du lot d'une maison seule** : *recommandation : celle saisie, sinon `1`*. Autre option : rendre la référence obligatoire même pour une maison seule.
5. **Casse de la référence** : *recommandation : « A1 » et « a1 » sont le même lot (refus du doublon)* ; la saisie garde la casse tapée. Autre option : sensible à la casse, comme le schéma l'écrit aujourd'hui.
6. **Désarchivage** : *recommandation : aucun au pilote* (une erreur d'archivage se corrige par le support) ; à revoir si le pilote le demande.
