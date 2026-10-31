# Étape 07 — Encaisser un loyer (paiement, affectation, numéro de quittance)

| Champ | Valeur |
|---|---|
| Statut | À faire |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T1 (F09 ; F10 : numérotation, instantané, aperçu) |
| Effort | L, environ 18 JD (deux lots : 07a domaine et base, 07b API et écran S04) |
| Dépend de | Étape 06 |
| Jalon | Contribue à J2 et J3 |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Un caissier enregistre un paiement en quelques gestes ; il est affecté aux bonnes échéances, jamais en double, jamais modifiable, et reçoit immédiatement un numéro de quittance sans trou avec son aperçu. C'est le cœur financier du produit.
Valeur : la vérité sur les encaissements (règle d'or 1) ; la quittance sort de la saisie (règle d'or 2), ce qui rend le tableau des impayés juste.

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections |
|---|---|---|
| Fonctionnalités | F09, F10 (partie : numéro, instantané, aperçu) | dossier §16 F09, F10 |
| Règles | R-037 à R-047, R-050, R-053, R-086, R-087, R-088, R-102 (R-089 : capture des dates seulement) | `DOMAINE.md` « Paiements », « Quittances » |
| Cas chiffrés | E-05, E-06, E-07, E-09, E-10, E-11, E-27, E-31, E-32, E-33, E-42 (partie paiement) | `DOMAINE.md` |
| Décisions | DP-13, DP-15, DP-20 à DP-23, DP-37, DP-45, DP-46, DP-47, DP-49, DP-50, DP-61, DP-64 ; n° 37 | `DECISIONS.md` |
| Architecture | `ARCHITECTURE.md` | §4.1 à §4.5, §4.9 |
| Schéma | `DATABASE_SCHEMA.md` | §3 « Encaissements », « Quittances et liens de partage » ; §4.2, §4.3, §4.5 |
| Contrat | `API_CONTRACTS.md` | §5.1, §5.3, §6.1 (parcours 1), §7.6 |
| Écrans | `UI_UX_DESIGN.md` | S04, S11, S25, « Champs et formulaires », « Microcopy » (erreurs de saisie, avertissements) |
| Tests | `TESTING_QA.md` | §2.1, §2.2 (P1 à P8, P13, P14, P16), §2.3, §3 |

## 3. Prérequis

Étape 06. **Contenu C-13** : téléphone Android d'entrée de gamme acheté (profil de test, `TESTING_QA.md` §8.1). Décisions confirmées : n° 37 (contrôles différés sous verrou du bail, niveau `READ COMMITTED`), seuil de confirmation d'un paiement (R-038 : 1 000 000 FCFA par défaut, valeur à fixer [À valider]). Prototype (c) rejoué en CI dès cette étape. Mentions de la quittance (C-08) nécessaires à l'étape 08, pas ici.

## 4. Fiches de spécification à produire ou valider

1. `specs/F09-paiements.md` — **règles** : montant entier strictement positif et borné (R-037, R-001) ; date jamais future (R-039, date du jour fournie par le serveur) ; modes `cash`, `mobile_money`, `bank_transfer`, `cheque` (R-040) ; référence externe unique par mode avec avertissement **bloquant jusqu'à confirmation explicite** (R-041, R-102, E-42) ; clé d'idempotence obligatoire (R-042, E-07) ; affectation par défaut aux échéances les plus anciennes puis loyer, charges, exceptionnelles (R-043, R-087, E-32), ajustable ; somme des affectations ≤ paiement (R-044) ; partiel avant J+1 puis « en retard » (R-045, DP-13) ; excédent = avance affectée à l'échéance suivante, sinon crédit du bail (R-046, R-086, E-06, E-31) ; paiement rattaché au bail, « payé par » locataire ou tiers (R-088, E-33) ; seuil de confirmation (R-038, E-09) ; paiement immuable (R-047). **Cas limites** : deux caissiers sur le même bail, coupure avant et après validation, paiement exactement égal au solde, paiement d'une somme supérieure à toutes les échéances, bail non en cours (`payment.lease_not_in_progress`). **Écrans** S04, S11. **Erreurs** : `payment.external_ref_duplicate`, `payment.confirmation_required`, `payment.surplus_not_confirmed`, `payment.allocation_invalid`, `idempotency.*`, `validation.date_in_future`. **Cas chiffrés** : E-05, E-06, E-07, E-09, E-31, E-32, E-33.
2. `specs/F10-numerotation-et-instantane.md` — compteur transactionnel `INSERT … ON CONFLICT DO UPDATE … RETURNING` pris **en dernier** dans la transaction du paiement (DP-46) ; numéro `{préfixe}-{année sur 4 chiffres}-{séquence sur 6 chiffres}` (DP-37), série `trial` (préfixe `ESSAI-`) en phase `setup`, `official` après la bascule (DP-61) ; année = année d'émission au fuseau de l'organisation ; contenu de l'instantané (parties, lot, période, ventilation, solde restant, avance, montant en lettres, « payé par », indicateur « TVA incluse », paramètres d'en-tête au moment de l'émission, filigranes) ; montant en lettres (R-053, DP-64). **Cas limites** : paiement du 31/12 saisi le 02/01 ; échec après attribution (annulation complète, pas de trou) ; rendu PDF jamais dans la transaction. **Cas chiffrés** : E-10, E-11 (75 000 = « soixante-quinze mille francs CFA »), E-27 (000812 en 2026, 000001 en 2027).

**S'arrêter pour la validation de Mimis après ces fiches.**

## 5. Tâches détaillées

### 5.1 Base de données
1. `payments` (`kind` `payment|reversal`, `amount` signé, `method`, `external_ref`, `paid_on`, `recorded_at`, `cash_day`, `received_by`, `payer` (locataire du bail ou tiers : nom, téléphone facultatif), `note`, `reverses_payment_id` unique) ; `allocations` (par ligne d'échéance, signées) ; `idempotency_keys` (`org_id`, `key`, `route`, `user_id`, `request_hash`, réponse enregistrée, 30 jours) ; `receipt_counters` ; `receipts` (série, année de numérotation, séquence, `kind`, `snapshot`, `replaces_receipt_id`, `UNIQUE (org_id, series, numbering_year, sequence)`) ; `cash_days` **créée vide** (un déclencheur de `payments` la lit, `cash_day ≥ paid_on`).
2. Gardes : `payments_amount_sign`, bornes 1 à 999 999 999, date non future (G-04, G-11), horodatage imposé (G-06), numéro égal au compteur et consécutif (G-07, G-08, G-09), affectations bornées et jamais entre deux baux (G-12, G-13), contrôles différés `chk_allocation` sous verrou consultatif du bail (n° 37), ajout seul pour `payments`, `allocations`, `receipts` (G-05).
3. Vues `v_payment_credit` et `v_lease_credit` (R-086) ; compléter `v_installment_balances`.
4. Migration qui touche une garde ou le compteur : second avis, contrôle nocturne des déclencheurs.

### 5.2 Noyau métier
1. `crates/domain` (tests d'or d'abord) : `allocate(amount, lines)` déterministe (date, puis loyer, charges, exceptionnelles), calcul de l'avance et du crédit, `amount_in_words` (0 à 999 999 999, orthographe traditionnelle, accords de « vingt » et « cent », « mille » invariable), `receipt_number(prefix, year, sequence, series)`.
2. `ledger::record_payment(tx, cmd, idempotency_key)` dans **une transaction** : contrôles métier ⇒ `pg_advisory_xact_lock` du bail ⇒ relecture des soldes ⇒ insertion du paiement et des affectations ⇒ application du crédit existant ⇒ `receipts::issue` (compteur pris en dernier, instantané) ⇒ audit ⇒ insertion de la tâche de rendu PDF (type `render_receipt`) ⇒ enregistrement de la clé d'idempotence et de la réponse. Aucun appel au stockage de fichiers ni au moteur PDF.
3. Verrou consultatif sur `(org, mode, référence)` pour R-041 ; avertissement bloquant jusqu'à `acknowledge_external_ref_duplicate`.
4. Cas d'usage `generate_and_apply_credit` : à la création d'une échéance, le crédit du bail s'y affecte (E-31) ; branché à la tâche de l'étape 06.
5. Contrôle nocturne des trous de numérotation (requête de `DATABASE_SCHEMA.md` §7.5) avec alerte S1.
6. Audit : `payment.recorded`, `receipt.issued`, `allocation.adjusted`. Événements écrits par le serveur : `payment_recorded` (durée depuis l'ouverture, mode, partiel, avance, crédit, nombre d'essais, retard de saisie), `payment_idempotent_replay`, `payment_external_ref_duplicate`.

### 5.3 API (`API_CONTRACTS.md` §6.1, §7.5, §7.6)
`GET /leases/{id}/payment-context`, `POST /payments/preview` (sans écriture), `POST /payments` (en-tête `Idempotency-Key` obligatoire ; réponse `Idempotent-Replay: true` au rejeu ; 422 `idempotency.key_reused_with_different_body`), `GET /payments`, `GET /payments/{id}`, `PUT|PATCH|DELETE /payments/{id}` ⇒ 405 `payment.immutable`, `GET /receipts`, `GET /receipts/{id}`. L'aperçu d'affectation, le solde, le statut et le montant en lettres sont calculés par le serveur.

### 5.4 Interface
S04 : recherche unique, carte du locataire avec solde, montant, date (valeur `today` du serveur), mode (« Espèces » par défaut), référence, « payé par », note, aperçu d'affectation modifiable, bouton « Encaisser et envoyer la quittance » collé en bas sur mobile (52 px), clavier numérique ; clé d'idempotence créée **à l'ouverture** du formulaire et réutilisée à chaque nouvel essai ; message « Connexion perdue. Vos informations sont conservées… » ; modales de confirmation (seuil, avance, référence en doublon) ; succès : aperçu de la quittance avec son numéro (S25 en HTML, actions PDF et WhatsApp à l'étape 08) et « Encaisser un autre loyer » (nouvelle clé) ; S11 : journal (filtres, détail, statuts « Validé » et « Corrigé » dérivés) ; S09 : onglet Paiements ; chargement du paquet S04 séparé au premier accès du caissier.

### 5.5 Documents PDF
Hors étape (aperçu HTML de la quittance seulement).

### 5.6 Audit et événements
Voir 5.2 point 6. Événement d'écran : `payment_form_opened`, `payment_form_abandoned` ; `validation_error` avec code de règle.

### 5.7 Tests (écrits avant le code pour les règles d'argent)
- Cas d'or E-05, E-06, E-07, E-09, E-10, E-11, E-27, E-31, E-32, E-33, E-42 aux niveaux `domain`, base et API.
- Propriétés (10 000 cas la nuit) : P1 à P8, P13, P14, P16.
- Concurrence : 4 × 50 validations par PR et 20 × 500 la nuit (3 organisations, 10 % d'annulations forcées) ; même clé 3 fois en parallèle ; après coupure (réponse perdue puis nouvel essai ; coupure avant validation) ; même clé avec contenu différent ; deux caissiers sur le même locataire ; test que la connexion applicative est en `READ COMMITTED`.
- Base : G-03, G-04, G-05 (3 rôles), G-06 à G-09, G-11 à G-13, compteur ramené en arrière ou avancé de 5 refusé, requête de trous = 0 ligne.
- API : A3, A4, A8, A9, A10 (montant `75000.5` refusé, hors bornes refusé, date future refusée, champs inconnus).
- E2E parcours 1 (390 et 1440 px) avec variantes : 50 000 (E-05), 100 000 avec avance (E-06), référence en doublon, connexion coupée puis nouvel essai sans doublon, montant au-dessus du seuil, paiement par un tiers.
- Performance : `POST /payments` p95 ≤ 300 ms sans PDF ; chemin nominal à 3 allers-retours après l'ouverture ; temps machine ≤ 5 s sur « 4G lente » (Playwright).

## 6. Critères d'acceptation vérifiables

1. Loyer 75 000, paiement 50 000 en espèces : échéance « partielle », 25 000 restants, quittance « solde restant 25 000 » ; à J+1 : « en retard », payé 50 000, reste 25 000 (E-05, E-25).
2. Paiement 100 000 pour 75 000 : 75 000 affectés, 25 000 d'avance sur l'échéance suivante ; sans échéance suivante : 25 000 de crédit du bail, affectés dès la génération (E-06, E-31).
3. Deux soumissions identiques (double clic) créent un seul paiement et une seule quittance (E-07).
4. Deux paiements successifs ont des quittances aux numéros consécutifs ; le premier de 2027 repart à 000001 (E-10, E-27) ; sur 10 000 validations concurrentes, 0 trou, 0 doublon.
5. 75 000 s'écrit « soixante-quinze mille francs CFA » sur l'aperçu (E-11).
6. Paiement supérieur au seuil : confirmation demandée (E-09). Référence mobile money déjà utilisée : avertissement qui bloque jusqu'à confirmation (E-42, partie enregistrement).
7. `PUT`, `PATCH`, `DELETE` d'un paiement répondent 405 `payment.immutable` ; `UPDATE`, `DELETE`, `TRUNCATE` échouent pour les trois rôles de base.
8. Échéance du 5 avec loyer 75 000 et charges 10 000, paiement 50 000 : 50 000 sur le loyer, reste 25 000 de loyer et 10 000 de charges (E-32) ; paiement du frère du locataire : rattaché au bail, « payé par » = tiers, mention sur l'aperçu (E-33).
9. Un caissier saisit un paiement en moins de 30 secondes sur le téléphone de test (mesure préliminaire, mesure officielle à l'étape 13).

## 7. Sécurité et déploiement à vérifier

Aucune décimale n'entre ni ne sort (désérialisation stricte, PP-17) ; aucune lecture directe de l'heure (lint) ; le compteur n'est jamais touché hors du module `receipts` (recherche en CI de `INTO receipts` et `nextval`) ; migration de garde relue par un second avis ; journaux sans numéro ni jeton ; livraison respectant le calendrier de `DEPLOYMENT.md` §4 dès qu'une agence aura des données (ici l'essai seulement).

## 8. Liste de démonstration (Mimis)

1. Sur le téléphone de test, ouvrir « Encaisser », taper « Tapsoba », choisir la carte « Due aujourd'hui 75 000 ».
2. Encaisser 50 000 en espèces : lire le message de succès, le numéro de quittance et « solde restant 25 000 ».
3. Encaisser 100 000 sur un autre bail de 75 000 : lire « avance 25 000 ».
4. Couper le réseau juste avant « Encaisser », rétablir, réessayer : voir un seul paiement dans le journal.
5. Saisir la même référence mobile money deux fois : lire l'avertissement bloquant, confirmer.
6. Ouvrir le journal S11 : vérifier que les numéros se suivent ; tenter de modifier un paiement : refus.

## 9. Définition de terminé, risques, hors étape

Terminé : critères 1 à 9 verts ; PP-01 (pour les tables de l'étape), PP-03 et PP-04 démontrés en CI ; événements ajoutés à la liste blanche ; `DOMAINE.md` inchangé (aucune règle inventée : tout doute a été noté en question ouverte) ; documents mis à jour ; `CLAUDE.md` à jour.

| Risque | Mesure |
|---|---|
| Contention sur le compteur en début de mois | Compteur pris en dernier, p95 mesuré ; une organisation à la fois |
| Règle de reddition ou d'avance différente chez l'agence | Règles [à confirmer avec l'agence pilote à la découverte] ; dates d'effet, jamais de recalcul |
| Caissier trop lent (> 30 s) | Mesure dès cette étape, écran retravaillé avant PP-25 |

Hors étape : PDF et partage (étape 08), correction par contre-passation (étape 09), clôture de caisse et lecture du SMS (étape 22), impayés (étape 10).

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 07
- Dépend de : 06
- Tranche : T1
- Jalon : J2
- F : F09, F10
- R : R-037, R-038, R-039, R-040, R-041, R-042, R-043, R-044, R-045, R-046, R-047, R-050, R-053, R-086, R-087, R-088, R-102
- E : E-05, E-06, E-07, E-09, E-10, E-11, E-27, E-31, E-32, E-33, E-42
- PP : PP-01, PP-03, PP-04, PP-06
- Contenus : C-13
