# Étape 03 — Rôles, invitations, journal lisible et accord d'accès support

| Champ | Valeur |
|---|---|
| Statut | À faire |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T1 (F01, F02, F03 : suite) |
| Effort | L, environ 15 JD |
| Dépend de | Étape 02 |
| Jalon | Contribue à J3 |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Le gérant invite son équipe par un lien, règle les droits, active sa double authentification, accorde (et révoque) l'accès de l'équipe HABEK, et lit le journal d'audit. Le caissier ne voit que ce que la matrice lui permet, vérifié côté serveur.
Valeur : l'agence peut donner de vrais comptes nominatifs à ses employés sans partage de mot de passe, et garder le contrôle de qui voit ses données.

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections |
|---|---|---|
| Fonctionnalités | F01 (invitations, mot de passe, plans), F02 (rôles, droits réglables), F03 (lecture, filtres) | dossier §16 |
| Règles | R-009 (accord support), R-081 (droits côté serveur, dernier gérant), R-082 (plan) | `DOMAINE.md` |
| Authentification | DP-68 ; n° 48, 50, 60 | `SECURITY.md` §3.1, §3.4 à §3.6 |
| Droits | DP-66 | `SECURITY.md` §4.1 à §4.3 ; `API_CONTRACTS.md` §4.6 |
| Accès support (côté client) | n° 38, 54 | `SECURITY.md` §5.6 ; `DATABASE_SCHEMA.md` §6.6 |
| Audit | R-007 | `SECURITY.md` §6.1, §6.4 |
| Contrat | `API_CONTRACTS.md` | §5.2 (`row_version`), §7.2, §7.3, §7.12 (`GET /audit`), §7.13 |
| Écrans | `UI_UX_DESIGN.md` | S01 (modes 2 à 4), S17, S18, S21, « Modale de confirmation » |
| Tables | `DATABASE_SCHEMA.md` §3 | `invitations`, `password_resets`, `role_settings`, `unit_assignments`, `support_grants`, `org_settings` |

## 3. Prérequis

Étape 02. Décisions : n° 50 et 60 (prises par DP-68), n° 54 (accord du client : 24 h par défaut et 7 jours au plus, à confirmer), n° 52 (chiffrement applicatif du secret TOTP, AES-256-GCM, clé hors base). Clé de chiffrement des champs créée par environnement et copiée hors ligne en deux exemplaires (`DEPLOYMENT.md` §2.4).

## 4. Fiches de spécification à produire ou valider

1. `specs/F01-comptes-organisation.md` (compléter) — invitations (lien à usage unique, 7 jours, jeton de 256 bits dont seule l'empreinte est stockée), mot de passe (10 caractères, 128 au plus, liste de mots de passe courants, interdit de contenir l'identifiant), réinitialisation par lien de 15 minutes généré par le gérant ou le support, TOTP (secret chiffré, décalage d'un pas toléré, rejeu refusé, 10 codes de secours), confirmation par mot de passe valable 5 minutes (`sessions.reauth_until`). Cas limites : invité déjà membre d'une autre organisation, invitation expirée ou révoquée, gérant unique qui perd son téléphone TOTP (procédure `SECURITY.md` §3.5 : support avec deux personnes), identifiant déjà utilisé. Cas chiffrés : invitation expirée au 8ᵉ jour ; lien de réinitialisation refusé à la 16ᵉ minute.
2. `specs/F02-roles-permissions.md` (compléter) — droits réglables `record_payment`, `import_data`, `export_data` pour gestionnaire et collaborateur ; effet sans reconnexion ; le dernier gérant actif ne peut pas être désactivé ni rétrogradé (R-081) ; agent terrain : lots assignés (écran en étape 05). Cas chiffrés : agent terrain assigné à 10 lots ne voit que ces 10 lots.
3. `specs/F03-journal-audit.md` (compléter) — lecture : filtres utilisateur, objet, action, période, `actor_kind` ; adresse IP visible du gérant et du comptable seulement ; export du journal par le gérant tracé (`export.created`, ouvert en étape 23).
4. `specs/F40-acces-support.md` — accord donné dans S17 : portée `read` ou `import_assist`, motif de 5 caractères au moins, durée choisie (24 h par défaut, 7 jours au plus), révocation immédiate, historique visible (qui, quand, combien de temps) ; l'accès réel par la console est en étape 11. Cas limites : accord expiré, révoqué, deux accords simultanés.

**S'arrêter pour la validation de Mimis après ces fiches.**

## 5. Tâches détaillées

### 5.1 Base de données
1. `invitations` (`token_hash`, `role`, `unit_ids`, `expires_at`, statut `pending|accepted|revoked`, `renewed_from`) ; `password_resets` (`token_hash`, `expires_at` +15 min, `used_at`, globale) ; `role_settings` (`row_version`, rôle, action, `allowed`) ; `unit_assignments` (clés étrangères composites vers `units` ajoutées en étape 05) ; `support_grants` (portée, motif, `expires_at` ≤ 7 jours imposé par `CHECK`, `revoked_at` ; colonnes modifiables limitées) ; `org_settings` (`row_version`, préfixe, en-tête, adresse, identifiants légaux, format de papier, durée des liens, seuil de confirmation) ; codes de secours TOTP : vérifier dans `DATABASE_SCHEMA.md` §3 (`users`) où sont stockées leurs empreintes, sinon ajouter une table par migration et le noter dans ce document.
2. Garde « dernier gérant » (G-16) : déclencheur refusant la désactivation ou le changement de rôle du dernier gérant actif ; garde « mélange comptes de démonstration et comptes réels » (`is_demo`).
3. Politique RLS dédiée à l'accès support (`DATABASE_SCHEMA.md` §6.6) : lecture et insertion pour `immo_platform` seulement sous un accord valide, non expiré, non révoqué ; test ISO-8.
4. Toutes les tables avec `org_id`, RLS forcée, `UNIQUE (org_id, id)` ; migration qui touche une garde = second avis.

### 5.2 Noyau métier
1. `identity` : `invite`, `accept_invitation`, `renew`, `revoke`, `change_role`, `disable`/`enable` (révoque toutes les sessions), `set_password`, `issue_password_reset`, `complete_password_reset` (révoque toutes les sessions de l'utilisateur), `enable_totp`, `verify_totp`, `reauth`, `grant_support`, `revoke_support`.
2. `authorize` lit `role_settings` ; plan Propriétaire : deux profils (`owner` tous droits, `collaborator` : enregistrer, relancer, lecture) ; actions réservées au plan Agence répondent `plan.feature_unavailable` (R-082).
3. Audit : `user.invited`, `membership.accepted`, `membership.role_changed`, `user.disabled`, `auth.password_changed`, `auth.mfa_enabled`, `auth.reset_issued`, `auth.reset_used`, `session.revoked_all`, `settings.updated`, `support.access_granted`, `support.access_revoked` ; notification aux autres gérants lors d'un changement de droits d'un gérant (ligne d'audit visible dans S21).
4. Chiffrement applicatif AES-256-GCM (nonce par valeur, identifiant de clé en préfixe) du secret TOTP ; lecture de la clé depuis l'environnement uniquement.

### 5.3 API (`API_CONTRACTS.md` §7.2, §7.3, §7.12, §7.13)
`PUT /me/password`, `PATCH /me` 🔐, `POST /me/totp`, `POST /me/totp/confirm`, `POST /auth/totp/verify`, `POST /auth/reauth`, `POST /users/{id}/password-resets` 🔐, `POST /auth/password-reset/complete`, `GET /invitations/{token}`, `POST /invitations/accept`, `GET/PATCH /organization`, `GET /users`, `POST/GET /invitations`, `POST /invitations/{id}/revoke|renew`, `PATCH /users/{membership_id}` 🔐, `POST /users/{membership_id}/disable|enable` 🔐, `PUT /users/{membership_id}/unit-assignments`, `GET/PUT /role-settings`, `GET/POST /support-grants` 🔐, `POST /support-grants/{id}/revoke` 🔐, `GET /audit`, `GET/PATCH /settings` (sans logo), gestionnaires de refus `PUT|PATCH|DELETE /audit/{id}` ⇒ 405 `audit.immutable`. `If-Match` sur `PATCH /organization`, `/settings`, `PUT /role-settings`. Limitation : réinitialisation et invitation 5 par heure et par identifiant, TOTP 5 essais par minute.

### 5.4 Interface
S01 : modes « accepter l'invitation », « mot de passe oublié » (texte « Demandez un lien de réinitialisation à votre gérant »), « nouveau mot de passe » ; S17 : sections Organisation (nom, ville, en-tête, préfixe de quittance, durée des liens, seuil de confirmation), Sécurité (activation TOTP : clé affichée une fois et lien `otpauth`, codes de secours affichés une fois ; QR code facultatif en paquet séparé si le budget de 150 Ko le permet [À valider]), Droits réglables, Accès support (accorder, révoquer, historique) ; S18 : liste (actif, invité, expiré), formulaire d'invitation avec lien copiable (bouton « Copier », ouverture WhatsApp), renvoyer, changer le rôle, désactiver (le sélecteur de lots assignés s'active à l'étape 05) ; S21 : filtres et lecture seule ; modale de confirmation par mot de passe. Textes dans `texts/fr.ts`.

### 5.5 Documents PDF
Aucun.

### 5.6 Audit et événements
Voir 5.2 point 3. Aucun événement d'usage (étape 04).

### 5.7 Tests
- Droits : A3 sur toutes les routes de l'étape ; droits réglables testés dans les deux états ; effet immédiat d'un changement de rôle sans reconnexion.
- Authentification : jetons d'invitation et de réinitialisation à usage unique, seule l'empreinte en base, expiration (7 jours, 15 minutes), rejeu d'un code TOTP refusé, 5 essais TOTP par minute, mot de passe courant refusé, sessions révoquées après changement de mot de passe, désactivation et retrait de rôle.
- Base : G-16, ISO-8 (accès support refusé sans accord, accordé, révoqué, expiré, durée maximale 7 jours), `support_grants` non modifiable hors colonnes autorisées.
- E2E : parcours invitation → premier mot de passe → connexion du caissier sur 390 px ; gérant : changer un rôle, désactiver, générer un lien de réinitialisation.
- A13 : aucun jeton dans les journaux ni dans les journaux du proxy.

## 6. Critères d'acceptation vérifiables

1. Un lien d'invitation s'utilise une fois ; le 8ᵉ jour il répond `invitation.expired` (410).
2. Un lien de réinitialisation expire à la 16ᵉ minute et ne sert qu'une fois.
3. Le dernier gérant actif ne peut être ni désactivé ni rétrogradé (`user.last_manager`).
4. Un caissier qui appelle `POST /payments/{id}/reversal` (ou toute action interdite) reçoit 403 côté serveur ; 100 % des couples interdits sont refusés.
5. Après changement de rôle, l'utilisateur voit ses droits à jour à l'actualisation, sans reconnexion.
6. Un accès support sans accord valide ne renvoie aucune ligne ; avec accord de plus de 7 jours, la création est refusée.
7. `GET /audit` n'expose l'adresse IP qu'au gérant et au comptable ; `PUT|PATCH|DELETE` répondent 405.
8. Le secret TOTP n'est jamais lisible en base ; un code de secours ne sert qu'une fois.
9. Les actions à enjeu (contre-passation, changement de rôle d'un gérant, désactivation, accord support) répondent `403 auth.reauth_required` sans confirmation datant de moins de 5 minutes.

## 7. Sécurité et déploiement à vérifier

Aucun mot de passe, jeton ni secret dans les journaux ; chiffrement des champs vérifié par lecture directe de la colonne ; limites de débit actives ; migration de `support_grants` et de la garde « dernier gérant » relue par un second avis ; nouvelle clé de chiffrement des champs créée en essai et en production, sauvegardée à part en deux exemplaires.

## 8. Liste de démonstration (Mimis)

1. En gérant, inviter un caissier par son numéro : copier le lien et l'ouvrir sur un second téléphone ; choisir le mot de passe ; voir que le caissier n'a que les entrées de menu permises.
2. Passer le caissier gestionnaire : actualiser sur le second téléphone, sans se reconnecter, voir le menu changer.
3. Tenter de désactiver le seul gérant : lire le refus expliqué.
4. Générer un lien de réinitialisation pour le caissier (confirmer par mot de passe) et attendre 16 minutes en essai raccourci : lien refusé.
5. Activer la double authentification du gérant avec une application d'authentification ; se reconnecter avec le code.
6. Accorder un accès support de 24 h, le révoquer, puis ouvrir le journal et retrouver chaque action avec l'auteur et l'heure.

## 9. Définition de terminé, risques, hors étape

Terminé : critères 1 à 9 verts ; fiches validées ; contrat mis à jour (journal des changements) ; `DECISIONS.md` (n° 52, 54 mis à jour) ; `CLAUDE.md` à jour.

| Risque | Mesure |
|---|---|
| Durée de session trop courte au comptoir | Valeurs paramétrables, à tester avec l'agence à la découverte (étape 14) |
| QR code qui dépasse le budget de poids | Paquet chargé à la demande ou saisie manuelle de la clé |
| Gérant unique qui perd son téléphone TOTP | Procédure manuelle à deux personnes, testée à blanc en étape 13 |

Hors étape : invitation par e-mail ou SMS (aucun au pilote, DP-68), sélection des lots assignés (étape 05), accès support réel par la console (étape 11), journal exporté (étape 23).

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 03
- Dépend de : 02
- Tranche : T1
- Jalon : J3
- F : F01, F02, F03
- R : R-009, R-081, R-082
- E : aucun
- PP : PP-11
- Contenus : aucun
