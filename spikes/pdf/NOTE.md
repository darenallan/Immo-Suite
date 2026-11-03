# Prototype (a) — PDF de quittance et montant en lettres

| Champ | Valeur |
|---|---|
| Statut | **Exécuté le 31/10/2026 (protocole validé par Mimis), sauf l'impression papier (à faire par Mimis).** Résultats en section 8 ; décision proposée en section 6, **en attente de validation de Mimis** |
| Durée maximale | 3 jours (dépasser est un résultat à noter) |
| Décision attendue | Décision n° 7 (moteur PDF) ; n° 83 (tests visuels) ; taille de serveur ; question 7 d'`API_CONTRACTS.md` (imprimer depuis le PDF ou depuis l'aperçu HTML) |
| Documents | `ARCHITECTURE.md` §6, §7 ; `TESTING_QA.md` §6 ; `UI_UX_DESIGN.md` (quittance) ; `DOMAINE.md` R-053, E-11 ; DP-64, DP-70, DP-71 |

## 1. Objectif

Comparer trois moteurs sur les 20 cas types de la quittance, en A4 et en A5, et valider le montant en lettres de 0 à 999 999 999. Produire des mesures (vitesse, mémoire, poids) et une décision. Le PDF émis étant conservé et immuable (DP-71), le moteur doit aussi produire un fichier stable et sans métadonnée parasite.

## 2. Environnement

- PC Windows 11 (i7-10510U, 7,8 Go de RAM). Les valeurs absolues ne sont **pas** celles d'un serveur Linux de 2 Go : elles servent à **comparer** les moteurs ; le p95 absolu sera revérifié sur l'essai à l'étape 01. Docker étant absent, la taille d'image de déploiement est approchée par la taille des composants (binaire, navigateur, polices) et remesurée à l'étape 01.
- Moteurs : **E1** Chrome sans interface (Chrome déjà installé, version relevée), en deux modes : un processus par rendu, et un navigateur maintenu ouvert ; **E2** tracé natif Rust (bibliothèque PDF la plus courante choisie en début de prototype, par exemple `printpdf` ou `pdf-writer`, version figée, à vérifier) ; **E3** Typst (installé), qualité mesurée avec la ligne de commande et vitesse avec la bibliothèque Rust, qui évite le biais du démarrage de processus.
- Police : IBM Plex Sans (licence OFL [À valider]) embarquée dans les trois cas. Python 3.13, `pypdf`, `pymupdf` (venv `spikes/.venv`) pour les contrôles.
- Montant en lettres : fonction pure `amount_in_words` dans `spikes/pdf/words/` (crate Rust), jamais produite par un moteur (`ARCHITECTURE.md` §6).

## 3. Protocole

1. **`amount_in_words`** (R-053, DP-64 : orthographe traditionnelle, « francs CFA ») : tests unitaires sur **tous** les entiers de 0 à 2 000 ; sur les frontières de `TESTING_QA.md` §6 (20, 21, 70, 71, 80, 81, 90, 91, 99, 100, 101, 200, 201, 1 000, 1 100, 80 000, 100 000, 200 000, 1 000 000, 1 250 000, 2 000 000, 21 000 000, 999 999 999) ; et sur 999 999 999. Comparaison avec un analyseur de référence **indépendant** (`num2words`, langue française) et avec un tableau écrit à la main pour les frontières. Chaque divergence est listée avec un verdict (règle choisie). Les accords de « vingt » et « cent » (« quatre-vingts », « deux cents » devant « mille »), « mille » invariable, « et un » et les traits d'union sont des règles à **faire valider par Mimis** ; elles ne sont pas inventées par le prototype.
2. **Jeu de cas** : 20 instantanés JSON (`spikes/pdf/cases/`) : accents et cédilles (« Ouédraogo », « Yaméogo », « ç », « œ ») ; noms longs et composés (deux lignes au plus) ; société à raison sociale longue ; adresse longue ; montant à 7 chiffres ; montant maximal 999 999 999 ; paiement partiel ; avance ; tiers payeur ; colocation ; bail commercial « TVA incluse » ; rectificative ; annulée (filigrane) ; série d'essai (« ESSAI ») ; organisation de démonstration (« DÉMONSTRATION — sans valeur ») ; apostrophes et guillemets ; nom contenant du balisage ou une adresse (`<b>`, `https://…`) ; logo absent ; logo maximal (40 × 20 mm). Chacun en A4 et en A5, soit 40 documents par moteur.
3. **Gabarits** : un gabarit par moteur à partir du même contenu ; le CSS d'impression de la maquette est repris pour E1 ; le contenu et les mentions viennent de `UI_UX_DESIGN.md`.
4. **Contrôles automatiques** par document (`TESTING_QA.md` §6) : une seule page (`pypdf`) ; extraction du texte comparée à l'instantané (numéro, noms accentués, montant en chiffres et en lettres, période, mode, solde, mentions, « TVA incluse ») ; polices embarquées ; aucune métadonnée d'auteur, titre neutre ; aucune référence externe ; **aucune requête sortante** (E1 : résolveur DNS neutralisé et journal réseau du navigateur, résultat attendu 0 requête ; E2 et E3 : aucun code réseau) ; lisibilité en noir et blanc (rendu en niveaux de gris à 150 dpi, aucune zone pleine colorée, contraste mesuré).
5. **Reproductibilité et DP-71** : deux rendus du même instantané, dates de création fixées, donnent-ils le même SHA-256 ? Relever ce qui varie (dates, identifiants internes). Ce résultat ne prouve plus rien sur les documents émis (le fichier original fait foi, DP-71) mais conditionne le test de non-régression du moteur.
6. **Performance** : **100 rendus** par moteur (les 20 cas répétés 5 fois, A4) ; médiane et p95 de la durée ; mémoire au pic de l'arbre de processus (relevé toutes les 20 ms avec `psutil`) ; poids : taille du binaire ou du navigateur et des polices. Un échauffement non compté précède chaque série.
7. **Impression papier par Mimis** (imprimante laser noir et blanc, **pas** « Microsoft Print to PDF ») : A4 et A5 pour trois cas (accents, montant maximal, annulée), imprimés en « taille réelle » sans « ajuster ». Pour le même document : impression depuis le PDF, puis depuis l'aperçu HTML (question 7 d'`API_CONTRACTS.md`). Fiche de relevé : lisibilité des accents, des chiffres et du montant en lettres, marges et zone non imprimable de l'imprimante, une seule page, contraste, remarques.

## 4. Résultats chiffrés (à remplir)

| Mesure | E1 Chrome (processus par rendu) | E1 Chrome (maintenu ouvert) | E2 natif Rust | E3 Typst |
|---|---|---|---|---|
| Cas qui tiennent sur 1 page (sur 40) | | | | |
| Texte extrait égal à l'instantané (sur 40) | | | | |
| Polices embarquées / métadonnées / requêtes sortantes | | | | |
| Lisible en noir et blanc | | | | |
| Médiane / p95 de rendu (ms), 100 rendus | | | | |
| Mémoire au pic (Mo) | | | | |
| Poids du moteur (Mo) | | | | |
| Même SHA-256 sur deux rendus | | | | |
| Effort de mise en page (h) | | | | |

| Montant en lettres | Résultat |
|---|---|
| Entiers de 0 à 2 000 : concordance avec la référence | |
| Frontières et 999 999 999 | |
| Divergences listées, règles à valider par Mimis | |

## 5. Critères d'acceptation (étape 00 §6)

7. Les 20 cas tiennent sur une page en A4 et en A5 ; p95 inférieur à 1 s (valeur absolue à confirmer sur l'essai) ; extraction de texte égale à l'instantané ; impression papier lisible en noir et blanc ; moteur proposé pour la décision n° 7.

## 6. Décision proposée

Voir la section 8.5. Les trois moteurs tiennent les 20 cas : le repli « aperçu HTML imprimable » n'est pas nécessaire.

## 7. Écarts et risques déjà connus

- Pas d'imprimante réelle sur place : le critère papier dépend de Mimis.
- Pas de Docker : taille d'image seulement approchée.
- Les règles d'accord du français sont une décision métier (DP-64 fixe l'orthographe traditionnelle, pas chaque accord) : à faire valider avant de figer la fonction.
- Implémenter Typst en bibliothèque demande de fournir un environnement de composition (polices, fichiers) : effort à consigner.

## 8. Résultats (exécution du 31/10/2026)

Prototype : `spikes/pdf/` (20 instantanés JSON dans `cases/`, gabarits `build_html.py`, `quittance.typ`, `native/`, `typst-lib/`, `chrome-open/`, banc `harness.py`). Moteurs mesurés : **E1p** Chrome 156, un processus par rendu ; **E1o** Chrome maintenu ouvert (puppeteer-core) ; **E2** tracé natif Rust (`pdf-writer` 0.15, `ttf-parser`, `subsetter`) ; **E3c** Typst 0.15.1 en ligne de commande ; **E3l** Typst 0.15.1 en bibliothèque. Polices : IBM Plex Sans **statique** Regular et Bold (instances de la police variable, `fonttools`), 633 Ko. Machine : i7-10510U, Windows 11.

### 8.1 Contrôles automatiques (40 documents par moteur : 20 cas × A4 et A5)

| Contrôle | E1p / E1o | E2 | E3c / E3l |
|---|---|---|---|
| Une seule page | 40/40 | 40/40 | 40/40 |
| Texte extrait égal à l'instantané | 40/40 | 40/40 | 40/40 |
| Polices embarquées | 40/40, **en Type3** (tracés sans programme de police) | 40/40, TrueType en sous-ensemble | 40/40, TrueType en sous-ensemble |
| Métadonnées | aucun auteur ; producteur « Skia/PDF m156 » et créateur = chaîne d'agent de Chrome | aucun auteur ; producteur « spike-native » | aucun auteur ; créateur « Typst 0.15.1 » |
| Référence externe (`/URI`, `/Launch`…) | 0 | 0 | 0 |
| Pixels colorés (rendu 150 dpi) ; texte noir | 0 ; oui | 0 ; oui | 0 ; oui |
| Requêtes sortantes | E1o : 407 requêtes, **0** hors fichiers locaux (interception) ; E1p : résolveur DNS neutralisé, journal réseau non relevé | aucun code réseau | aucun code réseau (aucun paquet `@preview`) |
| Même SHA-256 sur deux rendus | **non** (dates de création et de modification, 2 octets) | oui | oui |

Chrome convertit la police en Type3 **même avec des polices statiques** (constat, cause non creusée). Contrôle visuel : planche des huit cas difficiles (nom long, société, adresse, 999 999 999, annulée, balisage, logo maximal, rectificative) en A5 pour chaque moteur : troncature à deux lignes avec « … », filigrane lisible sous le texte, balisage affiché tel quel. Défaut connu de mon gabarit Typst : espacements irréguliers dans l'en-tête (blocs de mesure) ; non corrigé.

### 8.2 Performance (100 rendus A4 : les 20 cas × 5, échauffement de 3 rendus non compté)

| Mesure | E1p | E1o | E2 | E3c | E3l |
|---|---|---|---|---|---|
| Médiane / p95 / max (ms), processus par rendu | 1 038 / 1 228 / 1 932 | 261 / 314 / 349 | 16 / 19 / 61 | 146 / 171 / 188 | n. a. |
| Médiane / p95 (ms), 300 rendus dans un processus | n. a. | n. a. | 2,1 / 2,8 | n. a. | 2,6 / 3,6 |
| Mémoire au pic (Mo) | 490 | 481 | 7 | 22 | 17 |
| Poids du moteur | Chrome installé : 514 Mo | idem | binaire 9,7 Mo (polices comprises) | `typst` 50,1 Mo | binaire 75,6 Mo |
| Poids moyen d'un PDF | 109 Ko | 109 Ko | 29 Ko | 31 Ko | 31 Ko |
| Effort de mise en page (estimation, non chronométrée) | 0,5 h | 0,5 h | 2 h (mesure du texte, coupure, troncature, sous-ensemble de police, table Unicode) | 1 h | 1 h (+ 10 min de compilation) |

L'objectif « p95 inférieur à 1 s » n'est **pas** tenu par E1p (1 228 ms), l'est par tous les autres. Aucun moteur ne dépasse 500 Mo de mémoire ; E1 en demande environ 480, soit près d'un quart d'un serveur de 2 Go. Valeurs absolues d'un PC, à remesurer à l'étape 01 sur le serveur d'essai.

### 8.3 Montant en lettres

Fonction pure `amount_in_words` (`spikes/pdf/words/`, aucune dépendance). 22 011 valeurs contrôlées : tous les entiers de 0 à 2 000, les frontières de `TESTING_QA.md` §6, 999 999 999 et 20 000 valeurs tirées avec une graine fixe entre 0 et 999 999 999.
- Relecture par un analyseur « mots vers nombre » **écrit à part**, en Python : 22 011 concordances sur 22 011.
- Comparaison avec `num2words` (langue française) : 22 011 identiques, 0 divergence de mots ni de ponctuation.
- 26 frontières comparées à un tableau écrit à la main, 3 tests unitaires : réussis.

Limite de cette preuve : `num2words` applique les mêmes règles usuelles (orthographe traditionnelle) ; l'accord ne valide pas le libellé juridique. **Règles à faire valider par Mimis** (elles ne sont pas inventées par le prototype, elles sont celles de l'usage) : (1) trait d'union seulement entre dizaines et unités ; (2) « et » sans trait d'union pour 21, 31, 41, 51, 61, 71, mais pas 81 ni 91 ; (3) « quatre-vingts » et « cents » avec « s » quand rien ne suit, sans « s » devant « mille » (« quatre-vingt mille », « deux cent mille »), avec « s » devant « million » ; (4) « mille » invariable, jamais « un mille » ; (5) « de » devant la devise pour des millions entiers (« un million de francs CFA ») ; (6) « franc » au singulier pour 0 et 1 (« zéro franc CFA »).

### 8.4 Ce qui reste à faire

- **Impression papier par Mimis** (critère 7 du protocole) : A4 et A5 pour trois cas (accents, montant maximal, annulée), depuis le PDF puis depuis l'aperçu HTML. Les PDF à imprimer sont dans `spikes/pdf/build/e3-cli/` (et `e2/`, `e1-proc/`) ; la question 7 d'`API_CONTRACTS.md` reste donc ouverte.
- Relever le journal réseau de Chrome en mode processus (`--log-net-log`).
- La licence de Typst (Apache-2.0, à vérifier) et celle de la police (OFL, jointe) sont à valider.

### 8.5 Décision proposée (à valider par Mimis)

**Moteur de la décision n° 7 : Typst, en bibliothèque derrière l'interface `PdfRenderer`.** Raisons mesurées : 3 ms par rendu, 17 Mo, PDF de 31 Ko avec polices en sous-ensemble TrueType, résultat reproductible octet pour octet, aucun processus externe ni réseau, gabarit lisible en texte. Coûts : binaire de 75,6 Mo, langage de gabarit à apprendre, **API instable** (trois différences d'API ont bloqué ma première compilation en 0.15 : emplacement de PagedDocument, construction de FileId, signature de 	oday) : figer la version. Le tracé natif (E2) est le plus léger mais coûte le double d'effort et chaque changement de mise en page demande du code Rust. Chrome (E1) reste le repli : HTML et CSS connus, mais 1 s au p95 en processus par rendu, 480 Mo de mémoire, polices en Type3 et PDF non reproductibles. Cette proposition est **provisoire** tant que l'impression papier n'est pas faite.

