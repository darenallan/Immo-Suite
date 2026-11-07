# F08 — Échéancier automatique (étape 06, lot 06b)

| Champ | Valeur |
|---|---|
| Statut | **Rédigée le 04/11/2026, validée par Mimis le 04/11/2026** (recommandations du §10 acceptées, DP-94) |
| Étape | 06, `roadmap/etape-06-baux-et-echeancier.md` §4 point 2 |
| Dépend de | dossier §16 F08 ; DP-13, DP-27, DP-50, DP-51, DP-58 ; décision n° 33 ; `DATABASE_SCHEMA.md` `installments`, `installment_lines`, `installment_line_voids`, §5 (vues de soldes) ; `ARCHITECTURE.md` §3.9, §4.6, §4.8, §4.9 ; `UI_UX_DESIGN.md` « Statuts financiers » |
| Règles | R-027 à R-036, R-095, R-098, R-099, R-100 [ces trois : à confirmer avec l'agence pilote à la découverte], R-104 |
| Cas chiffrés | E-01, E-02, E-03, E-04, E-37, E-39, E-40, E-44 |
| Hors fiche | paiements, affectations et crédit du bail (étape 07 : la vue `v_lease_credit` et l'affectation du crédit à la génération, R-086) ; impayés et tranches (étape 10) ; préavis (étape 21) |

## 1. Objet

Produire sans saisie l'échéancier de chaque bail en cours, et lui donner un statut exact (à venir, due, partielle, payée, en retard) calculé par **une seule fonction** du domaine, appelée par toutes les lectures. Aucun statut n'est stocké : il dépend de la date du jour.

## 2. Règles

| N° | Règle | Source |
|---|---|---|
| 1 | **Une échéance par bail et par période** (mois civil, `period` = 1ᵉʳ du mois). Unicité en base `(org_id, lease_id, period)` ; écriture seule | R-027, schéma |
| 2 | **Date d'échéance** de la période : le jour d'échéance du bail ; s'il n'existe pas dans le mois (29, 30, 31), le **dernier jour** du mois (février 2027, jour 31 → 28/02/2027 ; février 2028 → 29/02/2028) | R-098, E-39 |
| 3 | **Lignes** d'une échéance : `rent` (loyer en vigueur) et `charges` (si non nulles), plus les lignes ajoutées à la main `arrears` (arriéré daté, R-035) et `exceptional` (libellé obligatoire). Montant dû = somme des lignes non annulées | R-035, schéma |
| 4 | **Montant en vigueur** : le loyer et les charges du dernier avenant applicable à cette échéance (R-104 : date d'échéance postérieure à sa date d'effet **et** à sa date de saisie), sinon ceux du bail. La ligne garde la référence de l'avenant (`amendment_id`) | R-023, R-104, E-02, E-44 |
| 5 | **Premier mois** : dû en entier, sans prorata, même pour un bail activé en cours de mois | R-099, E-40 |
| 6 | **Première période** : le mois de la date de début, ou le mois de l'activation si la date de début est passée (les périodes antérieures se saisissent en lignes d'arriéré, R-036) | R-100, E-01 |
| 7 | **Dernière période** : pas d'échéance dont la période commence après la fin effective du bail (résiliation, sinon date de fin) ; bail sans fin : aucune limite [question 2] | R-030 |
| 8 | **Fenêtre** : toutes les périodes dont la date d'échéance tombe d'ici **30 jours** (aujourd'hui inclus, fuseau de l'organisation) ; les périodes déjà échues depuis la première période sont aussi créées | R-029 |
| 9 | **Génération** : (a) dans la transaction de l'activation (R-028) ; (b) chaque jour à **00 h 05** au fuseau de l'organisation par la file de tâches (`immo work`), clé d'unicité par organisation et par jour, `INSERT … ON CONFLICT DO NOTHING` ; rejouée, elle ne crée rien (E-03) ; plusieurs travailleurs : aucun doublon ; (c) après un avenant de prolongation (E-35). Si aucune génération n'a réussi avant 02 h 00 : alerte `installments.generation_late` | R-029, §3.9, §4.8 |
| 10 | **Avenant sur une échéance déjà générée et non échue** (fenêtre de 30 jours) : la ligne `rent` (ou `charges`) est **annulée avec le motif « avenant du JJ/MM/AAAA »** et une nouvelle ligne au nouveau montant est ajoutée, dans la transaction de l'avenant ; rien n'est modifié en place [question 1] | R-023, R-104, écriture seule |
| 11 | **Statut** = `installment_status(dû, affecté, date d'échéance, aujourd'hui)` : « payée » si affecté ≥ dû ; sinon « en retard » dès le lendemain de la date d'échéance (J+1), même partiellement payée ; sinon « partielle » si affecté > 0 ; sinon « due » le jour même (J0) ; sinon « à venir ». `days_late` = jours depuis la date d'échéance. Rien de ce calcul n'est en SQL : la base ne renvoie que des montants (vue `v_installment_balances`) | R-032, R-033, R-095, DP-13, DP-27, DP-51, E-04, E-37 |
| 12 | **Annulation d'une ligne** (`POST /installment-lines/{id}/void`) : motif obligatoire (3 caractères au moins), par le gérant ou le gestionnaire ; refusée si la ligne porte un paiement net (`installment.line_has_payments`, garde posée à l'étape 07) | R-036 |
| 13 | **Ligne ajoutée** (`POST /leases/{id}/installment-lines`) : `arrears` ou `exceptional`, libellé, montant entier ≥ 1, date ; rattachée à l'échéance de sa période (créée si besoin pour une période antérieure : arriéré) | R-035, R-036 |
| 14 | Droits : lecture `catalog.read` (agent terrain : baux de ses lots) ; ajout et annulation de ligne `catalog.write` | §4.6 |
| 15 | Audit : `installment.line_added`, `installment.line_voided` ; la génération automatique n'écrit pas d'audit par échéance (tâche de plateforme, journal `tracing`) | étape §5.2 |

## 3. Données (migration de l'étape)

1. `installments`, `installment_lines`, `installment_line_voids` **comme dans `DATABASE_SCHEMA.md`** : écriture seule (`forbid_mutation`, `forbid_truncate`), `installments_due_in_period`, montants bornés ; `immo_app` : SELECT, INSERT seulement.
2. **Écart nécessaire (question 1)** : l'index unique `installment_lines_one_rent_charges (org_id, installment_id, kind)` interdirait la nouvelle ligne de loyer d'un avenant ; il est remplacé par `(org_id, installment_id, kind, coalesce(amendment_id, '00000000-0000-0000-0000-000000000000'))` : une ligne de loyer par échéance **et par version des conditions**. La garde de génération empêche toujours deux lignes de loyer actives.
3. Vue `v_installment_balances` (dû, affecté, reste) **sans règle de date** ; tant que `allocations` n'existe pas (étape 07), l'affecté vaut 0.
4. Aucune colonne de statut d'échéance (test de schéma, critère 10).

## 4. API

| Route | Droit | Réponse | Erreurs propres |
|---|---|---|---|
| `GET /leases/{id}/installments` | `catalog.read` | `[{ id, period, due_date, lines[], due_amount, allocated_amount, remaining_amount, status, days_late }]`, lignes annulées comprises (barrées) | |
| `POST /leases/{id}/installment-lines` | `catalog.write` | `{ kind: "arrears" ou "exceptional", label, amount, due_date }` → ligne | `validation.*` |
| `POST /installment-lines/{id}/void` | `catalog.write` | `{ reason }` → ligne annulée | `installment.line_has_payments` (étape 07) |

`status` et `days_late` viennent de la fonction du domaine, calculés avec la date du jour de l'organisation ; le front ne calcule rien.

## 5. Interface

Échéancier dans la fiche bail (S10) et dans les onglets Baux de S07 et S09 : mois, date, montant dû, payé, reste, **badge** (icône + libellé + couleur : « À venir », « Due aujourd'hui », « Partiel · payé … · reste … », « En retard · 7 jours », « Payé ») ; lignes annulées barrées avec leur motif ; [Ajouter une ligne] (arriéré ou exceptionnelle) et [Annuler la ligne] (motif). Signalement « jour d'échéance par défaut : le 1ᵉʳ ».

## 6. Cas limites

| Cas | Comportement |
|---|---|
| Bail à 75 000, jour 5, activé le 01/03 | échéance de mars au 05/03, 75 000 (E-01) |
| Tâche exécutée deux fois le même jour, ou par deux travailleurs | une seule échéance par période (E-03) |
| Échéance du 05 non payée | le 04 « à venir », le 05 « due », le 06 « en retard » (E-37, E-04) |
| Jour 31, février 2027 | 28/02/2027 (E-39) ; février 2028 : 29/02/2028 |
| Bail activé le 20/03 | mars dû en entier, 75 000 (E-40) |
| Bail commencé le 01/01, activé le 10/03 | première échéance : mars ; janvier et février en arriérés à la main (R-100) |
| Avenant à 80 000 d'effet 01/04 | mars 75 000, avril 80 000 (E-02) |
| Avenant d'effet 01/12 saisi le 15/12 | 05/12 reste à 75 000 ; 05/01 (déjà générée) : ligne de loyer annulée « avenant du 15/12/2026 » et nouvelle ligne à 80 000 (E-44) |
| Bail résilié le 15/04 | plus d'échéance après avril |
| Échéance dont toutes les lignes sont annulées | dû 0 : question 3 |

## 7. Tests (écrits avant le code)

- **Domaine d'abord** : `due_date_for(period, due_day)` (R-098, années bissextiles), `installment_status` et `days_late` (P9 : propriété sur le triplet dû, affecté, aujourd'hui), `periods_to_create` (R-027, R-029, R-030, R-099, R-100), `terms_at(bail, avenants, date d'échéance)` (R-104) ; tests d'or E-01, E-02, E-03, E-04, E-37, E-39, E-40, E-44.
- Propriété P15 : génération rejouée sans doublon ; date toujours dans le mois de la période.
- Base : unicité par période, écriture seule (`UPDATE` et `DELETE` refusés), vue sans colonne de statut.
- Tâche : deux travailleurs en même temps, clé du jour, reprise après échec ; horloge fixée au 13/11/2026 09:15 et bascule de minuit au fuseau de l'organisation.

## 8. Ce qui sera vérifié sur le poste, et ce qui ne le sera pas

- La tâche quotidienne tourne sur le poste avec `immo work` et une horloge de test ; pas sur l'essai (inexistant).
- Budget de 300 ms de la vue de soldes sur 500 lots : mesuré en local.

## 9. Écarts aux documents propriétaires

- Index `installment_lines_one_rent_charges` remplacé (question 1) : `DATABASE_SCHEMA.md` à corriger avant la migration.
- Alerte `installments.generation_late` : nouveau nom d'alerte.

## 10. Questions pour Mimis

1. **Avenant sur une échéance déjà générée** (dans la fenêtre de 30 jours, pas encore échue) : les échéances sont en écriture seule. *Recommandation : annuler la ligne de loyer avec un motif automatique et ajouter la ligne au nouveau montant* (trace complète, rien de modifié), ce qui impose le changement d'index du §3 point 2. Autre option : ne générer qu'au jour J−0 (fenêtre de 0 jour), ce qui contredit R-029 (30 jours).
2. **Dernière période** : *recommandation : la période qui contient la fin effective est due en entier* (cohérent avec R-099, sans prorata), aucune après. Autre option : pas d'échéance si la date d'échéance tombe après la fin.
3. **Échéance à dû nul** (toutes ses lignes annulées) : *recommandation : statut « payée » et montant 0* (rien n'est dû) ; affichée « Annulée » si toutes les lignes sont annulées. Autre option : la masquer.
4. **Heure de génération** : *recommandation : 00 h 05 au fuseau de l'organisation, alerte à 02 h 00* (valeurs d'`ARCHITECTURE.md` §4.8, proposées).
