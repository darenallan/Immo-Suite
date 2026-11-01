# F03 — Journal d'audit (partie écriture, étape 02)

| Champ | Valeur |
|---|---|
| Statut | **Validée par Mimis le 01/11/2026** avec les recommandations de la fiche (DP-85, DP-86) |
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
