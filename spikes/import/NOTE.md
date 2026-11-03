# Prototype (d) — Import d'un Excel réaliste

| Champ | Valeur |
|---|---|
| Statut | **Exécuté le 31/10/2026 (protocole validé par Mimis), y compris le passage de comptage sur Neon (section 8.5).** Résultats en section 8 ; décision proposée en section 6, **en attente de validation de Mimis** |
| Durée maximale | 3 jours (dépasser est un résultat à noter) |
| Décision attendue | Bibliothèque de lecture XLSX et CSV ; format du modèle Excel Immo Suite ; règles de normalisation ; limites de taille |
| Documents | `ARCHITECTURE.md` §7 ; `TESTING_QA.md` §7 ; `SECURITY.md` §7.4, §11 ; `DOMAINE.md` R-080, E-20, E-21 ; DP-70 |

## 1. Objectif

Vérifier la chaîne lecture, correspondance de colonnes, validation, rapport, annulation sur un fichier de 200 lignes de la maquette et sur des variantes sales, avec des fichiers piégés refusés ou neutralisés, sans qu'aucune ligne ne soit créée avant confirmation et sans trace après annulation.

## 2. Environnement

- Rust 1.99.1 : `spikes/import/` (lecteur, correspondance, validation, rapport). Bibliothèques candidates, à figer en début de prototype [À vérifier] : une bibliothèque de lecture XLSX en lecture seule (par exemple `calamine`), `csv`, `encoding_rs` avec détection d'encodage, `zip` pour inspecter l'archive avant lecture.
- PostgreSQL local 5433 pour les tables jouets (`import_batches`, `import_rows`, `tenants`, `units`, `import_batch_id` sur chaque ligne créée) ; un passage de comptage sur Neon (branche `spike-d`). La fonction d'annulation est une exception encadrée simplifiée.
- Python (venv `spikes/.venv`, `openpyxl`) pour **générer** les fichiers de test dans `spikes/import/fixtures/`. Données **synthétiques uniquement** (décision n° 15) ; la source du fichier de référence est `maquette/data/import.js` (`importExemple`).
- Dans le produit, le fichier d'origine ira dans le stockage objet (DP-70) ; le prototype le lit depuis le disque.

## 3. Protocole

1. **Fichier de référence** : 200 lignes ; erreurs aux lignes **14, 87 et 152** ; 2 paires de doublons de téléphone ; bloc fictif `+226 00 00 04 NN`. Attendu : **197 créables**, 3 erreurs listées, 2 doublons proposés à la fusion (E-20).
2. **Variantes sales** (`TESTING_QA.md` §7) : structure (colonnes décalées, titres différents « Tél / Téléphone / N° tel », lignes de titre au-dessus, cellules fusionnées, lignes et colonnes masquées, colonne obligatoire absente) ; encodage (CSV `;` UTF-8 avec BOM, CSV Windows-1252, accents composés et décomposés, retours à la ligne dans une cellule) ; dates (`13/11/2026`, `13/11/26`, `2026-11-13`, nombre de série Excel, `13 nov. 2026`, `31/02/2026`, `03/04/2026`) ; montants (`75 000`, `75.000`, `75000 F`, `75 000 FCFA`, `1,250,000`, `75000,5`, `0`, `-5000`, `1 000 000 000`) ; téléphones, énumérations, doublons, valeurs métier, limites (5 000 lignes pile et 5 001, 5 Mo).
3. **Fichiers piégés** : `.xls` ; `.xlsm` ; archive à fort ratio de compression (générée sans jamais être décompressée entièrement) ; XML avec entité externe pointant vers un fichier local témoin ; formules (`=HYPERLINK(…)`, `=cmd|…`) ; cellules commençant par `=`, `+`, `-` ou `@` ; faux XLSX (extension seule). Attendu : refusés ou neutralisés, **aucune formule évaluée**, aucune lecture d'un fichier du poste, rapport exporté avec cellules neutralisées.
4. **Garanties de processus** (comptage en base) : zéro ligne créée avant confirmation ; après annulation, rien ne reste du lot (E-21) ; double confirmation sans effet ; annulation refusée après un paiement.
5. **Performance** : 2 000 lignes analysées en moins de 5 secondes (dix passages, médiane et maximum, compilation en mode release) ; mémoire au pic pour 5 000 lignes et pour le fichier de 5 Mo (seuil proposé : 100 Mo [À valider]).
6. Chaque fichier a son `*.expected.json` (numéros de ligne, statut, code) ; le rapport est comparé octet à octet.

## 4. Résultats chiffrés (à remplir)

| Mesure | Résultat |
|---|---|
| Fichier de référence : créables / erreurs (lignes) / doublons | |
| Variantes sales lues correctement (sur le total) | |
| Fichiers piégés refusés ou neutralisés (sur le total) ; formules évaluées | |
| Lignes créées avant confirmation ; lignes restantes après annulation | |
| 2 000 lignes : médiane / maximum (s) | |
| Mémoire au pic : 5 000 lignes ; 5 Mo (Mo) | |
| Bibliothèques retenues et versions | |

## 5. Critères d'acceptation (étape 00 §6)

8. Rapport exact sur le fichier de référence (197 créables, 3 erreurs aux lignes 14, 87 et 152, 2 doublons proposés à la fusion) ; après annulation, aucune trace (E-21) ; fichiers piégés refusés.

## 6. Décision proposée

Voir la section 8.6.

## 7. Écarts et risques déjà connus

- Les fichiers sont synthétiques : l'Excel réel d'une agence (risque « plus sale que prévu », `ROADMAP.md` §9) sera confronté au moteur à l'étape 12, avec les fichiers de C-09.
- Une date comme `03/04/2026` est ambiguë : la règle (refus ou format imposé par le modèle) est une règle métier à faire trancher, non à inventer.
- Le prototype ne teste pas l'envoi du fichier vers le stockage objet (DP-70) : à l'étape 12.

## 8. Résultats (exécution du 31/10/2026)

Prototype : `spikes/import/` (`src/analyze.rs` : lecture, refus, correspondance, validation, rapport ; `src/db.rs` : garanties de processus ; `make_fixtures.py` : 31 fixtures et leur `*.expected.json` ; `check_import.py` ; `perf_import.py`). Bibliothèques : `calamine` 0.37.0 (lecture XLSX en lecture seule, aucune formule évaluée), `csv` 1.4.0, `encoding_rs` 0.8.44 (Windows-1252), `zip` 9.2.0 (inspection du répertoire central), `unicode-normalization` 0.1.26, `sqlx` 0.9.0. Les fichiers sont **synthétiques**. Le résultat attendu de chaque fichier est écrit **à la construction du fichier**, dans le générateur, indépendamment de l'analyseur ; celui du fichier de référence vient de `maquette/data/import.js` (extrait par un script Node, sans retranscription à la main).

### 8.1 Tableau des mesures

| Mesure | Résultat |
|---|---|
| Fichier de référence (200 lignes) : créables / erreurs / doublons | **197** / **3, aux lignes 14, 87, 152** (`import.phone_invalid`, `import.rent_not_positive`, `import.date_invalid`) / **2 paires** (33 et 118, 61 et 140) proposées à la fusion |
| Fichiers conformes à leur résultat attendu | **31 sur 31**, 294 lignes détaillées sur 294 |
| Variantes sales (structure 4, encodage 4, dates 3, montants 1, téléphones 1, énumérations 1, doublons et valeurs métier 2, limites 6) | 22 sur 22 lues comme attendu |
| Fichiers piégés (8) | 6 **refusés** (`.xls`, `.xlsm`, macro déguisée en `.xlsx`, archive bombe, entité XML externe, faux XLSX) ; 2 **neutralisés** (formules, injection dans un CSV) ; **0 formule évaluée** ; fichier témoin de l'entité externe **jamais lu** |
| Lignes créées avant confirmation | **0** |
| Lignes restantes après annulation (E-21) | **0** (locataires, lots, baux, lignes d'import) ; l'en-tête du lot reste, statut « annulé » |
| Double confirmation | sans effet (0 bail créé la seconde fois) |
| Annulation après un paiement | **refusée** (R-080), rien supprimé |
| 2 000 lignes : médiane / maximum, 10 passes, mode release | **41 ms / 44 ms** (seuil 5 s) |
| Mémoire au pic : 2 000 lignes / 5 000 lignes / fichier de 3,98 Mo | 12 Mo / 14 Mo / 29 Mo (seuil proposé 100 Mo) ; 5 000 lignes en 108 ms |

### 8.2 Ce qui a été vérifié, fichier par fichier

- **Structure** : titres au-dessus de l'en-tête, ligne vide au milieu, titres synonymes dans un autre ordre avec une colonne en trop (listée comme non reconnue), colonne obligatoire absente (`import.column_missing`, loyer), cellule fusionnée, colonne et ligne masquées.
- **Encodage** : CSV `;` UTF-8 avec BOM, CSV Windows-1252, CSV `,` avec guillemets et retour à la ligne dans une cellule, accents décomposés (e + U+0301) ramenés à la forme composée.
- **Dates** : `13/11/2026`, `13/11/26`, `2026-11-13`, nombre de série, vraie cellule date, `13 nov. 2026`, `31/02/2026` et `2026-02-30` (`import.date_invalid`), cellule vide, `03/04/2026` ; ordre ambigu signalé quand aucune date de la colonne ne le lève ; date de sortie avant l'entrée (`import.date_order`).
- **Montants** : `75 000`, `75.000`, `75000 F`, `75 000 FCFA`, `1,250,000`, espace insécable, nombre ; refusés sans troncature : `75000,5` et `75000.5` (`import.amount_not_integer`), `0` et `-5000` (`import.rent_not_positive`), `1 000 000 000` (`import.amount_out_of_range`).
- **Téléphones** vers `+226XXXXXXXX` ; refusés : lettre, 7 chiffres, deux numéros dans la cellule ; vide : avertissement.
- **Énumérations** insensibles à la casse et aux accents ; « Entrepôt » : `import.enum_unknown`.
- **Doublons** de téléphone (proposés à la fusion) ; chevauchement de deux baux fermés sur un même lot (`import.lease_overlap`).
- **Valeurs métier** : jour 31 accepté (R-098), jour absent = 1 avec avertissement (R-031), jours 0 et 32 refusés, dépôt de plus de deux mois en avertissement, arriéré daté accepté, arriéré sans date refusé (R-035).
- **Limites** : 5 000 lignes acceptées, 5 001 refusées (`import.too_many_rows`) ; 3,98 Mo acceptés, 5,94 Mo refusés (`import.too_large`) ; cellule de 600 caractères refusée, 400 acceptée, émoji conservé.

### 8.3 Constats qui demandent une décision (rien n'est inventé : ce sont des règles proposées)

1. **Le fichier de référence est incompatible avec « un lot, un bail actif » (R-020)** si « Maison/Ref » identifie un lot : ses 197 locataires créables se répartissent sur **36 lots**, chacun avec **2 à 6 locataires simultanés** et aucune date de sortie. L'importer ne sait pas dire s'il s'agit de colocations ou d'une référence non unique. Les données sont synthétiques et tirées au hasard, mais le constat vaut pour un vrai fichier : il faut soit une colonne « Bien » en plus de la référence, soit une règle de lecture (colocation).
2. **Dates** : JJ/MM/AAAA est le format du produit ; l'ordre est confirmé par une date de la colonne dont le premier élément dépasse 12 ; sinon les dates dont les deux premiers éléments diffèrent et sont inférieurs ou égaux à 12 sont **signalées** (`import.date_ambiguous`) sans être refusées. Année à deux chiffres : 20xx avec avertissement. Nombre de série sans format de date : accepté entre 2000 et 2100 avec avertissement.
3. **Montants** : un séparateur suivi d'exactement trois chiffres est un séparateur de milliers ; toute autre forme à séparateur (même `75000,00`) est refusée comme décimale.
4. **Énumération d'usage** : habitation = {habitation, logement, appartement, maison, villa, studio, chambre, résidence} ; commercial = {commercial, boutique, magasin, bureau, local commercial, commerce}. À valider avec l'agence pilote.
5. **Lignes et colonnes masquées** sont lues comme les autres (`calamine` n'expose pas l'attribut) : une ligne masquée par l'utilisateur pour l'exclure serait importée. À décider ; détecter le masquage demande de lire le XML de la feuille.
6. **Annulation** : les données importées disparaissent, l'en-tête du lot reste avec le statut « annulé » comme trace (exception de R-080 à R-005, à confirmer) ; les lignes d'import, qui contiennent des données personnelles, sont supprimées.
7. **Cellules** de plus de 500 caractères : refusées.

### 8.4 Écarts et limites

- Le rapport est comparé à l'attendu **sur les clés présentes dans l'attendu**, en JSON, et non octet à octet comme le protocole le disait ; c'est plus tolérant sur l'ordre des clés, aussi strict sur les valeurs.
- Aucun fichier réel : le risque « plus sale que prévu » reste entier jusqu'à l'étape 12 (C-09).
- Non testés : cellules fusionnées au milieu des données, CSV de 5 000 lignes, limite exacte de 5 Mo (testés à 3,98 et 5,94 Mo), XLSX produits par LibreOffice ou Google Sheets, résolution réelle d'une entité externe (le fichier est refusé avant la lecture, ce qui est le comportement voulu).
- L'envoi du fichier vers le stockage objet (DP-70) n'est pas testé.

### 8.5 Passage de comptage sur Neon (branche `spike-d`)

Les quatre garanties de la section 8.1 ont été rejouées sur la branche Neon spike-d (fichier de référence, 138 ms d'aller-retour, 4 minutes) avec des résultats **identiques** au local : 0 ligne avant confirmation ; confirmation 1 = 197 baux, 197 locataires, 36 lots ; confirmation 2 sans effet ; annulation = 0 locataire, 0 lot, 0 bail, 0 ligne d'import ; annulation refusée après un paiement.

### 8.6 Décision proposée (à valider par Mimis)

Bibliothèques : `calamine` pour XLSX, `csv` + `encoding_rs` pour CSV, `zip` pour inspecter l'archive avant lecture. Limites 5 000 lignes et 5 Mo **confirmées** (2 000 lignes en 41 ms, 5 000 en 108 ms, 29 Mo au plus) ; elles pourraient être relevées sans risque de performance. Modèle Excel Immo Suite : colonnes et synonymes de `analyze.rs` (8 colonnes de la maquette plus usage, date de sortie, dépôt, arriéré, date d'arriéré), **auxquelles ajouter une colonne « Bien »** si le constat 1 est confirmé. Règles de la section 8.3 à valider une à une.

