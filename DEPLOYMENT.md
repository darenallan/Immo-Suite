# Immo Suite — Déploiement et exploitation

| Champ | Valeur |
|---|---|
| Statut | Propriétaire de l'exploitation : environnements, coûts, calendrier de livraison, sauvegardes (procédures), incident, accueil et départ d'une agence. La base est chez Neon (DP-55) ; le reste de l'hébergement et les autres choix restent proposés [À valider]. Les coûts sont des **estimations à vérifier**, non des devis |
| Date | 30/10/2026 |
| Dépend de | `DECISIONS.md` (état au 30/10/2026, en particulier DP-52 à DP-55, DP-60 à DP-62, DP-68 et les décisions n° 5, 15, 24, 26, 46, 54), dossier de cadrage v1.0 (§12, §20.2, §21, §23), `ARCHITECTURE.md` v0.2 (§3.3, §3.10, §8.1, §8.2, §8.5 à §8.7, §10, §11), `DATABASE_SCHEMA.md` v0.2 (§4.2, §6.9, §7, §8), `SECURITY.md` v0.2 (§5, §8, §9), `API_CONTRACTS.md` v0.2, `CODING_RULES.md` v0.2, `TESTING_QA.md` v0.2, `CONTEXTE.md`, `CLAUDE.md` |
| Alimente | `ROADMAP.md`, `CLAUDE.md`, `contenus/` (engagement de confidentialité, guides) |
| Version | 0.2 |

Légende des marqueurs ([PILOTE], [APRÈS], [À valider], [Hypothèse], [À VÉRIFIER AVEC UN JURISTE]) : voir `CLAUDE.md`.

- **[Estimation à vérifier]** : coût estimé, à confronter à la grille tarifaire du fournisseur au moment du choix.
- **[Estimation à vérifier]** : coût estimé, à confronter à la grille tarifaire du fournisseur au moment du choix.

Hiérarchie en cas de conflit : voir `CLAUDE.md` (hiérarchie par sujet). Ce document ne contredit aucun autre.

Principe : **un développeur (environ 25 heures par semaine) et le porteur du projet exploitent le service** (DP-54). Tout ce qui suit privilégie le simple, le répétable et le réversible. Quatre sujets ne se reportent jamais : l'intégrité des écritures (migrations rétrocompatibles, sauvegarde avant migration), l'isolation entre organisations (secrets et rôles séparés par environnement), **les sauvegardes avec restauration testée**, et la traçabilité des accès.

Indispensable avant le premier pilote :

- **[PILOTE]** Trois environnements séparés ; une image unique de l'essai à la production ; secrets distincts par environnement ; HTTPS ; domaine sous contrôle de l'organisation.
- **[PILOTE]** Sauvegarde continue, copie quotidienne chiffrée chez un second fournisseur, **première restauration complète chronométrée**, alerte de sauvegarde manquante.
- **[PILOTE]** Mise en production et retour arrière exercés sur l'essai ; migrations rétrocompatibles précédées d'une sauvegarde.
- **[PILOTE]** Supervision minimale : sonde externe, alertes de la section 7, journaux sans donnée personnelle.
- **[PILOTE]** Procédure d'incident, de maintenance, d'accueil d'une agence, export complet d'une organisation, gouvernance des accès documentée.
- **[APRÈS]** Mise en production sans coupure (deux instances), métriques Prometheus, analyse et signature des images, page d'état automatisée, écran d'export pour le gérant.

## 1. Environnements

### 1.1 Les trois environnements (`ARCHITECTURE.md` §11.3, `SECURITY.md` §5.5)

| | Développement | Essai | Production |
|---|---|---|---|
| Rôle | poste de chaque développeur | recette, démonstration des nouveautés, répétition des migrations, test de restauration mensuel, séances de recette avec les référents | agences pilotes |
| Données | organisation de démonstration et jeux de tests, **aucune donnée réelle** | démonstration, organisations de test, copie **anonymisée** de la production rafraîchie chaque semaine (décision n° 15, accord écrit) | données réelles ; organisation de démonstration (`kind = demo`, décision n° 26) |
| Infrastructure | PostgreSQL en conteneur local, stockage de fichiers sur disque, horloge fixable | même forme que la production, en plus petit | serveur applicatif, PostgreSQL managé, stockage objet, sauvegardes chez un second fournisseur |
| Secrets, clés, seaux, noms d'hôte, noms de cookies | propres | **propres** | **propres** |
| Accès | le développeur | l'équipe et les référents pilotes (accès restreint) | le développeur et le porteur du projet (accès d'urgence) ; le support par la console |
| Déploiement | local | **automatique** à chaque fusion sur `main` | **promotion manuelle de la même image** (aucune reconstruction) |
| Accès réseau à la production | aucun | aucun | — |

- **Aucun compte, secret, clé ni jeu de données de production** sur un poste de développement ou sur l'essai. La seule donnée de production qui atteint l'essai est la copie **anonymisée** (section 10.2).
- L'essai a ses propres rôles PostgreSQL (`immo_owner`, `immo_app`, `immo_platform`, `immo_readonly`), ses clés de chiffrement et son seau : le jeu est recréé, jamais copié depuis la production.

### 1.2 Hébergement (reprise de `ARCHITECTURE.md` §11, décisions n° 5, 21 et 46)

**Décision prise (DP-55)** : la base de données est PostgreSQL managé chez **Neon**. Le serveur applicatif, le stockage objet et le second fournisseur de sauvegardes restent à choisir (décision n° 5, en partie ouverte). Ce document reprend les options d'`ARCHITECTURE.md` §11.1 et les critères de choix, **sans trancher** ces trois ressources :

| Option | Latence depuis Ouagadougou | Résidence des données | Exploitation | Coût mensuel estimé (production) |
|---|---|---|---|---|
| (a) Serveur virtuel et PostgreSQL managé chez un hébergeur européen à bas coût (France, Allemagne) | de l'ordre de 100 à 200 ms [Hypothèse : à mesurer, prototype (e)] | hors du pays : encadré par la loi n° 001-2021/AN (H12) ; contrat avec restitution et chiffrement | simple ; services managés, documentation abondante | 26 à 80 € [Estimation à vérifier] |
| (b) Région d'un grand fournisseur en Afrique (Afrique du Sud, Nigeria) | pas forcément meilleure (trafic souvent transité par l'Europe) [Hypothèse] | hors du pays, sur le continent | complète, facturation plus complexe | plus élevé [Estimation à vérifier] |
| (c) Hébergeur ou centre de données au Burkina Faso | la plus faible [Hypothèse] | dans le pays | services managés (base, objet, sauvegarde) à vérifier ; sauvegardes hors site nécessaires de toute façon | inconnu, à demander |

**Recommandation d'`ARCHITECTURE.md`, reprise** : (a) pour le pilote, **sous trois conditions** : latence réelle mesurée (prototype e) ; contrat avec clause de restitution, chiffrement au repos et localisation déclarée ; avis de la CIL ou d'un juriste avant de passer de l'essai aux données réelles, **ou risque accepté par écrit**. L'architecture ne dépend d'aucun service propriétaire : PostgreSQL standard, stockage compatible S3, conteneur. Un déménagement reste possible par sauvegarde et restauration.

**Grille de choix de l'hébergeur** (chaque ligne éliminatoire ou notée ; à remplir avant la signature, `SECURITY.md` §11 « revue des politiques de l'hébergeur ») :

| Critère | Éliminatoire ? | Source |
|---|---|---|
| PostgreSQL managé en version 16 ou plus avec restauration à un instant donné (15 minutes ou moins) **et restauration sur une nouvelle instance** (fenêtre de 6 heures au plan gratuit de Neon, écart assumé par DP-69 ; confirmé sur Neon par le prototype (b), DP-78) | oui | `ARCHITECTURE.md` §3.3 |
| Création de rôles personnalisés **sans** `BYPASSRLS` pour l'application, propriétaire distinct ; extension `btree_gist` activable | oui | `DATABASE_SCHEMA.md` §6, A3 |
| **Un rôle d'administration avec `BYPASSRLS` utilisable pour la copie logique** (`pg_dump` échoue sous RLS forcée) ; sinon, on ne force pas la RLS (décision n° 46) | oui, à trancher **avant** de choisir | `DATABASE_SCHEMA.md` §6.9, A1 |
| TLS avec vérification du certificat pour la base et le stockage objet | oui | `SECURITY.md` §5.1 |
| Stockage objet privé, versionnage, chiffrement côté serveur, adresses signées | oui | `ARCHITECTURE.md` §3.10 |
| Chiffrement du disque ; politique d'accès du personnel de l'hébergeur | oui | `SECURITY.md` §5.2 |
| Clauses de restitution, destruction, localisation, liste des sous-traitants | oui | dossier §2.4 [À VÉRIFIER AVEC UN JURISTE] |
| Latence p95 du parcours Encaisser (3G et 4G, deux opérateurs) | mesurée | `ARCHITECTURE.md` §7 (e) |
| Coût total mensuel, facturation en euros ou en FCFA, moyens de paiement acceptés sur place | noté | |
| Compte d'organisation avec double authentification, plusieurs administrateurs | oui | section 3.5 |
| Interface en ligne de commande ou interface de programmation pour automatiser | noté | section 3.2 |
| Support, historique de pannes, sauvegardes exportables | noté | |

**Pour la base (Neon)** : les trois lignes éliminatoires sur PostgreSQL ci-dessus sont vérifiées au prototype (b) [À vérifier : documentation de Neon à relire à cette date]. Points d'attention : les rôles créés par la console ou l'API de Neon reçoivent `BYPASSRLS` (`immo_app` et `immo_platform` sont donc créés en SQL, `DATABASE_SCHEMA.md` §6.9) ; la fenêtre d'historique dépend du plan (de 6 heures à 30 jours) et l'objectif de 30 jours en dépend ; mise en veille automatique désactivée en production ; région la plus proche de Ouagadougou ; connexion directe.

Candidats à comparer pour le serveur et le stockage (exemples, **sans recommandation**) : un hébergeur français tout-en-un à PostgreSQL managé, un hébergeur à service managé de type « plateforme », un VPS économique à PostgreSQL auto-hébergé. **PostgreSQL auto-hébergé** (décision n° 21) reste possible si le budget l'impose : il ajoute une charge d'exploitation (sauvegarde continue par un outil dédié vers le stockage objet, restauration à un instant donné outillée par l'équipe) et un risque d'erreur humaine.

### 1.3 Coûts mensuels estimés par environnement [Estimation à vérifier]

Estimations de planification, **à vérifier sur la grille tarifaire du fournisseur retenu** ; 1 € = 655,957 FCFA (parité fixe). Elles ne reposent sur aucun devis.

| Poste | Développement | Essai | Production | Remarque |
|---|---|---|---|---|
| Serveur applicatif (2 vCPU, 2 à 4 Go) | 0 (poste) | 5 à 10 € | 8 à 20 € | 4 Go si le moteur PDF est un navigateur (décision n° 7) |
| PostgreSQL managé (petite instance, restauration à un instant donné) | 0 (conteneur) | 10 à 15 € | 15 à 40 € | Neon (DP-55) : le prix dépend du plan, qui fixe aussi la fenêtre d'historique [À vérifier] ; essai sans haute disponibilité |
| Stockage objet principal (versionné) | 0 (disque) | 1 € | 1 à 3 € | quelques Go au pilote (PDF de quelques dizaines de Ko) |
| Second fournisseur de sauvegardes (stockage objet à verrouillage) | 0 | 0 | 1 à 5 € | copie logique quotidienne chiffrée |
| Nom de domaine (par mois) | 0 | 0 | 1 à 2 € | de l'ordre de 10 à 20 € par an |
| Gestionnaire de mots de passe d'équipe, sonde externe, page d'état | — | — | 0 à 10 € | offres gratuites ou d'entrée de gamme |
| Intégration continue, registre d'images | 0 | 0 à 5 € | | minutes gratuites suffisantes au pilote [Hypothèse] |
| **Total mensuel** | **0** | **16 à 31 €** | **26 à 80 €** | somme des lignes ci-dessus |
| **Ensemble** | | | **42 à 111 € par mois, soit environ 27 600 à 72 800 FCFA** | essai plus production ; source unique des coûts (`ARCHITECTURE.md` §11.1 et `DECISIONS.md` n° 86 y renvoient) |

Douze mois représentent 0,33 à 0,87 million de FCFA, soit plus que le budget de lancement du dossier (« quelques centaines de milliers de FCFA » [à confirmer]) : le budget disponible est une question ouverte (n° 18 ci-dessous).

**Coûts ponctuels du pilote** (hors tableau) :

| Poste | Coût |
|---|---|
| Téléphone Android d'entrée de gamme pour les tests | quelques dizaines d'euros [à chiffrer] |
| Avis juridique ponctuel (DP-62) | à chiffrer |
| Second avis externe sur les chemins sensibles (DP-54) | à chiffrer |
| Test d'intrusion externe (décision n° 58) | devis à demander |

Le dossier (§22.3) rappelle que le coût réel est le temps d'accompagnement, pas l'infrastructure.

## 2. Construction et livraison

### 2.1 Conteneur

- **Une image** par commit, construite par la CI : construction en plusieurs étapes (front, puis binaire Rust avec `--locked --release`), image finale **minimale**, exécutée par un utilisateur **sans droits**, système de fichiers en **lecture seule** hors dossier temporaire, aucun shell inutile. Étiquetée par le **commit** et identifiée par son **empreinte** ; jamais l'étiquette `latest` en production.
- **Même image** de l'essai à la production (`ARCHITECTURE.md` §10.5, `SECURITY.md` §7.6) : la promotion déplace l'empreinte, elle ne reconstruit rien. La configuration seule (variables d'environnement) distingue les environnements.
- **Rendu PDF dans un processus isolé** (`SECURITY.md` §7.5) : utilisateur sans droits, **sans réseau**, sans secret ni identifiants de base, système de fichiers en lecture seule, limites de temps et de mémoire. Avec un navigateur sans interface, ce processus est un conteneur ou un bac à sable séparé ; avec une bibliothèque native, un sous-processus restreint. Le choix dépend du moteur (décision n° 7).
- Analyse de l'image et signature des images **[APRÈS]** ; liste des composants (SBOM) **[APRÈS]**.

### 2.2 Chaîne de livraison continue

```text
fusion sur main → CI (TESTING_QA §10, étapes 1 et 2) → construction de l'image → publication au registre
   → déploiement AUTOMATIQUE sur l'essai (migrations puis nouvelle version) → tests de fumée
   → [promotion MANUELLE par une personne, après la liste de contrôle de TESTING_QA §11.4]
   → sauvegarde à la demande → migrations → nouvelle instance → contrôle de santé → bascule → arrêt de l'ancienne
```

- **Hébergeur du code et de la CI** (question 13 d'`ARCHITECTURE.md`, décision n° 87) : trois options. (a) une plateforme grand public de dépôt avec intégration continue intégrée ; (b) une plateforme concurrente équivalente ; (c) une forge auto-hébergée. **Recommandation : (a), sur un compte d'organisation** (jamais un compte personnel, section 3.5), avec double authentification obligatoire, **protection de la branche principale et relecture obligatoire** [à vérifier : ces protections peuvent dépendre de l'offre pour un dépôt privé], secrets de production réservés à l'étape de promotion manuelle. Une forge auto-hébergée ajouterait un service à exploiter pour une petite équipe.
- **Promotion** : une personne **autre que l'auteur** du changement déclenche la promotion quand c'est possible ; l'heure, l'empreinte déployée et la personne sont consignées (journal de déploiement, ligne `platform_events`).
- **Registre d'images** : celui de la plateforme de code ou de l'hébergeur ; accès en écriture limité à la CI.

### 2.3 Orchestration (décision n° 88)

- **Contexte** : une seule application, une base managée, un stockage objet ; un développeur ; pas de Kubernetes (dossier §12.2).
- **Options** : (a) un seul serveur avec **Docker Compose** et un serveur frontal à certificats automatiques ; (b) une plateforme gérée de conteneurs (déploiement par image, sans serveur à entretenir) ; (c) Kubernetes (écarté).
- **Choix recommandé** : (a) [À valider, décision n° 88] : serveur frontal **Caddy** (certificats automatiques, redirection HTTPS, journal configurable) devant le conteneur `immo` ; script de déploiement unique et idempotent qui prend l'**empreinte** en argument. (b) est une alternative sérieuse si l'hébergeur choisi la propose : moins de serveur à entretenir, mais dépendance accrue et rendu PDF isolé à vérifier.
- **Conséquences** : le serveur est **jetable** : il se reconstruit depuis l'image et les variables d'environnement ; aucune donnée n'y vit (base managée, objet, sauvegardes ailleurs). Pare-feu : seuls les ports 80 et 443 ouverts ; accès d'administration par clé, jamais par mot de passe, depuis des adresses connues si possible ; mises à jour de sécurité du système planifiées (jamais en zone rouge, section 8.3).
- **Journaux d'accès du proxy** : les chemins qui portent un jeton (`/s/…`, `/api/invitations/…`, `/api/auth/password-reset/…`) sont **masqués** par filtre, ou le journal d'accès est désactivé (`SECURITY.md` §7.7) ; un test de CI cherche des jetons dans les journaux d'un parcours complet.

### 2.4 Configuration et secrets par environnement (décision n° 92)

| Variable | Contenu | Où elle vit | Remarque |
|---|---|---|---|
| `APP_ENV` | `development`, `staging`, `production` | environnement du conteneur | change la sécurité des cookies, `app_now()` en essai |
| `PUBLIC_BASE_URL` | adresse publique de l'application | idem | sert aux liens de partage et à `Origin` |
| `DATABASE_URL` | rôle `immo_app` (sans `BYPASSRLS`, non propriétaire), TLS vérifié | environnement du conteneur | **un jeu par environnement** |
| `MIGRATION_DATABASE_URL` | rôle `immo_owner` | **étape de déploiement de la CI seulement**, jamais le conteneur de l'application | `SECURITY.md` §5.4 |
| `S3_ENDPOINT`, `S3_BUCKET`, `S3_REGION`, `S3_ACCESS_KEY`, `S3_SECRET_KEY` | stockage objet | environnement du conteneur | un seau par environnement |
| `SESSION_SECRET` | secret de session | idem | rotation annuelle (révoque les sessions) |
| Clé de chiffrement des clés applicatives | chiffrement de `payout_details_enc`, `totp_secret_enc`, `id_document_enc` | idem, **copie hors ligne** | sauvegardée à part en deux exemplaires ; sa perte rend ces champs illisibles |
| Variables de sauvegarde | voir section 5 | **tâche de sauvegarde seulement** | pas dans le conteneur applicatif |

- **Où vivent les secrets de production** : fichier d'environnement protégé sur le serveur (droits restreints, hors dépôt) **ou** variables de la plateforme gérée ; **pas de coffre dédié au pilote** (un coffre de plus à exploiter). Les copies de référence, et celles de récupération, sont dans le gestionnaire de mots de passe de l'équipe (section 3.5). Rotation : annuelle, et **le jour du départ** d'une personne qui y avait accès (`SECURITY.md` §5.4).
- **Écart signalé** : `CLAUDE.md` liste `BACKUP_ENCRYPTION_KEY` parmi les variables d'environnement, alors que `SECURITY.md` §5.3 exige que cette clé soit **hors de l'hébergeur**. La section 5.2 propose un chiffrement **à clé publique** : le serveur n'a que la clé publique et ne peut donc pas relire ses propres sauvegardes (décision n° 95).

### 2.5 Migrations de base de données pendant une livraison

Règles de `DATABASE_SCHEMA.md` §7 et d'`ARCHITECTURE.md` §8.1, appliquées ainsi :

1. **Avant toute migration en production** : sauvegarde à la demande (instantané du service managé) ; les **cinq requêtes de contrôle** de `DATABASE_SCHEMA.md` §7.5 exécutées (zéro anomalie ; totaux par organisation notés) ; heure et numéro de migration consignés dans `platform_events`.
2. **Les migrations tournent avant le démarrage de la nouvelle version**, avec `immo_owner`, depuis la CI (jamais depuis le serveur applicatif), en **avant seulement** (aucune migration appliquée n'est modifiée), avec `SET lock_timeout = '5s'` : une migration qui attend un verrou plus de 5 secondes échoue au lieu de bloquer les encaissements.
3. **Rétrocompatibles** : l'ancienne version du binaire doit fonctionner avec le nouveau schéma (étape « étendre »). C'est ce qui rend le retour arrière possible sans toucher à la base.
4. **Étendre, migrer, contracter** pour tout changement non additif : ajouter ; écrire dans les deux et remplir l'historique par lots de 1 000 lignes avec pause ; lire le nouveau ; retirer l'ancien **au moins une semaine plus tard**. Jamais de renommage ni de suppression d'une colonne lue en production en une étape.
5. **Une migration de données incompatible** est livrée **seule**, après sauvegarde et vérification complète sur une copie restaurée, avec une **procédure inverse écrite** et jouée une fois sur copie.
6. **Une migration qui touche une garde** (déclencheur d'écriture seule, politique d'isolation, contrainte d'exclusion) reçoit un second avis externe (`CODING_RULES.md` §8.3) ; le contrôle nocturne vérifie ensuite que les gardes sont actives.
7. **Après** : les contrôles du point 1 rejoués ; totaux inchangés ; test de fumée ; alertes surveillées une heure.
8. **Pas de migration** pendant les zones rouge et orange du calendrier (section 8.3), sauf correctif S1.

### 2.6 Mise en production sans interruption, quand c'est possible

- **Au pilote** (`ARCHITECTURE.md` §10.5) : remplacement du conteneur avec une coupure de **quelques secondes** (5 à 10 s [Hypothèse]) **hors heures d'encaissement**. Côté utilisateur : « Connexion perdue. Vos informations sont conservées. Réessayez… » ; la **clé d'idempotence** rend le nouvel essai sans risque (R-042). Contrôle de santé obligatoire avant la bascule du proxy ; si le contrôle échoue, l'ancienne instance reste servie.
- **[APRÈS]** : deux instances derrière le proxy, remplacées l'une après l'autre ; un travailleur de fond séparé par option de démarrage quand le rendu PDF gêne l'API. Les migrations rétrocompatibles le permettent déjà.
- Pas de mise en production sans la liste de contrôle de `TESTING_QA.md` §11.4.

### 2.7 Retour arrière

| Situation | Action | Durée cible |
|---|---|---|
| Défaut de la nouvelle version, schéma additif | **redéployer l'image précédente** (empreinte connue, journal de déploiement) ; le schéma reste en place | 10 minutes ou moins [Hypothèse] |
| Défaut après étape « contracter » | pas de retour arrière du schéma : correctif en avant, après sauvegarde | selon gravité |
| Migration de données incompatible | procédure inverse écrite ; sinon restauration (section 5.4) | selon la procédure |
| Aucun défaut visible mais données suspectes | geler les livraisons ; cinq requêtes de contrôle ; comparer à la dernière sauvegarde ; **ne pas** corriger à la main (R-005, immutabilité) ; suivre `SECURITY.md` §9 | |

Le retour arrière est **exercé une fois sur l'essai avant le pilote** (PP-22). Arbre de décision : *une donnée d'argent est-elle touchée ?* oui → incident S1 (gel, `SECURITY.md` §9) ; non → redéployer l'image précédente, puis rédiger le défaut.

### 2.8 Interrupteurs de fonctionnalités : livrer à une seule agence d'abord

Mécanisme d'`ARCHITECTURE.md` §8.2 (table `org_features`, lue côté serveur, modifiée **seulement** par la console, auditée `feature.toggled`).

1. La fonction est fusionnée **éteinte** (valeur par défaut).
2. Allumée sur l'**essai** et l'organisation de **démonstration** ; vérifiée par la personne qui n'a pas écrit le code.
3. Allumée pour **une seule agence pilote**, choisie à l'avance (celle qui a demandé la fonction ou la plus tolérante), **hors zone rouge**, avec information préalable.
4. Observation **de 2 à 3 jours** : événements `feature_used`, erreurs, signalements, retour de l'agence (point hebdomadaire).
5. Allumée pour toutes ; l'interrupteur est retiré du code à la date prévue.

Règles : au plus **10** interrupteurs actifs, chacun avec une date de retrait ; **un interrupteur ne change jamais une règle d'argent pour une seule organisation** (règle d'or 12) ; l'interrupteur est distinct du **plan** (R-082). Une fonction qui écrit de l'argent (F19 clôture de caisse, F25 lecture du SMS) n'est allumée qu'après ses tests d'or. Un interrupteur est **éteignable en minutes** : c'est le premier réflexe de confinement d'une fonction défaillante, avant le retour arrière.

## 3. Domaine, DNS, certificats et gouvernance des accès

### 3.1 Noms, adresses (décision n° 90)

Le nom de domaine n'est pas choisi : la décision n° 13 (nom et marque) est ouverte et la maquette porte le bandeau « nom provisoire ». `<domaine>` ci-dessous est un **espace réservé**.

| Adresse | Rôle | Hébergée | Remarque |
|---|---|---|---|
| `app.<domaine>` | application de production et organisation de démonstration | production | cookie `__Host-session` : sans attribut `Domain`, donc **propre à cet hôte** |
| `essai.<domaine>` | environnement d'essai | essai | accès restreint ; cookie propre ; jamais d'indexation |
| `console.<domaine>` | console HABEK | production | **nom d'hôte distinct**, cookie distinct, session plus courte, double authentification (`SECURITY.md` §3.7) ; restriction par adresse IP envisagée |
| `www.<domaine>` | page de présentation (le dossier `site/`) | **hors du serveur applicatif** (hébergement statique) | aucun cookie sur le domaine de l'application ; aucune donnée réelle |
| `status.<domaine>` | page d'état (section 7.5) | **hors du serveur principal** | doit rester joignable quand la production tombe |

- **Registrar** : compte de **l'organisation** (HABEK), jamais un compte personnel ; double authentification ; au moins deux administrateurs (section 3.5) ; renouvellement automatique avec alerte d'expiration ; **verrouillage du domaine** (transfert protégé) activé.
- **DNS** : fournisseur avec double authentification et journal des modifications ; TTL bas (300 s) pendant une migration d'hébergement, normal ensuite ; **DNSSEC** si le registrar le propose ; enregistrement **CAA** limitant les autorités de certification ; chaque modification DNS est consignée.
- **Courriel** : aucun e-mail n'est envoyé au pilote (décision n° 27). Le domaine publie quand même des enregistrements qui **interdisent l'usurpation** (SPF « aucun expéditeur », DMARC en rejet) tant qu'aucun service d'envoi n'existe, pour éviter l'hameçonnage au nom de l'agence (`SECURITY.md` §7.8). Il faut en revanche une **boîte de réception organisationnelle** pour recevoir les alertes du registrar et de l'hébergeur (section 3.5).
- **Hameçonnage des liens de partage** : message et domaine **constants** ; les agences annoncent à leurs locataires l'adresse officielle (`SECURITY.md` §7.8).

### 3.2 Certificats

- **Certificats automatiques** (autorité gratuite à renouvellement automatique) gérés par le proxy ; TLS 1.2 au minimum, 1.3 de préférence ; HSTS avec `includeSubDomains`. **Pas de « preload »** au pilote (irréversible).
- Le **renouvellement est surveillé** : la sonde externe vérifie la date d'expiration ; **alerte à moins de 14 jours** de l'échéance ; procédure de renouvellement manuel écrite.
- Connexion à la base et au stockage objet en TLS avec vérification (`verify-full`) ; aucun trafic de production en clair, y compris entre conteneurs sur un réseau partagé.

### 3.3 Gouvernance des accès : qui détient quoi (à proposer en question ouverte)

Aucun compte unique et personnel ne doit être un point de défaillance. **Proposition** (à valider, voir « Questions ouvertes ») : chaque ressource critique a un **compte d'organisation**, au moins **deux administrateurs humains**, une récupération documentée, et un inventaire daté.

| Ressource | Titulaire proposé | Administrateurs | Protection | Récupération |
|---|---|---|---|---|
| Nom de domaine (registrar) | HABEK (personne morale) | le développeur + le porteur du projet | double authentification, verrou de transfert | codes de secours scellés, contact de l'organisation |
| DNS | idem | idem | double authentification, journal des changements | idem |
| Hébergeur (compte cloud, base managée, stockage objet) | idem | le développeur ; facturation : porteur du projet | double authentification, rôles distincts (administrateur, lecture, facturation) | codes de secours, contact facturation |
| Second fournisseur de sauvegardes | idem, **compte distinct** de l'hébergeur principal | le développeur et le porteur du projet | double authentification ; identifiants du travail de sauvegarde en **écriture seule** | idem |
| Dépôt de code, CI, registre d'images | organisation de la plateforme (pas un compte personnel) | le développeur (propriétaire) + le porteur du projet (propriétaire de secours) | double authentification obligatoire pour tous | idem |
| Gestionnaire de mots de passe de l'équipe | HABEK | le développeur ; **accès d'urgence** pour le porteur du projet | double authentification ; coffre partagé | procédure d'urgence documentée |
| Clés de chiffrement des sauvegardes et des champs | deux exemplaires, **détenus par le développeur et le porteur du projet**, hors de l'hébergeur | le développeur et le porteur du projet | copie hors ligne | exercice trimestriel (`SECURITY.md` §9.4) |
| Sonde externe, page d'état, canal d'alerte | HABEK | le développeur et le porteur du projet | double authentification, aucune donnée personnelle | idem |
| Comptes de la console HABEK | nominatifs | — | TOTP obligatoire, aucun compte partagé | réinitialisation par le développeur et le porteur du projet |

Principes : double authentification **sur chaque** compte ; deux appareils d'authentification par personne ; codes de secours imprimés et conservés à **deux endroits physiques distincts** ; boîtes de réception **de rôle** (par exemple `ops@`, `facturation@`) redirigées vers au moins deux personnes ; cartes de paiement professionnelles avec alertes d'expiration ; **revue trimestrielle** de l'inventaire ; **exercice « personne indisponible »** chaque trimestre (le porteur du projet, guidé par les procédures écrites, ouvre les accès d'urgence et lance une restauration) ; **départ d'une personne** : désactivation le jour même, rotation des secrets auxquels elle avait accès, revue des accès de l'hébergeur (`SECURITY.md` §5.6).

## 4. Calendrier de livraison pendant le pilote

Fréquence visée : **plusieurs livraisons par semaine** (`ARCHITECTURE.md` §10.5), sans jamais compromettre les encaissements. Cette section est la **source unique** du calendrier de livraison : `ARCHITECTURE.md` §10.5 et `TESTING_QA.md` §11.4 y renvoient. Valeurs proposées [Hypothèse], à **recalibrer après le premier mois** avec la mesure réelle des encaissements par heure (`ARCHITECTURE.md` §10.2) et avec les agences (décision n° 91).

| Période | Règle | Pourquoi |
|---|---|---|
| **Zone rouge : jours 1 à 7 du mois** | **gel** : seulement un correctif prioritaire S1 ou S2, après appel à l'agence concernée | pic d'encaissement des loyers ; production des comptes rendus du mois précédent ; cadrage §4, §14.1 (« pics en début de mois ») |
| **Zone orange : jours 8 à 10 et 25 à 31** | fonctions et corrections de confort seulement ; **aucune migration de données, aucun changement d'un calcul d'argent** | clôtures et comptes rendus de fin de mois ; premières relances |
| **Zone verte : jours 11 à 24** | livraisons normales, migrations rétrocompatibles comprises | |

| Jour et heure | Règle |
|---|---|
| Jours | **mardi à jeudi** pour toute livraison planifiée ; **jamais le vendredi après-midi**, le week-end, ni la **veille d'un jour férié** (calendrier burkinabè à tenir à jour ; fêtes mobiles à surveiller) |
| Heure | avant l'ouverture des agences, **de 06 h 00 à 07 h 30 (heure locale, UTC+0)** [Hypothèse : à confirmer avec l'agence], ou en milieu de journée creuse si l'agence l'indique |
| Correctif S1 ou S2 | à tout moment, **avec appel préalable** à l'agence touchée quand l'encaissement peut être interrompu |
| Bascule `setup` → `live` d'une agence | hors zone rouge, hors vendredi, avec le développeur disponible, le porteur du projet joignable et le gérant présent |
| Après une bascule | **deux semaines de calme** : correctifs seulement pour cette agence |
| Test de restauration mensuel | 3ᵉ semaine du mois, jamais en zone rouge |

Chaque livraison est annoncée la veille aux agences pilotes **si elle est visible ou si une coupure est prévue** (section 8). Le calendrier est affiché dans le dépôt et dans le journal de déploiement.

## 5. Sauvegardes

Objectifs d'`ARCHITECTURE.md` §10.4 et de `SECURITY.md` §5.3, **reprises telles quelles** [À valider, décision n° 24], **sauf les deux objectifs suspendus par DP-69 (plan gratuit de Neon)** : la restauration à un instant donné est de **6 heures** (et non 30 jours) et la perte maximale est de **24 heures** si l'hébergeur de la base est perdu (une copie logique par jour), l'objectif de 15 minutes étant suspendu ; restent : reprise **4 heures** en journée ouvrée, **30 copies quotidiennes et 12 copies mensuelles**, **test de restauration chaque mois**, fait à partir d'une copie logique. Bascule `live` : risque de perte entre deux copies et suspension du calcul en fin de quota à faire accepter par écrit par Mimis (DP-69).

### 5.1 Les trois couches

| Couche | Contenu | Fréquence | Conservation | Où |
|---|---|---|---|---|
| **1. Sauvegarde continue de la base** | journal des transactions archivé + instantanés du service managé | continue ; plan gratuit : **6 heures** d'historique, sans engagement de perte maximale (DP-69) | **6 heures** en restauration à un instant donné (l'objectif de 30 jours est suspendu) | chez l'hébergeur principal |
| **2. Copie logique quotidienne chiffrée** | `pg_dump` au format personnalisé, avec un rôle de sauvegarde (décision n° 46), **chiffrée avant envoi** | une fois par jour, hors zone d'encaissement (par exemple 02 h 00) | 30 copies quotidiennes + **12 copies mensuelles** | **autre fournisseur**, identifiants en **écriture seule**, verrouillage d'objet ou versionnage |
| **3. Réplication du seau de fichiers** | PDF de quittances, comptes rendus, logos, documents | quotidienne | alignée sur la couche 2 | autre fournisseur |

- La couche 1 couvre l'erreur humaine récente et la panne ; la couche 2 couvre la **perte de l'hébergeur** (la copie doit rester lisible sans lui) ; la couche 3 couvre la perte de PDF : un PDF émis **ne se régénère pas** (le fichier original fait foi, DP-71), sa perte est une perte de preuve ; réplication quotidienne, versionnage et contrôle nocturne de l'empreinte (`ARCHITECTURE.md` §3.10).
- **Avant chaque migration en production** : sauvegarde à la demande en plus (section 2.5).
- **Rôle de sauvegarde** : `pg_dump` exige un rôle qui contourne la sécurité par ligne forcée ; il n'est utilisé que par le travail de sauvegarde, jamais par l'application. **Vérifié sur Neon le 31/10/2026 (prototype b, DP-78)** : un rôle `immo_backup` avec `BYPASSRLS`, créé par SQL, réussit `pg_dump -Fc` sous RLS forcée et `pg_restore` vers un autre projet (code 0, 18,4 s, comptes identiques) ; la RLS reste forcée. Chaque table nouvelle reçoit `GRANT SELECT … TO immo_backup` (sans lui, `pg_dump` échoue) et tout rôle cité par une politique doit exister avant une restauration. Une restauration à un instant donné sur une branche autre que celle par défaut passe par l'API de Neon (le CLI ne combine pas branche et instant).
- **Alertes** : copie absente depuis plus de 24 heures ; copie de taille anormale (inférieure à 50 % de la précédente [Hypothèse]) ; échec de chiffrement ou d'envoi ; verrouillage d'objet désactivé.

### 5.2 Chiffrement (décisions n° 53 et 95)

- **Avant envoi**, par l'équipe ; la clé n'est **jamais** stockée avec les sauvegardes ni chez l'hébergeur principal ; elle existe en **deux exemplaires**, détenus par le développeur et le porteur du projet (`SECURITY.md` §5.3).
- **Proposition** [À valider, décision n° 95] : **chiffrement à clé publique** (un outil de chiffrement moderne à destinataire public). Le serveur et le travail de sauvegarde ne portent que la **clé publique** ; la **clé privée** reste hors ligne, en deux exemplaires, et ne sert qu'à restaurer. Un attaquant qui prend le serveur ne peut alors pas relire les sauvegardes, même avec les identifiants d'envoi. Alternative : clé symétrique (plus simple, mais la clé est présente sur le serveur au moment du chiffrement) ; c'est ce que suppose la variable `BACKUP_ENCRYPTION_KEY` de `CLAUDE.md`.
- La clé de chiffrement des clés applicatives (qui protège les champs sensibles) est sauvegardée **à part**, en deux exemplaires : sans elle, les champs `*_enc` d'une restauration sont illisibles.
- **Intégrité** : chaque copie est accompagnée d'un manifeste (nom, taille, empreinte SHA-256, version du schéma, date) ; le manifeste est vérifié à la restauration.

### 5.3 Droits

- Identifiants du travail d'envoi : **écriture seule** (ni lecture, ni suppression, ni écrasement) ; **verrouillage d'objet** ou versionnage pour qu'un attaquant du serveur ne puisse pas effacer les copies. Si le second fournisseur ne le propose pas : copie hors ligne mensuelle (`SECURITY.md` §5.3) [À vérifier].
- Lecture des sauvegardes réservée au **développeur et au porteur du projet** ; chaque restauration est consignée (qui, quand, pourquoi) dans `platform_events`.

### 5.4 Restauration complète, pas à pas

À jouer **une fois avant le pilote** (PP-14), puis **chaque mois** (5.6). On restaure sur une **instance neuve**, jamais sur la production. Durée cible : **moins de 4 heures** ; on chronomètre **chaque étape**.

1. **Décider et prévenir** : incident ou test ? Pour un incident réel : gel des mises en production, information de l'agence dans l'heure (`SECURITY.md` §9). Noter l'heure de début.
2. **Choisir le point de restauration** : le dernier instant sain (couche 1) ou la dernière copie quotidienne (couche 2). Noter l'heure choisie et la perte de données correspondante.
3. **Retrouver la clé de déchiffrement** (clé privée hors ligne, deuxième personne présente pour un incident réel).
4. **Créer une instance PostgreSQL neuve**, de **même version majeure** et de **mêmes extensions** (`btree_gist`) que la production, avec les **mêmes rôles** (`immo_owner`, `immo_app`, `immo_platform`, rôle de sauvegarde) et le même rôle propriétaire des fonctions de garde (un test de schéma vérifie que le propriétaire des fonctions `SECURITY DEFINER` est celui que les gardes attendent, `DATABASE_SCHEMA.md` §4.2).
5. **Restaurer** : soit la restauration à un instant donné du service managé sur la nouvelle instance, soit le déchiffrement puis `pg_restore` de la copie logique. Une restauration complète charge les données avant de recréer les déclencheurs : les gardes ne se déclenchent pas (`DATABASE_SCHEMA.md` §8.4).
6. **Restaurer le stockage objet** (réplique de la couche 3) dans un seau neuf, ou pointer vers la réplique ; vérifier quelques PDF.
7. **Déployer l'application** (image de la production, **même empreinte**) sur un serveur neuf avec une configuration neuve : nouveaux secrets de session, mêmes clés de chiffrement des champs.
8. **Contrôles de cohérence**, sur l'instance restaurée : comptage des lignes de chaque table métier comparé à la production ; somme des paiements et des affectations **par organisation** ; les **cinq requêtes de contrôle** de `DATABASE_SCHEMA.md` §7.5 (aucun trou de numérotation, aucune affectation hors bornes, contre-passations exactes, gardes d'écriture seule présentes et actives, totaux) ; audit de l'isolation (RLS activée et forcée sur toutes les tables, rôle applicatif sans `BYPASSRLS`) ; connexion d'un utilisateur de test et lecture d'un PDF.
9. **Basculer** (incident réel) : mettre l'ancienne production hors service, faire pointer le DNS ou le proxy vers la nouvelle instance, **révoquer toutes les sessions** (nouveau `SESSION_SECRET`), tester un encaissement et son retour arrière en organisation de démonstration.
10. **Vérifier la numérotation** : le dernier numéro de chaque organisation est cohérent avec la dernière quittance remise ; si des quittances remises à des locataires ont disparu par la perte de données, **elles sont listées** et traitées avec l'agence (nouvelle émission contrôlée par un gérant, jamais d'`INSERT` manuel de numéro).
11. **Consigner** (compte rendu, 5.6), calculer la durée réelle de chaque étape et la comparer aux 4 heures.
12. **Après** : retour d'expérience écrit sans recherche de coupable dans les 5 jours (`SECURITY.md` §9) ; correction ajoutée à la liste de contrôle.

**Restauration d'une seule organisation** : depuis la copie logique restaurée sur une instance temporaire, extraction filtrée par `org_id`, puis réinjection par la **procédure d'exception** de `DATABASE_SCHEMA.md` §8.4 (organisation suspendue, gardes nécessaires désactivées puis réactivées immédiatement, contrôles, trace, seconde personne). Procédure **manuelle** écrite et répétée une fois avant le pilote [PILOTE] ; outillage [APRÈS]. Jamais par l'application ni avec `immo_app`.

### 5.5 Surveillance de la sauvegarde

Chaque nuit : la dernière copie existe et est lisible (taille, empreinte, manifeste) ; le verrouillage d'objet est actif ; les 25 déclencheurs d'écriture seule sont actifs ; le contrôle de trous de numérotation est à zéro. Une défaillance déclenche l'alerte « sauvegarde manquante » (section 7.3). **Une sauvegarde jamais restaurée n'existe pas** : aucune copie n'est considérée valide avant d'avoir été restaurée une fois.

### 5.6 Test de restauration mensuel et compte rendu

- **Qui** : le développeur, observé par le porteur du projet ; **quand** : 3ᵉ semaine du mois (section 4) ; **où** : instance neuve de l'environnement d'essai ou instance temporaire, avec les mêmes versions et le même rôle que la production.
- **Exercice trimestriel** : perte de la clé de chiffrement (la seconde personne retrouve la clé et relit une copie) et exercice de réponse à incident sur table (`SECURITY.md` §9.4).

```markdown
# Compte rendu du test de restauration — {JJ/MM/AAAA}
- Réalisé par :   Observateur :   Source restaurée : [instant donné | copie quotidienne du JJ/MM] [production]
- Instance cible (version, hébergeur) :   Rôles et extensions identiques à la production : oui / non
## Chronométrage (hh:mm)
| Étape | Début | Fin | Durée |
| Retrouver la clé | | | |
| Créer l'instance | | | |
| Restaurer la base | | | |
| Restaurer les fichiers | | | |
| Contrôles de cohérence | | | |
| **Total** (cible : moins de 4 h) | | | |
## Résultats
- Lignes par table : écarts avec la production : aucun / liste :
- Somme des paiements et des affectations par organisation : identiques : oui / non
- Cinq requêtes de contrôle : zéro anomalie : oui / non
- RLS activée et forcée sur toutes les tables, rôle applicatif sans BYPASSRLS : oui / non
- Lecture de PDF restaurés : n documents contrôlés, tous lisibles : oui / non
## Perte de données mesurée (RPO) :   Reprise mesurée (RTO) :
## Anomalies, décisions et corrections (avec échéance et responsable) :
## Prochaine échéance de test :
```

Le compte rendu est conservé dans le dépôt des procédures. **Un test non fait ou raté est une alerte** (section 7.3) : le mois suivant ne se déclare pas « normal ».

## 6. Exploitation courante

### 6.1 Procédure de réponse à un incident

Le plan de `SECURITY.md` §9 fait foi ; ce qui suit en donne la **version d'exploitation**.

- **Rôles** : un **commandant d'incident** (celui qui décide), un **scribe** (journal d'incident : chaque action avec l'heure), le **porteur du projet** comme contact client.
- **Les 15 premières minutes** : (1) qualifier la gravité S1 à S3 (`SECURITY.md` §9.2, `TESTING_QA.md` §11.1) ; (2) **geler les mises en production** ; (3) **confiner** : éteindre l'interrupteur de la fonction, couper la route, révoquer les sessions, révoquer un accès support, retour arrière de l'image ; au besoin couper le service plutôt que laisser fuir ; (4) noter les organisations touchées ; (5) ouvrir le journal d'incident.
- **Information de l'agence** : **dans l'heure** pour une interruption de l'encaissement ; **dans les 24 heures** après confirmation d'une fuite de données personnelles (délais proposés, `SECURITY.md` §9.3). Par **appel puis message écrit** ; **jamais par un groupe WhatsApp**.
- **Correction, restauration, vérification** (cinq requêtes de contrôle, totaux, numérotation, test d'isolation) avant la réouverture ; retour d'expérience en 5 jours.
- **Incidents types** et premières mesures : tableau de `SECURITY.md` §9.2 (fuite entre organisations, compte volé, paiement falsifié, secret exposé, perte du serveur, abus du support, fichier d'import piégé).

Modèle de **message à l'agence** :

```text
Objet : Immo Suite — {incident en deux mots} — {JJ/MM/AAAA} {hh:mm}
1. Ce qui s'est passé (mots simples) :
2. Ce qui est touché (encaissements, quittances, données) — et ce qui ne l'est pas :
3. Ce que nous avons fait et ce que nous faisons maintenant :
4. Ce que vous devez faire (par exemple : continuer au carnet, changer un mot de passe, prévenir un locataire) :
5. Prochain point à {hh:mm}. Contact direct : {nom, numéro}
```

### 6.2 Procédure de mise en maintenance

À utiliser pour une migration de données incompatible, une restauration, un remplacement d'hébergeur.

1. **Planifier** hors zone rouge (section 4), avec l'accord de l'agence ; durée estimée **doublée** dans l'annonce.
2. **Annoncer** au moins **24 heures** avant : appel au gérant puis message écrit ; mise à jour de la page d'état ; consigne de repli (« continuez au carnet, saisissez ensuite ; les quittances de la période seront émises à la reprise » : à valider avec l'agence).
3. **Sauvegarde à la demande** ; cinq requêtes de contrôle avant.
4. **Activer le mode maintenance** : l'API répond `503 server.maintenance` avec `Retry-After` ; le front affiche « Maintenance en cours. Retour prévu à {heure}. ». Un **mode lecture seule** (consultation permise, écritures refusées) est préférable à une coupure totale quand la migration le permet [APRÈS, à spécifier].
5. **Exécuter** ; **vérifier** (cinq requêtes, totaux inchangés, test de fumée d'un encaissement en organisation de démonstration, test d'isolation).
6. **Rouvrir**, mettre à jour la page d'état, **prévenir** l'agence, surveiller les alertes pendant une heure, noter dans le journal de déploiement.
7. Si la durée dépasse l'annonce de plus de 50 %, **appeler** l'agence : ne pas laisser le silence.

### 6.3 Communication aux clients

| Quand | Quoi | Canal | Délai |
|---|---|---|---|
| Changement visible ou coupure prévue | ce qui change, quand, ce que l'agence doit faire | message écrit au gérant (et au référent) | la veille |
| Maintenance planifiée | section 6.2 | appel puis message ; page d'état | 24 heures |
| Interruption de l'encaissement | modèle de 6.1 | **appel** puis message | dans l'heure |
| Fuite de données personnelles confirmée | modèle de 6.1 + texte pour les locataires et mandants fourni par l'équipe | appel puis message | dans les 24 heures |
| Fonction livrée | ce que la personne peut faire de nouveau (une phrase) | message court | après l'allumage de l'interrupteur |
| Point hebdomadaire | 30 minutes avec chaque agence pilote : usage, anomalies, alertes mineures, demandes | appel ou visite | chaque semaine (dossier §20.2) |

Pas de groupe WhatsApp pour les incidents ; un contact nominatif unique par agence.

## 7. Supervision

### 7.1 Journaux

- `tracing` en JSON sur la sortie standard, collecté par l'hébergeur ou par un fichier tourné ; **liste blanche** de champs (identifiant de requête, modèle de route, statut, durée, `org_id`, `user_id`, version) ; **aucune donnée personnelle**, aucun corps de requête, aucun jeton (`CODING_RULES.md` §4.5).
- **Conservation 30 jours**, accès réservé au développeur et au porteur du projet. Les journaux du proxy et de l'hébergeur sont soumis à la même règle (section 2.3).

### 7.2 Métriques minimales

Calculées depuis la base et les journaux, **sans système de métriques séparé au pilote** ; point `/internal/metrics` [APRÈS] (`ARCHITECTURE.md` §10.2) : requêtes par route (nombre, taux de 5xx, durée p50 et p95) ; encaissements par heure (pics de début de mois) et p95 de l'enregistrement d'un paiement ; file de tâches (en attente, en échec, âge de la plus ancienne ; **quittances sans PDF depuis plus de 10 minutes**) ; base (connexions, taille, requêtes lentes de plus de 500 ms) ; **âge de la dernière sauvegarde et de la dernière restauration testée** ; disque, mémoire, processeur.

### 7.3 Alertes (décision n° 89)

Canal **unique** vers le développeur et le porteur du projet, **sans donnée personnelle dans le message** : e-mail d'équipe et application de messagerie [À valider, décision n° 89]. **Qui est prévenu** : le développeur ; **accusé de réception** attendu en 30 minutes en heures de couverture ; sans accusé, l'alerte est répétée puis le **porteur du projet** est appelé. Heures de couverture : décision n° 84, ouverte (question 5).

| Alerte | Seuil | Gravité | Premier réflexe |
|---|---|---|---|
| Service injoignable (sonde externe toutes les minutes) | 2 échecs consécutifs | S1 ou S2 | contrôle de santé, journaux, retour arrière |
| Taux d'erreurs 5xx | supérieur à 2 % sur 5 minutes | S2 | dernière livraison ? retour arrière |
| Quittance sans PDF | plus de 10 minutes | **S2** | file de tâches, moteur PDF |
| Tâche en échec définitif | dès la première | S2 | journal de la tâche |
| Génération des échéances non faite | pas de succès avant 02 h 00, heure de l'organisation | S2 | relancer la tâche (idempotente) |
| Sauvegarde manquante | plus de 24 heures | S2 | section 5.5 |
| Test de restauration en retard | plus de 35 jours | S3 | planifier |
| **Trou de numérotation détecté** | dès le premier | **S1** | geler, comparer à la sauvegarde |
| **Tentative d'accès inter-organisations** (`security.cross_org_attempt`) | dès la première | **S1** (bogue ou attaque) | `SECURITY.md` §9.2 |
| Garde d'écriture seule désactivée | dès la détection | **S1** | réactiver, enquêter |
| Disque | plus de 80 % | S3 | nettoyer, agrandir |
| Certificat | expiration dans moins de 14 jours | S3 | renouvellement manuel |
| Connexions à la base | plus de 80 % du pool | S3 | pool, requêtes lentes |
| Échec d'une livraison | dès l'échec | S2 | journal de déploiement, retour arrière |
| Signalement d'un pilote | chaque signalement | à qualifier | `TESTING_QA.md` §13.6 |

La sonde externe appelle `/health` et ne reçoit **aucune donnée** (`API_CONTRACTS.md` §7.17).

### 7.4 Tableau de bord interne

Au pilote : une page de la console qui exécute des requêtes SQL fixes sur des vues agrégées : activité par jour, paiements saisis et part sous 48 h, médiane du temps pour encaisser, quittances en échec, imports et erreurs, signalements ouverts, écrans les plus et les moins utilisés (`ARCHITECTURE.md` §8.4). Organisations `demo` et `test` exclues. Pas d'outil de visualisation dédié.

### 7.5 Page d'état simple pour les clients (décision n° 93)

- **Options** : (a) page statique hébergée **hors du serveur principal**, mise à jour par l'équipe (à la main ou par script depuis la sonde) ; (b) service de page d'état tiers ; (c) rien (écarté : le silence en incident coûte la confiance des agences).
- **Choix recommandé** : (a) [À valider, décision n° 93]. Trois états : « Tout fonctionne », « Dégradé » (une fonction ralentie ou indisponible), « Interruption ». Date et heure de mise à jour, **maintenances planifiées**, un numéro de contact. **Aucune donnée personnelle, aucun nom d'agence.** Elle doit rester joignable quand la production est tombée (autre hébergeur, autre domaine d'erreur).
- Elle est un **complément** de l'appel et du message, pas un remplacement (section 6.3).

## 8. Procédures par agence pilote

### 8.1 Accueil d'une agence pilote

Dossier de cadrage §20.2 (étapes 1 à 10) et `ARCHITECTURE.md` §8.6, rendus opérationnels. Prérequis **[PILOTE]**, tous consignés au dossier de l'agence :

- lettre d'intention signée et référent désigné ;
- **engagement de confidentialité signé** (`SECURITY.md` §8.6, relu par un juriste ou risque accepté par écrit) ;
- accord écrit sur les **données de test anonymisées** (n° 15) et sur le **droit de purge en phase de reprise** (n° 25) ;
- position tranchée sur l'hébergement hors du pays (Neon, DP-55) et la CIL, ou risque accepté par écrit (DP-62) ;
- l'agence a reçu la page des bons usages (comptes nominatifs, téléphone perdu, lien de partage).

| # | Étape | Contenu | Qui | Durée |
|---|---|---|---|---|
| 1 | **Création de l'organisation** | par la **console** (l'inscription publique est fermée, DP-68) : nom, plan, préfixe de quittance (3 ou 4 lettres, DP-37), ville ; état `setup`, type `real` ; création du premier gérant et **lien d'invitation** transmis en main propre ou par l'agence elle-même (WhatsApp) | développeur | 30 min |
| 2 | **Découverte** | atelier de 2 heures : organisation, flux d'argent, vocabulaire, règles de commission, jeux de données ; état actuel (carnet, Excel) | développeur + porteur | 2 h |
| 3 | **Configuration** | logo, en-tête de quittance, signature ou cachet, modèles de messages, rôles et droits réglables, seuil de confirmation, durée des liens | gérant avec le développeur | 1 h |
| 4 | **Import** | fichier Excel nettoyé avec l'agence ; accord de **support** (portée `import_assist`, durée courte) ; import guidé ; **contrôle ligne à ligne sur un échantillon** (par exemple 20 locataires) ; rapprochement : nombre de baux, somme des loyers mensuels, arriérés totaux comparés aux chiffres de l'agence | développeur + gestionnaire | 1 à 3 jours |
| 5 | **Invitations** | un lien par utilisateur (7 jours) ; mot de passe choisi par la personne ; **TOTP du gérant** activé (obligatoire avant `live`, DP-68) ; comptes **nominatifs** | gérant | 1 h |
| 6 | **Vérification** | connexion de chaque rôle : il ne voit que ce que la matrice permet ; isolation (aucune donnée d'une autre organisation, y compris la démonstration) ; quittance d'essai imprimée **A4 et A5** et **validée par l'agence** (mise en page, mentions) ; encaissement, contre-passation et quittance rectificative **en série d'essai** ; sauvegarde contenant l'organisation (comptage) ; alertes actives | développeur + gérant | 2 h |
| 7 | **Formation courte** | demi-journée sur place (caissier, gestionnaire, gérant) avec scénarios réels ; guide d'une page ; vidéo de 5 minutes | développeur | ½ jour |
| 8 | **Double tenue** | deux semaines **en parallèle de l'ancien système** (dossier §20.2) ; quittances en série d'essai (préfixe `ESSAI`, filigrane) ; point hebdomadaire de 30 minutes | tous | 2 semaines |
| 9 | **Bascule en `live`** | **critères** : rapprochement sans écart (ou écart expliqué) ; TOTP du gérant actif ; accord écrit ; date **hors zone rouge** et hors vendredi ; la numérotation **officielle** démarre à 000001 ; la **purge n'est plus possible** après ce point ; arrêt du carnet pour les nouveaux paiements | développeur + gérant | 1 h |
| 10 | **Premier suivi** | **J+1** (appel : tout va ?), **J+7** (point hebdomadaire : usage, anomalies), **J+30** (premier compte rendu mensuel comparé au calcul manuel de l'agence), **J+60** et **J+90** (dossier §20.2, §23) | porteur + développeur | |

- **Mesures de départ à relever avant la mise en route** (dossier §23) : durée de la clôture mensuelle, taux d'impayés à J+30, nombre de litiges par trimestre ; à refaire à J+60.
- **Phase `setup`** : les quittances d'essai ne consomment pas la numérotation officielle (séries distinctes, `DATABASE_SCHEMA.md` §4.3) ; **la remise à zéro** (`immo org purge`) n'est possible qu'en `setup`, avec accord écrit, sauvegarde préalable, le développeur et le porteur du projet, tracée hors de l'organisation ; après `live`, repartir de zéro = **nouvelle organisation**, l'ancienne archivée.
- **Date de bascule et premier compte rendu** : question ouverte (section « Questions ouvertes »).

### 8.2 Départ d'une agence : export, suppression, preuve (décision n° 94)

**Contexte** : clause de restitution (dossier §2.4, décision n° 5), confiance de l'agence. **Incohérence relevée** : `SECURITY.md` §6.3 propose de **conserver** les écritures, quittances, comptes rendus et audit **10 ans après la sortie** [Hypothèse, juriste], alors que §8.4 propose une **purge** de la production **30 jours** après la fin du contrat ; et `DATABASE_SCHEMA.md` §4.2 n'autorise la purge qu'en phase `setup`. Aucun mécanisme ne supprime aujourd'hui une organisation `live`.

**Options** : (a) export complet, délai de grâce de 30 jours, puis **purge sur instruction écrite de l'agence** ; (b) conservation 10 ans après la sortie, accès fermé, sans purge ; (c) export, accès fermé, conservation chiffrée limitée puis purge programmée.

**Choix recommandé** : (a) [À valider, décision n° 94] [À VÉRIFIER AVEC UN JURISTE] : l'agence reçoit **tout** ce qu'elle doit conserver (c'est elle qui a l'obligation de conservation de ses pièces) ; HABEK, sous-traitant, détruit ensuite sur instruction. Cela exige une **fonction de purge pour une organisation `closed`** à ajouter au schéma avant le **premier** départ (la purge de `setup` ne suffit pas).

| # | Étape | Contenu | Statut |
|---|---|---|---|
| 1 | Demande écrite | de l'agence (gérant) ; confirmation du mot de passe et vérification de l'identité par le développeur et le porteur du projet | [PILOTE] |
| 2 | **Export complet** | `immo org export <org>` : un CSV par table (UTF-8, séparateur « ; »), PDF des quittances et comptes rendus, documents, **manifeste** (fichiers, nombres de lignes, empreintes SHA-256, version du schéma, date) ; exécuté dans le contexte de l'organisation (ne peut contenir qu'elle) ; cellules neutralisées contre les formules ; champs chiffrés déchiffrés pour le client ou exclus selon la décision de `SECURITY.md` | [PILOTE] |
| 3 | **Remise** | canal à accès limité (adresse signée de courte durée, ou **remise en main propre**), **jamais** une pièce jointe d'e-mail ; l'agence vérifie les empreintes et le nombre de lignes ; accusé de réception signé | [PILOTE] |
| 4 | **Fermeture** | l'organisation passe en `suspended` puis `closed` ; utilisateurs désactivés ; sessions révoquées ; interrupteurs retirés ; **délai de grâce de 30 jours** [proposé] pendant lequel l'agence peut redemander l'export | [PILOTE] |
| 5 | **Instruction de purge** | écrite, par le gérant, après le délai de grâce ; sauvegarde préalable ; le développeur et le porteur du projet | avant le premier départ |
| 6 | **Purge** | suppression des données de **cette organisation seule** par une fonction dédiée à droits du propriétaire (réservée à `immo_platform`, refusée si l'organisation n'est pas `closed`) ; ligne `org.purged` dans `platform_events` (qui, quand, pourquoi, tables, **nombre de lignes avant et après**) | avant le premier départ |
| 7 | **Preuve de suppression** | **certificat de destruction** (modèle ci-dessous), signé par le développeur et le porteur du projet, remis sur demande et conservé | [PILOTE] (modèle) |
| 8 | Sauvegardes | **elles expirent selon leur calendrier** : 30 jours (couche 1 et copies quotidiennes) puis 12 copies mensuelles, soit **jusqu'à 12 mois** ; le certificat le **dit** ; la copie du second fournisseur est effacée à échéance (verrouillage d'objet : date de fin de verrou) | [PILOTE] |

```markdown
# Certificat de destruction des données — {nom de l'organisation}
- Identifiant de l'organisation :   Date de fin de contrat :   Date de remise de l'export :   Empreinte SHA-256 du manifeste :
- Instruction de purge reçue le :   de (nom, fonction) :
- Purge exécutée le :   par (nom) et contrôlée par (nom) :
- Données supprimées de la production : tables concernées et nombres de lignes avant et après (extrait de platform_events)
- Fichiers supprimés du stockage objet : nombre, date
- Sauvegardes : les copies contenant des données de l'organisation expirent selon le calendrier suivant : …, au plus tard le JJ/MM/AAAA
- Ce certificat ne couvre pas : les copies détenues par l'agence ; les données anonymisées de l'environnement d'essai (voir ci-dessous)
- Signatures : HABEK (le développeur et le porteur du projet), agence (facultatif)
```

- **Données anonymisées de l'essai** : issues de la copie anonymisée (accord n° 15) ; elles ne permettent pas de remonter à une personne (contrôle automatique) ; la copie d'essai suivante n'inclut plus l'organisation.
- **Effacement individuel** d'un locataire : tension avec l'immuabilité traitée par `SECURITY.md` §8.3 (anonymisation de la fiche, conservation des instantanés pendant la durée de conservation) ; hors périmètre de cette procédure.

## 9. Organisation de démonstration et base d'essai anonymisée

### 9.1 Organisation de démonstration (décision n° 26)

- **Emplacement** : en **production** (`kind = demo`), pour les démonstrations commerciales et comme **témoin d'isolation permanent** ; en essai pour les tests (`ARCHITECTURE.md` §8.5) [À valider, décision n° 26]. Deux organisations : « Agence Horizon Démo » (plan Agence, préfixe `HOR`) et « Patrimoine Sawadogo » (plan Propriétaire, préfixe `SAW`).
- **Garde-fous** : comptes marqués `is_demo`, **mélange avec des comptes réels refusé par la base** ; filigrane « DÉMONSTRATION — sans valeur » sur chaque quittance et compte rendu ; exclue des indicateurs, des exports et de la facturation ; aucun utilisateur réel d'une autre organisation n'y adhère ; **aucune donnée réelle n'y entre, jamais** (y compris un import d'agence).
- **Construction** : le scénario de `seeds/demo/` rejoue les actions métier **par les services**, avec une horloge fixée ; les 28 contrôles de la maquette servent de tests d'intégration.
- **Réinitialisation** : `immo demo reset` crée une **nouvelle** organisation de démonstration et **archive** l'ancienne (aucune suppression). Les mots de passe des comptes de démonstration, connus de l'équipe commerciale, sont **renouvelés à chaque réinitialisation**, soumis aux mêmes limites de connexion, sans accès à d'autres organisations.
- **Date** : décalage des dates pour que « aujourd'hui » corresponde au 13/11/2026 de la maquette, ou date figée réservée aux organisations `demo` [À valider, `ARCHITECTURE.md` §8.5].
- **Sur le calendrier** : réinitialiser **avant** une démonstration, jamais pendant une zone rouge sur un serveur partagé avec la production si la charge est sensible (négligeable au pilote) [Hypothèse].

### 9.2 Base d'essai anonymisée (décision n° 15)

Rafraîchissement **hebdomadaire**, hors zone rouge, **avec accord écrit de l'agence** :

1. Restaurer la dernière copie logique sur une **instance temporaire** (même procédure que 5.4, étapes 3 à 5) ; **jamais d'anonymisation en production** (`immo anonymize` s'exécute sur une restauration).
2. **Anonymiser** : noms et raisons sociales remplacés par des combinaisons fictives ; téléphones remplacés dans le bloc fictif `+226 00 00 …` ; e-mails, pièces d'identité, notes, textes de relance et adresses effacés ou générés ; documents remplacés par un PDF neutre ; **montants, dates et structure conservés** (utiles aux tests de migration et de performance).
3. **Contrôler automatiquement** : aucun numéro de téléphone hors du bloc fictif, aucun e-mail, aucune pièce d'identité, aucun texte de relance ne subsiste ; si le contrôle échoue, **la copie est détruite** et l'alerte est levée.
4. **Remplacer** la base d'essai par la copie anonymisée ; recréer les comptes de l'essai et les secrets (**propres à l'essai**) ; détruire l'instance temporaire.
5. **Exécuter** sur cette copie les migrations en attente, puis les cinq requêtes de contrôle (`TESTING_QA.md` §3.1) : c'est la **répétition générale** de la prochaine livraison.

Accès à l'essai : l'équipe et les référents pilotes ; le rendre inoffensif par construction est le rôle de l'anonymisation, non d'un accès restreint seul.

## 10. Liste de contrôle de mise en production du premier pilote

La liste « prêt pour un pilote » est **unique** : `TESTING_QA.md` §12 (PP-01 à PP-38). Cette liste-ci est la **liste de mise en production** : elle ne contient que les points d'exploitation et renvoie à §12 pour le reste. **Rien n'est coché sans preuve consignée.** Revue finale par le développeur et le porteur du projet (décision écrite de mise en production du pilote).

**Hébergement et environnements**

- [ ] Serveur applicatif et stockage objet choisis par la grille de 1.2 (base chez Neon, DP-55) ; contrat avec clause de restitution, chiffrement, localisation et sous-traitants ; position sur la CIL tranchée ou risque accepté par écrit (DP-62)
- [ ] Sur Neon : `pg_dump` sous RLS forcée vérifié avec un rôle `immo_backup` (**fait au prototype b le 31/10/2026, DP-78 ; à rejouer sur l'essai**) ; `immo_app` et `immo_platform` créés en SQL sans `BYPASSRLS` ; PostgreSQL en version 16 ou plus avec `btree_gist` ; historique du plan : **écart accepté par DP-69 (6 heures au lieu de 30 jours)**, acceptation écrite de Mimis avant la bascule `live`
- [ ] Trois environnements créés ; secrets, clés, seaux, noms d'hôte et cookies **distincts** ; aucun secret de production hors production
- [ ] Domaine, DNS, certificats en place ; sous-domaines de 3.1 ; HSTS ; adresse de la console distincte ; CAA, SPF et DMARC publiés
- [ ] Gouvernance des accès documentée (3.3) : deux administrateurs par ressource, double authentification partout, codes de secours placés, inventaire daté

**Construction et livraison**

- [ ] Image unique de l'essai à la production, utilisateur sans droits, système de fichiers en lecture seule, rendu PDF isolé sans réseau ni secret
- [ ] CI : étapes 1 et 2 de `TESTING_QA.md` §10 vertes ; secrets de production réservés à la promotion manuelle ; protections de branche en place
- [ ] **Mise en production et retour arrière exercés une fois chacun sur l'essai**
- [ ] Migrations : rétrocompatibles ; sauvegarde à la demande avant ; cinq requêtes de contrôle avant et après
- [ ] Interrupteurs de fonctionnalités : au plus 10, dates de retrait, fonctions T3 éteintes

**Sauvegardes et reprise**

- [ ] Sauvegarde continue, copie quotidienne chiffrée chez un second fournisseur (identifiants en écriture seule, verrouillage d'objet), réplication du seau de fichiers
- [ ] Clé de chiffrement hors hébergeur en deux exemplaires, détenus par le développeur et le porteur du projet ; clé de chiffrement des champs sauvegardée à part
- [ ] **Première restauration complète chronométrée sur une instance neuve : durée inférieure à 4 heures, totaux par organisation identiques, compte rendu écrit**
- [ ] Alerte de sauvegarde manquante testée (copie volontairement retardée en essai) ; procédure de restauration d'une seule organisation écrite et répétée

**Supervision et incident**

- [ ] Sonde externe sur `/health`, journaux sans donnée personnelle (test), alertes de 7.3 actives et **testées une par une** ; canal d'alerte vers le développeur et le porteur du projet
- [ ] Page d'état en place, hors du serveur principal ; modèles de messages d'incident prêts
- [ ] Plan de réponse à incident écrit et **joué une fois sur table** ; procédure de maintenance écrite
- [ ] Heures de couverture et personne de repli définies

**Sécurité, données de test, cadre et usage**

Voir `TESTING_QA.md` §12 (PP-12, PP-17 à PP-24 et PP-27 à PP-38). Points propres à l'exploitation :

- [ ] Modèle de certificat de destruction prêt ; fonction de purge d'une organisation `closed` prévue avant le premier départ (n° 94)
- [ ] Calendrier de livraison (section 4) communiqué à l'agence ; date de bascule fixée hors zone rouge
- [ ] Décision écrite de mise en production du pilote, signée par le développeur et le porteur du projet, avec la liste PP-01 à PP-38 cochée et ses preuves

## 11. Indispensable avant le pilote et ce qui peut venir pendant les essais

| Indispensable avant le premier pilote | Peut venir pendant les essais |
|---|---|
| Trois environnements séparés ; image unique ; secrets distincts | Deuxième instance et mise en production sans coupure |
| Sauvegarde continue + copie chiffrée chez un second fournisseur ; **première restauration chronométrée** ; alerte de sauvegarde manquante | Outillage de restauration d'une seule organisation |
| Migrations rétrocompatibles ; sauvegarde avant migration ; retour arrière exercé | Écran d'export pour le gérant (F18) ; mode lecture seule pendant une maintenance |
| Domaine et certificats sous contrôle de l'organisation ; gouvernance des accès | Alertes plus fines, métriques Prometheus, tableau de bord riche |
| Sonde, alertes de 7.3, journaux sans donnée personnelle, page d'état | Analyse et signature des images, liste des composants |
| Procédures d'incident, de maintenance, d'accueil, d'export ; modèle de certificat | Automatisation de l'accueil d'une agence (script de création, jeux de vérification) |
| Calendrier de livraison et interrupteurs de fonctionnalités | Cache des interrupteurs, tableaux de suivi par agence |
| Organisation de démonstration et base d'essai anonymisée | Rafraîchissement automatisé de la copie d'essai |
| Fonction de purge d'une organisation `closed` : **avant le premier départ**, pas avant le premier pilote | Passerelle SMS et fournisseur d'e-mail (décision n° 27) |

## Questions ouvertes

1. **Gouvernance des accès (à trancher)** : qui est le titulaire légal des comptes (HABEK ?) ; quelle **messagerie d'organisation** existe pour recevoir les alertes du registrar et de l'hébergeur ; qui est le **troisième détenteur** des codes de secours (le porteur du projet ?) ; le gestionnaire de mots de passe est-il choisi et payé ? **Proposition** : section 3.3. Tant que ce point n'est pas tranché, un compte personnel d'un développeur est un point de défaillance unique.
2. **Hébergement (n° 5, 86)** : *base tranchée par DP-55 (Neon) ; n° 21 tranchée ; n° 46 tranchée sous condition (prototype b sur Neon).* Restent ouverts : serveur applicatif, stockage objet, second fournisseur de sauvegardes (écriture seule, verrouillage d'objet). Mesure de latence (prototype e) à faire avant.
3. **CIL et hébergement hors du pays (H12)** : une déclaration ou une autorisation est-elle nécessaire avant le pilote ? [À VÉRIFIER AVEC UN JURISTE]
4. **Départ d'une agence** (n° 94) : en attendant l'avis juridique (DP-62), la règle de l'engagement est celle de `SECURITY.md` §8.4 ; la fonction `purge_org_closed` est ajoutée avant le premier départ (`DATABASE_SCHEMA.md` §8.3). Ouverte jusqu'à l'avis.
5. **Heures de couverture et repli** (décision n° 84, ouverte) : jours et heures pendant lesquels le développeur répond (samedi ? soirs ?), personne de repli, numéro d'urgence pour les agences. Pas d'astreinte 24 h sur 24 (DP-54). À fixer par Mimis avant le premier pilote (TESTING_QA PP-37).
6. **Calendrier de livraison** : zones rouge et orange, heures (06 h 00 à 07 h 30) et jours : à confirmer avec une agence réelle ; les agences ouvrent-elles le samedi ? À quels jours tombent leurs pics d'encaissement et leurs comptes rendus ?
7. **Date de bascule et premier compte rendu** : la bascule `live` hors du début de mois laisse un premier mois mixte (paiements en série d'essai puis officielle). Comment produire le premier compte rendu mensuel d'une agence sans mélange ? Faut-il basculer au début d'un mois malgré la zone rouge, ou accepter un premier compte rendu partiel ?
8. **Quittances de la phase `setup`** : ont-elles une valeur pour le locataire (série d'essai avec filigrane « ESSAI ») pendant la double tenue ? L'agence continue-t-elle le carnet ? Question 6 d'`ARCHITECTURE.md`, n° 25 et n° 36.
9. **Chiffrement des sauvegardes** (n° 95) : clé publique ou clé symétrique ? Disponibilité de l'outil sur le second fournisseur.
10. **Sauvegarde du seau de fichiers** : réplication vers le second fournisseur seulement, ou aussi copie hors ligne mensuelle si le verrouillage d'objet n'existe pas ?
11. **Rendu PDF isolé** : conteneur séparé, ou sous-processus restreint ? Dépend du moteur (n° 7) et de l'hébergeur (conteneurs).
12. **Page d'état** : hébergement statique choisi, qui la met à jour, et à quelle fréquence ?
13. **Nom, domaine et marque (n° 13)** : tant que le nom est « provisoire », ni domaine ni certificats définitifs ; la maquette porte encore le bandeau « nom provisoire ».
14. **Environnements marketing** (`film/`, `flyer/`, `infographie/`, `logo/`, `site/` et leurs `.venv`) : hébergement de `www.<domaine>` indépendant du serveur applicatif ; qui publie ?
15. **Capacité (n° 14, H11)** : *tranchée par DP-54 : un développeur, 25 heures par semaine.* Les tests de restauration sont faits par le développeur, observé par le porteur du projet.
16. **Stratégie du pilote (n° 19)** : *tranchée par DP-52 : pilote avec de vraies données, sans enquête préalable (DP-07, DP-10, DP-34 remplacées).*
17. **Test d'intrusion externe avant le pilote** (n° 58) : budget et prestataire.
18. **Plafonds de coûts** : budget mensuel d'infrastructure accepté (estimation de 1.3 : 42 à 111 € par mois) ; le dossier annonce un budget de lancement de l'ordre de quelques centaines de milliers de FCFA [à confirmer].

## Décisions à enregistrer dans DECISIONS.md

Mise à jour du 30/10/2026 : les décisions n° 14 (DP-54), 19 (DP-52), 21 (DP-55), 25 (DP-60), 27 et 51 (DP-68) sont prises ; la n° 46 l'est sous condition (DP-55) ; la n° 5 l'est en partie. Les n° 15, 24, 26 et 54 restent ouvertes et sont reprises et précisées sans être tranchées. Décisions **ouvertes** [À valider], à ajouter à la suite de la n° 85 :

| N° | Décision | Recommandation de ce document |
|---|---|---|
| 86 | Hébergement : option (a) Europe (serveur + PostgreSQL managé + stockage objet) avec grille de choix éliminatoire (1.2) ; coûts estimés : voir 1.3 au total | (a) sous trois conditions ; précise les n° 5, 21 et 46 |
| 87 | Dépôt de code et CI : plateforme avec intégration continue sur compte d'**organisation**, double authentification, protection de branche | Oui (2.2) ; précise la question 13 d'`ARCHITECTURE.md` |
| 88 | Orchestration : Docker Compose sur un serveur jetable avec proxy à certificats automatiques ; image par empreinte ; promotion manuelle ; coupure de quelques secondes hors heures d'encaissement ; deux instances [APRÈS] | Oui (2.3, 2.6) ; plateforme gérée de conteneurs en alternative |
| 89 | Supervision : sonde externe sur `/health`, alertes de 7.3, canal unique vers le développeur et le porteur du projet sans donnée personnelle, accusé de réception en 30 minutes | Oui (7.3) ; précise `ARCHITECTURE.md` §10.3 |
| 90 | Noms d'hôte : `app.`, `essai.`, `console.` (hôte distinct), `www.` (hors serveur applicatif), `status.` (hors serveur principal) ; domaine au nom de l'organisation ; CAA ; SPF et DMARC « aucun expéditeur » | Oui (3.1) ; dépend de la n° 13 |
| 91 | Calendrier de livraison : zones rouge (1 à 7), orange (8 à 10 et 25 à 31), verte ; mardi à jeudi ; 06 h 00 à 07 h 30 ; deux semaines de calme après une bascule | Oui (section 4) ; à recalibrer après le premier mois |
| 92 | Secrets : variables d'environnement du conteneur ou de la plateforme, copies et récupération dans le gestionnaire de mots de passe de l'équipe, pas de coffre dédié au pilote, rotation annuelle et au départ | Oui (2.4) |
| 93 | Page d'état : page statique hors du serveur principal, trois états, maintenances planifiées | Oui (7.5) |
| 94 | Départ d'une agence : export complet, délai de grâce de 30 jours, **purge sur instruction écrite** après fermeture, certificat de destruction, expiration des sauvegardes annoncée ; fonction de purge d'une organisation `closed` ajoutée avant le premier départ | Oui (8.2), **à faire valider par un juriste** ; recoupe `SECURITY.md` §6.3 et §8.4 |
| 95 | Chiffrement des sauvegardes : chiffrement à clé publique, clé privée hors ligne en deux exemplaires, le serveur ne portant que la clé publique | Oui (5.2) ; précise la n° 53 et corrige `BACKUP_ENCRYPTION_KEY` de `CLAUDE.md` |
