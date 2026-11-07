# Étape 06 — Baux et échéancier automatique

| Champ | Valeur |
|---|---|
| Statut | Terminée le 04/11/2026 (validée par Mimis) ; fiches `specs/F07-baux.md` et `specs/F08-echeancier.md` rédigées le 04/11/2026, **à valider par Mimis avant tout code** |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T1 (F07, F08) |
| Effort | L, environ 18 JD (deux lots : 06a baux, 06b échéancier) |
| Dépend de | Étape 05 |
| Jalon | Contribue à J2 et J3 |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Créer et activer un bail, gérer ses avenants, et obtenir sans saisie l'échéancier des loyers dus avec un statut exact (à venir, due, partielle, payée, en retard) calculé par une seule fonction. C'est la base de tous les montants : sans échéance juste, ni encaissement ni impayés ne sont fiables.
Valeur : l'agence retrouve ses baux, ses loyers et ses retards sans Excel ; les règles de calendrier encore incertaines (jour 31, premier mois partiel, bail antérieur) sont codées, testées et confrontables au terrain.

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections |
|---|---|---|
| Fonctionnalités | F07, F08 | dossier §16 F07, F08 |
| Règles baux | R-006, R-013, R-020 à R-026, R-090 (brouillon, actif, terminé), R-091, R-092, R-093, R-104 | `DOMAINE.md` « Baux », « Patrimoine » |
| Règles échéances | R-027 à R-036, R-095, R-098, R-099, R-100 | `DOMAINE.md` « Échéances » |
| Cas chiffrés | E-01 à E-04, E-22, E-35, E-36, E-37, E-39, E-40, E-44 | `DOMAINE.md` « Exemples chiffrés » |
| Décisions | DP-13, DP-24, DP-25, DP-26, DP-27, DP-39, DP-41, DP-50, DP-51, DP-58 ; n° 33, 41, 42 | `DECISIONS.md` |
| Architecture | `ARCHITECTURE.md` | §4.6, §4.7, §4.8, §3.9, §4.9 (statut d'échéance : une seule fonction) |
| Schéma | `DATABASE_SCHEMA.md` | §3 « Locataires, baux et échéances », §4.1, §4.4, §5 (vues de soldes) |
| Contrat | `API_CONTRACTS.md` | §7.5 (baux, échéances, lignes) |
| Écrans | `UI_UX_DESIGN.md` | S10, S07 (onglet Baux), badges de statut, « Statuts financiers » |

## 3. Prérequis

Étape 05. Règles R-098, R-099, R-100, R-104 : à relire avec Mimis car elles portent le marqueur [à confirmer avec l'agence pilote à la découverte] ; elles sont codées avec leur valeur par défaut. Décision n° 33 (soldes : vues dérivées, table de synthèse seulement si une mesure l'impose). Avis juridique sur le plafond de dépôt de garantie (R-024, DP-62) : l'avertissement non bloquant est codé sans attendre l'avis.

## 4. Fiches de spécification à produire ou valider

1. `specs/F07-baux.md` — règles R-020 à R-026, R-090 à R-093, R-104 ; statuts brouillon, actif, terminé (préavis : étape 21) ; avenant daté jamais rétroactif ; prolongation de la date de fin = avenant (R-091, E-35) ; dépôt de garantie saisi et avertissement non bloquant au-delà de deux mois en habitation (R-024, E-22) ; commercial : durée, indexation, `vat_included` sans calcul (R-025, R-026, R-092) ; lot « en travaux » refuse l'activation (R-093, E-36). Cas limites : chevauchement avec un bail terminé (décision n° 41), colocation (plusieurs locataires, un seul bail), changement de locataire = nouveau bail, activation répétée (idempotente). Erreurs : `lease.overlap`, `unit.under_works`, `lease.not_draft`, `lease.amendment_retroactive`. Cas chiffrés : E-01, E-02, E-22, E-35, E-36, E-44.
2. `specs/F08-echeancier.md` — une échéance par bail et par période (R-027), génération des 30 jours à venir (R-029) par tâche quotidienne idempotente à 00 h 05 au fuseau de l'organisation, génération immédiate à l'activation (R-028) et après avenant ; lignes `rent`, `charges`, `arrears`, `exceptional` ; jour d'échéance 29 à 31 dans un mois court = dernier jour (R-098, E-39) ; premier mois dû en entier (R-099, E-40) ; bail activé avec date passée : échéances à partir du mois d'activation (R-100) ; avenant : échéances dont la date est postérieure à sa date d'effet **et** à sa date de saisie (R-104, E-44) ; jour absent : le 1ᵉʳ, signalé (R-031) ; statut calculé à la lecture jamais stocké (R-032 à R-034, R-095, DP-51, E-04, E-37). Cas limites : bail terminé (R-030), année bissextile, annulation de ligne avec motif (R-036). Cas chiffrés : jour 31 en février 2027 = 28/02/2027 ; le 4 « à venir », le 5 « due », le 6 « en retard ».

**S'arrêter pour la validation de Mimis après ces fiches.**

## 5. Tâches détaillées

### 5.1 Base de données
1. `leases` (statut `draft|active|notice|ended`, dates, loyer, charges, `due_day`, `deposit_amount`, `vat_included`, `notice_given_on`, `departure_date`, colonne générée `effective_end`, `row_version`) avec contrainte d'exclusion `EXCLUDE USING gist` sur `(org_id, unit_id, daterange(start_date, effective_end, '[]'))` pour les statuts actif, préavis et terminé (G-01, G-02) ; `lease_tenants` ; `lease_commercial_terms` ; `lease_amendments` (ajout seul, garde minimale : aucune échéance payée à partir de la date d'effet) ; `installments` (une par `(lease_id, period)`, `due_date` dans le mois de la période, ajout seul) ; `installment_lines` (types, ajout seul) ; `installment_line_voids` (ajout seul, motif).
2. Montants `bigint` bornés (`CHECK` 1 à 999 999 999 pour le loyer, 0 à 999 999 999 pour charges et dépôt) ; `leases_guard` autorisant la modification libre en brouillon, et seulement la fin de contrat et l'état ensuite ; `row_version` modifiable.
3. Vues `v_installment_balances` (dû, affecté, reste ; **aucune règle de date**, DP-51) ; `v_lease_credit` créée vide de sens jusqu'à l'étape 07.
4. `unit_assignments` : ajout des clés composites restantes. Statut d'occupation d'un lot dérivé du bail en cours (R-013).

### 5.2 Noyau métier (`crates/domain` d'abord, tests d'or avant le code)
1. `due_date_for(period, due_day)` (R-098, R-031), `installment_status(due, allocated, today)` et `days_late` (R-032, R-033, R-095), `amended_amounts(lease, due_date)` (R-023, R-104), génération de la liste des périodes à créer (R-027, R-029, R-099, R-100, R-030).
2. `lease` : `create_draft`, `update_draft`, `activate` (idempotent, sous verrou du lot, refus `lease.overlap`, `unit.under_works`, `lease.not_draft`), `add_amendment` (non rétroactif, avenant de prolongation), `terminate`, `add_exceptional_line` (arriéré ou exceptionnelle), `void_line` (motif, refusé si paiement net reçu), `archive_unit` refusé avec bail en cours (R-006).
3. Tâche planifiée `generate_installments` : par organisation à 00 h 05, clé d'unicité par organisation et par jour, `INSERT … ON CONFLICT DO NOTHING`, alerte si aucun succès avant 02 h 00 ; l'affectation du crédit du bail (R-086) est branchée à l'étape 07.
4. Audit : `lease.created`, `lease.activated`, `lease.amended`, `lease.ended`, `installment.line_added`, `installment.line_voided`, `unit.works_set`.

### 5.3 API (`API_CONTRACTS.md` §7.5, §7.4)
`GET/POST /leases`, `GET/PATCH /leases/{id}`, `POST /leases/{id}/activate`, `POST /leases/{id}/amendments`, `POST /leases/{id}/terminate`, `GET /leases/{id}/installments`, `POST /leases/{id}/installment-lines`, `POST /installment-lines/{id}/void`, `GET /units/{id}/leases`, `POST /units/{id}/works`. Réponses avec `warnings[]` (dépôt, jour par défaut). Le statut, l'ancienneté et les tranches viennent de l'API, jamais du front.

### 5.4 Interface
S10 : liste des baux (alertes), création en trois étapes (lot et locataires ; dates, loyer, charges, jour d'échéance, dépôt, type habitation ou commercial et champs commerciaux ; vérification et activation), fiche bail (statuts, échéancier avec badges texte + icône, avenants, résiliation, ligne exceptionnelle), garde-fous visibles (chevauchement avec dates, lot en travaux, dépôt supérieur à deux mois) ; S07 : onglet Baux et bouton « Mettre en travaux » ; S09 : onglet Baux. Aucune règle d'argent dans le front.

### 5.5 Documents PDF
Aucun (la fiche de bail imprimable est reportée, non requise).

### 5.6 Audit et événements
Voir 5.2. Événements : `screen_viewed` (S10) ; erreurs de validation `validation_error` avec code de règle.

### 5.7 Tests
- Tests d'or (`tests/fixtures/golden/E-xx.json`) E-01, E-02, E-03, E-04, E-22, E-35, E-36, E-37, E-39, E-40, E-44 au niveau `domain` puis base puis API.
- Propriétés : P9 (statut fonction du triplet dû, affecté, aujourd'hui), P15 (génération rejouée sans doublon ; date dans le mois de la période).
- Base : G-01, G-02 (y compris contre les baux terminés), G-03, G-10 et G-18 (avenants), échéance unique par période, `UPDATE` refusé sur `installments` et `lease_amendments` pour `immo_app` (G-05).
- API : A3, A4, A8 (activation avec clé d'idempotence), A9 non concerné, A10 (loyer nul, dates inversées) ; `lease.not_draft` ; 412 sur `PATCH`.
- Horloge : tests à la date fixée du 13/11/2026 09:15 et bascule de jour.
- E2E : créer un bail de 75 000 FCFA avec échéance le 5, l'activer le 1ᵉʳ mars (horloge fixée), lire l'échéancier ; ajouter un avenant à 80 000 en avril.

## 6. Critères d'acceptation vérifiables

1. Bail de 75 000 FCFA, échéance le 5, activé le 1ᵉʳ mars : échéance de mars au 5 mars pour 75 000 (E-01).
2. Avenant à 80 000 à partir d'avril : mars reste à 75 000, avril passe à 80 000 (E-02) ; avenant avec une date d'effet passée dont l'échéance est payée : refus.
3. La tâche exécutée deux fois le même jour ne crée aucune échéance en double (E-03).
4. Échéance du 5 non payée : le 4 « à venir », le 5 « due », le 6 « en retard » (E-37, E-04).
5. Bail avec jour d'échéance 31 : février 2027 échoit le 28/02/2027 (E-39) ; bail activé le 20 mars : mars dû en entier (E-40) ; avenant saisi le 15/12 d'effet 01/12 : décembre reste à 75 000 et janvier passe à 80 000 (E-44).
6. Un second bail en cours sur un lot, ou un bail qui chevauche un bail terminé, est refusé avec les dates en cause (R-020).
7. Bail d'habitation avec dépôt supérieur à deux mois : avertissement non bloquant (E-22) ; activation sur un lot « en travaux » refusée, puis acceptée après retour à « vacant » (E-36).
8. Prolonger la date de fin par avenant génère les échéances jusqu'à la nouvelle fin, sans toucher au dépôt (E-35).
9. Archiver un lot qui a un bail en cours est refusé avec explication (R-006).
10. Aucune colonne de statut d'échéance n'existe en base (test de schéma).

## 7. Sécurité et déploiement à vérifier

Migration de la contrainte d'exclusion et des gardes d'ajout seul relue par un second avis ; tâche planifiée testée avec plusieurs instances (pas de doublon) ; `immo_app` sans `UPDATE` sur `installments`, `installment_lines`, `lease_amendments` ; test d'isolation de chaque route ; migrations rétrocompatibles exercées sur l'essai.

## 8. Liste de démonstration (Mimis)

1. Créer un bail d'habitation de 75 000 FCFA, échéance le 5, dépôt de 200 000 FCFA : lire l'avertissement « supérieur à deux mois ».
2. L'activer : ouvrir l'échéancier et lire la première échéance et son badge.
3. Ajouter un avenant à 80 000 d'effet au mois suivant : voir le mois courant inchangé et le suivant mis à jour.
4. Tenter d'activer un second bail sur le même lot : lire le refus avec les dates.
5. Mettre un lot vacant « en travaux » puis tenter d'y activer un bail : lire le refus, annuler les travaux, réussir.
6. Le développeur avance l'horloge de l'essai au 6 : voir le badge passer à « En retard ».

## 9. Définition de terminé, risques, hors étape

Terminé : critères 1 à 10 verts ; tests d'or cités dans `TRACABILITE.md` (colonne Étape) ; `DOMAINE.md` : les règles R-098 à R-100 et R-104 restent [à confirmer avec l'agence pilote à la découverte] ; `API_CONTRACTS.md`, `DATABASE_SCHEMA.md`, `DECISIONS.md` (n° 33, 41, 42) et `CLAUDE.md` à jour.

| Risque | Mesure |
|---|---|
| Règles de calendrier différentes chez l'agence | Valeurs par défaut documentées, dates d'effet pour toute évolution (`ARCHITECTURE.md` §8.1), jamais de recalcul des écritures passées |
| Bail verbal ou sans date de fin | Date de fin facultative ; bail sans fin génère à l'infini dans la fenêtre de 30 jours |
| Vue de soldes lente | Mesure sur 500 lots ; table de synthèse seulement si le budget de 300 ms est dépassé (n° 33) |

Hors étape : paiements, affectations, crédit du bail (étape 07) ; préavis, restitution du dépôt (étape 21) ; indexation automatique (hors MVP, F30) ; fiche de bail imprimable.

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 06
- Dépend de : 05
- Tranche : T1
- Jalon : J2
- F : F07, F08
- R : R-006, R-013, R-020, R-021, R-022, R-023, R-024, R-025, R-026, R-027, R-028, R-029, R-030, R-031, R-032, R-033, R-034, R-035, R-036, R-090, R-091, R-092, R-093, R-095, R-098, R-099, R-100, R-104
- E : E-01, E-02, E-03, E-04, E-22, E-35, E-36, E-37, E-39, E-40, E-44
- PP : PP-06
- Contenus : aucun
