# F01 — Comptes, organisations, connexion (partie de l'étape 02)

| Champ | Valeur |
|---|---|
| Statut | Partie étape 02 (§1 à §8) **validée par Mimis le 01/11/2026** (DP-85, DP-86). **Partie étape 03 (§9) rédigée et validée par Mimis le 01/11/2026** (recommandations acceptées, DP-88) |
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

---

## 9. Partie de l'étape 03 : invitations, mots de passe, double authentification, confirmation, paramètres

Étape : `roadmap/etape-03-roles-invitations-audit-acces-support.md` §4 point 1. Dépend de : DP-68, DP-85 ; décisions n° 50, 52, 60 ; `SECURITY.md` §3.1, §3.4 à §3.6 ; `API_CONTRACTS.md` §5.2, §7.2, §7.3, §7.13. Règles : R-081, R-082. Critères visés : 1, 2, 8 et 9 de l'étape 03 ; PP-11. Hors fiche : invitation par e-mail ou SMS (aucun au pilote), sélection des lots assignés (étape 05), lien de réinitialisation émis par le support (console, étape 11).

### 9.1 Écarts de schéma (une migration, dite « migration d'identité 2 »)

Le schéma de `DATABASE_SCHEMA.md` ne suffit pas pour quatre besoins de l'étape ; chaque ajout est en avant seulement et rétrocompatible.

| N° | Ajout | Pourquoi |
|---|---|---|
| 1 | `invitations.renewed_from uuid` (clé étrangère composite vers `invitations`) et index unique partiel `(org_id, coalesce(invitee_phone, invitee_email)) WHERE accepted_at IS NULL AND revoked_at IS NULL` | « Renvoyer » crée une ligne qui garde la trace de la précédente ; au plus une invitation en attente par identifiant |
| 2 | `password_resets.revoked_at timestamptz` (droit `UPDATE`) | un nouveau lien annule le précédent sans fausser `used_at` |
| 3 | table **`totp_backup_codes`** (`id`, `user_id`, `code_hash bytea` unique, `used_at`, `created_at`), globale comme `users` (sans RLS, ajoutée à la liste blanche d'ISO-6), droits `SELECT`, `INSERT`, `UPDATE(used_at)` | `DATABASE_SCHEMA.md` ne dit pas où les 10 codes de secours sont stockés |
| 4 | `users.totp_last_step bigint` (droit `UPDATE`) | refuser le rejeu d'un code TOTP (SECURITY §3.4) |
| 5 | `organizations.row_version integer NOT NULL DEFAULT 1` (droit `UPDATE`) | `PATCH /organization` exige `If-Match` (étape 03 §5.3) ; la table n'a pas de version |
| 6 | `support_grants.staff_id` nullable, sans clé étrangère jusqu'à la console | F40 §3 point 4 |

La création d'une organisation (`create_organization()`) crée aussi sa ligne `org_settings` et ses droits par défaut (§9.8). Toutes les tables nouvelles ou touchées gardent RLS forcée et `UNIQUE (org_id, id)` quand elles ont un `org_id`.

### 9.2 Invitations

| N° | Règle |
|---|---|
| 1 | `POST /invitations` (`user.manage` ; **confirmation par mot de passe** si le rôle invité est `manager`) : `{ phone ou email, role, unit_ids[] }`. L'identifiant est normalisé comme à la connexion (règle 2) : forme invalide → `422 validation.phone_format`. Le rôle doit appartenir au plan de l'organisation (règle 6) : sinon `422 validation.failed` |
| 2 | `unit_ids` doit être **vide** à l'étape 03 (les lots n'existent pas, étape 05) : non vide → `422 validation.failed`. La colonne est conservée pour l'étape 05 |
| 3 | Le jeton : 32 octets du générateur du système, encodés en base64url (43 caractères). **Seule l'empreinte SHA-256 est stockée.** La réponse de création est la **seule** qui contienne le lien `{PUBLIC_BASE_URL}/invitation/{jeton}` ; `GET /invitations` ne le renvoie jamais. Pour retrouver un lien perdu : « Renvoyer » |
| 4 | Validité : `expires_at = created_at + 7 jours` exactement. États dérivés : `pending` (non acceptée, non révoquée, `maintenant < expires_at`), `expired`, `accepted`, `revoked` |
| 5 | Un identifiant déjà **membre** de l'organisation (actif ou désactivé) → `409 auth.identifier_taken` : on réactive (`enable`), on n'invite pas. Un identifiant qui a un compte dans **une autre** organisation est invité normalement et **ne le révèle pas** au gérant |
| 6 | Une invitation `pending` existe déjà pour cet identifiant → `409 invitation.already_pending` (code nouveau, §9.12 point 11). Si elle est expirée mais non révoquée, elle est révoquée automatiquement dans la même transaction |
| 7 | `POST /invitations/{id}/renew` : valable pour `pending` ou `expired` ; révoque l'ancienne et crée une ligne avec `renewed_from`, un nouveau jeton et 7 jours à partir de maintenant ; **confirmation** si le rôle est `manager`. Sur `accepted` ou `revoked` : `422 validation.failed` (`params.reason = not_renewable`) |
| 8 | `POST /invitations/{id}/revoke` : `revoked_at`. Le lien cesse de fonctionner à la requête suivante |
| 9 | Une organisation `demo` refuse toute invitation (le déclencheur `memberships_demo_guard` refuserait de toute façon un compte réel) : `422 validation.failed` |
| 10 | `GET /invitations/{jeton}` (public) : `{ organization_name, role, expires_at, identifier_exists }`. Jeton inconnu : `404 resource.not_found`. Invitation expirée, révoquée **ou déjà utilisée** : `410 invitation.expired`, **même corps pour les trois** (on ne dit pas pourquoi à qui tient le lien) |
| 11 | `POST /invitations/accept` (public, par le jeton), dans **une seule transaction** qui verrouille la ligne d'invitation (`FOR UPDATE`) : (a) **nouveau compte** : `{ token, display_name, password }` → crée l'utilisateur (identifiant = celui de l'invitation, non modifiable), l'adhésion avec le rôle de l'invitation, marque l'invitation acceptée, écrit `membership.accepted` et `auth.login`, ouvre la session de l'organisation, répond comme `POST /auth/login` ; (b) **compte existant** : la requête porte la session de ce compte (connexion préalable, S01 mode 1 puis retour au lien) et `{ token }` seul → crée l'adhésion ; la session garde son organisation active, la réponse liste les organisations et l'interface propose d'ouvrir la nouvelle. Compte existant **sans** session : `409 auth.identifier_taken` (« Voulez-vous vous connecter ? »). Session d'un **autre** compte : `403 auth.forbidden` |
| 12 | Deux acceptations simultanées du même jeton : une réussit, l'autre reçoit `410 invitation.expired` (test à deux requêtes concurrentes) |
| 13 | Compte existant désactivé (`users.status = disabled`) : l'acceptation est refusée comme une connexion refusée (`401 auth.invalid_credentials`) |

### 9.3 Mot de passe (règles de choix, changement)

| N° | Règle |
|---|---|
| 1 | **Règles de choix** (SECURITY §3.1) : 10 caractères au moins, 128 au plus (mesurés en caractères Unicode, avant tout calcul argon2id), aucune règle de composition. Refus (`422 auth.password_weak`, avec `params.reason` parmi `too_short`, `too_long`, `common`, `contains_identifier`, `same_as_current`) |
| 2 | `common` : le mot de passe, mis en minuscules et sans espaces, figure dans la liste embarquée de **1 000 mots de passe courants** (listes publiques, plus des termes locaux : « burkina », « ouagadougou », « immosuite », « faso »…). La liste est un fichier du dépôt, versionné et testé |
| 3 | `contains_identifier` : le mot de passe contient les 8 chiffres du téléphone, l'adresse e-mail entière, ou la partie locale de l'e-mail si elle a 4 caractères ou plus (comparaison en minuscules) |
| 4 | `PUT /me/password` `{ current_password, new_password }` : le mot de passe actuel est vérifié (même plafond de 4 vérifications simultanées). Un échec compte dans la limitation de la connexion de cet identifiant (§9.6). Succès : nouveau hachage, `password_set_at`, **toutes les autres sessions de l'utilisateur sont révoquées**, la session courante reçoit un **nouvel identifiant** (cookie renouvelé), ligne `auth.password_changed` dans l'audit de chaque organisation active de l'utilisateur (une seule si une seule) |
| 5 | Le support ne choisit jamais le mot de passe et ne le connaît jamais (SECURITY §3.5) |

### 9.4 Réinitialisation par lien

| N° | Règle |
|---|---|
| 1 | `POST /users/{membership_id}/password-resets` (`user.manage`, **confirmation par mot de passe**) → `{ url, expires_at }`. *Le contrat écrit `/users/{id}` : c'est l'identifiant d'**adhésion**, comme les autres routes `/users/{membership_id}`* (§9.12 point 4) |
| 2 | La cible doit être membre **actif** de l'organisation (sinon `404`). Elle ne peut pas être le demandeur (il utilise `PUT /me/password`). **Si la cible a le rôle gérant (ou propriétaire)** : `403 auth.forbidden` avec le texte « Pour un gérant, contactez le support HABEK » : la procédure à deux personnes de SECURITY §3.5 s'applique (§9.12 point 5) |
| 3 | Jeton : 32 octets en base64url, seule l'empreinte est stockée, affiché **une fois** dans la réponse. `expires_at = created_at + 15 minutes`. Un nouveau lien révoque les précédents non utilisés de la même personne (`revoked_at`) |
| 4 | `POST /auth/password-reset/complete` (public) `{ token, new_password }` : les règles de choix s'appliquent. Jeton inconnu, expiré, utilisé ou révoqué : `410 auth.reset_expired`, **même corps pour tous** ; ne révèle pas si le compte existe. Succès : nouveau hachage, `used_at`, **toutes** les sessions de l'utilisateur révoquées, ligne `auth.reset_used`. L'utilisateur se connecte ensuite avec son nouveau mot de passe (pas de session ouverte par le lien) |
| 5 | Émission : ligne `auth.reset_issued` (objet = l'adhésion visée, jamais le jeton) dans l'audit de l'organisation ; les autres gérants la voient dans S21 |
| 6 | Limitation : **5 liens par heure et par personne visée** (décompte des lignes de `password_resets` de la dernière heure) ; au-delà `429 auth.rate_limited` avec message neutre |
| 7 | `POST /auth/password-reset/complete` : 10 essais par heure et par adresse (jeton aléatoire de 256 bits, aucune devinette possible ; la limite protège le coût de calcul) |
| 8 | Le mot de passe oublié de S01 (mode 3) ne demande rien au serveur : texte « Demandez un lien de réinitialisation à votre gérant » et, pour un gérant, « …ou contactez le support HABEK » |

### 9.5 Double authentification (TOTP)

| N° | Règle |
|---|---|
| 1 | Option proposée au gérant et au propriétaire dès la première connexion (bandeau de l'accueil tant qu'elle n'est pas activée), ouverte à tout rôle. **Obligatoire avant la bascule en `live`** : le contrôle appartient à `immo org go-live` (étape 11) ; cette fiche ne livre que l'état « activée ou non » |
| 2 | `POST /me/totp` : crée un secret de 20 octets (base32), le **chiffre** (AES-256-GCM, nonce de 12 octets par valeur, identifiant de clé en préfixe, clé lue dans l'environnement seulement) dans `users.totp_secret_enc` avec `totp_enabled_at` encore vide, et répond `{ secret_base32, otpauth_url }`. Affiché **une fois** ; rappeler la route avant confirmation remplace le secret en attente. Déjà activée : `422 validation.failed` (`params.reason = already_enabled`) |
| 3 | Paramètres : SHA-1, 6 chiffres, pas de 30 secondes (compatibles avec toutes les applications d'authentification courantes). Émetteur affiché : « Immo Suite », libellé : l'identifiant de connexion |
| 4 | `POST /me/totp/confirm` `{ code }` : le code doit être valide pour le secret en attente (pas courant, un pas avant, un pas après : décalage d'horloge toléré, SECURITY §3.4). Succès : `totp_enabled_at`, **10 codes de secours** `{ backup_codes[10] }` affichés **une fois**, ligne `auth.mfa_enabled` |
| 5 | **Codes de secours** : 12 caractères d'un alphabet de 32 symboles sans ambiguïté (60 bits), affichés par groupes de 4. Stockés sous forme d'empreinte **HMAC-SHA256** avec une sous-clé dérivée de la clé de chiffrement des champs (§9.12 point 3). Usage unique : `UPDATE … SET used_at … WHERE used_at IS NULL` renvoie une ligne ou rien (sûr en concurrence). Ligne `auth.mfa_backup_used` (code nouveau) à chaque usage |
| 6 | **Connexion avec TOTP activé** : à l'étape 5 de F01 §3, si le mot de passe est juste, aucune session n'est créée ; la réponse est `200 { mfa_required: true, challenge }` où `challenge` est un jeton signé (HMAC avec `SESSION_SECRET`), valable **5 minutes**, lié à l'utilisateur et à cet usage seul (§9.12 point 2) |
| 7 | `POST /auth/totp/verify` `{ challenge, code }` : `code` est un code à 6 chiffres **ou** un code de secours. Succès : suite exacte de F01 §3 (choix d'organisation, session, cookie, `auth.login`) ; la ligne `login_attempts` de succès n'est écrite qu'ici. Défi absent, altéré ou expiré : `401 auth.session_expired` ; mauvais code : `401 auth.totp_invalid` |
| 8 | **Rejeu** : `users.totp_last_step` retient le dernier pas accepté ; un code n'est accepté que pour un pas strictement supérieur, mis à jour par `UPDATE … WHERE totp_last_step < $pas` (deux requêtes simultanées avec le même code : une seule réussit) |
| 9 | **Limitation** : 5 essais **ratés** par minute et par utilisateur (compteur dans `login_attempts`, clé `SHA-256("totp:" + identifiant)`, distincte de celle des connexions). Au-delà : `429 auth.rate_limited` avec `Retry-After`, sans tester le code ; l'attente ne compte pas |
| 10 | Pas de route pour **désactiver** le TOTP ni régénérer les codes à l'étape 03 : perte du téléphone ou des codes = procédure du support à deux personnes (SECURITY §3.5), testée à blanc à l'étape 13 (§9.12 point 6) |
| 11 | Interface : clé affichée en groupes de 4 avec le lien `otpauth://` (un clic ouvre l'application d'authentification du même téléphone) ; **pas de QR code** à l'étape 03 (§9.12 point 7) ; les codes de secours s'affichent une fois avec « Copier » et la case « J'ai conservé ces codes » avant de fermer |

### 9.6 Confirmation par mot de passe (« réauth »)

| N° | Règle |
|---|---|
| 1 | `POST /auth/reauth` `{ password }` : mot de passe juste → `sessions.reauth_until = maintenant + 5 minutes` (heure de `Clock`), `204`. Faux → `401 auth.invalid_credentials`, compté dans la limitation de la connexion de l'identifiant (mêmes attentes de 1, 5 et 15 minutes) : une session volée ne peut pas servir à deviner le mot de passe |
| 2 | Une action marquée « réauth » (F02) sans `reauth_until` dans le futur : `403 auth.reauth_required`, **avant tout effet**. L'interface ouvre la modale « Confirmez votre mot de passe », rappelle `POST /auth/reauth`, puis rejoue l'action |
| 3 | La confirmation n'est pas consommée par l'action : elle vaut 5 minutes pour toutes les actions à enjeu suivantes. Elle disparaît avec la session |
| 4 | Actions concernées à l'étape 03 (liste exacte dans F02 §7) : changer le rôle d'un membre, désactiver, générer un lien de réinitialisation, inviter ou renvoyer l'invitation d'un gérant, accorder ou révoquer un accès support, changer le téléphone du compte (`PATCH /me` avec `login_phone`) |
| 5 | L'ouverture de session ne vaut **pas** confirmation : même juste après la connexion, une action à enjeu demande le mot de passe (une session volée vaut une connexion) |

### 9.7 Compte et téléphone

`PATCH /me` `{ display_name, login_phone? }` : 2 à 120 caractères ; changer `login_phone` exige la confirmation (§9.6), la normalisation de la règle 2 et l'unicité (`409 auth.identifier_taken`). Il n'y a pas de vérification du nouveau numéro au pilote (aucun SMS) ; la ligne d'audit `user.updated` — *code nouveau* — écrit « téléphone modifié » sans les numéros. L'ancien numéro cesse de fonctionner immédiatement.

### 9.8 Paramètres de l'organisation

- `GET/PATCH /organization` : `{ name, city }`, `If-Match` sur `organizations.row_version`, `settings.write` pour écrire, `settings.read` pour lire (partiel selon le rôle, F02). Nom : 2 à 120 caractères.
- `GET/PATCH /settings` : `receipt_prefix` (`^[A-Z0-9]{2,8}$`), `receipt_header`, `address`, `legal_ids`, `default_paper` (`a4` ou `a5`), `share_link_ttl_days` (1 à 365, 30 par défaut), `payment_confirm_threshold` (entier FCFA de 1 à 999 999 999, 1 000 000 par défaut, R-038), `version`. `If-Match` sur `org_settings.row_version`. **Sans logo ni signature** (étape 08). Champ inconnu : `422 request.unknown_field`. Un montant décimal ou négatif est refusé (règle d'or 1).
- **Création** : `create_organization()` insère la ligne `org_settings` avec les défauts ci-dessus et un **préfixe provisoire `REC`** (le schéma l'exige non nul et `--prefix` a été retiré de `immo org create`, DP-85). Le préfixe se règle dans S17 **tant qu'aucune quittance n'existe** ; après la première quittance, la règle de changement est la question ouverte de `DATABASE_SCHEMA.md` §11.1 n° 9, tranchée avec la numérotation (étapes 06 à 08).
- `GET /me` renvoie les **vrais** réglages (`payment_confirm_threshold`, `default_paper`, `share_link_ttl_days`) au lieu des valeurs fixes de l'étape 02.
- Chaque modification écrit `settings.updated` (`object_type = settings`, `before`/`after` des seules clés changées).

### 9.9 Écrans

- **S01 mode 2** (`/invitation/{jeton}`) : organisation et rôle de l'invitation, nom affiché, mot de passe avec règles lisibles (« Au moins 10 caractères »), bouton « Rejoindre ». Si `identifier_exists` : « Vous avez déjà un compte. Connectez-vous pour rejoindre {organisation}. ». Invitation expirée : texte de `API_CONTRACTS.md` §3.4.
- **S01 mode 4** (`/reinitialisation/{jeton}`) : nouveau mot de passe, confirmation, bouton « Enregistrer » ; succès : retour à la connexion avec message.
- **S01 mode 1** : après un mot de passe juste avec TOTP, champ « Code de votre application d'authentification » et lien « Utiliser un code de secours ».
- **S17** : sections Organisation, Sécurité (TOTP), Droits réglables (F02), Accès support (F40) ; un seul écran à onglets sur mobile, chaque section dans son accordéon.
- **S18** : liste (nom, identifiant, rôle, statut `actif`, `invité`, `expiré`, `désactivé`), formulaire d'invitation, lien copiable (« Copier » ; « WhatsApp » ouvre `https://wa.me/?text=` avec le message « Bonjour, voici votre invitation à rejoindre {organisation} sur Immo Suite : {lien}. Elle est valable 7 jours. », composé par le navigateur, sans appel au serveur), « Renvoyer », « Changer le rôle », « Désactiver », « Générer un lien de réinitialisation ». Toute action à enjeu passe par la modale de confirmation (UI_UX_DESIGN.md).
- Tous les textes dans `texts/fr.ts` ; aucun jeton dans l'historique du navigateur après usage (la page d'invitation et celle de réinitialisation remplacent leur adresse par `/` à l'ouverture).

### 9.10 Cas limites

| Cas | Comportement |
|---|---|
| Invité qui ouvre le lien sur deux appareils | le premier à valider l'emporte ; l'autre reçoit `invitation.expired` |
| Gérant qui s'invite lui-même | `409 auth.identifier_taken` (déjà membre) |
| Identifiant invité déjà membre d'une autre organisation | invitation créée sans l'indiquer ; à l'acceptation, connexion préalable (règle 11b) |
| Invitation d'un rôle hors plan (`cashier` dans un plan Propriétaire) | `422 validation.failed` |
| Lien de réinitialisation ouvert deux fois | la seconde fois : `410 auth.reset_expired` |
| Lien de réinitialisation, compte désactivé entre-temps | `410 auth.reset_expired` |
| Gérant unique qui perd son téléphone TOTP | aucune route ; procédure du support à deux personnes (SECURITY §3.5), testée à blanc à l'étape 13 |
| Horloge du téléphone décalée d'une minute | le code est accepté (un pas de chaque côté) ; de deux minutes : refusé, message « Vérifiez l'heure de votre téléphone » |
| Code TOTP valide utilisé deux fois | la seconde fois : `401 auth.totp_invalid` (rejeu) |
| Code de secours utilisé deux fois | idem |
| `SESSION_SECRET` changé pendant un défi TOTP | le défi est invalide : `401 auth.session_expired`, l'utilisateur recommence |
| `PUT /me/password` avec le même mot de passe | `422 auth.password_weak`, `reason = same_as_current` |

### 9.11 Cas chiffrés de l'étape 03

| N° | Cas | Attendu |
|---|---|---|
| 1 | Invitation créée le 01/11/2026 à 10:00 | acceptable jusqu'au 08/11/2026 à 09:59:59 ; à 10:00:00 et au 8ᵉ jour : `410 invitation.expired` ; une seconde acceptation avant : `410` |
| 2 | Lien de réinitialisation émis à 10:00:00 | accepté à 10:14:59 ; refusé à 10:15:00 et à la 16ᵉ minute (`410 auth.reset_expired`) ; un second usage : `410` |
| 3 | 6ᵉ lien de réinitialisation en une heure pour la même personne | `429 auth.rate_limited` |
| 4 | Mot de passe de 9, 10, 128, 129 caractères | refusé (`too_short`), accepté, accepté, refusé (`too_long`) |
| 5 | Mot de passe `70123456abc` pour le compte `+226 70 12 34 56` | `422 auth.password_weak`, `contains_identifier` |
| 6 | Code TOTP : pas courant, pas −1, pas +1, pas ±2 | acceptés, acceptés, acceptés, refusé ; un code déjà accepté : refusé |
| 7 | 6 essais TOTP faux en 60 secondes | la 6ᵉ : `429` sans test du code |
| 8 | 10 codes de secours générés, un utilisé | 9 restent ; le même code une seconde fois : refusé |
| 9 | Lecture de `users.totp_secret_enc` en base | octets chiffrés (préfixe de clé + nonce + chiffré) ; le secret en clair n'apparaît ni en base ni dans les journaux |
| 10 | Sessions de l'utilisateur après changement de mot de passe | les autres : `revoked_at` posé ; la courante : nouvel identifiant |
| 11 | `PATCH /organization` avec `If-Match: "3"` alors que la version est 4 | `412 concurrency.version_mismatch`, aucune modification |
| 12 | `payment_confirm_threshold = 1 000 000,5` ou `-1` ou `1 000 000 000` | `422 validation.failed` |

### 9.12 Décisions et questions pour Mimis (étape 03, F01)

Chaque point est une **proposition** ; rien n'est appliqué avant la réponse, puis une entrée `DP-88` (ou suivante) dans `DECISIONS.md`.

1. **Écarts de schéma** (§9.1) : six ajouts par migration. *Recommandation : accepter.* La migration touche des gardes de `support_grants` et d'invitation : second avis (C-19).
2. **Défi TOTP signé au lieu d'une « session partielle »** : le contrat (`API_CONTRACTS.md` §7.2) parle de « session partielle ». Un défi signé de 5 minutes évite une session à demi ouverte que `authorize` devrait comprendre partout et ne laisse aucune ligne `sessions` pour une connexion non terminée. *Recommandation : défi signé* ; le contrat sera mis à jour dans son journal de changements. Autre option : colonne `sessions.mfa_pending` et session partielle.
3. **Codes de secours** par HMAC-SHA256 à clé (60 bits d'entropie) plutôt qu'argon2id : vérifier 10 empreintes argon2id à chaque saisie coûterait 10 calculs coûteux ; le HMAC à clé rend une fuite de la base seule inexploitable. *Recommandation : accepter.* Il crée une dépendance : la clé de chiffrement des champs (copie hors ligne en deux exemplaires, `DEPLOYMENT.md` §2.4) devient aussi nécessaire pour les codes de secours.
4. **`{id}` de `POST /users/{id}/password-resets` = identifiant d'adhésion** (cohérent avec les autres routes). *Recommandation : accepter* ; correction du contrat à noter.
5. **Réinitialisation d'un gérant** : jamais par un autre gérant, toujours par le support à deux personnes (SECURITY §3.5), donc **impossible tant que la console n'existe pas** (étape 11) : d'ici là, c'est une opération manuelle à deux personnes, sur l'instance, notée dans `platform_events`. *Recommandation : accepter.* Autre option : permettre à un gérant de réinitialiser un autre gérant avec confirmation et notification des autres gérants (plus commode, mais un gérant malveillant prend le compte d'un autre).
6. **Pas de désactivation du TOTP ni de régénération des codes** à l'étape 03. *Recommandation : accepter* ; à rouvrir si un pilote perd souvent son téléphone. Risque : un gérant qui a consommé 9 codes sur 10 sans régénération doit passer par le support.
7. **Pas de QR code** : sur un téléphone, l'application d'authentification et le navigateur sont sur le même appareil, donc un QR code ne se scanne pas ; le lien `otpauth://` et la clé saisie à la main suffisent. Un QR n'aiderait que sur ordinateur (poids estimé 8 à 15 Ko compressés sur les 150 Ko, dont 63 déjà utilisés). *Recommandation : reporter à un paquet chargé à la demande si un pilote le réclame.*
8. **Mots de passe courants** : liste de 1 000 entrées issue de listes publiques et de termes locaux (§9.3 point 2). *Recommandation : accepter ; elle grossira avec les constats du pilote.*
9. **Même corps et même code (`invitation.expired`, 410) pour une invitation expirée, révoquée ou utilisée**, texte ajusté : « Cette invitation n'est plus valable. Demandez au gérant de la renvoyer. » (le texte du contrat dit « a expiré (validité : 7 jours) »). *Recommandation : accepter.*
10. **Préfixe provisoire `REC`** à la création d'organisation (§9.8). *Recommandation : accepter.*
11. **Codes d'erreur et d'audit nouveaux** : `invitation.already_pending` (409, « Une invitation est déjà en attente pour cet identifiant. Renvoyez-la. ») ; audit `invitation.revoked`, `invitation.renewed`, `user.enabled`, `user.updated`, `auth.mfa_backup_used`, ajoutés à `SECURITY.md` §6.1 et à l'énumération Rust. *Recommandation : accepter.*
12. **Définition de « réauth »** : voir F02 §7 (liste exacte des actions et écart entre l'étape, le contrat et SECURITY).
