# Étape 16 — Mandats, avenants de taux et dépenses imputables minimales

| Champ | Valeur |
|---|---|
| Statut | À faire |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T2 (F05 ; F15 minimal) |
| Effort | M, environ 11 JD |
| Dépend de | Étapes 03 et 05 |
| Jalon | Lot 2 (avant la bascule `live`) |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Savoir pour qui l'agence gère chaque lot et à quelles conditions de commission, et saisir les dépenses imputables qui entreront dans le compte rendu.
Valeur : préalable indispensable du compte rendu propriétaire (étape 17), livrable que l'agence doit produire chaque mois.

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections |
|---|---|---|
| Fonctionnalités | F05 (mandats), F15 (version minimale) | dossier §16 F05, §7.3 |
| Règles | R-014, R-015, R-016 (déjà livrée), R-069, R-078, R-084 ; E-17, E-29 | `DOMAINE.md` |
| Décisions | DP-14, DP-16, DP-18, DP-26 ; n° 29 (précision du taux), 52 | `DECISIONS.md` |
| Schéma | `DATABASE_SCHEMA.md` | §3 `mandates`, `mandate_units`, `mandate_rate_changes`, `interventions`, `providers`, `expense_imputation_changes` ; §4.1, §4.6 |
| Contrat | `API_CONTRACTS.md` | §7.8 (mandants, mandats), §7.10, §6.5 |
| Écrans | `UI_UX_DESIGN.md` | S13, S14 (version minimale), S07 (onglet Travaux) |

## 3. Prérequis

Étapes 03 et 05. Décision n° 29 : taux en centièmes de point (750 = 7,5 %) ou autre précision [À valider]. Chiffrement applicatif des coordonnées de paiement du mandant (`payout_details_enc`, AES-256-GCM).

## 4. Fiche de spécification à produire ou valider

1. `specs/F05-mandats.md` — un lot, un seul mandat actif (exclusion sur `mandate_units`, R-014) ; taux de 0 à 100 % (R-015) ; base de commission par défaut « loyer » avec option « loyer + charges » (DP-16) ; avenant de taux non rétroactif (date au 1ᵉʳ d'un mois strictement futur, R-069, E-17) ; clôture de mandat, transfert de lots ; coordonnées de reversement chiffrées et révélées par une action tracée. Cas limites : mandat clôturé en cours de mois (R-109, étape 17), taux décimal.
2. `specs/F15-depenses-minimales.md` — intervention (date, nature, coût, prestataire, payeur propriétaire/agence/locataire, statut, pièce facultative) ; imputation par défaut : payée par l'agence ⇒ imputable, sinon non (R-084, E-29) ; changement d'imputation tracé ; dépense sans pièce : avertissement ; dépense figée une fois liée à un compte rendu validé (`expense.frozen`). **S'arrêter pour validation de Mimis.**

## 5. Tâches détaillées

### 5.1 Base de données
`mandates` (`rate_bp` 0 à 10 000, base, dates, `row_version`), `mandate_units` (exclusion), `mandate_rate_changes` (ajout seul), `providers`, `interventions` (`row_version`, archivage), `expense_imputation_changes` (ajout seul) ; garde « mandat seulement pour le plan Agence » ; `statement_expense_links` créé à l'étape 17.

### 5.2 Noyau métier
`property::mandates` (création, avenant de taux, clôture, transfert), `expenses` (saisie, imputation par défaut, changement d'imputation avec motif) ; `owners::payout_details` révélées avec audit ; audit `mandate.created`, `mandate.rate_changed`, `mandate.closed`, `expense.imputation_changed`, `owner.updated`.

### 5.3 API
`GET/POST /mandates`, `GET /mandates/{id}`, `POST /mandates/{id}/rate-changes`, `POST /mandates/{id}/close`, `PUT /mandates/{id}/units`, `GET /owners/{id}/payout-details`, `GET /interventions`, `POST /units/{id}/interventions`, `GET/PATCH /interventions/{id}`, `POST /interventions/{id}/imputation` ; réponse `plan.feature_unavailable` pour le plan Propriétaire ; agent terrain : proposition de dépense refusée (DP-66).

### 5.4 Interface
S13 : mandants, fiche, mandats (taux, base, dates, lots), avenant, clôture, transfert ; S14 minimal : journal des interventions avec payeur et imputation ; S07 : onglet Travaux ; états vide/chargement/erreur/succès.

### 5.5 Documents PDF
Aucun.

### 5.6 Audit et événements
Voir 5.2.

### 5.7 Tests
E-17, E-29 ; R-014 (chevauchement refusé), R-015 (101 % refusé) ; plan Propriétaire refusé ; coordonnées illisibles en base ; A3 (gérant seul pour mandats), A4.

## 6. Critères d'acceptation vérifiables

1. Chevauchement de mandats sur un lot refusé ; taux hors plage refusé.
2. Avenant daté du 1ᵉʳ du mois suivant : l'ancien taux reste appliqué au mois courant (E-17).
3. Deux dépenses de 30 000, l'une payée par l'agence, l'autre par le propriétaire : seule la première est imputable par défaut (E-29).
4. Le changement d'imputation écrit une ligne d'audit ; les coordonnées de paiement ne sont jamais lisibles en clair en base.

## 7. Sécurité et déploiement à vérifier

Clé de chiffrement des champs en place et sauvegardée à part ; migration de l'exclusion relue ; interrupteur éteint hors agence pilote si besoin.

## 8. Liste de démonstration (Mimis)

Créer un mandant et un mandat à 8 % sur trois lots, tenter un second mandat sur un lot : refus ; ajouter un avenant au mois suivant ; saisir deux dépenses et lire leur imputation.

## 9. Définition de terminé, risques, hors étape

Terminé : critères 1 à 4 verts ; `DECISIONS.md` (n° 29) à jour.

| Risque | Mesure |
|---|---|
| Barèmes de commission plus variés (minimum, plafond) | Règle d'or 12 : option si trois clients, sinon refus ou service |

Hors étape : travaux complets et documents (étape 20), compte rendu (étape 17).

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 16
- Dépend de : 03, 05
- Tranche : T2
- Jalon : Lot 2
- F : F05, F15
- R : R-014, R-015, R-069, R-078, R-084
- E : E-17, E-29
- PP : aucun
- Contenus : aucun
