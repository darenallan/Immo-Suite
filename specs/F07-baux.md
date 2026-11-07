# F07 — Baux (étape 06, lot 06a)

| Champ | Valeur |
|---|---|
| Statut | **Rédigée le 04/11/2026, validée par Mimis le 04/11/2026** (recommandations du §10 acceptées, DP-94) |
| Étape | 06, `roadmap/etape-06-baux-et-echeancier.md` §4 point 1 |
| Dépend de | dossier §16 F07 ; DP-24, DP-25, DP-26, DP-39, DP-41, DP-58 ; décisions n° 41, 42 ; `DATABASE_SCHEMA.md` `leases`, `lease_tenants`, `lease_commercial_terms`, `lease_amendments`, §4.1, §4.4, §4.7 ; `ARCHITECTURE.md` §4.6, §4.7 ; `API_CONTRACTS.md` §7.4, §7.5 ; `UI_UX_DESIGN.md` S10, S07, « Statuts financiers » |
| Règles | R-006, R-013, R-020 à R-026, R-090 (statuts), R-091, R-092, R-093, R-104 [à confirmer avec l'agence pilote à la découverte] |
| Cas chiffrés | E-01, E-02, E-22, E-35, E-36, E-44 |
| Hors fiche | échéancier et statut d'une échéance (F08) ; préavis et départ (étape 21, les colonnes existent) ; restitution du dépôt (étape 21) ; indexation automatique (F30, hors MVP) ; fiche de bail imprimable |

## 1. Objet

Formaliser la relation lot–locataire(s) et la base de calcul des échéances : un bail se crée en brouillon, s'active (il produit alors ses échéances et le lot devient « occupé »), évolue par avenants datés jamais rétroactifs, et se termine. Rien ne se supprime.

## 2. Règles

| N° | Règle | Source |
|---|---|---|
| 1 | Statuts : `draft` (brouillon), `active`, `notice` (préavis, étape 21), `ended` (terminé). Seul un brouillon se modifie librement ; après activation, un changement passe par un avenant, une résiliation, ou (étape 21) un préavis | DP-24, R-090 |
| 2 | Un bail porte : un lot, **un ou plusieurs locataires** dont un principal (colocation : un seul bail), un type `residential` ou `commercial`, une date de début, une date de fin facultative (bail sans fin), un loyer, des charges (0 par défaut), un jour d'échéance 1 à 31, un dépôt de garantie (0 par défaut) | dossier F07, schéma |
| 3 | **Montants entiers FCFA** : loyer de 1 à 999 999 999 (`validation.rent_positive` pour 0 ou moins) ; charges et dépôt de 0 à 999 999 999 | R-001, R-022 |
| 4 | Date de fin ≥ date de début, sinon `validation.date_order` | R-021 |
| 5 | Jour d'échéance absent : le 1ᵉʳ, signalé (`due_day_defaulted`, avertissement `lease.due_day_defaulted`) | R-031 |
| 6 | **Dépôt supérieur à deux mois de loyer** (dépôt > 2 × loyer, charges exclues) sur un bail d'habitation : avertissement **non bloquant** `lease.deposit_above_two_months` à la création, à la modification et à l'activation | R-024 [À VÉRIFIER AVEC UN JURISTE], E-22 |
| 7 | **Type du bail et usage du lot** : lot `residential` → bail `residential` ; lot `commercial` → bail `commercial` ; lot `mixed` → l'un ou l'autre, choisi à la création [proposition, question 1] | R-094, DP-25 |
| 8 | Bail commercial : durée en mois (1 à 600) et note d'indexation facultatives (`lease_commercial_terms`) ; case « TVA incluse » (`vat_included`) sans aucun calcul ; refusée sur un bail d'habitation. Alerte « révision à étudier » : affichée si une note d'indexation existe, aucune indexation automatique | R-025, R-026, R-092 |
| 9 | **Activation** (`POST /leases/{id}/activate`), sous verrou du lot : refus si le bail n'est pas un brouillon (`lease.not_draft`), si le lot est « en travaux » (`unit.under_works`), archivé (`unit.archived`), ou si un bail actif, en préavis **ou terminé** du même lot chevauche ses dates (`lease.overlap`, avec les dates du bail en cause, bornes incluses) ; sinon le bail passe `active`, `activated_at` et `activated_by` sont posés, les échéances sont générées dans la même transaction (F08, R-028), le lot est « occupé » (R-013, statut dérivé). Rejouée avec la même clé d'idempotence : même réponse, rien de plus | R-020, R-093, DP-39, n° 41, E-36 |
| 10 | Un locataire archivé ou fusionné ne peut pas être ajouté à un bail ; les locataires d'un bail sont figés à l'activation (`lease_tenants_guard`). **Changer de locataire = nouveau bail** | R-091 |
| 11 | **Avenant** (`POST /leases/{id}/amendments`) sur un bail actif : date d'effet, nouveau loyer et/ou nouvelles charges et/ou nouvelle date de fin (prolongation). Il s'applique aux échéances dont la date est **postérieure à la fois à sa date d'effet et à sa date de saisie** ; les échéances échues gardent l'ancien montant. Refus si une échéance dont la date est ≥ à la date d'effet a déjà reçu un paiement (`lease.amendment_retroactive`, garde de base). Deux avenants à la même date d'effet : refus | R-023, R-104, n° 42, E-02, E-44 |
| 12 | **Prolongation** = avenant qui recule la date de fin (jamais l'avance : `validation.date_order`) ; même bail, même dépôt, même historique ; les échéances jusqu'à la nouvelle fin sont générées par la fenêtre de 30 jours (F08) | R-091, E-35 |
| 13 | **Résiliation** (`POST /leases/{id}/terminate`) : date (≥ date de début) et motif obligatoires ; le bail passe `ended`, plus aucune échéance après la date (R-030) ; le lot redevient « vacant » | dossier F07, R-030 |
| 14 | **Archivage d'un lot** ou d'un **locataire** avec un bail actif ou en préavis : refusé (`unit.has_active_lease`, `lease.active_exists`), qui complète l'étape 05 | R-006 |
| 15 | **« En travaux »** (`POST /units/{id}/works`) : se pose sur un lot vacant seulement (`unit.works_requires_vacant`), se retire à la main ; audit `unit.works_set` | R-093, DP-25 |
| 16 | **Statut d'un lot** : `occupied` si un bail actif ou en préavis le couvre aujourd'hui (fuseau de l'organisation), sinon `under_works`, `archived` ou `vacant` — calculé à la lecture | R-013 |
| 17 | Droits : `catalog.read` pour lire (agent terrain : baux de ses lots ; il voit désormais les locataires de ces baux), `catalog.write` pour écrire (gérant, gestionnaire, propriétaire) | §4.6, R-081 |
| 18 | Audit dans la transaction : `lease.created`, `lease.updated` (brouillon), `lease.activated`, `lease.amended`, `lease.ended`, `unit.works_set` | étape §5.2 |

## 3. Données (migration de l'étape)

1. `leases`, `lease_tenants`, `lease_commercial_terms`, `lease_amendments` **comme dans `DATABASE_SCHEMA.md`** (contraintes, `effective_end` générée, exclusion `leases_no_overlap` sur `active`, `notice`, `ended` avec l'extension `btree_gist`, `row_version`), sans `import_batch_id` (étape 12).
2. `leases_guard` : en brouillon tout se modifie ; ensuite seuls changent l'état (activation, résiliation, préavis), `end_date` vers une date plus lointaine (avenant de prolongation) et `row_version`. Refuse l'activation sur un lot en travaux ou archivé.
3. `lease_amendments_guard` du schéma §4.4 ; la partie « paiement » s'appuie sur `allocations`, qui n'existe qu'à l'étape 07 : à l'étape 06, la garde vérifie seulement le statut du bail et la date d'effet, la partie paiement est ajoutée avec `allocations` [question 4].
4. `units_works_guard` (schéma §4.7) : « en travaux » et archivage refusés si un bail est en cours.
5. Droits `immo_app` : `leases` SELECT, INSERT, UPDATE par colonne ; `lease_tenants`, `lease_amendments` SELECT, INSERT ; `lease_commercial_terms` SELECT, INSERT, UPDATE. Aucun DELETE. Second avis C-19 : exclusion, gardes.

## 4. API (`API_CONTRACTS.md` §7.5, §7.4)

| Route | Droit | Corps → réponse | Erreurs propres |
|---|---|---|---|
| `GET /leases` | `catalog.read` | filtres `status`, `unit_id`, `tenant_id`, `ending_before` → liste (locataire principal, lot, statut, loyer en vigueur, alertes) | |
| `POST /leases` | `catalog.write` | `{ unit_id, tenant_ids[], primary_tenant_id?, kind, start_date, end_date?, rent, charges?, due_day?, deposit_amount?, vat_included?, commercial_terms? }` → `201` brouillon + `warnings[]` | `validation.*`, `lease.kind_not_allowed` |
| `GET /leases/{id}`, `PATCH /leases/{id}` | `catalog.read`, `catalog.write` | fiche, `version`, avenants, conditions en vigueur ; `PATCH` seulement en brouillon (`If-Match`) | `lease.not_draft`, `concurrency.*` |
| `POST /leases/{id}/activate` | `catalog.write` | → bail actif + échéances générées ; `Idempotency-Key` recommandée | `lease.overlap`, `unit.under_works`, `unit.archived`, `lease.not_draft` |
| `POST /leases/{id}/amendments` | `catalog.write` | `{ effective_from, rent?, charges?, new_end_date?, note? }` → bail + `warnings[]` | `lease.amendment_retroactive`, `lease.not_in_progress` |
| `POST /leases/{id}/terminate` | `catalog.write` | `{ terminated_on, reason }` → bail terminé | `lease.not_in_progress` |
| `GET /units/{id}/leases` | `catalog.read` | onglet Baux de S07 | |
| `POST /units/{id}/works` | `catalog.write` | `{ under_works }` → lot | `unit.works_requires_vacant` |

Les dates sont `AAAA-MM-JJ` sur le fil ; les avertissements ne bloquent jamais la réponse.

## 5. Interface

- **S10 Baux** : liste (locataire, lot, statut, loyer, prochaine échéance, alertes). **Création en trois étapes** : (1) lot et locataires (recherche, colocation, principal) ; (2) type, dates, loyer, charges, jour d'échéance, dépôt, champs commerciaux si le type est commercial ; (3) vérification, avertissements, [Enregistrer le brouillon] [Activer]. Fiche bail : statut (pastille neutre), conditions en vigueur, échéancier (F08), avenants, [Ajouter un avenant], [Résilier] (motif obligatoire, modale).
- **S07** : onglet Baux rempli, bouton « Mettre en travaux » / « Retirer « en travaux » » sur un lot vacant. **S09** : onglet Baux rempli ; la fiche locataire montre ses lots.
- Messages : chevauchement avec les dates en cause ; lot en travaux avec [Retirer « en travaux »] ; dépôt : avertissement jaune non bloquant. Aucune règle d'argent dans le front.

## 6. Cas limites

| Cas | Comportement |
|---|---|
| Bail du 01/01/2026 au 31/12/2026 terminé, nouveau bail du 31/12/2026 | refusé (`lease.overlap`, bornes incluses) |
| Même cas, nouveau bail du 01/01/2027 | accepté |
| Activation rejouée avec la même clé | même réponse, aucune échéance en double |
| Activation sur un lot « en travaux » | `unit.under_works` ; acceptée après retrait (E-36) |
| Dépôt de 200 000 pour un loyer de 75 000 (habitation) | avertissement, bail créé (E-22) |
| Même dépôt sur un bail commercial | aucun avertissement (R-024 vise l'habitation) |
| Loyer 0, dates inversées | `422` (`validation.rent_positive`, `validation.date_order`) |
| Deux locataires (colocation), un seul principal | accepté ; sans principal désigné, le premier de la liste |
| Avenant à 80 000 d'effet au 01/04, saisi le 15/03 | mars reste à 75 000, avril à 80 000 (E-02) |
| Avenant d'effet au 01/12 saisi le 15/12, échéance du 05/12 impayée | décembre reste à 75 000 ; le 05/01 passe à 80 000 (E-44) |
| Prolongation du 31/12 au 30/06 suivant | même bail, échéances jusqu'en juin au fil de la fenêtre, dépôt inchangé (E-35) |
| Archiver un lot à bail actif | `unit.has_active_lease` (R-006) |
| Modifier un bail actif par `PATCH` | `lease.not_draft` |

## 7. Tests (écrits avant le code)

- Tests d'or `tests/fixtures/golden/E-01.json`, `E-02`, `E-22`, `E-35`, `E-36`, `E-44` : domaine, puis base, puis API (horloge fixée).
- Base : exclusion G-01, G-02 (contre un bail terminé), garde de brouillon, avenants en ajout seul (G-10, G-18), `UPDATE` refusé à `immo_app` sur `lease_amendments`, clés composites.
- API : A3, A4, A8 (activation idempotente), A10 (loyer nul, dates inversées), `412` sur `PATCH`, `lease.not_draft`.
- E2E 390 et 1440 px : bail de 75 000 au 5, activation, échéancier, avenant à 80 000.

## 8. Ce qui sera vérifié sur le poste, et ce qui ne le sera pas

- Migrations sur l'essai : non (inexistant) ; contrainte d'exclusion et gardes : second avis C-19 avant la production.
- Avis juridique sur le dépôt (R-024) et le bail commercial (R-025) : non ; les messages restent des avertissements.

## 9. Écarts aux documents propriétaires

- Codes nouveaux : `lease.kind_not_allowed`, `lease.not_in_progress`, `lease.due_day_defaulted` (avertissement), `lease.active_exists` (archivage d'un locataire).
- `PATCH /leases/{id}` limité au brouillon : déjà écrit dans le contrat (`lease.not_draft`).

## 10. Questions pour Mimis

1. **Type de bail et usage du lot** : *recommandation : un lot d'habitation n'accepte qu'un bail d'habitation, un lot commercial qu'un bail commercial, un lot mixte les deux* (`lease.kind_not_allowed` sinon). Autre option : libre, avec un simple avertissement.
2. **Locataire principal** : *recommandation : le premier de la liste si l'utilisateur n'en désigne pas*. C'est à lui que s'adresseront la quittance et la relance (étapes 08 et 15).
3. **Dépôt « supérieur à deux mois »** : *recommandation : dépôt > 2 × loyer, charges exclues*. Autre option : loyer + charges.
4. **Garde de l'avenant « déjà payé »** : sans paiements avant l'étape 07, elle ne peut rien refuser. *Recommandation : la poser à l'étape 07 avec la table des affectations* (le test G-10 y sera écrit) ; à l'étape 06, la garde vérifie le statut du bail et la date d'effet (après le début).
