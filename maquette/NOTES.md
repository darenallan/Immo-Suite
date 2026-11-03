# Maquette Immo Suite : notes de réalisation

Doutes, hypothèses prises et écarts avec la spécification. Propriétaire par sujet en cas de conflit : voir `CLAUDE.md`. Ce document fait foi pour : les choix de réalisation de la maquette. Mis à jour à chaque étape.

## Démarrage

- **Audit de cohérence.** `AUDIT_COHERENCE.md` n'a pas de section intitulée « Corrections appliquées ». Il contient un paragraphe « Suivi (24/10/2026) » qui dit que les recommandations validées sont appliquées, et les autres documents le confirment (ils citent DP-29 à DP-43). Le travail a été lancé sur validation explicite de la personne qui pilote le projet. Il manque toujours une vraie section, à ajouter à l'audit.
- **Emplacement.** Les documents sont dans `C:\Immo Suite\Immo Suite\`. `maquette/` est créé à côté d'eux.

## Écarts techniques avec le brief (`MAQUETTE.md`)

- **`data/*.js` et non `data/*.json`.** Ouvert en `file://`, le navigateur interdit de lire des fichiers JSON locaux (`fetch`). Chaque fichier est un script classique qui remplit `window.DEMO` avec du JSON pur. Le contenu reste du JSON.
- **Deux organisations dans chaque fichier.** `DEMO.hor` (Agence Horizon Démo) et `DEMO.saw` (Patrimoine Sawadogo, rôle « Propriétaire direct »).
- **Vérificateur sous Node.** `MAQUETTE.md` prévoit `donnees/verifier.html` car Node n'était pas installé. Node 24 est maintenant présent, donc `scripts/verifier-donnees.js` (28 contrôles) s'exécute avec `node scripts/verifier-donnees.js`. Il ne partage aucun code avec l'interface. Une copie volontairement corrompue des données a bien été détectée.
- **Génération des données.** Le jeu de données a été produit par un script ponctuel, conservé hors du projet. Les fichiers `data/*.js` font foi. Toute modification se refait à la main, puis `node scripts/verifier-donnees.js`.
- **Playwright.** Installé en Python, sans ses propres navigateurs. Les captures utilisent Edge (`channel="msedge"`).
- **Icônes.** Feuille de symboles injectée par `js/icones.js` plutôt qu'un fichier `assets/icons.svg`, car `<use href="fichier.svg#...">` est bloqué en `file://`.
- **Identifiants de code.** Le domaine garde ses mots français (`echeance`, `quittance`), les identifiants techniques sont courts. La décision ouverte n° 18 (langue des identifiants) reste ouverte.

## Hypothèses sur le jeu de données

- **Jour d'échéance 16.** La liste de `MAQUETTE.md` (1, 4, 5, 10, 13, 14, 15, 20, 24, 25, 30) ne contient aucune échéance à J−3 le 13/11, alors que `S03` doit montrer le bloc « Rappels à envoyer » (R-097, DP-40). Un bail a donc une échéance le 16 (Wendyam Kafando, 30 000 FCFA, 15 000 FCFA déjà payés, partiel). Tous les jours listés restent présents. Le jour 20 est porté par un autre bail (payé d'avance).
- **Répartition de novembre.** 23 payés, 2 partiels, 2 avances, 5 en retard, 1 dû (Issouf Tapsoba), soit exactement la cible. Les échéances « à venir » non payées (Yacouba Compaoré le 14/11, Adama Ouédraogo le 24/11) appartiennent à des baux déjà classés « en retard ».
- **Avances.** Boukary Zoungrana : 120 000 FCFA payés pour 100 000, 20 000 affectés à l'échéance du 05/12 (déjà générée). Salamata Konaté : 140 000 pour 120 000, 20 000 en « crédit du bail » (l'échéance du 25/12 n'existe pas encore).
- **93 paiements, prochain numéro HOR-2026-000094.** Quatre paiements sont faits en deux fois (fractionnés) pour atteindre ce volume et coller à l'exemple de `UI_UX_DESIGN.md`.
- **Contre-passation (E-38).** Seuls les paiements encaissés ont une quittance. L'écriture de contre-passation (`C01`, −57 000 FCFA) n'en a pas. Le paiement de remplacement (75 000 FCFA) porte la date réelle du 06/11 et a été saisi le 09/11 ; son numéro de quittance suit la date de saisie. Sa quittance est rectificative et référence la quittance annulée.
- **Compte rendu M1 d'octobre.** Les six loyers sont triés par date d'encaissement. L'exemple de `MAQUETTE.md` place le 09/10 avant le 07/10. Les montants et les totaux sont identiques.
- **Taux de recouvrement.** Formule non précisée par le dossier. Le tableau de bord affiche « encaissé du mois divisé par attendu du mois » (86 %) avec la mention « Formule à confirmer ».
- **Caisse.** Les journées sont calculées sur les paiements en espèces reçus par la caissière. Le 11/11 porte l'écart de −1 000 FCFA expliqué.
- **Numéros de téléphone.** Bloc `+226 00 00 01 NN` (agence), `…02 NN` (Patrimoine Sawadogo), `…03 NN` et `…04 NN` (fichier d'import). Ce bloc non attribué est à vérifier (question ouverte de `MAQUETTE.md`).
- **Noms.** Combinaisons courantes de prénoms et de noms burkinabè, sans personne publique connue à notre connaissance. À faire relire avant la diffusion.
- **Lien de quittance.** Texte d'affichage fictif, jamais cliquable.

## Navigation et rôles

- **Entrées marquées « ? ».** La matrice du dossier ne dit rien du caissier pour Baux, Travaux et documents, Agenda. Choix prudent : elles sont masquées pour lui. Même choix pour les autres rôles qui n'ont pas de droit défini.
- **Écrans reportés (DP-31).** S19 (clôture de caisse), S20 (notifications) et S21 (journal d'audit) ne sont pas construits, donc absents du menu. La cloche ouvre une petite liste des notifications du jeu de données. Les données de caisse, de notifications et d'audit existent déjà.
- **Accès direct.** Une adresse interdite au rôle courant affiche « Accès refusé » (message de `UI_UX_DESIGN.md`).

## Étape 2 : les cinq parcours

- **Modifications de démonstration.** Chaque action (encaisser, relancer, valider un compte rendu, importer, ajouter une intervention) est une fonction nommée de `js/mutations.js`, consignée dans le journal de la session puis rejouée au rechargement. « Réinitialiser la démo » efface le journal et recharge la page : retour exact à l'état initial. Les modifications ne survivent pas à la fermeture de l'onglet (sessionStorage).
- **Parcours 1 (encaisser).** Le scénario complet existe pour Issouf Tapsoba seulement (75 000, 50 000, 100 000 FCFA, affectations précalculées). Pour les autres locataires, la fiche et le solde sont réels et le bouton affiche « Action de démonstration ». Un montant hors des trois proposés affiche « Dans la démo, utilisez les montants proposés » ; au-delà de 1 000 000 FCFA, la modale de confirmation (R-038) s'ouvre puis le même message s'affiche. Référence en doublon et connexion perdue : touche **D** (options de démonstration) ou menu du sélecteur de rôle ; une référence mobile money déjà utilisée déclenche aussi le doublon.
- **Quittance.** Aperçu HTML A5 ou A4, une seule page à l'impression (vérifié en PDF). Le format de page `@page` est posé selon le choix A4 ou A5 (A4 marges 15 mm, A5 marges 10 mm). Lien de quittance : texte fictif non cliquable.
- **Messagerie simulée.** Un seul composant pour les quittances et les relances : texte prêt, bouton « Envoyer », aucun appel à `wa.me`, `tel:` ou `sms:`.
- **Variables des modèles de relance.** Les modèles utilisent `{nom}`, `{montant}`, `{mois}`, `{lot}`, `{date}`, `{lien_quittance}`. `UI_UX_DESIGN.md` n'en liste que quatre ; `{lot}` et `{date}` sont nécessaires au texte de référence de `MAQUETTE.md`. Le message « variable inconnue » liste ces six variables. Le lot s'écrit avec son article (« la Maison V1 », « l'Appartement A3 »).
- **Choix du ton.** J+1 courtois si le retard le plus ancien est inférieur à 5 jours, J+5 ferme de 5 à 9 jours, J+10 dernier avis à partir de 10 jours. Proposition de la maquette, modifiable par l'utilisateur.
- **Impayés groupés par locataire.** Une ligne par bail, avec le nombre d'échéances en retard ; les totaux et tranches du haut comptent les échéances (8). La relance d'un locataire à plusieurs échéances cite le total et la liste des mois.
- **Locataire sans WhatsApp.** C'est Rasmané Bonkoungou (il est en impayé, donc joignable dans le parcours). Pascal Tiemtoré, prévu au départ, n'a pas d'impayé.
- **Comptes rendus d'octobre de M2 à M4.** Précalculés avec le statut « à produire » (vérifiés par le script). « Prévisualiser » les passe en brouillon. L'ajustement « Ajouter une dépense » est un scénario fixe (15 000 FCFA, sans pièce jointe) ; un autre montant affiche le message de la démo.
- **Lot sans mandat (R-075).** Option de démonstration (touche D) : la production affiche « 1 lot n'a pas de mandat (Local T9 (lot de test)) ». Aucun lot n'est créé dans les données.
- **Import.** Le contenu du fichier choisi est remplacé par le fichier d'exemple. Aucune donnée n'est ajoutée au portefeuille ; seule l'historique reçoit une ligne, annulable. Le rapport est un vrai fichier CSV généré dans le navigateur.
- **Fiche lot.** L'onglet actif est dans l'adresse (`#/lot/L001?onglet=travaux`). Lien depuis le compte rendu vers l'intervention et retour (`?intervention=I05`).
- **Toasts et couches.** Quand une modale, un panneau ou une feuille est ouvert, les messages passent en haut de l'écran pour ne pas masquer ses boutons. Toutes les couches se ferment quand l'adresse change.

## Étape 3 : écrans du niveau 2

- **Aucun lien mort.** Pour chacun des 4 rôles, toutes les entrées de menu mènent à un écran réel avec son repère (vérifié par Playwright aux deux largeurs). L'écran « en cours de construction » est supprimé ; une adresse inconnue affiche « Introuvable ».
- **Écrans ajoutés.** S08 Locataires, S09 Fiche locataire, S10 Baux (liste, création en 3 étapes, fiche bail), S13 Mandants et mandats, S15 Agenda, S17 Paramètres, S18 Utilisateurs, S01 Connexion (4 modes), S02 Assistant de démarrage et visite guidée de 3 écrans. Accueil du caissier (« Aujourd'hui ») et du propriétaire direct (« 7 payés sur 10, 2 en retard », rappel en un clic).
- **Accès aux écrans publics.** S01, S02 et la visite guidée n'ont pas d'entrée de menu (ils précèdent la connexion) : ils sont dans le menu du sélecteur de rôle et la feuille « Plus ».
- **Interactions simplifiées.** Locataires, baux, mandants et intervention ne modifient pas les données : formulaires avec leurs garde-fous, puis « (démonstration) ». Modifient vraiment la démo (et sont rejoués au rechargement) : tâches de l'agenda, invitations et rôles des utilisateurs, droits réglables, accès support.
- **Garde-fous montrés au niveau 2.** Téléphone invalide R-018, doublon R-017, pièce d'identité masquée R-019, chevauchement R-020, lot en travaux R-093, fin avant début R-021, loyer nul R-022, dépôt supérieur à 2 mois R-024 (non bloquant), taux hors plage R-015, un seul mandat par lot R-014, avenant non rétroactif R-023 et R-069, dernier gérant R-081, archivage refusé R-006.
- **Espace locataire (F29), indexation automatique (F30), portail mandant (F28), état des lieux (F24), abonnement en ligne (F31).** Éléments désactivés avec le repère « À venir », conformément à `MAQUETTE.md`.
- **Éléments « livré pendant le pilote ».** Agenda, fin de bail et dépôt de garantie de la fiche bail, exports de Paramètres et de S11, onglets Travaux et Documents, zone « Coller le SMS ».
- **Double authentification.** Interrupteur de démonstration ; la tranche est non définie par le dossier (question ouverte de `UI_UX_DESIGN.md`).
- **Propriétaire direct.** Il voit son organisation (Patrimoine Sawadogo) partout : lots, locataires, impayés, paiements, travaux, agenda, import (fichier d'exemple de l'agence), paramètres. L'encaissement complet reste réservé au scénario d'Issouf Tapsoba.
- **Nom provisoire et mentions.** Vouvoiement : valeur provisoire (décision ouverte n° 17). Palette : voir « Changement de palette ».

## Étape 4 : démonstration guidée et mode tournage

- **Accès.** Menu du sélecteur de rôle et feuille « Plus » : « Démonstration guidée » (liste des 5 parcours et du film) et « Mode tournage ». Adresse directe `index.html#/demo/p1` à `p5` ou `film`. Une adresse inconnue renvoie à l'accueil et ouvre la liste.
- **Principe.** Une bulle par étape (titre, explication, voix off du scénario de `MAQUETTE.md`), halo autour de l'élément, Précédent / Lecture / Suivant / Quitter. Clavier : flèche droite, flèche gauche, Espace (lecture automatique, durées issues des tableaux de vidéo), Échap. Les actions sont de vraies actions de l'application (clics et saisies réels), exécutées en quittant l'étape.
- **Départ propre.** Une démonstration repart toujours de l'état initial : si le journal contient des modifications, il est vidé et la page est rechargée. « Précédent » fait de même et rejoue les étapes sans animation, pour qu'aucune modification (un paiement, par exemple) ne reste d'une étape plus avancée.
- **Échap.** Si une fenêtre (modale, panneau, messagerie simulée) est ouverte, Échap la ferme d'abord ; un second Échap quitte la démonstration. Écart avec « Échap = quitter » de `MAQUETTE.md`, pour ne pas perdre la démonstration en fermant une fenêtre.
- **Fichiers.** `js/demo-guidee.js` (moteur), `js/demos/parcours1.js` à `parcours5.js`, `js/demos/film.js` (réutilise les parcours : les étapes non retenues sont jouées sans bulle), `css/demo.css`. Le fichier `demo.css` s'ajoute aux cinq feuilles de style prévues.
- **Film.** 7 séquences (accroche 15 s, encaisser 40, relancer 40, rendre compte 40, migrer 20, mémoire 15, clôture 10). L'accroche est une carte de texte (carnet, tableur, captures, messages) sans image ; la clôture ouvre la légende V1 / À venir.
- **Mode tournage (touche T ou options).** Curseur système masqué et remplacé par un curseur agrandi qui suit la souris, cercle au clic, saisie simulée caractère par caractère, latence coupée, barres de défilement masquées. La bulle devient un sous-titre en bas de l'écran (la voix off), sans boutons : pilotage au clavier. Les repères V1 / À venir ne sont jamais masqués. « Curseur de debug » : la maquette n'en a pas, le curseur système est donc masqué en tournage (hypothèse).
- **Taille fixe (options).** « Ordinateur 1440 × 900 » ou « Mobile 390 × 844 » : la page se recharge dans un cadre (iframe de la même page) de cette taille exacte, mis à l'échelle si la fenêtre est plus petite. Les points de rupture s'appliquent donc à la taille du cadre. Hypothèse : le stockage de session est partagé entre la page et son cadre (vérifié dans Edge, à revérifier dans le navigateur de tournage).
- **Données.** Le film enchaîne les parcours sur les mêmes données : le compte rendu de M1 est « Payé » avant l'étape « Mémoire », qui affiche la dépense dans ce compte rendu.

## Étape 5 : recette finale

- **Script de recette.** Contrôle automatisé (Playwright, hors du projet) de chaque écran de chaque rôle (4 rôles, une quinzaine d'écrans supplémentaires pour le gérant et le propriétaire) à 360, 390, 768, 1024 et 1440 px : console sans erreur ni avertissement, aucun appel réseau, aucun défilement horizontal de la page, repère V1 / À venir présent, aucune date ISO, aucun montant collé ou décimal, aucun « undefined » ou « NaN », aucun tutoiement.
- **Corrigé pendant la recette.** À 1024 px, le texte masqué (`sr-only`) du tableau des impayés sortait de son cadre défilant et élargissait la page de 201 px. À 360 px, deux badges de la chronologie des travaux ne passaient pas à la ligne. Le tableau des impayés défile à l'intérieur de son cadre entre 1024 et 1280 px environ (choix assumé, à ne pas confondre avec un défilement de la page).
- **Contrastes.** Calculés par script (ratio WCAG sur le texte rendu, 11 écrans en 1440 px pour le gérant et le propriétaire) : aucun texte sous 4,5 (3 pour le grand texte). Ce n'est pas un outil tiers : refaire un contrôle avec un outil reconnu avant la diffusion publique.
- **Réinitialisation.** Après chaque parcours et après le film, « Réinitialiser la démo » rétablit un jeu de données octet pour octet identique à l'état initial, au rôle Gérant.
- **Trois tours de suite.** Chaque parcours et le film, rejoués trois fois, donnent le même état final.
- **Recherche de contenus interdits.** Aucun `wa.me`, `tel:`, `sms:`, `mailto:`, e-mail, image distante, ni mention de F26 (paiement en ligne) ou F27 (API WhatsApp). Les seuls liens affichés sont `immo-suite.demo`, fictifs et non cliquables.
- **Non vérifié par machine, à relire à la main.** Orthographe et grammaire des textes ; absence de ressemblance des noms avec des personnes réelles ; fluidité sur le poste de tournage ; contraste avec un outil tiers.
- **Restent ouvertes avant diffusion publique** (`DECISIONS.md`) : n° 13 (nom et marque, d'où la mention « nom provisoire »), n° 17 (vouvoiement). La palette (n° 16) est arrêtée depuis DP-44.

## Changement de palette (DP-44, 25/10/2026)

- **Palette appliquée.** Bleu nuit `#142B4A`, blanc, fond `#F3F6FA`, turquoise `#159D91`, texte `#263449`, à la place du bronze et de l'ocre. Tout passe par `css/tokens.css`, mis à jour comme le bloc de tokens de `UI_UX_DESIGN.md` (copie conforme).
- **Teintes dérivées, pour tenir l'AA.** Le turquoise ne donne que 3,3:1 sur blanc : il reste réservé aux remplissages et aux bordures. Liens en turquoise foncé `#0E6F66` (6,0:1), texte `#0A1626` sur un fond turquoise (5,4:1), turquoise clair `#5FD3C6` sur fond bleu nuit (7,9:1).
- **Menu latéral sur fond bleu nuit** (ordinateur), comme la maquette de référence de la palette : entrée active en `#1E3A63` avec un liseré turquoise clair. Ajout de `--nav-text`, `--nav-text-muted`, `--nav-active-bg` et `--color-accent-300`. Écart : l'ancienne maquette avait un menu clair ; ce choix de mise en page vient de l'illustration de la palette, il se défait en trois lignes de `base.css`.
- **Statuts inchangés.** Seuls le gris « Annulé » (devenu un gris froid) et le repère graphique de « Due » (nouveau `--status-due-mark`, car il utilisait l'ancien accent doré) changent. Aucun statut n'utilise le turquoise.
- **Documents mis à jour.** `DECISIONS.md` (DP-44, n° 16 prise), `UI_UX_DESIGN.md` (tokens, contrastes recalculés, interdits, repli noir et blanc), `MAQUETTE.md` (checklist et question ouverte), `CLAUDE.md`. `AUDIT_COHERENCE.md` est un historique : non modifié, il cite encore les anciennes valeurs (constat daté du 24/10/2026).
- **Reste ouvert.** Logo non fourni : à réconcilier avec la palette. Contrastes calculés par script : à recontrôler avec un outil avant diffusion.

## Logo (26/10/2026)

- **Remplacement du « IS » provisoire.** Le symbole « quittance validée » (pastille bleu nuit, page au coin plié turquoise clair, coche turquoise) et le nom « Immo Suite » en IBM Plex Sans SemiBold remplacent le carré « IS » et le texte de la barre du haut, et ornent l'écran de connexion (S01). Favicon : version simplifiée du symbole.
- **Fichiers.** `assets/symbole.svg`, `assets/symbole-petit.svg` (favicon : sans ligne, coche renforcée) et `assets/nom.svg` (nom converti en tracés : aucune police à charger, la règle « aucune ressource externe » reste vraie). Ce sont des copies de `../logo/svg/`, produits par `../logo/construire_logo.py` (police IBM Plex Sans, licence OFL, dans `../logo/fonts/`).
- **Couleurs.** Celles de `tokens.css` : bleu nuit `#142B4A`, turquoise `#159D91`, turquoise clair `#5FD3C6`. Aucun token modifié.
- **Fichiers modifiés.** `index.html` (barre du haut, favicon), `css/base.css` (classes `.marque-logo`, `.marque-nom-img`), `js/ecrans/connexion.js` (fonction `marque()`).
- **Vérifié.** Chrome, 1440 et 390 px : logo net, aucune erreur console, aucun appel réseau, images chargées depuis `file://`.
- **Non fait.** Le film (`../film/`) n'a pas été régénéré : il montre encore l'ancienne barre du haut. Les documents `MAQUETTE.md` et `DECISIONS.md` disent toujours « logo non fourni » (DP-44 : « à réconcilier avec la palette ») ; `UI_UX_DESIGN.md` n'a pas de section logo.
- **Nom provisoire.** Le bandeau « nom provisoire » reste affiché partout (décision n° 13 ouverte).
