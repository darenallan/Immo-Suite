# Étape 10 — Tableau des impayés et accueil par rôle

| Champ | Valeur |
|---|---|
| Statut | À faire |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T1 (F11) |
| Effort | M, environ 8 JD |
| Dépend de | Étapes 06 et 07 (lecture des échéances et des affectations) ; pour la démonstration J2 : étapes 08 et 09 |
| Jalon | **J2 — Démonstration interne** (avec 05 à 09) |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Donner en une minute la liste de ce qui est dû, classée par ancienneté, filtrable, et un accueil adapté à chaque rôle. Les chiffres viennent de la même fonction que partout ailleurs, donc les totaux concordent par construction.
Valeur : « savoir en une minute qui doit quoi » (promesse du produit) ; l'agence voit enfin ses impayés sans Excel.

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections |
|---|---|---|
| Fonctionnalité | F11 | dossier §16 F11 |
| Règles | R-057, R-058, R-059, R-103 (et R-033, R-095 déjà livrées) | `DOMAINE.md` « Impayés et relances » |
| Cas chiffrés | E-13, E-13b, E-25, E-43 | `DOMAINE.md` |
| Décisions | DP-12 (tranches 1–7, 8–30, 31–59, 60 et +), DP-13, DP-36, DP-51, DP-58 (R-103) ; n° 33 | `DECISIONS.md` |
| Architecture | `ARCHITECTURE.md` | §2.3 module `collections`, §4.9 (une seule fonction de statut), §12.1 (budgets) |
| Contrat | `API_CONTRACTS.md` | §6.2 (parcours 2, partie lecture), §7.7 (`GET /home`, `GET /arrears`) |
| Écrans | `UI_UX_DESIGN.md` | S03 (toutes les variantes), S05, « Tableau responsive », « État vide » |
| Tests | `TESTING_QA.md` | §2.2 (P9, P10), §8.2, parcours 2 |

## 3. Prérequis

Étapes 06 et 07. Décision à consigner : définition précise de « encaissé du mois » et « attendu du mois » pour R-103 (le texte de la règle ne les définit pas : ne pas deviner, ajouter la question à `DOMAINE.md` si elle n'est pas écrite, valeur affichée avec la mention « Formule à confirmer » comme dans la maquette). Le filtre « mandant » s'appuie sur le propriétaire par défaut du bien jusqu'à l'étape 16.

## 4. Fiche de spécification à produire ou valider

`specs/F11-impayes.md` — **règles** : solde impayé = somme des échéances en retard moins les affectations (R-057) ; tranches d'ancienneté (R-058, DP-12) : un retard de 60 jours est dans « 60 j et + » ; une promesse non échue ne masque pas la ligne (R-059, effet visible en étape 15) ; une partielle dépassée reste « en retard » avec payé et reste dû (R-033, DP-13, E-25) ; taux de recouvrement en pourcentage entier arrondi au plus proche (R-103, E-43) ; les valeurs se recalculent à chaque paiement, sans cache obsolète. **Cas limites** : bail sans échéance (alerte discrète), locataire à plusieurs baux, agent terrain (ses lots), échéance annulée par ligne. **Écrans** : S03 (gérant et propriétaire : encaissé, attendu, taux, impayés par ancienneté ; gestionnaire : « À faire aujourd'hui » avec blocs de relance livrés en étape 15 ; caissier : « Aujourd'hui » ; agent terrain : « Ma tournée » ; comptable : lecture ; plan Propriétaire : « X payés sur Y, Z en retard »), S05. **Erreurs** : état vide positif « Aucun impayé. Bravo ». **Cas chiffrés** : E-13 (retards de 3, 20 et 70 jours ⇒ trois tranches), E-13b (30, 31, 59, 60 jours), E-25, E-43 (860 000 sur 1 000 000 = 86 %).

**S'arrêter pour la validation de Mimis après la fiche.**

## 5. Tâches détaillées

### 5.1 Base de données
Index `(org_id, lease_id, due_date)` sur `installments` et `(org_id, installment_line_id)` sur `allocations` ; vues de lecture (aucune table de synthèse tant que la mesure ne l'impose pas, n° 33) pour le total en retard par tranche et le tableau d'accueil ; jeu de 500 lots synthétiques pour la mesure.

### 5.2 Noyau métier
1. `crates/domain` : `arrears_band(days_late)`, `recovery_rate(collected, expected)` (arithmétique entière, arrondi au plus proche), agrégats d'impayés par tranche ; réutilisation de `installment_status` (étape 06).
2. `collections::arrears(scope, today)` : lecture calculée, filtres bien, propriétaire, gestionnaire, usage, portée de l'agent terrain (R-081) ; totaux calculés à partir des mêmes lignes que la liste.
3. `home(scope, role, today)` : contenu par rôle ; la promesse non tenue (DP-36) et les rappels J−3 apparaissent à l'étape 15 (blocs masqués, jamais promis avant livraison).
4. Événements : `arrears_viewed` (filtres, nombre de lignes).

### 5.3 API (`API_CONTRACTS.md` §7.7, §6.2)
`GET /home`, `GET /arrears` (totaux et tranches calculés à part, liste paginée par curseur, 25 lignes, filtres liste blanche, tri liste blanche). Le « Relancer » du tableau n'est pas exposé avant l'étape 15.

### 5.4 Interface
S05 : total en retard, répartition par ancienneté (barres avec texte), tableau filtrable (liste de cartes sur mobile), colonnes locataire, lot, ancienneté, reste dû (les colonnes de relance, promesse et prochaine action apparaissent à l'étape 15), action « Encaisser » qui ouvre S04 pré-rempli ; état vide positif ; S03 : cinq variantes par rôle avec squelette de chargement, bandeau d'erreur et « Réessayer » ; arrivée par rôle (caissier ⇒ Encaisser, gestionnaire ⇒ « À faire aujourd'hui », gérant ⇒ tableau de bord) ; navigation mobile à 4 entrées.

### 5.5 Documents PDF
Aucun (l'export des impayés est en étape 23).

### 5.6 Audit et événements
Lecture seule : aucun audit. Événement `arrears_viewed`, `screen_viewed` (S03, S05).

### 5.7 Tests
- Cas d'or E-13, E-13b (frontières 7/8, 30/31, 59/60), E-25, E-43 ; propriété P10 (chaque retard positif dans exactement une tranche) ; P9 déjà couverte.
- Concordance : le total du tableau de bord est égal à la somme des lignes de la liste (contrôle n° 20 de la maquette) ; un paiement enregistré se reflète au rafraîchissement (pas de cache).
- API : A3, A4, A5 (agent terrain : seulement ses lots), liste ≤ 30 Ko compressés, p95 ≤ 300 ms sur 500 lots.
- E2E parcours 2 (partie lecture) à 390 et 1440 px ; accessibilité automatique de S03 et S05.

## 6. Critères d'acceptation vérifiables

1. Trois échéances en retard de 3, 20 et 70 jours apparaissent dans les tranches 1–7, 8–30 et 60 et + avec les montants exacts (E-13) ; 30, 31, 59 et 60 jours tombent dans 8–30, 31–59, 31–59, 60 et + (E-13b).
2. Échéance de 75 000 payée 50 000 après J+1 : statut « en retard », payé 50 000, reste dû 25 000, présente au tableau (E-25).
3. Attendu 1 000 000 et encaissé 860 000 : « 86 % » avec la mention « Formule à confirmer » (E-43).
4. Un agent terrain assigné à 10 lots n'en voit que 10.
5. Après un paiement, le rafraîchissement montre le nouveau solde ; le total en haut de l'écran égale la somme des lignes.
6. Aucun impayé : état vide positif avec lien vers l'historique.
7. `GET /arrears` répond en 300 ms (p95) sur 500 lots et en moins de 30 Ko compressés par page.

## 7. Sécurité et déploiement à vérifier

Test d'isolation de chaque route ; portée de l'agent terrain testée avec lots assignés et non assignés ; aucune règle d'argent dans le front (recherche en CI) ; migration d'index exercée sur l'essai avec `CREATE INDEX CONCURRENTLY`.

## 8. Liste de démonstration — scénario complet de la démonstration interne (J2)

Mimis déroule, sur l'essai et sur le téléphone de test, le parcours qui fait **J2** :
1. En gérant, créer un bien, un lot, un locataire, un bail de 75 000 FCFA activé (étapes 05 et 06) ; lire l'échéance générée.
2. En caissier, encaisser 50 000, voir la quittance numérotée et son PDF, l'envoyer par WhatsApp (étapes 07 et 08).
3. Se tromper de montant sur un second paiement, demander la correction, puis en gérant contre-passer et remplacer (étape 09).
4. Le développeur avance l'horloge de l'essai au lendemain de l'échéance : ouvrir « Impayés », lire « en retard 25 000 » dans la bonne tranche, filtrer par bien (étape 10).
5. Lire l'accueil de chaque rôle ; vérifier qu'un agent terrain ne voit que son lot.
6. Chronométrer l'encaissement sur le téléphone (première mesure, mesure officielle à l'étape 13).

## 9. Définition de terminé, risques, hors étape

Terminé : critères 1 à 7 verts ; scénario J2 déroulé sans intervention du développeur et consigné (date, écarts) ; `DOMAINE.md` : question ouverte sur la définition d'« encaissé du mois » écrite si besoin ; `API_CONTRACTS.md`, `CLAUDE.md` à jour.

| Risque | Mesure |
|---|---|
| Lenteur du tableau sur de gros portefeuilles | Index, vues, budget mesuré ; table de synthèse seulement si dépassé (n° 33) |
| Taux de recouvrement contesté par l'agence | Mention « Formule à confirmer », règle confirmée à la découverte |
| Accueil trop chargé sur mobile | Variantes par rôle, test sur l'Android d'entrée de gamme |

Hors étape : relances, promesses, rappels J−3 (étape 15), fins de bail à 60 jours et notifications (étape 23), exports (étape 23).

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 10
- Dépend de : 06, 07
- Tranche : T1
- Jalon : J2
- F : F11
- R : R-057, R-058, R-059, R-103
- E : E-13, E-13b, E-25, E-43
- PP : PP-06
- Contenus : aucun
