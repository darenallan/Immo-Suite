# Étape 23 — Agenda, notifications internes et exports pour le comptable

| Champ | Valeur |
|---|---|
| Statut | À faire (T3, trois fonctions, une à la fois) |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T3 (F17, F22, F18) |
| Effort | L, environ 13 JD |
| Dépend de | Étapes 10 et 15 ; critères d'entrée de l'étape 19 |
| Jalon | J6 |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Ne plus rater une échéance non financière (agenda), savoir quoi faire à l'ouverture (cloche, résumé), et fournir au comptable des exports propres.
Valeur : confort et intégration avec l'existant, sans règle d'argent nouvelle.

## 2. Périmètre, règles et documents à lire

F17, F22, F18 ; `API_CONTRACTS.md` §7.12 (`/exports`), §7.19 (T3 à spécifier) ; `SECURITY.md` T6 (neutralisation des formules), `TESTING_QA.md` PP-19 ; `DATABASE_SCHEMA.md` §3 (`tasks`, `notification_reads` prévues, non créées) ; `UI_UX_DESIGN.md` S15, S20, S03.

## 3. Prérequis

Étapes 10, 15, 19. Les routes de F17 et F22 ne sont pas spécifiées : la fiche les définit, avec leurs tables et leurs tests, avant tout code (`TRACABILITE.md` §3).

## 4. Fiche de spécification à produire ou valider

1. `specs/F17-agenda.md` — `GET/POST /tasks`, `PATCH /tasks/{id}` ; alertes calculées (promesse non tenue, rappel J−3, fin de bail à 60 jours) et tâches saisies ; table `tasks`.
2. `specs/F22-notifications.md` — liste calculée, état « lu » (`notification_reads`), résumé quotidien du gérant sans e-mail au pilote (DP-68).
3. `specs/F18-exports.md` — CSV, Excel, PDF des paiements, impayés, comptes rendus, audit ; asynchrone (`202`) ; droit `export.run` avec confirmation du mot de passe ; cellules commençant par `=`, `+`, `-`, `@` neutralisées ; traçabilité `export.created`. **S'arrêter pour validation de Mimis.**

## 5. Tâches détaillées

- 5.1 Base : création de `tasks`, `notification_reads` (RLS, clés composites).
- 5.2 Noyau : agrégation des alertes calculées, export en tâche de fond, neutralisation des formules.
- 5.3 API : routes des trois fiches ; `GET /settings/export-formats`.
- 5.4 Interface : S15, S20 (cloche), exports depuis S11, S05, S12, S21.
- 5.5 PDF : exports PDF des listes. 5.6 Audit : `export.created`.
- 5.7 Tests : cas à écrire avec la règle ; PP-19 (cellule `=HYPERLINK(…)` neutralisée) ; droits ; isolation ; E2E.

## 6. Critères d'acceptation vérifiables

1. Une promesse non tenue et une fin de bail à 60 jours apparaissent dans l'agenda et la cloche.
2. Un export contenant un nom commençant par `=` ouvre dans le tableur comme du texte.
3. Un utilisateur sans droit d'export reçoit 403 ; un export confirme le mot de passe et écrit une ligne d'audit.

## 7. Sécurité et déploiement à vérifier

Exports jamais envoyés par e-mail au pilote ; fichiers temporaires purgés ; interrupteur par fonction.

## 8. Liste de démonstration (Mimis)

Ouvrir l'agenda, lire une alerte, exporter les paiements du mois et ouvrir le fichier dans un tableur.

## 9. Définition de terminé, risques, hors étape

Terminé : critères verts ; `API_CONTRACTS.md` §7.19 mis à jour.

| Risque | Mesure |
|---|---|
| Fuite par export | Droit réglable, confirmation, audit, neutralisation |

Hors étape : rapports avancés (F32), exports comptables particuliers (service payant).

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 23
- Dépend de : 10, 15
- Tranche : T3
- Jalon : J6
- F : F17, F22, F18
- R : aucune
- E : aucun
- PP : PP-19
- Contenus : aucun
