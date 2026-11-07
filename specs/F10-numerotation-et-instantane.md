# F10 — Quittance : numérotation, instantané, montant en lettres, aperçu (étape 07, partie de F10)

| Champ | Valeur |
|---|---|
| Statut | **Rédigée le 04/11/2026, validée par Mimis le 04/11/2026** (recommandations du §9 acceptées, DP-96) |
| Étape | 07, `roadmap/etape-07-encaisser.md` §4 point 2 |
| Dépend de | dossier §16 F10 ; DP-15, DP-37, DP-46, DP-47, DP-61, DP-64 ; décision n° 36 ; `DATABASE_SCHEMA.md` `receipt_counters`, `receipts`, `receipt_cancellations`, §4.3 ; `ARCHITECTURE.md` §4.4 ; `API_CONTRACTS.md` §6.1, §7.6 ; `UI_UX_DESIGN.md` S25, « Quittance : mise en page imprimable » ; F09 |
| Règles | R-049 (partie : une quittance par paiement), R-050, R-051 (immuable), R-052 [À VÉRIFIER AVEC UN JURISTE, décision n° 12], R-053 [À VÉRIFIER AVEC UN JURISTE], R-088 (mention du tiers) |
| Cas chiffrés | E-10, E-11, E-27 |
| Hors fiche | PDF A4 et A5, fichiers, envois, liens de partage, page publique S26 (étape 08) ; annulation par contre-passation et quittance rectificative (étape 09 : `receipt_cancellations` est créée vide) ; mentions légales complètes (contenu C-08, étape 08) |

## 1. Objet

Chaque paiement validé reçoit, **dans sa transaction**, une quittance numérotée sans trou et figée dans un instantané complet. L'écran et, à l'étape 08, le PDF se rendent depuis cet instantané : la quittance ne change jamais, même si le bail, le locataire ou les paramètres changent ensuite.

## 2. Règles

| N° | Règle | Source |
|---|---|---|
| 1 | **Une quittance par paiement** (`UNIQUE (org_id, payment_id)`), créée dans la transaction du paiement (F09 règle 18), jamais à part | R-049, DP-15 |
| 2 | **Compteur** `receipt_counters (org_id, series, year, last_number)` incrémenté par `INSERT … ON CONFLICT DO UPDATE SET last_number = last_number + 1 RETURNING last_number`, **en dernier** dans la transaction, après tous les contrôles métier ; la ligne du compteur reste verrouillée jusqu'au `COMMIT`, ce qui sérialise les émissions d'une même organisation, série et année | DP-46, DB §4.3 |
| 3 | **Gardes** : le compteur démarre à 1 et n'avance que de 1 (`receipt_counters_guard`) ; la quittance porte exactement le numéro du compteur et suit la précédente (`receipts_before_insert`) ; `issued_at` imposé par la base ; série compatible avec l'état de l'organisation | R-050, G-07 à G-09 |
| 4 | **Série** : `trial` tant que l'organisation est en `setup`, `official` après la bascule en `live` ; la série `official` démarre à 000001 | DP-61, n° 36 |
| 5 | **Année** = année de la **date d'émission** (`issued_on` = date locale de la saisie, fuseau de l'organisation), pas de la date de paiement : un paiement du 31/12/2026 saisi le 02/01/2027 est numéroté en 2027. La séquence repart à 000001 chaque année | DP-61, R-050, E-27 |
| 6 | **Numéro** : `{préfixe}-{année sur 4 chiffres}-{séquence sur 6 chiffres}`, préfixe `org_settings.receipt_prefix` (2 à 8 lettres majuscules ou chiffres, `REC` par défaut) ; en série d'essai, `ESSAI-` devant : `ESSAI-HOR-2026-000001` [question 1]. Contraintes `receipts_number_format` et `receipts_trial_label` | DP-37, DP-61 |
| 7 | **Échec après attribution** (contrôle différé, coupure, annulation forcée) : toute la transaction est annulée, numéro compris ; le paiement suivant reçoit ce numéro. **Aucun trou** | DP-46, P13 |
| 8 | **Contrôle nocturne** : la requête de `DATABASE_SCHEMA.md` §4.3 (`count(*) <> max(sequence)` par organisation, série et année) doit renvoyer zéro ligne ; sinon alerte de gravité S1 `receipts.numbering_gap` (journal et `platform_events`) par une tâche quotidienne `receipts.check_gaps` [question 5] | DP-46, étape §5.2 point 5 |
| 9 | **Instantané** (`receipts.snapshot`, JSON figé, versionné `"v": 1`) : voir §3. Aucun champ recalculé à la lecture | DP-47, R-052 |
| 10 | **Montant en lettres** : `immo_domain::words::amount_in_words(n)` pour 1 à 999 999 999, orthographe **traditionnelle** (règles du §4), suivi de « francs CFA » ; E-11 : 75 000 → « soixante-quinze mille francs CFA » | R-053, DP-64, E-11 |
| 11 | **Immuable** : `receipts` en ajout seul ; aucune route de modification. Statut dérivé : `issued` (`cancelled` à l'étape 09, par `receipt_cancellations`) | R-051, DP-47 |
| 12 | **Isolation du compteur** : seul le module `receipts` écrit `receipt_counters` et `receipts` ; vérifié en CI par une recherche de `INTO receipts`, `INTO receipt_counters` et `nextval` hors de ce module | étape §7 |
| 13 | **Droits** : lecture `receipt.issue` ou `payment.read` ; aucune route d'écriture propre (l'émission passe par `POST /payments`) | §7.6 |
| 14 | **Journaux** : jamais de numéro de quittance ni de montant dans les journaux techniques ; l'audit `receipt.issued` porte l'identifiant et le numéro (journal d'audit de l'organisation) | étape §7, `SECURITY.md` |

## 3. Contenu de l'instantané (version 1)

| Bloc | Champs | Source au moment de l'émission |
|---|---|---|
| Émetteur | nom de l'organisation, plan, en-tête (`letterhead`), identifiants légaux (`legal_ids`), mentions complémentaires (`extra_mentions`) | `organizations`, `org_settings` |
| Quittance | numéro, série, filigrane « ESSAI » si `trial`, date d'émission (JJ/MM/AAAA), émise par (nom) | compteur, `issued_on`, utilisateur |
| Locataires | noms du ou des locataires du bail, principal en premier | `lease_tenants` |
| Payé par | « le locataire » ou le nom du tiers (et son téléphone s'il est saisi) ; mention « Payé par {nom} pour le compte de {locataire} » pour un tiers | F09 règle 8, R-088, E-33 |
| Lot | référence du lot, nom du bien, adresse, quartier, ville ; usage ; « TVA incluse » si bail commercial avec `vat_included` | `units`, `properties`, `leases` |
| Paiement | montant en chiffres (« 75 000 FCFA ») et en lettres, date de paiement, mode, référence externe | `payments` |
| Ventilation | par échéance : période (« novembre 2026 »), puis par ligne : nature (loyer, charges, exceptionnelle, arriéré, libellé), montant affecté | `allocations` du paiement |
| Avance | montant affecté aux échéances futures, plus le crédit du bail créé par ce paiement | F09 règles 12 et 13 |
| Solde restant | reste dû du bail sur les échéances **échues ou dues** à la date d'émission, après ce paiement [question 3] | `v_installment_balances` |

Le paiement de remplacement (étape 09) ajoutera `replaces_receipt_number` ; le champ n'existe pas en version 1.

## 4. Montant en lettres (orthographe traditionnelle, DP-64)

- 1 à 16 : un, deux, trois, quatre, cinq, six, sept, huit, neuf, dix, onze, douze, treize, quatorze, quinze, seize ; 17 à 19 : dix-sept, dix-huit, dix-neuf.
- Dizaines : vingt, trente, quarante, cinquante, soixante, soixante-dix, quatre-vingt(s), quatre-vingt-dix ; trait d'union entre dizaine et unité (vingt-deux, quatre-vingt-dix-sept), sauf avec « et » : vingt et un, trente et un, quarante et un, cinquante et un, soixante et un, soixante et onze ; mais quatre-vingt-un, quatre-vingt-onze.
- « vingt » et « cent » prennent un **s** quand ils sont multipliés et terminent le nombre : quatre-vingts, deux cents ; pas de s s'ils sont suivis d'un autre nombre ou de « mille » : quatre-vingt-deux, deux cent un, deux cent mille, quatre-vingt mille ; un s devant « million(s) », qui est un nom : deux cents millions, quatre-vingts millions.
- « cent » et « mille » seuls, sans « un » : cent, mille, cent mille ; « mille » est invariable : deux mille, trois cent mille.
- « million » est un nom : un million, deux millions ; espaces (pas de trait d'union) autour de cent, mille, million.
- Unité : « francs CFA » ; « un franc CFA » pour 1 ; « de francs CFA » après un nombre exact de millions (« deux millions de francs CFA », mais « deux millions cinq cent mille francs CFA ») [question 2].
- Exemples de référence (tests d'or) : 1 « un franc CFA » ; 21 « vingt et un francs CFA » ; 71 « soixante et onze » ; 80 « quatre-vingts » ; 81 « quatre-vingt-un » ; 200 « deux cents » ; 201 « deux cent un » ; 1 000 « mille » ; 75 000 « soixante-quinze mille » ; 80 000 « quatre-vingt mille » ; 200 000 « deux cent mille » ; 1 000 000 « un million de francs CFA » ; 1 250 000 « un million deux cent cinquante mille » ; 999 999 999 « neuf cent quatre-vingt-dix-neuf millions neuf cent quatre-vingt-dix-neuf mille neuf cent quatre-vingt-dix-neuf ».
- Propriété P14 : un analyseur indépendant (écrit à part dans les tests, sans partager de table avec la fonction) relit le texte et retrouve le nombre, de 1 à 999 999 999.

## 5. Domaine et service

- `immo_domain::receipt::number(prefix, year, sequence, series) -> String` et sa réciproque pour les tests ; `immo_domain::words::amount_in_words(u32) -> String`.
- `crates/app/src/receipts.rs` : `issue(tx, payment, snapshot_input) -> Receipt` (compteur, numéro, instantané, insertion), appelé en dernier par `ledger::record_payment` ; seule écriture de `receipt_counters` et `receipts`.
- Tâche quotidienne `receipts.check_gaps` (fonction `SECURITY DEFINER` qui ne renvoie que des identifiants d'organisation, comme `orgs_with_leases_in_progress()` de l'étape 06).

## 6. API

| Route | Droit | Remarque |
|---|---|---|
| `GET /receipts` | `receipt.issue` ou `payment.read` | filtres `q` (numéro), `from`, `to` ; tri `-issued_at` ; 25 par page |
| `GET /receipts/{id}` | idem | quittance complète (`number`, `series`, `kind`, `status`, `issued_on`, `amount`, `amount_in_words`, `remaining_after`, `advance_amount`, `snapshot`, `pdf: { state: "pending", papers: ["a4","a5"], sha256: null }`) |

La réponse de `POST /payments` porte la même quittance (`receipt`).

## 7. Écran S25 (aperçu HTML)

Rendu depuis `receipt.snapshot` seulement, sans calcul : en-tête de l'émetteur, numéro et date, filigrane « ESSAI » en série d'essai, locataires, lot, période(s), ventilation, montant en chiffres et en lettres, mode et référence, « Payé par » pour un tiers, solde restant, avance. Mise en page lisible à 390 px et imprimable par le navigateur à cette étape (le PDF A4/A5 vient à l'étape 08). Boutons « Envoyer sur WhatsApp » et « Télécharger » **absents** (pas « grisés ») jusqu'à l'étape 08. Ouvert après un encaissement (S04) et depuis le détail d'un paiement (S11).

## 8. Critères et tests

- E-10 : deux paiements successifs → numéros consécutifs ; E-27 : au 31/12/2026 `…-2026-000812`, au 02/01/2027 `…-2027-000001` et la quittance de 2026 inchangée ; E-11 : 75 000 en lettres.
- Base : G-07, G-08, G-09 ; numéro 9 demandé à la place du 4 refusé ; compteur ramené en arrière ou avancé de 5 refusé ; annulation après attribution → numéro réattribué ; série incompatible avec l'état de l'organisation refusée ; UPDATE, DELETE, TRUNCATE de `receipts` refusés pour les trois rôles ; requête de trous = 0 ligne.
- Concurrence (prototype (c) rejoué) : 4 × 50 validations simultanées sur 3 organisations avec 10 % d'annulations forcées → 1 à N sans trou ni doublon par organisation (P13) ; 20 × 500 la nuit (CI, hors poste).
- Domaine : tests d'or du §4, propriété P14 (10 000 cas la nuit), numéro et sa réciproque.
- Front : S25 affiche les champs de l'instantané, le filigrane d'essai, la mention du tiers ; aucun calcul.

## 9. Questions pour Mimis (avec recommandation)

1. **Numéro d'essai** : *recommandation : `ESSAI-{préfixe}-{année}-{séquence}`* (le préfixe de l'organisation reste lisible, et le format satisfait les deux contraintes du schéma). Sinon `ESSAI-{année}-{séquence}` sans préfixe.
2. **« de » après un nombre exact de millions** (« un million de francs CFA ») et **singulier pour 1** (« un franc CFA ») : *recommandation : les appliquer, c'est la grammaire courante* ; à faire relire avec les autres mentions par le juriste (R-053 reste [À VÉRIFIER AVEC UN JURISTE]).
3. **« Solde restant » de la quittance** : *recommandation : reste dû du bail sur les échéances échues ou dues à la date d'émission, après ce paiement* (E-05 : 25 000) ; les échéances à venir n'y entrent pas. Sinon : reste dû des seules échéances touchées par le paiement.
4. **Mentions de l'émetteur dans l'instantané** : *recommandation : copier à l'émission l'en-tête, les identifiants légaux et les mentions complémentaires de `org_settings` tels qu'ils sont* ; les mentions obligatoires (C-08) seront fixées à l'étape 08 sans toucher aux quittances déjà émises.
5. **Contrôle des trous** : *recommandation : tâche quotidienne `receipts.check_gaps` à 03 h 00 locale, alerte S1 au journal et dans `platform_events`* ; en essai, une alerte suffit (pas de blocage de l'encaissement).
