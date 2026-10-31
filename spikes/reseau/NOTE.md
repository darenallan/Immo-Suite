# Prototype (e) — Réseau réel depuis Ouagadougou (protocole de mesure)

| Champ | Valeur |
|---|---|
| Statut | **Protocole validé par Mimis le 31/10/2026 ; mesures non faites** : elles se font à Ouagadougou, par Mimis, en deux vagues (DP-75), et la mesure finale en étape 01 quand l'essai existe. Aucun résultat dans cette note |
| Qui mesure | Mimis, à Ouagadougou |
| Durée maximale | 1 jour de préparation et d'exploitation des résultats ; environ 6 heures de terrain réparties sur 3 à 4 jours [À valider] |
| Décision attendue | Région d'hébergement du serveur applicatif (`ARCHITECTURE.md` §11) ; confirmation des budgets de poids et de temps (§12) ; effet de la mise en veille de Neon (DP-69) |
| Documents | `ARCHITECTURE.md` §7 (e) ; `TESTING_QA.md` §8.1 à §8.3 ; DP-69, DP-75 ; contenu C-13 |

## 1. Objectif

Évaluer Immo Suite dans les conditions réelles de connexion des agences : latence vers les régions d'hébergement candidates, chargement du front, parcours « Encaisser », réveil de la base, selon l'opérateur, la technologie (3G ou 4G) et la zone de la ville.

## 2. Quand mesurer (DP-75, calendrier proposé [À valider])

| Vague | Moment | Contenu | Prérequis |
|---|---|---|---|
| 1 | Étape 01 ou 02, dès que l'essai existe | Aller-retour vers l'hébergement, chargement d'un front de démonstration de 150 Ko, réveil de la base après 5 minutes d'inactivité | Serveur d'essai en HTTPS ; téléphone d'entrée de gamme (C-13) ; page de mesure |
| 2 | À partir de J2 (chaîne bien, bail, paiement, quittance) | Parcours Encaisser réel, ouverture de la quittance, liste d'impayés | Application assez avancée pour un test réel |

Optionnel, **sans serveur d'essai** : mesurer l'aller-retour vers les points d'accès publics HTTPS de services cloud de chaque région candidate (Francfort, Paris, Londres, Le Cap) comme approximation de la latence réseau [À vérifier : adresses et réponses] ; à décider seulement si Mimis veut une première indication avant la vague 1.

## 3. Plan d'expérience

- **Conditions** : 2 opérateurs × {3G, 4G} × 3 zones de la ville = **12 conditions**. Zones : une zone centrale, une zone périphérique, une zone à couverture faible, choisies par Mimis et notées avec l'adresse approximative. Deux créneaux (matin 8 h à 9 h ; soir 18 h à 19 h) pour voir la congestion, soit 24 séances d'environ 15 minutes.
- **Appareil** : Android d'entrée de gamme (2 Go de mémoire ou moins, navigateur d'origine), le même pour toutes les séances. Forcer 3G ou 4G dans les réglages du réseau mobile ; noter le type de réseau réellement affiché et, si possible, la force du signal. Si l'appareil ne permet pas de forcer la 3G, le noter.
- **Par séance** : 10 requêtes courtes (aller-retour, temps de premier octet) ; 3 chargements du front à froid (cache vidé) et 3 à chaud ; 3 parcours Encaisser complets (vague 2) ; 1 réveil de la base après au moins 6 minutes d'inactivité (vague 1 et 2).
- **Données personnelles** : aucune ; organisation de démonstration seulement.

## 4. Outil et relevé

- **Page de mesure** (préparée à l'étape 01) : HTML léger, mesures par l'API de chronométrage du navigateur (DNS, connexion, TLS, premier octet, téléchargement), bouton « Exporter » produisant un fichier CSV sur l'appareil. Aucun envoi automatique.
- **Colonnes du relevé** : date, heure, séance, opérateur, technologie affichée, zone, signal (si connu), cible (région ou serveur), type de mesure, valeur (ms), poids transféré (Ko), échec (oui ou non), remarque.

## 5. Seuils de lecture (hypothèses du dossier, `TESTING_QA.md` §8)

| Mesure | Seuil |
|---|---|
| Temps machine du parcours Encaisser sur « 4G lente » (hors saisie) | 5 s au plus ; chemin nominal à 3 allers-retours |
| Premier chargement HTML, CSS et JS compressés | 200 Ko au plus (JS 150 Ko, CSS 30 Ko) |
| LCP en « 4G lente », processeur 4× | 3 s au plus |
| Profil émulé | 4G lente : 150 ms d'aller-retour ; 3G : 400 ms ; hébergement en Europe : +100 à 200 ms [Hypothèse] |
| Serveur | enregistrement p95 ≤ 300 ms ; recherche p95 ≤ 150 ms |

Mimis fait ses mesures, je les analyse (médiane, p95, par condition) et je compare à ces seuils ; aucun seuil n'est modifié sans décision.

## 6. Résultats (à remplir)

| Condition (opérateur, technologie, zone, créneau) | Aller-retour (médiane, p95) | Front à froid | Réveil de la base | Parcours Encaisser |
|---|---|---|---|---|
| | | | | |

## 7. Décision proposée

À rédiger après les mesures : région du serveur (la base étant à Francfort, DP-69), budgets confirmés ou corrigés, nécessité d'un mécanisme contre le réveil de la base (un appel de maintien est incompatible avec la mise en veille voulue par le plan gratuit : à arbitrer).

## 8. Écarts et risques déjà connus

- Aucune mesure n'est possible avant l'essai (étape 01) ; la région du serveur reste provisoire jusqu'à la vague 1.
- Le téléphone d'entrée de gamme (C-13) n'est pas encore acheté.
- Les valeurs de 3G et de 4G « lentes » sont des hypothèses d'émulation, pas des mesures locales.

