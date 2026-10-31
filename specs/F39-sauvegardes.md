# F39 — Sauvegardes chiffrées, restauration testée, journalisation technique

| Champ | Valeur |
|---|---|
| Statut | **Validée par Mimis le 31/10/2026 (DP-82), sauf le choix du second fournisseur**, reporté ; Docker et outillage : DP-83 (délégation) |
| Étape | 01, `roadmap/etape-01-socle-ci-environnements-sauvegardes.md` §4 |
| Dépend de | DP-55, DP-69, DP-70, DP-71, DP-78 ; décisions n° 24, 53, 95 ; contenus C-01 et C-02 |
| Critères visés | PP-13, PP-14, PP-15, PP-33 (`TESTING_QA.md` §12) |
| Écart avec la fiche d'étape | Elle écrit « RPO 15 min » et « 30 jours d'historique » : **DP-69 suspend ces deux objectifs** (plan gratuit de Neon). Cette fiche applique DP-69 |

## 1. Objet

Garantir qu'une perte (erreur humaine, panne, perte de l'hébergeur de la base, attaque du serveur) ne détruit ni les écritures d'argent ni les PDF de quittance déjà remis aux locataires, et **prouver** par une restauration complète chronométrée que la sauvegarde sert. Aucun écran : des commandes d'exploitation (`immo backup …`), des alertes et des lignes `platform_events`.

## 2. Règles

| N° | Règle | Origine |
|---|---|---|
| F39-R1 | **Trois couches.** (1) Historique du service de base : au plan gratuit, **6 heures** de restauration à un instant donné. (2) **Copie logique quotidienne** : `pg_dump -Fc` avec le rôle `immo_backup`, **chiffrée avant envoi**, envoyée chez un **second fournisseur**. (3) **Réplication quotidienne du seau de fichiers** vers le même second fournisseur. | `DEPLOYMENT.md` §5.1 ; DP-69 |
| F39-R2 | **Objectifs au pilote** (remplacent ceux de la fiche d'étape, DP-69) : récupération d'une erreur récente par restauration à un instant donné dans les **6 dernières heures** ; **perte maximale de 24 heures** si la base est perdue (une copie par jour) ; **reprise sous 4 heures** en journée ouvrée ; **30 copies quotidiennes et 12 copies mensuelles** conservées ; **test de restauration chaque mois**, à partir d'une copie logique. Les objectifs de 15 minutes et de 30 jours sont **suspendus** jusqu'aux premiers abonnés payants. | DP-69, DP-82 (acceptés par Mimis) |
| F39-R3 | **Chiffrement à clé publique** (décision n° 95, DP-82) : le serveur et le travail de sauvegarde ne portent que la **clé publique** ; la **clé privée** est hors ligne, en **deux exemplaires** (le développeur et le porteur du projet), et ne sert qu'à restaurer. Outil retenu : `age` (DP-82). Clé privée en deux exemplaires, détenus par Mimis et le développeur. Un attaquant qui prend le serveur ne peut pas relire les copies. | `DEPLOYMENT.md` §5.2 ; `SECURITY.md` §5.3 |
| F39-R4 | **La clé de chiffrement des champs sensibles** (`*_enc`) est sauvegardée **à part**, en deux exemplaires : sans elle, une restauration rend ces champs illisibles. | `DEPLOYMENT.md` §5.2 |
| F39-R5 | **Droits d'envoi en écriture seule** : les identifiants du travail ne lisent, ne suppriment et n'écrasent rien ; **verrouillage d'objet** ou versionnage chez le second fournisseur. Si le fournisseur n'en offre pas : copie hors ligne mensuelle et écart noté. | `SECURITY.md` §5.3 ; décision n° 53 |
| F39-R6 | **Manifeste** de chaque copie : nom, taille, SHA-256, version du schéma, date, nombre de lignes par table. Vérifié à la restauration et chaque nuit. | `DEPLOYMENT.md` §5.2 et §5.5 |
| F39-R7 | **Rôle `immo_backup`** : `BYPASSRLS`, créé par SQL, jamais utilisé par l'application. Il reçoit `SELECT` sur **chaque** table : une migration qui crée une table sans ce droit fait échouer la copie, donc un **test de schéma** le vérifie (DP-78). Les rôles cités par les politiques doivent exister avant toute restauration. | DP-78 |
| F39-R8 | **Aucune copie n'est valide avant d'avoir été restaurée une fois.** La première restauration complète est faite avant toute donnée réelle (PP-14) ; le contrôle nocturne vérifie existence, taille, empreinte et manifeste de la dernière copie. | `DEPLOYMENT.md` §5.5 |
| F39-R9 | **Le PDF émis ne se régénère pas** (DP-71) : la couche 3 est donc la seule protection d'une preuve. Versionnage du seau, objet écrit « créer seulement », empreinte comparée à celle de la base. | DP-70, DP-71 |
| F39-R10 | **Journalisation** : une ligne `platform_events` pour chaque sauvegarde, vérification, restauration (qui, quand, pourquoi, empreinte, durée) et chaque déploiement. Aucune donnée personnelle dans les messages d'alerte. | `DEPLOYMENT.md` §2.2, §5.3, §7.3 |
| F39-R11 | **Restauration** (`DEPLOYMENT.md` §5.4) : toujours sur une **instance neuve**, jamais sur la production ; point de restauration choisi et perte de données notée ; **restauration à un instant donné** par branche Neon créée par l'**API** avec `parent_id` et `parent_timestamp` (le CLI ne combine pas les deux) ; au-delà de 6 heures, copie logique déchiffrée puis `pg_restore` ; contrôles de cohérence puis compte rendu. | DP-78 |
| F39-R12 | **Restauration d'une seule organisation** : procédure **manuelle** écrite et répétée une fois (copie logique sur une instance temporaire, extraction par `org_id`, réinjection par la procédure d'exception de `DATABASE_SCHEMA.md` §8.4, seconde personne présente). Outillage [APRÈS]. | `DEPLOYMENT.md` §5.4 |
| F39-R13 | **Bascule `live`** : avant les données réelles d'une agence, Mimis accepte **par écrit** le risque de perte entre deux copies et la suspension du calcul en fin de quota du plan gratuit. | DP-69 |

## 3. Commandes d'exploitation (aucun écran)

| Commande | Effet |
|---|---|
| `immo backup run` | copie logique, chiffrement, envoi, manifeste, ligne `platform_events` ; lancée chaque nuit à 02 h 00 (heure à confirmer, hors zone d'encaissement) |
| `immo backup verify [--latest \| --file <nom>]` | existence, taille, SHA-256 et manifeste ; avec la clé privée fournie, déchiffrement d'essai et `pg_restore --list` |
| `immo backup list` | copies présentes, tailles, dates, état du verrouillage |
| `immo backup restore --target <instance>` | restaure une copie sur une instance **neuve** ; **refuse** une cible qui n'est pas vide ou qui est la production |

## 4. Cas limites

| Cas | Comportement attendu |
|---|---|
| **Clé privée perdue** | les copies sont illisibles ; le second exemplaire est retrouvé par la seconde personne (exercice trimestriel, `DEPLOYMENT.md` §5.6) ; si les deux sont perdus, les copies sont perdues et l'alerte S1 est levée : nouvelle clé, nouvelle copie complète le jour même |
| **Copie corrompue** | écart d'empreinte ou de manifeste : alerte, la copie est marquée invalide, la précédente reste la dernière valide |
| **Copie de moins de 50 % de la précédente** | alerte `backup.too_small` ; faux positif possible après une remise à zéro d'organisation en `setup` (R-110) : acquittement avec motif |
| **Second fournisseur sans verrouillage d'objet** | écart noté, copie hors ligne mensuelle ; l'alerte « verrouillage désactivé » est inactive et remplacée par un rappel mensuel |
| **Plan Neon : 6 heures d'historique** | assumé par DP-69 ; l'écart est couvert par les copies logiques ; la perte maximale est de 24 heures |
| **Calcul Neon suspendu en fin de quota** (100 CU-heures par mois) | la copie échoue : alerte ; `pg_dump` réveille le calcul et consomme des CU-heures : à mesurer à l'étape 01 |
| **Nouvelle table sans `SELECT` pour `immo_backup`** | `pg_dump` échoue (constaté au prototype b) : le test de schéma de l'étape 01 doit l'attraper avant la fusion |
| **Restauration vers une base sans les rôles des politiques** | échec de `pg_restore` (constaté) : la procédure crée d'abord les rôles |
| **Envoi interrompu** | la copie locale temporaire est conservée jusqu'à l'envoi réussi, puis détruite ; nouvel essai ; alerte après le deuxième échec |
| **Disque du travail de sauvegarde à plus de 80 %** | alerte S3 ; la copie est écrite en flux quand c'est possible |
| **Écriture pendant la copie** | `pg_dump` est transactionnel : la copie est cohérente à un instant, sans verrou bloquant l'encaissement |
| **Test de restauration en retard de plus de 35 jours** | alerte S3 ; le mois suivant ne se déclare pas « normal » |

## 5. Erreurs et alertes

Alertes de `DEPLOYMENT.md` §7.3, canal unique sans donnée personnelle (décision n° 89 [À valider]). Codes techniques proposés (anglais, DP-67) : `backup.missing`, `backup.too_small`, `backup.manifest_mismatch`, `backup.encryption_failed`, `backup.upload_failed`, `backup.lock_disabled`, `backup.restore_overdue`.

| Alerte | Seuil | Gravité |
|---|---|---|
| Sauvegarde manquante | plus de 24 heures | S2 |
| Copie de moins de 50 % de la précédente | à la détection | S2 |
| Échec de chiffrement ou d'envoi | dès le deuxième échec | S2 |
| Verrouillage d'objet désactivé | dès la détection | S2 |
| Test de restauration en retard | plus de 35 jours | S3 |
| Disque du travail | plus de 80 % | S3 |
| Certificat | moins de 14 jours | S3 |

## 6. Cas chiffrés (critères de l'étape)

| N° | Cas | Résultat attendu | Preuve |
|---|---|---|---|
| F39-C1 | Une copie de la veille existe chez le second fournisseur | taille et empreinte conformes au manifeste ; **illisible sans la clé privée** | `immo backup verify` ; essai de lecture sans clé |
| F39-C2 | Suppression avec les identifiants du travail d'envoi | **refusée** | essai noté dans le compte rendu (PP-13) |
| F39-C3 | Jeu synthétique d'au moins 3 organisations fictives et 1 000 lignes, restauré sur une instance neuve | **durée totale inférieure à 4 heures** ; comptes de lignes et sommes des paiements et des affectations **identiques** par organisation ; cinq requêtes de contrôle à zéro | compte rendu au modèle de `DEPLOYMENT.md` §5.6 (PP-14) |
| F39-C4 | Sauvegarde volontairement retardée de 25 heures en essai | alerte reçue sur le canal | capture du canal (PP-15) |
| F39-C5 | Restauration à un instant donné dans les 6 dernières heures | l'état antérieur est retrouvé ; durée notée | prototype (b) : branche à T0 en 1,9 s (100 lignes sur 150) ; à rejouer sur l'essai |
| F39-C6 | Rôle `immo_backup` sur le schéma complet | `pg_dump` réussit ; RLS forcée conservée après restauration | prototype (b) sur tables jouets ; à rejouer sur le vrai schéma |

Ce que l'étape 00 a déjà éprouvé : `pg_dump -Fc` avec `immo_backup` puis `pg_restore` vers un second projet Neon, 10 000 lignes sur 3 organisations, 18,4 s, comptes identiques par table et par organisation (DP-78). Cela ne couvre ni le chiffrement, ni le second fournisseur, ni le schéma réel, ni le chronométrage de la chaîne complète.

## 7. Ce qui bloque le démarrage de l'étape 01 (à décider ou fournir par Mimis)

L'étape 01 ne peut pas être menée à son terme tant que ces éléments, qui ne dépendent pas du code, ne sont pas fournis. La fiche est écrite pour que la validation puisse se faire sans eux, mais l'**exécution** les exige.

1. **C-01 — Comptes** : plateforme de dépôt et de CI sur un compte d'organisation (décision n° 87, prise par DP-56), registrar, hébergeur du serveur, second fournisseur, gestionnaire de mots de passe ; double authentification ; deux administrateurs ; codes de secours à deux endroits.
2. **C-02 — Choix d'hébergement** (décisions n° 5 et 86, **ouvertes**) : serveur applicatif **dans la région de la base** (DP-77), stockage objet privé avec versionnage et verrouillage d'objet si possible (DP-70, DP-71), **second fournisseur avec identifiants en écriture seule**, par la grille de `DEPLOYMENT.md` §1.2 ; budget mensuel accepté (`DEPLOYMENT.md` §1.3 : 42 à 111 €). La région de la base est celle de `immo-spikes` (`aws-eu-central-1`) tant que rien d'autre n'est décidé.
3. **C-03 — Domaine technique provisoire** (décisions n° 13, 90, 100).
4. **Décisions à prendre ou confirmer** : n° 24 (objectifs adaptés par DP-69), n° 53 et 95 (droits et chiffrement à clé publique), n° 88 (Compose et proxy à certificats automatiques), n° 89 (canal d'alerte), n° 92 (secrets).
5. **Docker est absent du poste** (CLAUDE.md, « Poste réel »). L'étape 01 prévoit un Dockerfile, Compose et un développement local sous Docker. Sans droits d'administrateur à distance, l'installation de Docker Desktop (WSL ou Hyper-V) n'est pas possible. **Proposition** : construire et tester l'image **dans la CI** (Linux), développer en local avec le PostgreSQL du port 5433 déjà utilisé par les prototypes, et installer Docker plus tard si Mimis le souhaite [À valider].
6. **Prototype (e)** : la mesure réseau à Ouagadougou, reportée par DP-75, est une tâche de l'étape 01 (5.7, point 9) ; Mimis la fera plus tard.

## 8. Réponses de Mimis (31/10/2026)

1. Objectifs adaptés par DP-69 : **acceptables** pour le pilote (DP-82).
2. Chiffrement : **à clé publique** (DP-82).
3. Clé privée : **deux exemplaires détenus par Mimis et le développeur** (DP-82).
4. Second fournisseur : **reporté** ; PP-13 reste à atteindre.
5. Docker : image construite **dans la CI seulement**, par délégation (DP-83).
