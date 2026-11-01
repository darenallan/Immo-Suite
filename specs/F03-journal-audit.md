# F03 — Journal d'audit (partie écriture, étape 02)

| Champ | Valeur |
|---|---|
| Statut | Partie étape 02 (§1 à §7) **validée par Mimis le 01/11/2026** (DP-85, DP-86). **Partie étape 03 (§8) rédigée et validée par Mimis le 01/11/2026** (recommandations acceptées, DP-88) |
| Étape | 02, `roadmap/etape-02-organisations-connexion-squelette.md` §4 point 3 |
| Dépend de | DP-47, DP-48, DP-60, DP-77 ; `SECURITY.md` §6.1 et §6.2 ; `DATABASE_SCHEMA.md` §4.2 et table `audit_log` |
| Règles | R-004, R-007, R-008, R-009 |
| Critères visés | critères 4, 5 et 6 de l'étape 02 ; PP-01 (en partie), PP-21 ; tests G-05, ISO-4, A13 |
| Hors fiche | lecture et filtres du journal (écran S21, étape 03), export, chaîne de hachage [APRÈS] |

## 1. Objet

Toute action sensible laisse une ligne **dans la même transaction** que l'action. Si la ligne ne peut pas être écrite, l'action est annulée (R-008). Une ligne ne se modifie ni ne se supprime jamais (R-007) : deux barrières indépendantes (droits SQL et déclencheur).

## 2. Règles

| N° | Règle |
|---|---|
| 1 | `audit::record(tx: &mut OrgTx, entry: AuditEntry)` est le **seul** moyen d'écrire. Elle prend la transaction de l'action : pas de pool, pas de seconde connexion. Une erreur d'écriture remonte à l'appelant, qui annule la transaction (R-008) |
| 2 | Une ligne contient : organisation (celle du contexte, jamais passée en paramètre), qui (`actor_kind` `user`, `system` ou `support` ; utilisateur ou agent de support), `action` au format `objet.verbe`, `object_type`, `object_id` (texte), `before` et `after` (JSON, facultatifs), adresse IP, identifiant de requête, heure |
| 3 | L'heure est **imposée par la base** (`app_now()`, déclencheur `audit_before_insert`) : la valeur envoyée par l'application est ignorée. L'organisation de la ligne est forcée à celle du contexte |
| 4 | **Barrière 1 (droits)** : `immo_app` et `immo_platform` n'ont que `SELECT` et `INSERT` sur `audit_log`. **Barrière 2 (déclencheur)** : `UPDATE`, `DELETE` et `TRUNCATE` sont refusés **même au propriétaire** (`forbid_mutation`, `forbid_truncate`). Seules exceptions : la purge d'une organisation en phase `setup` et l'annulation d'un import non utilisé (R-110, R-080, DP-60), tracées dans `platform_events` |
| 5 | `before` et `after` ne contiennent **ni mot de passe, ni jeton, ni secret, ni empreinte** : `AuditEntry` refuse toute clé nommée `password`, `token`, `secret`, `hash`, `totp` ou `csrf` (insensible à la casse, récursif). Un champ chiffré est écrit « modifié ». Les données personnelles sont réduites aux champs modifiés |
| 6 | Les codes sont ceux de `SECURITY.md` §6.1, forme `^[a-z_]+\.[a-z_]+$` (contrainte de base). Un code hors de l'énumération du code Rust ne compile pas |
| 7 | Les échecs de connexion vont dans `login_attempts` (30 jours), pas dans `audit_log` ; seul leur résumé `security.rate_limited` y figure (F01 §4) |
| 8 | Les valeurs `occurred_at` de deux lignes d'une même organisation sont croissantes au sens large ; l'ordre d'insertion est donné par `id` (identité de la base) |

## 3. Lignes écrites à l'étape 02

| Code | Quand | `object_type` / `object_id` | `before` / `after` |
|---|---|---|---|
| `auth.login` | connexion réussie (F01 §3 point 7) | `user` / identifiant de l'utilisateur | `after` : `{ "org_role": "manager" }` |
| `session.revoked_all` | `POST /auth/logout-all` | `user` / identifiant | `after` : `{ "sessions": n }` |
| `security.cross_org_attempt` | demande d'une ressource ou d'une organisation qui n'est pas celle de l'appelant (§4) | type visé / identifiant visé | aucun |
| `security.rate_limited` | première attente imposée à un identifiant (F01 §4) | `user` / identifiant | `after` : `{ "wait_seconds": n }` |
| `security.permission_denied` | jeton anti-falsification absent ou faux ; action refusée sur une route sensible | `route` / modèle de route | `after` : `{ "action": "…", "reason": "csrf_invalid" }` |

Les autres codes de §6.1 sont écrits par les étapes qui créent les objets (paiements, baux, comptes rendus…). L'énumération Rust les contient dès maintenant pour que `authorize` et l'audit parlent la même langue.

## 4. Tentative d'accès à une autre organisation (point à décider)

Les contrats exigent : ressource d'une autre organisation → `404 resource.not_found`, aucune donnée, ligne `security.cross_org_attempt` dans l'audit de l'**appelant**, alerte immédiate. **Mais** la RLS forcée rend la ressource de B invisible depuis A : le code ne peut pas distinguer « n'existe pas » de « existe chez B ». Les documents ne disent pas comment.

**Proposition (à valider)** : une fonction SQL `SECURITY DEFINER`, `exists_in_other_org(object_type, id)`, qui ne rend qu'un **booléen**, ne lit que des tables d'une **liste blanche**, et n'est appelée que sur le chemin « introuvable ». Si elle répond vrai, la ligne `security.cross_org_attempt` est écrite et l'alerte journalisée (`alert=security.cross_org_attempt`, gravité S1). Elle ne renvoie jamais de donnée de B et n'est pas exposée par l'API.

À l'étape 02, elle est appliquée à **`POST /me/organization`** (l'organisation visée existe-t-elle sans que l'appelant en soit membre ?) et testée sur une **table jouet**, car les tables métier n'existent pas encore. Chaque nouvelle route ajoutera son type d'objet à la liste blanche. *Limite assumée* : un identifiant qui n'existe nulle part n'est pas journalisé comme tentative (pas de bruit pour une faute de frappe). **Alerte** : le canal d'alerte est la dette (5) de l'étape 01 ; en attendant, l'alerte est une ligne de journal structurée, comme celles des sauvegardes.

## 5. Tests

| Test | Ce qu'il prouve | Critère |
|---|---|---|
| G-05 / ISO-4 | `UPDATE`, `DELETE`, `TRUNCATE` sur `audit_log` échouent pour `immo_app`, `immo_platform` **et** `immo_owner` (3 rôles × 3 instructions = 9 cas) ; l'erreur vient des droits pour les deux premiers, du déclencheur pour le propriétaire | 5 |
| Altération du déclencheur | supprimer le déclencheur est détecté par le contrôle nocturne (requête de présence des gardes) | PP-01 |
| R-008 | une écriture d'audit qui échoue (droit retiré dans le test) annule l'action : la ligne métier jouet n'existe pas | 6 |
| Heure imposée | une ligne envoyée avec `occurred_at` de 2020 porte l'heure de la base | — |
| Organisation forcée | une insertion avec l'`org_id` d'une autre organisation est refusée par la RLS | ISO-2 |
| Isolation | l'organisation A ne lit aucune ligne de B ; sans contexte, 0 ligne et insertion refusée | 8 |
| Secrets | `AuditEntry::new` refuse les clés interdites (liste de la règle 5, imbriquées, majuscules) | PP-21 |
| Exemple chiffré | un paiement d'essai (table jouet) laisse une ligne avec `before` et `after` | critère de l'étape |
| Cross-org | A demande l'organisation de B : `404`, ligne dans l'audit de A et rien dans celui de B, alerte journalisée, aucune donnée de B dans la réponse | 4 |
| A13 | un parcours de connexion complet ne laisse aucun numéro `+226`, mot de passe ni jeton dans les journaux `tracing` | PP-21 |

## 6. Cas limites

| Cas | Comportement |
|---|---|
| Action sans organisation active (utilisateur de plusieurs organisations avant son choix) | aucune ligne d'organisation possible : `auth.login` attend le choix (F01 §3) |
| Écriture d'audit lente ou en échec pendant une connexion | la connexion échoue (`500 server.unexpected`) : mieux vaut ne pas se connecter que de se connecter sans trace |
| Console HABEK (`actor_kind = 'support'`) | la colonne `actor_staff_id` existe ; sa clé étrangère vers `platform_staff` est ajoutée **avec la console** (étape ultérieure) : voir §7 point 2 |
| Objet supprimé plus tard | impossible : rien ne se supprime (archivage) ; sinon `object_id` reste lisible tel quel |
| Gros volume | index `(org_id, occurred_at DESC)`, `(org_id, actor_user_id, …)`, `(org_id, object_type, object_id, …)` de `DATABASE_SCHEMA.md` |

## 7. Questions pour Mimis

1. **Mécanisme du cas « autre organisation »** (§4) : accepter `exists_in_other_org` (booléen, liste blanche, chemin « introuvable » seulement) ? Autre option : journaliser **tout** 404 sur identifiant bien formé comme tentative (plus simple, mais du bruit et des fausses alertes S1). *Recommandation : la fonction booléenne.* Le choix modifie une règle de sécurité : il sera inscrit dans `DECISIONS.md` (DP-85 proposé).
2. **Clé étrangère `audit_log.actor_staff_id → platform_staff`** : la table `platform_staff` n'existe qu'avec la console HABEK. *Proposition : créer `audit_log` à l'étape 02 avec la colonne `actor_staff_id` **sans** clé étrangère, ajouter la clé étrangère dans la migration de la console.* Alternative : créer `platform_staff` maintenant, vide. *Recommandation : la première (moins de surface).*
3. **Où écrire `security.rate_limited`** quand l'identifiant ne correspond à aucun compte : nulle part (aucune organisation). *Recommandation : accepter* (déjà posé en F01 §8 point 5).
4. **Journal pendant une panne de l'audit** : la connexion échoue plutôt que de continuer sans trace (R-008 appliqué aux connexions). *Recommandation : accepter.*

---

## 8. Partie de l'étape 03 : lire le journal (S21) et nouvelles lignes

Étape : `roadmap/etape-03-roles-invitations-audit-acces-support.md` §4 point 3. Dépend de : R-007 ; `SECURITY.md` §4.2, §6.1, §6.4 ; `API_CONTRACTS.md` §7.12 ; `UI_UX_DESIGN.md` S21. Critères visés : 7 de l'étape 03 ; PP-01 (en partie), PP-21. Hors fiche : l'export du journal (`export.created`, étape 23), la chaîne de hachage [APRÈS].

### 8.1 `GET /audit`

| N° | Règle |
|---|---|
| 1 | Droit `audit.read` (gérant, comptable, propriétaire direct). Les autres rôles : `403 auth.forbidden`. La lecture elle-même n'écrit rien |
| 2 | **Filtres** (tous facultatifs, combinables par ET) : `actor_id` (identifiant d'utilisateur), `action` (code exact `objet.verbe`), `object_type`, `object_id`, `from` et `to` (dates JJ/MM/AAAA du **fuseau de l'organisation**, bornes **incluses** : `from` à 00:00:00 local, `to` jusqu'à 23:59:59 local, converties en intervalle semi-ouvert), `actor_kind` (`user`, `system`, `support`). Valeur inconnue ou mal formée : `422 validation.failed`, jamais ignorée en silence |
| 3 | **Ordre** : du plus récent au plus ancien, `occurred_at DESC, id DESC`. **Pagination par curseur** (`limit` 25 par défaut, 100 au plus, `next_cursor` opaque sur le couple `(occurred_at, id)`) ; curseur altéré : `400 request.cursor_invalid`. Deux lignes de même `occurred_at` ne sont ni perdues ni doublées d'une page à l'autre |
| 4 | **Ligne rendue** : `{ id, occurred_at, actor { kind, user_id?, name?, label? }, action, object { type, id }, before?, after?, ip?, request_id }`. `name` vient du compte (`users.display_name`) ; pour `actor_kind = 'support'` : `label` = « Équipe HABEK » (puis le nom de l'agent avec la console, étape 11) ; pour `system` : « Système » |
| 5 | **Adresse IP** : renvoyée seulement au gérant et au comptable (`SECURITY.md` §4.2, `DATABASE_SCHEMA.md`). Pour les autres rôles qui peuvent lire (le propriétaire direct), le champ `ip` est **absent** de la réponse, pas vide (§8.7 point 1). La valeur est rendue au format texte ; le résumé de la tentative inter-organisations ne la masque pas |
| 6 | `meta.actors` (liste `{ id, name }` des personnes qui ont une adhésion dans l'organisation) alimente le filtre « utilisateur » de S21 sans exiger `user.manage` (le comptable ne l'a pas) |
| 7 | **Rien d'autre que la lecture** : `PUT`, `PATCH`, `DELETE /audit/{id}` répondent `405 audit.immutable` avec `Allow: GET`. Pour un appelant connecté, la tentative écrit une ligne `security.permission_denied` (`route = /audit/{id}`, `action = audit.write`) ; un appelant non connecté reçoit `401` d'abord |
| 8 | Aucune ligne d'une autre organisation n'est jamais rendue (RLS) ; un `object_id` d'une autre organisation donne simplement 0 résultat (pas de ligne `cross_org`, c'est un filtre, pas une ressource) |
| 9 | Budget serveur : 150 ms au 95ᵉ centile sur une organisation de 20 000 lignes (index `(org_id, occurred_at DESC)`, `DATABASE_SCHEMA.md`), mesure en CI |
| 10 | Défense en profondeur : même si une ligne ancienne contenait par erreur une clé interdite (F03 règle 5), la couche de lecture **retire** récursivement `password`, `token`, `secret`, `hash`, `totp`, `csrf` avant de répondre |

### 8.2 Lignes écrites par l'étape 03

Toutes dans la transaction de l'action (R-008). Les codes en *italique* sont **nouveaux** (absents de `SECURITY.md` §6.1, à ajouter, question 5 de §8.7).

| Code | Quand | `object_type` / `object_id` | `after` (et `before` si utile) |
|---|---|---|---|
| `user.invited` | création d'une invitation | `invitation` / id | `{ role, invitee_kind: "phone"\|"email", expires_at }` (jamais le numéro ni le lien) |
| *`invitation.renewed`* | « Renvoyer » | `invitation` / id | `{ renewed_from, expires_at }` |
| *`invitation.revoked`* | révocation | `invitation` / id | `{ }` |
| `membership.accepted` | acceptation | `membership` / id | `{ role }` |
| `membership.role_changed` | changement de rôle | `membership` / id | `before: { role }`, `after: { role, manager_affected? }` |
| `user.disabled` | désactivation | `membership` / id | `{ manager_affected? }` |
| *`user.enabled`* | réactivation | `membership` / id | `{ }` |
| *`user.updated`* | `PATCH /me` | `user` / id | `{ changed: ["display_name"\|"login_phone"] }` (aucune valeur de téléphone) |
| `auth.password_changed` | `PUT /me/password` ou réinitialisation | `user` / id | `{ sessions_revoked: n }` |
| `auth.reset_issued` | lien généré | `membership` / id de la cible | `{ expires_at }` |
| `auth.reset_used` | lien utilisé | `user` / id | `{ }` |
| `auth.mfa_enabled` | confirmation du TOTP | `user` / id | `{ backup_codes: 10 }` |
| *`auth.mfa_backup_used`* | connexion par un code de secours | `user` / id | `{ remaining: n }` |
| `session.revoked_all` | (étape 02) | | |
| `settings.updated` | `PATCH /organization`, `PATCH /settings`, `PUT /role-settings` | `organization`, `settings` ou `role_settings` / id | clés changées seulement, avant et après |
| `support.access_granted` | F40 | `support_grant` / id | `{ scope, reason, duration_hours, expires_at }` |
| `support.access_revoked` | F40 | `support_grant` / id | `{ }` |

**Où s'écrit une action de compte** (`auth.password_changed`, `auth.reset_used`, `auth.mfa_enabled`, `auth.mfa_backup_used`, `user.updated`) : le compte est global, l'audit est par organisation. La ligne est écrite dans l'audit de **chaque organisation où la personne a une adhésion active** (comme `security.rate_limited`, F01 §4) ; si elle n'en a aucune, nulle part. Les actions faites **par** un gérant sur un membre (`auth.reset_issued`, `membership.*`, `user.*`) s'écrivent dans l'organisation du gérant.

### 8.3 Lisibilité (S21)

- Chaque code est affiché par une phrase de `texts/fr.ts` (« {Nom} a invité un utilisateur », « {Nom} a changé le rôle de {Nom} : Caissier → Gestionnaire ») ; un code sans phrase s'affiche tel quel (jamais d'écran cassé).
- `before` et `after` s'affichent en liste « champ : avant → après » ; les valeurs ne sont jamais des secrets (règle 5) ; un champ chiffré affiche « modifié ».
- Heure : `JJ/MM/AAAA HH:mm` au fuseau de l'organisation. Adresse IP : colonne visible seulement si le serveur l'a envoyée.
- Filtres : utilisateur (liste `meta.actors`), période, type d'objet, action, qui (utilisateur, système, support). État vide : « Aucun événement pour ces filtres. ». Bouton « Exporter » : repère « À venir » (étape 23).
- Quatre états, 44 px, ligne dépliable au toucher pour `before`/`after`, `lang="fr"`, aucun ajout de poids notable (limite de 150 Ko respectée).

### 8.4 Tests

| Test | Contenu | Critère |
|---|---|---|
| Droits | `audit.read` : 7 rôles × `GET /audit` (3 autorisés, 4 refusés) ; `PUT/PATCH/DELETE /audit/{id}` : 405 pour les 7 | 7 |
| Adresse IP | gérant et comptable : `ip` présent ; propriétaire direct : `ip` absent (selon la réponse à la question 1) | 7 |
| Immuabilité | les trois verbes : 405 `audit.immutable`, `Allow: GET`, ligne `security.permission_denied` chez l'appelant ; rien en base n'a changé | 7 |
| Filtres | chaque filtre seul et combiné sur un jeu de 60 lignes (4 acteurs, 3 types d'objet, 3 mois) : comptes attendus exacts ; bornes testées à 00:00:00 et 23:59:59 ; la conversion passe par la fonction de fuseau de l'organisation (Africa/Ouagadougou = UTC+0 aujourd'hui, mais le code ne le suppose pas) | 7 |
| Pagination | 61 lignes dont 10 de même `occurred_at`, pages de 25 : 61 lignes distinctes, aucune perdue, aucun doublon ; curseur altéré : 400 | — |
| Isolation | A ne lit pas B, B ne lit pas A, avec les mêmes filtres | ISO |
| Secrets | une ligne écrite avec une clé interdite par un `INSERT` direct de test : la lecture la retire ; `AuditEntry::new` la refuse à l'écriture | PP-21 |
| Une ligne par action | pour chaque code du tableau 8.2 : l'action réelle laisse exactement une ligne, avec le bon `object_type`, dans la même transaction ; échec d'écriture : action annulée | R-008 |
| Compte global | un utilisateur membre de 2 organisations change son mot de passe : une ligne dans chacune ; membre d'aucune : aucune | — |
| Performance | 20 000 lignes, filtre par période : p95 sous 150 ms | 8.1 point 9 |
| A13 | un parcours complet (invitation, acceptation, TOTP, réinitialisation, accord support) ne laisse aucun jeton, mot de passe, code ni numéro dans les journaux `tracing`, ni dans `audit_log` | PP-21 |

### 8.5 Cas limites

| Cas | Comportement |
|---|---|
| Acteur supprimé ou désactivé plus tard | la ligne garde `actor_user_id` ; le nom vient du compte (qui reste, désactivé) |
| Ligne `support` avant que la console existe | aucune ; le test en insère pour vérifier le rendu |
| Filtre `to` avant `from` | `422 validation.failed` |
| Plus de 100 résultats demandés | plafonné à 100, sans erreur |
| Valeur `before`/`after` très grande | tronquée à 4 Kio à l'écriture avec la marque `"truncated": true` (jamais d'échec d'écriture pour cette raison) |
| Lecture pendant une écriture | le journal est en ajout seul ; une lecture voit les lignes validées à son début |

### 8.6 Cas chiffrés

| N° | Cas | Attendu |
|---|---|---|
| 1 | Gérant ouvre S21 après la démonstration de l'étape 03 | il retrouve, du plus ancien au plus récent (affichés du plus récent au plus ancien) : invitation, acceptation, changement de rôle, tentative de désactivation du dernier gérant refusée (aucune ligne : l'action n'a pas eu lieu), lien de réinitialisation, TOTP activé, accord support accordé, accord support révoqué ; chacune avec l'auteur et l'heure |
| 2 | `from = 01/11/2026`, `to = 01/11/2026` | toutes les lignes entre le 01/11/2026 00:00:00 et 23:59:59 (Africa/Ouagadougou, UTC+0 : même heure qu'en UTC) |
| 3 | 61 lignes, `limit = 25` | pages de 25, 25 et 11 |
| 4 | Tentative `DELETE /audit/{id}` par un gérant | `405`, 0 ligne supprimée, 1 ligne `security.permission_denied` ajoutée |

### 8.7 Décisions et questions pour Mimis (étape 03, F03)

1. **Adresse IP pour le propriétaire direct** : `audit.read` lui est ouvert (F02), mais `SECURITY.md` §4.2 et le critère 7 nomment le gérant et le comptable seulement. Appliqué à la lettre, le propriétaire direct lit le journal **sans** l'adresse IP, ce qui le prive d'un signal utile sur son propre compte. *Recommandation : lui montrer l'adresse IP (il est le gérant de son organisation) et corriger le texte de `SECURITY.md` §4.2.* Autre option : la lui cacher (lettre du critère 7).
2. **`meta.actors`** dans la réponse de `GET /audit` (filtre « utilisateur » du comptable) : ajout au contrat. *Recommandation : accepter.*
3. **Actions de compte écrites dans chaque organisation où la personne est membre actif** (§8.2). *Recommandation : accepter* (cohérent avec `security.rate_limited`).
4. **Une tentative de modifier le journal** (`PUT/PATCH/DELETE`) laisse une ligne `security.permission_denied`. *Recommandation : accepter.*
5. **Cinq codes d'audit nouveaux** (`invitation.renewed`, `invitation.revoked`, `user.enabled`, `user.updated`, `auth.mfa_backup_used`) ajoutés à `SECURITY.md` §6.1 et à l'énumération Rust (le code d'erreur d'API nouveau est dans F01 §9.12 point 11). *Recommandation : accepter.*
6. **Le bouton « Exporter » de S21** reste un repère « À venir » jusqu'à l'étape 23. *Recommandation : accepter.*
