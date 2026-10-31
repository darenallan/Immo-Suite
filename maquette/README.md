# Maquette interactive Immo Suite

Maquette statique pour les démonstrations vidéo et les démonstrations aux agences pilotes (agences et propriétaires, Burkina Faso). Toutes les données sont fictives. Le nom « Immo Suite » est provisoire.

## Ouvrir

Double-cliquez sur `index.html`. La maquette fonctionne hors connexion, sans serveur, sans installation. Navigateurs testés : Edge et Chrome récents (ordinateur) ; la mise en page mobile se vérifie en réduisant la fenêtre à 390 px de large.

Au premier chargement : rôle « Gérant » (Mariam Kaboré, Agence Horizon Démo), date de démonstration vendredi 13/11/2026.

## Naviguer

- **Changer de rôle** : bouton en haut à droite. Quatre rôles : Gérant, Gestionnaire, Caissier, Propriétaire direct (autre organisation, plan Propriétaire). Le menu change selon le rôle.
- **Repères** : chaque écran porte « V1 » (prévu pour la première version) ou « À venir » (hors première version). « Légende V1 / À venir » est accessible depuis le bas de chaque page et depuis le menu du rôle.
- **Recherche** : barre du haut (touche `/` sur ordinateur), elle ouvre « Encaisser ».
- **Options de démonstration** (touche `D` ou menu du rôle) :
  - latence simulée (300 à 600 ms) ;
  - prochain encaissement en doublon de référence (R-041) ou avec connexion perdue (R-042) ;
  - compte rendu bloqué par un lot sans mandat (R-075) ;
  - mode tournage et taille de fenêtre fixe.

## Les cinq parcours

| Parcours | Rôle | Point de départ |
|---|---|---|
| 1. Encaisser et envoyer une quittance | Caissier | Encaisser, chercher « Tapsoba » |
| 2. Voir et relancer les impayés | Gestionnaire | Accueil, puis Impayés |
| 3. Produire un compte rendu propriétaire | Gérant | Accueil, puis Comptes rendus |
| 4. Importer un fichier Excel | Gestionnaire | Import, fichier d'exemple |
| 5. Consulter l'historique d'un lot | Gestionnaire | Biens et lots, Appartement A3 |

Le détail des pas-à-pas et des variantes est dans `MAQUETTE.md`. Les messageries WhatsApp et SMS sont **simulées** : rien n'est envoyé, aucun lien externe n'est ouvert. Il n'y a ni paiement en ligne ni API WhatsApp.

## Réinitialiser

Les actions faites pendant la démonstration (paiement, relance, compte rendu validé, tâche d'agenda) sont gardées dans l'onglet du navigateur et rejouées au rechargement. Pour revenir à l'état initial : « Réinitialiser la démo » (bas de chaque page ou menu du rôle). Fermer l'onglet a le même effet. Réinitialiser remet aussi le rôle Gérant.

## Démonstration guidée

Menu du rôle, « Démonstration guidée », ou adresse directe :

| Adresse | Contenu | Durée |
|---|---|---|
| `index.html#/demo/p1` | Encaisser et envoyer une quittance | 45 s |
| `index.html#/demo/p2` | Voir et relancer les impayés | 60 s |
| `index.html#/demo/p3` | Produire un compte rendu propriétaire | 75 s |
| `index.html#/demo/p4` | Importer un fichier Excel | 60 s |
| `index.html#/demo/p5` | Consulter l'historique d'un lot | 45 s |
| `index.html#/demo/film` | Film complet | 3 min |

Une bulle explique chaque étape (avec la voix off prévue) et entoure l'élément concerné. Clavier : flèche droite = suivant, flèche gauche = précédent, `Espace` = lecture automatique (et pause), `Échap` = quitter (un premier `Échap` ferme d'abord une fenêtre ouverte). La démonstration repart toujours de l'état initial des données.

## Enregistrer une vidéo

1. Ouvrez `index.html` dans le navigateur, en plein écran (`F11`) ou dans une fenêtre propre.
2. Touche `T` : **mode tournage**. Curseur agrandi avec cercle au clic, saisie simulée lettre par lettre, latence coupée, barres de défilement masquées. La bulle devient un sous-titre (la voix off) ; les repères V1 / À venir restent visibles.
3. Pour une image à taille fixe : touche `D`, « Taille de la fenêtre en mode tournage », puis « Ordinateur 1440 × 900 » ou « Mobile 390 × 844 ». La maquette s'affiche dans un cadre de cette taille exacte, centré sur fond sombre. Réglez la zone de capture sur ce cadre.
4. Lancez l'enregistreur d'écran, puis la démonstration (par exemple `index.html#/demo/p1`). Avancez avec la flèche droite au rythme de la voix off, ou appuyez sur `Espace` pour la lecture automatique.
5. Après chaque prise : `Échap`, puis relancez la même adresse. Elle repart d'un état propre.
6. Touche `T` pour quitter le mode tournage.

Avant un tournage : lancer `node scripts/verifier-donnees.js` (28 contrôles du jeu de données, doit afficher « Tout est cohérent »). Chaque vidéo doit mentionner « nom provisoire » (décision ouverte n° 13) et montrer le repère « V1 ».

## Contenu du dossier

```
index.html            page unique
css/                  tokens.css (design tokens) · base · composants · ecrans · print-quittance · demo
js/                   app.js (routeur, rôles) · composants.js · mutations.js · demo-guidee.js · …
js/ecrans/            un fichier par écran
js/demos/             un fichier par parcours (1 à 5) et le film complet
data/                 jeu de données fictives (scripts qui remplissent window.DEMO)
scripts/              verifier-donnees.js (contrôles indépendants, Node)
NOTES.md              doutes, hypothèses et écarts avec la spécification
assets/               logo Immo Suite : symbole (quittance validée), symbole petite taille (favicon), nom en tracés
_captures/            captures d'écran des vérifications (supprimables)
```

Aucune ressource externe : pas de CDN, pas de police web, pas d'image distante, icônes en SVG intégré.

## Modifier

- **Couleurs et styles** : `css/tokens.css` uniquement (copie des tokens de `UI_UX_DESIGN.md`).
- **Textes** : dans les fichiers `js/ecrans/*.js` (français, vouvoiement provisoire).
- **Données** : `data/*.js`, puis `node scripts/verifier-donnees.js`. Les chiffres de tous les écrans en découlent.
- **Scénarios de démonstration** : `js/demos/*.js` (une étape = titre, texte, voix off, élément visé, action).

## Limites connues

Voir `NOTES.md`. En résumé : les formulaires de locataires, baux et mandants montrent leurs garde-fous puis affichent « (démonstration) » sans créer de données ; les écrans S19 à S24 (reportés, DP-31) ne sont pas construits ; le lien de quittance affiché (`immo-suite.demo`) est fictif et non cliquable.
