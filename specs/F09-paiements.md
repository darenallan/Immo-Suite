# F09 — Paiements et affectations (étape 07, lot 07a et 07b)

| Champ | Valeur |
|---|---|
| Statut | **Rédigée le 04/11/2026, validée par Mimis le 04/11/2026** (recommandations du §10 acceptées, DP-96) |
| Étape | 07, `roadmap/etape-07-encaisser.md` §4 point 1 |
| Dépend de | dossier §16 F09 ; DP-13, DP-20, DP-21, DP-22, DP-23, DP-45, DP-47, DP-49, DP-50, DP-58 ; décision n° 37 (**ouverte**, question 1) ; `DATABASE_SCHEMA.md` `payments`, `allocations`, `idempotency_keys`, `cash_days`, §4.2, §4.5 ; `ARCHITECTURE.md` §4.1 à §4.5 ; `API_CONTRACTS.md` §5.1, §5.3, §6.1, §7.6 ; `UI_UX_DESIGN.md` S04, S11 |
| Règles | R-001, R-033, R-037 à R-047, R-086, R-087, R-088, R-102 [à confirmer avec l'agence pilote à la découverte] ; R-089 (capture de `cash_day` seulement) ; R-023 (partie « déjà payé » reportée de l'étape 06, DP-94) ; R-036 (garde reportée de l'étape 06) |
| Cas chiffrés | E-05, E-06, E-07, E-09, E-25, E-31, E-32, E-33, E-42 |
| Hors fiche | numéro, instantané et montant en lettres de la quittance (F10) ; PDF, envoi WhatsApp, lien de partage (étape 08) ; demande de correction et contre-passation (étape 09 : les colonnes existent dès cette étape) ; clôture de caisse et lecture du SMS (étape 22) ; impayés (étape 10) |

## 1. Objet

Enregistrer un encaissement en une transaction : le paiement, ses affectations aux lignes d'échéance, le crédit éventuel du bail et la quittance numérotée (F10), sans doublon même après une coupure réseau, et sans qu'aucune de ces écritures puisse ensuite être modifiée ou supprimée.

## 2. Règles

| N° | Règle | Source |
|---|---|---|
| 1 | **Montant** : entier de FCFA, 1 à 999 999 999. Désérialisation stricte : `75000.5`, `"75000"`, `1e5`, une valeur négative ou nulle sont refusés (`validation.failed`, code de champ `amount.not_integer` ou `amount.out_of_range`) | R-001, R-037, DP-45, PP-17 |
| 2 | **Date** `paid_on` : la date du jour de l'organisation ou une date passée ; une date future est refusée (`validation.date_in_future`). « Aujourd'hui » vient du serveur (`me.today`, `payment-context.today`) | R-039, DP-50 |
| 3 | **Journée de caisse** `cash_day` : la date locale de la saisie, imposée par la base (déclencheur `payments_before_insert`) ; `cash_day ≥ paid_on`. Aucun avertissement à cette étape (pas de clôture avant l'étape 22) | R-089, DP-23 |
| 4 | **Modes** : `cash` (par défaut), `mobile_money`, `bank_transfer`, `cheque`. Référence externe facultative (1 à 100 caractères, espaces de bord retirés), proposée pour les trois derniers modes | R-040 |
| 5 | **Référence déjà utilisée pour le même mode** dans l'organisation : `409 payment.external_ref_duplicate` (avec le numéro de quittance du paiement existant) qui **bloque** tant que la requête ne porte pas `acknowledge_external_ref_duplicate: true`. Le contrôle se fait sous verrou consultatif `(organisation, mode, référence)` : deux saisies simultanées de la même référence ne passent donc pas toutes les deux sans confirmation. Comparaison sans tenir compte de la casse ni des espaces [question 6] | R-041, R-102, E-42 |
| 6 | **Clé d'idempotence** obligatoire (`Idempotency-Key`, UUID) ; table `idempotency_keys` insérée dans la transaction du paiement avec la réponse rendue. Rejeu identique : même statut et même corps, `Idempotent-Replay: true`, rien de créé. Même clé, autre corps ou autre route : `422 idempotency.key_reused_with_different_body`. Deux requêtes simultanées de même clé : la seconde attend la première (verrou consultatif sur la clé) puis reçoit la réponse enregistrée. Une erreur métier n'est pas mémorisée. Conservation 30 jours (purge quotidienne) | R-042, DP-49, E-07 |
| 7 | **Bail** : en cours (`active` ou `notice`) ; sinon `409 payment.lease_not_in_progress` [question 5]. Le paiement est rattaché au **bail** | R-088, DP-22 |
| 8 | **Payé par** : `tenant` (un locataire **de ce bail** ; par défaut le locataire principal) ou `third_party` (nom obligatoire, téléphone facultatif normalisé par `immo_domain::phone`). Un locataire d'un autre bail est refusé (`validation.failed`, `payer.not_on_lease`) | R-088, E-33 |
| 9 | **Lignes ouvertes** : lignes d'échéance non annulées du bail dont le reste (`dû − affecté`) est positif, triées par **date d'échéance** de la ligne (arriérés compris, ils sont datés), puis à date égale `rent`, `charges`, `exceptional` (puis `arrears`, puis identifiant, pour un ordre total) [question 7] | R-043, R-087, DP-21 |
| 10 | **Affectation par défaut** (`allocations: null`) : `allocate(montant, lignes ouvertes)` remplit chaque ligne dans l'ordre jusqu'à épuisement. Fonction pure du domaine, déterministe quel que soit l'ordre d'entrée (P7) | R-043, R-087, E-32 |
| 11 | **Affectation ajustée** : liste `{ installment_line_id, amount }` ; chaque ligne doit être ouverte, de ce bail, non annulée, et recevoir au plus son reste ; somme ≤ montant ; sinon `422 payment.allocation_invalid`. Si l'état a changé depuis l'aperçu (ligne soldée entre-temps) : `409 payment.stale_allocation` avec le nouvel aperçu. Audit `allocation.adjusted` quand la répartition diffère de celle par défaut | R-043, R-044 |
| 12 | **Excédent** (montant > reste dû total des lignes ouvertes) : exige `surplus: "advance"`, sinon `422 payment.surplus_not_confirmed` (`params.surplus_amount`). L'excédent s'affecte aux **échéances futures déjà générées**, dans l'ordre de la règle 9 ; ce qui reste est le **crédit du bail** (part non affectée du paiement) [question 3] | R-046, R-086, E-06, E-31 |
| 13 | **Crédit du bail** = somme sur les paiements du bail de `montant − affectations nettes` (vue `v_lease_credit`). À chaque **création d'échéance** (activation, tâche quotidienne, prolongation), le crédit s'affecte aux nouvelles lignes, en consommant d'abord la part non affectée du **paiement le plus ancien** ; ces affectations portent le `payment_id` d'origine et `created_by` nul (système) [question 4]. Bail terminé : le crédit reste visible, réglé hors flux (étape 21) | R-086, DP-20, E-31 |
| 14 | **Seuil de confirmation** : un montant **strictement supérieur** à `org_settings.payment_confirm_threshold` (1 000 000 par défaut) exige `confirm_high_amount: true`, sinon `422 payment.confirmation_required` (`params.threshold`) [question 2] | R-038, E-09 |
| 15 | **Statut de l'échéance** après paiement : celui de F08 règle 11 (une seule fonction) : « partielle » le jour J s'il reste dû, « en retard » à partir de J+1 même partiellement payée | R-045, DP-13, E-05, E-25 |
| 16 | **Immuable** : `payments`, `allocations`, `receipts` en ajout seul (`forbid_mutation`, `forbid_truncate`) ; le rôle `immo_app` n'a que SELECT et INSERT. `PUT`, `PATCH`, `DELETE /payments/{id}` → `405 payment.immutable` | R-047, DP-47, E-08 |
| 17 | **Concurrence** : la transaction prend `pg_advisory_xact_lock(hashtextextended(lease_id::text, 0))` **avant** de relire les soldes ; le contrôle différé `chk_allocation` reprend le même verrou (réentrant). Deux caissiers sur le même bail : le second est calculé sur l'état après le premier. Niveau `READ COMMITTED` vérifié par un test [question 1] | n° 37, DB §4.5 |
| 18 | **Ordre dans la transaction** : contrôles métier ⇒ verrou du bail ⇒ relecture des lignes ouvertes ⇒ paiement ⇒ affectations ⇒ quittance (F10 : compteur **en dernier**, instantané) ⇒ audit ⇒ clé d'idempotence et réponse. Aucun appel au stockage de fichiers ni au moteur PDF [question 9] | API §5.3, DP-46 |
| 19 | **Gardes reportées de l'étape 06** : (a) l'annulation d'une ligne qui porte des affectations nettes est refusée (`installment.line_has_payments`) ; (b) un avenant dont la date d'effet couvre une échéance déjà payée en partie ou en totalité est refusé (`lease.amendment_paid_installment`, R-023) ; (c) une affectation vers une ligne annulée est refusée (base) | R-023, R-036, DP-94 |
| 20 | **Droits** : `payment.record` pour enregistrer et prévisualiser (gérant, caissier, propriétaire ; gestionnaire et collaborateur selon le réglage) ; `payment.read` pour lire (tous sauf l'agent terrain). Matrice A3 mise à jour | F02, §4.6 |
| 21 | **Audit** dans la transaction : `payment.recorded`, `receipt.issued`, `allocation.adjusted` (si ajusté). **Événements** écrits par le serveur (liste blanche) : `payment_recorded` (durée depuis l'ouverture du formulaire, mode, partiel, avance, crédit, nombre d'essais, retard de saisie en jours), `payment_idempotent_replay`, `payment_external_ref_duplicate`. Événements d'écran : `payment_form_opened`, `payment_form_abandoned` ; `validation_error` porte le code de règle | étape §5.6 |

## 3. Données (migration de l'étape)

1. `payments`, `allocations`, `idempotency_keys`, `cash_days` (**vide**) **comme dans `DATABASE_SCHEMA.md`**, y compris les colonnes de contre-passation (`reverses_payment_id`, `reversal_reason`, `replaces_payment_id`) et `correction_requests`, inutilisées avant l'étape 09 ; gardes `payments_amount_sign`, `payments_cash_day_after_paid`, `payments_payer_fields`, `payments_reversal_fields` ; déclencheurs `payments_before_insert` (horodatage, `cash_day`, date non future par `org_local_date`), `allocations_before_insert`, `chk_allocation` (différé), `chk_reversal_cancels` (différé).
2. Écriture seule : `forbid_mutation`, `forbid_truncate` sur `payments`, `allocations`, `cash_days` ; `idempotency_keys` : SELECT, INSERT, DELETE (purge de 30 jours).
3. Vues : `v_installment_balances` complétée (`allocated_amount` = somme des affectations nettes de la ligne), `v_payment_credit` (par paiement : montant − affectations nettes), `v_lease_credit` (par bail).
4. Registres de l'étape 05 : `cross_org_registry` (clés composites), `purge_order` (positions après 707), RLS `org_isolation` forcée sur chaque table.
5. Migration de garde : second avis (C-19 élargi) et contrôle nocturne des déclencheurs (`DATABASE_SCHEMA.md` §4.2) étendu aux nouvelles tables.

## 4. Domaine (`crates/domain`, tests d'or d'abord)

- `payment::validate_amount(i64) -> Result<Fcfa, …>` (bornes).
- `allocation::open_lines_order(lines)` (règle 9) et `allocation::allocate(amount, &[OpenLine]) -> Allocation { parts, surplus }` : déterministe, somme des parts ≤ montant, chaque part ≤ reste de sa ligne (P1, P2, P4, P7).
- `allocation::check_manual(amount, &[OpenLine], &[Part])` (règle 11).
- `allocation::apply_credit(credits_by_payment, new_lines)` (règle 13).
- Aucune lecture de l'heure, aucune arithmétique implicite (lints de l'étape 05).

## 5. API (`API_CONTRACTS.md` §6.1, §7.6)

| Route | Droit | Remarque |
|---|---|---|
| `GET /leases/{id}/payment-context` | `payment.record` | bail, lot, locataires, solde (`remaining_amount`, `overdue_amount`, `credit_amount`), lignes ouvertes dans l'ordre d'affectation, `suggested_amount` (reste de l'échéance non soldée la plus ancienne), `default_method`, `today`, `confirm_threshold` ; un seul aller-retour |
| `POST /payments/preview` | `payment.record` | `{ lease_id, amount, allocations? }` → parts, `surplus_amount`, `requires_surplus_confirmation`, `requires_high_amount_confirmation`, `installments_after` ; aucune écriture, aucune clé |
| `POST /payments` | `payment.record` | corps de §6.1 plus `acknowledge_external_ref_duplicate` ; `201` → `{ payment, receipt, installments, warnings }` |
| `GET /payments` | `payment.read` | filtres `from`, `to`, `method`, `tenant_id`, `unit_id`, `lease_id`, `received_by`, `q` (numéro de quittance, référence) ; tri `-recorded_at` ; 25 par page avec curseur |
| `GET /payments/{id}` | `payment.read` | affectations, quittance, statut dérivé `validated` (`reversed` à l'étape 09) |
| `PUT`, `PATCH`, `DELETE /payments/{id}` | — | `405 payment.immutable` (enregistrées dans `ROUTES`, comptées par A3) |
| `GET /leases/{id}/payments` | `payment.read` | par bail ; l'onglet Paiements de S09 agrège les baux du locataire |

Erreurs : celles de §6.1 (`idempotency.key_missing`, `idempotency.key_reused_with_different_body`, `validation.failed` dont `validation.date_in_future`, `payment.surplus_not_confirmed`, `payment.confirmation_required`, `payment.allocation_invalid`, `payment.external_ref_duplicate`, `payment.lease_not_in_progress`, `payment.stale_allocation`, `auth.forbidden`, `resource.not_found`), plus `installment.line_has_payments` et `lease.amendment_paid_installment` (règle 19). Messages en français côté front (`fr.errors`), jamais le `detail` du serveur. `openapi.json` régénéré.

## 6. Écrans

- **S04 Encaisser** (`/encaisser`, et « Encaisser » depuis S09 et S10) : recherche unique, carte du locataire et du bail avec solde et crédit ; montant (clavier numérique, montant proposé), date (`today` du serveur), mode (« Espèces » par défaut), référence (si le mode n'est pas « Espèces »), « payé par » (locataires du bail, ou « Un tiers » : nom, téléphone), note ; aperçu d'affectation (du serveur, rafraîchi à la modification du montant, 300 ms de délai), modifiable ligne par ligne ; bouton « Encaisser et envoyer la quittance » collé en bas sur mobile (52 px). Clé d'idempotence créée **à l'ouverture** du formulaire, réutilisée à chaque nouvel essai ; coupure : « Connexion perdue. Vos informations sont conservées. Réessayez dès que le réseau revient. Si le paiement a déjà été enregistré, il ne sera pas créé deux fois. » Fenêtres de confirmation : seuil (R-038), avance (R-046), référence déjà utilisée (R-102, avec le numéro de la quittance existante). Succès : aperçu de la quittance (S25 en HTML, F10) avec son numéro, « solde restant », « avance », et « Encaisser un autre loyer » (nouvelle clé). Paquet S04 chargé à part (import dynamique).
- **S11 Paiements et quittances** (`/paiements`) : journal (date, locataire, lot, montant, mode, référence, reçu par, n° de quittance, statut « Validé »), filtres, détail (affectations, quittance) ; aucun bouton de modification ni de suppression.
- **S09** : onglet « Paiements » ; **S10** : bouton « Encaisser » sur un bail en cours et colonne « payé » de l'échéancier.

## 7. Critères (repris de l'étape §6) et tests

- Cas d'or E-05, E-06, E-07, E-09, E-25, E-31, E-32, E-33, E-42 aux niveaux domaine, base et API.
- Propriétés P1 à P8, P16 (domaine ; 10 000 cas la nuit, 256 par défaut sur le poste).
- Base : G-03, G-04, G-05 (UPDATE, DELETE, TRUNCATE refusés pour `immo_app`, `immo_platform`, `immo_backup`), G-06, G-11 à G-13 ; isolation entre organisations sur chaque nouvelle table.
- Concurrence : 4 × 50 validations sur un même bail sans dépassement ; même clé 3 fois en parallèle → un paiement ; réponse perdue puis nouvel essai ; même clé, contenu différent ; deux caissiers sur le même locataire ; `SHOW transaction_isolation` = `read committed` sur la connexion applicative.
- API : A3 (matrice), A4, A8 à A10 (`75000.5`, hors bornes, date future, champs inconnus refusés).
- Front : S04 (clé conservée sur nouvel essai, nouvelle clé après « Encaisser un autre loyer », confirmations, montants envoyés en entiers) ; S11 en lecture seule.
- Pile réelle (390 et 1440 px) : démonstration §8 de l'étape (50 000, 100 000 avec avance, référence en doublon, coupure puis nouvel essai, seuil, tiers).
- Performance : `POST /payments` p95 ≤ 300 ms sans PDF (mesure sur le poste, budget de `ARCHITECTURE.md` §12.1).

## 8. Cas limites

| Cas | Comportement |
|---|---|
| Paiement exactement égal au reste dû total | toutes les lignes soldées, aucun excédent, `surplus` non exigé |
| Montant supérieur à toutes les échéances générées | `surplus: "advance"` exigé ; échéances futures générées remplies, le reste en crédit du bail |
| Aucune ligne ouverte (tout est payé) | le montant entier est un excédent : `surplus: "advance"` exigé, puis échéances futures ou crédit |
| Coupure **avant** validation | rien n'est écrit ; le nouvel essai s'exécute normalement |
| Coupure **après** validation (réponse perdue) | le nouvel essai rend la réponse enregistrée (`Idempotent-Replay: true`) |
| Annulation forcée de la transaction après l'attribution du numéro | tout est annulé, numéro compris (pas de trou, F10) |
| Bail brouillon ou terminé | `409 payment.lease_not_in_progress` |
| Ligne annulée entre l'aperçu et la validation | `409 payment.stale_allocation` avec le nouvel aperçu |
| Paiement daté d'avant le début du bail | accepté (avance versée à la signature) [question 8] |

## 9. Ce qui n'est pas fait à cette étape

Contre-passation, demande de correction et réutilisation de la référence d'un paiement contre-passé (étape 09) ; avertissement de journée de caisse clôturée (étape 22) ; lecture du SMS mobile money (F25, T3) ; PDF et envoi (étape 08).

## 10. Questions pour Mimis (avec recommandation)

1. **Décision n° 37 (ouverte)** : contrôles des affectations. *Recommandation : contrôles différés en base sous verrou consultatif du bail, en plus des contrôles du code, niveau `READ COMMITTED` exigé et vérifié par un test* (`DATABASE_SCHEMA.md` §4.5). À enregistrer en DP-96 avant le code.
2. **Seuil de confirmation (R-038)** : *recommandation : garder 1 000 000 FCFA par défaut, réglable dans Paramètres (`payment_confirm_threshold`, existe depuis l'étape 03), confirmation exigée au-dessus, strictement*.
3. **Excédent (R-046)** : la règle dit « affectée à l'échéance suivante ». *Recommandation : l'excédent remplit les échéances futures déjà générées dans l'ordre, puis le reste devient crédit du bail*. L'autre lecture (seulement l'échéance suivante, le reste en crédit) donne le même total mais un affichage différent.
4. **Crédit appliqué à la génération (R-086, E-31)** : *recommandation : affectation « système » (`created_by` nul) qui porte le paiement d'origine, le plus ancien d'abord ; aucune nouvelle quittance* (la quittance du paiement d'origine mentionnait déjà l'avance).
5. **Bail en préavis ou terminé** : *recommandation : paiement accepté pour `active` et `notice`, refusé pour `draft` et `ended`* (comme le contrat) ; l'encaissement d'un arriéré après la fin du bail viendra avec l'étape 21.
6. **Doublon de référence (R-041)** : *recommandation : comparer la référence sans tenir compte de la casse ni des espaces, par mode, sur toute l'organisation et sans limite de durée* ; la réutilisation après contre-passation (R-102) viendra avec l'étape 09.
7. **Ordre à date égale entre `exceptional` et `arrears`** : DP-21 ne cite pas les arriérés à date égale. *Recommandation : loyer, charges, exceptionnelles, puis arriérés, puis identifiant* (ordre total pour P7). Un arriéré est daté et passe en général avant par sa date.
8. **Date de paiement antérieure au début du bail** : *recommandation : acceptée (dépôt ou avance versés à la signature), sans borne basse autre que celle du schéma*.
9. **Tâche de rendu PDF** : le moteur n'est pas choisi (n° 7) et le PDF est à l'étape 08. *Recommandation : ne pas créer la tâche `render_receipt` à cette étape ; `receipt.pdf.state` vaut `pending` ; l'étape 08 rendra les quittances sans fichier par une reprise unique*. Sinon le travailleur échouerait cinq fois par quittance puis alerterait.
10. **Contenu C-13 (téléphone d'entrée de gamme)** : la mesure des 30 secondes (critère 9) se fera sur Chrome en émulation de téléphone et « 4G lente » si le téléphone n'est pas là. *Recommandation : l'accepter comme mesure préliminaire ; la mesure officielle reste à l'étape 13*.
