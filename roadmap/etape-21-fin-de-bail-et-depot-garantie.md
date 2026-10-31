# Étape 21 — Fin de bail, préavis et dépôt de garantie

| Champ | Valeur |
|---|---|
| Statut | À faire (T3, derrière un interrupteur) |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T3 (F20, F23) |
| Effort | L, environ 14 JD |
| Dépend de | Étape 06 ; critères d'entrée de l'étape 19 ; avis juridique (plafond, préavis) |
| Jalon | J6 |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Suivre la réception, les retenues motivées et la restitution du dépôt de garantie, et gérer le préavis et le renouvellement.
Valeur : moins de litiges de caution et de vacance.

## 2. Périmètre, règles et documents à lire

F20, F23 ; R-066, R-067, R-090 (préavis), R-091 ; E-35 (renouvellement) ; DP-19, DP-24 ; `API_CONTRACTS.md` §7.5 (`/notice`, `/deposit-events`) ; `DATABASE_SCHEMA.md` §3 (table `security_deposit_events` prévue, non créée) ; `UI_UX_DESIGN.md` S10.

## 3. Prérequis

Étapes 06 et 19. Questions ouvertes de `DOMAINE.md` : qui détient la caution, délai de restitution, plafond commercial, durée légale du préavis [À VÉRIFIER AVEC UN JURISTE] (C-04).

## 4. Fiche de spécification à produire ou valider

`specs/F20-F23-depot-et-fin-de-bail.md` — événements du dépôt en ajout seul (réception, retenue avec motif obligatoire, restitution) ; préavis : statut `notice`, le bail reste en cours, échéances jusqu'au départ, annulation tracée ; renouvellement par avenant ; changement de locataire = nouveau bail ; dépôt hors compte rendu (R-085). Cas chiffrés à écrire avec la règle (R-066, R-067 sans test aujourd'hui). **S'arrêter pour validation de Mimis.**

## 5. Tâches détaillées

- 5.1 Base : création de `security_deposit_events` (ajout seul, RLS, `purge_order`).
- 5.2 Noyau : `lease::notice`, `cancel_notice`, `deposit_event` ; audit `lease.notice_given`, `lease.notice_cancelled`.
- 5.3 API : `POST /leases/{id}/notice`, `POST …/notice/cancel`, `GET/POST /leases/{id}/deposit-events`.
- 5.4 Interface : S10 (préavis, dépôt avec retenues, restitution), alertes de fin de bail (liste, sans agenda).
- 5.5 PDF : aucun. 5.6 Événements : `feature_used`.
- 5.7 Tests : cas d'or à écrire d'abord (R-066, R-067, R-090) ; E-35 ; retenue sans motif refusée ; échéances continuent en préavis (DP-39).

## 6. Critères d'acceptation vérifiables

1. Une retenue sans motif est refusée ; chaque retenue est tracée.
2. En préavis, les échéances continuent jusqu'au départ, puis le bail passe à terminé et le lot à vacant.
3. Le dépôt n'entre ni dans les loyers encaissés ni dans la base de commission.

## 7. Sécurité et déploiement à vérifier

Table en ajout seul à deux barrières ; second avis sur la migration ; interrupteur.

## 8. Liste de démonstration (Mimis)

Saisir un préavis, vérifier les échéances, enregistrer une retenue motivée et la restitution.

## 9. Définition de terminé, risques, hors étape

Terminé : critères verts ; `DOMAINE.md` complété par les réponses juridiques.

| Risque | Mesure |
|---|---|
| Règles légales divergentes | Avis juridique ; avertissements non bloquants |

Hors étape : états des lieux, indexation automatique.

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 21
- Dépend de : 06
- Tranche : T3
- Jalon : J6
- F : F20, F23
- R : R-066, R-067, R-090
- E : aucun
- PP : aucun
- Contenus : C-04, C-20
