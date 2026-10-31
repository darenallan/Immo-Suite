# Étape 02 — Organisations, connexion et squelette déployé

| Champ | Valeur |
|---|---|
| Statut | À faire |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T1 (F01, F02, F03 : socle) |
| Effort | M, environ 10 JD |
| Dépend de | Étape 01 (CI, essai, base, rôles) ; contenu C-19 si un seul développeur écrit le code (second avis sur les gardes) |
| Jalon | **J1 — Squelette déployé** |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Une personne se connecte sur l'essai déployé en HTTPS et arrive sur l'écran de son rôle ; ses données sont isolées de toute autre organisation par deux barrières ; chaque action sensible laisse une trace en ajout seul ; la matrice des droits est codée une seule fois ; la mise en production et le retour arrière sont exercés une fois.
Valeur : les trois sujets qui ne se reportent jamais (isolation, intégrité, audit) sont en place avant la première table d'argent.

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections |
|---|---|---|
| Fonctionnalités | F01 (organisation, connexion, plans), F02 (matrice codée), F03 (écriture du journal) | dossier §16 F01 à F03 |
| Règles | R-003, R-004, R-007, R-008 | `DOMAINE.md` « Fondamentaux » |
| Isolation | DP-48 | `ARCHITECTURE.md` §5 en entier ; `DATABASE_SCHEMA.md` §6.1 à §6.8 |
| Authentification | DP-68 ; n° 48, 49 | `SECURITY.md` §3.1 à §3.3, §3.2 (cookie `__Host-session`) |
| Droits | DP-66 | `CONTEXTE.md` « Matrice de permissions » ; `API_CONTRACTS.md` §4.6 |
| Audit | DP-47, R-007, R-008 | `SECURITY.md` §6.1, §6.2 ; `DATABASE_SCHEMA.md` §4.2 |
| Contrat | `API_CONTRACTS.md` | §1, §2, §3 (erreurs), §4.1 à §4.3, §4.7, §4.8, §7.2 (login, logout, `/me`), §7.17 |
| Écrans | `UI_UX_DESIGN.md` | « Navigation », S01 (mode connexion), États (vide, chargement, erreur) |
| Tables | `DATABASE_SCHEMA.md` §3 « Identité, accès et organisation » | `organizations`, `users`, `memberships`, `sessions`, `login_attempts`, `audit_log` |

## 3. Prérequis

Étape 01 terminée (J0). Décisions à avoir tranchées : n° 48 (paramètres argon2id, à calibrer sur le serveur retenu), n° 49 (durées de session), n° 59 (en-têtes), n° 22 (PWA : service worker de la coquille seulement ou aucun), n° 45 (rôles `owner` et `collaborator` distincts). Contenu C-19 : personne désignée pour le second avis externe sur les migrations de garde, si DP-54 reste « un développeur ».

## 4. Fiches de spécification à produire ou valider

1. `specs/F01-comptes-organisation.md` (partie de cette étape) — règles : un identifiant = un compte, une organisation possède ses données, cycle de vie `setup` puis `live` (R-110, détail en étape 11), plans `agency` et `owner` (DP-65), création par la console seulement (DP-68) ; cas limites : utilisateur membre de plusieurs organisations, identifiant téléphone `+226` normalisé, organisation suspendue ; écrans : S01 mode connexion, coquille de navigation par rôle ; erreurs : `auth.invalid_credentials` identique que l'identifiant existe ou non, `auth.rate_limited` ; cas chiffrés : 5 échecs puis attente de 1, 5, 15 minutes.
2. `specs/F02-roles-permissions.md` — la matrice de `CONTEXTE.md` et `API_CONTRACTS.md` §4.6 sous forme de table de données (action × rôle), refus par défaut, rôle relu à chaque requête, portée de l'agent terrain ; cas limites : droits réglables (`role_settings`, en étape 03), dernier gérant (R-081, étape 03) ; cas chiffrés : 7 rôles × toutes les actions de §4.6.
3. `specs/F03-journal-audit.md` (écriture) — codes `objet.verbe` de `SECURITY.md` §6.1, valeurs avant et après sans secret, adresse IP, identifiant de requête, échec d'écriture = action annulée (R-008), ajout seul à deux barrières ; cas limites : audit d'une tentative inter-organisations dans l'organisation de l'appelant ; cas chiffrés : un paiement d'essai laisse une ligne avec avant/après.

**S'arrêter pour la validation de Mimis après ces trois fiches.**

## 5. Tâches détaillées

### 5.1 Base de données (migrations en avant seulement)
1. `organizations` (nom, `plan` `agency|owner`, `kind` `real|demo|test`, `lifecycle` `setup|live|suspended|closed` avec retour à `setup` interdit, préfixe, ville, fuseau `Africa/Ouagadougou`, devise `XOF`, langue `fr` ; `row_version` non requis) ; `users` (`login_phone`, `login_email`, `display_name`, `password_hash` avec contrainte `$argon2id$`, `totp_secret_enc`, `is_demo`) ; `memberships` (`role` : `manager`, `property_manager`, `cashier`, `field_agent`, `accountant`, `owner`, `collaborator`, statut) ; `sessions` (`id_hash`, `csrf_hash`, `reauth_until`, dates d'expiration, `ip`) ; `login_attempts` (bigint, 30 jours) ; `audit_log` (bigint, `occurred_at` imposé par la base, `actor_kind` `user|system|support`).
2. RLS activée et forcée avec `org_isolation` sur `memberships` et `audit_log` ; politique de lecture de **ses propres adhésions** avant le choix d'organisation (A8) ; fonction `create_organization()` à droits du propriétaire (A8) ; politiques sur `immo_app` **et** `immo_owner` pour les fonctions de garde.
3. `audit_log` : privilèges `SELECT, INSERT` seulement pour `immo_app` et `immo_platform` ; déclencheur `forbid_mutation` refusant `UPDATE`, `DELETE`, `TRUNCATE` même au propriétaire ; `occurred_at` écrasé par `app_now()`.
4. Un test (`G-05`, `ISO-4`) pour chaque rôle ; contrainte de dernier gérant reportée à l'étape 03.

### 5.2 Noyau métier
1. `crates/app/identity` : `current_actor(session) -> Actor { user_id, org_id, role, plan }`, `authorize(actor, Action)` avec **toutes** les actions de `API_CONTRACTS.md` §4.6 en table de données ; test unitaire généré qui parcourt rôle × action et compare à la matrice (refus par défaut).
2. `identity::login` : vérification argon2id (plafond de 4 vérifications simultanées), vérification factice quand l'identifiant n'existe pas (durée comparable), compteurs dans `login_attempts` (5 échecs : attente 1, 5, 15 minutes ; 20 échecs par IP en 10 minutes), nouvel identifiant de session de 256 bits (seule l'empreinte stockée), cookie `__Host-session` (`Secure`, `HttpOnly`, `SameSite=Lax`, sans `Domain`), expiration par inactivité (30 min ; 60 pour gérant et propriétaire) et absolue (12 h).
3. `audit::record(tx, AuditEntry)` appelée dans la même transaction que l'action (R-008) ; codes `auth.login`, `session.revoked_all`, `security.cross_org_attempt`, `security.rate_limited`.
4. Type `OrgTx` du prototype (b) promu en code produit, extracteur Axum construit depuis la session vérifiée uniquement ; `after_release`/`before_acquire` si la mesure du prototype l'a justifié.
5. CLI : `immo org create --name … --plan agency|owner --prefix …` (état `setup`, `kind` réel) et `immo user create` **refusé si `APP_ENV=production`** (le premier gérant de production arrive par invitation, étape 03).

### 5.3 API (contrat `API_CONTRACTS.md` §7.2)
`POST /auth/login`, `POST /auth/logout`, `POST /auth/logout-all`, `GET /me` (utilisateur, rôle, organisation, `features`, `permissions[]`, `today`, `server_time`, `csrf_token`, `min_client_version`, `settings`), `POST /me/organization` (revérifie l'adhésion). Intergiciels : jeton anti-falsification lié à la session et contrôle d'`Origin` sur toute écriture (403 `auth.csrf_invalid`), `X-Request-Id`, `X-Client-Version`/`X-Min-Client-Version`, format `application/problem+json` avec code stable, en-têtes de sécurité (CSP stricte, HSTS, `nosniff`, `Referrer-Policy: no-referrer`, `Permissions-Policy`, `Cache-Control: no-store`), limitation du débit, ressource d'une autre organisation = 404 `resource.not_found` + ligne d'audit + alerte. Registre des routes comparé au registre des tests (une route sans test d'isolation fait échouer le build).

### 5.4 Interface
Écran S01 (mode connexion : identifiant, mot de passe, états chargement et erreur, message « Trop de tentatives. Réessayez dans {durée} ou réinitialisez votre mot de passe »), coquille de navigation (menu latéral 248 px ≥ 1024 px, barre du bas à 4 entrées dont « Plus » sur mobile, entrées masquées selon le rôle), écran d'accueil minimal par rôle (nom de l'organisation, rôle), menu utilisateur (déconnexion, « déconnecter tous mes appareils »), bandeau « Une mise à jour est disponible ». Textes dans `texts/fr.ts`, quatre états par écran, cibles tactiles 44 px, `lang="fr"`.

### 5.5 Documents PDF
Aucun.

### 5.6 Audit et événements
Lignes d'audit listées en 5.2 ; journaux `tracing` sans donnée personnelle (test A13 : aucun numéro `+226` ni jeton dans les journaux d'un parcours de connexion complet).

### 5.7 Tests
- Isolation : ISO-1 à ISO-9 applicables aux tables créées ; test de pool sous concurrence rejoué en CI (PP-10) ; test « sans contexte » (zéro ligne, insertion refusée) ; test négatif de la variante `SET`.
- API : A1, A2, A4 (via `POST /me/organization`), A6, A11, A12, A13 ; test de génération du contrat (C1 à C3, C5, C6, C10).
- Droits : test généré rôle × action (A3) sur `authorize`.
- Intégrité du journal : `UPDATE`, `DELETE`, `TRUNCATE` refusés pour `immo_app`, `immo_platform`, `immo_owner` ; échec d'audit simulé ⇒ transaction annulée (R-008).
- Parité `Clock` / `app_now()` (DP-50).
- Bout en bout (Playwright, 390 et 1440 px) : connexion, erreur de mot de passe, expiration de session, déconnexion, accès à une adresse interdite (« Accès refusé »).
- Calibrage argon2id : mesure de 100 à 500 ms par vérification sur le serveur d'essai, valeur consignée (n° 48).

## 6. Critères d'acceptation vérifiables

1. Un identifiant inconnu et un mot de passe faux renvoient la même réponse et un temps comparable.
2. Après 5 échecs consécutifs, la 6ᵉ tentative reçoit 429 avec `Retry-After`.
3. Le cookie posé est `__Host-session` avec `Secure`, `HttpOnly`, `SameSite=Lax`, sans `Domain`.
4. Un utilisateur de l'organisation A qui demande l'organisation B obtient 404 `resource.not_found`, aucune donnée de B, une ligne `security.cross_org_attempt` dans l'audit de A et une alerte.
5. `UPDATE`, `DELETE` et `TRUNCATE` sur `audit_log` échouent pour les trois rôles.
6. Si l'écriture d'audit échoue, l'action est annulée (R-008).
7. Le test rôle × action couvre 100 % des couples de la matrice ; toute action absente de la matrice est refusée.
8. Sans contexte d'organisation, toute table métier renvoie 0 ligne ; 0 lecture croisée sur 10 000 itérations de pool.
9. Chaque type de page porte les en-têtes de `SECURITY.md` §7.3 (test A12).
10. La mise en production de la même image puis le retour arrière vers l'image précédente sont exécutés sur l'environnement de production vide, avec procès-verbal (début de PP-22).

## 7. Sécurité et déploiement à vérifier

Inscription publique absente (aucune route `POST /auth/register`) ; secrets distincts essai/production ; `immo user create` impossible en production ; argon2id calibré ; rôle `immo_app` non propriétaire et sans `BYPASSRLS` (PP-08) ; migration de garde (`audit_log`) relue par un second avis (DP-54 ou revue croisée) avant la promotion.

## 8. Liste de démonstration (Mimis)

1. Le développeur crée une organisation et un gérant sur l'essai (`immo org create`, `immo user create`).
2. Sur un téléphone Android, ouvrir l'adresse de l'essai, se connecter avec un mauvais mot de passe : lire le message ; se connecter correctement : arriver sur l'accueil du gérant avec le nom de l'organisation.
3. Se déconnecter ; se reconnecter ; attendre l'expiration d'inactivité raccourcie en essai : voir le retour à la connexion.
4. Lire le rapport de tests d'isolation et de droits (compteurs « 7 rôles × N actions, 0 écart »).
5. Voir le procès-verbal de mise en production puis de retour arrière.

## 9. Définition de terminé, risques, hors étape

Terminé : critères 1 à 10 verts ; routes testées (isolation, droits, contrat) ; migrations rétrocompatibles ; `API_CONTRACTS.md` §9 (journal des changements) mis à jour ; `DECISIONS.md` mis à jour (n° 45, 48, 49, 59, 22) ; `CLAUDE.md` à jour.

| Risque | Mesure |
|---|---|
| Fuite de contexte avec le pool | Prototype (b) rejoué en CI, test négatif conservé |
| Calibrage argon2id trop lent sur un petit serveur | Plafond de 4 vérifications simultanées, paramètres revus selon la mesure |
| Sessions trop courtes pour un caissier | Durées paramétrables, test avec une agence à la découverte |

Hors étape : invitations, changement et réinitialisation de mot de passe, TOTP, confirmation par mot de passe, droits réglables, écrans S17, S18, S21 (étape 03) ; événements d'usage et signalements (étape 04).

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 02
- Dépend de : 01
- Tranche : T1
- Jalon : J1
- F : F01, F02, F03
- R : R-003, R-004, R-007, R-008
- E : aucun
- PP : PP-08, PP-09, PP-17, PP-21, PP-22
- Contenus : C-19
