# F01 — Comptes, organisations, connexion (partie de l'étape 02)

| Champ | Valeur |
|---|---|
| Statut | **Proposée le 01/11/2026, en attente de validation de Mimis** (aucun code écrit) |
| Étape | 02, `roadmap/etape-02-organisations-connexion-squelette.md` §4 point 1 |
| Dépend de | DP-48, DP-65, DP-68, DP-77 ; décisions n° 22, 45, 48, 49, 59 (à trancher, §8) ; `SECURITY.md` §3.1 à §3.3 ; `DATABASE_SCHEMA.md` §3 |
| Règles | R-003, R-004 (`DOMAINE.md`) |
| Critères visés | critères 1 à 4 et 8 à 10 de l'étape 02 ; PP-08, PP-09, PP-17 |
| Hors fiche | invitations, mot de passe changé ou réinitialisé, TOTP, confirmation par mot de passe, droits réglables, écrans S17, S18, S21 (étape 03) |

## 1. Objet

Une personne se connecte avec un téléphone `+226` ou un e-mail et un mot de passe, arrive sur l'écran de son rôle, dans **une** organisation à la fois. Ses données sont isolées par la base (RLS forcée) **et** par le code (`OrgTx`). Il n'y a pas d'inscription publique : les organisations et les premiers comptes se créent en ligne de commande.

## 2. Règles

| N° | Règle | Source |
|---|---|---|
| 1 | Un identifiant = un compte (`users`, table globale). Le même compte peut être membre de plusieurs organisations (`memberships`, un rôle par organisation) | F01, DB §3 |
| 2 | Identifiant : un `@` indique un e-mail (mis en minuscules), sinon un téléphone normalisé en `+226` suivi de 8 chiffres. Formes acceptées en saisie : `70 12 34 56`, `70123456`, `+226 70 12 34 56`, `00226 70 12 34 56`, `226 70123456`. Tout le reste est refusé (`validation.phone_format` à la création d'un compte ou d'une invitation ; à la **connexion**, traité comme un identifiant inconnu) | R-018, DB `users` |
| 3 | Mot de passe : argon2id uniquement (la base refuse tout autre préfixe) ; 10 caractères au moins, 128 au plus, pas de règle de composition. Les règles de choix sont appliquées au changement de mot de passe (étape 03) ; la ligne de commande de l'étape 02 applique déjà les mêmes règles | SECURITY §3.1 |
| 4 | Aucune route d'inscription. `POST /auth/register` répond 404 | DP-68, API §4.1 |
| 5 | Une organisation a un plan (`agency` ou `owner`), un type (`real`, `demo`, `test`), un cycle de vie (`setup` → `live` → `suspended`/`closed`, **jamais de retour à `setup`**, imposé par la base), le fuseau `Africa/Ouagadougou`, la devise `XOF`, la langue `fr` (valeurs uniques en V1, imposées par la base) | R-003, DP-61, DP-65 |
| 6 | Rôles autorisés par plan : `agency` → `manager`, `property_manager`, `cashier`, `field_agent`, `accountant` ; `owner` → `owner`, `collaborator` (décision n° 45 proposée, §8). Contrôlé par le code ; la base accepte les sept valeurs | DB `memberships` |
| 7 | Les comptes de démonstration (`is_demo`) n'entrent que dans une organisation `demo`, et inversement (déclencheur `memberships_demo_guard`) | DB |
| 8 | L'organisation active est **uniquement** celle de la session. Elle n'est jamais lue dans l'adresse, le corps ou un en-tête. Changer d'organisation passe par `POST /me/organization`, qui revérifie l'adhésion | API §4.3 |
| 9 | Le rôle, l'état de l'adhésion (`active`), l'état du compte (`active`) et le cycle de vie de l'organisation sont **relus en base à chaque requête**. Un rôle retiré ou un compte désactivé agit à la requête suivante, sans reconnexion | SECURITY §3.2, §4.2 |
| 10 | Créer une organisation : `immo org create --name … --plan agency|owner`. Créer un utilisateur : `immo user create`, **refusé si `APP_ENV=production`** (le premier gérant de production arrive par invitation, étape 03) | étape 02 §5.2 |

## 3. Connexion

**Étapes de `POST /auth/login`** (`{ identifier, password }`) :

1. Normaliser l'identifiant (règle 2). Forme invalide : même réponse que « identifiant inconnu » (pas de `validation.*` qui révélerait quelque chose).
2. Contrôler la limitation (§4). Si l'identifiant ou l'adresse est en attente : `429 auth.rate_limited` avec `Retry-After`, **sans vérifier le mot de passe**.
3. Vérifier argon2id. **Plafond de 4 vérifications simultanées** (les suivantes attendent). Si le compte n'existe pas, désactivé ou sans mot de passe : vérification factice sur une empreinte fixe de mêmes paramètres, pour un temps comparable.
4. Échec : une ligne dans `login_attempts` (empreinte de l'identifiant, adresse, `success = false`), réponse `401 auth.invalid_credentials`, **identique** que l'identifiant existe ou non.
5. Succès : ligne `login_attempts` (`success = true`) ; chercher les adhésions actives de l'utilisateur dans des organisations non `closed`.
   - **Aucune adhésion active** : `403 auth.no_active_membership` (code nouveau, §8 point 6). Aucune session n'est créée.
   - **Une seule** : elle devient l'organisation active de la session.
   - **Plusieurs** : la session est créée **sans organisation active** ; la réponse liste les organisations ; tout appel autre que `GET /me`, `POST /me/organization` et `POST /auth/logout` répond `403 auth.organization_required` (code nouveau).
6. Créer la session : identifiant de **256 bits** aléatoires, seule l'empreinte SHA-256 est stockée ; cookie `__Host-session` (`Secure`, `HttpOnly`, `SameSite=Lax`, chemin `/`, **sans** `Domain`). Un nouvel identifiant est créé à chaque connexion.
7. Écrire `auth.login` dans l'audit de l'organisation active, dans la même transaction (F03). Si aucune organisation n'est active (cas « plusieurs »), la ligne est écrite au premier `POST /me/organization` de la session.
8. Réponse `200` : `{ user, organizations[], csrf_token, mfa_required: false }` (TOTP en étape 03).

**Réponse en cas d'organisation suspendue** : la connexion réussit, mais toute route d'organisation répond `403 org.suspended` (code nouveau, §8 point 6), sauf `GET /me`, `POST /auth/logout`, `POST /auth/logout-all`. Une organisation `closed` n'est pas proposée.

**Durées** (propositions de `SECURITY.md` §3.2, décision n° 49) : inactivité 30 minutes (caissier, gestionnaire, agent terrain, comptable, collaborateur) ou 60 minutes (gérant, propriétaire) ; expiration absolue 12 heures. Pas de « se souvenir de moi ». La durée d'inactivité suit le rôle **de l'organisation active**. En essai, les durées sont paramétrables par variable d'environnement (démonstration de l'expiration).

**Jeton anti-falsification** : valeur dérivée de l'identifiant de session (HMAC-SHA256 avec `SESSION_SECRET`), renvoyée par la connexion et `GET /me`, exigée dans `X-CSRF-Token` sur toute requête qui écrit, avec un `Origin` identique à l'hôte de l'application. La colonne `csrf_hash` stocke l'empreinte du jeton à la création (contrôle de cohérence). Changer `SESSION_SECRET` invalide tous les jetons (§8 point 3). Absent ou faux : `403 auth.csrf_invalid` et une ligne `security.permission_denied`.

**Déconnexion** : `POST /auth/logout` marque la session `revoked_at` ; `POST /auth/logout-all` marque **toutes** les sessions de l'utilisateur (ligne d'audit `session.revoked_all`) ; le cookie est effacé. Un `GET` ne modifie jamais l'état.

**`GET /me`** : utilisateur, rôle, organisation (nom, plan, type, cycle de vie, fuseau, devise), `features`, `permissions[]` (calculées par `authorize`, F02), `today` (date du jour au fuseau de l'organisation, par `Clock`), `server_time`, `csrf_token`, `min_client_version`, `settings`. Aucune donnée d'une autre organisation.

**`POST /me/organization`** (`{ organization_id }`) : l'adhésion active est revérifiée. Pas de membre actif de cette organisation : `404 resource.not_found`, sans donnée, avec la ligne `security.cross_org_attempt` de F03. Sinon l'organisation de la session change, un nouvel identifiant de session est émis (rotation) et la réponse est celle de `GET /me`.

## 4. Limitation des tentatives (compteurs dans PostgreSQL)

| Cible | Règle | Effet |
|---|---|---|
| Par identifiant | échecs **consécutifs** depuis la dernière réussite : 5ᵉ échec → attente de **1 minute** ; 6ᵉ → **5 minutes** ; 7ᵉ et suivants → **15 minutes** | `429 auth.rate_limited` + `Retry-After` en secondes ; pas de verrouillage définitif |
| Par adresse | 20 échecs en 10 minutes | refus pendant 15 minutes |
| Pendant une attente | la requête est refusée **sans** vérifier le mot de passe et **ne compte pas** comme un échec | pas de prolongation par un tiers qui insiste |

Détails : l'attente part de l'heure du dernier échec ; l'identifiant est suivi par son empreinte (`identifier_hash`), qu'il existe ou non, pour que la réponse ne révèle rien. Les colonnes `users.failed_logins` et `users.locked_until` ne sont **pas utilisées** (elles ne couvrent pas les identifiants inconnus) ; leur retrait est proposé à une étape ultérieure (§8 point 7). L'événement `security.rate_limited` est écrit dans l'audit de chaque organisation du compte quand l'identifiant correspond à un compte, nulle part sinon (aucune organisation à laquelle le rattacher) [§8 point 5]. Les lignes de `login_attempts` sont purgées après 30 jours (tâche de fond).

## 5. Écrans (interface)

- **S01, mode connexion** : identifiant, mot de passe, bouton avec état de chargement. Erreurs : « Identifiant ou mot de passe incorrect. » ✱ ; « Trop de tentatives. Réessayez dans {durée} ou réinitialisez votre mot de passe. » ; « Votre session a expiré. Reconnectez-vous. » ✱. Le lien « Mot de passe oublié » affiche « Demandez un lien de réinitialisation à votre gérant » (mode 3 de S01).
- **Sélection d'organisation** si plusieurs (liste des organisations, rôle de chaque adhésion).
- **Coquille de navigation** : menu latéral de 248 px à partir de 1024 px ; sinon barre du haut et barre du bas à **4 entrées**, la dernière « Plus ». Entrées **masquées selon le rôle** (tableau de `UI_UX_DESIGN.md` « Navigation »), refus par défaut pour toute entrée que la matrice ne définit pas. Seules « Accueil » et le menu utilisateur existent à l'étape 02 ; les autres entrées sont des repères inactifs « À venir ».
- **Accueil minimal par rôle** : nom de l'organisation et rôle. **Menu utilisateur** : nom, rôle, organisation, « Déconnexion », « Déconnecter tous mes appareils ». **Bandeau** « Une mise à jour est disponible » (`client.upgrade_required`).
- Quatre états par écran, cibles tactiles de 44 px, `lang="fr"`, textes dans `texts/fr.ts`, aucune police téléchargée.

## 6. Cas limites

| Cas | Comportement |
|---|---|
| Même téléphone saisi sous deux formes | normalisé en une seule valeur : même compte |
| E-mail avec majuscules | mis en minuscules avant la recherche |
| Utilisateur membre de deux organisations | session sans organisation active jusqu'au choix ; `auth.login` écrit au choix |
| Membre d'une organisation `suspended` et d'une `live` | les deux sont listées ; la suspendue répond `org.suspended` |
| Adhésion désactivée pendant une session | la requête suivante échoue (`401 auth.session_expired`) : la session est révoquée |
| Compte désactivé pendant une session | idem |
| Rôle changé pendant une session | le nouveau rôle s'applique à la requête suivante ; l'identifiant de session change (étape 03) |
| Cookie volé puis session révoquée | refus immédiat (ligne `revoked_at`) |
| Deux onglets, un seul se déconnecte | l'autre reçoit `401` à sa prochaine requête |
| `POST /me/organization` vers une organisation inconnue ou non membre | `404 resource.not_found` identique dans les deux cas |
| `immo user create` en production | refus avec message ; code de sortie non nul |
| Mot de passe de 129 caractères | refusé avant tout calcul argon2id |
| Rafale de 100 connexions simultanées | 4 vérifications argon2id à la fois, les autres attendent ; aucun dépassement mémoire |

## 7. Cas chiffrés (critères de l'étape)

| N° | Cas | Attendu |
|---|---|---|
| 1 | Identifiant inconnu / mot de passe faux, même corps de requête | même statut `401`, même corps, durées du même ordre (écart médian ≤ 20 %) |
| 2 | 5 échecs puis une 6ᵉ tentative (bon mot de passe) | `429` + `Retry-After` ≈ 60 s ; après 1 minute, une tentative juste réussit et remet le compteur à zéro |
| 3 | 5ᵉ, 6ᵉ, 7ᵉ échecs puis 8ᵉ | attentes 1, 5, 15 minutes puis 15 minutes |
| 4 | 20 échecs depuis une adresse en 10 minutes, identifiants différents | refus de la 21ᵉ pendant 15 minutes |
| 5 | Cookie posé à la connexion | nom `__Host-session`, `Secure`, `HttpOnly`, `SameSite=Lax`, `Path=/`, sans `Domain` |
| 6 | Un utilisateur de A demande l'organisation B | `404 resource.not_found`, aucun champ de B, une ligne `security.cross_org_attempt` dans l'audit de A |
| 7 | Sans contexte d'organisation, toute table protégée | 0 ligne ; insertion refusée |
| 8 | 10 000 itérations de pool (50 tâches) | 0 lecture croisée |
| 9 | `POST /auth/register` | 404 |
| 10 | Un parcours de connexion complet, journaux relus | aucun numéro `+226`, mot de passe, jeton ni corps de requête (test A13) |

## 8. Décisions et questions pour Mimis

Chaque point est une **proposition** ; rien n'est appliqué avant la réponse, puis une entrée `DP-85` (ou suivante) dans `DECISIONS.md`.

1. **n° 48, mots de passe** : argon2id, 19 Mio, 2 itérations, parallélisme 1 comme valeur de départ ; calibrage à 100–500 ms **sur le serveur d'essai quand il existera** (mesure locale consignée dès maintenant, valeur de l'essai à confirmer). Plafond de 4 vérifications simultanées. *Recommandation : accepter.*
2. **n° 49, sessions** : durées de la section 3 (30/60 minutes, 12 heures). *Recommandation : accepter, à confirmer avec les caissiers au pilote.*
3. **Jeton anti-falsification** : la fiche dit que seule l'empreinte est stockée, mais `GET /me` doit renvoyer le jeton, et `immo_app` n'a pas le droit de modifier `csrf_hash`. *Proposition : jeton dérivé par HMAC de l'identifiant de session avec `SESSION_SECRET`, empreinte stockée à la création. Changer `SESSION_SECRET` rend tous les jetons invalides (cohérent avec « révoquer toutes les sessions » de `DEPLOYMENT.md` §5.4).* Autre option : autoriser `UPDATE(csrf_hash)` et changer le jeton à chaque `GET /me`.
4. **Plan et rôles (n° 45)** : `owner` et `collaborator` sont des rôles distincts réservés au plan Propriétaire ; les cinq autres au plan Agence. *Recommandation : accepter.*
5. **`security.rate_limited` dans l'audit** : écrit dans les organisations du compte quand il existe, nulle part sinon. *Recommandation : accepter ; sinon, dire où l'écrire.*
6. **Trois codes d'erreur nouveaux** (textes ✱ à valider) : `auth.no_active_membership` (403), `auth.organization_required` (403), `org.suspended` (403). *Recommandation : accepter ; les ajouter à `API_CONTRACTS.md` §3.4.*
7. **Colonnes inutilisées** `users.failed_logins` et `users.locked_until` : laissées vides à l'étape 02 ; retrait par une migration ultérieure. *Recommandation : accepter.*
8. **`--prefix` de `immo org create`** : la fiche d'étape cite ce paramètre, mais `organizations` n'a pas de colonne de préfixe (le préfixe appartient aux séries de numérotation, DP-61, étape 06 ou 07). *Proposition : retirer `--prefix` à l'étape 02 ; le préfixe sera demandé avec les séries.* Écart à noter dans `ROADMAP.md`.
9. **n° 22, application installable** : *proposition : aucun service worker à l'étape 02* (jamais de cache d'API ni d'écriture hors ligne, `ARCHITECTURE.md` §3.6) ; la coquille hors ligne est décidée à l'étape où elle sert.
10. **n° 59, en-têtes** : ceux de `SECURITY.md` §7.3, tels quels, avec `Cache-Control: no-store` sur toute l'API. *Recommandation : accepter.*
11. **Essai déployé** : les points 1, 2, 3 et 5 de la démonstration et le critère 10 de l'étape exigent un essai déployé, qui dépend de la dette de l'étape 01 (C-01, C-02, C-03). Le code et les tests locaux ne l'attendent pas ; la démonstration sera faite en local en attendant, **ce qui n'est pas** le critère 10.
