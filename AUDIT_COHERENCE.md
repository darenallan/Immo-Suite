# Immo Suite — Audit de cohérence de la première vague documentaire

## A. Synthèse

**Verdict.** Non, pas en l'état. Deux constats BLOQUANTS rendraient la maquette trompeuse : le repère « V1 » est appliqué à trois fonctionnalités (F19, F23, F25) qu'aucune tranche ne porte, et la légende « À venir : prévu plus tard » promet des fonctions que le dossier déclare conditionnelles à la demande. Les arithmétiques, les 15 décisions du dossier et les renvois (R, E, DP, S, F, H) sont sains. Les corrections sont surtout textuelles : des décisions courtes de Mimis, puis une passe d'édition.

**Suivi (24/10/2026).** Mimis a validé toutes les recommandations (cases cochées « OK recommandation »). Elles sont appliquées dans les fichiers concernés. Les décisions correspondantes sont les entrées DP-29 à DP-43 de `DECISIONS.md`, plus la décision ouverte n° 18. Le verdict et les chiffres de cette synthèse sont ceux de l'audit initial, avant correction. Quand une recommandation contenait une question que seul Mimis pouvait trancher, la réponse retenue est la proposition de la recommandation.

**Limite de l'audit.** L'auditeur est le même modèle que le rédacteur de ces documents. L'indépendance est réduite : une relecture humaine des constats BLOQUANT et IMPORTANT est recommandée. Les calculs ont été refaits par scripts PowerShell (conservés hors du projet).

### Nombre de constats par gravité et par type

| Gravité | Contradiction | Incohérence | Manque | Ambiguïté | Doublon | Invention non sourcée | Total |
|---|---|---|---|---|---|---|---|
| BLOQUANT MAQUETTE | 2 | 0 | 0 | 0 | 0 | 0 | **2** |
| IMPORTANT | 6 | 5 | 6 | 3 | 1 | 0 | **21** |
| À SURVEILLER | 1 | 1 | 4 | 4 | 2 | 1 | **13** |
| MINEUR | 0 | 5 | 2 | 1 | 0 | 0 | **8** |
| **Total** | **9** | **11** | **12** | **8** | **3** | **1** | **44** |

### Top 10 des points à trancher AVANT la maquette

1. **C-001** : Que signifie « V1 » pour F19, F23, F25 (aucune tranche) ?
2. **C-002** : Texte de la légende « À venir » et présence de F26, F27 et des écrans S22–S24 dans les vidéos.
3. **C-003** : « V1 » = T1+T2+T3 ? Les parcours 2 et 5 montrent des fonctions de T3 alors que le pilote démarre après T2.
4. **C-022** : Périmètre réel de « maquette complète » (12 écrans du dossier, 24 écrans du catalogue).
5. **C-016** : Les décisions n° 8 et n° 9 sont-elles closes ? CONTEXTE, UI et MAQUETTE les supposent closes.
6. **C-009** : Effet d'une contre-passation sur la quittance (annulée ? rectificative ?). Le jeu de données l'invente.
7. **C-007, C-008** : Le jeu de données est arithmétiquement inexact (30 locataires pour 33 baux avec une colocation, « baux actifs » et préavis).
8. **C-005, C-006** : Parcours 2 et 3 incohérents avec leurs propres données (promesse déjà existante, variante « lot sans mandat » impossible).
9. **C-010, C-011** : Libellé du statut d'un paiement et écrans manquants (aperçu de quittance, pages publiques des liens de partage).
10. **C-012** : Le rappel à J−3 n'a aucun écran et contredit la définition de « Relance ».

### Classement : avant la maquette ou avant le développement

| Moment | Constats |
|---|---|
| Avant la maquette | C-001 à C-017, C-022, C-023, C-027 à C-032, C-036, C-041 à C-044 |
| Avant le premier entretien (DP-10) | C-018, C-019, C-020, C-021 |
| Avant le développement (bloc 2) | C-024 à C-026, C-033 à C-035, C-037 à C-040 |

### Fichiers lus

| Fichier | Dernière modification | Lignes |
|---|---|---|
| `Immo-Suite_Dossier-de-cadrage.docx` (converti en Markdown hors du projet, original intact) | 24/10/2026 18:40 | 26 sections + annexe |
| `DECISIONS.md` | 24/10/2026 20:32 | 387 |
| `CONTEXTE.md` | 24/10/2026 20:32 | 259 |
| `DOMAINE.md` | 24/10/2026 20:24 | 401 |
| `UI_UX_DESIGN.md` | 24/10/2026 20:29 | 855 |
| `MAQUETTE.md` | 24/10/2026 20:32 | 565 |
| `CLAUDE.md` | 24/10/2026 20:29 | 83 |

Aucun autre `.md` à la racine. Pandoc n'est pas installé : la conversion du .docx a été faite par un script PowerShell (extraction de `word/document.xml`).

### Fichiers attendus mais absents

- `ARCHITECTURE.md` (annoncé par l'index de CONTEXTE.md).
- `DATABASE_SCHEMA.md`, `API_CONTRACTS.md`, `SECURITY.md`, `CODING_RULES.md`, `TESTING_QA.md`, `DEPLOYMENT.md`, `ROADMAP.md`, `specs/`, `contenus/` (annoncés « à créer » par CLAUDE.md).
- `maquette/` (le dossier de la maquette, objet de MAQUETTE.md).
- Non annoncés nulle part mais nécessaires : un guide d'entretien et un modèle de lettre d'intention (voir C-021).

## B. Constats

### C-001 — Le repère « V1 » est appliqué à des fonctions que aucune tranche ne porte

- **Gravité** : BLOQUANT MAQUETTE
- **Type** : Contradiction
- **Où** :
  - MAQUETTE.md, « Repère « V1 » et « À venir » » : « Règle : fonctionnalité principale en P0 ou P1 → « V1 ». P2 ou P3 → « À venir ». » et la ligne « | S01 à S21 | V1 | — | ».
  - CONTEXTE.md, « Périmètre » : « **P1 sans tranche assignée dans le dossier** : F19 (clôture de caisse journalière), F23 (fin de bail et changement de locataire), F25 (saisie assistée depuis un SMS mobile money). »
  - Dossier §7.3 : F15, F16, F17 sont notées « P1 (V1) », mais F19, F23, F25 sont notées « P1 » seul. Dossier §9.1 : T3 = « F15, F16, F17, F18, F20, F22 ».
  - DECISIONS.md, DP-02 : « F15, F16 (documents) et F17 (agenda) sont en V1. »
- **Constat** : dans le dossier et DECISIONS, « V1 » est explicitement attribué à F15, F16, F17. La règle de la maquette l'étend à tout P1, y compris F19, F23, F25 qui n'ont ni tranche ni mention « (V1) ». La maquette affiche donc « V1 » sur S19 (clôture de caisse), sur la fin de bail de S10 et sur la zone « Coller le SMS » de S04.
- **Impact** : la maquette promet à des prospects trois fonctions dont personne n'a décidé la livraison. C'est exactement ce que DP-07 veut éviter (« pour ne rien promettre à tort »).
- **Recommandation** : ouvrir DP-29 qui rattache F19, F23 et F25 à une tranche (proposition : T3, car P1 et non bloquantes pour clôturer un mois, dossier §8). La règle MAQUETTE devient « repère = tranche : T1, T2, T3 → V1 ; fonction hors tranche → À venir ». Fichiers à modifier : DECISIONS.md (nouvelle entrée), CONTEXTE.md (« P1 sans tranche » et ses Questions ouvertes), MAQUETTE.md (règle et tableau des repères).
- **Alternative** : créer un troisième repère « À confirmer » pour F19, F23, F25 tant que la tranche n'est pas décidée.
- **Question pour Mimis** : F19, F23 et F25 entrent-elles en T3 ? Si une seule doit venir plus tôt (clôture de caisse pour le pilote ?), laquelle ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-002 — La légende « À venir » promet plus que le dossier ne promet

- **Gravité** : BLOQUANT MAQUETTE
- **Type** : Contradiction
- **Où** :
  - MAQUETTE.md : « « V1 : prévu dans la première version. À venir : prévu plus tard, sans engagement de date. » »
  - Dossier §8 : P2 = « à décider avec des clients payants et des données d'usage » ; P3 = « sans preuve de demande ». Dossier §17, phase 7 : « Déclenchement de chaque P2 uniquement si au moins 5 clients la demandent et acceptent un supplément ou un palier. »
  - UI_UX_DESIGN.md, « Écrans « À venir » » : S22 Rapports avancés (F32), S23 Portail propriétaire mandant (F28), S24 État des lieux (F24). MAQUETTE.md, tableau des repères : éléments « Envoi automatique par WhatsApp Business (F27) » (S05), « Lien de paiement en ligne (F26) » (S11), « Espace locataire (F29) » (S09).
  - CONTEXTE.md, « Exclusions du MVP » : F24, F26, F27, F28, F29 sont des exclusions motivées.
- **Constat** : « prévu plus tard » affirme un plan. Le dossier dit que P2 et P3 sont conditionnels à la demande. De plus, la maquette met en scène des fonctions que le dossier exclut du MVP pour des raisons de coût et de risque (paiement en ligne, API WhatsApp).
- **Impact** : un gérant d'agence comprendra que le paiement en ligne et l'API WhatsApp seront livrés. Cela fausse les réponses à l'entretien (H5, H9) et crée une attente contraire à DP-07.
- **Recommandation** : changer la légende en « À venir : envisagé selon la demande des clients, non planifié, sans engagement de date. » et le sous-titre du repère en « À venir (selon la demande) ». Retirer de S05 et S11 les éléments F27 et F26 (fonctions exclues pour risque réglementaire et coût), et ne garder en « À venir » que S22–S24 si Mimis veut les montrer. Fichiers : MAQUETTE.md (légende, tableau des repères, scripts de film), UI_UX_DESIGN.md (S05, S11, tableau « Écrans À venir »).
- **Alternative** : garder F26 et F27 visibles et désactivés avec la nouvelle légende, pour mesurer la demande en entretien (règle « au moins 5 clients » du dossier).
- **Question pour Mimis** : veut-on montrer des fonctions exclues (paiement en ligne, API WhatsApp) pour mesurer la demande, au risque de créer une attente ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-003 — « V1 » et calendrier du pilote : les parcours 2 et 5 montrent des fonctions de T3

- **Gravité** : IMPORTANT
- **Type** : Ambiguïté
- **Où** :
  - Dossier §9.1 : « Le pilote démarre à la fin de T2 et T3 se construit pendant le pilote. »
  - MAQUETTE.md, Parcours 5 : onglet Travaux (S14), « chronologie des interventions (4 à 6 sur plusieurs années) », onglet Documents (« bail scanné, état des lieux en pièce jointe »). UI_UX_DESIGN.md S14 : « **Priorité** : P1 ». CONTEXTE.md : F15 complet, F16 sont en T3.
  - MAQUETTE.md, Parcours 2 étape 1 : « S03 « À faire aujourd'hui » : promesse non tenue » ; UI S03 : « (déduit de F12, F17, F22) ».
- **Constat** : la définition de « V1 » n'existe nulle part. DP-02 inclut F15–F17 dans la V1, mais l'agence pilote n'aura ni le journal complet des travaux, ni les documents, ni l'agenda avant la fin de la phase 4. Le parcours 5 est l'un des cinq parcours « qui déclenchent la décision d'achat » (dossier §14).
- **Impact** : une agence qui voit « V1 » sur le parcours 5 s'attend à l'avoir au démarrage du pilote.
- **Recommandation** : DECISIONS.md : définir « V1 = T1 + T2 + T3 ; T3 est livrée pendant le pilote ». MAQUETTE.md : le repère devient « V1 » avec une mention « livré pendant le pilote » pour les écrans et éléments de T3 (S14, S15, S20, onglets Documents et Travaux, dépôt de garantie complet). UI_UX_DESIGN.md : reporter la tranche dans chaque ligne « Priorité » du catalogue.
- **Alternative** : réduire le parcours 5 à la version minimale T2 (dépense imputable, F15 minimal) et le présenter comme tel.
- **Question pour Mimis** : « V1 » désigne-t-il le MVP complet (T1+T2+T3) ou la version du pilote (T1+T2) ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-004 — Le critère d'acceptation de F12 (T1) dépend de l'agenda F17 (T3)

- **Gravité** : IMPORTANT
- **Type** : Contradiction (dans le dossier)
- **Où** :
  - Dossier F12 : « Étant donné une promesse au 12 qui n'est pas tenue, quand le 13 arrive, alors une alerte apparaît dans l'agenda du gestionnaire. » F12 : P0, dépend de F11 seul. Dossier F17 : « Agenda et alertes », P1 (V1), T3. Dossier §11.4 : « Promesse de paiement non tenue | Gestionnaire | In-app | Oui ».
  - DOMAINE.md, E-15 : « Alerte dans l'agenda du gestionnaire le 13 ». R-065 : « génère une alerte au gestionnaire » (sans canal).
- **Constat** : la tranche T1 inclut F12 mais pas F17 (T3) ni F22 (notifications, T3). Le critère de F12 ne peut pas être testé en T1. DOMAINE.md reprend la contradiction dans E-15.
- **Impact** : la maquette (parcours 2, étape 1) montre l'alerte dans S03 sans dire à quelle tranche elle appartient ; le test E-15 est impossible à écrire en T1.
- **Recommandation** : DECISIONS.md : « l'alerte promesse non tenue de F12 = un indicateur dans S03 et S05 (T1). L'agenda F17 vient en T3. » DOMAINE.md : reformuler E-15 en « alerte visible dans l'accueil du gestionnaire (S03) ».
- **Alternative** : déplacer F17 (ou une version minimale) en T1. Coût : agrandit la tranche la plus risquée.
- **Question pour Mimis** : l'alerte de promesse non tenue doit-elle exister dès T1, sous quelle forme ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-005 — Parcours 2 : la promesse que l'on crée existe déjà dans les données

- **Gravité** : IMPORTANT
- **Type** : Incohérence
- **Où** : MAQUETTE.md, « Autres données » : « | Promesses | 4 : une non tenue (Adama Ouédraogo, 12/11), deux en cours (Mariam Sanou 16/11, Pauline Yaméogo 20/11), une tenue | » ; Parcours 2, étape 7 : « Résultat « Message envoyé », promesse de paiement au 20/11/2026 (montant 35 000), prochaine action planifiée. » (sur la ligne de Pauline Yaméogo, étape 4).
- **Constat** : l'état initial contient déjà la promesse de Pauline Yaméogo au 20/11. Le parcours la « crée » en direct. Le compteur de promesses passerait de 4 à 5, ou la démonstration doublonnerait une promesse existante. L'étape 8 dit « La ligne affiche la relance, la promesse et la prochaine action », ce qui est faux si la promesse existe déjà.
- **Impact** : le parcours 2 ne se déroule pas comme décrit, et la réinitialisation ne rétablit pas la même vue.
- **Recommandation** : MAQUETTE.md : retirer la promesse de Pauline Yaméogo des données initiales (3 promesses : 1 non tenue, 1 en cours, 1 tenue) et conserver sa création en direct dans le parcours 2. Mettre à jour le contrôle n° 20 (totaux) en conséquence.
- **Alternative** : faire relancer par le parcours un autre locataire sans promesse (ex. Yacouba Compaoré).
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-006 — Des variantes de parcours sont impossibles avec les règles des données

- **Gravité** : IMPORTANT
- **Type** : Incohérence
- **Où** :
  - MAQUETTE.md, Parcours 3 : « **Variantes** : lot sans mandat qui bloque la production (R-075) » ; données : « Chaque lot a une référence unique dans son bien et un mandat actif. » ; contrôle n° 6 : « Chaque lot couvert par un seul mandat actif ».
  - MAQUETTE.md, Parcours 2 : « **Variantes** : locataire sans numéro WhatsApp (propose l'ajout) » ; données : téléphones « forme `+226 00 00 01 NN` » pour chacun.
  - MAQUETTE.md, Parcours 4 : acteur « Gestionnaire » ; UI_UX_DESIGN.md, navigation : « Import | … | Gestionnaire É (si autorisé) ».
- **Constat** : trois démonstrations exigent des données qui violent les contrôles : un lot sans mandat, un locataire sans WhatsApp, un gestionnaire autorisé à importer. La spécification ne les prévoit pas.
- **Impact** : l'implémenteur devra soit déroger au vérificateur, soit supprimer les variantes.
- **Recommandation** : MAQUETTE.md : décrire ces cas comme des états scriptés (« lot fictif L-TEST sans mandat, créé uniquement par la variante »), exclus des contrôles n° 3 et n° 6 ; préciser un locataire sans numéro WhatsApp dans le jeu ; indiquer que le droit « import » est activé pour Salif Ilboudo.
- **Alternative** : supprimer ces deux variantes et garder doublon de référence et connexion perdue.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-007 — Le décompte « 30 locataires pour 33 baux » est faux avec une colocation

- **Gravité** : IMPORTANT
- **Type** : Incohérence
- **Où** : MAQUETTE.md : « **Locataires : 30** pour 33 baux actifs. Trois locataires ont deux baux. … Un bail en colocation (2 locataires). » ; contrôle n° 3 : « 4 mandants, 30 locataires, 33 baux actifs ».
- **Constat** : recalcul : 33 baux − 3 doubles = 30 locataires si chaque bail a un seul locataire. Un bail en colocation ajoute une personne : 31 locataires distincts. Le contrôle n° 3 échouerait sur ses propres données.
- **Impact** : le vérificateur est faux dès le premier jour ; les chiffres affichés (« 30 locataires ») diffèrent de la liste.
- **Recommandation** : MAQUETTE.md : remplacer « 30 » par « 31 » (volumes et contrôle n° 3), ou préciser quels locataires cumulent colocation et deuxième bail pour garder 30.
- **Alternative** : supprimer la colocation du jeu et garder 30 (mais perdre la démonstration de R-088).
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-008 — « Bail actif » et « bail en préavis » : deux sens

- **Gravité** : IMPORTANT
- **Type** : Ambiguïté
- **Où** :
  - DOMAINE.md, R-090 : « Statuts du bail : brouillon, actif, en préavis, terminé. En préavis, le bail reste actif et les échéances continuent jusqu'à la date de départ. » Cycle de vie : « Actif | En préavis | … Le bail reste actif ».
  - DOMAINE.md, R-020 : « Un lot a un seul bail actif à une date donnée. » R-013 : un lot est « occupé » à l'activation.
  - MAQUETTE.md : « 33 baux actifs » et « Un bail en préavis ».
- **Constat** : « actif » est à la fois un statut et un état. Un bail « en préavis » est-il un « bail actif » pour R-020, R-006 (archivage refusé) et R-013 ? Le décompte « 33 baux actifs » est ambigu (32 actifs + 1 en préavis ?).
- **Impact** : le vérificateur ne peut pas coder R-020 et R-013 sans choisir ; la maquette peut afficher 32 ou 33.
- **Recommandation** : DOMAINE.md : introduire le terme « bail en cours = statut actif ou en préavis » et l'utiliser dans R-006, R-013, R-020 et le glossaire. MAQUETTE.md : « 33 baux en cours (dont 1 en préavis) ».
- **Alternative** : faire de « en préavis » un indicateur du bail actif (date de départ renseignée) et non un statut.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-009 — Effet d'une contre-passation sur la quittance : règle absente, jeu de données qui l'invente

- **Gravité** : IMPORTANT
- **Type** : Manque
- **Où** :
  - DOMAINE.md, R-048 (« contre-passe, par une écriture inverse liée à l'original ») ; R-051 (« Une erreur se corrige par une quittance rectificative qui annule et remplace. La rectificative reçoit son propre numéro suivant et référence l'originale ») ; cycle de vie Quittance : « Émise | Annulée | Émission d'une quittance rectificative qui la remplace ».
  - MAQUETTE.md, ancrage « Correction » : « Un paiement de 57 000 saisi à tort pour un loyer de 75 000, contre-passé. Une quittance rectificative. Une demande de correction en attente ». Contrôle n° 13 : « Une rectificative référence l'originale. L'originale est « Annulée » ».
- **Constat** : aucune règle ne dit si la contre-passation d'un paiement annule sa quittance, ni si le paiement de remplacement reçoit une quittance normale ou une rectificative, ni ce que « remplace » signifie quand le montant change (57 000 → 75 000).
- **Impact** : le jeu de données crée un cas que DOMAINE ne définit pas ; la numérotation « sans trou » (R-050) en dépend.
- **Recommandation** : DOMAINE.md : ajouter une règle après R-051. Proposition : la contre-passation du paiement P passe sa quittance Q à « Annulée » ; le paiement de remplacement P' génère sa propre quittance Q', marquée « rectificative » et référençant Q. Cas d'exemple dérivé E-38 (57 000 → 75 000). Fichier : DOMAINE.md.
- **Alternative** : « rectificative » ne sert que pour une erreur sans effet sur le montant (nom, période). Une erreur de montant passe toujours par contre-passation et nouvelle quittance ordinaire, qui référence l'annulée.
- **Question pour Mimis** : quelle option correspond à la pratique des agences (à valider en entretien) ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-010 — Trois mots pour l'état d'un paiement

- **Gravité** : IMPORTANT
- **Type** : Incohérence
- **Où** : MAQUETTE.md, Parcours 1, étape 8 : « le paiement apparaît en tête du journal, statut « Payé », badge « Enregistré » » ; DOMAINE.md, cycle de vie Paiement : « Validé », « Contre-passé » ; UI_UX_DESIGN.md, tableau des statuts : « Paiement contre-passé : libellé « Corrigé » avec infobulle ». UI S11 : « statut » sans valeurs.
- **Constat** : « Payé » est le statut d'une échéance (DOMAINE R-032). Pour le paiement, DOMAINE dit « Validé », UI ne définit que « Corrigé », et MAQUETTE invente « Enregistré ».
- **Impact** : l'écran S11, montré dans la vidéo 1, affiche un vocabulaire non défini ; le test visuel ne peut pas valider un libellé que personne ne spécifie.
- **Recommandation** : UI_UX_DESIGN.md : ajouter au tableau des statuts le libellé de paiement « Validé » (famille « Payé » ou neutre) et « Corrigé » ; MAQUETTE.md Parcours 1 : « statut « Validé » ». Réserver « Payé » à l'échéance.
- **Alternative** : « Enregistré » partout (UI et MAQUETTE), avec une correspondance « Enregistré = Validé » dans le glossaire de DOMAINE.md.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-011 — Écrans nécessaires aux parcours absents du catalogue

- **Gravité** : IMPORTANT
- **Type** : Manque
- **Où** :
  - MAQUETTE.md, parcours 1 : « | 1 | Encaisser et envoyer une quittance | Caissier | S04, quittance, S11 | 45 s | ». Le catalogue UI_UX_DESIGN.md va de S01 à S24 sans écran « Quittance ». UI S04 : « succès (écran de quittance avec son n°) ».
  - Dossier F10 : « Les liens de partage expirent après une durée configurable et ne donnent accès qu'à ce document. » DOMAINE.md, E-12 : « Lien de quittance ouvert après expiration → Accès refusé ». UI S12 : « lien de partage ». MAQUETTE.md, parcours 3 : « « Partager le lien » ».
  - MAQUETTE.md : « WhatsApp simulé » (écran de la maquette).
- **Constat** : trois écrans n'ont ni identifiant ni spécification : l'aperçu/détail de quittance (A4/A5), la page ouverte par le destinataire d'un lien de partage (quittance ou compte rendu) avec son cas « lien expiré », et l'écran de messagerie simulé.
- **Impact** : les parcours 1 et 3 se terminent sur des écrans non spécifiés ; la règle « chaque écran porte un repère » ne s'applique à rien ; E-12 n'est pas démontrable.
- **Recommandation** : UI_UX_DESIGN.md : ajouter S25 « Quittance (aperçu, détail, impression) » (F10, P0) et S26 « Page de partage publique et lien expiré » (F10, F13, P0). MAQUETTE.md : lister « messagerie simulée » comme écran propre à la maquette, sans repère.
- **Alternative** : décrire S25 et S26 comme des états de S04/S11/S12 (modale, vue) sans identifiant.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-012 — Le rappel à J−3 n'a aucun écran, et « Relance » est défini autrement

- **Gravité** : IMPORTANT
- **Type** : Contradiction
- **Où** :
  - DOMAINE.md, glossaire : « Relance | Contact pour un impayé : … » ; R-060 : « Étapes de relance : rappel courtois à J−3, puis modèles différents à J+1, J+5, J+10 ».
  - Dossier §11.4 : « Loyer à J−3 (rappel courtois) | Locataire | WhatsApp / SMS | Assisté au MVP ».
  - UI_UX_DESIGN.md S05 : « total en retard · répartition par ancienneté · tableau filtrable » ; S03 : « échéances dues aujourd'hui » ; S17 : modèles « J−3, J+1, J+5, J+10 ».
- **Constat** : à J−3 il n'y a pas d'impayé, donc pas de « relance » au sens du glossaire. Aucun écran ne liste les échéances à rappeler à J−3. Le modèle existe (S17) mais pas le point d'entrée.
- **Impact** : une fonction du dossier est invisible dans l'interface. La maquette ne la montre pas, donc ne la teste pas.
- **Recommandation** : DOMAINE.md : redéfinir « Relance » = « Contact avec un locataire au sujet d'une échéance : rappel avant l'échéance (J−3) ou relance après retard ». UI_UX_DESIGN.md S03 (gestionnaire) : ajouter un bloc « Rappels à envoyer » (échéances à J−3), qui ouvre le panneau « Relancer » de S05.
- **Alternative** : retirer J−3 du MVP et le déclarer dans DECISIONS (le dossier le prévoit pourtant « assisté au MVP »).
- **Question pour Mimis** : le rappel J−3 est-il dans le MVP ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-013 — « Accueil » des rôles caissier et agent terrain non défini

- **Gravité** : IMPORTANT
- **Type** : Manque
- **Où** : UI_UX_DESIGN.md, S03 : contenu défini pour « Gérant et propriétaire », « Gestionnaire », « Plan Propriétaire ». Tableau « Entrées de menu par rôle » : « | Accueil | É | É | É | É (ses lots) | L | É | ». « **Arrivée par rôle** : caissier → Encaisser. … Agent terrain → ses lots (déduit). » MAQUETTE.md : « Caissier … Arrivée | S04 Encaisser ».
- **Constat** : la barre du bas propose « Accueil » à tous. Pour le caissier et l'agent terrain, le contenu de « Accueil » n'existe pas. Pour le caissier, « Encaisser » est à la fois l'arrivée et une autre entrée de la barre.
- **Impact** : deux entrées de barre pointeraient vers le même écran, ou « Accueil » serait vide.
- **Recommandation** : UI_UX_DESIGN.md S03 : ajouter la variante caissier (« Aujourd'hui » : encaissements du jour, journal de caisse du jour, bouton « Encaisser ») et la variante agent terrain (sa tournée). Garder l'arrivée caissier sur S04 comme dans le dossier §14.1.
- **Alternative** : pour le caissier, remplacer « Accueil » par « Caisse » (S19) dans la barre.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-014 — Barre du bas à 4 entrées fixes contre masquage des entrées interdites

- **Gravité** : IMPORTANT
- **Type** : Contradiction
- **Où** : UI_UX_DESIGN.md, Navigation : « **barre du bas à 4 entrées : Accueil, Encaisser, Impayés, Plus** » et « Droits : une entrée interdite est masquée. » Tableau des entrées : « | Encaisser | É | É (réglable) | É | — | — | É | » (agent terrain et comptable : « — »). Dossier §14.3 : « barre du bas à 4 entrées sur mobile (Accueil, Encaisser, Impayés, Plus) ».
- **Constat** : pour l'agent terrain et le comptable, « Encaisser » est interdit mais fait partie des 4 entrées imposées. Masquer donne une barre à 3 entrées ; garder viole les droits.
- **Impact** : les deux règles ne peuvent pas être respectées ensemble.
- **Recommandation** : UI_UX_DESIGN.md : définir la barre par rôle en gardant 4 entrées : agent terrain = Accueil, Impayés, Locataires, Plus ; comptable = Accueil, Impayés, Paiements, Plus ; les autres rôles = barre du dossier.
- **Alternative** : afficher « Encaisser » grisé avec la raison pour les rôles sans droit (moins propre sur Android d'entrée de gamme).
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-015 — « Propriétaire » : trois sens, et un sélecteur de rôle ambigu

- **Gravité** : IMPORTANT
- **Type** : Ambiguïté
- **Où** : MAQUETTE.md, sélecteur : « Gérant (Mariam Kaboré) … Propriétaire (Lassané Sawadogo) ». UI_UX_DESIGN.md : colonne « Propriétaire (plan Propriétaire) » ; S10 : « gérant É · gestionnaire É · propriétaire É » ; S12 : « Comptes rendus propriétaires » (destinataire = mandant) ; S13 : « Propriétaires et mandats ». DOMAINE.md, glossaire : « Propriétaire direct », « Mandant ». Occurrences du mot « propriétaire » : UI 33, MAQUETTE 21.
- **Constat** : « propriétaire » désigne l'utilisateur du plan Propriétaire (propriétaire direct), le destinataire d'un compte rendu (mandant) et le titre d'un écran (S13). Le sélecteur de rôle mélange rôle (gérant) et plan (propriétaire).
- **Impact** : un lecteur confond l'acheteur du plan Propriétaire et le mandant d'une agence ; la vidéo 3 parle de « vos propriétaires ».
- **Recommandation** : UI_UX_DESIGN.md et MAQUETTE.md : « Propriétaire direct » pour le rôle et le plan ; « mandant » pour le bénéficiaire d'un compte rendu ; garder « compte rendu propriétaire » comme nom du document (dossier §14.2) en le définissant dans DOMAINE.md ; renommer S13 « Mandants et mandats ». Libellé du sélecteur : « Propriétaire direct ».
- **Alternative** : renommer le document « compte rendu mandant » (s'écarte du dossier).
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-016 — Les décisions n° 8 et n° 9 sont ouvertes dans DECISIONS mais supposées closes ailleurs

- **Gravité** : IMPORTANT
- **Type** : Contradiction
- **Où** :
  - DECISIONS.md, DP-08 : « Le paiement en ligne par le locataire (F26) reste hors MVP (P2). La décision n° 8 reste ouverte. » Tableau : « | 8 | Paiement en ligne | MVP ; P2 ; jamais | … | Ouverte | » et « | 9 | Relances | Lien pré-rempli ; API WhatsApp | … | Ouverte | ». Règle de mise à jour : « Une recommandation du dossier n'est jamais une décision tant qu'elle n'a pas d'entrée DP-XX. »
  - CONTEXTE.md, Exclusions : « Paiement en ligne par le locataire (F26) » et « WhatsApp Business API (F27) … Lien WhatsApp pré-rempli, sans coût ».
  - MAQUETTE.md : « WhatsApp simulé », « Ouvrir WhatsApp » (lien pré-rempli) dans tous les parcours. CLAUDE.md : « Pas d'IA ni de fonction hors du périmètre de `CONTEXTE.md` ».
- **Constat** : DECISIONS affirme que F26 est hors MVP et que la décision n° 8 (dont l'option « MVP ») reste ouverte : contradiction interne. CONTEXTE, UI et MAQUETTE construisent sur les recommandations n° 8 et n° 9 comme si elles étaient prises. Même cas de n° 10 : voir C-036.
- **Impact** : si Mimis choisissait l'API WhatsApp (n° 9) ou le paiement en ligne (n° 8), le périmètre, les parcours 1 et 2 et le prototype PDF changeraient.
- **Recommandation** : DECISIONS.md : fermer n° 8 (« P2 ») et n° 9 (« lien pré-rempli ») par DP-30 et DP-31, telles que recommandées par le dossier et déjà appliquées partout. Corriger la phrase de DP-08 en conséquence.
- **Alternative** : garder n° 8 et n° 9 ouvertes et ajouter « (hypothèse de travail) » dans CONTEXTE, UI et MAQUETTE.
- **Question pour Mimis** : confirmez-vous P2 pour le paiement en ligne et le lien pré-rempli pour les relances ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-017 — La règle d'or n° 7 de CONTEXTE ignore la remise à zéro annuelle

- **Gravité** : IMPORTANT
- **Type** : Contradiction
- **Où** : CONTEXTE.md, Règles d'or n° 7 : « Quittances numérotées sans trou. Numérotation continue par organisation. » DECISIONS.md, DP-15 : « la séquence repart à 1 chaque année, sans trou dans l'année ». DOMAINE.md, R-050. CLAUDE.md : « Quittances numérotées sans trou, par organisation et par année (R-050). »
- **Constat** : CONTEXTE n'a pas été mis à jour après DP-15. Texte fautif : CONTEXTE.md (DECISIONS prime).
- **Impact** : une lecture de CONTEXTE seul fait coder une séquence qui ne repart jamais à 1.
- **Recommandation** : CONTEXTE.md règle 7 : « Numérotation continue et sans trou, par organisation et par année (DP-15, R-050). »
- **Alternative** : aucune.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-018 — DP-28 : le « prix viable » se contredit et s'écarte de H5

- **Gravité** : IMPORTANT
- **Type** : Contradiction
- **Où** :
  - DECISIONS.md, DP-28 : « un prix mensuel … au moins égal au bas de la grille du plan : 5 000 FCFA pour le plan Propriétaire, 25 000 FCFA pour le plan Agence … Un prix cité sous 10 000 FCFA déclenche aussi la revue de cible prévue par le dossier (§20.1). »
  - Dossier §22.2 : « Propriétaire Essentiel | … | 5 000 – 10 000 ». Dossier §20.1 : « Prix cité inférieur à 10 000 FCFA par mois | Revoir la cible (propriétaires) ou le plan de valeur ». Dossier annexe H5 : « Le prix de 20–50 000 FCFA/mois est accepté pour les agences ; 5–20 000 pour les propriétaires ».
- **Constat** : (a) un propriétaire qui cite 7 000 FCFA est « viable » (≥ 5 000) et déclenche en même temps la revue de cible (< 10 000). Tout le palier Essentiel est dans cette zone. (b) Le plancher Agence de 25 000 exclut 20 000–24 999, que H5 tient pour acceptable.
- **Impact** : le seuil n° 4 produit des résultats ambigus pour tout propriétaire du palier d'entrée ; une agence à 20 000 serait écartée alors que H5 la valide.
- **Recommandation** : DECISIONS.md : remplacer DP-28 par une entrée qui (1) garde les planchers de la grille §22.2 et (2) précise que le déclencheur « sous 10 000 FCFA » est un signal consigné qui n'annule pas le seuil n° 4. Mettre CONTEXTE à jour (voir C-034).
- **Alternative** : abaisser le plancher Agence à 20 000 FCFA (bas de H5) et relever le plancher Propriétaire à 10 000.
- **Question pour Mimis** : quel plancher considérez-vous comme viable pour chaque plan ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-019 — La checklist de tournage oublie la décision n° 13 (nom et marque)

- **Gravité** : IMPORTANT
- **Type** : Manque
- **Où** : Dossier §26, n° 13 : « Vérifier la disponibilité du nom, du domaine et d'éventuelles marques existantes avant la vidéo de démonstration ». DECISIONS.md : « | 13 | Nom et marque | … | Avant publication | Ouverte | ». MAQUETTE.md, checklist : dernier item « Palette et vouvoiement : décisions n° 16 et n° 17 vérifiées … avant la diffusion publique ». Les vidéos portent « Immo Suite ».
- **Constat** : la décision n° 13 conditionne les vidéos selon le dossier ; MAQUETTE ne la mentionne pas.
- **Impact** : des vidéos publiques peuvent sortir sous un nom non vérifié.
- **Recommandation** : MAQUETTE.md, checklist : ajouter « Nom, domaine et marque vérifiés (décision n° 13) ou mention « nom provisoire » sur chaque vidéo ».
- **Alternative** : diffusion des vidéos uniquement en entretien, sans publication (la décision reste « avant publication »).
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-020 — La maquette ne sert pas explicitement à tester H1–H12

- **Gravité** : IMPORTANT
- **Type** : Manque
- **Où** : MAQUETTE.md : aucune occurrence de H1…H12 ni de « hypothèse » ; la seule mention de l'échelle est dans l'objectif (« 15 entretiens, échelle d'engagement, seuils Go/No-Go »), sans lien avec les parcours. Dossier §20.1 : échelle 0 à 4. CONTEXTE.md : H3 (reddition pénible), H4 (saisie au comptoir), H7 (commercial), H9 (relances WhatsApp assistées) sont les hypothèses que les parcours visent. DECISIONS.md, DP-10 : « La maquette est montrée dans les entretiens suivants. »
- **Constat** : les scénarios vidéo ont une « phrase-clé de valeur » mais aucune question d'entretien ni niveau d'engagement associé. H5 (prix) ne peut pas être testée par des vidéos sans prix ; H10, H11 et H12 ne le peuvent pas non plus.
- **Impact** : la maquette sera montrée sans protocole de mesure, ce qui ramène aux « compliments » que le dossier veut éviter.
- **Recommandation** : MAQUETTE.md : ajouter une section « Usage en entretien » avec un tableau parcours → hypothèse (P1 → H4 ; P2 → H9 ; P3 → H3, H8 ; P4 → coût de changement ; P5 → valeur de l'historique ; commercial → H7) → question à poser → niveau d'engagement visé (2 à 4) → hypothèses non testables par la maquette (H5, H6, H10 à H12).
- **Alternative** : mettre ce tableau dans le futur guide d'entretien (C-021) et renvoyer depuis MAQUETTE.md.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-021 — Guide d'entretien et modèle de lettre d'intention : aucun document ne les prévoit

- **Gravité** : IMPORTANT
- **Type** : Manque
- **Où** : Dossier §26 : « préparer le guide d'entretien de la section 20 ». Dossier §17 phase 0, livrables : « Grille d'engagement remplie », « Avis juridique succinct ». DECISIONS.md : une seule mention (« préparer le guide d'entretien »). CONTEXTE.md (index) et CLAUDE.md (tableau) : ni guide, ni modèle de lettre d'intention. DP-28 exige « au moins 3 signent une lettre d'intention de pilote ».
- **Constat** : le seuil n° 3 de DP-28 repose sur un document (la lettre d'intention) que personne n'a prévu de rédiger, et DP-10 fait démarrer les entretiens en parallèle.
- **Impact** : le premier entretien aura lieu sans guide, sans grille d'engagement et sans modèle de lettre d'intention.
- **Recommandation** : CLAUDE.md et CONTEXTE.md (index) : ajouter `contenus/GUIDE_ENTRETIEN.md` (à créer) avec la grille d'engagement, la check-list d'observation (dossier §20.1) et le modèle de lettre d'intention (`contenus/LETTRE_INTENTION.md`).
- **Alternative** : un seul fichier `contenus/ENQUETE.md`.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-022 — « Maquette complète » : 24 écrans au catalogue contre les 12 de DP-09

- **Gravité** : IMPORTANT
- **Type** : Incohérence
- **Où** :
  - DECISIONS.md, DP-09 : « la maquette couvre tous les écrans de la section 14.2 du dossier » ; Questions ouvertes : « Interprété comme les 12 écrans de la section 14.2 du dossier. À confirmer : les cinq rôles, l'écran Paramètres et l'écran Import sont-ils inclus ? »
  - UI_UX_DESIGN.md : « Écrans au-delà des 12 du dossier §14.2 : S01, S02, S13, S19, S20, S21 … et S22 à S24 ». MAQUETTE.md : Niveau 2 = 13 écrans, dont ces ajouts.
  - Dossier §24 : risque « Maquette trop ambitieuse qui retarde l'enquête » ; §17 : phase 0 de 4 à 6 semaines, capacité « environ 25 heures par semaine » (H11).
- **Constat** : le catalogue compte 24 écrans, soit 12 de plus que le dossier. Neuf sont nouveaux (S01, S02, S13, S19, S20, S21, S22, S23, S24) et trois sont des sous-écrans de lignes du dossier (S07 fiche lot, S09 fiche locataire, S18 utilisateurs dans Paramètres) que UI_UX_DESIGN.md ne compte pas dans « au-delà des 12 ». Aucune entrée de décision ne couvre ces ajouts. De plus la question posée dans DECISIONS est mal formulée : Paramètres et Import font partie des 12 écrans du dossier (§14.2), donc la question n'a pas d'objet.
- **Impact** : le périmètre de la maquette dépasse DP-09 et fragilise le calendrier de la phase 0.
- **Recommandation** : DECISIONS.md : nouvelle entrée qui fixe la liste : les 12 écrans du dossier, plus S01 et S02 (onboarding du parcours 4) et S13 (nécessaire à S12). Reporter S19 à S21 et S22 à S24 après les premiers entretiens. Corriger la question ouverte de DP-09.
- **Alternative** : valider les 24 écrans et accepter l'allongement de la phase 0.
- **Question pour Mimis** : combien d'heures pouvez-vous consacrer à la maquette avant le premier entretien ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-023 — L'index de CONTEXTE est périmé et double celui de CLAUDE

- **Gravité** : IMPORTANT
- **Type** : Doublon
- **Où** : CONTEXTE.md, « Index des documents » : « | `UI_UX_DESIGN` | … | Prévu | » et « | `MAQUETTE` | … | Prévu | » ; aucune ligne pour CLAUDE.md. CLAUDE.md : « | `UI_UX_DESIGN.md`, `MAQUETTE.md` | Créés | ».
- **Constat** : deux index des documents, déjà divergents. CLAUDE.md est exact, CONTEXTE.md est périmé.
- **Impact** : une source dit « prévu », l'autre « créé ». Le risque grandira avec les documents du bloc 2.
- **Recommandation** : source unique = le tableau « Quel fichier lire et quand » de CLAUDE.md. Remplacer l'index de CONTEXTE par un renvoi à CLAUDE.md, ou le mettre à jour (statuts « Créé », ajout de CLAUDE.md).
- **Alternative** : conserver les deux et ajouter un contrôle manuel à chaque création de document.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-024 — « Aucun code avant le Go » et maquette en code

- **Gravité** : À SURVEILLER
- **Type** : Ambiguïté
- **Où** : DECISIONS.md, DP-07 : « une maquette interactive complète est produite avant tout code » ; DP-11 : « Seule exception à « aucun code » avant le Go. » MAQUETTE.md : « HTML, CSS et JavaScript statiques dans `maquette/` » et `donnees/verifier.js`.
- **Constat** : la maquette est elle-même du code. DP-11 prétend être la seule exception à « aucun code ». CLAUDE.md corrige en « Aucun code de l'application ».
- **Impact** : un lecteur strict de DECISIONS refuserait la maquette ou le script de vérification.
- **Recommandation** : DECISIONS.md : préciser « aucun code applicatif (hors maquette et prototype PDF jetable) » dans DP-07 et DP-11, par une nouvelle entrée DP qui précise les deux.
- **Alternative** : laisser, la précision de CLAUDE.md suffit.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-025 — Champs commerciaux : portés par le lot (R-012) ou par le bail (R-025) ?

- **Gravité** : À SURVEILLER
- **Type** : Doublon
- **Où** : DOMAINE.md, R-012 : « un lot « boutique » propose « activité autorisée » et « indexation » » ; R-025 : « Le bail commercial porte des champs en plus : activité, indexation, durée ». Dossier F04 (critère) et F07 (règle métier) : même ambiguïté.
- **Constat** : « activité » et « indexation » sont attribuées à deux entités.
- **Impact** : le modèle de données (bloc 2) dupliquera ou divergera ; l'écran S06 et l'écran S10 afficheront les mêmes champs deux fois.
- **Recommandation** : DOMAINE.md : lot = « activité autorisée » (nature des locaux) ; bail = « indexation » et « durée » (conditions). Mettre R-012 et R-025 en accord.
- **Alternative** : tous les champs sur le bail ; le lot ne porte que son type.
- **Question pour Mimis** : l'activité autorisée est-elle une propriété du local ou du contrat ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-026 — « Comptable » : rôle interne, comptable externe, ou profil « caissier / comptable » ?

- **Gravité** : À SURVEILLER
- **Type** : Ambiguïté
- **Où** : CONTEXTE.md : « | Caissier / comptable | Encaisser vite … | » ; « Autres utilisateurs indirects : … comptable externe (exports) » ; matrice : colonne « Comptable (lecture) ». MAQUETTE.md : « Rasmata Belem | Comptable en lecture » (utilisateur avec compte). DOMAINE.md : le terme « comptable » est absent du glossaire (0 occurrence).
- **Constat** : trois sens pour « comptable » : un profil qui encaisse, un tiers qui reçoit des exports, un rôle avec compte en lecture.
- **Impact** : le modèle de permissions et la maquette (compte « Rasmata Belem ») supposent un compte, ce que le dossier ne confirme pas.
- **Recommandation** : DOMAINE.md : ajouter « Comptable (lecture) : rôle avec compte, lecture seule, qui peut être un tiers de l'agence ». Mettre à jour CONTEXTE (persona « caissier / comptable » renommée « Caissier »).
- **Alternative** : le comptable n'a pas de compte et reçoit seulement des exports (supprimer la colonne de la matrice et l'utilisateur de la maquette).
- **Question pour Mimis** : le comptable a-t-il un compte ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-027 — Accès support : le client n'a aucun écran pour l'accorder ou le révoquer

- **Gravité** : À SURVEILLER
- **Type** : Manque
- **Où** : Dossier §12.1 : « Accès support HABEK : compte nominatif, consentement du client, entrée d'audit à chaque consultation. » DOMAINE.md, R-009. UI_UX_DESIGN.md, parcours gérant, étape 10 : « Déconnexion. Accès support révocable » ; « Hors catalogue : la console HABEK (F40) est un outil interne ».
- **Constat** : le consentement du client est exigé, mais aucun écran du catalogue ne permet de l'accorder ou de le révoquer.
- **Impact** : une exigence de sécurité P0 (F40, T2) n'est visible ni dans la maquette ni dans le catalogue.
- **Recommandation** : UI_UX_DESIGN.md S17 Paramètres : ajouter un bloc « Accès support » (autoriser pour une durée, révoquer, historique).
- **Alternative** : reporter au bloc 2 et le noter dans les Questions ouvertes de UI_UX_DESIGN.md.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-028 — « Visite guidée » : deux choses sous le même nom

- **Gravité** : À SURVEILLER
- **Type** : Ambiguïté
- **Où** : Dossier §14.1 : « Visite guidée de 3 écrans » (onboarding du caissier). UI_UX_DESIGN.md : mêmes mots dans les parcours, sans spécification. MAQUETTE.md : « Mode « Visite guidée » pour les vidéos ».
- **Constat** : la visite guidée du produit (onboarding) n'est spécifiée nulle part. La maquette emploie le même terme pour un outil de tournage.
- **Impact** : confusion à la lecture ; la visite guidée du produit sera oubliée.
- **Recommandation** : MAQUETTE.md : renommer en « Mode démonstration guidée ». UI_UX_DESIGN.md : ajouter un composant « Visite guidée du produit (3 écrans) » au catalogue des composants, rattaché à S02 et aux parcours.
- **Alternative** : conserver les deux et noter la différence en Questions ouvertes.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-029 — Contraste : l'indicateur d'onglet actif ne tient pas la règle des 3:1

- **Gravité** : À SURVEILLER
- **Type** : Incohérence
- **Où** : UI_UX_DESIGN.md, Onglets : « actif (soulignement `--color-accent-500` + texte gras) » ; Accessibilité : « contrastes de composants ≥ 3:1 » ; tableau de contraste : « `--color-text-on-accent` sur `--color-accent-500` | 7,1:1 ».
- **Constat** : recalcul par script : accent `#DCA53E` sur blanc = 2,21:1 ; sur `--color-bg` = 2,09:1 (< 3:1). Le texte gras est un second indice, mais le soulignement seul échoue. Le contraste texte sur accent est 6,93:1, pas 7,1:1 (AA tenu). Les 15 autres valeurs annoncées (14 lignes du tableau et les deux valeurs « interdits », moins celle-ci) sont exactes à ±0,05.
- **Impact** : écart d'accessibilité sur un composant central de S07 et S09 ; valeur annoncée inexacte.
- **Recommandation** : UI_UX_DESIGN.md : soulignement de l'onglet actif en `--color-brand-600` (4,73:1 sur `--color-bg`, 5,01:1 sur blanc), l'accent restant décoratif. Corriger « 7,1:1 » en « 6,9:1 ».
- **Alternative** : conserver l'accent et ajouter une icône ou un fond teinté `--color-brand-100` à l'onglet actif.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-030 — Badges « neutres » : règle contredite par ses propres exceptions, tokens manquants

- **Gravité** : À SURVEILLER
- **Type** : Contradiction
- **Où** : UI_UX_DESIGN.md : « Autres statuts, en style neutre (gris, aucune couleur sémantique) pour ne pas être confondus avec l'argent » ; lignes suivantes : « En préavis (famille « Due »), Terminé (famille « Annulé ») » et « Payé (famille « Payé ») ». Le bloc `:root` ne définit aucun token pour un badge neutre.
- **Constat** : la règle interdit de réemployer les couleurs de l'argent, puis le fait pour le bail et le relevé. Aucun fond/texte neutre n'est défini.
- **Impact** : l'implémenteur de la maquette choisira les couleurs ; le résultat pourrait introduire des contrastes non vérifiés.
- **Recommandation** : UI_UX_DESIGN.md : ajouter `--neutral-badge-bg: var(--color-surface-sunken)` et `--neutral-badge-fg: var(--color-text-secondary)` ; réécrire la règle : « les couleurs de statut financier sont réservées à l'échéance, au paiement et au relevé « Payé » ».
- **Alternative** : supprimer les exceptions et tout rendre neutre sauf l'argent.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-031 — Le jeu de données ne couvre pas plusieurs règles de DOMAINE

- **Gravité** : À SURVEILLER
- **Type** : Manque
- **Où** : MAQUETTE.md, spécification des données et 24 contrôles. DOMAINE.md : R-023 (« Un changement de loyer ou de charges est un avenant daté. Il n'est jamais rétroactif »), E-02, R-024 (dépôt > 2 mois), R-021, R-022, R-017.
- **Constat** : aucun avenant n'existe dans les données, et aucun contrôle ne vérifie R-023, R-021, R-022 ni R-024 ; E-02 n'est pas reproduit. Le brief demandait de ne violer ni « avenants non rétroactifs » ni « un seul bail actif par lot ».
- **Impact** : le vérificateur ne détecterait pas un avenant rétroactif écrit à la main dans les données.
- **Recommandation** : MAQUETTE.md : ajouter deux baux avec un avenant daté du 01/11/2026 (échéances de septembre et octobre inchangées) et les contrôles correspondants (R-021, R-022, R-023), ainsi qu'un bail d'habitation avec dépôt supérieur à deux mois pour afficher l'avertissement R-024.
- **Alternative** : accepter une couverture partielle et la signaler dans la checklist.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-032 — Détails inventés sans source

- **Gravité** : À SURVEILLER
- **Type** : Invention non sourcée
- **Où** : UI_UX_DESIGN.md : « Préfixe + année + séquence sur 6 chiffres (R-050) » ; DOMAINE.md, E-27 : « numéroté 000812 ». MAQUETTE.md : « HOR-2026-000094 » écrit en dur alors que le texte dit « Le numéro de quittance réel est généré par le jeu de données » ; « Environ 100 paiements » (estimation de l'auteur : 88 à 92). UI : « filigrane « ANNULÉE » », « 25 lignes », « 8 résultats ».
- **Constat** : DP-15 et R-050 ne fixent pas « 6 chiffres ». Plusieurs valeurs chiffrées n'ont pas de source dans le dossier (certaines sont signalées « proposition », d'autres non).
- **Impact** : un détail inventé devient une règle de fait dans la maquette, puis dans le produit.
- **Recommandation** : DECISIONS.md : une entrée « Format du numéro de quittance » (longueur de la séquence, préfixe) ; MAQUETTE.md : remplacer « HOR-2026-000094 » par le placeholder `{numéro suivant}`.
- **Alternative** : marquer ces valeurs « [proposition] » dans UI et MAQUETTE.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-033 — Parties du dossier non reprises dans aucun document dérivé

- **Gravité** : À SURVEILLER
- **Type** : Manque
- **Où** : dossier §23 (KPIs : activation < 48 h, adoption ≥ 60 % puis 85 %, rétention > 85 % à 6 mois), §24 (20 risques), §19 (plan de tests), §20.2 (déploiement), §21 (scalabilité), §22 (paiement annuel avec « deux mois offerts »). Recherche dans les six `.md` : « KPI » = 0 occurrence, « rétention » = 0.
- **Constat** : aucune de ces sections n'a de destination connue, sauf TESTING_QA et DEPLOYMENT dans la liste de CLAUDE.md.
- **Impact** : sans effet sur la maquette. À traiter au bloc 2 pour ne pas perdre les KPIs et les risques.
- **Recommandation** : CLAUDE.md : ajouter une colonne « Sources du dossier » dans le tableau des fichiers à créer (§19 → TESTING_QA, §20.2 → DEPLOYMENT, §23 et §24 → ROADMAP, §22 → CONTEXTE).
- **Alternative** : créer un fichier `contenus/KPIS_RISQUES.md`.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-034 — Contenus dupliqués qui vont diverger

- **Gravité** : À SURVEILLER
- **Type** : Doublon
- **Où** :
  - Seuils Go/No-Go : CONTEXTE.md (« Critères Go/No-Go de la phase 0 ») et DECISIONS.md DP-28.
  - Règles d'or : CONTEXTE.md (12 règles), CLAUDE.md (8 règles ; il n'a pas les règles 1, 2, 5, 11 et 12 de CONTEXTE : livre de comptes, quittance comme accroche, Bien/Lot, humain dans la boucle, règles d'argent jamais sur mesure, et ajoute « Pas d'IA ni de fonction hors du périmètre de `CONTEXTE.md` »), DOMAINE.md (R-001, R-047, R-050).
  - Matrice de permissions : CONTEXTE.md et tableau « Entrées de menu par rôle » d'UI_UX_DESIGN.md.
  - Statuts financiers : DOMAINE.md (cycles de vie) et UI_UX_DESIGN.md (tableau des statuts).
  - Index des documents : voir C-023.
- **Constat** : le même contenu vit à deux endroits. C-017 en est déjà une conséquence.
- **Impact** : chaque décision future devra être recopiée dans plusieurs fichiers.
- **Recommandation** : sources uniques : seuils = DP-28 (CONTEXTE renvoie) ; règles d'or = CONTEXTE (CLAUDE cite les numéros) ; permissions = CONTEXTE (UI en dérive et le dit) ; statuts = DOMAINE (UI cite les identifiants R-xxx).
- **Alternative** : accepter les doublons et ajouter une ligne « à tenir en phase avec » dans chaque fichier.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-035 — CLAUDE.md : points de détail

- **Gravité** : À SURVEILLER
- **Type** : Manque
- **Où** : CLAUDE.md (83 lignes, < 100 : conforme). Tableau « Quel fichier lire et quand » : le dossier `Immo-Suite_Dossier-de-cadrage.docx` n'y figure pas (il apparaît seulement dans « Ne jamais modifier »). « **Bloc 1 : maquette interactive** » : le mot « bloc » n'est défini nulle part (le dossier parle de « phases »). « Identifiants de code … anglais. **Convention proposée, à confirmer dans DECISIONS.md.** » : DECISIONS.md ne l'a que comme puce de Questions ouvertes, pas comme décision ouverte du tableau. « Commits : … avec l'identifiant DP-xx ou F-xx » : non sourcé.
- **Constat** : quatre petits manques qui affaiblissent le fichier chargé à chaque session.
- **Impact** : Claude Code ne sait pas où est la hiérarchie de vérité ; la convention de langue n'est pas suivie dans les décisions.
- **Recommandation** : CLAUDE.md : (1) ajouter une ligne pour le dossier de cadrage et la hiérarchie « DECISIONS > dossier > DOMAINE > CONTEXTE > UI_UX_DESIGN > MAQUETTE > CLAUDE » ; (2) « Bloc 1 (= phase 0, dossier §17) » ; (3) DECISIONS.md : ajouter la décision ouverte n° 18 « Langue des identifiants de code ».
- **Alternative** : supprimer la règle de commit tant qu'elle n'est pas décidée.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-036 — La décision n° 10 (date d'encaissement) conditionne DP-16, R-071 et le parcours 3

- **Gravité** : À SURVEILLER
- **Type** : Ambiguïté
- **Où** : DECISIONS.md, DP-16 : « Dépend de R-071 (date d'encaissement), qui reste [À valider] (décision ouverte n° 10). Si R-071 change, cette décision est à revoir. » DOMAINE.md, R-071 : « [règle à confirmer sur le terrain] ». MAQUETTE.md : le relevé d'octobre (E-18) suppose cette règle ; ses Questions ouvertes ne la mentionnent pas.
- **Constat** : le parcours 3, au cœur de la proposition de valeur, repose sur une règle encore ouverte. Les données actuelles donnent le même résultat pour les deux options (tous les loyers d'octobre sont payés en octobre), mais ce n'est pas écrit.
- **Impact** : une vidéo qui affirme « encaissé en octobre » peut contredire la pratique réelle des agences.
- **Recommandation** : MAQUETTE.md, Questions ouvertes : ajouter « Parcours 3 : repose sur la date d'encaissement (décision n° 10, H8) ». Garder tous les loyers du relevé d'octobre encaissés en octobre.
- **Alternative** : demander à Mimis de fermer n° 10 avant le tournage.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-037 — DECISIONS.md : phrases périmées après DP-28

- **Gravité** : MINEUR
- **Type** : Incohérence
- **Où** : DECISIONS.md, DP-07 : « Les seuils Go/No-Go doivent être fixés avant le premier entretien (décision ouverte n° 1). » ; introduction du tableau : « Les n° 1 (DP-28) et n° 4 (DP-09) sont prises. Les 13 autres sont **Ouvertes**. » (le tableau compte 13 + n° 16 et 17 = 15 ouvertes) ; « Liens avec les décisions prises : … n° 1 ↔ DP-07, DP-10 » (DP-28 absent).
- **Recommandation** : DECISIONS.md : « (fixés par DP-28) » dans DP-07 ; « Les 15 autres sont Ouvertes (13 du dossier, plus n° 16 et n° 17) » ; ajouter DP-28 aux liens.
- **Alternative** : aucune.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-038 — Glossaire de DOMAINE.md non aligné avec ses propres règles

- **Gravité** : MINEUR
- **Type** : Incohérence
- **Où** : DOMAINE.md, glossaire : « Paiement partiel | Paiement inférieur au dû. L'échéance devient « partielle » » ; R-045 : « Avant J+1, l'échéance passe à « partielle » … Après J+1, elle reste « en retard » ». Glossaire : « Relevé … Figé une fois validé » ; R-083 : réouverture possible avant reversement.
- **Recommandation** : DOMAINE.md : « … l'échéance devient « partielle » avant J+1, « en retard » ensuite (DP-13) » et « Figé une fois validé, sauf réouverture avant reversement (R-083) ».
- **Alternative** : aucune.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-039 — H1–H12 reformulées dans CONTEXTE.md

- **Gravité** : MINEUR
- **Type** : Incohérence
- **Où** : comparaison par script entre l'annexe du dossier et CONTEXTE.md : 6 hypothèses sur 12 diffèrent par le texte. H1 : « (section 20) » supprimé. H3 : « de comptes » supprimé. H5 : « Le prix de 20–50 000 … est accepté » devenu « Prix de 20–50 000 … accepté ». H6 : « via réseaux et bouche-à-oreille » supprimé. H7 : « (boutiques, entrepôts) » supprimé, « nécessite » devenu « exige ». H12 : « les garanties contractuelles » devenu « garanties contractuelles ».
- **Constat** : le sens est conservé ; les identifiants et 6 hypothèses identiques (H2, H4, H8, H9, H10, H11).
- **Recommandation** : CONTEXTE.md : reprendre les textes à l'identique (le tableau est une citation du dossier).
- **Alternative** : ajouter « (reformulé) » dans l'en-tête du tableau.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-040 — Marqueurs [À valider] de CONTEXTE absents de ses Questions ouvertes

- **Gravité** : MINEUR
- **Type** : Manque
- **Où** : CONTEXTE.md : « [À valider] » sur la persona A (ligne 156), sur les frais d'agrégateur de 2 à 3,5 % (ligne 184) et sur quatre points de cadre légal (lignes 189–192). Questions ouvertes de CONTEXTE.md : seul le plafond de lots (ligne 257) est repris. DOMAINE.md « Règles à confirmer sur le terrain » reprend le légal mais pas les frais ni la persona.
- **Recommandation** : CONTEXTE.md, Questions ouvertes : ajouter une puce « [À valider] du cadre légal : voir décision n° 12 et DOMAINE.md », et deux puces pour les frais d'agrégateur et la persona A.
- **Alternative** : aucune.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-041 — Mise en forme : lignes collées et identifiants hors ordre

- **Gravité** : MINEUR
- **Type** : Incohérence
- **Où** : MAQUETTE.md ligne 561 : « … à confirmer avec DP-09.- **Numéros de téléphone fictifs.** … » (deux puces sur une ligne). DOMAINE.md : les règles R-020, R-027, R-037, R-049, R-079 rompent l'ordre croissant parce que R-083 à R-095 ont été insérées dans des tableaux thématiques ; l'exemple E-30 vient après E-37 ; E-13b existe en plus de E-13.
- **Recommandation** : MAQUETTE.md : séparer les deux puces. DOMAINE.md : conserver les identifiants (ils sont cités ailleurs) et ajouter un index « Règles par identifiant » en fin de section, ou trier les lignes de chaque tableau par identifiant.
- **Alternative** : renuméroter tout (déconseillé : les identifiants sont cités dans les six fichiers).
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-042 — Parcours 3 et 4 : écrans annoncés mais jamais visités

- **Gravité** : MINEUR
- **Type** : Incohérence
- **Où** : MAQUETTE.md : « | 3 | Produire un compte rendu propriétaire | Gérant | S03, S12, S13 | 75 s | » et « | 4 | Importer un fichier Excel | Gestionnaire | S16, S02 | 60 s | ». Les étapes du parcours 3 ne passent ni par S13 ni par une sélection de mandant dans S13 ; celles du parcours 4 ne passent pas par S02.
- **Recommandation** : MAQUETTE.md : retirer S13 du parcours 3 et S02 du parcours 4, ou ajouter l'étape (consulter le mandat M1 depuis S12).
- **Alternative** : aucune.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-043 — Identifiant F38 inexistant dans le dossier

- **Gravité** : MINEUR
- **Type** : Manque
- **Où** : dossier §7 : les tableaux définissent F01 à F37, F39 et F40 (39 identifiants, vérifié par script) ; F38 n'existe pas. CONTEXTE.md : « P3 = F29, F33–F37 » et « P0 = F01–F14, F39, F40 ».
- **Recommandation** : CONTEXTE.md, Questions ouvertes : « F38 n'est pas défini dans le dossier (numérotation F01–F40 avec un trou) ».
- **Alternative** : demander à Mimis si F38 a été supprimée ou oubliée.
- **Question pour Mimis** : que devait être F38 ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

### C-044 — Vocabulaire : synonymes à unifier

- **Gravité** : MINEUR
- **Type** : Ambiguïté
- **Où** :
  - « compte rendu » / « relevé » : UI_UX_DESIGN.md S12 : « Objectif : produire, valider et partager le relevé mensuel d'un mandant » (titre de l'écran : « Comptes rendus propriétaires »). MAQUETTE.md parcours 3 : « S12 : liste des relevés ».
  - « Journal de caisse du jour » (UI, parcours caissier) / « Journée de caisse » (DOMAINE) / « journal des paiements » (S11).
  - « Partiel » (badge UI) / « partielle » (DOMAINE R-032).
  - « caution » / « dépôt de garantie » (UI tranche : libellé « Dépôt de garantie »).
- **Recommandation** : UI_UX_DESIGN.md, tableau « Vocabulaire » : ajouter « Compte rendu (écran, texte) = Relevé (donnée) », « Journal de caisse = Journée de caisse (S19) » et « Partiel = échéance partielle ». MAQUETTE.md : « compte rendu » partout dans les textes affichés.
- **Alternative** : aucune.
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________

## C. Plan de correction

Ordre d'application des corrections validées. Règle générale : corriger le fichier de rang supérieur avant ceux qui en dépendent (DECISIONS, puis DOMAINE, CONTEXTE, UI_UX_DESIGN, MAQUETTE, CLAUDE). Les réponses de Mimis aux questions ci-dessus sont un préalable.

| Étape | Fichier | Constats à appliquer | Dépend de |
|---|---|---|---|
| 0 | (décisions de Mimis) | Réponses aux questions de C-001, C-002, C-003, C-004, C-009, C-012, C-016, C-018, C-022, C-025, C-026, C-043 | — |
| 1 | `DECISIONS.md` | Nouvelles entrées : tranches de F19, F23, F25 et définition de V1 (C-001, C-003) ; F12 et agenda (C-004) ; périmètre de la maquette (C-022) ; fermeture ou non des n° 8 et n° 9 (C-016) ; prix viable (C-018) ; format du numéro de quittance (C-032) ; « code » (C-024) ; langue des identifiants (C-035) ; phrases périmées (C-037) | Étape 0 |
| 2 | `DOMAINE.md` | C-009 (contre-passation et quittance), C-008 (bail en cours), C-012 (définition de Relance), C-025 (champs commerciaux), C-026 (comptable), C-038 (glossaire), C-041 (index des règles), E-15 (C-004) | Étape 1 |
| 3 | `CONTEXTE.md` | C-017 (règle 7), C-023 (index), C-018 et C-034 (renvoi à DP-28), C-039 (H1–H12), C-040 (questions ouvertes), C-043 (F38), P1 sans tranche (C-001) | Étapes 1 et 2 |
| 4 | `UI_UX_DESIGN.md` | C-010, C-011 (S25, S26), C-012 (bloc J−3 dans S03), C-013, C-014, C-015, C-027, C-028, C-029, C-030, C-044, tranche dans chaque ligne « Priorité » (C-003), suppression de F26 et F27 dans S05 et S11 (C-002) | Étapes 1 à 3 |
| 5 | `MAQUETTE.md` | C-001 et C-002 (règle des repères, légende), C-003, C-005, C-006, C-007, C-008, C-019, C-020, C-022, C-028, C-031, C-032, C-036, C-041 (ligne 561), C-042, C-015 (libellé du sélecteur) | Étapes 1 à 4 |
| 6 | `CLAUDE.md` | C-021 (guide d'entretien, lettre d'intention), C-033 (colonne des sources), C-034, C-035 | Étapes 1 à 5 |
| 7 | Nouveaux documents | `contenus/GUIDE_ENTRETIEN.md` et `contenus/LETTRE_INTENTION.md` (C-021) avant le premier entretien | Étapes 1 et 5 |
| 8 | Bloc 2 | C-033 (KPIs, risques, tests, déploiement), C-024 | Après la maquette |

Dépendances clés :

- C-001 doit être tranché avant C-003, qui doit être tranché avant la règle des repères de MAQUETTE.md.
- C-022 (liste des écrans) précède C-011, C-013, C-027, car chacun ajoute ou retire des écrans.
- C-009 et C-008 précèdent C-006, C-007 et C-031 (le jeu de données ne peut être écrit qu'avec des règles stables).
- C-016 précède le tournage des vidéos.

## D. Ce qui est cohérent

- Les 15 décisions de la section 26 sont reprises dans DECISIONS.md avec les mêmes intitulés ; seules les ponctuations diffèrent (vérifié par script). Les décisions n° 16 et n° 17 (palette, vouvoiement) sont présentes.
- Tranches T1, T2, T3 et priorités P0 à P3 : les listes de CONTEXTE.md reproduisent celles du dossier (F01–F04, F06–F12, F39 ; F05, F13, F14, F40, F15 minimal ; F15, F16, F17, F18, F20, F22).
- Les priorités affichées dans le catalogue d'UI_UX_DESIGN.md correspondent aux priorités du dossier pour chaque F-xx cité.
- Les 12 écrans du dossier §14.2 existent tous dans le catalogue, et chaque écran S01–S24 porte au moins une fonctionnalité F-xx.
- Renvois : aucun identifiant R-xxx, E-xx, DP-xx, S-xx, F-xx ou H-xx cité n'est orphelin ; aucun doublon d'identifiant dans les 95 règles ni les 38 exemples.
- Exemples chiffrés recalculés : 400 000 × 8 % = 32 000 et net 338 000 ; 333 333 × 7,5 % arrondi à 25 000 ; 8 % de 110 000 = 8 800 ; 3 600 000 / 288 000 / ≈ 10,4 % ; 900 000 / ≈ 1,7 % ; six loyers de M1 = 400 000.
- Dates : le 13/11/2026 est un vendredi ; retards de 3, 20, 31, 60, 70, 30, 40 et 9 jours exacts ; fenêtre de 30 jours jusqu'au 13/12.
- Jeu de données : 7+12+11+10 = 40 lots ; 30 habitation, 9 commercial, 1 mixte ; 33 occupés, 5 vacants, 2 en travaux ; statuts de novembre 23+4+5+1 = 33, dans les tolérances annoncées.
- Montage vidéo : 285 secondes cumulées pour les cinq scénarios ; le film de 3 minutes totalise exactement 180 secondes.
- Contrastes : 15 des 16 valeurs annoncées sont exactes à ±0,05 après recalcul (voir C-029 pour l'exception).
- CLAUDE.md : 83 lignes ; statuts « Créé / À créer » exacts ; règles d'or compatibles avec DOMAINE et CONTEXTE (sauf C-017 pour la remise à zéro annuelle).
