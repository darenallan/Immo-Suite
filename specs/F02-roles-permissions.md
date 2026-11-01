# F02 — Rôles et permissions (matrice codée une seule fois)

| Champ | Valeur |
|---|---|
| Statut | Partie étape 02 (§1 à §6) **validée par Mimis le 01/11/2026** (DP-85, DP-86). **Partie étape 03 (§7) rédigée et validée par Mimis le 01/11/2026** (recommandations acceptées, DP-88) |
| Étape | 02, `roadmap/etape-02-organisations-connexion-squelette.md` §4 point 2 |
| Dépend de | DP-66 (compléments), DP-65 (plans), décision n° 45 ; `CONTEXTE.md` « Matrice de permissions » (source unique) ; `API_CONTRACTS.md` §4.6 ; `SECURITY.md` §4.1 à §4.3 |
| Règles | R-081 (étape 03 pour le dernier gérant), R-082 (plans) |
| Critères visés | critère 7 de l'étape 02 ; test A3 (`TESTING_QA.md`) |

## 1. Objet

La matrice des droits est écrite **une seule fois**, sous forme de données, dans `identity::authorize(actor, Action)`. Chaque route déclare l'action qu'elle exige ; l'interface ne fait que masquer. **Refus par défaut** : toute action absente de la table est refusée. Le rôle est relu en base à chaque requête (F01 règle 9).

## 2. Règles

| N° | Règle |
|---|---|
| 1 | `authorize(actor, action)` renvoie une décision : `Allow`, `AllowScoped` (limité à ses lots assignés : agent terrain), `AllowPartial` (lecture des seuls éléments utiles au rôle : `settings.read`), `NeedsSetting(clé)` (droit réglable, voir règle 4) ou `Deny(raison)` |
| 2 | Raisons de refus : `auth.forbidden` (403, avec `params.action` et `params.required_roles`) ; `plan.feature_unavailable` (403) quand l'action est réservée au plan Agence et que l'organisation est du plan Propriétaire |
| 3 | Un nom d'action inconnu ne compile pas : les actions forment une énumération fermée. Une action sans ligne dans la table est refusée (test sur l'énumération complète) |
| 4 | Droit réglable (`*`) : la décision dépend d'un réglage de l'organisation (`role_settings`, **étape 03**). Tant que `role_settings` n'existe pas, un droit réglable est **refusé** (refus par défaut). Valeurs par défaut à fixer avec Mimis avant l'étape 03 (§6 point 2) |
| 5 | Lecture et écriture sont des actions **distinctes** dans la table (`catalog.read` / `catalog.write`). Pour une action de lecture, la mention « (L) » de la matrice signifie **autorisé** ; pour une action d'écriture, « (L) » signifie **refusé** (le rôle lit, il n'agit pas). Exemple : le comptable a `payment.read` mais pas `receipt.issue` |
| 6 | Les actions marquées « réauth » exigent en plus la confirmation du mot de passe valable 5 minutes (étape 03) : l'étape 02 code la **marque**, pas le mécanisme |
| 7 | Les entrées de menu et le tableau `permissions[]` de `GET /me` sont **dérivés** de `authorize`, jamais écrits à part |
| 8 | Agent terrain : `AllowScoped` ne filtre rien par lui-même ; le filtre de portée est appliqué par les requêtes de `collections`, `lease` et `property` (étapes suivantes). Un test à jeux de lots assignés et non assignés le couvrira alors (`DATABASE_SCHEMA.md` §6.10) |

## 3. La table (25 actions × 7 rôles = 175 couples)

Rôles : **G** gérant, **Ge** gestionnaire, **C** caissier, **A** agent terrain, **Co** comptable, **P** propriétaire direct, **Col** collaborateur. Cellules : **O** autorisé · **N** refusé · **s** autorisé pour ses lots · **\*** réglable · **p** partiel · **—** action du plan Agence, `plan.feature_unavailable` pour le plan Propriétaire.

| Action | G | Ge | C | A | Co | P | Col | Marque |
|---|---|---|---|---|---|---|---|---|
| `catalog.read` | O | O | O | s | O | O | O | |
| `catalog.write` | O | O | N | N | N | O | N | |
| `payment.record` | O | \* | O | N | N | O | \* | |
| `payment.read` | O | O | O | N | O | O | O | |
| `payment.request_correction` | O | O | O | N | N | O | O | |
| `payment.reverse` | O | N | N | N | N | O | N | réauth |
| `receipt.issue` | O | O | O | N | N | O | O | |
| `arrears.read` | O | O | O | s | O | O | O | |
| `reminder.write` | O | O | N | s | N | O | O | |
| `statement.read` | O | O | N | N | O | — | — | plan Agence |
| `statement.validate` | O | N | N | N | N | — | — | plan Agence, réauth |
| `payout.record` | O | N | N | N | N | — | — | plan Agence, réauth |
| `mandate.write` | O | N | N | N | N | — | — | plan Agence |
| `expense.write` | O | O | N | N¹ | N | O | N | |
| `expense.impute` | O | N | N | N | N | O | N | |
| `export.run` | O | \* | N | N | O | O | N | réauth |
| `import.run` | O | \* | N | N | N | O | N | |
| `user.manage` | O | N | N | N | N | O | N | réauth (rôle gérant, désactivation) |
| `audit.read` | O | N | N | N | O | O | N | |
| `settings.read` | O | p | p | p | p | O | p | |
| `settings.write` | O | N | N | N | N | O | N | |
| `support.grant` | O | N | N | N | N | O | N | réauth |
| `search.use` | O | O | O | s | O | O | O | |
| `feedback.create` | O | O | O | O | O | O | O | |
| `events.create` | O | O | O | O | O | O | O | |

¹ Agent terrain : « proposer » une dépense n'est pas défini (statut, validation par qui) : la route est **refusée** au pilote (DP-66).

Lecture de la table : le tableau reprend `API_CONTRACTS.md` §4.6 cellule par cellule, en appliquant la règle 5 aux mentions « (L) » et les compléments de DP-66 (collaborateur : voit biens, lots, locataires et impayés, demande une correction, émet et réimprime une quittance ; n'importe pas, n'exporte pas, ne saisit pas de dépense, n'a ni paramètres ni journal d'audit). Les trois notes ¹ de `API_CONTRACTS.md` §4.6 sur le collaborateur sont ainsi levées par DP-66.

## 4. Tests

| Test | Contenu | Seuil |
|---|---|---|
| A3 généré | une boucle sur **25 actions × 7 rôles** compare `authorize` à la table attendue. La table attendue est **lue dans `API_CONTRACTS.md` §4.6** par le test (pas recopiée), avec la règle 5 pour « (L) » et DP-66 pour le collaborateur ; une cellule illisible fait échouer le test | 175 couples sur 175, 0 écart |
| Refus par défaut | une action fabriquée hors énumération n'existe pas ; une action de l'énumération sans ligne fait échouer le test de complétude | 100 % des variantes de l'énumération présentes dans la table |
| Plan | les quatre actions du plan Agence × les deux rôles du plan Propriétaire → `plan.feature_unavailable` (pas `auth.forbidden`) | 8 couples |
| Marque « réauth » | la liste des actions marquées est égale à celle de `API_CONTRACTS.md` §4.6 (🔐) | liste identique |
| Menus | pour chaque rôle, les entrées visibles sont exactement celles du tableau « Entrées de menu par rôle » de `UI_UX_DESIGN.md` | 17 entrées × 7 rôles |
| Rôle relu | un changement de rôle en base est effectif à la requête suivante, sans reconnexion | 1 cas |
| Route sans droit | une route qui n'appelle pas `authorize` fait échouer le build (registre des routes, comme pour l'isolation) | 0 route sans droit déclaré |

## 5. Cas limites

| Cas | Comportement |
|---|---|
| Rôle `owner` dans une organisation du plan `agency` | impossible à créer (F01 règle 6) ; s'il apparaît en base, `authorize` refuse tout (`auth.forbidden`) |
| Droit réglable avant l'étape 03 | refusé ; la table dit `NeedsSetting`, le réglage manquant vaut « non » |
| Action de lecture « partielle » (`settings.read`) | autorisée ; la route ne renvoie que les éléments utiles au rôle (en-tête de quittance pour le caissier, modèles de messages pour qui relance) |
| Gestionnaire sans droit d'encaisser | l'entrée « Encaisser » devient « Locataires » dans la barre du bas (`UI_UX_DESIGN.md`) |
| Dernier gérant retiré | hors étape 02 : contrainte de base et code en étape 03 (R-081) |
| Actions « en base aussi » (contre-passer, valider, rouvrir, reverser, mandat hors plan Agence, accès support sans accord) | la base les impose aussi (`SECURITY.md` §4.3) ; leurs garde-fous arrivent avec leurs tables |

## 6. Questions pour Mimis

1. **Lecture de « (L) »** (règle 5) : confirmer que « (L) » sur une action d'écriture signifie « refusé, le rôle lit seulement ». Exemple décisif : le comptable ne peut **pas** émettre ni réimprimer une quittance (il la lit). *Recommandation : accepter ; c'est la seule lecture cohérente avec la matrice.*
2. **Valeurs par défaut des droits réglables** (utile à l'étape 03, à trancher avant) : gestionnaire et collaborateur peuvent-ils encaisser par défaut ? gestionnaire peut-il exporter et importer par défaut ? Le dossier dit « Réglable » sans valeur. *Je propose, pour ne rien inventer : « non » partout par défaut, le gérant ou le propriétaire active.* Question à ajouter à `DOMAINE.md` « Questions ouvertes ».
3. **Journal d'audit pour le propriétaire direct** : `API_CONTRACTS.md` le marque « oui », `UI_UX_DESIGN.md` « L » (lecture seule). Aucune action d'écriture n'existe sur le journal, donc `audit.read` suffit. *Recommandation : accepter.*
4. **Exportation par le comptable** : « Oui » (réauth). Un comptable tiers à l'agence peut donc exporter toutes les données de l'organisation. *Recommandation : conserver tel quel (matrice du dossier) et le rappeler dans l'accord pilote.*

---

## 7. Partie de l'étape 03 : droits réglables, gestion des membres, dernier gérant, agent terrain

Étape : `roadmap/etape-03-roles-invitations-audit-acces-support.md` §4 point 2. Dépend de : DP-66, DP-68, DP-85 ; R-081, R-082 ; `SECURITY.md` §4.1 à §4.3 ; `API_CONTRACTS.md` §4.6, §5.2, §7.3. Critères visés : 3, 4, 5 et 9 de l'étape 03 ; test A3 ; garde G-16.

### 7.1 Droits réglables (`role_settings`)

| Réglage (`action`) | Rôle | Action gouvernée (F02 §3) | Plan |
|---|---|---|---|
| `record_payment` | gestionnaire (`property_manager`) | `payment.record` | Agence |
| `import_data` | gestionnaire | `import.run` | Agence |
| `export_data` | gestionnaire | `export.run` | Agence |
| `record_payment` | collaborateur (`collaborator`) | `payment.record` | Propriétaire |

**Quatre** réglages seulement : ce sont les quatre cellules « Réglable » de la table de F02 §3. Les autres couples (collaborateur × `import_data`, collaborateur × `export_data`) sont **refusés** et non réglables, bien que la contrainte de `role_settings` accepte ces valeurs : `PUT` sur ces couples ou sur un rôle étranger au plan répond `422 validation.failed`.

| N° | Règle |
|---|---|
| 1 | **Valeur par défaut : « non »** (DP-85, question ouverte de `DOMAINE.md`). Pas de ligne = non. Aucune ligne n'est semée (`set_by` est obligatoire) |
| 2 | `GET /role-settings` (`user.manage`) renvoie les **quatre** réglages applicables au plan de l'organisation, avec leur état et `version` (0 tant qu'aucune ligne n'existe) |
| 3 | `PUT /role-settings` (`user.manage`) `{ role, action, allowed }` avec `If-Match: "version"` (`"0"` pour créer la ligne) : version périmée `412 concurrency.version_mismatch`, absente `428 concurrency.version_required`. Succès : ligne créée ou mise à jour (`set_by`, `set_at`, `row_version` + 1), ligne d'audit `settings.updated` (`object_type = role_settings`, `before` et `after` de `allowed`) |
| 4 | `authorize` lit les quatre réglages **à chaque requête** dans la transaction de la requête (une lecture indexée) : un réglage change la décision **à la requête suivante, dans la même session**, sans reconnexion (critère 5). Aucun cache |
| 5 | `GET /me` calcule `permissions[]` avec les réglages ; les entrées de menu et boutons suivent (F02 règle 7). La barre du bas du gestionnaire sans droit d'encaisser montre « Locataires » au lieu d'« Encaisser » (déjà en place à l'étape 02) |
| 6 | La décision `NeedsSetting(clé)` de la règle 4 de §2 est **résolue** : réglage actif → `Allow`, sinon `Deny` (`auth.forbidden`, `params.required_roles` = rôles qui l'ont toujours) |
| 7 | Le plan Propriétaire n'a que `owner` (tous droits) et `collaborator` (enregistrer si réglé, relancer, lecture, demander une correction, émettre une quittance : DP-66) |

### 7.2 Gestion des membres

| N° | Règle |
|---|---|
| 1 | `GET /users` (`user.manage`) : `membership_id`, nom, identifiant, rôle, statut (`active`, `disabled`). Les invitations ont leur liste (F01 §9.2). Un membre d'une autre organisation n'apparaît jamais |
| 2 | `PATCH /users/{membership_id}` `{ role }` : le rôle doit appartenir au plan de l'organisation (F01 règle 6). `owner` ne s'attribue **jamais** par l'API (création en ligne de commande seulement) : dans le plan Propriétaire, aucun changement de rôle n'est donc possible (`422 validation.failed`). Effet **à la requête suivante**, sans reconnexion et **sans révoquer les sessions** de la personne (le rôle est relu à chaque requête, F01 règle 9) |
| 3 | `POST /users/{membership_id}/disable` : `status = disabled`, `disabled_at = maintenant`, **sessions de cette personne ouvertes sur cette organisation révoquées** (les sessions sur une autre organisation, ou sans organisation active, ne sont pas touchées : §7.9 point 4). `POST …/enable` : `status = active`, `disabled_at = NULL` ; aucun mot de passe n'est changé |
| 4 | **Dernier gérant (R-081, G-16)** : un déclencheur sur `memberships` refuse tout `UPDATE` de `role` ou de `status` qui laisserait l'organisation **sans adhésion active de rôle `manager`** (plan Agence) ou **`owner`** (plan Propriétaire). Erreur de base traduite en `409 user.last_manager`. Le déclencheur verrouille les lignes des gérants actifs de l'organisation (`FOR UPDATE`) avant de compter : deux gérants qui se retirent **simultanément** l'un l'autre ne peuvent pas tous deux réussir. `DELETE` d'une adhésion reste réservé à la purge d'une organisation en phase `setup` (DP-60) |
| 5 | **Pas d'auto-retrait** : on ne peut ni changer son propre rôle ni se désactiver soi-même (`403 auth.forbidden`, texte « Demandez à un autre gérant »). Un autre gérant le fait, sous la garde du point 4 |
| 6 | **Comptes de démonstration** (`is_demo`) : le déclencheur existant `memberships_demo_guard` s'applique aussi au changement de rôle ; G-16 est testé dans une organisation `real` et une `demo` |
| 7 | **Information des autres gérants** : toute ligne `membership.role_changed` ou `user.disabled` qui touche un gérant porte `after.manager_affected = true` et se lit dans S21 (aucune notification poussée avant l'étape 20) |
| 8 | Chaque action écrit sa ligne d'audit (`membership.role_changed`, `user.disabled`, `user.enabled`) dans la même transaction (R-008) |

### 7.3 Confirmation par mot de passe : liste exacte (à valider)

Trois sources se contredisent légèrement : l'étape 03 (critère 9 : « changement de rôle d'un gérant, désactivation, accord support, contre-passation »), `SECURITY.md` §3.4 (liste plus longue, « changement de rôle d'un gérant ») et `API_CONTRACTS.md` §7.3 (🔐 sur **tout** changement de rôle). La fiche applique **le contrat d'API, propriétaire des routes**, le plus strict, et ajoute la réactivation :

| Route | Confirmation exigée |
|---|---|
| `PATCH /users/{membership_id}` | **toujours** (tout changement de rôle) |
| `POST /users/{membership_id}/disable` | toujours |
| `POST /users/{membership_id}/enable` | **toujours** (ajout : rendre un accès à quelqu'un est plus grave que le retirer ; le contrat ne la marque pas) |
| `POST /users/{membership_id}/password-resets` | toujours |
| `POST /invitations`, `POST /invitations/{id}/renew` | seulement si le rôle invité est `manager` |
| `POST /support-grants`, `POST /support-grants/{id}/revoke` | toujours |
| `PATCH /me` | seulement si `login_phone` change |
| `PUT /role-settings`, `PUT …/unit-assignments`, `POST /invitations/{id}/revoke`, `PATCH /organization`, `PATCH /settings` | **jamais** |

La marque « réauth » de la table de §3 reste portée par **l'action** (`user.manage`, `support.grant`, `payment.reverse`…) ; la condition par route (« seulement si le rôle invité est gérant ») est un attribut de la **route**. Le test « liste des marques égale à celle de `API_CONTRACTS.md` §4.6 » compare donc deux listes : les actions (inchangé) et les routes (nouveau, lue dans le tableau de §7.3 de l'API).

### 7.4 Agent terrain et lots assignés

`unit_assignments` est créée à l'étape 03 **sans** la clé étrangère composite vers `units` (la table `units` arrive à l'étape 05, qui ajoute la clé). `PUT /users/{membership_id}/unit-assignments` n'accepte donc qu'une liste **vide** (retrait de toutes les affectations) ; une liste non vide répond `422 validation.failed` jusqu'à l'étape 05.

Le **filtre de portée** est livré maintenant sous forme d'une fonction `identity::scope_of(actor) -> Scope` (`All` ou `Units(ensemble des lots non révoqués de l'agent)`) que les requêtes de `property`, `lease` et `collections` utiliseront (F02 règle 8). Il est testé sur une table jouet de 25 lots dont 10 sont assignés à l'agent, puis **rejoué sur les vrais lots à l'étape 05** (R-081, `DATABASE_SCHEMA.md` §6.10).

### 7.5 Routes de l'étape et action exigée

| Route | Action |
|---|---|
| `GET /users`, `PATCH /users/{id}`, `POST …/disable`, `POST …/enable`, `PUT …/unit-assignments`, `POST …/password-resets` | `user.manage` |
| `POST/GET /invitations`, `POST /invitations/{id}/revoke`, `POST /invitations/{id}/renew` | `user.manage` |
| `GET/PUT /role-settings` | `user.manage` |
| `GET /organization`, `GET /settings` | `settings.read` |
| `PATCH /organization`, `PATCH /settings` | `settings.write` |
| `GET/POST /support-grants`, `POST /support-grants/{id}/revoke` | `support.grant` |
| `GET /audit` | `audit.read` |
| `PUT /me/password`, `PATCH /me`, `POST /me/totp`, `POST /me/totp/confirm`, `POST /auth/reauth` | session valable seulement (aucune action de la table : elles ne concernent que le compte de l'appelant) |
| `GET /invitations/{jeton}`, `POST /invitations/accept`, `POST /auth/totp/verify`, `POST /auth/password-reset/complete` | publiques par jeton (liste blanche du registre des routes, avec leur justification) |
| `PUT, PATCH, DELETE /audit/{id}` | aucune : `405 audit.immutable` |

Le registre des routes de l'étape 02 (« toute route déclare son droit ») est étendu : les routes « session valable seulement » et « publiques par jeton » sont déclarées comme telles et listées, de sorte qu'**aucune route ne puisse exister sans déclaration**.

### 7.6 Tests

| Test | Contenu | Seuil |
|---|---|---|
| A3, toutes les routes de l'étape | pour chaque route du tableau 7.5 et chacun des 7 rôles : décision attendue (autorisé, `auth.forbidden`, `plan.feature_unavailable`) ; la table est dérivée du registre, pas recopiée | 100 % des couples route × rôle |
| Droits réglables | les quatre réglages × deux états (`non`, `oui`) × le rôle concerné, **dans la même session**, sans reconnexion ; chaque autre rôle inchangé ; les couples non réglables (collaborateur × import/export) refusés même avec une ligne en base | 8 cas + 2 |
| Effet immédiat | rôle changé en base par `PATCH` : `GET /me` de la session de la cible, sans reconnexion, montre les nouveaux droits ; la session reste valable | 1 cas |
| G-16 | seul gérant : désactivation et rétrogradation refusées (`user.last_manager`, 409), par l'API **et** par un `UPDATE` direct de SQL ; deux gérants : l'un se retire, l'autre non ; retrait simultané (deux connexions) : exactement un succès ; plan Propriétaire : dernier `owner` ; organisation `demo` | 6 cas |
| Rôle hors plan | `cashier` dans un plan Propriétaire, `collaborator` dans un plan Agence : refusés à l'invitation et au `PATCH` | 4 cas |
| Réauth | chaque route du tableau 7.3 sans confirmation valable : `403 auth.reauth_required`, **aucun effet** ; avec confirmation de 4 min 59 s : accepté ; 5 min 01 s : refusé | 9 routes |
| Auto-retrait | se rétrograder ou se désactiver soi-même : `403` | 2 cas |
| Portée de l'agent | 25 lots, 10 assignés : `scope_of` rend exactement les 10 ; un lot révoqué disparaît ; autre organisation : 0 | 3 cas |
| Menus | les entrées « Utilisateurs », « Paramètres », « Journal d'audit » sont visibles exactement pour les rôles du tableau « Entrées de menu par rôle » | 3 entrées × 7 rôles |
| Sessions | `disable` révoque les sessions de l'organisation de la cible et épargne celles d'une autre organisation ; le changement de rôle ne révoque rien | 3 cas |

### 7.7 Cas limites

| Cas | Comportement |
|---|---|
| Gestionnaire dont le droit d'encaisser est retiré pendant qu'il saisit | sa requête suivante est refusée (`403 auth.forbidden`) ; l'interface bascule au prochain chargement de `GET /me` |
| Collaborateur dans le plan Propriétaire, droit d'encaisser non activé (état par défaut) | il lit, relance, demande une correction et émet une quittance ; il n'enregistre pas de paiement |
| Réglage modifié par deux gérants simultanément | le second reçoit `412 concurrency.version_mismatch` et recharge |
| Membre désactivé qui a un lien de réinitialisation actif | le lien appartient au compte, pas à l'adhésion : il reste utilisable pour changer le mot de passe, mais n'ouvre aucun accès à l'organisation où l'adhésion est désactivée |
| Promotion d'un caissier en gérant | exige la confirmation ; ligne `membership.role_changed` avec `manager_affected` |
| Désactivation d'un membre qui a des invitations en cours | sans effet sur elles (elles sont indépendantes d'un compte) |
| Rôle changé pendant une requête en vol | la requête en vol a lu l'ancien rôle au début de sa transaction ; la suivante lit le nouveau |

### 7.8 Cas chiffrés

| N° | Cas | Attendu |
|---|---|---|
| 1 | Agent terrain assigné à 10 lots sur 25 | `scope_of` rend 10 identifiants ; les 15 autres ne sont jamais rendus |
| 2 | Organisation avec 1 gérant A et 2 caissiers ; A tente de se désactiver, puis un `UPDATE` direct de SQL met `status = disabled` sur l'adhésion de A | l'API répond `403 auth.forbidden` (auto-retrait) ; le `UPDATE` direct est refusé par le déclencheur (`user.last_manager`) |
| 3 | 2 gérants A et B ; A désactive B (réussit) ; A tente de désactiver B encore ou de se retirer | B déjà désactivé : sans effet ; A est le dernier : `409 user.last_manager` |
| 4 | Gestionnaire, droit d'encaisser `non` (défaut) puis `oui` puis `non` | `payment.record` : refusé, autorisé à la requête suivante, refusé à la requête suivante, même session |
| 5 | 175 couples de l'étape 02 | toujours 175 sur 175, 0 écart (les 4 cellules `*` sont évaluées avec réglage `non`) |
| 6 | Rôle changé à 10:00:00 d'un caissier en gestionnaire | `GET /me` à 10:00:01 : permissions du gestionnaire, même session |

### 7.9 Décisions et questions pour Mimis (étape 03, F02)

1. **Défaut « non » des droits réglables, avec sa conséquence** : en plan Propriétaire, un collaborateur **ne peut pas enregistrer un paiement** tant que le propriétaire n'a pas activé le réglage. C'est le défaut le plus sûr (DP-85) mais le propriétaire doit le savoir : S18 affichera un rappel « Autoriser {nom} à encaisser ». *Recommandation : accepter.* Autre option : « oui » pour le collaborateur uniquement (le plan Propriétaire existe pour déléguer l'encaissement).
2. **Liste de confirmation par mot de passe** (§7.3) : tout changement de rôle, toute réactivation, toute révocation d'accord support. *Recommandation : accepter* (le contrat d'API fait foi ; la réactivation est un ajout).
3. **Garde « dernier gérant »** étendue au `owner` du plan Propriétaire, et **interdiction de se retirer soi-même** (§7.2 points 4 et 5). *Recommandation : accepter.*
4. **Sessions et changement de rôle** : le contrat dit « sessions révoquées » **et** « effet sans reconnexion », ce qui est contradictoire ; `SECURITY.md` §3.2 parle d'un nouvel identifiant de session. *Recommandation* : un changement de rôle ne révoque **rien** (le rôle est relu à chaque requête, il n'y a rien à fixer) ; une **désactivation** révoque les sessions de cette organisation seulement. La ligne « sessions révoquées » du contrat et celle de `SECURITY.md` §3.2 sont corrigées dans leurs journaux.
5. **Les quatre réglages applicables** seulement (§7.1) ; les couples collaborateur × import et × export sont refusés et non réglables, malgré la contrainte plus large de `role_settings`. *Recommandation : accepter.*
6. **`unit_assignments` sans clé vers `units` jusqu'à l'étape 05**, la route n'accepte qu'une liste vide (§7.4). *Recommandation : accepter.*
