# Étape 09 — Corrections : demande, contre-passation, quittance annulée et rectificative

| Champ | Valeur |
|---|---|
| Statut | À faire |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T1 (fin de F09 et de F10) |
| Effort | M, environ 8 JD |
| Dépend de | Étapes 07 et 08 |
| Jalon | Contribue à J2 et J3 (PP-02) |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Corriger une erreur de saisie sans jamais modifier un paiement : le caissier demande, le gérant contre-passe par une écriture inverse liée, la quittance d'origine passe à « Annulée » et le paiement de remplacement reçoit une quittance rectificative qui la référence.
Valeur : intégrité des écritures (règle d'or 4) et réponse directe à l'erreur humaine du comptoir ; sans cette étape, la tranche d'encaissement n'est pas finie (`ARCHITECTURE.md` §9.2).

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections |
|---|---|---|
| Fonctionnalités | F09 (correction), F10 (annulation, rectificative) | dossier §16 F09, F10 |
| Règles | R-048, R-051, R-096 (et R-047, R-102 pour la réutilisation d'une référence) | `DOMAINE.md` |
| Cas chiffrés | E-08, E-38, E-42 (référence réutilisable après contre-passation) | `DOMAINE.md` |
| Décisions | DP-38, DP-47, DP-64 (rectificative sans effet : hors pilote) | `DECISIONS.md` |
| Architecture | `ARCHITECTURE.md` | §4.3 (contre-passation, aucune quittance propre), §4.5 |
| Schéma | `DATABASE_SCHEMA.md` | §3 `payments`, `allocations`, `correction_requests`, `receipt_cancellations` ; §4.2, §4.5 (G-14), §4.3 (`receipts_one_replacement`) |
| Contrat | `API_CONTRACTS.md` | §6.1 (fin), §7.6 (`/correction-requests`, `/reversal`) |
| Écrans | `UI_UX_DESIGN.md` | S11, S25, « Modales de confirmation » (contre-passer), « Microcopy » |
| Droits | `SECURITY.md` | §3.4 (confirmation par mot de passe), §4.3 (restriction aussi imposée par la base) |

## 3. Prérequis

Étapes 07 et 08. La décision sur le rendu de la copie « annulée » (nouveau fichier lié à l'annulation ou filigrane posé à la lecture) est à trancher dans la fiche, car les fichiers et les quittances sont en ajout seul. La règle R-096 reste [à confirmer avec l'agence pilote à la découverte].

## 4. Fiche de spécification à produire ou valider

`specs/F09-correction-et-contre-passation.md` — **règles** : un caissier ou un gestionnaire ne fait que **demander** une correction (R-048) ; seul le gérant contre-passe, avec confirmation du mot de passe ; la contre-passation est une écriture de type `reversal`, de montant opposé, avec des affectations négatives qui annulent exactement celles d'origine, liée au paiement d'origine (unique), avec motif obligatoire ; elle n'a pas de quittance propre (choix de `maquette/NOTES.md`) ; la quittance d'origine passe à « Annulée » (`receipt_cancellations`) ; le paiement de remplacement, créé dans la même transaction, reçoit une quittance rectificative qui référence l'annulée et prend le numéro suivant (R-096, R-051, E-38) ; les statuts « Validé »/« Corrigé » et « Émise »/« Annulée » sont dérivés des écritures liées (DP-47). **Cas limites** : double contre-passation (refusée), contre-passation sans remplacement, remplacement de montant différent, paiement dont la journée de caisse est clôturée (T3), référence externe réutilisable (E-42), contre-passation d'un paiement dont l'échéance est déjà couverte par un relevé (étape 17). **Écrans** : S11 (file du gérant), modale de contre-passation, S25. **Erreurs** : `payment.already_reversed`, `payment.reversal_reason_required`, `correction.already_pending`, `auth.reauth_required`. **Cas chiffrés** : E-08 et E-38 (57 000 saisis à tort pour un loyer de 75 000).

**S'arrêter pour la validation de Mimis après la fiche.**

## 5. Tâches détaillées

### 5.1 Base de données
`correction_requests` (statut `pending|resolved|rejected`, motif, demandeur, résolution ; colonnes modifiables limitées) ; `receipt_cancellations` (cause `reversal|corrective_receipt`, ajout seul) ; `receipts.replaces_receipt_id` avec index unique `receipts_one_replacement` ; garde `payments_before_insert` : une ligne `reversal` n'est acceptée que si l'auteur du contexte (`app.user_id`) est gérant ou propriétaire, même par SQL direct ; contrainte de contre-passation exacte (G-14, requête n° 3 de `DATABASE_SCHEMA.md` §7.5) ; `payments.cash_day` propre à la contre-passation.

### 5.2 Noyau métier
1. `crates/domain` (cas d'or d'abord) : `reverse(payment, allocations)` produisant des affectations exactement opposées ; statut dérivé d'un paiement et d'une quittance.
2. `ledger::request_correction`, `reject_correction`, `reverse_payment(tx, payment, reason, replacement?)` en **une transaction** : verrou du bail ⇒ contre-passation ⇒ annulation de la quittance ⇒ éventuel paiement de remplacement (même clé d'idempotence, même chemin que l'étape 07) ⇒ quittance rectificative ⇒ audit ⇒ tâches de rendu ; refus si déjà contre-passé.
3. La référence externe d'un paiement contre-passé redevient utilisable (R-102, E-42).
4. Audit : `payment.correction_requested`, `payment.correction_rejected`, `payment.reversed`, `receipt.cancelled`, `receipt.corrective_issued`. Événements : `correction_requested`, `payment_reversed` (rôle, délai entre paiement et correction).

### 5.3 API
`POST /payments/{id}/correction-requests`, `GET /correction-requests`, `POST /correction-requests/{id}/reject`, `POST /payments/{id}/reversal` (🔐, `Idempotency-Key` obligatoire, corps `{ reason, correction_request_id?, replacement? }` ⇒ `{ reversal, replacement?, receipt_cancellation, receipts[], installments[] }`). `POST /receipts/{id}/corrective` reste réservé (404, DP-64).

### 5.4 Interface
S11 : bouton « Demander une correction » (motif obligatoire, texte « Un paiement validé ne peut pas être modifié. Vous pouvez demander une correction au gérant. »), file des demandes pour le gérant avec compteur visible aussi dans l'accueil du gérant (calculé, la cloche de notifications est en T3), modale « Contre-passer » avec confirmation du mot de passe, formulaire du paiement de remplacement pré-rempli (montant corrigé, mêmes champs que S04, nouvelle clé d'idempotence) ; S25 : statut « Annulée » (filigrane et lien vers la quittance qui la remplace), quittance rectificative (lien vers l'annulée) ; S09 : onglet Paiements avec statuts.

### 5.5 Documents PDF
Copie « annulée » : filigrane « ANNULÉE — remplacée par la quittance n° … » selon la décision de la fiche ; la rectificative porte la mention de son statut et la référence à l'annulée.

### 5.6 Audit et événements
Voir 5.2 point 4.

### 5.7 Tests
- Cas d'or E-08 et E-38 (domaine, base, API, E2E), E-42 (réutilisation).
- Propriétés : P5 (paiement puis contre-passation ramène chaque ligne à l'état antérieur), P6 (une seule contre-passation, montant opposé) ; requête de contrôle n° 3 = 0 ligne.
- Base : un caissier ne peut pas insérer de `reversal` même en SQL direct avec `immo_app` (garde `payments_before_insert`) ; G-14 ; `UPDATE`/`DELETE` refusés (G-05).
- API : A3 (caissier et gestionnaire : demande seulement), A8 (rejeu de la contre-passation avec la même clé), `auth.reauth_required`, `payment.already_reversed`, E-08 (modification refusée, demande proposée).
- Numérotation : la rectificative prend le numéro suivant sans trou (requête de trous = 0).
- E2E scénario R3 de `TESTING_QA.md` §13.2 (57 000 au lieu de 75 000).

## 6. Critères d'acceptation vérifiables

1. Un caissier qui tente de modifier un paiement validé est refusé et se voit proposer une demande de correction (E-08).
2. Paiement de 57 000 saisi à tort, quittance Q ; le gérant contre-passe avec remplacement de 75 000 : Q passe à « Annulée », le nouveau paiement a une quittance Q' rectificative qui référence Q, avec son propre numéro suivant (E-38) ; l'échéance est revenue à l'état attendu.
3. Une seconde contre-passation du même paiement est refusée (`payment.already_reversed`) ; deux soumissions identiques créent une seule contre-passation.
4. Sans confirmation du mot de passe de moins de 5 minutes, la contre-passation répond `403 auth.reauth_required`.
5. La somme des affectations nettes de chaque ligne est nulle après contre-passation sans remplacement (P5).
6. La référence mobile money d'un paiement contre-passé peut servir à nouveau, sans avertissement (E-42).
7. Les numéros de quittance restent consécutifs sans trou après une correction.

## 7. Sécurité et déploiement à vérifier

Séparation des tâches appliquée **aussi par la base** ; confirmation du mot de passe sur l'action à enjeu ; demandes de correction en rafale visibles dans l'audit (indicateur de fraude à étudier en étape 19, `SECURITY.md` T2) ; migration qui touche `payments_before_insert` relue par un second avis ; livraison respectant le calendrier de `DEPLOYMENT.md` §4.

## 8. Liste de démonstration (Mimis)

1. Comme caissier : encaisser 57 000 FCFA pour un loyer de 75 000 ; tenter de modifier : lire le refus ; demander une correction avec un motif.
2. Comme gérant : voir la demande dans la file, ouvrir « Contre-passer », saisir le mot de passe, corriger en 75 000.
3. Ouvrir la quittance d'origine : « Annulée — remplacée par … » ; ouvrir la nouvelle : « Rectificative de … ».
4. Vérifier dans le journal des paiements que les numéros se suivent et que le solde du locataire est juste.
5. Rejouer la contre-passation : refus.

## 9. Définition de terminé, risques, hors étape

Terminé : critères 1 à 7 verts ; PP-02 vérifiable ; `DOMAINE.md` inchangé ; `API_CONTRACTS.md`, `DECISIONS.md` (décision de rendu de la copie annulée), `CLAUDE.md` à jour.

| Risque | Mesure |
|---|---|
| R-096 différente chez l'agence | Marquée [à confirmer avec l'agence pilote à la découverte] ; se corrige par date d'effet, pas par recalcul |
| Demandes de correction nombreuses | Indicateur dans les événements ; arbitrage à la boucle d'apprentissage |
| Quittance annulée déjà remise au locataire | Message d'information pour l'agence (texte du guide d'accueil, C-14) |

Hors étape : quittance rectificative sans effet sur le montant (DP-64, hors pilote), cloche de notifications (étape 23), correction d'un relevé validé (étape 17), clôture de caisse (étape 22).

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 09
- Dépend de : 07, 08
- Tranche : T1
- Jalon : J2
- F : F09, F10
- R : R-048, R-051, R-096
- E : E-08, E-38
- PP : PP-02
- Contenus : aucun
