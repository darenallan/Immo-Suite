# Étape 12 — Import des données d'une agence (moteur et commande HABEK)

| Champ | Valeur |
|---|---|
| Statut | À faire |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T2 (F14 : moteur) |
| Effort | L, environ 18 JD |
| Dépend de | Étapes 05, 06 et 11 ; prototype (d) de l'étape 00 ; contenu C-09 |
| Jalon | Contribue à J3 |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Faire entrer le portefeuille réel d'une agence (biens, lots, locataires, baux, arriérés) depuis son Excel, avec aperçu, rapport ligne à ligne et annulation tant que rien n'est utilisé. Le moteur est piloté par HABEK en ligne de commande ; l'assistant pour le gérant vient à l'étape 18.
Valeur : le coût de changement (ressaisir 200 locataires) est le premier obstacle d'adoption.

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections |
|---|---|---|
| Fonctionnalité | F14 | dossier §16 F14 |
| Règles | R-035, R-080 ; E-20, E-21 | `DOMAINE.md` |
| Décisions | DP-57 (import par ligne de commande dès la tranche 2), DP-60 ; n° 15, 40, 65 | `DECISIONS.md` |
| Architecture | `ARCHITECTURE.md` | §2.3 module `imports`, §7 (d), §8.6 |
| Sécurité | `SECURITY.md` | §7.4 (fichiers), T7 |
| Schéma | `DATABASE_SCHEMA.md` | §3 « Import », §4.2 (`cancel_import_batch`), §8.1 |
| Contrat | `API_CONTRACTS.md` | §6.4, §7.9, §5.4, §5.6 |
| Tests | `TESTING_QA.md` | §7 en entier, parcours 4 |

## 3. Prérequis

Étapes 05, 06, 11 ; accord support de type `import_assist` ; **C-09** : modèle Excel de l'éditeur (colonnes) et 1 à 2 fichiers d'exemple anonymisés de l'agence candidate avec accord écrit (n° 15). Bibliothèque de lecture XLSX retenue par le prototype (d).

## 4. Fiche de spécification à produire ou valider

`specs/F14-import.md` — colonnes du modèle (bien, lot, usage, loyer, charges, jour d'échéance, locataire, téléphone, début de bail, dépôt, arriérés datés, propriétaire) ; correspondance mémorisée par organisation ; états `uploaded→mapped→previewed→confirmed|cancelled|failed` ; lignes `valid|warning|error|skipped|merged` ; rien n'est créé avant confirmation (R-080) ; doublons de téléphone ou de lot proposés à la fusion ; arriérés créés comme lignes `arrears` datées (R-035) ; annulation refusée dès qu'un paiement, une relance ou une promesse référence un bail du lot ; normalisation des dates, montants (`75 000`, `75.000`, `75000 F`), téléphones `+226` ; limites 5 Mo et 5 000 lignes ; fichiers `.xls`/`.xlsm` refusés, aucune formule évaluée. Cas chiffrés : E-20 (200 lignes, 197 créables dont 2 doublons à fusionner, 3 erreurs aux lignes 14, 87, 152), E-21. **S'arrêter pour validation de Mimis.**

## 5. Tâches détaillées

### 5.1 Base de données
`import_batches`, `import_rows` (contenu brut purgé après confirmation), `import_mappings` ; colonne `import_batch_id` (nullable, clé composite) ajoutée à `owners`, `properties`, `units`, `tenants`, `leases` ; `cancel_import_batch` (`SECURITY DEFINER`) refusée si le lot est utilisé ; compteurs `total = valid + warning + error`.

### 5.2 Noyau métier
Module `imports` : `upload` (extension et contenu vérifiés, plafonds d'archive, XML sans entités externes), `propose_mapping`, `preview` (tâche asynchrone à mémoire et temps bornés), `confirm` (une transaction, idempotente), `cancel`, rapport téléchargeable ; création par les façades de `property` et `lease` (aucun `INSERT` direct) ; audit `import.confirmed`, `import.cancelled` ; événements `import_*`.

### 5.3 API
`POST /import-batches` (multipart), mapping, aperçu (`202` + `/operations/{id}`), `POST …/confirm`, `POST …/cancel`, `GET /import-batches`, `…/rows`, `GET /import-mappings`, `GET /import-template`. Commande `immo import --org … --file …` sous accord support.

### 5.4 Interface
Aucun écran (S16 à l'étape 18) ; modèle Excel téléchargeable.

### 5.5 Documents PDF
Rapport d'import en CSV neutralisé.

### 5.6 Audit et événements
Voir 5.2.

### 5.7 Tests
Fixtures `tests/fixtures/import/` avec fichiers `*.expected.json` : référence 200 lignes ; variantes sales (colonnes décalées, CSV `;` Windows-1252, dates en nombre de série, cellules fusionnées, téléphones) ; limites 5 000/5 001 lignes ; fichiers piégés (bombe d'archive, entité XML, `.xlsm`, formules) ; zéro ligne avant confirmation ; annulation sans trace ; double confirmation sans effet ; 2 000 lignes en moins de 5 s.

## 6. Critères d'acceptation vérifiables

1. Fichier de référence : 197 lignes créables (dont 2 doublons à fusionner) et 3 erreurs listées aux lignes 14, 87, 152 (E-20).
2. Avant confirmation, 0 ligne créée ; après annulation sans paiement, plus aucune trace (E-21) ; annulation refusée après un paiement.
3. Aucune formule évaluée ; fichiers piégés refusés ; rapport exporté neutralisé (PP-18).
4. Dates impossibles ou ambiguës, montants décimaux ou hors bornes signalés avec leur numéro de ligne, sans troncature silencieuse.
5. Les arriérés importés apparaissent dans l'échéancier comme lignes « arriéré » datées et dans les impayés.

## 7. Sécurité et déploiement à vérifier

Analyse isolée avec limites ; fichier brut supprimé après 30 jours [Hypothèse] ; accord support `import_assist` exigé ; jeu d'essai sans donnée réelle.

## 8. Liste de démonstration (Mimis)

1. Le développeur importe le fichier de 200 lignes : lire « 197 créables, 3 erreurs (14, 87, 152) ».
2. Corriger une erreur dans le fichier, réimporter, fusionner un doublon.
3. Confirmer : ouvrir les lots, locataires, baux et arriérés créés.
4. Annuler un autre import : tout a disparu.

## 9. Définition de terminé, risques, hors étape

Terminé : critères 1 à 5 verts ; modèle Excel versionné ; `DECISIONS.md` (n° 15) à jour.

| Risque | Mesure |
|---|---|
| Excel plus sale que prévu | Import assisté, rapport ligne à ligne, annulation |
| Règles R-098 à R-100 divergentes chez l'agence | Confirmées à la découverte (étape 14) |

Hors étape : assistant S16 (étape 18), import de paiements historiques (hors MVP).

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 12
- Dépend de : 05, 06, 11
- Tranche : T2
- Jalon : J3
- F : F14, F40
- R : R-035, R-080
- E : E-20, E-21
- PP : PP-18
- Contenus : C-09
