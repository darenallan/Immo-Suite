# F06 — Fiche locataire (étape 05)

| Champ | Valeur |
|---|---|
| Statut | **Rédigée le 03/11/2026, validée par Mimis le 03/11/2026** (recommandations du §10 acceptées, DP-92) |
| Étape | 05, `roadmap/etape-05-biens-lots-locataires.md` §4 point 2 |
| Dépend de | dossier §16 F06 ; DP-58 (R-017), n° 43 (ouverte), n° 64 ; `DATABASE_SCHEMA.md` `tenants`, §4.7 ; `API_CONTRACTS.md` §2.4, §2.6, §4.5, §5.2, §7.5, §7.17 ; `UI_UX_DESIGN.md` S08, S09, « Carte locataire », « Recherche unique » ; `ARCHITECTURE.md` §13.2 (pièce d'identité désactivée) |
| Règles | R-005, R-017 [à confirmer avec l'agence pilote à la découverte], R-018, R-019, R-081 |
| Critères visés | critères 3, 4, 6, 7, 8 de l'étape 05 |
| Hors fiche | baux, paiements, relances d'un locataire (étapes 06 à 15 : les onglets sont vides) ; pièce d'identité réelle (interrupteur, après le pilote) ; espace locataire F29 (« À venir ») ; colocation (étape 06, question ouverte de `DOMAINE.md`) |

## 1. Objet

Centraliser l'identité et les contacts d'un locataire, personne ou société, pour le retrouver en deux secondes (nom, téléphone, lot) et ne jamais créer deux fiches pour la même personne sans le savoir.

## 2. Règles

| N° | Règle | Source |
|---|---|---|
| 1 | Type `person` ou `company`. Personne : nom (obligatoire) et prénom (facultatif), 1 à 120 caractères chacun. Société : raison sociale dans `last_name`, `first_name` vide, activité exercée (`trade_activity`) facultative. Nom affiché : « Prénom Nom » ou la raison sociale | schéma, dossier F06 |
| 2 | **Téléphone** : facultatif, forme canonique `+226` suivie de 8 chiffres. Entrée tolérante : `70123456`, `70 12 34 56`, `+226 70 12 34 56`, `0022670123456`, `226 70123456`, avec espaces, points ou tirets ; tout autre format : `422`, code `validation.phone_format`, message « Le numéro doit comporter 8 chiffres, avec ou sans +226. ». 7 ou 9 chiffres : refusés. Une seule fonction de normalisation (`crates/domain`), partagée avec l'identifiant de connexion de F01 | R-018, §2.4 |
| 3 | Case « Ce numéro est sur WhatsApp » (`phone_is_whatsapp`) : seulement si un téléphone est saisi | schéma |
| 4 | **Unicité du téléphone** parmi les fiches **non archivées** de l'organisation. Création ou modification avec un numéro déjà porté : `409 tenant.phone_duplicate` avec `conflict: { id, display_name }`, **rien n'est créé**. Deux fiches ne partagent jamais un numéro au pilote (couple, colocataires : un seul porte le numéro, l'autre est sans numéro ou avec le sien) | R-017, DP-58, n° 43 |
| 5 | Une fiche archivée **libère** son numéro : une nouvelle fiche peut le prendre | R-017, index partiel |
| 6 | E-mail facultatif, enregistré en minuscules, format vérifié (`validation.email_format`) | schéma |
| 7 | **Contact de secours** (nom et téléphone `+226`) et **notes** : visibles du gérant, du gestionnaire et du propriétaire seulement ; **absents** de la réponse pour le caissier, l'agent terrain, le comptable et le collaborateur (absents, pas `null`) | §4.5, R-019 |
| 8 | **Pièce d'identité désactivée au pilote** : aucun champ de saisie ; `id_document_enc` existe en base et reste vide ; `GET /tenants/{id}/id-document` répond `403 feature.disabled` pour tous ; tout corps qui contient `id_document` : `422 request.unknown_field` | R-019, §13.2 |
| 9 | **Fusion** de deux fiches existantes A (doublon) dans B (conservée) : A est archivée avec `merged_into_id = B` dans la même transaction ; **aucune donnée n'est déplacée ni recopiée** (l'historique reste sur l'identifiant de A : les écritures sont immuables) ; B n'est pas modifiée. Conditions : A ≠ B, même organisation, aucune des deux archivée, A pas déjà fusionnée. Audit `tenant.merged` (A, B). La fiche de A, ouverte par son ancien lien, affiche « Fiche fusionnée dans {B} » et y renvoie | R-017, schéma |
| 10 | **Lecture à travers la fusion** : à partir de l'étape 06, les baux, paiements et relances d'un locataire incluent ceux des fiches fusionnées en lui (chaîne `merged_into_id` suivie jusqu'au bout). À l'étape 05, il n'y a rien à agréger | schéma (déduit) |
| 11 | **Archivage** : `archived_at`, `archived_by` ; la fiche reste lisible avec son historique, quitte les listes par défaut, ne se modifie plus (`409 tenant.archived`). Refus si le locataire a un bail en cours (vérification ajoutée à l'étape 06). Pas de désarchivage au pilote (F04, question 6) | R-005 |
| 12 | Modification concurrente : `version`, `ETag`, `If-Match` sur `PATCH` (`428`, `412`) | n° 64 |
| 13 | **Portée de l'agent terrain** : il ne voit que les locataires de ses lots, c'est-à-dire ceux qui ont un bail sur un lot assigné. À l'étape 05, sans baux, **il n'en voit aucun** (liste vide, `404` sur une fiche) | R-081 |
| 14 | Droits : lecture `catalog.read` (caissier, comptable, collaborateur en lecture ; agent terrain selon la règle 13) ; création, modification, fusion, archivage `catalog.write` (gérant, gestionnaire, propriétaire) | §4.6, dossier F06 |
| 15 | Audit dans la transaction : `tenant.created`, `tenant.updated` (champs changés, le numéro avant et après : question 5), `tenant.merged`, `tenant.archived` | étape §5.2 |

## 3. Données (migration de l'étape)

1. `tenants` **comme dans `DATABASE_SCHEMA.md`** : `last_name`, `first_name`, `trade_activity`, un seul `phone` et `phone_is_whatsapp`, `email`, `backup_contact_name`, `backup_contact_phone`, `id_document_enc` (vide), `notes`, `merged_into_id` (clé composite vers `tenants`), archivage, `row_version` ; index unique partiel `tenants_phone_unique` ; contrôle `tenants_merge_archives` ; sans `import_batch_id` (étape 12).
2. Contrôle ajouté : `merged_into_id <> id`.
3. `search_key` généré par `app_fold()` (F04 §3 point 3) sur « prénom nom » et la raison sociale ; le téléphone est cherché sur ses chiffres.
4. Droits de `immo_app` : SELECT, INSERT, UPDATE par colonne (jamais `org_id`, `id`, `created_*`) ; aucun DELETE. Le masquage selon le rôle se fait dans l'application (règle 7) : la base ne distingue pas les rôles de l'agence.

## 4. API (`API_CONTRACTS.md` §7.5)

| Route | Droit | Corps → réponse | Erreurs propres |
|---|---|---|---|
| `GET /tenants` | `catalog.read` | filtres `q`, `archived` (`has_arrears` à l'étape 11) ; tri `name`, `-created_at` → liste (nom affiché, téléphone, lots : `[]` jusqu'à l'étape 06, solde : `null`) | |
| `POST /tenants` | `catalog.write` | `{ kind, last_name, first_name?, trade_activity?, phone?, phone_is_whatsapp?, email?, backup_contact? { name, phone }, notes? }` → `201` fiche | `tenant.phone_duplicate` (avec `conflict`), `validation.phone_format`, `validation.email_format` |
| `GET /tenants/{id}` | `catalog.read` | fiche, `version` ; champs masqués selon le rôle ; `merged_into_id` si fusionnée | `404` |
| `PATCH /tenants/{id}` | `catalog.write` | champs de la fiche ; `If-Match` | `concurrency.*`, `tenant.phone_duplicate`, `tenant.archived` |
| `POST /tenants/{id}/merge` | `catalog.write` | `{ into_id }` → fiche conservée | `tenant.merge_invalid` (même fiche, archivée, déjà fusionnée), `404` |
| `POST /tenants/{id}/archive` | `catalog.write` | → fiche archivée | `lease.active_exists` (étape 06) |
| `GET /tenants/{id}/id-document` | `catalog.write` | → toujours `403 feature.disabled` au pilote | `feature.disabled` |

- `GET /tenants/{id}/leases|payments|reminders` : non livrées (étapes 06 à 15) ; les onglets de S09 affichent leur état vide sans appel.
- `display_name` est **calculé** dans la réponse ; il n'est pas accepté en entrée (question 1).

## 5. Interface

- **S08 Locataires** : liste (nom, solde, statut sur mobile ; en plus le téléphone « +226 70 12 34 56 », les lots et le bail sur ordinateur, vides à cette étape) ; filtre « archivés » ; « Afficher plus » par 25.
- **Création** : choix Personne / Société (les champs suivent), téléphone avec la case WhatsApp, e-mail, contact de secours (absent du formulaire pour qui ne peut pas le voir), notes. Le format du numéro est vérifié à la sortie du champ, le doublon à l'envoi. Pas de champ « pièce d'identité ».
- **Doublon** : bandeau « Un locataire avec ce numéro existe déjà : Issouf Tapsoba. » avec [Ouvrir la fiche] (question 2 pour [Fusionner]).
- **Fusion** : depuis une fiche, « Fusionner avec une autre fiche » : recherche de la fiche à conserver, les deux fiches côte à côte, modale « Fusionner {A} dans {B} ? {A} sera archivée ; son historique restera visible depuis {B}. » [Annuler] [Fusionner].
- **S09 Fiche locataire** : coordonnées ; onglets Aperçu, Baux, Paiements, Relances, Documents (vides, avec leur état vide) ; carte avec Encaisser et Relancer **désactivés** jusqu'aux étapes 07 et 15, Appeler et WhatsApp selon la question 4 ; « Archiver » derrière une modale ; repère « Espace locataire : à venir ».
- États vide, chargement, erreur (numéro invalide, doublon), succès ; cibles de 44 px ; étiquettes liées ; erreurs liées au champ et résumées ; 390 et 1440 px.
- Événements : `screen_viewed` (S08, S09), `validation_error` (champs `phone`, `email`, `last_name`).

## 6. Cas limites

| Cas | Comportement |
|---|---|
| `70123456`, `+226 70 12 34 56`, `0022670123456` | même valeur `+22670123456`, donc la même fiche (la seconde création : `409`) |
| `7012345` (7 chiffres), `+2267012345678` | `422 validation.phone_format` avec le message explicite |
| « Issouf Tapsoba » `+226 00 00 01 01`, puis « I. Tapsoba » avec le même numéro | `409 tenant.phone_duplicate`, proposition d'ouvrir « Issouf Tapsoba » ; aucune seconde fiche |
| Couple qui partage un numéro | refusé au pilote : la seconde fiche se crée sans numéro (R-017, à confirmer avec l'agence pilote) |
| Fiche archivée avec `+22670123456`, puis nouvelle fiche avec ce numéro | acceptée |
| Changer le numéro d'une fiche pour celui d'une autre | `409 tenant.phone_duplicate` |
| Fusion de A dans B, puis de B dans C | acceptée ; A → B → C, la lecture suit la chaîne jusqu'à C |
| Fusion de A dans A, ou dans une fiche archivée | `409 tenant.merge_invalid` |
| Caissier qui ouvre une fiche | nom, téléphone, e-mail ; ni contact de secours, ni notes (absents du JSON) |
| `id_document` dans un corps | `422 request.unknown_field` |
| Société « Boutique Wend-Panga SARL », activité « Quincaillerie » | `kind = company`, raison sociale affichée, pas de prénom |
| Deux gestionnaires changent le numéro en même temps | le second : `412`, sa saisie reste dans le formulaire |

## 7. Tests (écrits avant le code)

| Test | Contenu | Critère |
|---|---|---|
| Domaine | normalisation : les formes valides donnent la même valeur ; 7 et 9 chiffres, lettres, `++226` refusés (propriété : toute sortie respecte `^\+226[0-9]{8}$`) | 3 |
| Base | unicité parmi les non archivées (archivée : numéro libéré) ; `merged_into_id` exige l'archivage ; fusion vers une fiche d'une autre organisation refusée (clé composite) ; aucun `DELETE` | 3, 7 |
| API | doublon : `409` avec `conflict`, aucune ligne créée ; fusion (conditions, audit, rien de déplacé) ; masquage par rôle (7 rôles × champs) ; `feature.disabled` ; A4 ; A10 (téléphone invalide, champ inconnu) ; `412` | 3, 4, 6, 8 |
| Portée (A5) | agent terrain : aucun locataire visible à l'étape 05 | 5 |
| Recherche | « tapsoba », « TAPSOBA », « Tapsóba », « 00000101 », « +226 00 00 01 01 » trouvent la fiche | démo 4 |
| E2E 390 et 1440 px | créer un locataire ; provoquer le doublon ; ouvrir la fiche existante ; fusionner deux fiches ; rechercher | 3 |

## 8. Ce qui sera vérifié sur le poste, et ce qui ne le sera pas

- Fixtures : uniquement le bloc fictif `+226 00 00 …` ; aucune donnée réelle.
- Colocation et numéro partagé : non tranchés (question ouverte de `DOMAINE.md`) ; R-017 reste [à confirmer avec l'agence pilote à la découverte].

## 9. Écarts aux documents propriétaires (à corriger avant la migration)

- Corps de `POST /tenants` (question 1) : `API_CONTRACTS.md` §7.5 écrit `display_name`, `phones[]`, `secondary_contact` ; le schéma a `last_name`/`first_name`, un `phone` et `backup_contact_*`.
- Codes nouveaux : `tenant.archived`, `tenant.merge_invalid`, `lease.active_exists` (réservé à l'étape 06).
- Message de `tenant.phone_duplicate` : bouton [Fusionner] (question 2).
- Décision n° 43 (unicité parmi les non archivées) : appliquée comme le propose DP-58 ; à inscrire comme prise dans `DECISIONS.md` si vous validez.

## 10. Questions pour Mimis

1. **Champs du locataire** : *recommandation : suivre le schéma* (nom et prénom séparés, **un seul téléphone** avec la case WhatsApp, contact de secours avec nom et téléphone) et corriger le contrat d'API. Un seul numéro garde simple la règle d'unicité ; un second pourra s'ajouter si le pilote le demande. Autre option : plusieurs numéros par fiche (table à part, unicité sur chacun).
2. **Bouton [Fusionner] du doublon** : au moment du doublon, la seconde fiche n'existe pas, il n'y a donc rien à fusionner. *Recommandation : n'afficher que [Ouvrir la fiche]* ; la fusion reste une action entre deux fiches existantes (créées sans numéro, avec deux numéros différents, ou plus tard par l'import). La démonstration 3 devient : doublon refusé, puis fusion de deux fiches existantes. Autre option : [Fusionner] ouvre la fiche existante en modification, pré-remplie des champs vides saisis.
3. **Fusion sans recopie** : *recommandation : la fiche conservée n'est pas modifiée* ; si A avait un e-mail que B n'a pas, l'utilisateur le recopie. Autre option : compléter automatiquement les champs vides de B à partir de A (règle à écrire dans `DOMAINE.md`).
4. **Boutons Appeler et WhatsApp** : ils peuvent marcher dès cette étape (lien `tel:` et lien `wa.me` sans message), puisque le numéro est là. *Recommandation : les activer pour un locataire qui a un numéro* (la case WhatsApp conditionne le second) ; Encaisser et Relancer restent désactivés jusqu'aux étapes 07 et 15.
5. **Numéro dans l'audit** : *recommandation : garder l'ancien et le nouveau numéro en clair dans `tenant.updated`* : c'est la donnée la plus utile pour réparer un écrasement, et le journal n'est lisible que du gérant et du comptable.
