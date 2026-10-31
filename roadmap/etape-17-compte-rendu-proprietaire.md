# Étape 17 — Compte rendu propriétaire, commission et reversement

| Champ | Valeur |
|---|---|
| Statut | À faire |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T2 (F13) |
| Effort | L, environ 18 JD |
| Dépend de | Étapes 08, 09 et 16 |
| Jalon | Lot 2 : condition recommandée de la bascule `live` (J5) |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Produire en deux clics le compte rendu mensuel de chaque mandant : loyers encaissés, commission, dépenses imputées, net à reverser, impayés ; le valider (figé), l'exporter en PDF identique, le partager par lien et enregistrer le reversement.
Valeur : c'est le livrable que l'agence doit produire chaque mois ; sans lui elle garde son Excel (définition de « terminé » du MVP).

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections |
|---|---|---|
| Fonctionnalité | F13 | dossier §16 F13 |
| Règles | R-068, R-070 à R-077, R-083, R-085, R-105 à R-109 | `DOMAINE.md` « Commission et reddition » |
| Cas chiffrés | E-16, E-18, E-19, E-26, E-28, E-30, E-45, E-46 | `DOMAINE.md` |
| Décisions | DP-14 à DP-19, DP-45, DP-59 ; n° 10, 29, 44 | `DECISIONS.md` |
| Architecture | `ARCHITECTURE.md` | §2.3 `statements`, §4.2 (formule de commission entière) |
| Schéma | `DATABASE_SCHEMA.md` | §3 « Comptes rendus et reversements », §4.6 |
| Contrat | `API_CONTRACTS.md` | §6.3, §7.8 |
| Écrans | `UI_UX_DESIGN.md` | S12, S26 ; `MAQUETTE.md` « Compte rendu propriétaire (M1, octobre 2026) » |

## 3. Prérequis

Étapes 08, 09, 16. Règle de reddition R-071 (date d'encaissement) confirmée à la découverte (étape 14). Précision du taux (n° 29). Séance de recette R7 et R8 avec l'agence.

## 4. Fiche de spécification à produire ou valider

`specs/F13-compte-rendu.md` — commission = `(base × taux_bp + 5 000) / 10 000` en entiers, un seul arrondi par relevé et par mandant (R-068, E-16, E-26) ; base « loyer » ou « loyer + charges » (E-28) ; charges reversées sous base loyer (R-105, E-45) ; arriérés et avances comptés à la date d'encaissement ; loyer dans un seul relevé (R-072) ; relevé validé figé, paiement tardif ⇒ période suivante (R-073, E-19) ; base négative ⇒ production refusée, net négatif ⇒ aucun reversement (R-106) ; dépenses imputables datées dans la période et « terminées » (R-107) ; lot encaissé sans mandat valide bloque (R-075) ; un seul reversement du montant exact du net (R-108, E-46) ; réouverture avec motif avant reversement (R-083, E-30) ; mandat clôturé en cours de mois (R-109) ; dépôt de garantie hors relevé (R-085) ; export PDF identique à la dernière version (R-074) ; versions conservées. Cas chiffrés : E-18 (400 000, 8 %, 30 000 ⇒ 32 000 et 338 000). **S'arrêter pour validation de Mimis.**

## 5. Tâches détaillées

### 5.1 Base de données
`statements`, `statement_versions` (instantané complet, contraintes arithmétiques sur commission et net), `statement_payment_links` (un paiement, un relevé), `statement_expense_links`, `statement_reopenings`, `payouts` (unique par relevé, montant exact) ; gardes G de validation réservée au gérant ; ajout seul.

### 5.2 Noyau métier
`crates/domain` : `commission(base, rate_bp)`, `net(collected, commission, expenses)` (tests d'or d'abord, P11, P12) ; `statements::preview/validate/reopen/record_payout/render_pdf` ; clé d'idempotence sur validation et reversement ; audit `statement.validated`, `statement.reopened`, `payout.recorded`.

### 5.3 API
`GET /statements`, `GET /statements/preview`, `POST /statements`, `POST /statements/{id}/versions`, `POST …/reopen`, `POST …/payout`, `POST …/shares`, `GET /statements/{id}`, `GET …/versions/{v}/pdf` ; erreurs `statement.units_without_mandate`, `statement.preview_stale`, `statement.already_validated`, `statement.reopen_forbidden`, `payout.amount_mismatch`.

### 5.4 Interface
S12 : liste, production (mandant + période), aperçu, ajout d'une dépense, validation avec modale et confirmation du mot de passe, PDF, lien de partage, reversement, réouverture avant reversement ; S26 pour le compte rendu partagé ; état « à produire » calculé.

### 5.5 Documents PDF
Compte rendu A4 identique à la version validée (reproductible, empreinte SHA-256).

### 5.6 Audit et événements
Voir 5.2 ; événements `statement_previewed`, `statement_validated`, `statement_reopened`, `payout_recorded`.

### 5.7 Tests
E-16, E-18, E-19, E-26, E-28, E-30, E-45, E-46 ; cas d'or à écrire pour R-106, R-107, R-109 (à créer avec la fiche) ; P11, P12 ; piège `sum(bigint)` (commission 8 000, pas 8 001) ; E2E parcours 3 (R7, R8) ; PDF identique à la version validée ; lot sans mandat bloque.

## 6. Critères d'acceptation vérifiables

1. 400 000 encaissés, 8 %, 30 000 de dépenses : commission 32 000, net 338 000 (E-18) ; 333 333 à 7,5 % : commission 25 000 (E-26).
2. Reversement de 338 000 accepté, de 337 999 refusé (E-46).
3. Un paiement saisi après validation, daté de la période, apparaît dans le relevé suivant (E-19).
4. Réouverture acceptée avant reversement, refusée après (E-30).
5. L'export PDF est identique à la dernière version validée (R-074).
6. Les contrôles de la maquette liés aux mandats, relevés et dépenses sont verts sur l'organisation de démonstration complétée (PP-07).
7. Le compte rendu du premier mois de l'agence est comparé à son calcul manuel (écart nul ou expliqué).

## 7. Sécurité et déploiement à vérifier

Validation et reversement réservés au gérant (base et API) ; confirmation du mot de passe ; liens de partage expirables ; migration de gardes relue par un second avis.

## 8. Liste de démonstration (Mimis)

Produire le compte rendu d'octobre du mandant M1, lire 400 000 → 32 000 → 338 000, ajouter une dépense, valider, ouvrir le PDF, enregistrer le reversement, essayer de rouvrir : refus.

## 9. Définition de terminé, risques, hors étape

Terminé : critères 1 à 7 verts ; `DOMAINE.md` : règles R-105 à R-109 confirmées ou ajustées ; `PP-05` et `PP-07` complétés.

| Risque | Mesure |
|---|---|
| Règles de reddition propres à l'agence | R-071 à R-109 confirmées à la découverte ; règle d'or 12 |
| Écarts d'arrondi contestés | Un seul arrondi, formule entière documentée dans le PDF |

Hors étape : portail mandant (F28), comptabilité générale.

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 17
- Dépend de : 08, 09, 16
- Tranche : T2
- Jalon : Lot 2
- F : F13
- R : R-068, R-070, R-071, R-072, R-073, R-074, R-075, R-076, R-077, R-083, R-085, R-105, R-106, R-107, R-108, R-109
- E : E-16, E-18, E-19, E-26, E-28, E-30, E-45, E-46
- PP : PP-07, PP-05
- Contenus : aucun
