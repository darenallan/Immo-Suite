# Immo Suite — Traçabilité

| Champ | Valeur |
|---|---|
| Statut | Document **généré** par script à partir des documents propriétaires ; ne pas l'éditer à la main, le régénérer après toute modification des sources |
| Version | 0.2 |
| Date | 30/10/2026 |
| Dépend de | Dossier de cadrage v1.0 (priorités), `CONTEXTE.md` (tranches), `UI_UX_DESIGN.md` (écrans), `API_CONTRACTS.md` §7 (routes), `DATABASE_SCHEMA.md` §3 et §4 (tables et gardes), `DOMAINE.md` (R-001 à R-110, E-01 à E-48), `TESTING_QA.md` (tests) |
| Alimente | `ROADMAP.md` (colonne « Étape » : étape de `roadmap/` qui livre la ligne), `TESTING_QA.md` §2.1 (couverture des règles), `AUDIT_COHERENCE_2.md` (constats C2-031 et C2-032) |

La colonne « Étape » (ajoutée le 30/10/2026) donne l'étape de `roadmap/` qui livre la fonctionnalité ou applique et teste la règle ; elle est lue dans la section « Couverture » de chaque `roadmap/etape-NN-*.md` (« — » : aucune étape, F38 ou hors V1).

Deux matrices : fonctionnalité → écran → route → table → test (section 1), règle → base → API → écran → test (section 2). Elles montrent ce qui existe sur le papier ; elles ne prouvent pas qu'un test passe.

## 1. Matrice F01 à F40

« Statut au pilote » : **Au démarrage** = T1 et T2 (le pilote démarre à la fin de T2, DP-29) ; **Pendant** = T3, livré pendant le pilote ; **Hors V1** = non planifié. Colonne « Manque » : ce qui reste à spécifier ou à créer pour que la ligne soit complète.

| F | Fonctionnalité | Prio | Tranche | Écran(s) | Endpoint(s) | Table(s) | Test(s) | Statut au pilote | Manque | Étape |
|---|---|---|---|---|---|---|---|---|---|---|
| F01 | Comptes, organisation et plans | P0 | T1 | S01, S02, S17, S18 | POST /auth/login [P/T1]; POST /auth/totp/verify [P/T1]; POST /auth/logout (+1) [P/T1]; GET /me [P/T1] … (+11 routes) | organizations, users, memberships, invitations, sessions, login_attempts, password_resets, org_settings | ISO-1 à ISO-9, A1, A2, A11, PP-08 à PP-10, PP-17, PP-31, E-47, E-48, E2E transverses | Au démarrage |  | 02, 03 |
| F02 | Rôles et permissions | P0 | T1 | S17, S18 | GET /me [P/T1]; GET /users [P/T1]; PATCH /users/{membership_id} [P/T1]; POST /users/{membership_id}/disable (+1) [P/T1] … (+2 routes) | memberships, role_settings, unit_assignments | A3, A5, G-16, PP-11 | Au démarrage |  | 02, 03 |
| F03 | Journal d'audit | P0 | T1 | S17, S21 | GET /audit [P/T1]; PUT, PATCH, DELETE /audit/{id} [P/T1] | support_grants, audit_log | G-05, ISO-4, PP-01, règle CODING §7 n° 9 | Au démarrage |  | 02, 03 |
| F04 | Biens et lots | P0 | T1 | S06, S07 | GET /properties [P/T1]; POST /properties [P/T1]; GET /properties/{id} (+1) [P/T1]; POST /properties/{id}/archive [P/T1] … (+6 routes) | properties, units | E2E-P5 | Au démarrage |  | 05 |
| F05 | Propriétaires mandants et mandats | P0 (Agence) | T2 | S12, S13 | GET /owners (+1) [P/T2]; GET /owners/{id} (+1) [P/T2]; GET /owners/{id}/payout-details [P/T2]; GET /mandates (+1) [P/T2] … (+3 routes) | owners, mandates, mandate_units, mandate_rate_changes | E-16, E-17, E2E-P3 | Au démarrage |  | 05, 16 |
| F06 | Fiche locataire | P0 | T1 | S08, S09 | GET /tenants [P/T1]; POST /tenants [P/T1]; GET /tenants/{id} (+1) [P/T1]; GET /tenants/{id}/leases (+2) [P/T1] … (+3 routes) | tenants | G-19 (R-017, R-018), A10, E2E-P1 | Au démarrage |  | 05 |
| F07 | Baux | P0 | T1 | S07, S10 | GET /units/{id}/leases (+2) [P/T1]; GET /leases [P/T1]; POST /leases [P/T1]; GET /leases/{id} (+1) [P/T1] … (+3 routes) | leases, lease_tenants, lease_commercial_terms, lease_amendments | E-01, E-02, E-35, E-36, E-39, E-40, E-44 | Au démarrage |  | 06 |
| F08 | Échéancier automatique | P0 | T1 | S10 | POST /leases/{id}/activate [P/T1]; GET /leases/{id}/installments [P/T1]; POST /leases/{id}/installment-lines [P/T1]; POST /installment-lines/{id}/void [P/T1] | installments, installment_lines, installment_line_voids, jobs | E-03, E-04 | Au démarrage |  | 06 |
| F09 | Enregistrement des paiements | P0 | T1 | S03, S04, S09, S11 | GET /leases/{id}/payment-context [P/T1]; GET /payments [P/T1]; GET /payments/{id} [P/T1]; POST /payments/preview [P/T1] … (+4 routes) | payments, allocations, correction_requests, idempotency_keys | E-05 à E-08, E-42, E2E-P1 | Au démarrage |  | 07, 09 |
| F10 | Quittances PDF numérotées | P0 | T1 | S04, S11, S17, S25, S26 | POST /payments [P/T1]; POST /payments/{id}/reversal [P/T1]; GET /receipts [P/T1]; GET /receipts/{id} [P/T1] … (+6 routes) | payments, receipt_counters, receipts, receipt_cancellations, receipt_files, receipt_sends, share_links, org_settings, jobs | E-10, E-11, E-12, E-27, E-38, E-48, TESTING §6 | Au démarrage |  | 07, 08, 09 |
| F11 | Tableau de bord des impayés | P0 | T1 | S03, S05 | GET /home [P/T1]; GET /arrears [P/T1] | vues `v_installment_*` (calcul, pas de table) | E-13, E-13b, E-25, E-43, P9, P10, E2E-P2 | Au démarrage |  | 10 |
| F12 | Relances assistées | P0 | T1 | S03, S05, S09, S17 | GET /home [P/T1]; GET /reminders [P/T1]; GET /reminders/due [P/T1]; POST /reminders/draft (+1) [P/T1] … (+5 routes) | message_templates, reminders, reminder_installments, promises | E-14, E-15, E-41, E2E-P2 | Au démarrage |  | 15 |
| F13 | Compte rendu propriétaire | P0 (Agence) | T2 | S12, S26 | GET /statements [P/T2]; GET /statements/preview [P/T2]; POST /statements (+4) [P/T2]; GET /statements/{id} (+1) [P/T2] … (+2 routes) | share_links, statements, statement_versions, statement_reopenings, statement_payment_links, statement_expense_links, payouts, expense_imputation_changes | E-16 à E-19, E-26, E-28 à E-30, E-43, E-45, E-46, E2E-P3 | Au démarrage |  | 17 |
| F14 | Import guidé Excel / CSV | P0 | T2 | S02, S16 | GET /import-batches [P/T2]; GET /import-batches/{id}/rows [P/T2]; GET /import-mappings [P/T2]; GET /import-template [P/T2] | import_batches, import_rows, import_mappings | E-20, E-21, E2E-P4 | Au démarrage |  | 12, 18 |
| F15 | Journal des travaux et dépenses par lot | P1 (V1) | T3 (min. T2) | S07, S12, S14 | GET /units/{id}/leases (+2) [P/T1]; GET /interventions [P/T2]; POST /units/{id}/interventions [P/T2]; GET /interventions/{id} [P/T2] … (+4 routes) | statement_expense_links, providers, interventions, expense_imputation_changes | E2E-P3, E2E-P5 | Minimal au démarrage ; complet pendant |  | 16, 20 |
| F16 | Documents par bien, lot, locataire, bail | P1 (V1) | T3 | S07, S14 | GET /units/{id}/leases (+2) [P/T1]; POST /documents [A/T3]; GET /documents [A/T3]; GET /documents/{id}/download [A/T3] … (+1 routes) | documents, document_links | E2E-P5 | Pendant |  | 20 |
| F17 | Agenda et alertes | P1 (V1) | T3 | S03, S15 | à spécifier : GET/POST /tasks (API §7.19) | `tasks` (prévue, non créée) | cas à écrire avec la règle (T3) | Pendant | route à spécifier, test à écrire, table à créer | 23 |
| F18 | Exports CSV / Excel / PDF pour le comptable | P1 | T3 | S11, S17 (pas d’écran propre) | POST /exports [A/T3]; GET /settings/export-formats [A/T3] | — (aucune) | PP-19 (neutralisation des formules) ; cas à écrire avec la règle (T3) | Pendant | test à écrire | 23 |
| F19 | Clôture de caisse journalière | P1 | T3 | S19 | GET /cash-days (+1) [A/T3] | cash_days | TESTING:F19 | Pendant |  | 22 |
| F20 | Dépôt de garantie | P1 | T3 | S10 | GET /leases/{id}/deposit-events (+1) [A/T3] | `security_deposit_events` (prévue, non créée) | cas à écrire avec la règle (T3) ; R-066, R-067 sans test | Pendant | test à écrire, table à créer | 21 |
| F21 | Pénalités / frais de retard configurables | P2 | — | — | — | — | — | Hors V1 |  | — |
| F22 | Notifications internes | P1 | T3 | S20 | à spécifier : GET /notifications (API §7.19) ; GET /home (partiel) | `notification_reads` (prévue, non créée) | cas à écrire avec la règle (T3) | Pendant | route à spécifier, test à écrire, table à créer | 23 |
| F23 | Fin de bail et changement de locataire | P1 | T3 | S10 | POST /leases/{id}/notice (+1) [A/T3] | leases | E-35 (renouvellement, R-091) ; R-090 sans test | Pendant |  | 21 |
| F24 | États des lieux | P2 | — | S24 | — | — | — | Hors V1 |  | — |
| F25 | Saisie assistée depuis un SMS mobile money collé | P1 | T3 | S04 | POST /payment-sms/parse [A/T3] | — (aucune) | cas à écrire avec la règle (T3) ; exemples réels de SMS à fournir | Pendant | test à écrire | 22 |
| F26 | Paiement en ligne par le locataire | P2 | — | — | — | — | — | Hors V1 |  | — |
| F27 | WhatsApp Business API | P2 | — | — | — | — | — | Hors V1 |  | — |
| F28 | Portail propriétaire mandant | P2 | — | S23 | — | — | — | Hors V1 |  | — |
| F29 | Espace locataire | P3 | — | — | — | — | — | Hors V1 |  | — |
| F30 | Révision / indexation du loyer | P2 | — | — | — | — | — | Hors V1 |  | — |
| F31 | Abonnement Immo Suite en ligne | P2 | — | — | — | subscriptions (prévue, non créée) | — | Hors V1 |  | — |
| F32 | Rapports avancés | P2 | — | S22 | — | — | — | Hors V1 |  | — |
| F33 | Application mobile hors-ligne pour agents terrain | P3 | — | — | — | — | — | Hors V1 |  | — |
| F34 | Signature électronique | P3 | — | — | — | — | — | Hors V1 |  | — |
| F35 | Copropriété | P3 | — | — | — | — | — | Hors V1 |  | — |
| F36 | Vitrine et annonces de biens vacants | P3 | — | — | — | — | — | Hors V1 |  | — |
| F37 | Assistant IA conversationnel | P3 | — | — | — | — | — | Hors V1 |  | — |
| F38 | (n'existe pas dans le dossier) | — | — | — | — | — | — | — | Trou de numérotation du dossier | — |
| F39 | Sauvegardes chiffrées, restauration testée, journalisation technique | P0 | T1 | — (opération) | commandes `immo …` (hors API) | — (opération) | PP-13 à PP-16 ; test mensuel de restauration (DEPLOYMENT §5.4) | Au démarrage |  | 00, 01, 04, 11, 13, 19 |
| F40 | Console HABEK | P0 | T2 | S17 (accès support) ; console hors catalogue | /console/… réservées (API §7.18) ; commandes `immo org create`, `immo org go-live` ; /support-grants [P/T2] | support_grants, import_batches, platform_staff, platform_events, org_features | ISO-8, PP-12 ; console en ligne de commande au pilote, interface à spécifier | Au démarrage | écrans de la console | 11, 12, 14 |

**Éléments techniques sans fonctionnalité source** : tables `product_events`, `error_reports`, `feedback_reports`, `org_features`, `org_settings`, `jobs`, `purge_order` (observation du pilote et interrupteurs : `ARCHITECTURE.md` §8.2 à §8.4, sans F-xx) ; routes `POST /events`, `POST /client-errors`, `POST /feedback`, `GET /search`, `GET /health` ; écrans S25 et S26 (rattachés à F10 et F13 par `UI_UX_DESIGN.md`).

## 2. Matrice R-001 à R-110

Générée par **citation explicite** de l'identifiant. Base : tables dont la fiche cite la règle, `§4` = garantie de `DATABASE_SCHEMA.md` §4, « Calcul » = règle portée par le crate `domain` ou le service (DP-51), sans mécanisme en base. API : code d'erreur ou route qui cite la règle ; « mention » = citée seulement en texte. Tests : cas E-xx ou règle citée dans `TESTING_QA.md` ; « famille » = couverture relevée à la main dans une famille de tests qui ne cite pas l'identifiant ; « à écrire » = pas encore de test (règle ouverte, T3 ou règle récente).

| Règle | Portée | Base | API | UI | Maquette | Tests cités | Écart | Étape |
|---|---|---|---|---|---|---|---|---|
| R-001 | Tous | payments +§4 | mention | UI | MAQ | TESTING(R) |  | 01 |
| R-002 | Tous | organizations | mention | UI | — | famille : TESTING §3 Horloge (parité Clock / app_now), G-04 |  | 01 |
| R-003 | Tous | organizations | mention | — | — | — | écran, test | 02 |
| R-004 | Tous | organizations +§4 | resource.not_found | UI | — | famille : A4, ISO-1 à ISO-9, PP-08 à PP-10 |  | 02 |
| R-005 | Tous | properties, units | route | — | — | E-47 | écran | 05 |
| R-006 | Tous | units +§4 | route, unit.has_active_lease | UI | — | — | test | 06 |
| R-007 | Tous | audit_log +§4 | route | UI | — | famille : G-05, ISO-4, A9 |  | 02 |
| R-008 | Tous | audit_log | — | — | — | famille : CODING §7 n° 9 (l'action échoue si l'audit échoue) | API, écran | 02 |
| R-009 | Tous | audit_log, platform_events, platform_staff … +§4 | route | UI | — | famille : ISO-8, PP-12 |  | 03, 11 |
| R-010 | Tous | properties | route | — | — | — | écran, test | 05 |
| R-011 | Tous | units | route, unit.reference_duplicate | — | — | — | écran, test | 05 |
| R-012 | Habitation, Commercial | units | route | — | — | — | écran, test | 05 |
| R-013 | Tous | Calcul | mention | — | MAQ | — | test | 06 |
| R-014 | Agence | mandate_units, mandates +§4 | mandate.overlap, route | — | MAQ | — | test | 16 |
| R-015 | Agence | mandates | mandate.rate_out_of_range, route | — | — | — | écran, test | 16 |
| R-016 | Tous | owners | route | — | — | — | écran, test | 05 |
| R-017 | Tous | tenants +§4 | route, tenant.phone_duplicate | UI | — | TESTING(R) |  | 05 |
| R-018 | Tous | tenants, users +§4 | route | — | — | — | écran, test | 05 |
| R-019 | Tous | tenants | route | UI | — | — | test | 05 |
| R-020 | Tous | leases +§4 | lease.overlap, payment.lease_not_in_progress, route | UI | MAQ | — | test | 06 |
| R-021 | Tous | — | route | — | MAQ | — | base, test | 06 |
| R-022 | Tous | — | route | UI | MAQ | famille : G-03, A10 | base | 06 |
| R-023 | Tous | lease_amendments, leases +§4 | lease.amendment_retroactive, lease.not_draft, route | — | MAQ | E-02, E-44 |  | 06 |
| R-024 | Habitation | Calcul | route | UI | MAQ | E-22 |  | 06 |
| R-025 | Commercial | lease_commercial_terms, leases | — | — | — | — | API, écran, test | 06 |
| R-026 | Tous | lease_commercial_terms | — | — | — | — | API, écran, test | 06 |
| R-027 | Tous | installments | route | — | MAQ | E-03, TESTING(R) |  | 06 |
| R-028 | Tous | Calcul | mention | — | — | E-01 | écran | 06 |
| R-029 | Tous | installments, jobs | mention | — | — | E-03, TESTING(R) | écran | 06 |
| R-030 | Tous | installments | — | — | — | — | API, écran, test | 06 |
| R-031 | Tous | leases | — | — | — | — | API, écran, test | 06 |
| R-032 | Tous | Calcul | — | UI | MAQ | TESTING(R) | API | 06 |
| R-033 | Tous | Calcul | mention | UI | MAQ | E-04, E-25, E-37, TESTING(R) |  | 06 |
| R-034 | Tous | installments | — | — | — | — | API, écran, test | 06 |
| R-035 | Tous | installment_lines | route | — | — | TESTING(R) | écran | 06, 12 |
| R-036 | Tous | installment_line_voids, installment_lines +§4 | route | — | — | — | écran, test | 06 |
| R-037 | Tous | payments +§4 | route | UI | — | famille : G-03, A10 |  | 07 |
| R-038 | Tous | org_settings | payment.confirmation_required, route | UI | MAQ | E-09, TESTING(R) |  | 07 |
| R-039 | Tous | payments +§4 | mention | UI | MAQ | — | test | 07 |
| R-040 | Tous | payments | mention | UI | — | — | test | 07 |
| R-041 | Tous | payments | payment.external_ref_duplicate | UI | MAQ | E-42, TESTING(R) |  | 07 |
| R-042 | Tous | idempotency_keys, payments +§4 | mention | UI | MAQ | E-07, TESTING(R) |  | 07 |
| R-043 | Tous | allocations | route | UI | — | TESTING(R) |  | 07 |
| R-044 | Tous | allocations +§4 | payment.allocation_invalid, route | — | MAQ | TESTING(R) |  | 07 |
| R-045 | Tous | Calcul | — | — | — | E-05, E-25 | API, écran | 07 |
| R-046 | Tous | allocations | payment.surplus_not_confirmed, route | UI | — | E-06, TESTING(R) |  | 07 |
| R-047 | Tous | payments +§4 | payment.immutable, route | UI | MAQ | E-08, TESTING(R) |  | 07 |
| R-048 | Tous | correction_requests, payments +§4 | payment.already_reversed, payment.reversal_reason_required, route | UI | MAQ | E-08, TESTING(R) |  | 09 |
| R-049 | Tous | jobs, receipt_files, receipts | receipt.pdf_not_ready, route | UI | MAQ | TESTING(R) |  | 08 |
| R-050 | Tous | receipt_counters, receipts +§4 | — | UI | MAQ | E-10, E-27, E-38, E-48 | API | 07 |
| R-051 | Tous | receipt_cancellations, receipts | receipt.immutable, route | UI | MAQ | E-38 |  | 09 |
| R-052 | Tous | receipts | — | UI | — | famille : TESTING §6 (textes attendus du PDF) | API | 08 |
| R-053 | Tous | receipts | mention | UI | — | E-11, TESTING(R) |  | 07 |
| R-054 | Tous | org_settings, share_links | route, share_link.expired | UI | — | E-12 |  | 08 |
| R-055 | Tous | receipt_sends | route | UI | — | — | test | 08 |
| R-056 | Tous | receipt_files | route | — | — | famille : TESTING §6 (une page en A4 et en A5) | écran | 08 |
| R-057 | Tous | Calcul | route | — | — | TESTING(R) | écran | 10 |
| R-058 | Tous | Calcul | mention | — | MAQ | E-13, E-13b, TESTING(R) |  | 10 |
| R-059 | Tous | promises | route | UI | MAQ | TESTING(R) |  | 10 |
| R-060 | Tous | message_templates | route | UI | — | E-14 |  | 15 |
| R-061 | Tous | reminders | mention | — | — | famille : E2E parcours 2 (WhatsApp assisté) | écran | 15 |
| R-062 | Tous | reminder_installments, reminders | route | — | — | famille : A9 (reminder.immutable) | écran | 15 |
| R-063 | Tous | reminders | mention | UI | MAQ | TESTING(R) |  | 15 |
| R-064 | Tous | message_templates | route, template.unknown_variable, template.variable_missing | UI | MAQ | E-14, TESTING(R) |  | 15 |
| R-065 | Tous | promises | route | — | — | E-15, E-41 | écran | 15 |
| R-066 | Tous | — | route | — | — | TESTING(R) | base, écran | 21 |
| R-067 | Tous | — | route | — | — | TESTING(R) | base, écran | 21 |
| R-068 | Agence | mandates, statement_versions +§4 | route | — | MAQ | E-16, E-26, E-28, E-32 |  | 17 |
| R-069 | Agence | mandate_rate_changes +§4 | mandate.rate_change_retroactive, route | — | — | E-17 | écran | 16 |
| R-070 | Agence | statement_versions +§4 | mention | — | MAQ | E-18, TESTING(R) |  | 17 |
| R-071 | Agence | Calcul | mention | — | MAQ | — | test | 17 |
| R-072 | Agence | statement_payment_links, statements +§4 | statement.preview_stale | — | — | TESTING(R) | écran | 17 |
| R-073 | Agence | statement_payment_links, statement_versions | route, statement.already_validated, statement.preview_stale | — | — | E-19 | écran | 17 |
| R-074 | Agence | statement_expense_links, statement_versions +§4 | expense.frozen, route | — | — | TESTING(R) | écran | 17 |
| R-075 | Agence | Calcul | route, statement.units_without_mandate | UI | MAQ | TESTING(R) |  | 17 |
| R-076 | Agence | payouts, statement_versions +§4 | route | — | — | — | écran, test | 17 |
| R-077 | Agence | payouts +§4 | payout.amount_mismatch, route | — | — | — | écran, test | 17 |
| R-078 | Agence | interventions | route | — | — | — | écran, test | 16 |
| R-079 | Tous | cash_days | route | — | — | — | écran, test | 22 |
| R-080 | Tous | import_batches, import_rows, owners | import.not_cancellable, import.wrong_state, route | UI | MAQ | E-20, E-21 |  | 12, 18 |
| R-081 | Tous | memberships, role_settings, unit_assignments +§4 | auth.forbidden, route, user.last_manager | UI | — | — | test | 03 |
| R-082 | Tous | mandates, organizations +§4 | feature.disabled | — | — | — | écran, test | 03 |
| R-083 | Agence | payouts, statement_reopenings, statement_versions +§4 | route, statement.reopen_forbidden | UI | — | E-30 |  | 17 |
| R-084 | Agence | expense_imputation_changes, interventions, statement_expense_links | route | — | MAQ | E-29 |  | 16 |
| R-085 | Agence | Calcul | mention | — | — | — | écran, test | 17 |
| R-086 | Tous | allocations | mention | — | — | E-31, TESTING(R) | écran | 07 |
| R-087 | Tous | allocations, installment_lines | route | UI | MAQ | E-32, TESTING(R) |  | 07 |
| R-088 | Tous | lease_tenants, payments | route | UI | — | E-33 |  | 07 |
| R-089 | Tous | cash_days, payments | route | UI | — | E-34 |  | 22 |
| R-090 | Tous | leases | route | — | — | TESTING(R) | écran | 06, 21 |
| R-091 | Tous | lease_amendments, leases +§4 | route | — | — | E-35 | écran | 06 |
| R-092 | Commercial | leases | route | UI | — | famille : TESTING §6 (cas « TVA incluse ») |  | 06 |
| R-093 | Tous | leases, units +§4 | route, unit.under_works, unit.works_requires_vacant | UI | MAQ | E-36 |  | 06 |
| R-094 | Habitation, Commercial | units | — | — | — | — | API, écran, test | 05 |
| R-095 | Tous | Calcul | — | UI | MAQ | E-37, TESTING(R) | API | 06 |
| R-096 | Tous | payments, receipt_cancellations, receipts +§4 | route | UI | MAQ | E-38 |  | 09 |
| R-097 | Tous | reminders | route | UI | — | — | test | 15 |
| R-098 | Tous | Calcul | mention | — | — | E-39, TESTING(R) ; E-39 | écran | 06 |
| R-099 | Tous | Calcul | mention | — | — | E-40, TESTING(R) ; E-40 | écran | 06 |
| R-100 | Tous | Calcul | mention | — | — | à écrire (cas d'or de T1) | écran, test | 06 |
| R-101 | Tous | Calcul | mention | — | — | E-41, TESTING(R) ; E-41 | écran | 15 |
| R-102 | Tous | Calcul | mention | — | — | E-42, TESTING(R) ; E-42 | écran | 07 |
| R-103 | Tous | Calcul | — | UI | — | E-43, TESTING(R) ; E-43 | API | 10 |
| R-104 | Tous | Calcul | — | — | — | E-44, TESTING(R) ; E-44 | API, écran | 06 |
| R-105 | Agence | Calcul | mention | — | — | E-45, TESTING(R) ; E-45 | écran | 17 |
| R-106 | Agence | Calcul | — | — | — | à écrire (cas d'or de T2) | API, écran, test | 17 |
| R-107 | Agence | Calcul | — | — | — | à écrire (cas d'or de T2) | API, écran, test | 17 |
| R-108 | Agence | Calcul | mention | — | — | E-46, TESTING(R) ; E-46 | écran | 17 |
| R-109 | Agence | Calcul | — | — | — | à écrire (cas d'or de T2) | API, écran, test | 17 |
| R-110 | Tous | organizations (cycle de vie) | mention | — | — | E-47, E-48, TESTING(R) ; E-47, E-48 | écran | 11 |

**Règles techniques sans règle métier source** : les gardes de `DATABASE_SCHEMA.md` §4 citent toutes une règle R-xxx ou une décision DP-xx, sauf le contrôle `payer_third_party_phone` (R-088 partielle). La borne de 1 à 999 999 999 est portée par R-001 (DP-56).

## 3. Écarts restants

- Fonctionnalités avec au moins un manque : F17 (route à spécifier, test à écrire, table à créer), F18 (test à écrire), F20 (test à écrire, table à créer), F22 (route à spécifier, test à écrire, table à créer), F25 (test à écrire), F40 (écrans de la console).
- Règles sans mécanisme en base (4) : R-021, R-022, R-066, R-067.
- Règles sans citation dans l’API (17) : R-008, R-025, R-026, R-030, R-031, R-032, R-034, R-045, R-050, R-052, R-094, R-095, R-103, R-104, R-106, R-107, R-109.
- Règles absentes de l’interface et de la maquette (52) : R-003, R-005, R-008, R-010, R-011, R-012, R-015, R-016, R-018, R-025, R-026, R-028, R-029, R-030, R-031, R-034, R-035, R-036, R-045, R-056, R-057, R-061, R-062, R-065, R-066, R-067, R-069, R-072, R-073, R-074, R-076, R-077, R-078, R-079, R-082, R-085, R-086, R-090, R-091, R-094, R-098, R-099, R-100, R-101, R-102, R-104, R-105, R-106, R-107, R-108, R-109, R-110.
- Règles sans test cité ni famille (36) : R-003, R-006, R-010, R-011, R-012, R-013, R-014, R-015, R-016, R-018, R-019, R-020, R-021, R-025, R-026, R-030, R-031, R-034, R-036, R-039, R-040, R-055, R-071, R-076, R-077, R-078, R-079, R-081, R-082, R-085, R-094, R-097, R-100, R-106, R-107, R-109.

Ces écarts sont suivis dans `AUDIT_COHERENCE_2.md` (C2-031, C2-032, section « Constats restés ouverts »). Une règle « couverte par famille » reste à relier à un test nommé dès que le test existe.

## 4. Régénérer

Les deux matrices viennent de scripts d'extraction (expressions régulières sur les tableaux des documents propriétaires). Après une modification de `CONTEXTE.md`, `UI_UX_DESIGN.md`, `API_CONTRACTS.md`, `DATABASE_SCHEMA.md`, `DOMAINE.md` ou `TESTING_QA.md`, relancer l'extraction et remplacer ce fichier. Les colonnes « Test(s) » des fonctionnalités et les familles de tests de la section 2 sont des annotations manuelles, à revoir à ce moment.
