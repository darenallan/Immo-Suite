# F02 — Rôles et permissions (matrice codée une seule fois)

| Champ | Valeur |
|---|---|
| Statut | **Validée par Mimis le 01/11/2026** avec les recommandations de la fiche (DP-85, DP-86) |
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
