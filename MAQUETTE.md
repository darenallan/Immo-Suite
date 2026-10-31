# Immo Suite — Brief de la maquette interactive

| Champ | Valeur |
|---|---|
| Statut | Brief de la maquette, **réalisée** (`maquette/`, notes de réalisation : `maquette/NOTES.md`). Conservée comme support de démonstration (DP-52). Fichier à archiver après les vidéos |
| Version | 0.2 |
| Date | 30/10/2026 |
| Dépend de | `CONTEXTE.md`, `DOMAINE.md`, `UI_UX_DESIGN.md`, `DECISIONS.md` (DP-09, DP-11, DP-29 à DP-31, DP-43, DP-52) |
| Alimente | `maquette/`, `film/`, `site/` |

Sources : `CONTEXTE.md`, `DOMAINE.md`, `UI_UX_DESIGN.md`, `DECISIONS.md`. La maquette est faite ; ce brief garde ses règles et son jeu de données.

## Objectif

Montrer Immo Suite dans sa version finale, sans logique réelle, à des agences et à des propriétaires pilotes potentiels, par des vidéos de présentation et des démonstrations (DP-52). Aucune enquête terrain n'est menée avec la maquette.

Contraintes issues des décisions :

- Rien n'est promis à tort : chaque écran porte un repère « V1 » ou « À venir » (DP-07, DP-29, DP-30).
- La première maquette couvre le périmètre de DP-31 : les cinq parcours d'abord, puis les écrans du niveau 2. S19 à S24 sont reportés après le démarrage du pilote.
- Le prototype PDF de quittance est un travail séparé et jetable (DP-11). La maquette affiche la quittance en HTML.

## Périmètre à deux niveaux

### Niveau 1 : cinq parcours de bout en bout, très soignés

Ordre de réalisation = ordre ci-dessous. Chaque parcours est cliquable du début à la fin, avec états de chargement, variantes d'erreur déclenchables et transitions soignées.

| N° | Parcours | Rôle de départ | Écrans (catalogue UI_UX_DESIGN) | Vidéo |
|---|---|---|---|---|
| 1 | Encaisser et envoyer une quittance | Caissier | S04, S25 (quittance), S11 | 45 s |
| 2 | Voir et relancer les impayés | Gestionnaire | S03, S05 (panneau Relancer) | 60 s |
| 3 | Produire un compte rendu propriétaire | Gérant | S03, S12, S26 (page de partage) | 75 s |
| 4 | Importer un fichier Excel | Gestionnaire | S16 | 60 s |
| 5 | Consulter l'historique d'un lot (travaux) | Gestionnaire | S06, S07, S14 | 45 s |

### Niveau 2 : tous les autres écrans

Navigables et visuellement finis. Interactions simplifiées : listes triables et filtrables sur les données de démonstration, formulaires qui s'ouvrent et se ferment, boutons d'action qui affichent le toast « Action de démonstration ».

S01 Connexion · S02 Assistant de démarrage · S08 Locataires · S09 Fiche locataire · S10 Baux · S13 Mandants et mandats · S15 Agenda · S17 Paramètres · S18 Utilisateurs et invitations.

Les écrans S03 à S07, S11, S12, S14, S16, S25 et S26 sont aussi consultables hors parcours.

**Reportés après le démarrage du pilote (DP-31)** : S19 Clôture de caisse · S20 Notifications · S21 Journal d'audit · S22 Rapports avancés · S23 Portail mandant · S24 État des lieux. Ils restent spécifiés dans `UI_UX_DESIGN.md`.

### Repère « V1 » et « À venir »

- Chaque écran affiche un repère discret en haut à droite de son en-tête (sous le titre sur mobile).
- Règle (DP-29) : le repère suit la tranche de la fonctionnalité. T1, T2 ou T3 → « V1 ». Fonction de T3 → « V1 » avec la mention « livré pendant le pilote ». Fonction hors tranche → « À venir ».
- Composant : pastille de 24 px de haut.
  - « V1 » : fond `--color-brand-100`, texte `--color-brand-700`, icône coche.
  - « À venir » : fond `--color-accent-100`, texte `--color-brand-700` (14,5:1), contour pointillé, icône sablier. Les textes portent le sens, pas seulement la couleur.
- Légende (DP-30) : lien « Légende » en pied de page, qui ouvre un encart : « V1 : prévu dans la première version ; la mention « livré pendant le pilote » signale une fonction livrée après le démarrage du pilote. À venir : envisagé selon la demande des clients, non planifié, sans engagement de date. »
- Éléments « À venir » à l'intérieur d'un écran V1 : mini-repère à côté de l'élément, élément désactivé.

| Écran | Repère de l'écran | Éléments « À venir » à l'intérieur |
|---|---|---|
| S01 à S13, S16, S17, S18, S25, S26 | V1 | — |
| S09 Fiche locataire | V1 | Espace locataire (F29) |
| S10 Baux | V1 | Indexation automatique du loyer (F30) |
| S12 Comptes rendus | V1 | Portail mandant (F28) |
| S14 Travaux et documents | V1 (livré pendant le pilote) | État des lieux complet (F24) |
| S15 Agenda | V1 (livré pendant le pilote) | — |
| S17 Paramètres | V1 | Abonnement en ligne (F31) |
| S22 Rapports avancés (F32), S23 Portail mandant (F28), S24 État des lieux (F24) | À venir | Reportés après le démarrage du pilote (DP-31) |

Le paiement en ligne (F26) et l'envoi automatique par WhatsApp Business (F27) ne sont montrés nulle part (DP-30).

Éléments de T3 à l'intérieur d'un écran de T1 ou T2, marqués « V1 · livré pendant le pilote » : onglets Travaux et Documents de S07, bloc « fins de bail » de S03, fin de bail et dépôt de garantie complet de S10, zone « Coller le SMS » de S04, exports de S11 et S17.

## Règles de la maquette

- **Aucune logique serveur et aucun vrai calcul métier.** Tous les montants, statuts et totaux viennent du jeu de données. L'écran Encaisser propose des montants préparés (voir parcours 1). Un autre montant affiche le toast « Dans la démo, utilisez les montants proposés ».
- **Pilotée par un jeu de données fictives cohérent.** Voir « Jeu de données fictives ».
- **S'ouvre sans réseau**, par double-clic sur `index.html`. Aucune ressource externe : ni police, ni icône, ni script, ni image distants.
- **Testée en 390 px (mobile) et 1440 px (ordinateur).** Contrôle aussi à 360 px, 768 px et 1024 px.
- **Sélecteur de rôle** en haut de l'écran : Caissier, Gestionnaire, Gérant, Propriétaire direct. Il change le menu, l'écran d'arrivée, les actions permises et les données (voir tableau). L'entrée de menu respecte `UI_UX_DESIGN.md` (tableau « Entrées de menu par rôle »).
- **Bouton « Réinitialiser la démo »** (menu du sélecteur de rôle). Une confirmation, puis retour à l'état initial, au rôle Gérant, à l'écran d'accueil.
- **Mode « Démonstration guidée »** pour les vidéos (voir plus bas).
- **WhatsApp simulé.** « Envoyer sur WhatsApp » ouvre un écran de messagerie simulé avec le texte prêt, sans lien réel et sans logo de marque (le mot « WhatsApp » en texte seulement). Aucun appel à `wa.me`, `tel:` ou `sms:` réel.
- **Quittance affichée en HTML** avec la feuille d'impression de `UI_UX_DESIGN.md`. « Imprimer » ouvre l'aperçu d'impression du navigateur.
- **Latence simulée** de 300 à 600 ms sur les actions (réglable, désactivée en mode tournage).
- **Erreurs déclenchables à la demande** (touche cachée ou option du mode démonstration guidée) : référence de paiement en doublon (R-041) et connexion perdue (R-042).

| Rôle | Organisation | Arrivée | Particularités |
|---|---|---|---|
| Caissier (Awa Zongo) | Agence | S04 Encaisser | Pas de compte rendu ni de paramètres |
| Gestionnaire (Salif Ilboudo) | Agence | S03 « À faire aujourd'hui » | Compte rendu en lecture |
| Gérant (Mariam Kaboré) | Agence | S03 tableau de bord | Accès complet |
| Propriétaire direct (Lassané Sawadogo) | Patrimoine Sawadogo (plan Propriétaire) | S03 accueil simplifié | Pas de mandats ni de comptes rendus |

L'agent terrain et le comptable n'ont pas d'entrée dans le sélecteur (voir Questions ouvertes).

### Mode « Démonstration guidée »

À ne pas confondre avec la visite guidée du produit (onboarding de 3 écrans, `UI_UX_DESIGN.md`).

- Bouton « Démonstration guidée » dans le menu du sélecteur de rôle, ou adresse `index.html#/demo/{parcours}`.
- Une bulle explicative par étape, avec mise en évidence de l'élément. Boutons Précédent, Suivant, Quitter.
- Clavier : flèche droite = suivant, flèche gauche = précédent, Échap = quitter, Espace = lecture automatique (pause avec Espace).
- **Mode tournage** (touche T) :
  - curseur agrandi et cercle visible au clic ;
  - saisie simulée caractère par caractère ;
  - latence simulée coupée ;
  - barres de défilement masquées ;
  - fenêtre de taille fixe 1440 × 900 ou 390 × 844 ;
  - repères « V1 » et « À venir » **toujours visibles**.
- Textes des bulles en français, vouvoiement (provisoire). Un fichier par parcours dans `js/demos/`.

## Choix techniques

HTML, CSS et JavaScript statiques dans `maquette/`, sans backend ni outil de build. Le dossier s'ouvre par double-clic hors réseau, et Node 24, présent sur le poste, sert seulement à vérifier le jeu de données (`maquette/NOTES.md`).

Les tokens du design system sont des variables CSS (`css/tokens.css`, copie du bloc de `UI_UX_DESIGN.md`). Ils sont indépendants du choix du futur frontend (décision n° 6) et se réutilisent tels quels dans le vrai produit.

Contraintes liées à l'ouverture en `file://` :

- Pas de modules ES (`type="module"`), pas de `fetch` de fichiers locaux. Les données sont des scripts classiques qui remplissent `window.DEMO`.
- Pas de service worker.
- `localStorage` et `sessionStorage` entourés de `try/catch`. La maquette fonctionne sans.
- Icônes : une feuille SVG intégrée. Aucune image externe. Logo : nom en texte (aucun logo fourni).
- Navigation par adresse à ancre (`#/encaisser`).

### Structure réalisée

```
maquette/
  index.html · NOTES.md · README.md
  css/        tokens.css · base.css · composants.css · ecrans.css · demo.css · print-quittance.css
  js/         app.js · etat.js · donnees.js · mutations.js · roles.js · format.js · icones.js · composants.js · demo-guidee.js
    ecrans/   un fichier par écran
    demos/    un fichier par parcours
  data/       agence.js · biens.js · lots.js · locataires.js · baux.js · echeances.js · paiements.js · relances.js
              releves.js · travaux.js · documents.js · proprietaires.js · import.js
  scripts/    verifier-donnees.js (28 contrôles)
  assets/     nom.svg · symbole.svg · symbole-petit.svg
```

## Jeu de données fictives

Contrainte absolue : **aucune personne, entreprise ou numéro réel.**

- Noms de personnes : combinaisons de prénoms et noms burkinabè courants, choisies pour ne correspondre à aucune personnalité publique connue. Personnes de la liste fournie, modifiables.
- Entreprises : descriptions génériques (« Quincaillerie (M. X) », « Atelier de couture (Mme Y) »), pas de marque inventée qui pourrait exister.
- Agence : « Agence Horizon Démo » (préfixe `HOR`). Le mot « Démo » reste dans le nom.
- Téléphones : forme `+226 00 00 01 NN`, bloc non attribué [à vérifier]. Aucun numéro réel. Aucun e-mail.
- Montants fictifs, entiers, multiples de 500, entre 25 000 et 350 000. Ils ne représentent pas le marché (le dossier n'a aucune donnée fiable sur les loyers).

### Temps et volumes

| Élément | Valeur |
|---|---|
| Date de démonstration | vendredi 13/11/2026, 09:15 (Africa/Ouagadougou), fixe |
| Mois de référence | novembre 2026 (en cours) |
| Mois clos | octobre 2026 |
| Historique | depuis le 01/09/2026 (début d'usage d'Immo Suite). La numérotation des quittances 2026 commence ce jour-là à 000001 |
| Travaux et documents | peuvent être antérieurs au 01/09/2026 (historique importé) |

### Organisation « Agence Horizon Démo » (plan Agence)

**Utilisateurs.**

| Nom | Rôle |
|---|---|
| Mariam Kaboré (personne A du dossier) | Gérante |
| Salif Ilboudo | Gestionnaire |
| Awa Zongo | Caissière |
| Boureima Yaméogo | Agent terrain (8 lots assignés) |
| Rasmata Belem | Comptable en lecture |

**Mandants et mandats (4).**

| Mandant | Commission | Base | Lots | Habitation / commercial |
|---|---|---|---|---|
| M1 : Mme Georgette Ouédraogo | 8 % | Loyer | 7 | 7 / 0 |
| M2 : M. Jean-Baptiste Compaoré | 7,5 % | Loyer | 12 | 9 / 3 |
| M3 : Succession Lingani | 10 % | Loyer + charges | 11 | 6 / 4 + 1 mixte |
| M4 : M. Alassane Traoré | 6 % | Loyer | 10 | 8 / 2 |

**Lots : 40.**

- 30 habitation, 9 commercial, 1 mixte (bail commercial). Soit 10 lots à usage commercial ou mixte.
- Statuts : 33 occupés, 5 vacants, 2 en travaux (un chez M2, un chez M3).
- Biens : environ 22 (maisons seules, cours, immeubles). Quartiers : Ouaga 2000, Zogona, Gounghin, Tampouy, Patte d'Oie, Cissin, Karpala, Pissy, Tanghin.
- Chaque lot a une référence unique dans son bien et un mandat actif.

**Locataires : 31** pour 33 baux en cours (32 actifs et 1 en préavis, DP-39). Trois locataires ont deux baux. Un bail est en colocation (2 locataires) : 33 − 3 + 1 = 31 locataires distincts. Les locataires commerciaux sont des activités (« Quincaillerie », « Atelier de couture », « Restauration »). Un bail commercial porte l'indicateur « TVA incluse ».

**Jours d'échéance.** Répartis sur 1, 4, 5, 10, 13, 14, 15, 16, 20, 24, 25 et 30 pour que les statuts « à venir », « due », « partiel » et « en retard » existent tous à la date de démonstration.

### Répartition des statuts de novembre 2026

| Catégorie (33 baux) | Cible | Part |
|---|---|---|
| Payé | 23 | ≈ 70 % |
| Partiel ou avance | 4 (2 partiels avant J+1, 2 avances) | ≈ 12 % |
| En retard | 5 | ≈ 15 % |
| Due ou à venir, non payé | 1 (Issouf Tapsoba) | ≈ 3 % |

Tolérance du vérificateur : payé 66–74 %, partiel ou avance 10–18 %, en retard 12–18 %. Le reliquat est l'échéance due non payée du parcours 1.

Modes de paiement (fictifs) : environ 60 % espèces, 30 % mobile money, 7 % virement, 3 % chèque.

Avances : deux. L'une dont l'échéance de décembre est déjà générée (jour d'échéance ≤ 13) : « Avance » affectée. L'autre dont l'échéance de décembre n'existe pas encore (jour d'échéance 25) : « Crédit du bail » (DP-20).

### Ancrages obligatoires (cas de référence de DOMAINE.md)

Ces lignes reprennent des cas de DOMAINE.md à l'identique. Elles ne se changent pas.

| Ancrage | Données | Cas de référence | Utilisé par |
|---|---|---|---|
| Échéance due aujourd'hui | Issouf Tapsoba, Villa B2, Zogona, mandant M2, loyer 75 000, jour 13. Échéance du 13/11/2026 : due, non payée | E-05, E-06, E-37 | Parcours 1 |
| Impayés à 3 jours | Mariam Sanou, loyer 75 000, échéance du 10/11, 50 000 payés. Statut « En retard », payé 50 000, reste 25 000. Promesse en cours au 16/11 | E-25, E-04 | Parcours 2 |
| Impayé à 20 jours + promesse non tenue | Adama Ouédraogo, 60 000, mandant M4, échéance du 24/10. Promesse au 12/11 non tenue : alerte au 13/11 | E-13, E-15 | Parcours 2 |
| Impayé à 31 jours | Rasmané Bonkoungou, 45 000, mandant M1, Karpala, jour 13. Échéance du 13/10 en retard de 31 jours, échéance du 13/11 due | E-13b | Parcours 2, 3 |
| Impayés à 60 et 30 jours | Yacouba Compaoré, atelier commercial, 120 000, mandant M3, jour 14. Échéances du 14/09 (60 j) et du 14/10 (30 j) en retard. Échéance du 14/11 à venir | E-13b | Parcours 2 |
| Impayés à 70, 40 et 9 jours | Pauline Yaméogo, 35 000, mandant M3, Cissin, jour 4. Échéances du 04/09 (70 j), 04/10 (40 j), 04/11 (9 j) en retard | E-13 | Parcours 2 |
| Compte rendu de référence | Mandant M1, octobre 2026 : 400 000 FCFA encaissés par six lots (90 000, 85 000, 75 000, 60 000, 50 000, 40 000), commission 8 %, dépenses imputées 30 000. Commission 32 000, net à reverser 338 000. Impayé au 31/10 : Rasmané Bonkoungou, 45 000 | E-16, E-18 | Parcours 3 |
| Dépense imputée | Lot « Appartement A3 », Ouaga 2000, loyer 90 000. Intervention « Réparation d'une fuite, salle d'eau », 22/10/2026, 30 000, payée par l'agence, imputée au compte rendu M1 d'octobre | E-18 | Parcours 3 et 5 |
| Import | Fichier de 200 lignes, 3 erreurs (lignes 14, 87, 152), 2 paires de doublons de téléphone | E-20, E-21 | Parcours 4 |
| Numérotation | Quittances de 2026 numérotées 000001 à N sans trou, y compris une rectificative | E-10, E-27 | Tous |
| Correction | Un paiement de 57 000 saisi à tort pour un loyer de 75 000, contre-passé : sa quittance Q est « Annulée ». Le paiement de remplacement de 75 000 a sa quittance Q', rectificative, avec son propre numéro et qui référence Q (DP-38). Une demande de correction en attente sur un autre paiement | R-047, R-048, R-096, E-38 | Parcours 1, S11 |

Les six lots de M1 qui totalisent 400 000 sont, dans l'ordre : Appartement A3 (Ouaga 2000, Habiba Kiemdé, 90 000), Villa B1 (Ouaga 2000, Ousmane Belem, 85 000), Appartement C2 (Patte d'Oie, Estelle Sory, 75 000), Maison D4 (Cissin, Moussa Nana, 60 000), Studio E1 (Tanghin, Clarisse Tiendrébéogo, 50 000), Chambre F3 (Pissy, Dramane Kinda, 40 000). Le septième lot de M1 est la Maison G2 (Karpala, Rasmané Bonkoungou, 45 000).

### Autres données

| Objet | Volume et contenu |
|---|---|
| Paiements | 94 depuis le 01/09/2026 (dont une contre-passation), 93 quittances (R-049). Un paiement contre-passé et sa rectificative |
| Affectations | Cohérentes : la somme n'excède jamais le paiement (R-044) |
| Relances | Environ 20, tous canaux (WhatsApp, SMS, appel), avec auteur et résultat |
| Promesses | 3 : une non tenue (Adama Ouédraogo, 12/11), une en cours (Mariam Sanou 16/11), une tenue. La promesse de Pauline Yaméogo (20/11) est créée en direct par le parcours 2 |
| Interventions | 8 à 10, sur plusieurs lots et plusieurs années. Payeurs variés : agence (imputable), propriétaire et locataire (non imputables, DP-18). Statuts : planifiée, en cours, terminée |
| Prestataires | 4 à 5 (plombier, électricien, peintre, serrurier). Noms génériques |
| Documents | Environ 15 : baux scannés, factures, photos de lieux. Représentés par des vignettes SVG fictives |
| Journées de caisse | Aujourd'hui (ouverte, 3 paiements en espèces) ; 12/11, 11/11 et 10/11 clôturées ; une journée avec écart non nul expliqué (− 1 000 FCFA, motif saisi) |
| Relevés | Septembre : M1, M3, M4 « Payé » ; M2 « Validé ». Octobre : M1 « Brouillon » (parcours 3). M2 à M4 d'octobre non produits |
| Agenda et notifications | Fin de bail à 60 jours, promesse non tenue, demande de correction, compte rendu prêt |
| Journal d'audit | Lignes cohérentes avec les actions ci-dessus (qui, quoi, objet, avant/après, heure, adresse IP fictive) |
| Utilisateurs | 5 plus un en invitation expirée et un en attente (S18) |
| Avenants | 2 baux avec un avenant daté du 01/11/2026 (loyer ou charges). Les échéances de septembre et d'octobre gardent l'ancien montant (R-023, E-02) |
| Dépôt de garantie | Un bail d'habitation avec un dépôt supérieur à deux mois de loyer : avertissement non bloquant (R-024, E-22) |

### États scriptés pour les variantes de parcours

Ces états n'existent que le temps de la variante. Ils sont exclus des contrôles de volume (n° 3) et de couverture par un mandat (n° 6) :

- Parcours 3, variante « lot sans mandat » : un lot fictif `L-TEST` sans mandat, créé par la variante et supprimé à la fin (R-075).
- Parcours 2, variante « sans numéro WhatsApp » : un locataire du jeu de données sans numéro WhatsApp, déjà présent dans les données (l'un des 31).
- Parcours 4 : le droit d'importer est activé pour Salif Ilboudo dans les données (réglage « gestionnaire autorisé »).

### Organisation « Patrimoine Sawadogo » (plan Propriétaire)

Utilisée pour le rôle « Propriétaire direct » (personne B du dossier, Lassané Sawadogo).

- 12 lots : 10 habitation et boutiques occupés, 1 vacant, 1 en travaux. 10 locataires, 10 baux (8 habitation, 2 boutiques).
- Novembre 2026 : 7 payés, 1 partiel, 2 en retard.
- Collaborateur : un parent chargé de la collecte (profil « collaborateur »).
- Quartiers : Zogona, Tampouy, Gounghin. Préfixe de quittance `SAW`.
- Aucun mandat, aucun compte rendu.

### Script de vérification de cohérence

- Fichier : `maquette/scripts/verifier-donnees.js` (JavaScript pur, sans accès au navigateur), lancé par `node maquette/scripts/verifier-donnees.js` (Node 24 est présent) : une ligne de réussite ou d'échec par contrôle.
- Le vérificateur recalcule indépendamment tous les montants à partir des règles de `DOMAINE.md` et les compare aux valeurs stockées. Il ne partage aucun code de calcul avec l'interface.
- Il est exécuté après chaque modification du jeu de données et avant tout tournage.

| N° | Contrôle | Règle |
|---|---|---|
| 1 | Tous les montants sont des entiers | R-001 |
| 2 | Aucune date de paiement dans le futur de la date de démonstration | R-039 |
| 3 | Volumes : 40 lots, 10 d'usage commercial ou mixte, 4 mandants, 31 locataires, 33 baux en cours (dont 1 en préavis) | DP-39 |
| 4 | Un seul bail en cours par lot. Pas de chevauchement | R-020 |
| 5 | Statut du lot cohérent avec ses baux (occupé = bail en cours ; en travaux = pas de bail en cours) | R-013, R-093 |
| 6 | Chaque lot couvert par un seul mandat actif | R-014 |
| 7 | Une échéance par bail et par période | R-027 |
| 8 | Somme des affectations ≤ montant du paiement | R-044 |
| 9 | Statut de chaque échéance recalculé à la date de démonstration = statut stocké | R-032, R-033, R-095 |
| 10 | Une échéance au solde non nul est « en retard » dès J+1, même partielle | R-033, DP-13 |
| 11 | Ordre d'affectation : date d'échéance, puis loyer, charges, exceptionnelles | R-087 |
| 12 | Une quittance par paiement. Numéros consécutifs sans trou par année | R-049, R-050 |
| 13 | La quittance d'un paiement contre-passé est « Annulée ». La rectificative du paiement de remplacement la référence | R-051, R-096, DP-38 |
| 14 | Chaque contre-passation est liée à un paiement d'origine | R-047, R-048 |
| 15 | Ancienneté et tranches des impayés (1–7, 8–30, 31–59, 60 et +) | R-058, DP-12 |
| 16 | Commission = taux × encaissé, arrondie au plus proche. Net = encaissé − commission − dépenses imputées | R-068, R-070, DP-14 |
| 17 | Relevé M1 d'octobre : 400 000 → 32 000 → 338 000 | E-18 |
| 18 | Dépenses : imputables seulement si payées par l'agence (par défaut) | R-084, DP-18 |
| 19 | Répartition des statuts de novembre dans les tolérances | — |
| 20 | Totaux du tableau de bord = sommes des listes | — |
| 21 | Cas de référence reproduits à l'identique : E-05, E-06, E-13, E-15, E-16, E-18, E-20, E-25, E-37 | DOMAINE.md |
| 22 | Cas limites du calcul d'ancienneté : 30, 31, 59 et 60 jours | R-058 |
| 23 | Numéros de téléphone au format fictif. Aucune donnée réelle | — |
| 24 | Aucune référence à un identifiant orphelin (clé étrangère cassée) | — |
| 25 | La date de fin d'un bail ne précède pas sa date de début | R-021 |
| 26 | Le loyer de chaque bail est strictement positif | R-022 |
| 27 | Un avenant n'est jamais rétroactif : les échéances antérieures à sa date gardent l'ancien montant | R-023 |
| 28 | Un bail d'habitation dont le dépôt dépasse deux mois de loyer porte l'avertissement non bloquant | R-024 |

## Parcours du niveau 1

Chaque parcours liste : écrans, pas-à-pas cliquable, états à montrer, variante à déclencher.

### Parcours 1 : encaisser et envoyer une quittance (caissière)

- **Données** : Issouf Tapsoba, loyer 75 000, échéance du 13/11/2026 due aujourd'hui.
- **Pas-à-pas** :
  1. La caissière arrive sur S04. Le champ de recherche est actif.
  2. Elle tape « Tapsoba ». La carte du locataire apparaît avec « Due aujourd'hui » et le solde 75 000 FCFA.
  3. Le montant est proposé à 75 000. Trois montants préparés sont disponibles : 75 000 (échéance entière), 50 000 (partiel), 100 000 (avance).
  4. Le mode « Espèces » est présélectionné. La date du jour est affichée.
  5. L'aperçu d'affectation montre l'échéance du 13/11 soldée.
  6. Elle appuie sur « Encaisser et envoyer la quittance ». Chargement bref, puis aperçu de la quittance (S25) : n° suivant le dernier numéro, aperçu A5.
  7. Elle appuie sur « Envoyer sur WhatsApp » : écran de messagerie simulé avec le texte et le lien.
  8. Elle ouvre S11 : le paiement apparaît en tête du journal, statut « Validé » (« Payé » est réservé à l'échéance, DP-43).
- **États à montrer** : chargement du bouton, succès, aperçu A4/A5 de la quittance.
- **Variantes** :
  - 50 000 : « Partiel », reste 25 000 FCFA, quittance avec solde restant (E-05).
  - 100 000 : « Avance » de 25 000 FCFA déduite de l'échéance du 13/12 (E-06).
  - Référence en doublon (mobile money) : message « Ce paiement semble déjà enregistré » (R-041).
  - Connexion perdue : saisie conservée, [Réessayer], aucun doublon (R-042).
  - Paiement > 1 000 000 FCFA (seuil indicatif) : modale de confirmation (R-038).

### Parcours 2 : voir et relancer les impayés (gestionnaire)

- **Données** : les cinq échéances en retard de la liste d'ancrages.
- **Pas-à-pas** :
  1. S03 « À faire aujourd'hui » : promesse non tenue (Adama Ouédraogo, indicateur de T1, DP-36), impayés du jour, bloc « Rappels à envoyer » (échéances à J−3).
  2. Clic sur « Voir les impayés » : S05.
  3. Total en retard et répartition par ancienneté (1–7, 8–30, 31–59, 60 et +). Filtre par mandant M3.
  4. Clic sur la ligne de Pauline Yaméogo : « Relancer ».
  5. Le panneau s'ouvre : modèles courtois, ferme, dernier avis. Choix du modèle selon l'ancienneté. Texte prêt avec nom, montant, mois.
  6. Clic sur « Ouvrir WhatsApp » : écran de messagerie simulé. Retour dans Immo Suite : « Avez-vous envoyé le message ? » → Oui.
  7. Résultat « Message envoyé », promesse de paiement au 20/11/2026 (montant 35 000), prochaine action planifiée.
  8. La ligne affiche la relance, la promesse et la prochaine action. La ligne reste dans la liste (R-059).
- **États à montrer** : tableau chargé, panneau ouvert, avertissement « Une relance a déjà été envoyée aujourd'hui » en relançant une seconde fois (R-063), état vide positif en filtrant sur un bien sans impayé.
- **Variantes** : locataire sans numéro WhatsApp (propose l'ajout) ; variable inconnue dans un modèle (R-064).

### Parcours 3 : produire un compte rendu propriétaire (gérante)

- **Données** : mandant M1, octobre 2026.
- **Pas-à-pas** :
  1. Gérante sur S03, alerte « Comptes rendus d'octobre à produire ».
  2. S12 : liste des comptes rendus. Clic sur « Produire un compte rendu ».
  3. Choix du mandant M1 et de la période octobre 2026.
  4. Aperçu : six loyers encaissés (400 000 FCFA), commission 8 % = 32 000, dépenses imputées 30 000 (Réparation de fuite, A3), net à reverser 338 000 FCFA, impayé de 45 000 FCFA (Rasmané Bonkoungou).
  5. Clic sur « Ajouter une dépense » (démonstration d'ajustement), puis annulation.
  6. « Valider » : modale de confirmation (le relevé sera figé).
  7. Statut « Validé ». « Exporter en PDF » : aperçu du compte rendu. « Partager le lien » : aperçu de la page publique que verrait le mandant (S26).
  8. « Enregistrer le reversement » : date, mode (virement), référence → modale (action définitive, DP-17). Statut « Payé ».
- **États à montrer** : aperçu, validation, version figée, bouton « Rouvrir » visible avant reversement et absent après.
- **Variantes** : lot sans mandat qui bloque la production (R-075) ; dépense sans pièce (avertissement).

### Parcours 4 : importer un fichier Excel (gestionnaire)

- **Données** : `import-exemple.csv`, présenté comme `portefeuille-agence.xlsx` (200 lignes).
- **Pas-à-pas** :
  1. S16, étape 1 : téléverser. Bouton « Télécharger le modèle Excel » et zone de dépôt. Choix du fichier d'exemple.
  2. Étape 2 : association des colonnes, propositions automatiques cochées en vert avec icône.
  3. Étape 3, aperçu : 197 lignes valides, 3 erreurs avec numéro (lignes 14, 87, 152), 2 doublons de téléphone proposés à la fusion (E-20).
  4. Correction : l'utilisateur choisit « Continuer sans ces lignes ». Choix de fusion pour les doublons.
  5. Étape 4, confirmation : modale (rien n'est créé avant). Rapport téléchargeable.
  6. Historique des imports : la ligne apparaît avec « Annulable » (R-080).
- **Règle de cohérence** : l'import de la démonstration **n'ajoute aucune donnée** au jeu de données principal. Il montre le résultat et inscrit une ligne d'historique. Les compteurs du reste de la maquette ne changent pas.
- **États à montrer** : analyse en cours (squelette), erreurs listées, rapport.
- **Variante** : format de fichier invalide (message qui liste les colonnes attendues).

### Parcours 5 : consulter l'historique d'un lot (gestionnaire)

- **Données** : Appartement A3, Ouaga 2000, mandant M1, loyer 90 000.
- **Pas-à-pas** :
  1. S06 : filtre « Ouaga 2000 », clic sur l'Appartement A3.
  2. S07, onglet Aperçu : bail actif, locataire Habiba Kiemdé, échéance du mois payée.
  3. Onglet Travaux (S14) : chronologie des interventions (4 à 6 sur plusieurs années), dont « Réparation d'une fuite, salle d'eau », 22/10/2026, 30 000 FCFA, payée par l'agence, « Imputée au compte rendu d'octobre ».
  4. Clic sur une intervention : prestataire, coût, qui paie, pièces jointes (facture, photos).
  5. Onglet Documents : bail scanné, état des lieux en pièce jointe.
  6. Retour au parcours 3 : lien « Voir dans le compte rendu d'octobre ».
- **États à montrer** : chronologie, détail d'intervention, vignettes de documents.
- **Variante** : ajouter une intervention (formulaire, payeur, imputable), sans modification durable des données.

## Scénarios de vidéo

Ton : sobre, concret, vouvoiement (provisoire). Aucun prix, aucune date de disponibilité. Chaque vidéo montre le repère « V1 ». Sous-titres en français recommandés. Aucune promesse hors du périmètre du repère.

### Vidéo 1 : encaisser et envoyer une quittance (45 s)

| Temps | Écran | Action visible | Voix off |
|---|---|---|---|
| 0:00 | S04 | Recherche « Tapsoba » | « Un locataire arrive pour payer son loyer. » |
| 0:08 | S04 | Carte du locataire, « Due aujourd'hui », 75 000 FCFA | « Vous voyez tout de suite ce qu'il doit. » |
| 0:15 | S04 | Montant, mode Espèces, aperçu d'affectation | « Montant, mode de paiement. L'affectation se fait toute seule. » |
| 0:22 | S04 | Clic « Encaisser et envoyer la quittance » | « Un clic. » |
| 0:27 | Quittance | Quittance n°, montant en chiffres et en lettres | « La quittance est numérotée, avec le montant en lettres. » |
| 0:35 | WhatsApp simulé | Message prêt avec le lien | « Elle part sur le WhatsApp du locataire. » |
| 0:42 | S11 | Paiement dans le journal | « Le paiement est tracé. » |

**Phrase-clé** : « Un paiement, une quittance, en quelques secondes. »

### Vidéo 2 : voir et relancer les impayés (60 s)

| Temps | Écran | Action visible | Voix off |
|---|---|---|---|
| 0:00 | S03 | « À faire aujourd'hui » | « Chaque matin, vous savez par où commencer. » |
| 0:10 | S05 | Répartition par ancienneté | « Qui doit combien, depuis combien de jours. » |
| 0:20 | S05 | Filtre par mandant | « Filtrez par mandant, par bien, par gestionnaire. » |
| 0:28 | S05 | Clic « Relancer » | « Choisissez le ton : courtois, ferme, dernier avis. » |
| 0:38 | WhatsApp simulé | Message prêt | « Le message est prêt. Vous restez maître de l'envoi. » |
| 0:46 | S05 | Promesse de paiement, prochaine action | « Vous notez la promesse. Immo Suite vous le rappellera. » |
| 0:55 | S05 | Ligne mise à jour | « Plus aucune relance oubliée ni perdue. » |

**Phrase-clé** : « Qui doit quoi, depuis quand, et ce qui a déjà été dit. »

### Vidéo 3 : produire un compte rendu propriétaire (75 s)

| Temps | Écran | Action visible | Voix off |
|---|---|---|---|
| 0:00 | S12 | Liste des comptes rendus | « Fin de mois. Vos mandants attendent leur compte rendu. » |
| 0:10 | S12 | Choix du mandant et du mois | « Choisissez le mandant et le mois. » |
| 0:18 | S12 | Aperçu : 400 000 · 32 000 · 30 000 · 338 000 | « Loyers encaissés, commission, travaux : le net à reverser est calculé. » |
| 0:35 | S12 | Détail de la dépense imputée | « Chaque ligne se justifie. » |
| 0:45 | S12 | Validation | « Vous validez : le compte rendu est figé. » |
| 0:55 | S12 | PDF et partage | « Un PDF à partager en un clic. » |
| 1:05 | S12 | Reversement enregistré | « Et le reversement est enregistré avec sa référence. » |

**Phrase-clé** : « Le compte rendu du propriétaire, en deux clics. »

### Vidéo 4 : importer un fichier Excel (60 s)

| Temps | Écran | Action visible | Voix off |
|---|---|---|---|
| 0:00 | S16 | Zone de dépôt | « Vous avez déjà votre portefeuille dans un fichier Excel. » |
| 0:10 | S16 | Association des colonnes | « Immo Suite reconnaît vos colonnes. » |
| 0:20 | S16 | 197 valides, 3 erreurs | « Les erreurs sont listées ligne par ligne. » |
| 0:32 | S16 | Doublons à fusionner | « Les doublons sont proposés à la fusion. » |
| 0:42 | S16 | Confirmation | « Rien n'est créé tant que vous n'avez pas confirmé. » |
| 0:52 | S16 | Rapport | « Et un rapport complet reste disponible. » |

**Phrase-clé** : « Votre carnet devient un outil de gestion, sans tout ressaisir. »

### Vidéo 5 : consulter l'historique d'un lot (45 s)

| Temps | Écran | Action visible | Voix off |
|---|---|---|---|
| 0:00 | S06 | Liste, filtre par quartier | « Un lot, un appartement, une boutique. » |
| 0:08 | S07 | Aperçu du lot | « Le bail, le locataire, l'échéance du mois. » |
| 0:16 | S14 | Chronologie des travaux | « Tous les travaux, avec le coût et qui a payé. » |
| 0:28 | S14 | Pièces jointes | « Factures et photos restent avec le lot. » |
| 0:36 | S12 | Dépense visible dans le compte rendu | « Et la dépense apparaît dans le compte rendu du propriétaire. » |

**Phrase-clé** : « La mémoire du bien, consultable dans cinq ans. »

### Film complet (3 minutes)

| Temps | Séquence | Contenu | Voix off (extrait) |
|---|---|---|---|
| 0:00 à 0:15 | Accroche | Carnet, tableur, captures d'écran, puis écran d'accueil d'Immo Suite | « Les loyers, les preuves, les propriétaires : tout est dispersé. » |
| 0:15 à 0:55 | Encaisser | Vidéo 1 condensée | « Un paiement, une quittance. » |
| 0:55 à 1:35 | Relancer | Vidéo 2 condensée | « Qui doit quoi, et ce qui a été dit. » |
| 1:35 à 2:15 | Rendre compte | Vidéo 3 condensée | « Le compte rendu, en deux clics. » |
| 2:15 à 2:35 | Migrer | Vidéo 4 condensée | « Sans tout ressaisir. » |
| 2:35 à 2:50 | Mémoire | Vidéo 5 condensée | « La mémoire du bien. » |
| 2:50 à 3:00 | Clôture | Écran d'accueil, légende « V1 / À venir » visible | « Voici la première version. Parlons de vos besoins. » |

La clôture invite à une conversation (démonstration chez l'agence, partage d'un échantillon de données), pas à un achat.

## Usage en démonstration

La maquette sert de support aux démonstrations aux agences pilotes potentielles (DP-52). Pour chaque parcours : l'hypothèse du dossier à repérer, et la question à poser à la découverte de l'agence (`DEPLOYMENT.md` §8.1 étape 2). Aucune échelle d'engagement ni aucun seuil : l'observation du pilote est décrite dans `DECISIONS.md` DP-53.

| Parcours | Hypothèse repérée | Question à poser |
|---|---|---|
| 1. Encaisser et envoyer une quittance | H4 (saisie au comptoir), H2 (modes de paiement) | « Vos caissiers pourraient-ils saisir chaque paiement au comptoir ? Quelle part passe en espèces, en mobile money ? » |
| 2. Voir et relancer les impayés | H9 (relances WhatsApp assistées suffisantes), H1 | « Comment relancez-vous aujourd'hui ? Un message prêt à envoyer vous suffirait-il ? » |
| 3. Produire un compte rendu | H3 (reddition pénible), H8 (date d'encaissement) | « Combien de temps prend un compte rendu ? Sur quelle date rattachez-vous un loyer à un mois ? » |
| 4. Importer un fichier Excel | Coût de changement (dossier §2.3) | « Dans quel fichier est votre portefeuille ? Pouvez-vous nous en montrer un extrait anonymisé ? » |
| 5. Historique d'un lot | Valeur de la mémoire du bien | « Comment retrouvez-vous l'historique des travaux d'un lot aujourd'hui ? » |
| Lot commercial (S06, S07, S10) | H7 (champs spécifiques sans moteur différent) | « Vos locaux commerciaux ont-ils des conditions qu'on ne voit pas ici (loyer variable, TVA, charges à part) ? » |

Hypothèses que la maquette ne teste pas : H5 (prix, par la méthode « trop cher / bon marché / limite » de la grille §22.2, jamais affichée dans les vidéos), H6, H10, H11 et H12. Les documents du pilote (lettre d'intention, accord de pilote) sont à créer dans `contenus/` (voir `CLAUDE.md`).

## Textes de démonstration

Les valeurs sont des exemples cohérents avec le jeu de données. Le numéro de quittance, noté `{numéro suivant}`, est généré par le jeu de données (suite sans trou, format de DP-37, par exemple HOR-2026-000094).

### Quittance (A5, paiement entier du parcours 1)

```
AGENCE HORIZON DÉMO
Secteur 15, Ouaga 2000, Ouagadougou — Tél. +226 00 00 01 00

QUITTANCE DE LOYER                      N° {numéro suivant}
                                        Émise le 13/11/2026

Reçu de        : Issouf Tapsoba — +226 00 00 01 12
Pour le lot    : Villa B2 — Zogona (habitation)
Période        : loyer de novembre 2026 (échéance du 13/11/2026)
Paiement       : 13/11/2026 — Espèces

Loyer                                   75 000 FCFA
Charges                                      0 FCFA
TOTAL REÇU                              75 000 FCFA
En lettres : soixante-quinze mille francs CFA

Solde restant dû après ce paiement : 0 FCFA — échéance soldée

Reçu par : Awa Zongo, caissière            [cachet]

Mentions légales complémentaires : à valider
Document généré par Immo Suite
```

- Variante partielle : « Total reçu 50 000 FCFA — Solde restant dû : 25 000 FCFA ».
- Variante avance : « Total reçu 100 000 FCFA — Avance de 25 000 FCFA déduite de l'échéance du 13/12/2026 ».

### Message de relance WhatsApp (ton courtois, retard de 3 jours)

```
Bonjour {nom},

Sauf erreur de notre part, il reste {25 000 FCFA} à régler sur votre loyer
de {novembre 2026} pour la {Maison … Gounghin}, échu le {10/11/2026}.
Si vous avez déjà payé, merci de nous envoyer la référence du paiement.

Cordialement,
Agence Horizon Démo — +226 00 00 01 00
```

Variantes de ton, mêmes variables :

- **Rappel avant échéance (J−3)** : « … votre loyer de {mois} ({montant}) arrive à échéance le {date}. Merci de prévoir le règlement. »
- **Ferme (J+5)** : « … le loyer de {mois} ({montant}) n'est toujours pas réglé. Merci de régulariser ou de nous indiquer une date de paiement. »
- **Dernier avis (J+10)** : « … le montant dû est de {montant}. Merci de contacter l'agence au plus vite pour convenir d'une solution. »

### Message de quittance WhatsApp

```
Bonjour {nom}, voici votre quittance n° {numéro suivant} pour {75 000 FCFA}
(loyer de {novembre 2026}) : {lien_quittance}
Agence Horizon Démo
```

### Compte rendu propriétaire (mandant M1, octobre 2026)

```
COMPTE RENDU DE GESTION — Octobre 2026
Mandant : Mme Georgette Ouédraogo — Mandat à 8 % (base : loyer)
Agence Horizon Démo — établi le 13/11/2026

LOYERS ENCAISSÉS
 04/10  Appartement A3, Ouaga 2000  — Habiba Kiemdé        90 000 FCFA
 05/10  Villa B1, Ouaga 2000        — Ousmane Belem        85 000 FCFA
 06/10  Appartement C2, Patte d'Oie — Estelle Sory         75 000 FCFA
 09/10  Maison D4, Cissin           — Moussa Nana          60 000 FCFA
 07/10  Studio E1, Tanghin          — Clarisse Tiendrébéogo 50 000 FCFA
 10/10  Chambre F3, Pissy           — Dramane Kinda        40 000 FCFA
 Total encaissé                                           400 000 FCFA

Commission de l'agence (8 %)                             − 32 000 FCFA
Dépenses imputées
 22/10  Réparation d'une fuite, salle d'eau, Appartement A3 − 30 000 FCFA

NET À REVERSER                                            338 000 FCFA

IMPAYÉS AU 31/10/2026
 Maison G2, Karpala — Rasmané Bonkoungou — échéance du 13/10  45 000 FCFA

Statut : brouillon
```

La ligne « Statut » passe à « Validé », puis « Payé » (avec date, mode et référence du reversement) au fil du parcours 3.

## Checklist de qualité avant tournage

- [ ] `node maquette/scripts/verifier-donnees.js` : les 28 contrôles sont verts.
- [ ] Les chiffres de chaque écran sont identiques d'un écran à l'autre (tableau de bord, impayés, relevé, fiche locataire).
- [ ] Les cas de référence de `DOMAINE.md` donnent exactement les résultats attendus (E-05, E-06, E-13, E-15, E-16, E-18, E-20, E-25, E-37).
- [ ] Aucune erreur ni avertissement dans la console, sur les cinq parcours et en changeant de rôle.
- [ ] Aucun appel réseau (onglet réseau vide). La maquette s'ouvre sans connexion.
- [ ] Repères « V1 » et « À venir » présents sur chaque écran et chaque élément concerné, légende accessible, visibles en mode tournage.
- [ ] Le bouton « Réinitialiser la démo » rétablit exactement l'état initial, y compris après chaque parcours et chaque variante.
- [ ] Aperçu correct à 390 px et à 1440 px. Contrôle à 360, 768 et 1024 px. Aucun défilement horizontal de la page.
- [ ] Contrastes vérifiés avec un outil sur les statuts et les boutons. Aucun sens porté par la couleur seule.
- [ ] Aucune donnée réelle : noms, entreprises, numéros. Aucun e-mail. Aucun logo de marque tierce.
- [ ] Chaque parcours se déroule sans accroc trois fois de suite.
- [ ] Les textes sont en français, sans faute, vouvoiement uniforme.
- [ ] Quittance, relance et compte rendu : montants exacts, numéro de quittance cohérent avec la suite des paiements.
- [ ] Les dates affichent le format JJ/MM/AAAA, les montants le format « 1 250 000 FCFA ».
- [ ] Le mode tournage coupe la latence simulée et garde les repères visibles.
- [ ] Les trois variantes d'erreur (doublon de référence, connexion perdue, seuil de confirmation) se déclenchent et se referment proprement.
- [ ] Poids total du dossier raisonnable (≤ 3 Mo). Rendu de chaque écran fluide sur le poste de tournage.
- [ ] Palette (décision n° 16, prise : DP-44) et vouvoiement (décision n° 17, ouverte) vérifiés dans `DECISIONS.md` avant la diffusion publique.
- [ ] Nom, domaine et marque vérifiés (décision n° 13) ou mention « nom provisoire » sur chaque vidéo.
- [ ] Le paiement en ligne (F26) et l'API WhatsApp Business (F27) n'apparaissent dans aucun écran ni aucune voix off (DP-30).

## Questions ouvertes

- **Palette et vouvoiement.** La palette est arrêtée (décision n° 16, DP-44 : bleu nuit, blanc, turquoise). Le vouvoiement reste la décision ouverte n° 17 : la maquette utilise la valeur provisoire de `UI_UX_DESIGN.md`. Si elle change, seuls les textes sont à refaire. Si la palette change, seul `tokens.css` est à refaire (et le menu latéral, qui est sur fond de couleur de marque).
- **Le prototype de quittance PDF (DP-11)** est un travail distinct. La quittance de la maquette est en HTML. Faut-il que la vidéo montre la quittance du prototype PDF quand il existera ?
- **Organisation « Patrimoine Sawadogo ».** Le rôle « Propriétaire direct » demande un second jeu de données (12 lots) que le brief de départ ne décrivait pas. Taille et contenu à confirmer.
- **Agent terrain et comptable en lecture.** Absents du sélecteur de rôle. À ajouter si la matrice de permissions doit être démontrée.
- **Numéros de téléphone fictifs.** La forme `+226 00 00 01 NN` suppose que ce bloc n'est pas attribué. À vérifier.
- **Parcours 3 et décision n° 10.** Le compte rendu d'octobre suppose la règle de la date d'encaissement (R-071, H8), encore à confirmer. Tous les loyers du compte rendu de M1 sont encaissés en octobre, ce qui donne le même résultat avec la règle de la date d'échéance.
- **Montant en lettres et mentions de la quittance** : formulation « francs CFA » arrêtée (DP-64) ; mentions à faire relire par un juriste (DP-62).
- **Ton des relances.** Les trois niveaux (courtois, ferme, dernier avis) et la correspondance avec J−3, J+1, J+5, J+10 sont des propositions.
- **Logo.** Livré le 26/10/2026 (`logo/`), repris dans la maquette (`maquette/assets/`).
