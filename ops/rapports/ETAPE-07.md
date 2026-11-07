# Étape 07 — Rapport de tests (05/11/2026)

**Étape validée par Mimis le 06/11/2026.** Étape : « Encaisser un loyer » (`roadmap/etape-07-encaisser.md`). Fiches `specs/F09-paiements.md` et `specs/F10-numerotation-et-instantane.md` **validées par Mimis le 04/11/2026** (DP-96, décision n° 37 prise) ; DP-97 (écarts d'implémentation, **à relire**).

## Résultats réels

| Famille | Résultat | Commande |
|---|---|---|
| Domaine (nouveaux) | **18 réussis** : E-05, E-06, E-11, E-31, E-32, ordre R-087, affectation ajustée, P1, P2, P4, P7, P14 (analyseur indépendant), numéro DP-37/DP-61 | `cargo test -p immo-domain --test payments` |
| Base et API (étape 07) | **12 réussis** (`tests/db/tests/step07_payments.rs`) | `cargo test -p immo-db-tests --test step07_payments` (port 5435) |
| Rust complet | 349 réussis, 1 échec corrigé ensuite (règle de la liste blanche : un événement a toujours une propriété), puis `immo-app` 93 sur 93 ; clippy `-D warnings` vert | `cargo test --workspace --locked --jobs 2` |
| Front | **70 réussis** (4 pour S04, S11, S25 : `web/src/Step07.test.tsx`), typage et ESLint verts | `npm run typecheck`, `lint`, `test` |
| Pile réelle | **18 parcours réussis** (2 sautés : inactivité courte), dont `web/e2e-real/etape-07.spec.ts` à 390 et 1440 px ; deux passages précédents ont échoué pour le poste (port de l'API non vu en 60 s, mise en veille) et une fois pour le parcours (montant proposé après un partiel) | `powershell -File ops/dev/real-stack.ps1 -Port 5435` |

## Critères d'acceptation (§6)

| # | Critère | État |
|---|---|---|
| 1 | 50 000 sur 75 000 : partielle, reste 25 000, quittance « solde restant 25 000 » ; à J+1 en retard (E-05, E-25) | vérifié (API) |
| 2 | 100 000 sur 75 000 : 25 000 d'avance sur l'échéance suivante ; sans échéance suivante, crédit affecté à la génération (E-06, E-31) | vérifié (API) |
| 3 | Double soumission : un paiement, une quittance (E-07) ; même clé 3 fois en parallèle : un paiement | vérifié |
| 4 | Numéros consécutifs ; 000001 en 2027 (E-10, E-27) ; concurrence sans trou ni doublon | vérifié (4 × 10 sur le poste, pas 10 000 : DP-97) |
| 5 | 75 000 en lettres (E-11) | vérifié (domaine, quittance) |
| 6 | Seuil (E-09) ; référence en double bloquante puis confirmée (E-42) | vérifié |
| 7 | `PUT`, `PATCH`, `DELETE` → 405 ; `UPDATE`, `DELETE`, `TRUNCATE` refusés en base | vérifié pour `immo_app` (les deux autres rôles : non testés ici) |
| 8 | Loyer avant charges (E-32) ; tiers mentionné (E-33) | vérifié |
| 9 | Saisie en moins de 30 s sur le téléphone de test | **non mesuré** (C-13 absent, DP-96 : mesure préliminaire à faire) |

## Démonstration (§8)

`web/e2e-real/etape-07.spec.ts` : bail dû aujourd'hui, 50 000 en espèces (numéro d'essai, « solde restant 25 000 », montant en lettres), 100 000 sur un autre bail (confirmation de l'avance, « avance 25 000 »), coupure du réseau puis nouvel essai (un seul paiement), même référence mobile money deux fois (avertissement bloquant, confirmation), journal S11 sans action de modification.

## Reste à faire avant la validation

- Onglet Paiements de S09 ; aperçu d'affectation modifiable dans S04 ; recherche unique dans S04.
- Mesure du budget de 300 ms et des 30 secondes ; concurrence 20 × 500 et propriétés à 10 000 cas (CI).
- Second avis C-19 sur la migration de l'étape.
