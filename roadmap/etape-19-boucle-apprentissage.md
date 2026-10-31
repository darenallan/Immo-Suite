# Étape 19 — Boucle d'apprentissage pendant le pilote (cycle récurrent)

| Champ | Valeur |
|---|---|
| Statut | À faire (récurrente, un cycle de deux semaines) |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | Pilote |
| Effort | Continu : environ 20 % de la capacité (≈ 2 JD par semaine à deux développeurs) [Hypothèse] |
| Dépend de | Étape 14 |
| Jalon | **J6 — Cadence pendant le pilote** |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, `TESTING_QA.md` §11 et §13.6, `DEPLOYMENT.md` §4 et §6.3 |

## 1. Objectif et valeur pour le pilote

Transformer chaque retour d'agence en correction priorisée, livrée sans casser les encaissements, et décider avec des chiffres quelle tranche suivante construire.
Valeur : le pilote sert à apprendre ; sans boucle, il devient du support.

## 2. Périmètre, règles et documents à lire

Gravités S1 à S4 et délais (`TESTING_QA.md` §11) ; transformation d'un retour en correction (§13.6) ; événements (`ARCHITECTURE.md` §8.4) ; calendrier de livraison et interrupteurs (`DEPLOYMENT.md` §2.8, §4) ; règle d'arbitrage « trois clients » (dossier §20.3, règle d'or 12) ; critères du pilote DP-53 ; KPI du dossier §23. Décision n° 102 (grille de priorité et seuils d'entrée) [À valider].

## 3. Prérequis

Étape 14 (J4) ; observation (étape 04) ; heures de couverture (C-12).

## 4. Fiche de spécification à produire ou valider

`specs/boucle-apprentissage.md` : étiquettes (`type:bogue|friction|demande|question`, `gravité:S1..S4`, `règle:R-xxx`, `écran:Sxx`, `agence:`, `statut:nouveau|qualifié|planifié|livré|vérifié|refusé`, `décision:DP-xx`, `triple:n/3`), grille de priorité, rituels, indicateurs, critères d'entrée d'une tranche. **S'arrêter pour validation de Mimis.**

## 5. Tâches détaillées

### Chaîne d'un retour
1. **Consigner** le jour même (signalement ou note orale, `feedback_reports`). 2. **Qualifier** (bogue, friction, demande, question), **reproduire** sur l'essai, chercher la règle ou le cas concerné ; un retour qui contredit une règle d'argent est une question de règle, jamais corrigé « au passage ». 3. **Gravité** puis **priorité** = gravité × fréquence × portée, avec S1 = passe toujours devant (toute erreur de montant ou d'isolation, une seule occurrence suffit) [grille chiffrée à valider : S1 prioritaire, S2 = 8, S3 = 3, S4 = 1 ; fréquence 1 à 3 ; portée 1 à 3]. 4. **Arbitrer** par la règle des trois clients (paramètre) sinon refus ou service. 5. **Décider** (corriger, paramètre, texte, formation, refus, `DP-xx`). 6. **Critère de vérification** et test de non-régression nommé `regression_<id>_<règle>`. 7. **Livrer** derrière un interrupteur, **confirmer** avec l'agence, **mesurer** à J+7.

### Délais cibles par gravité (n° 84, à valider avec l'agence)
| Gravité | Prise en compte | Contournement | Correctif |
|---|---|---|---|
| S1 | 1 h (information de l'agence dans l'heure si l'encaissement est interrompu) | 4 h | 24 h, gel des livraisons |
| S2 | 4 h ouvrées | 24 h | 72 h |
| S3 | 2 jours ouvrés | — | prochain lot hebdomadaire (14 jours au plus) |
| S4 | tri hebdomadaire | — | règle des trois clients |

### Indicateurs (issus du journal d'événements)
Médiane de `payment_recorded.duration_ms` (cible 30 s) ; part des paiements saisis sous 48 h (cible DP-53 : 60 % ; ensuite 85 %) ; taux de `payment_form_abandoned` ; erreurs d'import par type ; `validation_error` par champ ; `permission_denied` ; `receipt_pdf_failed` ; part des quittances envoyées par WhatsApp ; utilisateurs actifs par semaine (≥ 70 % des invités) ; délai jusqu'à la première quittance (< 48 h) ; signalements ouverts et leur âge ; corrections demandées par caissier. Requêtes dans `ops/sql/indicateurs.sql`.

### Rythme
Point hebdomadaire de 30 minutes avec chaque agence ; revue des indicateurs le vendredi ; livraisons mardi à jeudi, 06 h 00 à 07 h 30, zone verte du 11 au 24 (`DEPLOYMENT.md` §4), jamais en zone rouge sauf S1 ou S2 ; test de restauration en 3ᵉ semaine du mois ; exercice d'incident chaque trimestre ; plafond de 10 interrupteurs actifs.

### Critères pour entamer la tranche suivante (étapes 20 à 23) [À valider]
1. Aucune anomalie S1 ouverte et aucune S2 de plus de 72 h. 2. Deux semaines consécutives avec indicateurs calculés et sans S1. 3. Besoin exprimé par au moins une agence pilote (ou trois clients pour une option). 4. Les corrections absorbent moins de 20 % de la capacité ; sinon différer. 5. Une seule fonction nouvelle à la fois, derrière un interrupteur. 6. Chaque fonction qui écrit de l'argent (caisse, lecture du SMS) n'est allumée qu'après ses cas d'or.

### Décision de fin de pilote (DP-53)
Au moins 60 % des paiements saisis sous 48 h, au moins 2 clients acceptent le tarif cible, aucune anomalie S1 ouverte ; niveau d'engagement 0 à 4 noté par agence.

### Tests
Chaque correction ajoute son test de non-régression ; un S1 ajoute un cas d'or ou une propriété et un retour d'expérience écrit sous 5 jours.

## 6. Critères d'acceptation vérifiables

1. Chaque retour reçu a une étiquette de gravité et un statut dans les 2 jours ouvrés.
2. Les indicateurs ci-dessus sont produits chaque vendredi pour chaque agence.
3. Chaque S1 ou S2 a un test de non-régression et respecte son délai.
4. Une demande sans trois clients n'entre pas dans la règle d'argent.

## 7. Sécurité et déploiement à vérifier

Aucune donnée personnelle dans les indicateurs ; accès aux données client seulement sous accord ; calendrier de livraison respecté.

## 8. Liste de démonstration (Mimis)

Suivre un retour réel de bout en bout : note de séance, étiquettes, priorité, correctif livré un mardi à 06 h 30, confirmation de l'agence, mesure à J+7.

## 9. Définition de terminé, risques, hors étape

Terminé (par cycle) : retours qualifiés, indicateurs publiés, une livraison au plus risquée refusée si le calendrier l'interdit, décisions `DP-xx` enregistrées.

| Risque | Mesure |
|---|---|
| Demandes sur mesure | Règle d'or 12, registre des demandes avec compteur de clients |
| Support qui absorbe la capacité | Plafond de 20 %, services facturés |

Hors étape : fonctions T3 (étapes 20 à 23), paiement en ligne, API WhatsApp (P2).

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 19
- Dépend de : 14
- Tranche : Pilote
- Jalon : J6
- F : F39
- R : aucune
- E : aucun
- PP : aucun
- Contenus : aucun
