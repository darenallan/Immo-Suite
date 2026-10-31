# Étape 13 — Durcissement et préparation du pilote (critères « prêt pour un pilote »)

| Champ | Valeur |
|---|---|
| Statut | À faire |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T1/T2 (porte du pilote) |
| Effort | L, environ 15 JD |
| Dépend de | Étapes 03, 04, 09, 10, 11, 12 ; contenus C-04, C-05, C-06, C-12, C-14 |
| Jalon | **J3 — Prêt pour un pilote** |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, `TESTING_QA.md` §12 |

## 1. Objectif et valeur pour le pilote

Vérifier, preuve par preuve, les 38 cases PP-01 à PP-38 pour le périmètre du lot 1, jouer les exercices (restauration, incident, intrusion, mise en production), mesurer le temps pour encaisser sur un téléphone réel et rassembler les documents signés.
Valeur : rien n'est confié à une agence tant qu'une case n'a pas sa preuve (`ARCHITECTURE.md` §9.3).

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections |
|---|---|---|
| Liste unique | `TESTING_QA.md` | §12 en entier, §2.2, §3, §4, §5, §8, §11.4, §13 |
| Sécurité | `SECURITY.md` | §9, §11 |
| Exploitation | `DEPLOYMENT.md` | §5.4 à §5.6, §6, §7.3, §7.5, §10 |
| Décisions | DP-52, DP-53, DP-54/96, DP-55, DP-62, DP-68 ; n° 24, 58, 84, 85, 89, 93 | `DECISIONS.md` |

Convention proposée [À valider, décision n° 97] : PP-05, PP-06 et PP-07 portent sur les cas, propriétés et contrôles **applicables au lot livré** ; les cas de relance (E-14, E-15, E-41) et de reddition (E-16 à E-19, E-26, E-28 à E-30, E-45, E-46) et les contrôles correspondants de la maquette sont vérifiés aux étapes 15 à 17, **avant la bascule `live`**.

## 3. Prérequis

Étapes citées ; **C-04** avis juridique (ou risque accepté par écrit, DP-62) ; **C-05** engagement de confidentialité et accord de pilote ; **C-06** agence candidate et lettre d'intention ; **C-12** heures de couverture, repli, numéro d'urgence (n° 84) ; **C-14** bons usages, certificat de destruction, plan d'incident, guide d'accueil ; téléphone Android réel (C-13).

## 4. Fiche de spécification à produire ou valider

`specs/PP-verification.md` : pour chaque case PP-01 à PP-38, l'étape livreuse, la commande ou le document de preuve, la personne qui vérifie, le statut (vert, applicable plus tard, écart accepté par écrit). **S'arrêter pour validation de Mimis.**

## 5. Tâches détaillées

### 5.1 Base de données
Rejouer les 44 tests du schéma sur Neon en version retenue ; requêtes de contrôle de `DATABASE_SCHEMA.md` §7.5 = 0 anomalie ; audit RLS et gardes actives.

### 5.2 Noyau métier
Aucune fonctionnalité nouvelle ; corrections issues des vérifications uniquement ; propriétés P1 à P16 à 10 000 cas la nuit.

### 5.3 API
Registre de routes = registre des tests d'isolation (100 %) ; contrat OpenAPI sans rupture ; tous les codes d'erreur ont un texte front.

### 5.4 Interface
Accessibilité automatique (zéro violation critique ou sérieuse sur 14 écrans), clavier et TalkBack sur les parcours 1 et 2, budgets de poids et LCP ≤ 3 s sur l'appareil réel, textes relus.

### 5.5 Documents PDF
Impression A4 et A5 vérifiée ; quittance validée par l'agence candidate (PP-27).

### 5.6 Audit et événements
Alertes de `DEPLOYMENT.md` §7.3 testées **une par une** ; page d'état publique et canal d'alerte (PP-36).

### 5.7 Exercices et tests
1. Restauration complète chronométrée sur instance neuve avec les données de l'essai : moins de 4 h, totaux identiques (PP-14).
2. Exercice d'incident sur table (« compte du gérant volé », « fuite entre organisations ») avec compte rendu (PP-16).
3. Auto-test d'intrusion de deux jours par la personne qui n'a pas écrit le module : aucune faille critique ou haute ouverte (PP-24).
4. Mise en production et retour arrière exercés sur l'essai (PP-22).
5. Chronométrage du temps pour encaisser : médiane sous 30 s, 20 paiements variés en moins de 10 minutes (PP-25) ; séances de recette R1 à R5 et R9 (fichier anonymisé) avec l'agence candidate sur l'organisation de démonstration.
6. Gouvernance des accès : inventaire daté, codes de secours placés (PP-33) ; copie d'essai anonymisée (PP-35) ; engagement interne signé et bons usages remis (PP-38).

## 6. Critères d'acceptation vérifiables

1. PP-01 à PP-38 : chaque case a sa preuve consignée ou une mention « applicable plus tard » validée (conventions ci-dessus) ou un risque accepté par écrit par Mimis.
2. Aucune anomalie S1 ou S2 ouverte.
3. Médiane du temps pour encaisser sous 30 s sur le téléphone de test.
4. Décision écrite de mise en production du pilote, signée par le développeur et le porteur du projet.

## 7. Sécurité et déploiement à vérifier

Liste de `DEPLOYMENT.md` §10 entièrement cochée avec preuves ; hébergement hors du pays : position CIL tranchée ou risque accepté par écrit (PP-30).

## 8. Liste de démonstration (Mimis)

1. Lire le tableau PP-01 à PP-38 : preuve, vérificateur, date.
2. Assister à une restauration (durée affichée) et à l'alerte de sauvegarde.
3. Encaisser 20 paiements variés chronométrés sur le téléphone.
4. Lire le rapport d'auto-test d'intrusion et signer la décision de mise en production.

## 9. Définition de terminé, risques, hors étape

Terminé : critères 1 à 4 verts ; `TRACABILITE.md` à jour ; `DECISIONS.md` (n° 24, 58, 84, 85, 89, 93, 97) à jour.

| Risque | Mesure |
|---|---|
| Avis juridique tardif | Risque accepté par écrit selon DP-62 |
| Auto-test trouvant des failles critiques | Correctifs avant J3, pas de dérogation |
| Piste de repli absente | C-12 obligatoire |

Hors étape : test d'intrusion externe (avant la version commerciale), charge de début de mois.

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 13
- Dépend de : 03, 04, 09, 10, 11, 12
- Tranche : T1
- Jalon : J3
- F : F39
- R : aucune
- E : aucun
- PP : PP-05, PP-07, PP-16, PP-24, PP-25, PP-26, PP-29, PP-30, PP-36, PP-37, PP-38
- Contenus : C-04, C-05, C-06, C-12, C-14
