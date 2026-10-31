# Étape 22 — Clôture de caisse et saisie assistée depuis un SMS mobile money

| Champ | Valeur |
|---|---|
| Statut | À faire (T3, deux fonctions, deux interrupteurs, l'une après l'autre) |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T3 (F19, F25) |
| Effort | L, environ 14 JD |
| Dépend de | Étape 07 ; critères d'entrée de l'étape 19 ; contenu C-16 |
| Jalon | J6 |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Comparer les espèces attendues et comptées en fin de journée (F19) ; pré-remplir un paiement mobile money depuis le SMS collé, sans écriture automatique (F25).
Valeur : écarts de caisse visibles ; moins d'erreurs de montant et de référence.

## 2. Périmètre, règles et documents à lire

F19, F25 ; R-079, R-089 ; E-34 ; DP-23 ; `DATABASE_SCHEMA.md` §3 `cash_days`, §11.1 n° 5 ; `API_CONTRACTS.md` §7.6 (`/cash-days`, `/payment-sms/parse`) ; `UI_UX_DESIGN.md` S19, S04 (zone « Coller le SMS »).

## 3. Prérequis

Étapes 07 et 19. **C-16** : exemples réels de SMS Orange Money et Moov Money (obligatoires avant F25). Question ouverte de la journée de caisse (contre-passation d'un paiement en espèces : journée d'origine ou de correction) à trancher dans la fiche.

## 4. Fiche de spécification à produire ou valider

1. `specs/F19-cloture-de-caisse.md` — journée = paiements en espèces reçus par le caissier ; espèces attendues calculées, comptées saisies, écart et motif ; verrouillage ; saisie après clôture : le paiement entre dans la journée ouverte de la saisie, date réelle conservée, avertissement (R-089, E-34) ; version simple du plan Propriétaire.
2. `specs/F25-lecture-sms.md` — lecture par règles, suggestions seulement (montant, référence, mode, date), l'utilisateur confirme, aucune IA ; formats couverts par C-16. **S'arrêter pour validation de Mimis.**

## 5. Tâches détaillées

- 5.1 Base : activation des garde-fous de `cash_days` (déclencheur de `payments`), `cash.close`.
- 5.2 Noyau : `ledger::close_cash_day` (idempotent), analyseur de SMS dans `crates/domain` (cas d'or tirés des SMS réels).
- 5.3 API : `GET /cash-days`, `POST /cash-days/{date}/close` (`Idempotency-Key`), `POST /payment-sms/parse` (`payment_sms.unrecognized`).
- 5.4 Interface : S19, zone repliable dans S04, avertissement de journée clôturée.
- 5.5 PDF : aucun. 5.6 Audit : clôture ; événements `feature_used`.
- 5.7 Tests : E-34, propriété sur l'écart, SMS réels (fichier de cas), fichiers sans montant, double clôture.

## 6. Critères d'acceptation vérifiables

1. Espèces reçues le 10, saisies le 11, journée du 10 clôturée : paiement daté du 10, compté dans la journée du 11, avertissement affiché (E-34).
2. La clôture compare attendu et compté et verrouille la journée ; une seconde clôture est refusée.
3. Un SMS collé ne crée aucun paiement tant que l'utilisateur ne valide pas ; un SMS non reconnu répond sans erreur bloquante.

## 7. Sécurité et déploiement à vérifier

Fonctions qui touchent l'argent allumées après leurs cas d'or ; le texte du SMS n'est jamais journalisé.

## 8. Liste de démonstration (Mimis)

Encaisser en espèces, clôturer la journée, saisir un paiement du lendemain, coller un SMS réel et valider.

## 9. Définition de terminé, risques, hors étape

Terminé : critères verts ; interrupteurs retirés à la date prévue.

| Risque | Mesure |
|---|---|
| Formats de SMS qui changent | Règles versionnées, repli sur la saisie manuelle |

Hors étape : paiement en ligne (F26), intégration opérateurs.

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 22
- Dépend de : 07
- Tranche : T3
- Jalon : J6
- F : F19, F25
- R : R-079, R-089
- E : E-34
- PP : aucun
- Contenus : C-16
