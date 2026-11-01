# Immo Suite — Sécurité

| Champ | Valeur |
|---|---|
| Statut | Proposition à valider. Aucune décision de ce document n'est prise tant qu'une entrée DP-xx n'existe pas dans `DECISIONS.md`. Les éléments juridiques sont marqués [À VÉRIFIER AVEC UN JURISTE] et ne sont affirmés nulle part comme acquis |
| Date | 30/10/2026 |
| Dépend de | `DECISIONS.md` (état au 30/10/2026), dossier de cadrage v1.0 (§11.2 matrice des droits, §12, §12.1, §16 F01 à F03 et F39/F40, §19, §24), `ARCHITECTURE.md` v0.2 (§3.8, §5, §8, §10, §11), `DATABASE_SCHEMA.md` v0.2, `DOMAINE.md`, `CONTEXTE.md`, `UI_UX_DESIGN.md` (S01, S17, S18, S21, S26) |
| Alimente | `API_CONTRACTS.md`, `CODING_RULES.md`, `TESTING_QA.md`, `DEPLOYMENT.md`, `ROADMAP.md`, `contenus/` (engagement de confidentialité) |
| Version | 0.2 |

Légende des marqueurs ([PILOTE], [APRÈS], [À valider], [Hypothèse], [À VÉRIFIER AVEC UN JURISTE]) : voir `CLAUDE.md`. Ce document ne cite aucun texte de loi ni aucune autorité comme certains ; il reprend seulement ce que le dossier de cadrage et `ARCHITECTURE.md` en disent.

Points de sécurité obligatoires avant le premier pilote (détail et cases à cocher en section 10) :

- **[PILOTE]** Isolation entre organisations à deux barrières, avec test d'isolation par route bloquant en CI (`DATABASE_SCHEMA.md` §6).
- **[PILOTE]** Écritures financières en ajout seul garanti par la base, journal d'audit écrit dans la même transaction.
- **[PILOTE]** Mots de passe argon2id, sessions serveur, protection contre la falsification de requête, limitation des tentatives, inscription publique fermée.
- **[PILOTE]** Double authentification TOTP obligatoire pour les comptes HABEK ; confirmation par mot de passe avant les actions à enjeu.
- **[PILOTE]** Accès support HABEK uniquement avec l'accord du client, limité dans le temps et journalisé ; aucun accès de routine à la base de production.
- **[PILOTE]** Chiffrement en transit, chiffrement applicatif des coordonnées de paiement et des secrets, sauvegardes chiffrées hors de l'hébergeur principal, restauration testée et chronométrée.
- **[PILOTE]** Secrets hors du code, environnements séparés, journaux sans donnée personnelle ni jeton.
- **[PILOTE]** Import de fichiers durci (taille, type, archives, formules), PDF rendus dans un processus isolé.
- **[PILOTE]** Engagement de confidentialité signé par chaque agence pilote, plan de réponse à incident écrit et répété une fois.

## 1. Actifs, acteurs, hypothèses de confiance

### 1.1 Actifs à protéger

| Actif | Où il vit | Pourquoi il compte | Atteinte redoutée |
|---|---|---|---|
| **Écritures financières** : paiements, affectations, contre-passations, reversements, versions de comptes rendus | `payments`, `allocations`, `payouts`, `statement_versions` | C'est la vérité sur l'argent. Le produit n'a de valeur que si elles sont exactes et inaltérables (règles d'or 4 et 9) | Modification, suppression, ajout frauduleux, antidatage |
| **Preuves remises aux locataires** : quittances et leur numérotation | `receipts`, PDF dans le stockage objet, `share_links` | Une quittance est une preuve opposable. Un trou ou un doublon de numéro la rend contestable (R-050) | Contrefaçon, numéro réutilisé, PDF différent de l'instantané |
| **Données personnelles de locataires** | `tenants`, `reminders.body`, `receipts.snapshot`, `import_rows.raw` | Noms, téléphones, adresses de lots, dettes, parfois pièce d'identité | Fuite, usage détourné, relance envoyée au mauvais destinataire |
| **Données personnelles de mandants** | `owners`, `statement_versions.snapshot` | Noms, revenus locatifs, **coordonnées de paiement** (`payout_details_enc`) | Fuite, détournement d'un reversement |
| **Documents** (T3) | stockage objet privé, `documents` | Baux scannés, pièces, photos | Fuite, fichier piégé |
| **Identifiants et secrets des utilisateurs** | `users.password_hash`, `totp_secret_enc`, `sessions`, jetons d'invitation et de réinitialisation | Prise de contrôle d'un compte | Vol, rejeu, devinette |
| **Secrets d'exploitation** | variables d'environnement, clé de chiffrement applicatif, clé de sauvegarde, accès à la base et au stockage | Accès à tout | Fuite dans le dépôt, un journal, une copie d'écran |
| **Journal d'audit** | `audit_log`, `platform_events` | Dernier recours pour trancher un litige ou une fraude | Effacement, falsification, lacune |
| **Sauvegardes** | copie continue, copie logique chiffrée, seau répliqué | Seule défense contre la perte totale | Perte, corruption, chiffrement par un attaquant, restauration impossible |
| **Disponibilité de l'encaissement** | service web, base, rendu PDF | Un caissier ne peut pas attendre un jour de début de mois | Panne, blocage par un attaquant |
| **Code, chaîne de construction, images** | dépôt, CI, registre d'images | Une porte dérobée dans le code ou une dépendance contourne tout le reste | Compromission |

### 1.2 Acteurs

| Acteur | Confiance | Accès légitime | Risque propre |
|---|---|---|---|
| Gérant, propriétaire direct | Élevée | Tous les droits de son organisation | Compte compromis = tout l'argent de l'organisation (contre-passation, reversement, exports, droits) |
| Gestionnaire | Moyenne à élevée | Baux, locataires, relances ; encaisse si autorisé | Fuite de contacts, relances abusives |
| **Caissier** | **Moyenne** | Encaisse et émet les quittances | Détournement d'espèces (dossier §24 : risque « fraude interne », probabilité moyenne, impact élevé) |
| Agent terrain | Moyenne | Ses lots seulement, sur un téléphone d'entrée de gamme | Perte ou vol du téléphone, partage d'appareil |
| Comptable (lecture) | Moyenne, peut être un tiers de l'agence (DP-42) | Lecture seule, exports | Fuite par export |
| Collaborateur du plan Propriétaire | Moyenne | Encaisse, relance | Parent ou proche : confiance personnelle, pas de procédure |
| **Locataire** | Aucune | Reçoit un lien ; pas de compte | Devine ou transmet un lien, usurpation |
| **Mandant** | Aucune | Reçoit un lien vers son compte rendu | Idem |
| **Équipe HABEK (support)** | Élevée mais encadrée | Console ; accès aux données **seulement avec l'accord du client** (R-009) | Curiosité, erreur, abus, compte compromis |
| **Développeurs HABEK** | Élevée mais encadrée | Code, CI, exploitation | Accès à la production, secrets, porte dérobée |
| Hébergeur et sous-traitants (stockage, sauvegarde, alertes) | Contractuelle | Infrastructure | Lecture des disques, réquisition, défaillance |
| **Attaquant externe** | Aucune | Internet | Devinette de mots de passe, hameçonnage, injection, fichier piégé |
| Ancien salarié d'une agence | Aucune (révoquée) | Aucun si le compte est désactivé | Compte non désactivé, mot de passe partagé |
| Fournisseur de messagerie (WhatsApp, Meta) | Aucune | Reçoit le texte et le numéro des liens pré-remplis | Métadonnées, mauvais destinataire |

### 1.3 Hypothèses de confiance

- **H-S1.** Les navigateurs sont à jour sur des Android d'entrée de gamme, souvent partagés ou perdus (`UI_UX_DESIGN.md`, principe 3). Aucune donnée n'est stockée durablement dans le navigateur : le service worker ne met jamais en cache une réponse d'API (`ARCHITECTURE.md` §3.6).
- **H-S2.** TLS protège le trajet entre le navigateur et le serveur. La base est chez Neon (DP-55) et le serveur chez un hébergeur à choisir, tous deux hors du pays [Hypothèse, décision n° 5] : on suppose qu'il ne lit pas la base de façon routinière, **sans pouvoir l'affirmer** ; d'où le chiffrement applicatif des champs les plus sensibles et le chiffrement des sauvegardes avant envoi.
- **H-S3.** L'équipe HABEK compte un développeur à environ 25 heures par semaine et le porteur du projet (DP-54). Elle est digne de confiance **mais pas au-dessus du contrôle** : chaque accès est journalisé, aucun compte n'est partagé, aucun développeur n'a d'accès de routine à la production.
- **H-S4.** Un caissier peut être négligent ou malhonnête ; un gérant est présumé honnête mais son compte peut être volé.
- **H-S5.** WhatsApp chiffre le contenu des messages de bout en bout, mais ses métadonnées (qui écrit à qui, quand) ne sont pas sous notre contrôle [Hypothèse ; politique du fournisseur à vérifier]. Immo Suite ne l'utilise que par liens pré-remplis (DP-33) : aucune API, aucun message envoyé par le serveur.
- **H-S6.** Les données de pilote sont réelles. Les environnements de développement et d'essai n'en contiennent jamais sans anonymisation et accord écrit (décision n° 15).
- **H-S7.** La maquette ne touche pas à la production : aucune donnée réelle, aucune ressource externe (`MAQUETTE.md`).

## 2. Modèle de menaces

Priorité : **P** = mesure indispensable avant le pilote ; **A** = peut venir pendant les essais. Les mesures renvoient aux sections de ce fichier et de `DATABASE_SCHEMA.md` (DB).

| # | Menace | Scénario concret dans ce produit | Impact | Mesure | Priorité |
|---|---|---|---|---|---|
| T1 | **Accès entre organisations** | Une route oublie le filtre d'organisation ; un identifiant de quittance de l'agence B est demandé par un utilisateur de l'agence A ; une requête manuelle de support omet `org_id` | Fuite de loyers, noms et téléphones d'une agence concurrente ; perte de confiance définitive | Deux barrières (filtre dans le code, RLS forcée) ; clés étrangères composites ; réponse « Introuvable » (jamais « interdit ») plus ligne d'audit et alerte immédiate ; test d'isolation par route bloquant (`ARCHITECTURE.md` §5.4, DB §6) ; organisation de démonstration comme témoin | **P** |
| T2 | **Caissier négligent ou malveillant** | Encaisse des espèces et ne saisit rien ; saisit 50 000 au lieu de 75 000 et garde la différence ; encaisse un loyer deux fois ; demande des corrections en rafale ; réimprime des quittances ; prête son compte | Perte d'argent de l'agence et du mandant ; litige de preuve | Quittance numérotée remise au locataire à chaque paiement (l'absence de quittance est visible du locataire) ; **séparation des tâches** : le caissier ne contre-passe jamais (R-048, appliqué par la base) ; demande de correction tracée ; clôture de caisse F19 [APRÈS, T3] ; indicateurs de fraude : écart entre date de paiement et de saisie, corrections par caissier, paiements espèces sans quittance envoyée [APRÈS] ; comptes nominatifs, déconnexion automatique | **P** (séparation, audit, comptes nominatifs) ; **A** (indicateurs, caisse) |
| T3 | **Falsification ou suppression d'un paiement** | Un développeur lance un `UPDATE` pour « corriger » ; une injection SQL modifie un montant ; une migration réécrit des lignes ; un support supprime une ligne gênante | Les chiffres d'une agence deviennent contestables ; fin de la confiance dans le produit | Ajout seul à deux barrières (privilèges + déclencheur), `TRUNCATE` refusé, contre-passation seule voie de correction ; aucune migration ne modifie une écriture (DB §7.2) ; contrôle nocturne de la présence des gardes ; audit dans la même transaction (R-008) ; sauvegardes pour comparer | **P** |
| T4 | **Quittance contrefaite** | Un locataire fabrique un PDF avec un numéro plausible et un faux montant ; un employé réédite une quittance annulée ; un lien est modifié | Preuve falsifiée, litige de caution, fraude à l'agence | Instantané immuable en base, PDF avec empreinte SHA-256 ; statut « annulée » visible sur la copie réimprimée ; **code de contrôle imprimé et page publique de vérification** (section 7.5, décision n° 55) ; numéros sans trou : un numéro absent de la base est faux | **P** (instantané, empreinte) ; **P/A** (code de contrôle) |
| T5 | **Vol de compte** | Hameçonnage par un faux message WhatsApp « votre compte expire » ; mot de passe faible ou réutilisé ; téléphone volé avec session ouverte ; mot de passe partagé entre caissiers | Un attaquant avec les droits du gérant contre-passe, change des droits, exporte tout | argon2id, liste de mots de passe courants, limitation et verrouillage progressifs, sessions courtes et révocables, **TOTP** pour HABEK et, avant la bascule en `live`, pour le gérant, **confirmation par mot de passe** avant les actions à enjeu, notification des autres gérants après un changement de droits, comptes nominatifs ; formation (une page) | **P** |
| T6 | **Fuite par exports ou PDF** | Un gestionnaire non autorisé exporte tous les locataires ; une cellule d'export commence par `=` et devient une formule dans le tableur du comptable ; un PDF contient des métadonnées (nom de l'utilisateur) ; un lien de partage est transmis en groupe WhatsApp | Fuite massive ; compromission du poste du comptable | Droit d'export réglable et audité (`export.created`) ; neutralisation des formules (préfixe) ; PDF sans métadonnées d'auteur ; liens expirables, révocables, limités à un document ; `Referrer-Policy: no-referrer` ; exports jamais envoyés par e-mail au pilote | **P** |
| T7 | **Fichier d'import piégé** | Un XLSX qui contient une archive « bombe » ; un document qui référence une entité XML externe ; un `.xlsm` avec macro ; un CSV de plusieurs gigaoctets ; une cellule qui est une formule malveillante ; une valeur qui tente une injection | Déni de service, lecture de fichiers du serveur, exécution de code, données corrompues | Types autorisés (`.csv`, `.xlsx`) vérifiés par contenu ; taille et nombre de lignes plafonnés ; limites sur l'archive (taille décompressée, nombre d'entrées, ratio) ; **aucune évaluation de formule** (valeurs en cache seulement) ; analyseur XML sans entités externes ; requêtes paramétrées ; rien n'est créé avant confirmation (R-080) ; traitement dans un contexte à ressources limitées (section 7.4) | **P** |
| T8 | **Abus du support HABEK** | Un agent consulte le dossier d'un client sans demande ; copie la liste des locataires ; un compte de la console est volé ; un agent modifie des données d'une agence amie | Fuite et atteinte à l'intégrité par une personne de confiance | Accès **uniquement** avec l'accord du client, limité à 7 jours au plus et révocable (politique RLS dédiée, DB §6.6) ; comptes nominatifs avec TOTP obligatoire ; ligne d'audit `support` à chaque consultation, visible du client (S17) ; pas d'accès aux hachages, secrets ni champs chiffrés ; aucune lecture de routine de la base ; accès d'urgence à deux personnes, tracé (section 5.6) | **P** |
| T9 | **Perte ou vol de téléphone** | Un agent terrain perd son Android avec la session ouverte ; un caissier le laisse sur le comptoir ; des PDF de quittances restent dans la galerie | Accès aux dettes et aux contacts de ses lots ; usurpation | Expiration par inactivité courte, expiration absolue, **révocation à distance** (le gérant désactive l'utilisateur ; toutes ses sessions sont révoquées) ; aucune donnée d'API en cache local ; portée limitée de l'agent terrain (ses lots) ; « déconnecter tous mes appareils » | **P** |
| T10 | **Injection, scripts intersites, falsification de requête** | Une note de locataire contient `<script>` ; une requête forgée depuis un autre site déclenche une contre-passation ; une valeur d'import contient du SQL | Vol de session, actions à l'insu de l'utilisateur, lecture ou écriture de la base | SQL paramétré (SQLx), échappement par défaut du front, CSP stricte, jeton anti-falsification lié à la session plus `SameSite`, en-têtes, validation typée (section 7) | **P** |
| T11 | **Déni de service** | Rafale de tentatives de connexion ; envois répétés de gros fichiers ; demandes de PDF en boucle ; début de mois chargé | Les caissiers ne peuvent plus encaisser | Limitation du débit par identifiant, adresse et route ; plafonds de taille ; file de tâches pour le PDF avec limite d'essais ; sonde externe, alertes (`ARCHITECTURE.md` §10.3) | **P** (limites) ; **A** (protection en amont) |
| T12 | **Chaîne logicielle compromise** | Une dépendance Rust ou npm est piégée ; un script d'installation exécute du code ; une image de base contient une faille connue | Prise de contrôle du serveur ou du poste d'un développeur | Verrouillage des versions, `cargo audit` et `cargo deny`, audit npm bloquants en CI, peu de dépendances, revue de toute nouvelle dépendance, construction en CI seulement (section 7.6) | **P** |
| T13 | **Fuite de secrets** | Une clé dans le dépôt ; un mot de passe de base dans un journal ; un `.env` partagé par messagerie | Accès direct à la base, aux fichiers, aux sauvegardes | Secrets hors du dépôt, détection de secrets en CI, journaux filtrés, rotation, séparation par environnement (section 5.4) | **P** |
| T14 | **Perte, corruption ou chiffrement des données** | Disque perdu ; migration défectueuse ; attaquant qui chiffre ou supprime la base **et** les sauvegardes accessibles | Perte d'une agence entière | Trois couches de sauvegarde, la copie logique chez un **autre fournisseur** avec identifiants en écriture seule et verrouillage d'objet ; clé de chiffrement hors hébergeur ; **restauration testée chaque mois** (section 9.4) | **P** |
| T15 | **Énumération et accès direct à un objet** | Essai de numéros de téléphone sur la connexion pour savoir qui est inscrit ; changement d'un identifiant dans une adresse | Découverte de clients ; accès à un objet voisin | Messages d'erreur identiques et durée comparable ; UUID non devinables ; 404 pour tout objet hors organisation ; limitation du débit | **P** |
| T16 | **Lien de partage deviné ou diffusé** | Un locataire transmet son lien ; un robot parcourt des adresses ; le lien reste dans l'historique ou un journal | Lecture d'une quittance ou d'un compte rendu par un tiers | Jeton aléatoire d'au moins 128 bits stocké sous forme d'empreinte ; **un seul document par lien** ; expiration (30 jours par défaut) ; révocation ; jeton absent des journaux ; `noindex` | **P** |
| T17 | **Message de relance détourné** | Relance envoyée au mauvais numéro (homonyme, saisie fautive) ; locataire qui reçoit un faux message « vous devez payer » imitant l'agence | Divulgation d'une dette à un tiers ; fraude par hameçonnage | Le panneau montre le nom et la fin du numéro avant d'ouvrir WhatsApp ; texte minimal ; gabarit constant et domaine unique, annoncé aux locataires (section 7.8) | **P** (confirmation) ; **A** (communication aux locataires) |
| T18 | **Données de test réelles** | Une copie de production est utilisée en développement sans anonymisation | Fuite par un environnement moins protégé | Environnement d'essai alimenté seulement par une copie anonymisée avec contrôle automatique, accord écrit de l'agence (décision n° 15) ; aucun rôle d'accès à la production depuis le poste de développement | **P** |
| T19 | **Moteur de rendu de PDF détourné** | Un nom de locataire contient du balisage qui charge une ressource externe ou un fichier local pendant le rendu | Lecture de fichiers, requêtes sortantes depuis le serveur | Gabarit fixe, échappement systématique, processus de rendu **isolé** sans réseau, sans secret, sans accès à la base, limites de temps et de mémoire (section 7.5) | **P** |
| T20 | **Abus de l'inscription publique** | Création en masse d'organisations factices ; courrier indésirable ; consommation de ressources | Coût, pollution, surface d'attaque | **Inscription publique fermée au pilote** : les organisations sont créées par la console HABEK (décision n° 51) | **P** |
| T21 | **Réquisition ou accès légal à l'hébergeur** | Une autorité demande l'accès aux serveurs d'un hébergeur étranger | Divulgation de données clients | Chiffrement applicatif des champs sensibles, clauses contractuelles de restitution et de localisation [À VÉRIFIER AVEC UN JURISTE], procédure de réponse | **A** |

## 3. Authentification

### 3.1 Mots de passe

- **Contexte** : l'identifiant est un numéro de téléphone ou un e-mail ; il n'y a pas de fournisseur d'identité externe (dossier §12, F01). Les utilisateurs sont sur téléphone, souvent peu à l'aise avec des règles de composition.
- **Options** : (a) argon2id ; (b) bcrypt ; (c) scrypt.
- **Choix recommandé** : (a) argon2id, avec les paramètres minimum recommandés pour une mémoire de 19 Mio, 2 itérations, 1 degré de parallélisme, **à calibrer sur le serveur retenu** pour viser de l'ordre de 100 à 500 ms par vérification [Hypothèse, décision n° 48].
- **Raison** : argon2id résiste mieux aux attaques par cartes graphiques que bcrypt ; c'est le choix du dossier (§12). Le coût mémoire est la contrepartie : il limite le nombre de vérifications simultanées (un plafond de 4 vérifications en parallèle évite qu'une rafale de connexions épuise la mémoire d'un petit serveur).
- **Conséquences** :
  - le paramétrage est inscrit dans l'empreinte ; après une revue des paramètres, l'empreinte est recalculée à la connexion suivante ;
  - **règles de choix du mot de passe** : au moins 10 caractères, pas de règle de composition, refus des mots de passe les plus courants (liste embarquée) et de ceux qui contiennent l'identifiant ; plafond de 128 caractères pour éviter un abus de calcul [valeurs proposées] ;
  - un poivre (secret serveur mélangé au mot de passe avant le hachage) est possible [APRÈS] : il protège si la base fuit seule ;
  - comparaison en temps constant ; aucun mot de passe, même haché, dans les journaux ;
  - la base refuse toute valeur qui ne commence pas par `$argon2id$` (`users_hash_format`, DB §3).

### 3.2 Sessions ou jetons

| | Sessions côté serveur (cookie) | Jetons JWT sans état |
|---|---|---|
| Révocation | Immédiate (ligne supprimée ou marquée) : désactiver un utilisateur, changer un rôle, perdre un téléphone | Impossible avant l'expiration sans liste de révocation, donc un état serveur de toute façon |
| Rôle effectif | Relu à chaque requête : un changement de rôle s'applique **sans reconnexion** (F02) | Figé dans le jeton jusqu'à son expiration |
| Secret à faire tourner | Aucun (identifiant aléatoire) | Clé de signature |
| Exposition au vol par un script | Cookie `HttpOnly` : inaccessible au JavaScript | Souvent placé dans le stockage du navigateur : lisible par un script injecté |
| Charge | Une lecture indexée par requête | Aucune |

- **Choix recommandé** : sessions côté serveur (confirme `ARCHITECTURE.md` §3.8) [À valider, décision n° 49].
- **Raison** : la révocation immédiate est une exigence (F02) et un téléphone volé en est le cas type. Le seul coût (une lecture par requête) est négligeable aux volumes du pilote.
- **Conséquences** :
  - identifiant de session : 256 bits tirés du générateur aléatoire du système ; **seule son empreinte SHA-256 est stockée** (`sessions.id_hash`) ;
  - cookie nommé `__Host-session` : `Secure`, `HttpOnly`, `SameSite=Lax`, chemin `/`, sans attribut `Domain` ;
  - nouvel identifiant à la connexion, au changement de mot de passe et au changement d'organisation ; **un changement de rôle ne révoque rien** (le rôle est relu à chaque requête, DP-88) ;
  - **durées proposées** : inactivité 30 minutes pour caissier, gestionnaire, agent terrain et comptable ; 60 minutes pour gérant et propriétaire ; expiration absolue 12 heures pour tous ; console HABEK : 15 minutes d'inactivité, 8 heures au plus. Pas de « se souvenir de moi » au pilote [valeurs proposées, à confirmer avec les caissiers : une expiration trop courte fait partager les mots de passe] ;
  - changement de mot de passe : **toutes** ses sessions sont révoquées ; désactivation d'un membre : ses sessions **de cette organisation** (DP-88) ; une action « déconnecter tous mes appareils » est offerte [PILOTE] ; la liste des sessions actives avec leur appareil [APRÈS] ;
  - **falsification de requête** : `SameSite=Lax`, **plus** un jeton propre à la session (`sessions.csrf_hash`) exigé sur toute requête qui écrit, plus contrôle de l'en-tête `Origin` ;
  - aucune réponse d'API n'est mise en cache (`Cache-Control: no-store`).

### 3.3 Verrouillage et limitation du débit

Compteurs dans PostgreSQL (`login_attempts`), sans Redis (`ARCHITECTURE.md` §3.8).

| Cible | Limite proposée | Effet |
|---|---|---|
| Connexion, par identifiant | 5 échecs consécutifs | Attente croissante : 1, 5, 15 minutes, puis 15 minutes à chaque nouvel échec. **Pas de verrouillage définitif** : il permettrait à un tiers de bloquer le compte d'un gérant |
| Connexion, par adresse IP | 20 échecs en 10 minutes | Refus 15 minutes |
| Réinitialisation de mot de passe, invitation | 5 demandes par heure et par identifiant | Refus, message neutre |
| Code TOTP | 5 essais par minute | Attente |
| Écritures d'argent | 60 par minute et par session [Hypothèse] | Refus 429, la clé d'idempotence rend le nouvel essai sans risque |
| API en général | Plafond par session, plus un plafond global par adresse | Refus 429 |
| Journal des échecs | Conservé 30 jours | Alerte au-delà d'un seuil (`ARCHITECTURE.md` §10.3) |

- Message de connexion **identique** que l'identifiant existe ou non, et temps de réponse comparable (vérification factice) : la page S01 ne révèle pas qui est inscrit (T15).
- Le message affiché est celui de `UI_UX_DESIGN.md` (« Trop de tentatives. Réessayez dans {durée} ou réinitialisez votre mot de passe »).
- Un identifiant bloqué n'empêche pas le gérant de réinitialiser le mot de passe d'un utilisateur (3.5).

### 3.4 Double authentification

- **Contexte** : le dossier prévoit un TOTP optionnel pour le gérant (P1) et un TOTP obligatoire pour la console (F39, F40). Les utilisateurs ont des Android d'entrée de gamme ; les SMS sont payants, peu fiables, et vulnérables au détournement de carte SIM ; WhatsApp n'est pas une API (DP-33).
- **Options** : (a) TOTP par application d'authentification (mots de passe à usage unique temporels) ; (b) code par SMS ; (c) clés d'accès (WebAuthn) ; (d) pas de second facteur, mais confirmation du mot de passe avant les actions à enjeu.
- **Décision** (DP-68, décision n° 50) :
  - **TOTP obligatoire pour tous les comptes HABEK** de la console **[PILOTE]** ;
  - **confirmation par mot de passe (« réauthentification ») avant chaque action à enjeu** **[PILOTE]** : contre-passation, validation d'un compte rendu, réouverture, reversement, accord ou révocation d'un accès support, invitation ou changement de rôle d'un gérant, désactivation d'un utilisateur, export, changement de numéro de téléphone du compte. Elle vaut pour 5 minutes ;
  - **TOTP proposé** à tout gérant ou propriétaire dès la première connexion, et **obligatoire avant la bascule de son organisation en `live`** **[PILOTE]** ; sinon il reste à l'état « proposé » avec un rappel ;
  - 10 **codes de secours** à usage unique, affichés une seule fois, stockés sous forme d'empreinte ;
  - SMS **jamais obligatoire** ; clés d'accès [APRÈS], si les téléphones des utilisateurs les gèrent.
- **Raison** : le compte du gérant est le plus précieux (T5). (d) seule laisse un voleur de session agir pendant l'inactivité ; (a) ajoute un second facteur sans coût par message ni dépendance à un opérateur. L'exigence avant `live` laisse le premier mois de reprise (`setup`) à la prise en main.
- **Conséquences** :
  - le secret TOTP est chiffré par l'application (`users.totp_secret_enc`, section 5.2) ; décalage d'horloge toléré d'un pas de chaque côté, car l'horloge des téléphones d'entrée de gamme dérive ; un code déjà utilisé est refusé (rejeu) ;
  - la perte du téléphone du gérant passe par la procédure 3.5, qui exige une vérification renforcée ;
  - `UI_UX_DESIGN.md` S17 prévoit un interrupteur « 2FA du gérant » : il devient un parcours d'activation, rattaché à une tranche (question ouverte du fichier d'interface).

### 3.5 Récupération de compte

Au pilote, ni passerelle SMS ni e-mail (décision n° 27) : la réinitialisation passe par un **lien généré** puis transmis par l'utilisateur lui-même, par exemple sur WhatsApp. Le « code à usage unique valable 15 minutes » de F01 devient un lien à usage unique de 15 minutes.

| Cas | Procédure | Garde |
|---|---|---|
| Un utilisateur oublie son mot de passe | Il demande à son gérant ; le gérant génère le lien (confirmation par mot de passe), qui expire en 15 minutes | Lien à usage unique, jeton stocké sous forme d'empreinte ; ligne d'audit ; toutes les sessions de l'utilisateur sont révoquées après usage ; les autres gérants sont informés |
| Le **gérant** oublie son mot de passe, ou perd son téléphone TOTP | Il contacte le support HABEK. **Vérification renforcée** : rappel au numéro enregistré par une seconde personne de HABEK, question sur des données que seul le gérant connaît (par exemple, le dernier numéro de quittance et son montant) [Hypothèse], et si un autre gérant existe, **son accord** | Deux personnes de HABEK ; entrée dans `platform_events` ; le lien est à usage unique ; le TOTP est réinitialisé ; notification visible à tous les gérants |
| Perte de tous les codes de secours | Même procédure que le gérant | Idem |
| Compte volé suspecté | Le gérant ou le support révoque toutes les sessions et force le changement du mot de passe | Voir le plan d'incident (section 9) |

- Le support ne choisit **jamais** un mot de passe à la place de l'utilisateur et ne le connaît jamais.
- Une réinitialisation ne révèle pas si l'identifiant existe (message neutre).

### 3.6 Invitation des utilisateurs

- Le gérant crée l'invitation (téléphone ou e-mail, rôle, lots assignés pour un agent terrain) : lien à usage unique, **7 jours** (F01), jeton de 256 bits dont seule l'empreinte est stockée (`invitations.token_hash`), copiable dans WhatsApp.
- L'invité fixe son mot de passe à l'acceptation ; son rôle est celui de l'invitation. Un utilisateur qui existe déjà (autre organisation) se connecte puis accepte.
- Inviter un **gérant** ou changer le rôle d'un gérant exige la confirmation par mot de passe et informe les autres gérants.
- Une invitation expirée ou révoquée est refusée avec le message de `UI_UX_DESIGN.md` ; le gérant peut la renvoyer.
- Le lien peut être transmis à un tiers : la durée courte et l'usage unique limitent le risque ; l'acceptation est visible dans l'audit (`user.invited`, `membership.accepted`). Une vérification supplémentaire du numéro de l'invité est possible [APRÈS].

### 3.7 Authentification de la console HABEK

- Comptes **nominatifs**, TOTP obligatoire (`platform_staff.totp_required` est vrai par contrainte), session plus courte (3.2), jamais de compte partagé.
- Console servie sous un **nom d'hôte distinct** de l'application, avec un cookie distinct ; restriction par adresse IP envisageable [Hypothèse, selon la stabilité des connexions de l'équipe].
- La console n'affiche ni empreintes de mots de passe, ni secrets TOTP, ni champs chiffrés (section 5.6).

## 4. Autorisation

### 4.1 Matrice des rôles et permissions (organisation Agence)

Reprise de la section 11.2 du dossier de cadrage, source unique (`CONTEXTE.md`). Appliquée **côté serveur**, jamais seulement masquée dans l'interface (F02). Un test par couple rôle × action, généré depuis cette matrice (dossier §19).

| Action | Gérant | Gestionnaire | Caissier | Agent terrain | Comptable (lecture) |
|---|---|---|---|---|---|
| Créer / modifier biens, lots, baux | Oui | Oui | Non | Non | Non |
| Voir biens, lots, locataires | Oui | Oui | Lecture | Ses lots assignés | Lecture |
| Enregistrer un paiement | Oui | Réglable | Oui | Non (P1 : sur place avec validation) | Non |
| Corriger / annuler un paiement | Oui | Demande | Demande | Non | Non |
| Émettre / réimprimer une quittance | Oui | Oui | Oui | Non | Lecture |
| Voir le tableau des impayés | Oui | Oui | Lecture | Ses lots | Lecture |
| Envoyer une relance, noter une promesse | Oui | Oui | Non | Oui (ses lots) | Non |
| Voir / produire le compte rendu propriétaire | Oui | Voir | Non | Non | Voir |
| Valider un reversement | Oui | Non | Non | Non | Non |
| Saisir un travail / une dépense | Oui | Oui | Non | Proposer | Non |
| Exporter les données | Oui | Réglable | Non | Non | Oui |
| Gérer utilisateurs et rôles | Oui | Non | Non | Non | Non |
| Voir le journal d'audit | Oui | Non | Non | Non | Lecture |

- **Plan Propriétaire** : mêmes rôles, deux profils prédéfinis. Propriétaire = tous les droits. Collaborateur = enregistrer les paiements et relancer (`role_settings` règle son droit d'encaisser).
- **Compléments à la matrice (DP-66)** : voir `CONTEXTE.md`, section « Matrice de permissions » (collaborateur, agent terrain, entrées de menu non définies masquées).
- **Actions que la matrice ne nomme pas** (déduites, [À valider]) : accorder ou révoquer un accès support (gérant seul) ; modifier les paramètres de l'organisation et les modèles de messages (gérant seul) ; activer l'import (gérant ; gestionnaire si autorisé) ; valider ou annuler une ligne exceptionnelle d'échéance (gérant, gestionnaire).
- Les entrées de menu « ? » de `UI_UX_DESIGN.md` (Baux, Travaux et documents, Agenda pour caissier, agent terrain et comptable) sont **refusées par défaut** tant que la matrice ne les ouvre pas.

### 4.2 Principe du moindre droit

- **Refus par défaut** : une action sans règle explicite est refusée. `identity::authorize(actor, Action)` est l'unique endroit où la matrice est codée (`ARCHITECTURE.md` §4.9).
- Le **rôle** est relu en base à chaque requête (pas dans la session) ; un retrait de droit est immédiat.
- **Agent terrain** : filtre de portée appliqué dans toutes les lectures de `collections`, `lease` et `property` (ses lots) ; la recherche unique est limitée à ses lots (R-081). Le schéma ne le garantit pas (DB §6.10) : il est testé par un jeu de lots assignés et non assignés.
- **Visibilité des champs par rôle** [À valider] :

| Champ | Gérant, gestionnaire | Caissier | Agent terrain | Comptable |
|---|---|---|---|---|
| Nom, téléphone, lot, solde du locataire | Oui | Oui | Ses lots | Oui |
| Contact de secours, notes | Oui | Non | Non | Non |
| Pièce d'identité (si activée) | Masquée, affichage tracé (`tenant.id_document_viewed`) | Jamais | Jamais | Jamais |
| Coordonnées de paiement du mandant | Gérant seul, masquées par défaut | Jamais | Jamais | Jamais |
| Adresse IP de l'audit | Gérant, propriétaire direct, comptable (lecture) (DP-88) | Jamais | Jamais | Lecture |

- **Données sensibles masquées par défaut**, révélées par un geste explicite et tracé (R-019).
- Les comptes d'intégration ou de service **n'existent pas** au pilote (pas d'API publique, `CONTEXTE.md`).

### 4.3 Défense en profondeur

| Couche | Rôle | Ce qui la contourne |
|---|---|---|
| 1. Interface | Masque les entrées interdites | Une requête forgée : d'où les couches suivantes |
| 2. Route HTTP | Extracteur `Actor` ; `authorize` avant tout traitement ; jeton anti-falsification ; limitation du débit | Un oubli dans un gestionnaire : testé par un test par route |
| 3. Service métier | Contrôles métier et de portée (agent terrain) | Un appel direct à un autre module : interdit par les façades (`ARCHITECTURE.md` §2.2) |
| 4. **PostgreSQL** | RLS forcée par organisation ; clés composites ; privilèges de colonne ; gardes en base pour quelques actions d'argent | Un rôle propriétaire ou un super-utilisateur : n'existe que hors de l'application |
| 5. Audit | Toute action sensible laisse une trace dans la même transaction (R-008) | Rien : c'est le filet final |

Actions dont la restriction est **aussi** imposée par la base, quelle que soit la route : contre-passer (gérant, `payments_before_insert`), valider un compte rendu, le rouvrir, enregistrer un reversement (gérant), retirer le dernier gérant, créer un mandat hors plan Agence, accéder à une organisation sans accord support, mélanger comptes de démonstration et comptes réels.

Les autres restrictions de la matrice ne sont appliquées que par les couches 1 à 3 ; elles sont couvertes par le test rôle × action.

## 5. Données

### 5.1 Chiffrement en transit

- **[PILOTE]** HTTPS partout, TLS 1.2 au minimum (1.3 de préférence), certificats renouvelés automatiquement, `Strict-Transport-Security` sur le domaine.
- **[PILOTE]** Connexion à la base en TLS avec vérification du certificat du serveur ; connexion au stockage objet en TLS ; aucun trafic de production en clair, y compris entre conteneurs si l'hébergeur les place sur un réseau partagé.
- Les liens WhatsApp pointent vers le domaine HTTPS de l'application.

### 5.2 Chiffrement au repos

| Donnée | Mécanisme | Statut |
|---|---|---|
| Disque de la base et du serveur | Chiffrement du volume par l'hébergeur [À vérifier au contrat] | **[PILOTE]** |
| Stockage objet (PDF, documents) | Chiffrement côté serveur du seau [À vérifier] | **[PILOTE]** |
| Sauvegardes | Chiffrées **avant envoi** par l'équipe, clé hors de l'hébergeur (5.3) | **[PILOTE]** |
| Champs applicatifs : `owners.payout_details_enc`, `users.totp_secret_enc`, `tenants.id_document_enc` | **Chiffrement par l'application**, authentifié, avec nonce aléatoire par valeur et identifiant de clé en préfixe | `payout_details_enc` et `totp_secret_enc` : **[PILOTE]** ; `id_document_enc` : avant d'activer le champ [À valider, `ARCHITECTURE.md` §13.2] |

- **Contexte** : le chiffrement du disque protège d'un disque perdu, pas d'un administrateur de l'hébergeur ni d'une copie de la base. La pièce d'identité et les coordonnées de paiement sont les champs dont la fuite fait le plus de tort.
- **Options** : (a) chiffrement dans l'application avant l'écriture ; (b) extension de chiffrement de PostgreSQL, clé transmise dans la requête ; (c) aucun chiffrement de champ.
- **Choix recommandé** : (a) [À valider, décision n° 52].
- **Raison** : avec (b), la clé voyage dans les requêtes et peut apparaître dans les journaux de requêtes lentes ou de l'hébergeur. Avec (a), la base ne contient jamais ni clé ni valeur en clair.
- **Conséquences** : algorithme de chiffrement authentifié éprouvé (AES-256-GCM, par une bibliothèque reconnue) ; **clé de chiffrement des clés** fournie par l'environnement, jamais en base ni dans le dépôt ; une clé de données par type de champ, chiffrée par la première ; identifiant de clé stocké avec la valeur pour permettre la **rotation** (rechiffrement par tâche de fond) ; ces champs ne se recherchent pas et ne s'indexent pas ; la sauvegarde contient les valeurs chiffrées, donc la perte de la clé de chiffrement des clés rend ces champs illisibles : elle est sauvegardée à part, en deux exemplaires (5.3).

### 5.3 Sauvegardes chiffrées

Objectifs de `ARCHITECTURE.md` §10.4 [À valider, décision n° 24], **adaptés au plan gratuit de Neon (DP-69)** : restauration à un instant donné de **6 heures** (l'objectif de 30 jours est suspendu), perte maximale de **24 heures** si l'hébergeur de la base est perdu (une copie logique par jour ; l'objectif de 15 minutes est suspendu), reprise 4 heures, 30 copies quotidiennes et 12 copies mensuelles, **test de restauration chaque mois** à partir d'une copie logique.

| Mesure | Détail | Statut |
|---|---|---|
| Trois couches | Sauvegarde continue du service managé ; copie logique quotidienne chiffrée chez un **autre fournisseur** ; réplication quotidienne du seau de fichiers | **[PILOTE]** |
| Chiffrement | Avant envoi, par une clé qui n'est **jamais** stockée avec les sauvegardes ni chez l'hébergeur principal ; conservée en **deux exemplaires**, détenus par le développeur et le porteur du projet | **[PILOTE]** |
| Droits du travail de sauvegarde | Identifiants du second fournisseur limités à l'**écriture** : ils ne peuvent ni lire ni supprimer ni écraser. Verrouillage d'objet ou versionnage pour qu'un attaquant qui prend le serveur ne puisse pas effacer les copies | **[PILOTE]** si le second fournisseur le propose ; sinon compensé par une copie hors ligne mensuelle [Hypothèse] |
| Rôle de sauvegarde | `pg_dump` exige un rôle qui contourne la RLS forcée ; il n'est jamais utilisé par l'application (`DATABASE_SCHEMA.md` §6.9, décision n° 46) | **[PILOTE]** |
| Alerte | Aucune copie réussie depuis 24 heures (F39) | **[PILOTE]** |
| Restauration | Test mensuel chronométré sur une instance neuve : comptage des lignes, totaux par organisation, contrôle de numérotation, audit de l'isolation (section 9.4) | **[PILOTE]** |
| Accès | Lecture des sauvegardes réservée au développeur et au porteur du projet ; chaque restauration est consignée | **[PILOTE]** |

Les sauvegardes contiennent des données personnelles : leur durée de vie borne la rapidité d'un effacement (section 8.3).

### 5.4 Gestion des secrets

| Secret | Où il vit | Qui y accède | Rotation |
|---|---|---|---|
| `DATABASE_URL` (rôle `immo_app`) | Variable d'environnement du conteneur de production | Le processus applicatif | À chaque départ d'une personne ayant eu accès ; annuelle |
| `MIGRATION_DATABASE_URL` (rôle `immo_owner`) | Étape de déploiement de la CI de production seulement | La CI, le développeur et le porteur du projet en cas d'urgence | Idem |
| Rôle de sauvegarde (`immo_backup`) | Tâche de sauvegarde seulement | La tâche | Idem |
| `S3_*` (stockage objet) | Environnement du conteneur ; un jeu **par environnement** | Le processus | Annuelle |
| `SESSION_SECRET` | Environnement | Le processus | Annuelle (révoque les sessions en cours) |
| Clé de chiffrement des clés applicatives | Environnement de production, **copie hors ligne** | Le processus ; le développeur et le porteur du projet pour la copie | Sur départ ; rechiffrement des clés de données |
| `BACKUP_ENCRYPTION_KEY` | **Hors de l'hébergeur** : gestionnaire de mots de passe de l'équipe, deux exemplaires | Le développeur et le porteur du projet | Annuelle ; l'ancienne reste utilisable pour relire les anciennes copies |
| Accès aux comptes de l'hébergeur, du nom de domaine, du dépôt | Gestionnaire de mots de passe, **double authentification** | Le développeur et le porteur du projet | Sur départ |

Règles :

- **Aucun secret dans le dépôt.** Un fichier `.env` ne s'y trouve jamais (`.gitignore`). Une détection de secrets (outil dédié) est exécutée en CI et en pré-validation locale **[PILOTE]**.
- Aucun secret dans un journal, un message d'erreur, une capture d'écran de support ou une conversation.
- Un secret exposé est **révoqué et remplacé immédiatement** ; la procédure est écrite (section 9).
- Séparation stricte par environnement (5.5) : un secret de production n'existe nulle part ailleurs.
- Les postes de développement n'ont **aucun** secret de production.

### 5.5 Séparation des environnements

| | Développement | Essai | Production |
|---|---|---|---|
| Données | Organisation de démonstration et jeux de tests | Démonstration, test, copie **anonymisée** de la production (décision n° 15) | Réelles, plus l'organisation de démonstration (`kind = demo`) |
| Secrets, clés, seaux, domaines, noms de cookies | Propres | Propres | Propres |
| Accès réseau à la production | Aucun | Aucun | — |
| Qui | Le développeur | L'équipe et les référents pilotes | Le développeur ; le support par la console |
| CI | Tests | Déploiement automatique | **Promotion manuelle de la même image**, aucune reconstruction (`ARCHITECTURE.md` §10.5) |

- La copie anonymisée est produite sur une restauration, jamais en production ; un contrôle automatique vérifie qu'aucun numéro de téléphone réel, e-mail, pièce d'identité ni texte de relance ne subsiste (`ARCHITECTURE.md` §11.3) **[PILOTE]**.
- Les comptes de développement ne sont jamais présents en production, ni l'inverse.

### 5.6 Accès du personnel HABEK aux données des clients

Règle : **aucun accès à des données client sans entrée d'audit** (F39), et **aucun accès sans accord du client** (R-009, F40).

| Qui | Pour quoi | Comment | Durée | Trace |
|---|---|---|---|---|
| Support HABEK | Aider, importer, corriger | Console + **accord donné par le gérant** dans S17 (motif obligatoire) ; politique de la base qui ne rend rien sans accord valide (DB §6.6) | Choisie par le client ; **24 heures par défaut, 7 jours au plus** [valeur proposée] ; révocable à tout moment | Ligne d'audit `support` à chaque consultation, visible du client avec l'identité de l'agent |
| Développeurs | Exploitation, correction d'un bogue | **Aucun accès de routine** à la base de production. Les bogues se reproduisent sur l'essai, avec des données anonymisées | — | — |
| Développeurs, **accès d'urgence** (« bris de glace ») | Incident grave qui exige de lire la production (restauration, perte de données) | Le développeur et le porteur du projet présents ; rôle d'administration utilisé le temps nécessaire ; l'organisation concernée est prévenue | Le temps de l'incident | `platform_events` : qui, quand, pourquoi, quelles tables ; compte rendu écrit après coup |
| Support HABEK | Import assisté (F40) | Même accord, portée `import_assist` | Idem | `import_batches.assisted_by_staff` plus audit |

- La console **ne montre jamais** : empreintes de mots de passe, secrets TOTP, empreintes de jetons, valeurs chiffrées en clair (pièce d'identité, coordonnées de paiement). La lecture d'un champ chiffré par le support exige une mention explicite dans l'accord [À valider].
- À l'arrivée d'une personne : compte nominatif, TOTP, engagement de confidentialité interne. Au départ : désactivation le jour même, rotation des secrets auxquels elle avait accès, revue des accès du fournisseur d'hébergement.
- Le client voit dans S17 l'historique complet des accès (qui, quand, combien de temps).

### 5.7 Organisation de démonstration et inscription

- L'organisation de démonstration (`kind = demo`) ne contient que des données fictives (`MAQUETTE.md`) ; ses comptes sont marqués `is_demo` et **la base refuse** tout mélange avec des comptes réels (DB §7.6). Elle est exclue des exports, des indicateurs et de la facturation, et ses quittances portent le filigrane « DÉMONSTRATION — sans valeur ».
- Les mots de passe des comptes de démonstration sont connus de l'équipe commerciale ; ils sont renouvelés à chaque `immo demo reset`, soumis aux mêmes limites de connexion, et **sans accès** à d'autres organisations.
- **L'inscription publique de F01 est désactivée au pilote** : les organisations sont créées par la console, après la lettre d'intention (décision n° 51, DP-68). Réouverture à la version commerciale, avec validation du numéro de téléphone, limitation par adresse et surveillance des créations.

## 6. Journal d'audit

### 6.1 Ce qui est tracé

Chaque ligne : qui (utilisateur, système ou agent de support), quoi (code `objet.verbe`), sur quel objet, valeurs avant et après, quand (imposé par la base), adresse IP, identifiant de requête (`audit_log`, DB §3). **Écrite dans la même transaction que l'action** : si l'écriture du journal échoue, l'action est annulée (R-008).

| Domaine | Actions tracées (codes) |
|---|---|
| Argent | `payment.recorded`, `payment.correction_requested`, `payment.correction_rejected`, `payment.reversed`, `allocation.adjusted` |
| Preuves | `receipt.issued`, `receipt.cancelled`, `receipt.corrective_issued`, `receipt.sent`, `receipt.link_created`, `receipt.link_revoked`, `receipt.pdf_failed` |
| Baux | `lease.created`, `lease.activated`, `lease.amended`, `lease.notice_given`, `lease.notice_cancelled`, `lease.ended`, `installment.line_added`, `installment.line_voided` |
| Patrimoine | `unit.works_set`, `unit.archived`, `tenant.created`, `tenant.updated`, `tenant.merged`, `tenant.archived`, `tenant.id_document_viewed`, `owner.updated`, `mandate.created`, `mandate.rate_changed`, `mandate.closed` |
| Comptes rendus | `statement.validated`, `statement.reopened`, `payout.recorded`, `expense.imputation_changed` |
| Accès et droits | `auth.login`, `auth.password_changed`, `auth.mfa_enabled`, `auth.reset_issued`, `auth.reset_used`, `user.invited`, `invitation.renewed`, `invitation.revoked`, `membership.accepted`, `membership.role_changed`, `user.disabled`, `user.enabled`, `user.updated`, `auth.mfa_backup_used`, `session.revoked_all` (cinq codes ajoutés par DP-88) |
| Données | `export.created`, `import.confirmed`, `import.cancelled`, `settings.updated`, `feature.toggled` |
| Support | `support.access_granted`, `support.access_revoked`, `support.record_viewed`, `support.import_assist` |
| Sécurité | `security.cross_org_attempt` (R-004), `security.permission_denied` sur action sensible, `security.rate_limited` |

- Les **valeurs avant et après** ne contiennent ni mot de passe, ni jeton, ni secret ; pour un champ chiffré, seule la mention « modifié » est écrite. Les données personnelles y sont réduites aux champs modifiés (minimisation).
- Une **tentative d'accès à une ressource d'une autre organisation** est écrite dans le journal de l'organisation de l'appelant, avec l'identifiant visé, et déclenche une alerte immédiate (`ARCHITECTURE.md` §10.3).
- Les échecs de connexion sont dans `login_attempts` (30 jours), pas dans l'audit des organisations, pour ne pas le polluer ; leur résumé (`security.rate_limited`) y figure.

### 6.2 Immuabilité

- **Deux barrières** (DB §4.2) : le rôle applicatif n'a que `SELECT` et `INSERT` sur `audit_log` ; un déclencheur refuse `UPDATE`, `DELETE` et `TRUNCATE`, même au propriétaire du schéma. L'heure d'écriture est imposée par la base.
- **Contrôle nocturne** : les gardes d'écriture seule existent et sont actives ; sinon alerte **[PILOTE]**.
- **Au-delà du pilote [APRÈS]** : chaîne de hachage par organisation (chaque ligne contient l'empreinte de la précédente) pour détecter une altération faite avec les droits du propriétaire du schéma ; condensé quotidien conservé hors de la base. Cette chaîne sérialise les écritures d'une organisation ; elle n'est pas nécessaire au pilote.
- La purge d'une organisation en phase `setup` (DB §8.3) et l'annulation d'un import non utilisé (R-080) sont les **seules** suppressions possibles ; elles sont tracées dans `platform_events`, hors de l'organisation (DP-60).

### 6.3 Durée de conservation

Valeurs **proposées** pour tout ce qui touche à l'obligation de conservation [À VÉRIFIER AVEC UN JURISTE] (décision n° 56, DP-62).

| Donnée | Durée proposée | Raison |
|---|---|---|
| `audit_log`, écritures financières, quittances, comptes rendus | Durée du contrat, puis jusqu'à la purge sur instruction écrite de l'agence (§8.4) ; durée après la sortie : 10 ans proposés [Hypothèse : durée usuelle de conservation de pièces comptables] [À VÉRIFIER AVEC UN JURISTE] (DP-62) | Preuve en cas de litige |
| `platform_events` | 10 ans | Preuve des accès et purges de la plateforme |
| `login_attempts`, journaux applicatifs, `error_reports` | 30 jours | Limitation du débit, diagnostic |
| `sessions` | Jusqu'à l'expiration, plus 30 jours | Investigation |
| `idempotency_keys` | 30 jours | Nouvel essai après coupure |
| `product_events` | 13 mois, puis agrégats (`ARCHITECTURE.md` §8.4) | Pilotage |
| `import_rows.raw` et fichier importé | 30 jours après confirmation [Hypothèse] | Contrôle de l'import, puis suppression |
| Sauvegardes | Restauration à un instant donné : 6 heures (plan gratuit, DP-69) ; 30 copies quotidiennes plus 12 copies mensuelles | `ARCHITECTURE.md` §10.4 |
| Liens de partage | Jusqu'à expiration, puis conservés inactifs avec leur empreinte | Preuve d'un envoi |

### 6.4 Consultation

- Le gérant et le comptable en lecture consultent le journal (S21) ; filtres par utilisateur, objet, période ; export par le gérant, **lui-même tracé** (`export.created`).
- Le support HABEK ne le consulte que sous accord (5.6).
- Aucune suppression, aucune modification, quel que soit le rôle (R-007).
- Alertes automatiques à l'équipe : tentative inter-organisations, accès support, trou de numérotation, garde d'écriture seule désactivée.

## 7. Sécurité applicative

### 7.1 Validation des entrées

- **Côté serveur, toujours.** La validation du front n'est qu'un confort.
- Désérialisation **typée** : montants en entiers de 64 bits bornés, dates en dates, énumérations fermées, identifiants en UUID ; champs inconnus refusés ; longueurs maximales ; texte nettoyé des caractères de contrôle.
- Téléphone normalisé et vérifié (`+226` + 8 chiffres, R-018) ; e-mail en minuscules.
- Taille maximale des requêtes (JSON : 64 Kio ; envois de fichiers : plafonds de 7.4) et délai maximal de traitement.
- **SQL** : requêtes paramétrées uniquement (SQLx) ; aucune construction de requête par concaténation ; les noms de colonnes ou de tri venant du client passent par une **liste blanche** ; règle de revue et de lint.
- La base refuse d'ailleurs les formats invalides (téléphone, montant, énumérations) : deuxième ligne de défense (DB §3).
- Les messages d'erreur ne contiennent ni pile d'appels, ni nom de table, ni valeur d'un autre utilisateur ; format `problem+json` avec un code d'incident court (`ARCHITECTURE.md` §3.7).

### 7.2 Protections courantes du web

| Menace | Mesure |
|---|---|
| Injection SQL | Requêtes paramétrées, vérifiées à la compilation (`query!`), listes blanches pour tri et filtres |
| Scripts intersites (XSS) | Échappement par défaut du front, **aucune insertion de HTML brut** ; texte libre (notes, noms) toujours traité comme du texte ; CSP stricte (7.3) ; le HTML de la page publique S26 est rendu avec échappement automatique |
| Falsification de requête (CSRF) | `SameSite=Lax`, jeton lié à la session sur toute requête qui écrit, contrôle de `Origin` (3.2) |
| Accès direct à un objet | 404 « Introuvable » hors organisation (R-004) ; UUID ; test par route |
| Affectation de masse | Structures d'entrée explicites par route ; jamais de liaison automatique d'un JSON vers une ligne de table |
| Requêtes sortantes détournées | Aucune requête sortante construite à partir d'une adresse fournie par l'utilisateur ; le rendu de PDF n'a pas accès au réseau |
| Redirection ouverte | Les redirections n'acceptent que des chemins internes |
| Parcours de répertoires | Les clés de stockage sont générées par le serveur à partir de l'`org_id` du contexte, jamais d'un nom de fichier fourni |
| Mise en cache de données personnelles | `Cache-Control: no-store` sur l'API et la page publique ; le service worker ne met jamais en cache une réponse d'API (`ARCHITECTURE.md` §3.6) |
| Cross-origin | Aucune configuration CORS : l'API ne répond qu'à son propre domaine |
| Déni de service | 3.3, 7.1, 7.4 |

### 7.3 En-têtes de sécurité

| En-tête | Valeur proposée | Raison |
|---|---|---|
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains` | Pas de retour en HTTP |
| `Content-Security-Policy` | `default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'; object-src 'none'` | Aucun script ou style en ligne, aucune ressource externe (cohérent avec l'absence de polices distantes de `UI_UX_DESIGN.md`) |
| `X-Content-Type-Options` | `nosniff` | Pas de devinette de type |
| `Referrer-Policy` | `no-referrer` | **Le jeton d'un lien de partage est dans l'adresse** : il ne doit pas fuiter par l'en-tête de provenance |
| `Permissions-Policy` | `camera=(), microphone=(), geolocation=(), payment=()` | Aucune de ces fonctions n'est utilisée |
| `Cross-Origin-Opener-Policy`, `Cross-Origin-Resource-Policy` | `same-origin` | Isolation du contexte |
| `X-Frame-Options` | `DENY` | Anciens navigateurs |
| `Cache-Control` | `no-store` sur l'API, les pages publiques et les PDF servis | Pas de copie locale |
| `X-Robots-Tag` | `noindex, nofollow` sur la page publique | Les liens de partage ne sont pas indexés |

- Test automatique en CI : une requête de chaque type de page vérifie la présence de ces en-têtes **[PILOTE]**.
- Les cookies suivent 3.2.
- La page publique S26 ne charge **aucun** JavaScript applicatif (`ARCHITECTURE.md` §3.5).

### 7.4 Envoi et traitement de fichiers

| Fichier | Règles |
|---|---|
| **Import** (CSV, XLSX), **[PILOTE]** | Extension **et** contenu vérifiés (signature de l'archive pour XLSX) ; **refus** de `.xls`, `.xlsm` et de tout fichier avec macros ; taille maximale de 5 Mo et 5 000 lignes [valeurs proposées, prototype d : 2 000 lignes en moins de 5 s] ; lecture de l'archive avec plafonds sur la taille décompressée, le nombre d'entrées et le ratio de compression (**anti-bombe d'archive**) ; **analyseur XML sans entités externes ni définition de type** (anti-lecture de fichier) ; **aucune formule évaluée** : seules les valeurs en cache sont lues ; encodage détecté (Windows-1252, UTF-8) ; cellules traitées comme du texte ; analyse dans une tâche à mémoire et temps limités |
| Logos et signatures de l'organisation | PNG ou JPEG, 200 Ko au plus à l'envoi ; **réencodés** par le serveur (suppression des métadonnées, dont la localisation) et réduits à 30 Ko pour l'affichage (`UI_UX_DESIGN.md`, Performance) |
| Documents (T3) : PDF, JPEG, PNG | Types autorisés par contenu, 10 Mo au plus (`documents`, DB §3) ; **jamais affichés dans l'origine de l'application** : téléchargement avec `Content-Disposition: attachment` et `nosniff` ; réencodage des images ; les PDF téléversés peuvent contenir du code actif : ils ne sont ouverts que par le lecteur du système de l'utilisateur |
| Tous | Nom de fichier d'origine **jamais** utilisé comme clé : clé générée par le serveur (`org/{org_id}/…`) ; seau privé ; adresses signées de quelques minutes ; isolation vérifiée par test (`ARCHITECTURE.md` §5.4) ; empreinte SHA-256 enregistrée |

- **Analyse antivirus** : une analyse par un moteur de signatures (par exemple ClamAV) demande plus d'un gigaoctet de mémoire [Hypothèse], excessif pour un serveur de 2 à 4 Go partagé avec le rendu de PDF. Elle est [APRÈS] et seulement avant d'ouvrir l'envoi de documents (T3) ; au pilote, la protection est la restriction des types et le téléchargement forcé.
- L'import est **réversible tant que rien n'est utilisé** (R-080) : un fichier qui passe les contrôles et crée des données douteuses est annulable.

### 7.5 Génération de PDF sécurisée

Le moteur n'est pas choisi (décision n° 7). Quel qu'il soit, ces exigences s'appliquent **[PILOTE]** :

- **Entrée** : l'instantané JSON de la quittance ou du compte rendu, jamais une adresse ni un fichier fournis par l'utilisateur.
- **Gabarits fixes** dans le dépôt ; toute valeur utilisateur est **échappée** (échappement automatique) ; polices embarquées, aucune ressource distante.
- Le rendu s'exécute dans un **processus isolé** : utilisateur système sans droits, **pas de réseau**, **pas de secret**, **pas d'identifiants de base**, système de fichiers en lecture seule hors dossier temporaire, **limites de temps et de mémoire**. Avec un navigateur sans interface : bac à sable du navigateur activé, désactivation de JavaScript et du réseau, profil jetable.
- Aucune métadonnée d'auteur dans le PDF (ni nom d'utilisateur, ni nom d'agent) ; titre neutre.
- **Filigranes** : « DÉMONSTRATION — sans valeur » (organisation `demo`), « ESSAI » (série `trial`), « ANNULÉE — remplacée par la quittance n° … » (`UI_UX_DESIGN.md`).
- Empreinte SHA-256 du PDF enregistrée (`receipt_files`) ; le PDF peut toujours être reproduit depuis l'instantané.
- **Code de contrôle** (décision n° 55) [À valider] : un court code (10 caractères) calculé à partir du numéro, du montant, de la date et d'une clé secrète du serveur est imprimé sur la quittance, avec l'adresse d'une page publique de **vérification** qui, à partir du numéro et du code, confirme « quittance émise par {agence}, montant, date, **statut** (émise ou annulée) », sans autre donnée personnelle. Cette page **n'expire pas** (contrairement au lien de partage de R-054) et est limitée en débit. Elle répond à T4 et à la question ouverte de `UI_UX_DESIGN.md` sur l'empreinte imprimée.

### 7.6 Dépendances et chaîne de construction

- **Rust** : `Cargo.lock` versionné, construction avec `--locked`, **`cargo audit`** (avis de sécurité de la base RustSec) et **`cargo deny`** (licences, sources, doublons, crates interdites) en CI, **bloquants** pour un avis de gravité haute ou critique **[PILOTE]**.
- **Front** : `package-lock.json` versionné, installation par `npm ci`, `npm audit` bloquant au même seuil **[PILOTE]**, scripts d'installation désactivés en CI sauf liste blanche [Hypothèse].
- Dépendances peu nombreuses et **justifiées** (`ARCHITECTURE.md` §3.5) ; toute nouvelle dépendance est lue en revue : maintenance, nombre de dépendances transitives, usage de code non sûr.
- Mises à jour **hebdomadaires** (échéance courte pour une faille connue : 72 heures, comme les correctifs prioritaires du dossier §20.2).
- **Image de conteneur** : base minimale, construite en CI seulement, étiquetée par le commit, **la même image** de l'essai à la production ; analyse de l'image [APRÈS] ; signature des images [APRÈS].
- Les accès au dépôt et à la CI sont protégés par la double authentification ; les secrets de production ne sont pas accessibles aux branches de test.

### 7.7 Journalisation sans données sensibles

- **Liste blanche** de champs journalisables (`ARCHITECTURE.md` §10.1) : identifiant de requête, route (avec son **modèle**, pas l'adresse réelle), statut, durée, `org_id`, `user_id`, version.
- **Jamais** : nom, téléphone, corps de requête, texte de relance, mot de passe, jeton, cookie, valeur chiffrée.
- Les adresses qui portent un jeton (`/s/{jeton}`, invitations, réinitialisations) sont **masquées** dans les journaux du serveur d'application **et du proxy** : un test de CI cherche des motifs de numéros `+226` et de jetons dans les journaux d'un parcours complet **[PILOTE]**.
- Le suivi d'erreurs ne conserve pas les corps de requête (`ARCHITECTURE.md` §8.4).
- Conservation 30 jours ; accès réservé au développeur et au porteur du projet.

### 7.8 Liens WhatsApp pré-remplis

L'application ouvre WhatsApp avec un texte prêt (DP-33). Le texte et le numéro du destinataire transitent par l'adresse `wa.me` et le navigateur ; le contenu des messages est chiffré par WhatsApp de bout en bout, **pas ses métadonnées** (H-S5).

- **Minimisation du message** : prénom ou nom du locataire (selon le modèle), montant, mois, nom de l'agence et téléphone de l'agence (`MAQUETTE.md`) ; le lien de la quittance. **Rien d'autre** : ni adresse complète, ni numéro de pièce, ni historique, ni nom d'un autre locataire, ni solde global.
- **Mauvais destinataire** : avant l'ouverture de WhatsApp, le panneau affiche le **nom** et les **deux derniers chiffres** du numéro (**[PILOTE]**) ; la relance n'est journalisée qu'après confirmation de l'utilisateur (R-061).
- **Lien** : jeton de 128 bits au moins, un seul document, expiration (R-054), révocation ; l'adresse ne contient aucune donnée personnelle ; en-tête `Referrer-Policy: no-referrer` (7.3) ; jamais dans les journaux.
- **Hameçonnage** : un message qui imite celui de l'agence est possible. Mesures : texte et domaine **constants** ; les agences annoncent à leurs locataires l'adresse officielle ; le locataire n'a jamais à saisir de mot de passe ni d'argent via ce lien [APRÈS : message type à remettre aux agences].
- **Variables** : un modèle refuse les variables inconnues (R-064), donc ne peut pas injecter de champ imprévu.
- Aucun envoi automatique par le serveur (règle d'or 11) : l'humain confirme.

### 7.9 Autres mesures

- Aléa : générateur cryptographique du système pour tous les jetons, identifiants de session, codes de secours.
- Comparaison de jetons **par empreinte**, en temps constant.
- Horodatage et dates d'expiration calculés par le serveur.
- Délais d'attente sur toutes les requêtes sortantes (base, stockage objet) ; arrêt propre.
- Un test de non-régression est écrit pour chaque faille corrigée.

## 8. Données personnelles

Cette section décrit des **pratiques**. Elle ne dit pas ce que la loi exige : tout ce qui est juridique est [À VÉRIFIER AVEC UN JURISTE]. Le dossier de cadrage (§2.4, §12.1) et `ARCHITECTURE.md` citent la loi n° 001-2021/AN et la CIL ; **ce document ne les reprend pas comme acquis** et n'en tire aucune obligation précise.

### 8.1 Finalité et minimisation

| Donnée | Finalité | Minimisation retenue |
|---|---|---|
| Locataire : nom, téléphone, e-mail, activité | Identifier le débiteur, émettre la quittance, relancer | E-mail facultatif ; pas d'adresse personnelle (seule l'adresse du lot est utile) ; pas de date de naissance, pas de photo |
| Locataire : contact de secours | Joindre un proche si le locataire est injoignable | Facultatif |
| Locataire : pièce d'identité | Lutte contre la fraude locative (facultatif, F06) | **Champ désactivé au pilote**, chiffré, masqué ; le **numéro de CNIB n'est pas demandé** s'il n'est pas utile (dossier §12.1) |
| Mandant : nom, téléphone, mode et coordonnées de reversement | Rendre compte et reverser | Coordonnées chiffrées par l'application |
| Tiers payeur : nom, téléphone facultatif | Mentionner qui a payé sur la quittance (R-088) | Téléphone facultatif |
| Utilisateur : nom, téléphone ou e-mail, adresse IP, sessions | Authentification, audit, sécurité | Adresse IP conservée 30 jours dans les journaux, durée de l'audit dans le journal |
| Événements d'usage (`product_events`) | Mesurer l'usage pendant le pilote | **Aucune** donnée personnelle dans les propriétés : liste blanche fermée, valeurs refusées si elles contiennent `@`, `+226` ou six chiffres de suite ; référence de session pseudonyme (16 caractères de l'empreinte du jeton, jamais le jeton) ; conservation 13 mois ; ni cookie publicitaire, ni service tiers (DP-90) |
| Erreurs (`error_reports`) | Corriger des bogues | Agrégées par empreinte ; jamais de corps de requête, de paramètre ni de message de bibliothèque ; message du front nettoyé (limite connue : un mot libre écrit par le code du front ne se reconnaît pas) |
| Signalements d'utilisateurs (`feedback_reports`) | Corriger des problèmes | Pas de capture d'écran au pilote ; contexte technique minimal ; texte libre stocké tel quel, **jamais journalisé** ni copié dans un événement ou une alerte ; lu par l'équipe HABEK **sans accord support** (exception à R-009, DP-90, avertissement dans le formulaire) |

- Aucune donnée n'est vendue, cédée ni utilisée pour de la publicité ou de l'IA (règle d'or 11) ; les données de test viennent d'une copie **anonymisée, avec l'accord écrit de l'agence** (décision n° 15).

### 8.2 Durées de conservation

Voir la table de 6.3. Principe : **une donnée ne vit pas plus longtemps que sa finalité**, sauf ce qui doit rester comme preuve (écritures, quittances, comptes rendus, audit), dont la durée est à fixer [À VÉRIFIER AVEC UN JURISTE].

### 8.3 Droits des personnes, effacement

- **Accès, rectification, opposition** : la personne s'adresse à l'agence (ou au propriétaire direct), qui est en contact avec elle. L'agence transmet la demande à HABEK, qui répond en 30 jours au plus [valeur proposée]. La qualité de responsable du traitement (l'agence) et de sous-traitant (HABEK) est à confirmer [À VÉRIFIER AVEC UN JURISTE].
- **Rectification** : les fiches (`tenants`, `owners`) sont modifiables, la modification est tracée.
- **Effacement** : **tension à trancher.** Les écritures, les quittances, les relances et les comptes rendus sont en ajout seul et contiennent un nom (dans l'instantané) ; leur effacement détruirait la preuve. Approche proposée [À VÉRIFIER AVEC UN JURISTE] :
  1. la fiche `tenants` ou `owners` est **anonymisée** (nom remplacé, téléphone et contacts effacés) tant que l'obligation de conservation le permet ;
  2. les instantanés, relances et pièces comptables restent pendant leur durée de conservation (6.3), puis l'organisation est purgée à sa sortie ;
  3. les sauvegardes disparaissent par expiration (30 jours, puis 12 mois pour les copies mensuelles) : **l'effacement complet n'est pas immédiat**, ce que l'engagement de confidentialité dit clairement.
  Un champ de marquage de l'anonymisation est à ajouter au schéma (`erased_at`), par migration [APRÈS].

### 8.4 Export des données d'une organisation qui part

- Commande `immo org export` (`DATABASE_SCHEMA.md` §8.5) **[PILOTE]** : un CSV par table, les PDF, les documents, un manifeste avec empreintes. Remis au gérant par un canal à accès limité (adresse signée de courte durée ou remise en main propre), jamais en pièce jointe d'e-mail.
- Après l'export et la fin du contrat : **délai de grâce de 30 jours** [valeur proposée], puis purge de l'organisation (suppression des données de production) ; les sauvegardes expirent selon leur calendrier ; un **certificat de destruction** est remis sur demande [Hypothèse].
- Le contrat avec l'hébergeur doit prévoir la **restitution** et la **destruction** des données (dossier §2.4, décision n° 5) [À VÉRIFIER AVEC UN JURISTE].

### 8.5 Sous-traitants et localisation

Sous-traitants prévus au pilote : hébergeur du serveur et de la base, fournisseur du stockage objet, **second fournisseur de sauvegardes**, outil d'alerte de l'équipe (sans donnée personnelle). **Aucun outil d'analyse ou de suivi d'erreurs tiers** (décision n° 28). WhatsApp n'est pas un sous-traitant de HABEK : c'est le canal du client [À VÉRIFIER AVEC UN JURISTE]. La liste est communiquée aux agences dans l'engagement de confidentialité. La déclaration ou l'autorisation préalable et l'acceptabilité d'un hébergement hors du pays (décision n° 5, H12) sont à traiter par un juriste **avant** de passer de l'essai aux données réelles, ou le risque est accepté par écrit [À VÉRIFIER AVEC UN JURISTE].

### 8.6 Engagement de confidentialité des agences pilotes [À valider]

Contenu **proposé** d'un document simple, de deux pages, signé par chaque agence pilote avant la première donnée réelle. À faire relire par un juriste avant usage [À VÉRIFIER AVEC UN JURISTE]. Ce n'est pas un texte juridique.

| Rubrique | Contenu proposé |
|---|---|
| Parties et objet | HABEK (éditeur) et l'agence pilote ; essai d'Immo Suite avec ses données réelles de loyers, pendant une période et un périmètre définis |
| Données couvertes | Locataires, mandants, loyers, quittances, comptes rendus, documents, utilisateurs de l'agence |
| Qualité des parties | Rôles du responsable du traitement et du sous-traitant [À VÉRIFIER AVEC UN JURISTE] |
| Ce que fait HABEK | N'utilise les données que pour faire fonctionner le service et améliorer le produit **sans les divulguer** ; ne les vend pas ; **n'y accède qu'avec l'accord de l'agence**, pour une durée limitée, avec trace visible de l'agence ; garde son personnel tenu à la confidentialité ; applique les mesures de ce fichier (résumé en une page) |
| Sous-traitants et localisation | Liste des hébergeurs et pays [à compléter après la décision n° 5] |
| Sauvegardes et incident | Sauvegardes chiffrées, restauration testée ; **information de l'agence dans l'heure** en cas d'interruption du service qui touche l'encaissement ; **dans les 24 heures** après confirmation d'une fuite de données personnelles, avec ce qui s'est passé, ce qui est concerné et ce qu'il faut faire [délais proposés] |
| Données de test | Utilisées seulement sous forme **anonymisée**, avec l'accord écrit de l'agence (décision n° 15) |
| Phase de reprise | L'agence peut demander la **remise à zéro** de ses données tant que l'organisation est en phase `setup` (décision n° 25) ; elle l'accepte par écrit |
| Sortie | Export complet remis à la demande ; délai de grâce de 30 jours ; destruction ensuite ; les sauvegardes expirent selon leur calendrier (30 jours, puis 12 copies mensuelles) |
| Obligations de l'agence | Comptes **nominatifs** (aucun partage de mot de passe) ; désactivation immédiate d'un utilisateur parti ; signalement sans délai de la perte d'un téléphone ou d'un soupçon d'accès non autorisé ; information de ses locataires et mandants sur l'utilisation d'un outil de gestion ; confidentialité des liens de partage ; usage de l'outil conforme à la loi et au bail |
| Responsabilité, droit applicable, durée | À rédiger par un juriste |
| Durée | Pendant le pilote, plus la période de sortie |

## 9. Reprise sur incident

### 9.1 Plan de réponse

Le développeur est **commandant d'incident** (celui qui décide) ; le porteur du projet est **scribe** (il écrit chaque action, avec l'heure, dans un journal d'incident) et **contact client**.

| Étape | Contenu | Délai visé |
|---|---|---|
| 1. Détection | Alerte (`ARCHITECTURE.md` §10.3), signalement d'un pilote, constat par l'équipe | — |
| 2. Qualification | Gravité (9.2) ; ce qui est touché : organisations, données, intégrité | 15 minutes |
| 3. **Confinement** | Gel des mises en production ; révocation des sessions concernées ; désactivation d'un compte ; révocation d'un accès support ; blocage d'une route ; au besoin coupure du service plutôt que de laisser fuir | 30 minutes |
| 4. Information | Appel de l'agence pilote touchée (9.3) | **1 heure** pour une interruption de l'encaissement |
| 5. Correction | Correction de la cause ; remplacement des secrets exposés | selon gravité |
| 6. Restauration | Si des données sont perdues ou corrompues : restauration (9.4), **réinjection** par la procédure d'exception (`DATABASE_SCHEMA.md` §8.4) | RTO 4 heures en journée ouvrée |
| 7. Vérification | Cinq requêtes de contrôle (`DATABASE_SCHEMA.md` §7.5), totaux par organisation, numérotation, test d'isolation | avant réouverture |
| 8. Retour d'expérience | Compte rendu écrit sans recherche de coupable : chronologie, cause, ce qui a marché, ce qui change ; une correction au moins est ajoutée à la liste de contrôle | 5 jours |

### 9.2 Gravité et premières actions

Échelle unique S1 à S4 de `TESTING_QA.md` §11.1 (S4, cosmétique, n'a pas d'incident de sécurité associé).

| Gravité | Exemples | Première action |
|---|---|---|
| **S1 — critique** | Fuite entre organisations ; données personnelles exposées ; paiements altérés ; perte de données ; secret de production exposé | Confiner immédiatement ; geler les mises en production ; prévenir les agences concernées |
| **S2 — élevée** | Compte de gérant ou de la console compromis ; indisponibilité de plus d'une heure en journée ; sauvegarde absente depuis plus de 24 heures ; quittance sans PDF depuis plus de 10 minutes | Révoquer les sessions, changer les mots de passe, enquêter par l'audit |
| **S3 — moyenne** | Échecs de connexion répétés ; dépendance vulnérable sans exploitation connue | Traiter dans les 72 heures |

| Incident | Premières mesures spécifiques |
|---|---|
| Fuite entre organisations | Désactiver la route ; chercher dans l'audit `security.cross_org_attempt` ; dénombrer les accès réels (journaux) ; informer les deux organisations |
| Compte volé | Révoquer les sessions, forcer un nouveau mot de passe et réinitialiser le TOTP ; lister dans l'audit tout ce que le compte a fait ; contre-passer ce qui doit l'être **par le gérant légitime** |
| Paiement ou quittance falsifié | Comparer à la dernière sauvegarde ; vérifier l'activation des gardes d'écriture seule ; chercher l'origine dans l'audit et les journaux |
| Secret exposé | Révoquer, remplacer, vérifier les journaux d'usage, redéployer |
| Rançongiciel ou perte du serveur | Reconstruire une instance depuis l'image, restaurer la base (9.4), vérifier, informer |
| Abus du support | Révoquer l'accès, retirer le compte de la console, enquêter par `platform_events` et l'audit, informer l'agence |
| Fichier d'import piégé | Annuler le lot ; isoler le fichier ; vérifier le serveur ; corriger le contrôle |

### 9.3 Communication

- **Aux agences touchées** : appel téléphonique puis message écrit ; **pas par WhatsApp de groupe**. Délais proposés : dans l'heure pour une interruption de l'encaissement ; dans les 24 heures après confirmation d'une fuite de données personnelles [valeurs proposées, à inscrire dans l'engagement de confidentialité].
- **Contenu du message** : ce qui s'est passé (en mots simples), les données concernées, ce que l'équipe a fait, ce que l'agence doit faire (changer un mot de passe, prévenir un locataire), l'heure du prochain point.
- **Locataires et mandants** : informés par l'agence, avec un texte fourni par HABEK.
- **Autorités** : savoir si une notification est obligatoire, à qui et sous quel délai est une question juridique [À VÉRIFIER AVEC UN JURISTE] ; la réponse est écrite dans le plan **avant** le pilote.
- **Hors incident** : un point hebdomadaire de 30 minutes avec chaque pilote (dossier §20.2) sert aussi à signaler les alertes de sécurité mineures.

### 9.4 Sauvegardes et test de restauration mensuel

Protocole **[PILOTE]**, une fois avant le pilote puis chaque mois (dossier §12.1, F39) :

1. Restaurer la dernière copie logique **et** la sauvegarde continue à un instant donné, sur une **instance neuve**, avec les mêmes versions et le même rôle que la production.
2. **Chronométrer** : à comparer au délai cible de 4 heures.
3. Comparer le nombre de lignes de chaque table métier, et la somme des paiements et des affectations **par organisation**, à ceux de la production.
4. Exécuter les cinq requêtes de contrôle (`DATABASE_SCHEMA.md` §7.5), le test d'audit de l'isolation (RLS activée et forcée sur toutes les tables) et le contrôle de présence des gardes d'écriture seule.
5. Vérifier la lecture de quelques PDF et de quelques documents du stockage restauré.
6. Écrire le compte rendu (durée, écarts, correction) dans le dépôt de procédures.

Chaque trimestre : exercice de **perte de la clé de chiffrement des sauvegardes** (la seconde personne retrouve la clé et relit une copie) et exercice de réponse à incident sur table.

## 10. Liste de contrôle de sécurité avant le premier pilote

La liste « prêt pour un pilote » est **unique** : `TESTING_QA.md` §12 (PP-01 à PP-38), avec la preuve attendue pour chaque case. Les exigences de sécurité de ce document y figurent ainsi : isolation, écriture seule, numérotation et idempotence (PP-01 à PP-11), accès support et TOTP des comptes HABEK (PP-12), sauvegardes et restauration (PP-13 à PP-16), mots de passe, sessions, en-têtes, limitation des tentatives et inscription fermée (PP-17), import durci, exports neutralisés et PDF isolé (PP-18, PP-19), dépendances et secrets (PP-20), journaux (PP-21), auto-test d'intrusion (PP-24), engagement de confidentialité (PP-29), hébergement et CIL (PP-30), TOTP du gérant avant la bascule `live` (PP-31). Les vérifications propres à la sécurité sont décrites en section 11.

## 11. Plan de tests de sécurité

| Type | Contenu | Quand | Obligatoire avant des données réelles ? |
|---|---|---|---|
| **Revue de code ciblée** | Modules `identity`, `ledger`, `receipts`, `imports`, `platform` (fichiers, PDF) : lire chaque requête et chaque route avec la liste « isolation, droits, validation, journaux » ; revue de toute nouvelle dépendance | À chaque modification de ces modules ; revue complète avant le pilote | **Oui** |
| Tests d'isolation | Par route, par table, sans contexte, sous concurrence, rôles, fichiers, liens publics, vues (`ARCHITECTURE.md` §5.4, `DATABASE_SCHEMA.md` §6.8) | En CI à chaque fusion | **Oui, bloquants** |
| Tests de permissions | Un test par couple rôle × action ; portée de l'agent terrain | En CI | **Oui** |
| Tests d'authentification | Verrouillage progressif, non-divulgation de l'existence d'un compte, durée de vie des liens, révocation à la désactivation, rejeu d'un code TOTP, réutilisation d'un jeton | En CI | **Oui** |
| Tests de contrats d'argent | Écritures en ajout seul, numérotation, idempotence, contre-passation exacte, avenants (E-01 à E-48) | En CI | **Oui** |
| Tests de fichiers | Archive « bombe », XML avec entité externe, `.xlsm`, fichier trop gros, CSV avec formules, nom de fichier hostile, téléchargement avec type forcé | Avant l'ouverture de l'import ; puis en CI | **Oui** (import) |
| Tests de PDF | Nom contenant du balisage ou une adresse de ressource ; absence de requête sortante ; métadonnées vides ; filigranes | Avant le pilote ; puis en CI | **Oui** |
| Tests de journaux | Recherche de numéros, jetons et corps de requêtes sur un parcours complet | En CI | **Oui** |
| Analyse des dépendances et des secrets | `cargo audit`, `cargo deny`, `npm audit`, détection de secrets | En CI à chaque fusion | **Oui, bloquants** |
| **Auto-test d'intrusion** | Deux jours, par la personne qui n'a pas écrit le module : tenter de lire une autre organisation, escalader un rôle, contourner la limite de connexion, détourner un lien de partage, empoisonner un import, injecter dans les champs libres | Avant le pilote | **Oui** |
| Exercice de restauration | Protocole 9.4 | Avant le pilote, puis chaque mois | **Oui** |
| Exercice d'incident sur table | Scénario « compte du gérant volé » et « fuite entre organisations » | Avant le pilote, puis chaque trimestre | **Oui** (une fois) |
| Analyse dynamique automatique des pages (outil d'analyse de vulnérabilités web, version « de base ») | Contre l'environnement d'essai | Pendant le pilote | Non [APRÈS] |
| **Test d'intrusion externe** | Prestataire spécialisé : authentification, isolation, import, liens publics, console | **Avant la version commerciale** (dossier §19) ; avant le pilote si le budget le permet [À valider, décision n° 58] | Non pour le pilote ; **Oui** pour la version commerciale |
| Revue des politiques de l'hébergeur | Chiffrement, accès du personnel, journaux, localisation | Avant la signature | **Oui** |
| Test de charge raisonnable | Début de mois, 20 utilisateurs simultanés | Avant la phase de stabilisation | Non [APRÈS] |
| Suivi des vulnérabilités | Veille des avis sur les dépendances et sur l'hébergeur ; correctif prioritaire sous 72 heures | En continu | Oui |

Critère de réussite commun : **aucune faille critique ou haute ouverte** (dossier §19) avant d'accueillir de vraies données.

## Questions ouvertes

1. **Inscription publique** (décision n° 51, tranchée par DP-68) : la fermer au pilote contredit le texte de F01 (« création publique »). Est-ce accepté, avec création par la console ?
2. **Double authentification du gérant** (décision n° 50, tranchée par DP-68 : obligatoire avant `live`) : obligatoire avant la bascule en `live` (proposé), ou seulement proposée ? Le dossier la classe P1 sans tranche ; `ARCHITECTURE.md` la reporte [APRÈS].
3. **Durées de session** (décision n° 49) : 30 minutes d'inactivité pour un caissier au comptoir en début de mois est-il supportable ? Il faut en parler avec une vraie agence.
4. **Qui est responsable du traitement** (l'agence ou HABEK) et qui est sous-traitant ? [À VÉRIFIER AVEC UN JURISTE]
5. **Déclaration ou autorisation préalable, et hébergement hors du pays** (décision n° 5, H12) : à traiter avant le passage aux données réelles, ou risque accepté par écrit. [À VÉRIFIER AVEC UN JURISTE]
6. **Durée de conservation des pièces** (10 ans proposés) et **effacement face à l'immuabilité** (section 8.3). [À VÉRIFIER AVEC UN JURISTE]
7. **Obligation de notifier une fuite** à une autorité : à qui, sous quel délai ? [À VÉRIFIER AVEC UN JURISTE]
8. **Code de contrôle sur la quittance** (décision n° 55) : valider l'impression d'un code et d'une adresse de vérification, et la page publique qui n'expire pas.
9. **Lecture par le support des champs chiffrés** : jamais, ou avec mention explicite dans l'accord ?
10. **Second fournisseur de sauvegardes** : propose-t-il des identifiants en écriture seule et un verrouillage d'objet ? Sinon, copie hors ligne mensuelle.
11. **Rôle de sauvegarde avec `BYPASSRLS`** : l'hébergeur le permet-il (`DATABASE_SCHEMA.md` §6.9, décision n° 46) ?
12. **Test d'intrusion externe avant le pilote** : budget et prestataire (décision n° 58).
13. **Restriction par adresse IP de la console** : les connexions de l'équipe sont-elles assez stables ?
14. **Pièce d'identité du locataire** : l'activer un jour ? Si oui, uniquement avec le chiffrement applicatif et l'affichage tracé.
15. **Agence sans autre gérant** : la procédure de récupération du compte de l'unique gérant (3.5) repose sur la vérification par HABEK ; est-elle acceptable pour les agences ?
16. **Message type à remettre aux locataires** pour reconnaître la communication officielle de l'agence [APRÈS] : qui le rédige ?

## Décisions à enregistrer dans DECISIONS.md

Mise à jour du 30/10/2026 : les décisions n° 50, 51 et 60 sont prises par DP-68. Aucune autre décision n'est présentée comme **prise** dans ce fichier : les exigences de sécurité du dossier (§12.1) n'ont pas d'entrée DP-xx et restent donc des recommandations (règle de `DECISIONS.md`). Toutes les décisions ci-dessous sont **ouvertes** [À valider], à ajouter au tableau à la suite de la n° 47 (celles de `DATABASE_SCHEMA.md`). Elles ne ferment aucune décision existante, sauf indication contraire.

| N° | Décision | Recommandation de ce document |
|---|---|---|
| 48 | Mots de passe : argon2id, 19 Mio, 2 itérations, parallélisme 1 (à calibrer) ; 10 caractères au moins, sans règle de composition, liste de mots de passe courants, plafond de 128 caractères | Oui (3.1) |
| 49 | Sessions côté serveur : cookie `__Host-session`, inactivité 30 minutes (60 pour gérant et propriétaire), expiration absolue 12 heures ; console 15 minutes et 8 heures ; révocation immédiate | Oui (3.2) ; durées à tester avec une agence |
| 50 | Double authentification : TOTP obligatoire pour la console HABEK [PILOTE] ; confirmation par mot de passe avant les actions à enjeu [PILOTE] ; TOTP du gérant obligatoire avant la bascule en `live` ; pas de SMS obligatoire | Oui (3.4) |
| 51 | Inscription publique fermée au pilote ; organisations créées par la console | Oui (5.7) |
| 52 | Chiffrement applicatif (AES-256-GCM) de `payout_details_enc`, `totp_secret_enc`, `id_document_enc` ; clé hors base et hors dépôt, rotation prévue | Oui (5.2) |
| 53 | Sauvegardes : identifiants du second fournisseur en écriture seule avec verrouillage d'objet, clé hors hébergeur en deux exemplaires, test mensuel | Oui (5.3). Complète la décision n° 24 |
| 54 | Accès du personnel HABEK : accord du client dans l'application, 24 heures par défaut et 7 jours au plus, aucun accès de routine à la base de production, accès d'urgence à deux personnes, tracé | Oui (5.6). Précise F40 et R-009 |
| 55 | Code de contrôle imprimé sur la quittance et page publique de vérification qui n'expire pas | Oui (7.5), à valider avec les agences |
| 56 | Durées de conservation proposées (6.3) | [À VÉRIFIER AVEC UN JURISTE] avant le pilote (DP-62) |
| 57 | Engagement de confidentialité des agences pilotes : contenu de 8.6 | À faire relire par un juriste ; signé avant la première donnée réelle |
| 58 | Tests de sécurité : éléments obligatoires avant des données réelles (section 11) ; test d'intrusion externe avant la version commerciale | Oui |
| 59 | En-têtes de sécurité et CSP strictes, `Referrer-Policy: no-referrer`, adresses à jeton masquées dans les journaux | Oui (7.3, 7.7) |
| 60 | Récupération de compte : lien généré de 15 minutes, vérification renforcée pour le gérant | Oui (3.5), en cohérence avec la décision n° 27 |

Autres mises à jour proposées : `ARCHITECTURE.md` §3.8 (double authentification du gérant reportée [APRÈS]) a été révisé selon la décision n° 50 (DP-68, 30/10/2026) ; `UI_UX_DESIGN.md` S17 (« 2FA du gérant ») et S01 (inscription) sont à adapter.
