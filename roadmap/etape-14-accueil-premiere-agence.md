# Étape 14 — Accueil de la première agence pilote

| Champ | Valeur |
|---|---|
| Statut | À faire |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | Pilote (procédure d'exploitation) |
| Effort | M, environ 10 JD de développeur et de support, sur 6 à 8 semaines calendaires |
| Dépend de | Étape 13 ; contenus C-05, C-06, C-07, C-11, C-15, C-17 |
| Jalon | **J4 — Premier pilote** (données réelles en `setup`) puis **J5 — Bascule `live`** |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, `DEPLOYMENT.md` §4 et §8.1 |

## 1. Objectif et valeur pour le pilote

Appliquer la procédure d'accueil de `DEPLOYMENT.md` §8.1 : créer l'organisation, découvrir les règles réelles de l'agence, configurer, importer, inviter, vérifier, former, mener deux semaines de double tenue en `setup`, puis basculer en `live`.
Valeur : premières données réelles, premières règles confirmées sur le terrain (R-071, R-096, R-098 à R-109), premiers indicateurs du pilote.

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections |
|---|---|---|
| Procédure | `DEPLOYMENT.md` | §4 (calendrier), §6.3, §8.1 (10 étapes), Questions ouvertes 6 à 8 |
| Règles à confirmer | `DOMAINE.md` | « Règles à confirmer avec l'agence pilote » |
| Critères | DP-53 ; `CONTEXTE.md` H1 à H12 | `DECISIONS.md`, `CONTEXTE.md` |
| Recette | `TESTING_QA.md` | §13 en entier |
| Mesures de départ | dossier §23 | durée de la clôture mensuelle, taux d'impayés à J+30, litiges par trimestre |

## 3. Prérequis

J3 atteint ; lettre d'intention signée et référent désigné (C-06) ; engagement de confidentialité et accord écrit (données anonymisées, droit de purge) signés (C-05) ; conditions du pilote fixées (C-07) ; calendrier des jours fériés (C-17) ; mesures de départ de l'agence relevées (C-15) ; tarifs à tester prêts (C-11).

## 4. Fiche de spécification à produire ou valider

`specs/accueil-agence-01.md` : planning daté des 10 étapes, personnes, règles à confirmer (liste ci-dessus), questions de découverte tirées de `MAQUETTE.md` « Usage en démonstration », date de bascule candidate (jours 11 à 24, hors vendredi, `DEPLOYMENT.md` §4) et condition « étape 17 livrée avant le premier mois de clôture » [À valider, décision n° 103]. **S'arrêter pour validation de Mimis.**

## 5. Tâches détaillées

1. **Création** : `immo org create` (état `setup`, `kind` réel, préfixe de 3 ou 4 lettres) ; lien d'invitation du gérant transmis en main propre (30 min).
2. **Découverte** (2 h, développeur + porteur) : flux d'argent, vocabulaire, règles de commission, reddition (date d'encaissement ou d'échéance), jour d'échéance, premier mois, bail antérieur, doublons de références, plafond de dépôt ; consigner chaque règle confirmée, infirmée ou ajustée comme question dans `DOMAINE.md` ou entrée `DP-xx`.
3. **Configuration** (1 h) : logo, en-tête, signature, modèles, rôles et droits, seuil de confirmation, durée des liens.
4. **Import** (1 à 3 jours) : fichier nettoyé avec l'agence, accord `import_assist`, import, contrôle ligne à ligne d'un échantillon de 20 locataires, rapprochement (nombre de baux, somme des loyers mensuels, arriérés totaux).
5. **Invitations** (1 h) : un lien par utilisateur, comptes nominatifs, TOTP du gérant.
6. **Vérification** (2 h) : chaque rôle ne voit que ce que la matrice permet ; quittance d'essai A4 et A5 validée ; paiement, contre-passation et rectificative en série d'essai ; sauvegarde contenant l'organisation.
7. **Formation** (demi-journée) : caissier, gestionnaire, gérant ; séances de recette R1 à R5 et R9 ; guide d'une page.
8. **Double tenue** (2 semaines) : quittances `ESSAI-…`, point hebdomadaire de 30 minutes, relevé des indicateurs.
9. **Bascule `live`** : critères de `DEPLOYMENT.md` §8.1 étape 9 plus étape 17 livrée ou accord écrit sur un premier compte rendu produit autrement ; TOTP actif ; date hors zone rouge et hors vendredi ; arrêt du carnet pour les nouveaux paiements.
10. **Suivis** : J+1, J+7, J+30 (premier compte rendu comparé au calcul manuel), J+60, J+90.

### Tests et exercices
Aucun code : vérifications de la procédure, rapprochement chiffré, comptes rendus de séance (modèle de `TESTING_QA.md` §13.5), aucune capture d'écran de données réelles.

## 6. Critères d'acceptation vérifiables

1. Rapprochement : nombre de baux, somme des loyers mensuels et arriérés égaux aux chiffres de l'agence, ou écart expliqué par écrit.
2. Les 20 locataires d'échantillon sont identiques au carnet.
3. Chaque rôle déroule son scénario sans intervention ; temps d'encaissement mesuré chez l'agence.
4. Règles confirmées ou ajustées toutes consignées avant la bascule.
5. Bascule exécutée avec numérotation officielle à 000001 et TOTP du gérant actif (PP-31).

## 7. Sécurité et déploiement à vérifier

Livraisons selon `DEPLOYMENT.md` §4 ; deux semaines de calme après la bascule ; pas de capture ni enregistrement de données réelles ; accès support limité dans le temps.

## 8. Liste de démonstration (Mimis)

Présenter à l'agence l'organisation en `setup` avec ses propres lots, encaisser en sa présence, imprimer sa quittance, ouvrir le tableau de ses impayés.

## 9. Définition de terminé, risques, hors étape

Terminé : J4 atteint à l'étape 7 ; J5 atteint à l'étape 9 ; compte rendu de découverte et décisions enregistrés ; indicateurs de la première semaine relevés.

| Risque | Mesure |
|---|---|
| Règle métier infirmée par l'agence | Date d'effet, jamais de recalcul ; entrée `DP-xx` |
| Double saisie lassante | Durée limitée à deux semaines, point hebdomadaire |
| Étape 17 en retard | Bascule repoussée, pas de `live` sans reddition |

Hors étape : conception de nouvelles fonctions (étape 19).

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 14
- Dépend de : 13
- Tranche : Pilote
- Jalon : J4, J5
- F : F40
- R : aucune
- E : aucun
- PP : PP-31
- Contenus : C-05, C-06, C-07, C-11, C-15, C-17
