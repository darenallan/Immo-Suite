# Étape 05 — Biens, lots, locataires et fiches mandants

| Champ | Valeur |
|---|---|
| Statut | Terminée le 04/11/2026 (fiches validées, DP-92 ; écarts DP-93 ; rapport `ops/rapports/ETAPE-05.md`) |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T1 (F04, F06) ; T2 pour la fiche mandant (partie de F05 nécessaire à F04 et à l'import) |
| Effort | M, environ 10 JD |
| Dépend de | Étape 02 |
| Jalon | Contribue à J2 et J3 |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Représenter le patrimoine réel d'une agence : biens, lots (habitation, commercial, mixte), locataires (personnes ou sociétés) et fiches des mandants, avec recherche unique et portée de l'agent terrain.
Valeur : les données que l'import (étape 12) fera entrer ont leur place exacte ; une maison seule se crée en un geste ; le caissier retrouve un locataire par nom, téléphone ou lot.

Choix de découpage justifié : la **fiche** du mandant (table `owners`, sans mandat ni commission) est livrée ici parce que `properties.default_owner_id`, l'import et la règle R-016 (plan Propriétaire) en dépendent ; les mandats, taux et comptes rendus restent en étapes 16 et 17.

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections |
|---|---|---|
| Fonctionnalités | F04, F06, F05 (fiche mandant seulement) | dossier §16 F04, F06, F05 |
| Règles | R-005, R-010, R-011, R-012, R-016, R-017, R-018, R-019, R-094 | `DOMAINE.md` « Patrimoine, locataires, mandats » |
| Décisions | DP-03, DP-25, DP-41, DP-58 (R-017), DP-66 ; n° 43, 45, 64 | `DECISIONS.md` |
| Contrat | `API_CONTRACTS.md` | §2.4 (téléphones), §2.5 (pagination), §2.6 (recherche), §5.2 (`If-Match`), §7.4, §7.5 (locataires), §7.8 (`/owners`), §7.17 (`/search`) |
| Écrans | `UI_UX_DESIGN.md` | S06, S07, S08, S09, « Carte locataire et carte lot », « Recherche unique », « Filtres », « Tableau responsive » |
| Tables | `DATABASE_SCHEMA.md` §3 | « Patrimoine, mandants et mandats » (`properties`, `units`, `owners`), « Locataires, baux et échéances » (`tenants`), `unit_assignments`, §4.7 (téléphone unique) |
| Permissions | `API_CONTRACTS.md` §4.5 (champs selon le rôle), §4.6 (`catalog.read`, `catalog.write`) | idem |

## 3. Prérequis

Étape 02 (authentification, `authorize`, `OrgTx`) ; étape 03 pour l'écran des lots assignés. Décisions : n° 43 (unicité du téléphone parmi les fiches non archivées, proposée par DP-58 pour le pilote), n° 45 (rôles `owner` et `collaborator`), n° 64 (`row_version` et `If-Match`, prise). Modèle Excel de l'agence (C-09) : consulter ses colonnes pour valider les champs de lot et de locataire.

## 4. Fiches de spécification à produire ou valider

1. `specs/F04-biens-lots.md` — règles R-005, R-010 à R-012, R-094 ; lot : référence unique dans le bien, usage (habitation, commercial, mixte), type, surface, loyer de référence, « activité autorisée » seulement pour un lot commercial (DP-41) ; statut dérivé du bail (R-013, étape 06) ; « en travaux » posé à la main (R-093, étape 06). Cas limites : maison seule ⇒ 1 bien + 1 lot ; archivage d'un bien dont un lot a un bail en cours (refusé, étape 06) ; recherche avec accents. Écrans S06, S07. Erreurs `unit.reference_duplicate`, `concurrency.version_mismatch`. Cas chiffrés : deux lots « A1 » dans le même bien refusés ; deux « A1 » dans deux biens acceptés.
2. `specs/F06-fiche-locataire.md` — personne ou société, téléphones (dont WhatsApp), e-mail, contact de secours (invisible du caissier), pièce d'identité **désactivée** (R-019 : chiffrée, masquée, `feature.disabled`) ; doublon de téléphone : proposer la fiche existante ou la fusion (R-017, G-19) ; fusion conservant l'historique par `merged_into_id` ; format `+226` suivi de 8 chiffres (R-018). Cas limites : couple partageant un numéro (non autorisé au pilote), fiche archivée qui libère le numéro, colocation. Cas chiffrés : `70123456`, `+226 70 12 34 56` et `0022670123456` donnent `+22670123456` ; 7 chiffres refusé.
3. `specs/F05-mandants.md` (partie fiche) — nom, type, téléphone, mode de reversement ; fiche `is_self` créée avec une organisation du plan Propriétaire (R-016) ; coordonnées de paiement hors étape (étape 16). Cas limites : mandant archivé qui possède des lots.

**S'arrêter pour la validation de Mimis après ces fiches.**

## 5. Tâches détaillées

### 5.1 Base de données
1. `owners` (`kind`, `display_name`, `phone`, `is_self`, `payout_method`, `archived_at`/`archived_by`, `row_version`) ; `properties` (`name`, `address`, `neighborhood`, `city`, `default_owner_id`, archivage, `row_version`) ; `units` (`property_id`, `reference` avec `UNIQUE (org_id, property_id, reference)`, `kind`, `usage`, `surface`, `reference_rent`, `allowed_activity`, `under_works`, archivage, `row_version`) ; `tenants` (`kind`, `display_name`, téléphones, `email`, `trade_activity`, `secondary_contact`, `id_document_enc` présent et inutilisé, `merged_into_id`, archivage, `row_version`) ; complément de `unit_assignments` avec clé étrangère composite vers `units`.
2. Colonnes de recherche normalisées (minuscules, sans accents) et index `(org_id, …)` ; index unique partiel sur le téléphone parmi les fiches non archivées (R-017).
3. Clés étrangères composites `(org_id, id)` ; RLS forcée ; `row_version` sur `properties`, `units`, `tenants`, `owners` ; les gardes de mise à jour laissent `row_version` modifiable (cas de test noté en `DATABASE_SCHEMA.md` §1.2).
4. Colonne `import_batch_id` : ajoutée par la migration de l'étape 12, pas ici.

### 5.2 Noyau métier
1. `crates/domain` : normalisation et validation du téléphone `+226`, champs applicables selon le type de lot (R-012), règle de fusion.
2. `property` : `create_property` (avec `single_unit` ⇒ 1 bien + 1 lot, R-010), `create_unit`, `update`, `archive` (R-005) ; `lease::tenants` : création, modification, fusion, archivage ; `owners` : création, modification ; hook de création d'organisation du plan Propriétaire ⇒ fiche `is_self` (R-016).
3. Recherche unique : insensible aux accents et à la casse, téléphone reconnu avec ou sans `+226`, huit résultats au plus par groupe, portée limitée aux lots assignés pour l'agent terrain (R-081).
4. Audit : `tenant.created`, `tenant.updated`, `tenant.merged`, `tenant.archived`, `unit.archived`, `owner.updated`, `unit.works_set` (le geste « en travaux » est posé à l'étape 06).

### 5.3 API
`GET/POST /properties`, `GET/PATCH /properties/{id}`, `POST /properties/{id}/archive`, `GET/POST /units`, `GET/PATCH /units/{id}`, `POST /units/{id}/archive`, `GET/POST /tenants`, `GET/PATCH /tenants/{id}`, `POST /tenants/{id}/merge`, `POST /tenants/{id}/archive`, `GET /tenants/{id}/id-document` (répond `feature.disabled`), `GET/POST /owners`, `GET/PATCH /owners/{id}`, `GET /search`, complétion de `PUT /users/{membership_id}/unit-assignments`. Pagination par curseur, filtres de la liste blanche, champs masqués selon le rôle (§4.5), `If-Match` sur tous les `PATCH`.

### 5.4 Interface
S06 (liste des biens avec occupation, création bien + lots, « maison seule »), S07 (en-tête et onglet Aperçu ; onglets Baux, Paiements, Travaux et Documents visibles mais vides avec leur état vide), S08 (liste, création, fusion des doublons, archivage), S09 (coordonnées, onglets, boutons Appeler et WhatsApp désactivés tant que non utilisables), liste des mandants dans le formulaire de bien, barre de recherche unique, sélecteur de lots assignés dans S18, états vide/chargement/erreur/succès, filtres mémorisés côté front, cartes sur mobile et tableaux sur ordinateur.

### 5.5 Documents PDF
Aucun.

### 5.6 Audit et événements
Voir 5.2. Événements : `screen_viewed` (S06 à S09), `search_performed` (longueur, groupe choisi, aucun résultat), `validation_error`.

### 5.7 Tests
- Règles : R-010 (maison seule), R-011, R-012 (champs selon le type), R-017 (doublon : proposition, pas de création silencieuse), R-018, R-019 (`feature.disabled`), R-005 (un bien archivé garde son historique visible).
- Base : unicité de la référence par bien, téléphone unique parmi les non archivées, clés composites (un lot d'une organisation ne peut pas viser le bien d'une autre : ISO-3), `row_version`.
- API : A3 (rôle × action), A4 (identifiant d'une autre organisation ⇒ 404), A5 (agent terrain : lots assignés visibles, non assignés introuvables, y compris dans la recherche), A10 (téléphone invalide, champs inconnus refusés), 412 `concurrency.version_mismatch` sur `PATCH` périmé, recherche « ouedraogo » trouve « Ouédraogo ».
- E2E 390 et 1440 px : créer un bien et deux lots, créer un locataire, provoquer et fusionner un doublon, rechercher.
- Propriété P16 : toute valeur de montant d'un lot (`reference_rent`) est un entier dans les bornes.

## 6. Critères d'acceptation vérifiables

1. Créer une maison seule produit exactement 1 bien et 1 lot ; deux lots de même référence dans un bien sont refusés avec `unit.reference_duplicate`.
2. Un lot « boutique » propose « activité autorisée » ; un lot d'habitation ne la propose pas.
3. Créer un locataire avec un téléphone existant propose d'ouvrir la fiche existante ; `70123456`, `+226 70 12 34 56` et `0022670123456` désignent la même fiche ; 7 chiffres est refusé avec un message explicite.
4. La pièce d'identité n'est ni saisissable ni lisible (`feature.disabled`).
5. Un agent terrain assigné à 2 lots sur 5 ne voit que ces 2 lots (liste, fiche, recherche).
6. Deux gestionnaires qui modifient la même fiche : le second reçoit un conflit de version, aucune saisie n'est écrasée en silence.
7. Un lot ou un locataire archivé reste consultable avec son historique ; rien n'est supprimé.
8. Le caissier voit les fiches en lecture, sans contact de secours ni notes.

## 7. Sécurité et déploiement à vérifier

Un test d'isolation par route ajoutée (couverture 100 % du registre) ; champs sensibles masqués selon le rôle ; recherche de l'agent terrain bornée ; migrations rétrocompatibles exécutées sur l'essai ; aucune donnée réelle dans les fixtures (bloc fictif `+226 00 00 …`).

## 8. Liste de démonstration (Mimis)

1. Créer le bien « Zogona » avec un lot d'habitation « A1 » et une boutique « B1 » ; remarquer le champ « activité autorisée » sur la boutique.
2. Créer une maison seule « Villa Kaboré » : voir 1 bien et 1 lot.
3. Créer le locataire « Issouf Tapsoba » (`+226 00 00 01 01`), puis tenter un second locataire avec le même numéro : lire la proposition d'ouvrir la fiche ; fusionner.
4. Taper « tapsoba » puis « 00000101 » dans la recherche : retrouver la fiche dans les deux cas.
5. Se connecter avec un agent terrain assigné à un seul lot : ne voir que ce lot.

## 9. Définition de terminé, risques, hors étape

Terminé : critères 1 à 8 verts ; routes couvertes (isolation, droits, contrat) ; événements ajoutés à la liste blanche ; `API_CONTRACTS.md` §9 et `DATABASE_SCHEMA.md` à jour ; `DECISIONS.md` (n° 43, 45) mis à jour ; `CLAUDE.md` à jour.

| Risque | Mesure |
|---|---|
| Colocation ou téléphone partagé plus fréquent que prévu | R-017 est [à confirmer avec l'agence pilote à la découverte] ; la fusion reste la voie de sortie |
| Types de lots ou champs commerciaux manquants | Ajout d'une valeur par `CHECK` remplacé sans verrou long (`DATABASE_SCHEMA.md` §7.2) |
| Recherche lente sans `pg_trgm` | Budget 150 ms mesuré sur 500 lots ; `pg_trgm` reporté tant que tenu |

Hors étape : baux, échéances, statut d'occupation (étape 06) ; mandats et taux (étape 16) ; documents et travaux d'un lot (étape 20) ; `import_batch_id` (étape 12).

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 05
- Dépend de : 02
- Tranche : T1
- Jalon : J2
- F : F04, F06, F05
- R : R-005, R-010, R-011, R-012, R-016, R-017, R-018, R-019, R-094
- E : aucun
- PP : PP-09
- Contenus : C-09
