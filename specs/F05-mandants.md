# F05 — Fiche mandant (partie livrée à l'étape 05)

| Champ | Valeur |
|---|---|
| Statut | **Rédigée le 03/11/2026, validée par Mimis le 03/11/2026** (recommandations du §8 acceptées, DP-92) |
| Étape | 05, `roadmap/etape-05-biens-lots-locataires.md` §4 point 3 ; la suite de F05 (mandats, taux, lots couverts, coordonnées de paiement) viendra à l'étape 16 et complétera cette fiche |
| Dépend de | dossier §16 F05 ; `DATABASE_SCHEMA.md` `owners` ; `API_CONTRACTS.md` §4.5, §4.6, §5.2, §7.8 ; DP-85 (n° 45) |
| Règles | R-005, R-016, R-018 |
| Critères visés | critères 1 (propriétaire du bien) et 7 de l'étape 05 ; prépare l'import (étape 12) |
| Hors fiche | `mandates`, `mandate_units`, `mandate_rate_changes`, commission (R-014, R-015, R-069) ; coordonnées de paiement `payout_details_enc` et `GET /owners/{id}/payout-details` ; comptes rendus (étape 17) |

## 1. Objet

Savoir à qui appartient chaque bien : la fiche du **mandant** (propriétaire pour qui l'agence gère) sert de propriétaire par défaut d'un bien (`properties.default_owner_id`), et l'import (étape 12) la remplira. Dans le plan Propriétaire, le propriétaire lui-même a sa fiche, créée avec l'organisation.

## 2. Règles

| N° | Règle | Source |
|---|---|---|
| 1 | Un mandant a un type (`person`, `company`), un nom affiché (2 à 160 caractères), un téléphone facultatif (`+226` et 8 chiffres, même normalisation que F06) et un mode de reversement facultatif (`cash`, `mobile_money`, `bank_transfer`, `cheque`) | schéma, dossier F05 |
| 2 | **Pas d'unicité du téléphone** entre mandants (un même numéro peut servir à deux mandants d'une famille) [proposition, question 3] | aucune règle écrite |
| 3 | **Plan Propriétaire** : `create_organization()` crée, dans sa transaction, une fiche `is_self = true`, type `person`, nom = nom de l'organisation [proposition, question 2]. Une seule fiche `is_self` non archivée par organisation (index du schéma). La migration crée la fiche des organisations du plan Propriétaire qui existent déjà. Un nouveau bien du plan Propriétaire prend cette fiche comme propriétaire par défaut | R-016 |
| 4 | **Plan Propriétaire** : pas d'autre mandant (les mandats n'y existent pas) ; `POST /owners` répond `403 plan.feature_unavailable`. La fiche `is_self` se modifie (nom, téléphone) par le rôle `owner` ; elle ne s'archive pas | R-016, R-082 |
| 5 | **Plan Agence** : création et modification par le **gérant** (`mandate.write` ; dossier : « gérant uniquement ») ; aucune fiche `is_self` | dossier F05, §4.6 |
| 6 | **Archivage** (jamais de suppression) : refusé tant que le mandant est propriétaire par défaut d'un bien non archivé (`409 owner.has_properties`, avec le nombre de biens) [proposition, question 4] ; à l'étape 16, refusé aussi tant qu'un mandat est actif. Un mandant archivé reste lisible et ne s'affiche plus dans le choix du propriétaire d'un bien | R-005 |
| 7 | Modification concurrente : `version`, `ETag`, `If-Match` | n° 64 |
| 8 | Audit dans la transaction : `owner.created`, `owner.updated`, `owner.archived` | étape §5.2 |

## 3. Données (migration de l'étape)

1. `owners` **comme dans `DATABASE_SCHEMA.md`** (y compris `payout_details_enc`, vide jusqu'à l'étape 16), sans `import_batch_id` (étape 12) ; RLS forcée ; clé composite `(org_id, id)` pour `properties.default_owner_id`.
2. Contrôle ajouté : `NOT (is_self AND archived_at IS NOT NULL)` (la fiche du propriétaire direct ne s'archive pas).
3. `create_organization()` modifiée (plan `owner` : insertion de la fiche `is_self`), avec complément pour les organisations existantes dans la même migration. C'est une fonction de garde de l'étape 02 : **second avis C-19** à ajouter à la liste.
4. Droits de `immo_app` : SELECT, INSERT, UPDATE par colonne ; aucun DELETE.

## 4. API (`API_CONTRACTS.md` §7.8, partie fiche)

| Route | Droit (proposé, question 1) | Corps → réponse | Erreurs propres |
|---|---|---|---|
| `GET /owners` | `catalog.read` | filtres `q`, `archived` → liste (nom, type ; téléphone et mode de reversement pour `mandate.write` seulement) | |
| `POST /owners` | `mandate.write` (plan Agence) | `{ kind, display_name, phone?, payout_method? }` → `201` mandant | `validation.phone_format`, `plan.feature_unavailable` |
| `GET /owners/{id}` | `catalog.read` | fiche, `version`, biens dont il est propriétaire par défaut | `404` |
| `PATCH /owners/{id}` | `mandate.write` ; rôle `owner` sur sa fiche `is_self` | champs ; `If-Match` | `concurrency.*`, `owner.archived` |
| `POST /owners/{id}/archive` | `mandate.write` | → mandant archivé | `owner.has_properties`, `owner.is_self` |

## 5. Interface

- Pas d'écran « Mandants » complet à cette étape : il vient avec les mandats (étape 16). Livré : le **choix du propriétaire** dans le formulaire de bien (S06), avec recherche, et pour le gérant d'agence « Créer un mandant » (feuille : type, nom, téléphone, mode de reversement).
- Fiche bien : nom du propriétaire. Plan Propriétaire : le champ n'apparaît pas (c'est toujours la fiche `is_self`) ; son nom se modifie dans les paramètres de l'organisation [proposition].

## 6. Cas limites

| Cas | Comportement |
|---|---|
| Organisation du plan Propriétaire créée par `immo org create --plan owner` | une fiche `is_self` au nom de l'organisation |
| Organisation du plan Propriétaire déjà en base | la migration lui crée sa fiche `is_self` |
| `POST /owners` dans le plan Propriétaire | `403 plan.feature_unavailable` |
| Gestionnaire d'agence qui crée un mandant | `403 auth.forbidden` (gérant seul) |
| Archivage d'un mandant propriétaire de 2 biens | `409 owner.has_properties` (« Ce mandant est propriétaire de 2 biens. Changez leur propriétaire avant de l'archiver. ») |
| Archivage de la fiche `is_self` | `409 owner.is_self` |
| Bien d'une organisation A qui vise un mandant de B | `404 owner.not_found` (clé composite) |

## 7. Tests (écrits avant le code)

- Base : `is_self` créé par `create_organization()` en plan `owner`, jamais en plan `agency` ; une seule par organisation ; non archivable ; clé composite ; aucun `DELETE`.
- API : droits (A3, par rôle et par plan) ; `plan.feature_unavailable` ; archivage refusé avec biens ; A4 ; `412`.

## 8. Questions pour Mimis

1. **Qui voit et qui crée un mandant ?** Le contrat d'API donne `GET /owners` au droit `statement.read` (gérant ; gestionnaire et comptable en lecture), que le caissier, l'agent terrain et tout le plan Propriétaire n'ont pas. Or le formulaire de bien du gestionnaire doit lister les mandants, et le dossier F04 laisse le gestionnaire « changer le propriétaire par défaut ». *Recommandation : liste (nom, type) lisible par `catalog.read` ; téléphone et mode de reversement réservés à `mandate.write` ; création, modification et archivage au gérant seul (`mandate.write`), comme le dossier F05 ; choix du propriétaire d'un bien par `catalog.write` (gestionnaire compris).* `API_CONTRACTS.md` §7.8 est à corriger en conséquence.
2. **Nom de la fiche `is_self`** : *recommandation : le nom de l'organisation*, modifiable. Autre option : le nom de l'utilisateur qui crée l'organisation.
3. **Téléphone d'un mandant** : *recommandation : non unique* (aucune règle ne l'exige, et une famille peut partager un numéro). Autre option : la même règle que les locataires (R-017).
4. **Archivage d'un mandant propriétaire de biens** : *recommandation : refuser tant qu'il est propriétaire par défaut d'un bien non archivé*. Autre option : accepter ; les biens gardent un propriétaire archivé, affiché comme tel.
