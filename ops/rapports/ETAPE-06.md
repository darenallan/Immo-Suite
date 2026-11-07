# Étape 06 — Rapport de tests (04/11/2026)

Étape : « Baux et échéancier automatique » (`roadmap/etape-06-baux-et-echeancier.md`). Fiches `specs/F07-baux.md` et `specs/F08-echeancier.md` **validées par Mimis le 04/11/2026** (DP-94) ; DP-95 (écarts d'implémentation, **à relire**).

**Étape validée par Mimis le 04/11/2026.** **Verdict : les 10 critères sont vérifiés par l'API sur le poste, et la démonstration de l'étape (§8) passe sur la pile réelle à 390 et 1440 px.** Reste le second avis C-19.

## Résultats réels

| Famille | Résultat | Commande |
|---|---|---|
| Rust | **320 réussis** (295 à l'étape 05) ; `pdf_process` (délai du rendu) a échoué deux fois juste après une nouvelle compilation de `immo.exe` (5,04 s mesurées, lancement du processus compris), puis réussi seul en 0,37 s : instabilité connue, à rendre robuste | `cargo test --workspace --locked --jobs 2` (port 5435) |
| Rust : clippy `-D warnings`, `cargo fmt` | verts | |
| Front : typage, ESLint, tests | verts, **66 réussis** (6 pour S10 : `web/src/Step06.test.tsx`) | `npm run typecheck`, `lint`, `test` |
| Pile réelle | **16 parcours réussis** (2 sautés : inactivité courte), dont `web/e2e-real/etape-06.spec.ts` à 390 et 1440 px | `powershell -File ops/dev/real-stack.ps1 -Port 5435` |

Tests ajoutés : `crates/domain/tests/schedule.rs` (13 : E-01, E-02, E-22, E-35, E-37, E-39, E-40, E-44, R-030, R-100, propriétés P9 et P15) ; `tests/db/tests/step06_leases.rs` (12 : critères 1 à 10, E-03 avec la tâche réelle rejouée, arriérés et annulation motivée, activation rejouée, alerte de 02 h 00). Matrice A3 : 58 routes à droit, 406 couples, 0 écart.

## Critères d'acceptation (§6)

| # | Critère | État |
|---|---|---|
| 1 | Bail de 75 000 au 5, activé le 1er mars : échéance du 05/03 à 75 000 (E-01) | vérifié (API) |
| 2 | Avenant à 80 000 en avril : mars 75 000, avril 80 000 (E-02) | vérifié ; la garde « échéance déjà payée » viendra avec l'étape 07 (DP-94) |
| 3 | Tâche rejouée : aucun doublon (E-03) | vérifié (2 puis 0 échéance créée) |
| 4 | Le 4 à venir, le 5 due, le 6 en retard (E-37, E-04) | vérifié ; 7 jours de retard le 12 |
| 5 | Jour 31 → 28/02/2027 (E-39) ; mars entier (E-40) ; avenant saisi le 15/12 (E-44) | vérifiés ; la ligne de janvier est annulée « avenant du 15/12/2026 » et remplacée |
| 6 | Chevauchement refusé avec les dates, y compris avec un bail terminé | vérifié (bornes incluses) |
| 7 | Dépôt > 2 mois : avertissement ; lot en travaux refusé puis accepté (E-22, E-36) | vérifié |
| 8 | Prolongation par avenant (E-35) | vérifié, dépôt inchangé |
| 9 | Archiver un lot (ou un locataire) à bail en cours : refusé | vérifié |
| 10 | Aucune colonne de statut d'échéance | vérifié (schéma) ; écriture seule vérifiée |

## Démonstration (§8) sur la pile réelle

`web/e2e-real/etape-06.spec.ts`, horloge réelle, à 390 et 1440 px :

1. Bien « Gounghin » avec les lots A1 et B1, locataire « Awa Sawadogo ».
2. Assistant en trois étapes : bail de 75 000 au 5 commençant le 1er du mois, dépôt de 200 000, [Activer] ; la fiche affiche « Actif », l'avertissement du dépôt et l'échéance du 5 avec sa pastille.
3. Le mois précédent se saisit en arriéré daté (F08 règle 6, R-100) : la ligne apparaît « En retard · N jours ».
4. Avenant à 80 000 à partir du mois suivant : il figure dans la liste des avenants, et le loyer en vigueur reste 75 000.
5. Second bail sur A1 aux mêmes dates : le brouillon s'enregistre, l'activation est refusée avec « Ce lot a déjà un bail du JJ/MM/AAAA au sans date de fin… Le brouillon est enregistré. »
6. B1 « Mis en travaux » depuis sa fiche : l'activation d'un bail sur B1 est refusée, et le brouillon s'ouvre ensuite.

Deux écarts entre ce parcours et sa première version sont venus du parcours, pas du serveur : un bail commencé avant le mois d'activation démarre au mois d'activation (R-100), et le chevauchement se vérifie à l'activation, pas au brouillon (F07). Une première exécution a échoué entièrement pour une raison du poste (WMI indisponible un instant : le contrôle « port prêt » de `task.ps1` n'a pas vu le front).

## Reste à faire avant la validation

- Second avis C-19 sur l'exclusion et les gardes ; mesure de la vue des soldes sur 500 lots.
- Petits manques notés dans DP-95 : champs commerciaux détaillés dans l'assistant, modale de résiliation, recherche de locataire dans l'assistant.
