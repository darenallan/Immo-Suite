# Immo Suite — Contrats d'API

| Champ | Valeur |
|---|---|
| Statut | Propriétaire des routes, des formats d'erreur, des codes et des en-têtes. Le contrat est généré depuis le code (DP-56) ; ce document est la spécification écrite avant le code. Les choix qui n'ont pas d'entrée DP-xx restent [À valider] |
| Date | 30/10/2026 |
| Dépend de | `DECISIONS.md` (état au 30/10/2026), dossier de cadrage v1.0 (§11.2, §12, §14, §16, §19), `ARCHITECTURE.md` v0.2 (§2.3, §3.7, §3.8, §4.5, §5, §8.4), `DATABASE_SCHEMA.md` v0.2 (§3, §4, §6, §10), `SECURITY.md` v0.2 (§3, §4, §7), `DOMAINE.md` (R-001 à R-110, E-01 à E-48), `CONTEXTE.md`, `UI_UX_DESIGN.md` (microcopy, S01 à S26), `MAQUETTE.md` (parcours 1 à 5) |
| Alimente | `CODING_RULES.md`, `TESTING_QA.md`, `DEPLOYMENT.md`, `specs/`, `CLAUDE.md` |
| Version | 0.2 |

Légende :

- Marqueurs ([PILOTE], [APRÈS], [À valider], [Hypothèse], [À VÉRIFIER AVEC UN JURISTE]) : voir `CLAUDE.md`.
- **T1, T2, T3** : tranches du dossier (DP-29). Le pilote démarre à la fin de T2.

Hiérarchie en cas de conflit : voir `CLAUDE.md` (hiérarchie par sujet). Ce document ne contredit aucun autre. Là où une règle manque, il le dit (« Questions ouvertes ») et ne la devine pas. Les noms de champs et d'en-têtes marqués « proposition » sont des choix de ce document, non des règles du domaine.

Indispensable avant le premier pilote :

- **[PILOTE]** Contrat REST/JSON décrit par un fichier OpenAPI versionné, client TypeScript généré, format d'erreur unique et codes stables (sections 1 à 3).
- **[PILOTE]** Authentification par session, jeton anti-falsification, identité d'organisation tirée de la session seule, permissions par route, limitation du débit (section 4).
- **[PILOTE]** Clé d'idempotence sur toute écriture d'argent, verrouillage optimiste des fiches modifiables, opérations longues suivies par état (section 5).
- **[PILOTE]** Tous les points d'accès de T1 et T2 (sections 6 et 7), un test d'isolation par route, un test de contrat par route (section 8).
- **[APRÈS]** Points d'accès de T3 (documents, agenda, notifications, clôture de caisse, saisie assistée du SMS, dépôt de garantie, fin de bail, exports), page publique de vérification d'un code de quittance si la décision n° 55 est retenue.

## 1. Principes

### 1.1 Forme générale

- **REST, JSON, un seul client** : le front TypeScript servi par le même binaire (`ARCHITECTURE.md` §2.1). Pas d'API publique, pas de compte de service, pas d'intégration tierce au pilote (`CONTEXTE.md`, exclusions). Le contrat est donc **interne** ; il doit pourtant rester stable parce que des onglets et une PWA plus anciens que le serveur sont en circulation (section 1.3).
- **Préfixe** : tout est sous `/api/`. Hors `/api/` : `/health`, `/s/{jeton}` (page publique de partage, S26, HTML sans JavaScript applicatif), `/v/…` (vérification d'un code de quittance, décision n° 55 [APRÈS]). La console HABEK est servie sur un nom d'hôte distinct (`SECURITY.md` §3.7) ; ses routes sont réservées en section 7.18.
- **Méthodes** : `GET` ne modifie jamais rien. `POST` crée ou déclenche une action. `PATCH` modifie les champs d'une fiche modifiable. `PUT` remplace une configuration entière. **Aucun `DELETE`** sur une donnée métier (R-005, règle d'or 10) : on archive par `POST …/archive`. Un `PUT`, `PATCH` ou `DELETE` sur un paiement, une quittance, un reversement, une relance ou une version de compte rendu répond `405` avec le code `payment.immutable` (ou `receipt.immutable`, etc.) et le message de `UI_UX_DESIGN.md` « Un paiement validé ne peut pas être modifié. Vous pouvez demander une correction au gérant. » (R-047, E-08). Ces gestionnaires de refus sont écrits explicitement et testés.
- **Montants** : entiers JSON. Le front n'effectue aucun calcul d'argent (DP-45) : l'aperçu d'affectation, les soldes, les statuts, la commission et le montant en lettres viennent de l'API.
- **Une action métier = une requête = une transaction** (`ARCHITECTURE.md` §2.2). Pas de transaction qui s'étend sur plusieurs appels.

### 1.2 Source de vérité du contrat (décision n° 61)

- **Contexte** : le front et le back évoluent plusieurs fois par semaine pendant le pilote, avec un seul développeur. Il faut que le front ne puisse pas compiler contre un contrat qui n'existe plus. `ARCHITECTURE.md` §3.7 recommande un contrat OpenAPI généré depuis le code Rust ; la décision n° 20 qui le porte reste ouverte.
- **Options** :
  - (a) **Code d'abord** : le contrat OpenAPI est généré depuis les types et routes Rust. Le fichier généré est versionné dans le dépôt ; la CI le régénère et échoue si la différence n'est pas validée.
  - (b) **Contrat d'abord** : un fichier OpenAPI écrit à la main fait foi ; le serveur et le client en sont dérivés ou vérifiés.
- **Comparaison** :

| Critère | (a) Code d'abord | (b) Contrat d'abord |
|---|---|---|
| Dérive entre contrat et serveur | Impossible par construction | Possible : exige des tests de conformité de chaque réponse |
| Coût d'un changement | Un seul endroit (le type Rust) | Deux endroits (fichier, puis code) |
| Qualité de la conception | Risque de laisser l'implémentation dicter le contrat | Force à réfléchir au contrat avant de coder |
| Outillage Rust | Génération depuis les structures (bibliothèque à choisir, par exemple `utoipa` ou équivalent) | Générateurs de serveur peu mûrs en Rust ; on écrit les gestionnaires à la main de toute façon |
| Revue | La différence du fichier généré se lit en revue | La différence est le point de départ de la revue |
| Adapté à un seul développeur | Oui | Plus lourd |

- **Choix recommandé** : (a) [À valider, décision n° 61], **complété par la conception sur le papier** : ce document est écrit d'abord et décrit les routes, droits, règles et exemples ; le fichier généré doit ensuite lui correspondre, ce que vérifient les tests de contrat (section 8).
- **Raison** : le client TypeScript généré attrape au build un champ renommé ; avec un seul développeur, un seul endroit à modifier est décisif. Le risque de conception dictée par l'implémentation est traité par la relecture de la différence OpenAPI dans chaque revue.
- **Conséquences** :
  - `openapi.json` est versionné à la racine de `crates/server/` (emplacement à confirmer) ; la CI le régénère et fait échouer la fusion si le fichier du commit diffère du généré.
  - Un détecteur de changement incompatible compare le fichier à celui de la branche principale (outil de comparaison OpenAPI à choisir) et bloque tout retrait, renommage, changement de type ou passage d'optionnel à obligatoire en requête (section 1.3).
  - Le client TypeScript est généré dans le même commit (`web/src/api/generated/`), jamais écrit à la main.
  - Les exemples de ce document deviennent des fichiers `tests/contract/examples/*.json` validés contre le schéma (section 8).

### 1.3 Versionnement et évolution compatible (décision n° 62)

- **Contexte** : `ARCHITECTURE.md` §3.7 et le dossier (§12) disent « versionnée seulement quand elle devient publique ». Pendant le pilote, il reste des versions du front en cache (onglet ouvert, PWA) face à un serveur plus récent, et un retour arrière redéploie un serveur plus ancien (`ARCHITECTURE.md` §10.5).
- **Options** : (a) pas de numéro dans les adresses, évolution additive seulement, version minimale de client déclarée ; (b) préfixe `/api/v1` et coexistence de versions.
- **Choix recommandé** : (a) [À valider, décision n° 62]. (b) double le code à maintenir pour un seul client.
- **Règles** :
  - **Changements autorisés sans condition** : nouvelle route ; nouveau champ **facultatif** en requête ; nouveau champ en réponse ; nouveau code d'erreur ; nouvelle valeur d'énumération **si** le front traite toute valeur inconnue sans planter (affichage neutre « Autre »). Le client généré type les énumérations en union ouverte pour l'imposer.
  - **Changements interdits sans dépréciation datée** : retrait ou renommage d'une route, d'un champ, d'un code d'erreur ; changement de type ; passage d'un champ de requête à obligatoire ; changement de sens d'un champ ; resserrement d'une validation sur des données déjà acceptées.
  - **Dépréciation datée** : l'élément reste en place, marqué `deprecated: true` dans OpenAPI, les réponses portent `Deprecation: true` et `Sunset: <date>`, et une ligne est ajoutée au journal des changements (section 9) avec la date de retrait. **Délai minimal avant retrait : 14 jours ET une livraison complète du front qui n'utilise plus l'élément, ET zéro appel mesuré dans les journaux** (version du client lue dans `X-Client-Version`). Ce délai est une proposition [Hypothèse] ; le dossier (§20.2) fixe seulement 72 heures pour les correctifs prioritaires.
  - **Version du client** : le front envoie `X-Client-Version: <identifiant de build>` sur chaque requête. Le serveur répond avec `X-Min-Client-Version`. Si la version du client est inférieure, la réponse reste servie mais le front affiche le bandeau « Une mise à jour est disponible » et recharge à la prochaine navigation (`ARCHITECTURE.md` §3.6). Une version trop ancienne pour être servie sans risque reçoit `426 client.upgrade_required` : réservé à une migration de données incompatible, annoncée à l'avance (`DEPLOYMENT.md`).
  - **Retour arrière** : un front plus récent peut parler à un serveur de la version précédente (le front et le serveur partent ensemble, mais un onglet reste ouvert). Les requêtes n'utilisent donc jamais un champ nouveau comme obligatoire dans les deux premières livraisons qui suivent son introduction. Les champs inconnus en requête sont refusés (section 4.4) : un champ ajouté par un front plus récent qu'un serveur revenu en arrière provoque `422 request.unknown_field`. Le risque est faible (front et serveur déployés ensemble) et accepté ; le bandeau de mise à jour le résorbe.
  - Toute modification du contrat passe par une ligne du journal des changements (section 9) et par la relecture de la différence du fichier OpenAPI.

## 2. Conventions

### 2.1 Noms

- **Ressources** : noms anglais au pluriel, en minuscules, mots séparés par des traits d'union dans l'adresse (`/api/correction-requests`). Sous-ressources imbriquées une fois au plus (`/api/leases/{id}/amendments`).
- **Champs JSON** : `snake_case`, noms anglais identiques aux colonnes de `DATABASE_SCHEMA.md` quand ils existent (`paid_on`, `external_ref`, `rate_bp`) (décision n° 67). Alternative écartée : `camelCase` (idiome TypeScript) qui obligerait à une conversion à chaque frontière. Les textes affichés sont en français, jamais dans les noms.
- **Énumérations** : valeurs anglaises en `snake_case` comme la base (`cash`, `mobile_money`, `bank_transfer`, `cheque`; `residential`, `commercial`; `draft`, `active`, `notice`, `ended`). Le libellé français vit dans le front (décision n° 30).
- **Identifiants** : UUID v4 en chaîne, opaques, jamais séquentiels (`DATABASE_SCHEMA.md` §1.2). Le numéro de quittance (`number`, « HOR-2026-000094 ») est un champ distinct, jamais un identifiant de route. Les journaux (`audit_log`) n'exposent pas leur `bigint` mais un identifiant opaque.
- **`org_id` jamais dans les ressources** : toutes les ressources appartiennent à l'organisation de la session ; l'identifiant n'est ni reçu ni renvoyé, hors `GET /api/me` (section 4.3).

### 2.2 Montants, taux, pourcentages

| Élément | Convention | Exemple |
|---|---|---|
| Montant | Entier JSON, FCFA, **nom sans suffixe d'unité** : `amount`, `rent`, `charges`, `deposit_amount`, `remaining_amount` | `"amount": 75000` |
| Type OpenAPI | `integer`, `format: int64`, bornes `minimum`/`maximum` : 1 à 999 999 999 en valeur absolue pour un paiement, un loyer, un reversement ; 0 à 999 999 999 pour charges, dépôt, dépense (décision n° 35) | |
| Signe | Positif pour un paiement et ses affectations ; **négatif** pour une contre-passation (`kind: reversal`) et ses affectations ; jamais de signe dans les autres champs | `"amount": -75000` |
| Interdits | Chaîne, flottant, séparateur de milliers, devise dans la valeur, abréviation (« 1,25 M ») (R-001) | |
| Taux de commission | Entier en centièmes de point : `rate_bp` (750 = 7,5 %), 0 à 10 000 (R-015, décision n° 29) | `"rate_bp": 800` |
| Devise | `GET /api/me` renvoie `organization.currency = "XOF"` une fois ; les montants ne répètent pas la devise (R-003) | |
| Montant en lettres | Champ **calculé par le serveur** (`amount_in_words`), jamais par le front ni le moteur PDF (R-053, `ARCHITECTURE.md` §4.2). Formulation « francs CFA » et orthographe traditionnelle : à valider (décision n° 12) | `"soixante-quinze mille francs CFA"` |
| Affichage | « 1 250 000 FCFA » (espace insécable), « − 25 000 FCFA » (vrai signe moins) : **formaté par le front** à partir de l'entier (`UI_UX_DESIGN.md`, Formats) | |

Une requête qui contient un montant non entier (`75000.5`), une chaîne, ou hors bornes est refusée en `422` avec `validation.amount_not_integer` ou `validation.amount_out_of_range`. La base arrondit silencieusement un décimal lié à une colonne `bigint` (`DATABASE_SCHEMA.md` §1.2) : la désérialisation stricte du serveur est donc la seule barrière, et un test le vérifie.

### 2.3 Dates, heures, fuseau

| Élément | Format | Exemple |
|---|---|---|
| Date métier (paiement, échéance, début et fin de bail) | `YYYY-MM-DD`, sans heure ni fuseau, au fuseau de l'organisation (R-002) | `"paid_on": "2026-11-13"` |
| Période | Premier jour du mois, `YYYY-MM-01` | `"period": "2026-10-01"` |
| Instant (saisie, émission, validation) | RFC 3339 en UTC avec `Z`. `Africa/Ouagadougou` vaut UTC+0 sans changement d'heure, mais le code ne repose pas sur cette coïncidence (`ARCHITECTURE.md` §4.6) | `"recorded_at": "2026-11-13T09:15:42Z"` |
| Affichage | JJ/MM/AAAA et 24 h, par le front | `13/11/2026`, `09:15` |

- **« Aujourd'hui » vient du serveur.** `GET /api/me` renvoie `today` (date locale de l'organisation) et `server_time` ; le front les utilise pour la date par défaut du paiement et le contrôle de date future (R-039). Il n'utilise pas l'horloge du téléphone, qui dérive sur les appareils d'entrée de gamme. Le serveur revérifie.
- Une date future pour un paiement est refusée (`validation.date_in_future`). Les champs `recorded_at`, `issued_at`, `occurred_at` ne sont jamais acceptés en requête : la base les impose (`DATABASE_SCHEMA.md` §4, garantie 14).

### 2.4 Téléphones

- **Sortie** : forme canonique `+22670123456` (E.164, sans espace). Le front affiche « +226 70 12 34 56 ».
- **Entrée tolérante** : avec ou sans `+226`, avec espaces, points ou tirets (`70 12 34 56`, `0022670123456`) ; le serveur normalise et refuse ce qui n'est pas `+226` suivi de 8 chiffres : `validation.phone_format` (R-018).

### 2.5 Pagination, tri, filtres

- **Pagination par curseur** (décision n° 63) : `GET /api/payments?limit=25&cursor=…` renvoie `{ "items": [...], "next_cursor": "…" | null }`. `limit` vaut 25 par défaut et 100 au plus (`UI_UX_DESIGN.md` : 25 lignes, « Afficher plus », pas de défilement infini). Le curseur est **opaque, signé et lié à l'organisation, à l'utilisateur et aux filtres** : un curseur rejoué dans un autre contexte est refusé (`request.cursor_invalid`). Alternative écartée : `offset`/`page` (instable quand des paiements arrivent pendant la navigation, lent sur grands volumes).
- **Total** : `total` n'est renvoyé que là où il est peu coûteux et utile (liste des impayés : totaux et tranches calculés à part, section 6.2). Jamais un `COUNT(*)` systématique.
- **Tri** : `sort=-paid_on,tenant_name` (`-` = décroissant). Liste blanche de champs par route ; un autre champ répond `422 request.sort_not_allowed`. Jamais de nom de colonne issu du client dans le SQL (`SECURITY.md` §7.1).
- **Filtres** : paramètres nommés par la ressource (`property_id`, `owner_id`, `manager_id`, `usage`, `status`, `from`, `to`, `method`). Un filtre inconnu répond `422 request.filter_unknown`. Les filtres choisis par l'utilisateur sont mémorisés **côté front** (`UI_UX_DESIGN.md`), pas côté serveur au pilote.
- **Taille d'une réponse de liste** : 30 Ko compressés au plus (`ARCHITECTURE.md` §12.1). Les listes renvoient des **résumés** ; le détail s'obtient par la fiche.

### 2.6 Recherche unique

`GET /api/search?q=Tapsoba` répond en 150 ms au plus (p95 serveur) pour des suggestions dès 2 caractères avec un délai de saisie de 200 ms côté front.

```json
{
  "tenants":  [ { "id": "…", "display_name": "Issouf Tapsoba", "phone": "+22670010203", "unit_labels": ["Villa B2"], "balance": { "remaining_amount": 75000, "status": "due" } } ],
  "units":    [ { "id": "…", "label": "Villa B2", "property_name": "Zogona", "status": "occupied" } ],
  "receipts": [ { "id": "…", "number": "HOR-2026-000093", "amount": 75000, "status": "issued" } ]
}
```

- Huit résultats au plus par groupe. Recherche insensible aux accents et à la casse ; téléphone reconnu avec ou sans `+226` et avec espaces ; un numéro de quittance est reconnu par son préfixe (`UI_UX_DESIGN.md`).
- Implémentation au pilote : `ILIKE` sur colonnes normalisées (`ARCHITECTURE.md` §13.2) ; `pg_trgm` [APRÈS].
- **Portée** : l'agent terrain ne trouve que ses lots et leurs locataires (R-081). Une quittance ne figure dans les résultats que si l'utilisateur a le droit `receipt.read`. Un caissier ne reçoit ni contact de secours ni notes (section 4.5).
- Aucun résultat : `200` avec trois listes vides (le front propose « Créer un locataire »).

### 2.7 Langue et messages

- Une seule langue : le français (R-003). L'en-tête `Accept-Language` est ignoré.
- Les messages sont **à deux niveaux** (décision n° 66) : le serveur renvoie toujours un `code` stable, des `params`, et un `detail` français de repli ; le **front affiche son propre texte centralisé** indexé par `code` (`CODING_RULES.md`). Le texte du front suit `UI_UX_DESIGN.md` (Microcopy) : vouvoiement, dit quoi faire, jamais de code technique. Alternative écartée : texte affiché tel que renvoyé par le serveur (impossible à garder cohérent avec les actions des boutons, qui sont du front).

### 2.8 Transport

| Élément | Règle |
|---|---|
| Type de contenu | `application/json; charset=utf-8` ; erreurs `application/problem+json` |
| Compression | gzip et brotli (connexion lente) |
| Taille maximale d'un corps JSON | 64 Kio (`SECURITY.md` §7.1). Envois de fichiers : limites propres (section 5.6) |
| Cache | `Cache-Control: no-store` sur toute réponse d'API, page publique et PDF servi (`SECURITY.md` §7.3) |
| CORS | Aucun : l'API ne répond qu'à son propre domaine |
| Identifiant de requête | Chaque réponse porte `X-Request-Id` ; c'est le « code d'incident » court affiché dans les écrans d'erreur (`ARCHITECTURE.md` §3.7) |
| Version du client | `X-Client-Version` en requête, `X-Min-Client-Version` en réponse (section 1.3) |

## 3. Erreurs

### 3.1 Format standard

Toute erreur est un `application/problem+json` (RFC 9457) :

```json
{
  "type": "about:blank",
  "title": "Paiement déjà enregistré",
  "status": 409,
  "detail": "Ce paiement semble déjà enregistré (quittance n° HOR-2026-000091). Vérifiez la référence avant de continuer.",
  "code": "payment.external_ref_duplicate",
  "params": { "receipt_number": "HOR-2026-000091" },
  "conflict": { "kind": "payment", "id": "…" },
  "request_id": "INC-7K3F9A"
}
```

| Champ | Rôle |
|---|---|
| `code` | **Stable**, `domaine.motif` en `snake_case`. Jamais renommé ni retiré sans dépréciation datée (section 1.3). Le front s'appuie sur lui, jamais sur `title` ni `detail` |
| `status` | Code HTTP (3.2) |
| `title`, `detail` | Français, repli d'affichage (décision n° 66) ; jamais de pile d'appels, de nom de table ni de valeur d'une autre organisation (`SECURITY.md` §7.1) |
| `params` | Valeurs pour interpoler le texte du front (`{n°}`, `{durée}`, `{rôle}`) |
| `errors` | Tableau des erreurs de validation par champ (3.3) |
| `conflict` | Pour un `409` : la ressource en cause, **seulement si l'appelant a le droit de la voir** (un agent terrain ne reçoit pas le nom d'un locataire hors de ses lots ; la réponse reste générique) |
| `request_id` | Code d'incident court, affiché en petit sur l'écran d'erreur ; le support retrouve la requête en une recherche |
| `retry_after_seconds` | Pour `429` et `503`, avec l'en-tête `Retry-After` |

### 3.2 Correspondance avec les codes HTTP

| HTTP | Usage | Exemples de `code` |
|---|---|---|
| 400 | Requête mal formée ; en-tête obligatoire absent | `request.malformed`, `idempotency.key_missing` |
| 401 | Non authentifié, session expirée, identifiants refusés | `auth.unauthenticated`, `auth.session_expired`, `auth.invalid_credentials` |
| 403 | Droit refusé ; jeton anti-falsification absent ; confirmation du mot de passe requise ; fonction désactivée ou hors plan | `auth.forbidden`, `auth.csrf_invalid`, `auth.reauth_required`, `feature.disabled`, `plan.feature_unavailable` |
| 404 | Ressource inexistante **ou d'une autre organisation** (R-004). Jamais 403 pour une ressource d'une autre organisation | `resource.not_found` |
| 405 | Méthode interdite sur une écriture immuable | `payment.immutable`, `receipt.immutable` |
| 409 | Conflit avec l'état courant : doublon, chevauchement, état incompatible | `payment.external_ref_duplicate`, `lease.overlap`, `unit.has_active_lease` |
| 410 | Lien expiré, révoqué, invitation expirée | `share_link.expired`, `invitation.expired` |
| 412 | Version périmée (`If-Match` ne correspond plus) | `concurrency.version_mismatch` |
| 413 | Corps ou fichier trop gros | `request.too_large`, `upload.too_large` |
| 415 | Type de fichier ou de contenu non accepté | `upload.type_not_allowed`, `import.format_not_supported` |
| 422 | Validation d'un champ ; règle métier non respectée par la saisie ; clé d'idempotence réutilisée avec un autre contenu | `validation.*`, `payment.surplus_not_confirmed`, `idempotency.key_reused_with_different_body` |
| 426 | Version du client trop ancienne | `client.upgrade_required` |
| 428 | `If-Match` obligatoire et absent | `concurrency.version_required` |
| 429 | Limitation du débit | `rate_limit.exceeded`, `auth.rate_limited` |
| 500 | Erreur non prévue (journalisée avec l'identifiant de requête) | `server.unexpected` |
| 503 | Maintenance ou indisponibilité | `server.maintenance` |

**Règle 422 contre 409** : `422` quand la **requête** est fausse en elle-même ou incomplète (montant nul, date future, surplus non confirmé) ; `409` quand elle est correcte mais **incompatible avec l'état** (doublon de référence, chevauchement de baux, lot avec bail actif).

### 3.3 Erreurs de validation par champ

```json
{
  "type": "about:blank",
  "title": "Saisie à corriger",
  "status": 422,
  "detail": "Une ou plusieurs informations sont à corriger.",
  "code": "validation.failed",
  "errors": [
    { "field": "amount", "code": "validation.amount_positive", "params": {} },
    { "field": "payer.third_party.phone", "code": "validation.phone_format", "params": {} }
  ],
  "request_id": "INC-2B8D1C"
}
```

- `field` est un chemin pointé dans le corps de la requête. Toutes les erreurs de validation sont renvoyées **ensemble**, pas une par une (le front affiche le résumé en tête de formulaire, `UI_UX_DESIGN.md` Accessibilité).
- Codes de champ : `validation.required`, `validation.too_long`, `validation.enum`, `validation.amount_positive`, `validation.amount_not_integer`, `validation.amount_out_of_range`, `validation.date_in_future`, `validation.date_order` (R-021), `validation.phone_format`, `validation.rent_positive` (R-022), `validation.rate_out_of_range` (R-015), `validation.email_format`.
- La validation du front est un confort ; le serveur valide toujours (`SECURITY.md` §7.1).

### 3.4 Catalogue des codes d'erreur stables

Les messages de la colonne « Texte du front » reprennent `UI_UX_DESIGN.md` (Microcopy) mot pour mot. Un texte marqué ✱ **n'existe pas** dans `UI_UX_DESIGN.md` : c'est une proposition à valider avec le reste de la microcopy.

| Code | HTTP | Texte du front | Règle |
|---|---|---|---|
| `auth.unauthenticated` / `auth.session_expired` | 401 | ✱ « Votre session a expiré. Reconnectez-vous. » | `SECURITY.md` §3.2 |
| `auth.invalid_credentials` | 401 | ✱ « Identifiant ou mot de passe incorrect. » (identique que l'identifiant existe ou non) | `SECURITY.md` §3.3 |
| `auth.rate_limited` | 429 | « Trop de tentatives. Réessayez dans {durée} ou réinitialisez votre mot de passe. » | `SECURITY.md` §3.3 |
| `auth.forbidden` | 403 | « Vous n'avez pas le droit de {action}. Rôle requis : {rôle}. » (`params.action`, `params.required_role`) | R-081, F02 |
| `auth.csrf_invalid` | 403 | ✱ « Votre session n'est plus valide pour cette action. Rechargez la page. » | `SECURITY.md` §3.2 |
| `auth.reauth_required` | 403 | ✱ « Confirmez votre mot de passe pour continuer. » | `SECURITY.md` §3.4 |
| `auth.totp_required`, `auth.totp_invalid` | 401 | ✱ « Saisissez le code de votre application d'authentification. » | `SECURITY.md` §3.4 |
| `auth.identifier_taken` | 409 | « Cet identifiant a déjà un compte. Voulez-vous vous connecter ? » | F01 |
| `auth.no_active_membership` | 403 | ✱ « Votre compte n'a accès à aucune organisation. Contactez votre gérant. » | F01, DP-85 |
| `auth.organization_required` | 403 | ✱ « Choisissez une organisation pour continuer. » | F01, DP-85 |
| `org.suspended` | 403 | ✱ « Cette organisation est suspendue. Contactez le support. » | F01, DP-85 |
| `invitation.expired` | 410 | « Cette invitation a expiré (validité : 7 jours). Demandez au gérant de la renvoyer. » | F01 |
| `resource.not_found` | 404 | « Introuvable. Cette page n'existe pas ou vous n'y avez pas accès. » | R-004 |
| `feature.disabled`, `plan.feature_unavailable` | 403 | ✱ « Cette fonction n'est pas disponible pour votre organisation. » | R-082, `ARCHITECTURE.md` §8.2 |
| `request.malformed`, `request.unknown_field`, `request.cursor_invalid` | 400 / 422 | ✱ « Un problème est survenu. Réessayez. » + code d'incident | `SECURITY.md` §7.1 |
| `request.too_large`, `upload.too_large` | 413 | ✱ « Ce fichier est trop volumineux. » (`params.max_bytes`) | `SECURITY.md` §7.4 |
| `idempotency.key_missing` | 400 | (erreur de programmation du front : « Un problème est survenu. ») | DP-49 |
| `idempotency.key_reused_with_different_body` | 422 | idem | DP-49 |
| `concurrency.version_required` | 428 | idem | décision n° 64 |
| `concurrency.version_mismatch` | 412 | ✱ « Cette fiche a été modifiée par {nom} à {heure}. Rechargez-la avant de continuer. » | décision n° 64 |
| `rate_limit.exceeded` | 429 | ✱ « Trop de demandes. Réessayez dans {durée}. » | `SECURITY.md` §3.3 |
| `client.upgrade_required` | 426 | ✱ « Une mise à jour est disponible. » | §1.3 |
| `server.unexpected` | 500 | ✱ « Un problème est survenu. Réessayez. Si le problème continue, communiquez le code {request_id} au support. » | `ARCHITECTURE.md` §3.7 |
| `server.maintenance` | 503 | ✱ « Maintenance en cours. Retour prévu à {heure}. » | `DEPLOYMENT.md` |
| `payment.immutable` | 405 | « Un paiement validé ne peut pas être modifié. Vous pouvez demander une correction au gérant. » | R-047, E-08 |
| `payment.external_ref_duplicate` | 409 | « Ce paiement semble déjà enregistré (quittance n° {n°}). Vérifiez la référence avant de continuer. » + [Voir le paiement] | R-041 ; bloquant ou non : question ouverte |
| `payment.surplus_not_confirmed` | 422 | « Le montant dépasse le solde de {25 000 FCFA}. Enregistrer le surplus comme avance ? Il sera déduit de l'échéance suivante. » + [Enregistrer l'avance] [Corriger le montant] | R-046 |
| `payment.confirmation_required` | 422 | « Confirmer un paiement de {1 250 000 FCFA} ? Ce montant est élevé. Vérifiez le montant et le mode avant de valider. » | R-038 |
| `payment.allocation_invalid` | 422 | ✱ « La répartition du paiement n'est pas valide. » (somme supérieure au montant, ligne d'un autre bail, ligne annulée) | R-044 |
| `payment.lease_not_in_progress` | 409 | ✱ « Ce bail n'est pas en cours. » | R-020, DP-39 |
| `payment.already_reversed` | 409 | ✱ « Ce paiement a déjà été corrigé. » | R-048 |
| `payment.reversal_reason_required` | 422 | « Motif obligatoire. » (modale de contre-passation) | R-048 |
| `correction.already_pending` | 409 | ✱ « Une demande de correction est déjà en attente pour ce paiement. » | F09 |
| `receipt.immutable` | 405 | ✱ « Une quittance émise ne peut pas être modifiée. Émettez une quittance rectificative. » | R-051 |
| `receipt.pdf_not_ready` | 409 | « Le paiement est enregistré. La quittance n'est pas encore prête : un nouvel essai est en cours. » + [Actualiser] | R-049 |
| `receipt.recipient_has_no_phone` | 422 | ✱ « Ce locataire n'a pas de numéro WhatsApp. Imprimez la quittance ou ajoutez un numéro. » | F10 |
| `share_link.expired`, `share_link.revoked` | 410 | Page publique S26 « lien expiré » (texte de `UI_UX_DESIGN.md`, S26) | R-054, E-12 |
| `tenant.phone_duplicate` | 409 | « Un locataire avec ce numéro existe déjà : {nom}. » + [Ouvrir la fiche] [Fusionner] | R-017 |
| `unit.reference_duplicate` | 409 | ✱ « Cette référence est déjà utilisée dans ce bien. » | R-011 |
| `unit.has_active_lease` | 409 | « Ce lot a un bail actif. Terminez le bail avant d'archiver le lot. » | R-006 |
| `unit.under_works` | 409 | « Ce lot est en travaux. Remettez-le à « vacant » pour activer un bail. » + [Retirer « en travaux »] | R-093, E-36 |
| `unit.works_requires_vacant` | 409 | ✱ « Seul un lot vacant peut être mis en travaux. Ajoutez plutôt une intervention. » | R-093 |
| `lease.overlap` | 409 | « Ce lot a déjà un bail actif du {date} au {date}. Terminez-le ou choisissez d'autres dates. » | R-020 |
| `lease.not_draft` | 409 | ✱ « Ce bail est déjà activé. Utilisez un avenant pour le modifier. » | R-023 |
| `lease.amendment_retroactive` | 409 | ✱ « Un avenant ne peut pas changer une échéance déjà payée. » | R-023 |
| `user.last_manager` | 409 | « Vous ne pouvez pas retirer le dernier gérant. Nommez d'abord un autre gérant. » | R-081 |
| `template.unknown_variable` | 422 | « La variable {variable} n'existe pas. Variables disponibles : {liste}. » | R-064 |
| `template.variable_missing` | 422 | ✱ « Il manque une information pour ce message : {variable}. » | R-064 |
| `reminder.recipient_has_no_phone` | 422 | ✱ « Ce locataire n'a pas de numéro. Ajoutez-en un pour le relancer. » | F12 |
| `mandate.overlap` | 409 | ✱ « Ce lot a déjà un mandat actif sur cette période. » | R-014 |
| `mandate.rate_out_of_range` | 422 | ✱ « Le taux doit être compris entre 0 % et 100 %. » | R-015 |
| `mandate.rate_change_retroactive` | 422 | ✱ « Un nouveau taux s'applique à partir du 1er d'un mois à venir. » | R-069 |
| `statement.units_without_mandate` | 422 | « Impossible de produire le compte rendu : {n} lots n'ont pas de mandat ({liste}). » + [Attribuer un mandat] | R-075 |
| `statement.preview_stale` | 409 | ✱ « Les chiffres ont changé depuis l'aperçu. Vérifiez le nouvel aperçu avant de valider. » | R-072, R-073 |
| `statement.negative_collected` | 422 | ✱ « Les corrections de la période dépassent les encaissements : le compte rendu ne peut pas être produit. » | DB §11.1 n° 3 |
| `statement.already_validated` | 409 | ✱ « Ce compte rendu est déjà validé. » | R-073 |
| `statement.reopen_forbidden` | 409 | ✱ « Un reversement est enregistré : le compte rendu ne peut plus être rouvert. La correction ira dans la période suivante. » | R-083, DP-17 |
| `payout.amount_mismatch` | 422 | ✱ « Le reversement doit être égal au net à reverser. » | R-077, DB §11.1 n° 8 |
| `expense.frozen` | 409 | ✱ « Cette dépense est déjà imputée à un compte rendu validé. » | R-074 |
| `import.format_not_supported` | 415 | « Format de fichier invalide » + liste des colonnes attendues (F14) | F14 |
| `import.too_many_rows` | 422 | ✱ « Ce fichier dépasse {5 000} lignes. Découpez-le en plusieurs fichiers. » | `SECURITY.md` §7.4 |
| `import.not_cancellable` | 409 | ✱ « Cet import ne peut plus être annulé : des paiements ou des relances s'y rattachent. » | R-080 |
| `import.wrong_state` | 409 | ✱ « Cette étape n'est pas disponible pour cet import. » | R-080 |
| `upload.type_not_allowed` | 415 | ✱ « Ce type de fichier n'est pas accepté. » | `SECURITY.md` §7.4 |

Les codes `import.row_*` (erreurs de ligne d'un fichier : `import.phone_invalid`, `import.date_invalid`, `import.amount_not_integer`, `import.unit_unknown`, `import.duplicate_phone`, …) vivent dans le rapport d'import (section 6.4), pas dans une réponse d'erreur HTTP.

Codes complémentaires (même format, textes ✱ à valider) : `payment.stale_allocation` (409, 3.5), `auth.password_weak` (422), `auth.reset_expired` (410), `statement.not_validated` (409), `statement.reopen_required` (409), `installment.line_has_payments` (409, R-036), `reminder.immutable`, `audit.immutable` (405), `payment_sms.unrecognized` (422, [APRÈS]). `lease.deposit_above_two_months` n'est **jamais** une erreur : c'est un avertissement dans `warnings[]` (R-024, E-22) avec le texte « Le dépôt de garantie dépasse deux mois de loyer. Vérifiez qu'il respecte la loi sur le bail d'habitation. Vous pouvez continuer. ».

### 3.5 Cas des conflits

- Un `409` porte `conflict` quand l'appelant peut voir la ressource : `{ "kind": "payment" | "tenant" | "lease" | "mandate", "id": "…" }`. Le front en tire les boutons [Voir le paiement], [Ouvrir la fiche], [Fusionner].
- **Conflit d'édition** (deux gestionnaires sur la même fiche) : `412 concurrency.version_mismatch` avec `params.current_version`, `params.updated_at` et `params.updated_by_name`. Le front propose de recharger la fiche ; il ne fusionne pas silencieusement (section 5.2).
- **Conflit de concurrence d'argent** (deux caissiers sur le même locataire) : le serveur sérialise par bail (`ARCHITECTURE.md` §4.3). Le second reçoit une réponse normale calculée sur l'état après le premier ; si son affectation n'a plus de sens (échéance déjà soldée), `409 payment.stale_allocation` avec le nouvel aperçu d'affectation.
- **Doublon de référence externe** : code `payment.external_ref_duplicate`. Que l'utilisateur puisse passer outre dépend de la question ouverte n° 4 d'`ARCHITECTURE.md` (R-041). Le contrat réserve le champ de requête `acknowledge_external_ref_duplicate` (booléen, absent par défaut) **sans en fixer le sens** tant que la règle n'est pas tranchée.

## 4. Sécurité du contrat

### 4.1 Authentification

- **Cookie de session** `__Host-session` (`Secure`, `HttpOnly`, `SameSite=Lax`, sans `Domain`). Aucun jeton dans un en-tête, aucune clé d'API (`SECURITY.md` §3.2, décision n° 49).
- **Routes sans session** (liste fermée, toute autre exige une session) : `POST /api/auth/login`, `POST /api/auth/password-reset/complete`, `GET /api/invitations/{token}`, `POST /api/invitations/accept`, `GET /s/{jeton}` et `GET /s/{jeton}/pdf`, `GET /v/…` [APRÈS], `GET /health`. Les jetons passent dans l'adresse ou le corps, jamais dans une requête de recherche ; ils sont masqués dans les journaux du serveur et du proxy (`SECURITY.md` §7.7).
- **Inscription publique** : `POST /api/auth/register` n'existe pas au pilote (404). Les organisations sont créées par la console (décision n° 51, écart avec F01 signalé par `SECURITY.md`).
- Le message de connexion est identique que l'identifiant existe ou non, avec un temps de réponse comparable.

### 4.2 Protection contre la falsification de requête

- Toute requête **non sûre** (`POST`, `PUT`, `PATCH`) exige l'en-tête `X-CSRF-Token` (valeur renvoyée par `GET /api/me` et par la connexion, liée à la session : `sessions.csrf_hash`) **et** un en-tête `Origin` identique à l'hôte de l'application. Absent ou faux : `403 auth.csrf_invalid`, ligne `security.permission_denied` dans l'audit.
- Les envois de fichiers (`multipart/form-data`) sont soumis à la même règle.
- La déconnexion est un `POST`. Aucun `GET` ne modifie l'état, y compris les liens envoyés par message.

### 4.3 Identité de l'organisation

- L'organisation active est **uniquement** celle de la session (`sessions.org_id`). Elle n'est jamais lue dans l'adresse, le corps, un en-tête ou un paramètre. Un changement d'organisation (utilisateur membre de plusieurs) passe par `POST /api/me/organization`, qui revérifie l'adhésion.
- Toute route ouvre un `OrgTx` lié à cette organisation (DP-48). Une ressource d'une autre organisation répond **404 `resource.not_found`**, jamais 403, sans aucune donnée de l'autre organisation, et une ligne `security.cross_org_attempt` est écrite dans l'audit de l'organisation de l'appelant avec une alerte immédiate (R-004, `SECURITY.md` §6.1).
- Les identifiants imbriqués sont revérifiés : un `lease_id` de l'organisation A dans le corps d'une requête sur une ressource de A, mais appartenant à B, échoue de la même façon (clés étrangères composites, `DATABASE_SCHEMA.md` §1.2).
- Les curseurs de pagination (2.5) et les opérations longues (5.4) sont liés à l'organisation. La table `jobs` n'ayant pas de politique de sécurité par ligne (`DATABASE_SCHEMA.md` §6.5), `GET /api/operations/{id}` filtre **explicitement** sur l'organisation de la session ; un test d'isolation le couvre.

### 4.4 Validation à l'entrée

- Désérialisation typée et stricte : champs inconnus refusés (`request.unknown_field`), énumérations fermées, UUID, dates, entiers bornés, longueurs maximales, caractères de contrôle retirés.
- Structures d'entrée **explicites par route**, jamais de liaison automatique d'un JSON vers une ligne de table (affectation de masse, `SECURITY.md` §7.2).
- Listes blanches pour le tri et les filtres (2.5).

### 4.5 Visibilité des champs selon le rôle

Reprend `SECURITY.md` §4.2 [À valider] : un champ non visible est **absent** de la réponse (pas `null`), accompagné d'un drapeau `…_masked: true` quand l'écran doit afficher un masque.

| Champ | Gérant, gestionnaire, propriétaire | Caissier | Agent terrain | Comptable |
|---|---|---|---|---|
| Nom, téléphone, lot, solde du locataire | oui | oui | ses lots | oui |
| Contact de secours, notes du locataire | oui | absent | absent | absent |
| Pièce d'identité (si activée) | masquée ; `GET …/id-document` la révèle, action tracée (`tenant.id_document_viewed`) | jamais | jamais | jamais |
| Coordonnées de paiement du mandant | gérant seul : `GET /api/owners/{id}/payout-details`, tracé | jamais | jamais | jamais |
| Adresse IP de l'audit | gérant, comptable | jamais | jamais | lecture |

### 4.6 Droits par action

La matrice de `CONTEXTE.md` (reprise dans `SECURITY.md` §4.1) est codée **une seule fois** (`identity::authorize`) ; chaque route déclare l'action qu'elle exige, et un test par couple rôle × action en est généré (`TESTING_QA.md`). Refus par défaut : une action sans règle explicite est refusée.

Abréviations : **G** gérant (`manager`), **Ge** gestionnaire (`property_manager`), **C** caissier (`cashier`), **A** agent terrain (`field_agent`), **Co** comptable en lecture (`accountant`), **P** propriétaire direct (`owner`, plan Propriétaire : tous les droits), **Col** collaborateur (`collaborator`, plan Propriétaire). « * » : droit réglable (`role_settings`). « (s) » : limité à ses lots assignés. « (L) » : lecture seule.

| Action (code de droit) | G | Ge | C | A | Co | P | Col | Source |
|---|---|---|---|---|---|---|---|---|
| `catalog.read` : voir biens, lots, locataires, baux | oui | oui | (L) | (s) | (L) | oui | (L)¹ | dossier §11.2 |
| `catalog.write` : créer, modifier biens, lots, locataires, baux, lignes d'échéance | oui | oui | non | non | non | oui | non | idem |
| `payment.record` : enregistrer un paiement | oui | * | oui | non² | non | oui | * | idem, `role_settings` |
| `payment.read` : journal des paiements | oui | oui | oui | non | (L) | oui | oui | S11 |
| `payment.request_correction` | oui | oui | oui | non | non | oui | ¹ | R-048 |
| `payment.reverse` : contre-passer (confirmation du mot de passe) | oui | non | non | non | non | oui | non | R-048 |
| `receipt.issue` : émettre, réimprimer, envoyer une quittance | oui | oui | oui | non | (L) | oui | oui | dossier §11.2 |
| `arrears.read` : tableau des impayés | oui | oui | (L) | (s) | (L) | oui | oui¹ | idem |
| `reminder.write` : relancer, noter une promesse | oui | oui | non | (s) | non | oui | oui | idem |
| `statement.read` : voir le compte rendu (plan Agence) | oui | (L) | non | non | (L) | — | — | idem, R-082 |
| `statement.validate` : produire, valider, rouvrir (plan Agence) | oui | non | non | non | non | — | — | R-076 |
| `payout.record` : enregistrer un reversement (plan Agence) | oui | non | non | non | non | — | — | R-076 |
| `mandate.write` : mandants, mandats, avenants de taux (plan Agence) | oui | non | non | non | non | — | — | F05 |
| `expense.write` : saisir un travail ou une dépense | oui | oui | non | proposer³ | non | oui | non | dossier §11.2 |
| `expense.impute` : changer l'imputation | oui | non | non | non | non | oui | non | R-084 |
| `export.run` : exporter | oui | * | non | non | oui | oui | non | dossier §11.2 |
| `import.run` : importer | oui | * | non | non | non | oui | non | F14 |
| `user.manage` : utilisateurs, rôles, invitations, sessions | oui | non | non | non | non | oui | non | R-081 |
| `audit.read` : journal d'audit | oui | non | non | non | (L) | oui | non | F03 |
| `settings.read` : paramètres de l'organisation | oui | partiel⁴ | partiel⁴ | partiel⁴ | partiel⁴ | oui | partiel⁴ | S17 |
| `settings.write` : paramètres, modèles de messages | oui | non | non | non | non | oui | non | `SECURITY.md` §4.1 (déduit) |
| `support.grant` : accorder ou révoquer un accès support | oui | non | non | non | non | oui | non | R-009 (déduit) |
| `search.use` : recherche unique | oui | oui | oui | (s) | oui | oui | oui | R-081 |
| `feedback.create`, `events.create` | tout utilisateur connecté | | | | | | | |

¹ Collaborateur : la matrice dit « enregistrer les paiements et relancer ». Voir les impayés, les biens et demander une correction en découlent mais ne sont pas écrits : **à confirmer** (Questions ouvertes).
² Agent terrain : « Non (P1 : sur place avec validation) ». Refusé au pilote.
³ Agent terrain : « Proposer ». Le mécanisme de proposition (statut, validation par qui) n'est pas défini : **la route est refusée à l'agent terrain tant que la règle manque** (Questions ouvertes).
⁴ Lecture des seuls éléments utiles au rôle (par exemple les modèles de messages pour qui relance, l'en-tête de quittance pour qui encaisse).

Les routes marquées 🔐 exigent aussi la **confirmation par mot de passe** valable 5 minutes (`SECURITY.md` §3.4) : contre-passation, validation, réouverture et reversement d'un compte rendu, accord ou révocation d'un accès support, invitation ou changement de rôle d'un gérant, désactivation d'un utilisateur, export, changement de numéro de téléphone du compte. Elles répondent `403 auth.reauth_required` si la confirmation manque, après `POST /api/auth/reauth`. La table `sessions` de `DATABASE_SCHEMA.md` n'a pas de colonne pour mémoriser cette confirmation : un ajout est nécessaire (Questions ouvertes).

Les actions réservées au plan Agence répondent `403 plan.feature_unavailable` pour une organisation du plan Propriétaire (R-082, F01). La base impose aussi la restriction pour certaines actions d'argent (contre-passer, valider, rouvrir, reverser, mandat hors plan Agence, accès support sans accord) : `SECURITY.md` §4.3.

### 4.7 Limitation du débit

Compteurs dans PostgreSQL (`SECURITY.md` §3.3) ; réponse `429` avec `Retry-After` en secondes.

| Cible | Limite proposée |
|---|---|
| Connexion, par identifiant | 5 échecs consécutifs, puis attente croissante 1, 5, 15 minutes |
| Connexion, par adresse IP | 20 échecs en 10 minutes |
| Réinitialisation de mot de passe, invitation | 5 demandes par heure et par identifiant |
| Code TOTP | 5 essais par minute |
| Écritures d'argent (`POST` de paiement, contre-passation, validation, reversement) | 60 par minute et par session [Hypothèse] |
| API en général | plafond par session (300 requêtes par minute [Hypothèse]) et plafond global par adresse |
| Pages publiques (`/s/…`, `/v/…`) | plafond par adresse, plus bas |
| `POST /api/events`, `POST /api/client-errors` | plafond par session, réponses `204` même si des événements sont écartés |

La clé d'idempotence rend tout nouvel essai après `429` sans risque.

### 4.8 En-têtes de réponse de sécurité

Ceux de `SECURITY.md` §7.3 (HSTS, CSP stricte, `nosniff`, `Referrer-Policy: no-referrer`, `Permissions-Policy`, COOP/CORP, `X-Frame-Options`, `Cache-Control: no-store`, `X-Robots-Tag: noindex` sur la page publique) s'appliquent à toute réponse. Un test de contrat les vérifie sur chaque type de page.

## 5. Fiabilité

### 5.1 Clés d'idempotence (DP-49)

- **En-tête** `Idempotency-Key: <UUID v4>`, créé par le front **à l'ouverture du formulaire** de saisie et réutilisé à chaque nouvel essai. « Encaisser un autre loyer » ouvre un nouveau formulaire, donc une nouvelle clé.
- **Obligatoire** (absence : `400 idempotency.key_missing`) sur :

| Route | Pourquoi |
|---|---|
| `POST /api/payments` | R-042, E-07 |
| `POST /api/payments/{id}/reversal` | une contre-passation ne se double pas |
| `POST /api/statements`, `POST /api/statements/{id}/versions` | validation d'un compte rendu |
| `POST /api/statements/{id}/payout` | un reversement ne se double pas |
| `POST /api/cash-days/{date}/close` [APRÈS] | clôture de caisse |

- **Acceptée et recommandée** sur toute autre création qui écrit un journal en ajout seul : `POST /api/reminders`, `POST /api/leases/{id}/activate`, `POST /api/import-batches/{id}/confirm`, `POST /api/receipts/{id}/sends`.
- **Comportement** (table `idempotency_keys`, clé unique par organisation, insérée dans la même transaction que l'écriture) :
  - première requête : exécutée, réponse enregistrée ;
  - nouvel essai identique (double clic, perte de réponse, reprise après coupure) : **même statut, même corps**, en-tête `Idempotent-Replay: true`, rien n'est créé ;
  - deux requêtes simultanées : la seconde attend la fin de la première puis reçoit la même réponse ;
  - même clé, **contenu différent** (empreinte du corps différente) : `422 idempotency.key_reused_with_different_body`, aucun effet ;
  - même clé sur **une autre route** : même refus (`DATABASE_SCHEMA.md` §11.2 A11) ;
  - une requête qui a échoué avant validation n'a rien écrit : le nouvel essai s'exécute normalement. Une erreur métier (`422`, `409`) n'est donc **pas** mémorisée.
- **Conservation** : 30 jours [valeur proposée]. Une clé purgée ne protège plus ; le contrôle de doublon de référence externe (R-041) reste distinct.
- **Réponse perdue** : le front conserve les données saisies et la clé, affiche « Connexion perdue. Vos informations sont conservées. Réessayez dès que le réseau revient. Si le paiement a déjà été enregistré, il ne sera pas créé deux fois. » (`UI_UX_DESIGN.md`).

### 5.2 Verrouillage optimiste (décision n° 64)

- **Contexte** : deux gestionnaires peuvent modifier la même fiche locataire ou lot. `DATABASE_SCHEMA.md` (§1.2, question ouverte 15) laisse « le dernier écrit l'emporte » et reporte `row_version` [APRÈS]. Un numéro de téléphone écrasé sans avertissement fausse ensuite les relances et la détection de doublons.
- **Options** : (a) `row_version` entier sur les fiches modifiables dès T1, contrôlé par `If-Match` ; (b) dernier écrit l'emporte, audit avant/après pour rattraper ; (c) comparaison de tous les champs à la sauvegarde.
- **Choix recommandé** : (a) [À valider, décision n° 64], **ce qui avance `row_version` de [APRÈS] à [PILOTE]** pour les tables modifiables. (b) perd des saisies sans prévenir ; (c) coûte plus cher que (a) et ne détecte pas un aller-retour A→B→A.
- **Contrat** :
  - Toute ressource modifiable renvoie `version` (entier) et `ETag: "7"`. Les ressources en ajout seul n'en ont pas.
  - `PATCH` et `PUT` exigent `If-Match: "7"`. Absent : `428 concurrency.version_required`. Périmé : `412 concurrency.version_mismatch`, sans modification.
  - Après une modification réussie, la réponse renvoie la ressource avec sa nouvelle `version`.
- **Ressources concernées** : `properties`, `units`, `tenants`, `owners`, `mandates`, `leases` (champs modifiables), `interventions`, `providers`, `message_templates`, `org_settings`, `role_settings`.

### 5.3 Transactions, ordre et délais

- Une requête d'argent s'exécute en une transaction : contrôles métier, verrou consultatif du bail (même clé que la base, `DATABASE_SCHEMA.md` §4.5), insertion du paiement et des affectations, **compteur de quittance en dernier**, instantané de la quittance, ligne d'audit, insertion de la tâche de rendu PDF. Aucun appel au stockage de fichiers ni au moteur PDF dans la transaction.
- Délais d'attente côté serveur sur toutes les opérations ; une requête qui dépasse son délai répond `503` sans effet partiel.
- Budgets serveur (p95, `ARCHITECTURE.md` §12.1) : lecture simple 150 ms ; recherche 150 ms ; tableau des impayés (500 lots) 300 ms ; enregistrement d'un paiement avec quittance, sans PDF, 300 ms ; aperçu d'un compte rendu (50 lots) 1 s.

### 5.4 Opérations longues et suivi d'état (décision n° 65)

- **Contexte** : l'import (analyse jusqu'à 5 000 lignes), le rendu PDF et l'export ne tiennent pas dans une requête courte sur un réseau lent.
- **Options** : (a) la requête répond `202` avec une ressource `operation` que le front interroge (polling) ; (b) flux d'événements poussés (SSE ou WebSocket) ; (c) requête qui attend la fin.
- **Choix recommandé** : (a) [À valider, décision n° 65]. Plus simple à exploiter (pas de connexion longue à travers un proxy), reprise naturelle après coupure, et déjà couvert par la table `jobs`.
- **Contrat** :
  - Une route asynchrone répond `202 Accepted`, `Location: /api/operations/{id}` et `{ "operation": { "id": "…", "kind": "import_preview", "state": "queued" } }`.
  - `GET /api/operations/{id}` renvoie `{ "id", "kind", "state": "queued" | "running" | "done" | "failed", "progress": { "done": 1200, "total": 2000 } | null, "result": { … } | null, "error": { "code": "…", "params": {} } | null }`. Les états sont ceux de `jobs.state`. L'accès est limité à l'organisation et à l'utilisateur à l'origine.
  - **Cadence du polling** : une requête après 1 s, puis toutes les 2 s, puis toutes les 5 s au-delà de 20 s ; l'en-tête `Retry-After` du serveur prime. Au-delà de 10 s sans fin, le front affiche « La connexion est lente… » et [Réessayer] (`UI_UX_DESIGN.md`).
  - Une opération est **idempotente** : la redemander (même fichier, même clé) renvoie la même opération.
- **Opérations asynchrones du contrat** : analyse d'un import (6.4), confirmation d'un import, rendu PDF d'une quittance ou d'un compte rendu (5.5), export (7.12 [APRÈS]).

### 5.5 Téléchargement de fichiers et de PDF (décision n° 68)

- **Options** : (a) l'API sert le fichier en flux après contrôle d'accès ; (b) l'API renvoie une adresse signée du stockage objet que le navigateur ouvre.
- **Choix recommandé** : (a) [À valider, décision n° 68]. Les PDF pèsent quelques dizaines de Ko et les volumes sont faibles ; (a) garde la CSP `connect-src 'self'`, `Cache-Control: no-store`, le contrôle d'organisation et l'audit en un seul endroit. (b) reste prévu pour de gros documents [APRÈS].
- **Contrat** :
  - Quittance : `GET /api/receipts/{id}/pdf?paper=a4|a5` ; compte rendu : `GET /api/statements/{id}/versions/{version}/pdf`. `200 application/pdf` si prêt, sinon `409 receipt.pdf_not_ready` avec `Retry-After`. Le front lit d'abord `receipt.pdf.state` (`pending`, `ready`, `failed`) et n'expose [Télécharger] et [Imprimer] que si `ready`.
  - En-têtes : `Content-Type: application/pdf`, `Content-Disposition: attachment; filename="quittance-HOR-2026-000094.pdf"` (nom généré depuis le numéro, jamais depuis une saisie), `X-Content-Type-Options: nosniff`, `Cache-Control: no-store`. `?disposition=inline` n'est accepté que pour les PDF **produits par le système** ; un document téléversé est toujours servi en `attachment` (`SECURITY.md` §7.4).
  - Aucun PDF ne porte de métadonnée d'auteur ; l'empreinte SHA-256 est enregistrée (`receipt_files`) et exposée dans `receipt.pdf.sha256`.
  - Le rendu produit les deux formats A4 et A5 (A5 par défaut au pilote, `org_settings.default_paper`) pour éviter une route de génération à la demande [À valider avec le moteur PDF, décision n° 7].
  - L'aperçu à l'écran vient de `receipt.snapshot` (champs structurés). Que l'impression parte du PDF ou d'un aperçu HTML est une question ouverte : elle conditionne les tests visuels.

### 5.6 Envoi de fichiers

| Fichier | Route | Limites |
|---|---|---|
| Import CSV ou XLSX | `POST /api/import-batches` | 5 Mo, 5 000 lignes ; extension **et** contenu vérifiés ; `.xls` et `.xlsm` refusés ; aucune formule évaluée (`SECURITY.md` §7.4) |
| Logo, signature | `PUT /api/settings/logo`, `PUT /api/settings/signature` | PNG ou JPEG, 200 Ko ; réencodés (métadonnées retirées) |
| Documents [APRÈS] | `POST /api/documents` | PDF, JPEG, PNG, 10 Mo ; jamais affichés dans l'origine de l'application |

- `multipart/form-data`, jeton anti-falsification obligatoire, clé de stockage générée par le serveur (`org/{org_id}/…`), jamais le nom d'origine. Réponse : métadonnées (`id`, `size_bytes`, `content_type`, `sha256`). Pas d'envoi fractionné ni de reprise au pilote : 5 Mo passent en quelques secondes en « 4G lente » et un échec se relance.
- Erreurs : `413 upload.too_large`, `415 upload.type_not_allowed`, `415 import.format_not_supported`.

## 6. Les cinq parcours de la maquette, en détail

Les cinq parcours de `MAQUETTE.md` (niveau 1) sont les premiers à construire et à tester de bout en bout. Pour chacun : séquence d'appels, exemples de requête et de réponse, erreurs, règles, droits, priorité.

**Lecture des exemples.** Identifiants fictifs (`…0001`), organisation « Agence Horizon Démo » (préfixe `HOR`), date de la maquette (vendredi 13/11/2026, 09:15). Les en-têtes `Cookie`, `X-CSRF-Token`, `Origin` et `X-Client-Version` sont omis ; `Idempotency-Key` est montré quand il est obligatoire. Les statuts calculés sont `upcoming`, `due`, `partial`, `paid`, `late` (échéance) ; `validated`, `reversed` (paiement) ; `issued`, `cancelled` (quittance). Les chemins sont préfixés par `/api`. Les exemples sont **illustratifs** ; les fichiers de référence sont `tests/contract/examples/*.json` (section 8).

### 6.1 Parcours 1 : encaisser et envoyer une quittance (caissière)

F09, F10 · R-037 à R-056, R-088, R-096 · droits `payment.record`, `receipt.issue` · **[PILOTE] T1**.

| # | Écran S04 | Appel | Remarque |
|---|---|---|---|
| 1 | Recherche « Tapsoba » (200 ms de délai) | `GET /search?q=Tapsoba` | 150 ms au plus ; suggestions dès 2 caractères (2.6) |
| 2 | Carte du locataire, solde, échéance due | `GET /leases/{lease_id}/payment-context` | un seul aller-retour : bail, locataires, lignes ouvertes dans l'ordre d'affectation, montant proposé, seuil, date du jour |
| 3 | Montant modifié (facultatif) | `POST /payments/preview` | aperçu d'affectation calculé **par le serveur** ; aucune écriture, aucune clé |
| 4 | « Encaisser et envoyer la quittance » | `POST /payments` + `Idempotency-Key` | transaction unique : paiement, affectations, quittance numérotée, audit, tâche PDF |
| 5 | Aperçu de la quittance (S25) | (réponse du 4) | `receipt.snapshot` suffit à l'affichage ; le PDF suit |
| 6 | « Envoyer sur WhatsApp » | `POST /receipts/{id}/sends` | crée le lien, consigne l'envoi, renvoie l'adresse `wa.me` prête |
| 7 | Journal (S11) | `GET /payments?sort=-recorded_at&limit=25` | statut « Validé » |

**Allers-retours** après l'ouverture de l'écran : recherche, contexte, validation = **3** pour le chemin nominal (`ARCHITECTURE.md` §12.1) ; l'aperçu s'y ajoute seulement si le montant proposé est modifié.

**Contexte de paiement.**

```
GET /api/leases/00000000-0000-4000-8000-000000000101/payment-context
```

```json
{
  "lease": { "id": "…0101", "status": "active", "kind": "residential", "due_day": 13,
             "unit": { "id": "…0201", "label": "Villa B2", "property_name": "Zogona" } },
  "tenants": [ { "id": "…0301", "display_name": "Issouf Tapsoba", "phone": "+22670010203", "whatsapp": true } ],
  "balance": { "remaining_amount": 75000, "overdue_amount": 0, "credit_amount": 0 },
  "open_lines": [
    { "installment_id": "…0401", "installment_line_id": "…0501", "period": "2026-11-01",
      "due_date": "2026-11-13", "kind": "rent", "due_amount": 75000, "remaining_amount": 75000, "status": "due" }
  ],
  "suggested_amount": 75000,
  "default_method": "cash",
  "today": "2026-11-13",
  "confirm_threshold": 1000000
}
```

`suggested_amount` : reste dû de l'échéance la plus ancienne non soldée [proposition]. `confirm_threshold` : paramètre de l'organisation (R-038, 1 000 000 FCFA par défaut, valeur à fixer).

**Enregistrement.**

```
POST /api/payments
Idempotency-Key: 5d0c7a52-3b1e-4c9d-8a47-1f2e3d4c5b6a
```

```json
{
  "lease_id": "…0101",
  "amount": 75000,
  "paid_on": "2026-11-13",
  "method": "cash",
  "external_ref": null,
  "payer": { "kind": "tenant", "tenant_id": "…0301" },
  "note": null,
  "allocations": null,
  "surplus": null,
  "confirm_high_amount": false,
  "replaces_payment_id": null
}
```

| Champ | Règle |
|---|---|
| `amount` | entier 1 à 999 999 999 (R-037) |
| `paid_on` | date du jour ou passée (R-039) ; la journée de caisse (`cash_day`) est celle de la saisie, calculée par le serveur (R-089, DP-23) |
| `method` | `cash`, `mobile_money`, `bank_transfer`, `cheque` (R-040) |
| `external_ref` | facultatif ; contrôle de doublon par mode (R-041) |
| `payer` | `tenant` (un locataire **du bail**) ou `third_party` avec `third_party_name`, `third_party_phone` facultatif (R-088, E-33) |
| `allocations` | `null` : répartition par défaut du serveur (échéance la plus ancienne ; à date égale loyer, charges, exceptionnelles : R-043, R-087). Sinon liste `{ installment_line_id, amount }` ajustée par l'utilisateur ; la somme ne dépasse pas `amount` (R-044) |
| `surplus` | `"advance"` **obligatoire** si `amount` dépasse le reste dû total (R-046) ; sinon `422 payment.surplus_not_confirmed` avec `params.surplus_amount` |
| `confirm_high_amount` | `true` obligatoire si `amount` dépasse `confirm_threshold` (R-038) ; sinon `422 payment.confirmation_required` |
| `replaces_payment_id` | réservé au paiement qui remplace un paiement contre-passé (R-096, E-38) ; voir 7.6 |

```
201 Created
```

```json
{
  "payment": {
    "id": "…0601", "kind": "payment", "status": "validated",
    "lease_id": "…0101", "amount": 75000, "method": "cash", "external_ref": null,
    "paid_on": "2026-11-13", "recorded_at": "2026-11-13T09:15:42Z", "cash_day": "2026-11-13",
    "received_by": { "id": "…0701", "display_name": "Awa Zongo" },
    "payer": { "kind": "tenant", "tenant_id": "…0301", "display_name": "Issouf Tapsoba" },
    "allocations": [ { "installment_line_id": "…0501", "installment_id": "…0401", "period": "2026-11-01", "kind": "rent", "amount": 75000 } ],
    "unallocated_amount": 0
  },
  "receipt": {
    "id": "…0801", "number": "HOR-2026-000094", "series": "official", "kind": "standard", "status": "issued",
    "issued_on": "2026-11-13", "amount": 75000, "amount_in_words": "soixante-quinze mille francs CFA",
    "remaining_after": 0, "advance_amount": 0,
    "snapshot": { "…": "contenu figé : parties, lot, période, ventilation, solde restant, payeur, mentions" },
    "pdf": { "state": "pending", "papers": ["a4", "a5"], "sha256": null }
  },
  "installments": [
    { "id": "…0401", "period": "2026-11-01", "due_date": "2026-11-13", "status": "paid",
      "due_amount": 75000, "allocated_amount": 75000, "remaining_amount": 0 }
  ],
  "warnings": []
}
```

- Tant que l'organisation est en phase `setup`, la série est `trial`, le numéro porte le préfixe `ESSAI-` et la quittance le filigrane « ESSAI » (DB §4.3, décision n° 36). La numérotation officielle démarre à 000001 à la bascule en `live`.
- `warnings[]` porte les avertissements **non bloquants** (code et `params`). Au pilote il reste vide hors cas T3 : `payment.paid_on_in_closed_cash_day` (R-089, [APRÈS]).
- **Nouvel essai** (même clé, même corps) : `201` identique, `Idempotent-Replay: true`, aucune nouvelle ligne (E-07).

**Variantes (cas de référence de DOMAINE.md).**

| Cas | Entrée | Résultat attendu dans la réponse |
|---|---|---|
| Paiement partiel (E-05) | `amount: 50000` sur un loyer dû de 75 000 | `installments[0].status = "partial"`, `remaining_amount: 25000` ; `receipt.remaining_after: 25000`. À partir de J+1 le même état s'affiche `late` (R-033, DP-13, E-25) |
| Avance (E-06) | `amount: 100000`, `surplus: "advance"` | deux affectations : 75 000 sur la ligne du 13/11, 25 000 sur la ligne de l'échéance du 13/12 si elle est générée ; `receipt.advance_amount: 25000` |
| Avance sans échéance suivante (E-31) | idem, échéance du mois suivant non générée | `unallocated_amount: 25000` ; le crédit du bail (`balance.credit_amount`) s'affecte à la génération de l'échéance (R-086) |
| Doublon de référence (R-041) | `method: "mobile_money"`, `external_ref` déjà utilisée | `409 payment.external_ref_duplicate`, `conflict: { kind: "payment", id }`, `params.receipt_number` |
| Paiement par un tiers (E-33) | `payer: { kind: "third_party", third_party_name: "…" }` | quittance au nom du locataire du bail avec mention du tiers |
| Montant supérieur au seuil (E-09) | `amount: 1250000` | `422 payment.confirmation_required` ; renvoyer avec `confirm_high_amount: true` |
| Date future (R-039) | `paid_on: "2026-11-14"` | `422 validation.failed`, `errors[0].code = "validation.date_in_future"` |
| Connexion perdue (R-042) | même requête rejouée avec la même clé | réponse enregistrée, aucun doublon |
| Deux caissiers, même locataire | requêtes simultanées | sérialisation par bail ; la seconde est calculée sur l'état après la première (3.5) |

**Erreurs de `POST /payments`.**

| Code | HTTP | Cause |
|---|---|---|
| `idempotency.key_missing` | 400 | en-tête absent |
| `idempotency.key_reused_with_different_body` | 422 | même clé, contenu différent |
| `validation.failed` | 422 | montant nul ou non entier, mode inconnu, date future, `payer` incohérent |
| `payment.surplus_not_confirmed` | 422 | montant supérieur au reste dû sans `surplus: "advance"` |
| `payment.confirmation_required` | 422 | montant au-dessus du seuil sans `confirm_high_amount` |
| `payment.allocation_invalid` | 422 | somme supérieure au montant, ligne d'un autre bail ou annulée |
| `payment.external_ref_duplicate` | 409 | référence déjà utilisée pour ce mode |
| `payment.lease_not_in_progress` | 409 | bail brouillon ou terminé |
| `payment.stale_allocation` | 409 | l'affectation proposée n'a plus de sens (échéance soldée entre-temps) ; la réponse porte le nouvel aperçu |
| `auth.forbidden` | 403 | rôle sans `payment.record` (agent terrain, comptable ; gestionnaire ou collaborateur si le droit réglable est retiré) |
| `resource.not_found` | 404 | bail d'une autre organisation |

**Envoi de la quittance.**

```
POST /api/receipts/…0801/sends
Idempotency-Key: 9a1b2c3d-…   (recommandée)
```

```json
{ "channel": "whatsapp" }
```

```json
{
  "send": { "id": "…0901", "channel": "whatsapp", "sent_at": "2026-11-13T09:16:10Z", "sent_by": "…0701" },
  "share_link": { "id": "…0a01", "url": "https://app.<domaine>/s/<jeton>", "expires_at": "2026-12-13T09:16:10Z" },
  "recipient": { "display_name": "Issouf Tapsoba", "phone_last2": "03" },
  "message": { "text": "Bonjour …", "whatsapp_url": "https://wa.me/22670010203?text=…" }
}
```

- `channel` : `whatsapp`, `print_a4`, `print_a5`, `link_copied`, `sms` (valeurs de `receipt_sends.channel`). L'envoi est **consigné au clic** (R-055, F10).
- Le serveur compose le message (minimisation de `SECURITY.md` §7.8 : nom, montant, mois, nom et téléphone de l'agence, lien ; rien d'autre) et l'adresse `wa.me`. Le front affiche **le nom et les deux derniers chiffres** (`recipient`) avant d'ouvrir WhatsApp.
- **Le jeton du lien n'apparaît que dans cette réponse** : seule son empreinte est stockée (DB §6.7). « Copier le lien » crée donc un nouveau lien. Durée : `share_link_ttl_days` de l'organisation (1 à 365, 30 par défaut, R-054).
- Locataire sans numéro WhatsApp : `422 receipt.recipient_has_no_phone` (le front propose l'impression ou la saisie du numéro).
- **Impression** : `GET /receipts/{id}/pdf?paper=a5`, puis `POST …/sends` avec `print_a5` pour consigner.

**Correction (E-38)** : le caissier demande, le gérant contre-passe. Voir 7.6.

### 6.2 Parcours 2 : voir et relancer les impayés (gestionnaire)

F11, F12, F22 (partiel) · R-057 à R-065, R-097, DP-36, DP-40 · droits `arrears.read`, `reminder.write` · **[PILOTE] T1**.

| # | Écran | Appel | Remarque |
|---|---|---|---|
| 1 | S03 « À faire aujourd'hui » | `GET /home` | promesses non tenues (indicateur T1, DP-36), rappels à J−3, échéances dues aujourd'hui |
| 2 | S05, total et tranches | `GET /arrears?owner_id=…&sort=-oldest_days_late` | une ligne par bail ; totaux par échéance |
| 3 | « Relancer » | `POST /reminders/draft` | texte rendu par le serveur ; avertissements non bloquants ; **aucune écriture** |
| 4 | « Ouvrir WhatsApp » puis retour « Oui » | `POST /reminders` | seule écriture, après la confirmation de l'utilisateur (R-061) |
| 5 | La ligne affiche relance, promesse, prochaine action | `GET /arrears` | la ligne reste (R-059) |

**Tableau des impayés.** Exemple filtré sur le mandant M3 d'après les ancrages de la maquette (valeurs illustratives).

```
GET /api/arrears?owner_id=…0b03&sort=-oldest_days_late&limit=25
```

```json
{
  "summary": {
    "total_overdue_amount": 345000,
    "by_band": [
      { "band": "1_7",     "installments": 0, "amount": 0 },
      { "band": "8_30",    "installments": 2, "amount": 155000 },
      { "band": "31_59",   "installments": 1, "amount": 35000 },
      { "band": "60_plus", "installments": 2, "amount": 155000 }
    ]
  },
  "items": [
    { "lease_id": "…0102", "tenant": { "id": "…0302", "display_name": "Pauline Yaméogo" },
      "unit": { "id": "…0202", "label": "Maison …", "property_name": "Cissin" },
      "overdue_installments": 3, "oldest_days_late": 70, "remaining_amount": 105000,
      "last_reminder": null, "promise": null, "next_action_on": null },
    { "lease_id": "…0103", "tenant": { "id": "…0303", "display_name": "Yacouba Compaoré" },
      "unit": { "id": "…0203", "label": "Atelier …", "property_name": "…" },
      "overdue_installments": 2, "oldest_days_late": 60, "remaining_amount": 240000,
      "last_reminder": null, "promise": null, "next_action_on": null }
  ],
  "next_cursor": null
}
```

- **Tranches** : clés `1_7`, `8_30`, `31_59`, `60_plus` (DP-12, R-058). Un retard de 30 jours est en `8_30`, de 31 en `31_59`, de 59 en `31_59`, de 60 en `60_plus` (E-13b). **Les totaux comptent des échéances ; la liste regroupe par bail** (comportement de la maquette, `maquette/NOTES.md`).
- **Solde** : somme des échéances en retard moins les affectations (R-057). Une échéance partielle dépassée est `late` et figure au tableau avec le reste dû (R-033, E-25).
- **Statut et ancienneté** : calculés par la fonction unique du crate `domain` (DP-51), au jour de l'organisation ; jamais stockés.
- **Filtres** : `property_id`, `owner_id`, `manager_id`, `usage` (`residential`, `commercial`, `mixed`), `band`. L'agent terrain ne reçoit que ses lots (R-081). Un caissier et un comptable ont la lecture seule.
- **Statut de la promesse** : `in_progress`, `kept`, `missed`, **calculé** à la lecture ; seule la clôture manuelle est écrite (DB §11.1 n° 7 : définition de « tenue » à fixer avant T1).

**Brouillon de relance.**

```
POST /api/reminders/draft
```

```json
{ "lease_id": "…0102", "installment_ids": ["…0411", "…0412", "…0413"], "channel": "whatsapp", "template_id": null }
```

```json
{
  "template": { "id": "…0c04", "step_offset": 10, "tone": "final_notice" },
  "suggested_step_offset": 10,
  "text": "Bonjour …",
  "recipient": { "display_name": "Pauline Yaméogo", "phone_last2": "07" },
  "whatsapp_url": "https://wa.me/226…?text=…",
  "warnings": [ { "code": "reminder.already_sent_today", "params": { "at": "08:40", "by_name": "Salif Ilboudo" } } ]
}
```

- Les étapes sont J−3, J+1, J+5, J+10 (`step_offset` : −3, 1, 5, 10, R-060) ; les tons `reminder`, `courteous`, `firm`, `final_notice`. L'étape suggérée suit l'ancienneté (J+1 sous 5 jours, J+5 de 5 à 9, J+10 dès 10 : proposition de la maquette, modifiable par l'utilisateur).
- Le serveur rend le texte avec les variables du modèle et **refuse une variable inconnue ou manquante** (R-064). Le jeu de variables est à fixer (quatre dans `UI_UX_DESIGN.md`, six dans la maquette : Questions ouvertes).
- L'avertissement `reminder.already_sent_today` (R-063) est **non bloquant**.
- Aucun envoi par le serveur (règle d'or 11, DP-33). Le front ouvre `whatsapp_url` ; l'utilisateur confirme ensuite.

**Confirmation de l'envoi, résultat, promesse.**

```
POST /api/reminders
Idempotency-Key: …   (recommandée)
```

```json
{
  "lease_id": "…0102", "installment_ids": ["…0411", "…0412", "…0413"],
  "channel": "whatsapp", "step_offset": 10, "tone": "final_notice", "template_id": "…0c04",
  "body": "Bonjour …",
  "outcome": "promise",
  "promise": { "promised_on": "2026-11-20", "amount": 35000 },
  "next_action_on": "2026-11-20", "next_action_note": "Rappeler si pas de paiement"
}
```

```json
{
  "reminder": { "id": "…0d01", "channel": "whatsapp", "outcome": "promise", "occurred_at": "2026-11-13T09:31:02Z", "created_by": "…0702" },
  "promise": { "id": "…0e01", "status": "in_progress", "promised_on": "2026-11-20", "amount": 35000 },
  "next_action": { "on": "2026-11-20", "note": "Rappeler si pas de paiement" }
}
```

- `channel` : `whatsapp`, `sms`, `call`. `outcome` : `message_sent`, `promise`, `no_answer`, `call_completed`. `body` est obligatoire pour `whatsapp` et `sms` (le texte réellement envoyé, `reminders.body`) ; absent pour `call`.
- Une relance est en **ajout seul** (R-062) : aucune route de modification.
- **Rappels à J−3** (R-097, DP-40) : `GET /reminders/due?step_offset=-3` ; même brouillon, même confirmation. Aucun envoi automatique.

**Variantes et erreurs.**

| Cas | Résultat |
|---|---|
| Locataire sans numéro | `422 reminder.recipient_has_no_phone` ; le front propose `PATCH /tenants/{id}` (avec `If-Match`) |
| Seconde relance le même jour | brouillon avec `reminder.already_sent_today` ; `POST` accepté (R-063) |
| Variable inconnue | `422 template.unknown_variable` sur `PUT /message-templates/{id}` avec `params.variable`, `params.allowed` |
| Lot hors de la portée de l'agent terrain | `404 resource.not_found` |
| Rôle sans `reminder.write` (caissier, comptable) | `403 auth.forbidden` |

### 6.3 Parcours 3 : produire un compte rendu propriétaire (gérante, plan Agence)

F13, F05, F15 (minimal) · R-068 à R-078, R-083 à R-085, DP-14 à DP-19 · droits `statement.read`, `statement.validate`, `payout.record` · **[PILOTE] T2**.

| # | Écran S12 | Appel | Remarque |
|---|---|---|---|
| 1 | Liste, « à produire » | `GET /statements?period=2026-10-01` | l'état « à produire » n'est pas un statut stocké : c'est l'absence de relevé pour une période échue (`ARCHITECTURE.md` §2.3) |
| 2 | Aperçu mandant M1, octobre | `GET /statements/preview?owner_id=…&period=2026-10-01` | calculé, **non stocké** (décision n° 44) |
| 3 | « Ajouter une dépense » | `POST /units/{id}/interventions` puis nouvel aperçu | pas de route d'ajustement dédiée |
| 4 | « Valider » (modale) | `POST /statements` 🔐 + `Idempotency-Key` | écrit la version 1 ; instantané complet |
| 5 | « Exporter en PDF », « Partager » | `GET /statements/{id}/versions/1/pdf`, `POST /statements/{id}/shares` | page publique S26 |
| 6 | « Enregistrer le reversement » (modale) | `POST /statements/{id}/payout` 🔐 + clé | action définitive (DP-17) |
| — | « Rouvrir » (avant reversement) | `POST /statements/{id}/reopen` 🔐, puis `POST /statements/{id}/versions` 🔐 | motif obligatoire (R-083) |

**Aperçu (E-18).**

```
GET /api/statements/preview?owner_id=…0b01&period=2026-10-01
```

```json
{
  "owner": { "id": "…0b01", "display_name": "Georgette Ouédraogo" },
  "period": "2026-10-01",
  "mandate": { "id": "…0f01", "rate_bp": 800, "commission_base": "rent" },
  "collected": {
    "amount": 400000,
    "items": [
      { "payment_id": "…0610", "paid_on": "2026-10-07", "unit_label": "Appartement A3", "tenant_name": "Habiba Kiemdé", "kind": "rent", "amount": 90000 },
      { "payment_id": "…0611", "paid_on": "2026-10-09", "unit_label": "Villa B1", "tenant_name": "Ousmane Belem", "kind": "rent", "amount": 85000 }
    ]
  },
  "commission": { "rate_bp": 800, "base": "rent", "base_amount": 400000, "amount": 32000 },
  "expenses": {
    "amount": 30000,
    "items": [ { "intervention_id": "…0e10", "occurred_on": "2026-10-22", "label": "Réparation d'une fuite, salle d'eau",
                 "unit_label": "Appartement A3", "amount": 30000, "payer": "agency", "imputable": true, "document_count": 2 } ],
    "information_only": []
  },
  "previous_adjustment": 0,
  "net_amount": 338000,
  "arrears": [ { "tenant_name": "Rasmané Bonkoungou", "unit_label": "Maison G2", "amount": 45000, "days_late": 18 } ],
  "blocking_issues": [],
  "warnings": [],
  "preview_hash": "sha256:…"
}
```

*(La liste `collected.items` compte six lignes dans le cas de référence : 90 000 + 85 000 + 75 000 + 60 000 + 50 000 + 40 000 = 400 000 ; deux sont montrées.)*

- **Règles d'arithmétique** : commission = `(base × rate_bp + 5 000) / 10 000` en division entière (R-068, DP-14, E-26) ; net = encaissé − commission − dépenses imputées ± ajustement de la période précédente (R-070). Les montants viennent du crate `domain`. Piège SQL de `DATABASE_SCHEMA.md` §4.6 (`sum(bigint)` renvoie un `numeric`) : conversion en `bigint` avant toute division, testée.
- **Base** : loyers encaissés dans la période, charges exclues par défaut (DP-16) ; arriérés et avances comptent à leur date d'encaissement. Un paiement n'entre dans la période qu'à sa date d'encaissement (R-071, **à confirmer sur le terrain**, décision n° 10) et dans **un seul** relevé (R-072). Le dépôt de garantie n'y entre pas (R-085).
- **Dépenses** : les dépenses payées par l'agence sont imputables par défaut ; celles du propriétaire ou du locataire figurent dans `information_only` (R-084, E-29). Quelles dépenses sont retenues (date de la dépense, statut `done` seulement) : question ouverte (DB §11.1 n° 4).
- `blocking_issues` : `statement.units_without_mandate` avec `params.units: [{ id, label }]` (R-075). `warnings` : `expense.no_document` (R-078, non bloquant).
- **Net négatif ou base négative** : aucune règle (DB §11.1 n° 3). En attendant, `422 statement.negative_collected`.

**Validation (écrit la version 1).**

```
POST /api/statements
Idempotency-Key: …
```

```json
{ "owner_id": "…0b01", "period": "2026-10-01", "expected_net_amount": 338000, "preview_hash": "sha256:…" }
```

```json
{
  "statement": { "id": "…1001", "owner_id": "…0b01", "period": "2026-10-01", "status": "validated", "current_version": 1, "can_reopen": true },
  "version": {
    "version": 1, "rate_bp": 800, "commission_base": "rent",
    "collected_amount": 400000, "commission_base_amount": 400000, "commission_amount": 32000,
    "expenses_amount": 30000, "previous_adjustment": 0, "net_amount": 338000,
    "validated_by": "…0701", "validated_at": "2026-11-13T10:02:11Z",
    "pdf": { "state": "pending" }
  }
}
```

- **Le serveur recalcule l'instantané à la validation** ; il ne reprend jamais les chiffres du client. `expected_net_amount` et `preview_hash` servent de **garde-fou** : si un paiement ou une dépense est apparu entre l'aperçu et la validation, `409 statement.preview_stale` renvoie le nouvel aperçu (garde proposée, non issue du domaine).
- Un relevé validé est **figé** ; la base refuse une version dont la commission ou le net sont faux (`DATABASE_SCHEMA.md` §4.6). La validation exige la confirmation du mot de passe (🔐).
- Le statut est **dérivé** : `validated` (au moins une version, pas de reversement), `paid` (reversement enregistré). Un « brouillon » n'est stocké nulle part (décision n° 44).
- L'export PDF est identique à la dernière version validée (R-074) : c'est le rendu de l'instantané.

**Partage, reversement, réouverture.**

| Appel | Corps → réponse | Erreurs |
|---|---|---|
| `POST /statements/{id}/shares` | `{ version, ttl_days }` → `{ share_link: { url, expires_at } }` ; le mandant ne se connecte pas (CONTEXTE) | `statement.not_validated` |
| `POST /statements/{id}/payout` 🔐 | `{ paid_on, method, reference, amount }` → `{ payout, statement: { status: "paid", can_reopen: false } }` | `payout.amount_mismatch`, `statement.not_validated`, `idempotency.*` |
| `POST /statements/{id}/reopen` 🔐 | `{ reason }` → `{ statement: { status: "draft", can_reopen: false } }` | `statement.reopen_forbidden` (reversement enregistré, R-083, E-30), `validation.failed` (motif absent) |
| `POST /statements/{id}/versions` 🔐 | `{ expected_net_amount, preview_hash }` → version suivante | `statement.preview_stale`, `statement.reopen_required` |

- **Un seul reversement** par compte rendu, du montant exact du net (`UNIQUE (org_id, statement_id)`, DB §11.1 n° 8) : un reversement partiel ou fractionné n'est pas possible au pilote.
- La réouverture crée une nouvelle version ; les précédentes sont conservées (R-083, DP-17).
- Rôles : validation, réouverture, reversement : **gérant seul**. Consultation : gérant, gestionnaire, comptable. Plan Propriétaire : `403 plan.feature_unavailable` (R-082).

### 6.4 Parcours 4 : importer un fichier Excel (gestionnaire)

F14 · R-080 · E-20, E-21 · droit `import.run` (G, P, Ge réglable) · **[PILOTE] T2** (HABEK peut importer avant par la console, `ARCHITECTURE.md` §9.2).

| # | Écran S16 | Appel | Remarque |
|---|---|---|---|
| 0 | « Télécharger le modèle Excel » | `GET /import-template` | fichier XLSX statique |
| 1 | Téléverser | `POST /import-batches` (multipart) | `202` + opération ; analyse du fichier en tâche de fond |
| 2 | Association des colonnes | `GET /import-batches/{id}` puis `PUT /import-batches/{id}/mapping` | propositions automatiques avec niveau de confiance |
| 3 | Aperçu | `POST /import-batches/{id}/preview` | `202` + opération ; 2 000 lignes en 5 s au plus |
| 4 | Erreurs et doublons | `GET /import-batches/{id}/rows?status=error` | numéros de ligne |
| 5 | « Continuer sans ces lignes », fusion | `PUT /import-batches/{id}/resolutions` | |
| 6 | Confirmer (modale) | `POST /import-batches/{id}/confirm` | `202` + opération ; **première** création de données |
| 7 | Rapport | `GET /import-batches/{id}/report` | CSV |
| 8 | Historique, annulation | `GET /import-batches`, `POST /import-batches/{id}/cancel` | R-080 |

**Principes.**

- **Rien n'est créé avant la confirmation** (R-080) : l'analyse n'écrit que dans `import_rows` ; la confirmation crée tout dans **une seule transaction**, et chaque entité porte `import_batch_id`.
- **États du lot** (`import_batches.status`) : `uploaded` → `mapped` → `previewed` → `confirmed`, plus `cancelled` et `failed`. Une étape appelée dans un autre état : `409 import.wrong_state`.
- **Limites** : 5 Mo, 5 000 lignes, CSV et XLSX seulement, aucune formule évaluée (section 5.6, `SECURITY.md` §7.4). L'analyse s'exécute avec mémoire et temps bornés.

**Téléversement.**

```
POST /api/import-batches
Content-Type: multipart/form-data   (champ « file »)
```

```
202 Accepted
Location: /api/operations/…2001
```

```json
{
  "batch": { "id": "…1101", "file_name": "portefeuille-agence.xlsx", "file_size": 184320, "status": "uploaded", "cancellable": false },
  "operation": { "id": "…2001", "kind": "import_analyze", "state": "queued" }
}
```

**Correspondance des colonnes.**

```json
{
  "batch": { "id": "…1101", "status": "mapped" },
  "columns": [
    { "source": "Nom du locataire", "target": "tenant.display_name", "confidence": "high" },
    { "source": "Tél", "target": "tenant.phone", "confidence": "high" },
    { "source": "Loyer mensuel", "target": "lease.rent", "confidence": "medium" }
  ],
  "unmapped_sources": ["Remarques"],
  "missing_required": []
}
```

`PUT …/mapping` avec `{ "columns": [{ "source", "target" }], "save_as": "Modèle de l'agence" }` mémorise la correspondance par organisation (`import_mappings`) pour le fichier suivant.

**Aperçu et lignes.** Cas de référence : 200 lignes, 3 erreurs aux lignes 14, 87 et 152, 2 paires de doublons de téléphone (E-20).

```json
{
  "batch": { "id": "…1101", "status": "previewed",
             "total_rows": 200, "valid_rows": 195, "warning_rows": 2, "error_rows": 3, "creatable_rows": 197 }
}
```

```
GET /api/import-batches/…1101/rows?status=error
```

```json
{
  "items": [
    { "row_number": 14,  "status": "error", "error_field": "tenant.phone", "error_code": "import.phone_invalid", "params": { "value": "70 12 AB 56" } },
    { "row_number": 87,  "status": "error", "error_field": "lease.start_date", "error_code": "import.date_invalid", "params": { "value": "31/02/2026" } },
    { "row_number": 152, "status": "error", "error_field": "lease.rent", "error_code": "import.amount_not_integer", "params": { "value": "75000,5" } }
  ],
  "next_cursor": null
}
```

- Les statuts de ligne sont ceux de `import_rows.status` : `valid`, `warning`, `error`, `skipped`, `merged`. Un doublon est une ligne `warning` avec `error_code: "import.duplicate_phone"` et `duplicate_of_row`.
- **« 197 lignes valides, 3 erreurs »** (microcopy, E-20) : le contrat lit « valides » comme **créables** = `valid_rows + warning_rows` (195 + 2). Les compteurs de la base vérifient `total = valid + warning + error` ; la lecture de E-20 est une interprétation à confirmer (Questions ouvertes).
- Le contenu brut d'une ligne (`raw`) contient des données personnelles : il n'est renvoyé qu'à l'utilisateur qui a téléversé le fichier, jamais journalisé, et purgé après la confirmation (`SECURITY.md` §6.3).
- Codes de ligne (propositions) : `import.required_missing`, `import.phone_invalid`, `import.date_invalid`, `import.date_order`, `import.amount_not_integer`, `import.amount_out_of_range`, `import.rent_not_positive`, `import.unit_ref_missing`, `import.unit_duplicate_in_file`, `import.lease_overlap`, `import.enum_unknown`, `import.duplicate_phone` (avertissement), `import.duplicate_unit` (avertissement).

**Résolutions, confirmation, rapport.**

```
PUT /api/import-batches/…1101/resolutions
```

```json
{ "skip_error_rows": true,
  "duplicates": [ { "row_number": 88, "action": "merge" }, { "row_number": 131, "action": "create_new" } ] }
```

`action` : `merge` (fusion avec la fiche existante ou la ligne précédente), `create_new`, `skip` (R-017, R-080).

```
POST /api/import-batches/…1101/confirm      → 202 + opération
GET  /api/operations/…2002                  → state: "done"
```

```json
{
  "id": "…2002", "kind": "import_confirm", "state": "done", "progress": { "done": 197, "total": 197 },
  "result": { "created": { "owners": 0, "properties": 22, "units": 40, "tenants": 31, "leases": 33, "installments": 66 },
              "merged": { "tenants": 2 }, "skipped_rows": 3 },
  "error": null
}
```

- `GET …/report` : `text/csv`, UTF-8 avec marque d'ordre des octets, séparateur « ; » (Excel en français) ; une ligne par ligne du fichier, avec statut et code ; **cellules commençant par `=`, `+`, `-`, `@` préfixées d'une apostrophe** (`SECURITY.md` §7.4, DB §8.5).
- **Annulation** : `POST …/cancel` répond `200` si aucun paiement, relance ni promesse ne référence un bail du lot ; sinon `409 import.not_cancellable` (R-080, E-21). Le champ `cancellable` de la ressource est **calculé** à la lecture.
- **Import assisté par HABEK** : mêmes routes depuis la console, avec un accord support de portée `import_assist` ; `assisted_by_staff` renseigné (F40, R-009).

**Erreurs.**

| Code | HTTP | Cause |
|---|---|---|
| `import.format_not_supported` | 415 | extension ou contenu refusé (`.xls`, `.xlsm`, archive suspecte) ; `params.expected_columns` liste les colonnes attendues |
| `upload.too_large` | 413 | plus de 5 Mo |
| `import.too_many_rows` | 422 | plus de 5 000 lignes |
| `import.wrong_state` | 409 | étape appelée hors séquence |
| `import.not_cancellable` | 409 | lot déjà utilisé |
| `auth.forbidden` | 403 | gestionnaire sans le droit réglable `import_data` |

### 6.5 Parcours 5 : consulter l'historique d'un lot (gestionnaire)

F04, F07, F15, F16 · R-005, R-006, R-078, R-084, R-093 · droits `catalog.read`, `expense.write` · **aperçu, baux, paiements : [PILOTE] T1 ; dépense imputable : [PILOTE] T2 ; journal des travaux complet et documents : [APRÈS] T3** (DP-29).

| # | Écran | Appel | Remarque |
|---|---|---|---|
| 1 | S06, filtre « Ouaga 2000 » | `GET /units?neighborhood=Ouaga%202000` | le filtre `neighborhood` est le quartier du bien (proposition) |
| 2 | S07, onglet Aperçu | `GET /units/{id}` | bail en cours, locataire, échéance du mois, solde |
| 3 | Onglet Baux | `GET /units/{id}/leases` | historique complet, y compris les baux terminés |
| 4 | Onglet Paiements | `GET /payments?unit_id={id}` | filtre du journal des paiements |
| 5 | Onglet Travaux | `GET /units/{id}/interventions` | chronologie ; T2 minimal, T3 complet |
| 6 | Détail d'une intervention | `GET /interventions/{id}` | prestataire, coût, payeur, pièces |
| 7 | Onglet Documents | `GET /units/{id}/documents` | [APRÈS] T3 |
| 8 | « Voir dans le compte rendu d'octobre » | `GET /statements/{id}` | via `statement_link` |
| 9 | Variante : ajouter une intervention | `POST /units/{id}/interventions` | |

**Aperçu d'un lot.**

```json
{
  "id": "…0210", "reference": "A3", "label": "Appartement A3", "kind": "apartment", "usage": "residential",
  "status": "occupied",
  "property": { "id": "…0a10", "name": "…", "neighborhood": "Ouaga 2000" },
  "owner": { "id": "…0b01", "display_name": "Georgette Ouédraogo" },
  "current_lease": { "id": "…0110", "status": "active", "tenant_names": ["Habiba Kiemdé"], "rent": 90000, "charges": 0 },
  "current_installment": { "period": "2026-11-01", "status": "paid", "remaining_amount": 0 },
  "version": 3
}
```

- `status` du lot est **dérivé** du bail en cours (`occupied`, `vacant`) ; seuls `under_works` et `archived` sont posés à la main (DB §10.1). `owner` n'est visible que pour G, Ge et Co.
- L'agent terrain n'accède qu'aux lots qui lui sont assignés ; les autres répondent `404`.

**Chronologie des travaux.**

```
GET /api/units/…0210/interventions?sort=-occurred_on&limit=25
```

```json
{
  "items": [
    { "id": "…0e10", "occurred_on": "2026-10-22", "label": "Réparation d'une fuite, salle d'eau",
      "cost": 30000, "payer": "agency", "status": "done", "before_adoption": false,
      "provider": { "id": "…0d10", "display_name": "Plombier …", "trade": "plomberie" },
      "imputable": true,
      "statement_link": { "statement_id": "…1001", "period": "2026-10-01", "version": 1 },
      "document_count": 2 }
  ],
  "next_cursor": null
}
```

- `imputable` est **dérivé** (dernier changement d'imputation, sinon défaut selon le payeur : agence → imputable, propriétaire ou locataire → non, R-084, DP-18). `before_adoption` marque une intervention antérieure à l'usage d'Immo Suite (`horsHistorique` de la maquette).
- `statement_link` est présent quand la dépense est imputée à un compte rendu validé ; le coût est alors **figé** (R-074) : `PATCH` répond `409 expense.frozen`.

**Ajout d'une intervention (variante).**

```
POST /api/units/…0210/interventions
```

```json
{ "occurred_on": "2026-11-13", "label": "Remplacement d'un robinet", "description": null,
  "cost": 12000, "provider_id": null, "payer": "agency", "status": "done", "before_adoption": false }
```

`201` avec l'intervention et `warnings: [{ "code": "expense.no_document" }]` si aucune pièce n'est jointe (non bloquant, R-078). Rôles : G, Ge, P ; l'agent terrain « propose » (mécanisme non défini : section 4.6, note 3).

## 7. Catalogue des points d'accès par module

### 7.1 Lecture du catalogue

- Toutes les routes sont préfixées par `/api/` (DP-56), sauf `/health`, `/ready`, `/s/…`, `/v/…` et `/console/…`. Pour la lisibilité, **les tables de cette section omettent le préfixe** : `POST /auth/login` s'appelle `POST /api/auth/login`. Les exemples complets des cinq parcours sont en section 6 ; pour les autres points d'accès, la colonne « Corps → réponse » donne la **forme** (champs clés) ; l'exemple complet est un fichier `tests/contract/examples/` validé contre le schéma (section 8).
- **Priorité** : **P/T1**, **P/T2** = [PILOTE] (tranche du dossier) ; **A/T3** = [APRÈS].
- **Erreurs communes à toutes les routes**, non répétées : `auth.unauthenticated` (401), `auth.csrf_invalid` et `auth.forbidden` (403), `resource.not_found` (404, dont ressource d'une autre organisation), `validation.failed` (422), `request.*`, `rate_limit.exceeded` (429), `server.unexpected` (500), `server.maintenance` (503). Les routes à clé d'idempotence ajoutent `idempotency.*` ; les routes `PATCH` ajoutent `concurrency.*`.
- **Droit** : code de la table de 4.6. 🔐 = confirmation du mot de passe (5 minutes).
- **Schémas communs** (champs clés) :

| Schéma | Champs |
|---|---|
| `Installment` | `id`, `lease_id`, `period`, `due_date`, `status` (`upcoming`, `due`, `partial`, `paid`, `late`), `due_amount`, `allocated_amount`, `remaining_amount`, `days_late` (entier ou `null`), `band` (`1_7`, `8_30`, `31_59`, `60_plus`, ou `null`), `lines[]` (`id`, `kind` : `rent`, `charges`, `arrears`, `exceptional` ; `label`, `due_amount`, `allocated_amount`, `voided`) |
| `Payment` | `id`, `lease_id`, `kind` (`payment`, `reversal`), `status` (`validated`, `reversed`), `amount`, `method`, `external_ref`, `paid_on`, `recorded_at`, `cash_day`, `received_by`, `payer`, `note`, `allocations[]`, `unallocated_amount`, `receipt` (`id`, `number`, `status`), `reversal` (`payment_id`, `reason`, `at`) ou `null`, `replaced_by` ou `null`, `correction_request` (`id`, `status`) ou `null` |
| `Receipt` | `id`, `payment_id`, `number`, `series`, `kind` (`standard`, `corrective`), `status` (`issued`, `cancelled`), `replaces_receipt_id`, `cancellation` (`cause`, `at`, `replaced_by_number`) ou `null`, `issued_on`, `issued_at`, `amount`, `amount_in_words`, `remaining_after`, `advance_amount`, `snapshot`, `pdf` (`state`, `papers`, `sha256`) |
| `Lease` | `id`, `unit_id`, `kind`, `status`, `start_date`, `end_date`, `rent`, `charges`, `due_day`, `due_day_defaulted`, `deposit_amount`, `vat_included`, `tenants[]`, `commercial_terms`, `notice`, `current_terms`, `credit_amount`, `warnings[]`, `version` |
| `Tenant` | `id`, `kind` (`person`, `company`), `display_name`, `phones[]` (`number`, `whatsapp`), `email`, `trade_activity`, `balance`, `archived`, `version` ; selon le rôle : `secondary_contact`, `notes`, `id_document_masked` |
| `Operation` | `id`, `kind`, `state` (`queued`, `running`, `done`, `failed`), `progress`, `result`, `error` (5.4) |

### 7.2 Authentification, session, comptes

| Méthode et chemin | Droit | Corps → réponse | Erreurs propres | Règles · F · Prio |
|---|---|---|---|---|
| `POST /auth/login` | public | `{ identifier, password }` → `{ user, organizations[], csrf_token, mfa_required }` ; cookie posé, nouvel identifiant de session | `auth.invalid_credentials`, `auth.rate_limited` | SECURITY §3 · F01 · P/T1 |
| `POST /auth/totp/verify` | session partielle | `{ code }` → comme `login` | `auth.totp_invalid` (5 essais par minute, code déjà utilisé refusé) | SECURITY §3.4 · F01 · P/T1 |
| `POST /auth/logout`, `POST /auth/logout-all` | session | → `204` ; `logout-all` révoque toutes les sessions (`session.revoked_all`) | | F01 · P/T1 |
| `POST /auth/reauth` | session | `{ password }` → `204` ; valable 5 minutes | `auth.invalid_credentials` | SECURITY §3.4 · P/T1 |
| `GET /me` | session | → `{ user, membership: { role }, organization: { name, plan, kind, lifecycle, timezone, currency }, features, permissions[], today, server_time, csrf_token, min_client_version, settings: { payment_confirm_threshold, default_paper, share_link_ttl_days } }` | | F01, F02 · P/T1 |
| `POST /me/organization` | session | `{ organization_id }` → comme `GET /me` ; revérifie l'adhésion | `resource.not_found` | F01 · P/T1 |
| `PATCH /me` | session | `{ display_name, login_phone? 🔐 }` → utilisateur | `auth.identifier_taken` | F01 · P/T1 |
| `PUT /me/password` | session | `{ current_password, new_password }` → `204` ; **révoque les autres sessions** | `auth.password_weak` (✱ au moins 10 caractères, mot de passe courant ou contenant l'identifiant refusé, 128 au plus) | SECURITY §3.1 · P/T1 |
| `POST /me/totp`, `POST /me/totp/confirm` | session | création du secret (affiché **une fois**) puis `{ code }` → `{ backup_codes[10] }` (affichés une fois) | `auth.totp_invalid` | SECURITY §3.4 · P/T1 (obligatoire avant `live`, DP-68) |
| `POST /users/{id}/password-resets` | `user.manage` 🔐 | → `{ url, expires_at }` ; lien à usage unique de **15 minutes**, jeton affiché une fois | | SECURITY §3.5 · F01 · P/T1 |
| `POST /auth/password-reset/complete` | public (jeton) | `{ token, new_password }` → `204` ; révoque toutes les sessions de l'utilisateur | `auth.reset_expired` (410), `auth.password_weak` | SECURITY §3.5 · P/T1 |
| `GET /invitations/{token}` | public | → `{ organization_name, role, expires_at, identifier_exists }` | `invitation.expired` (410) | F01 · P/T1 |
| `POST /invitations/accept` | public (jeton) | `{ token, display_name, password? }` → connexion ; l'invité fixe son mot de passe ; un compte existant se connecte d'abord | `invitation.expired`, `auth.identifier_taken` | F01 · P/T1 |

Réponses de connexion et de refus **identiques** que l'identifiant existe ou non (SECURITY §3.3). `POST /auth/register` est absent au pilote (4.1).

### 7.3 Organisations et utilisateurs

| Méthode et chemin | Droit | Corps → réponse | Erreurs propres | Règles · F · Prio |
|---|---|---|---|---|
| `GET /organization` | `settings.read` | → nom, ville, plan, type, cycle de vie, `version` | | F01 · P/T1 |
| `PATCH /organization` | `settings.write` | `{ name, city }` → organisation | `concurrency.*` | F01 · P/T1 |
| `GET /users` | `user.manage` | → utilisateurs (`membership_id`, nom, identifiant, rôle, statut) ; les invitations ont leur liste | | F02 · P/T1 |
| `POST /invitations` | `user.manage` (🔐 si rôle gérant) | `{ phone ou email, role, unit_ids[] }` → `{ invitation, url, expires_at }` ; lien de **7 jours**, jeton affiché une fois, copiable dans WhatsApp | `auth.identifier_taken`, `validation.phone_format` | F01 · P/T1 |
| `GET /invitations` | `user.manage` | → invitations (`pending`, `expired`, `accepted`, `revoked`) | | F01 · P/T1 |
| `POST /invitations/{id}/revoke`, `POST /invitations/{id}/renew` | `user.manage` | → invitation ; `renew` crée un nouveau jeton | | F01 · P/T1 |
| `PATCH /users/{membership_id}` | `user.manage` 🔐 | `{ role }` → utilisateur ; effet **sans reconnexion**, sessions révoquées (le rôle est relu à chaque requête) | `user.last_manager` | R-081 · F02 · P/T1 |
| `POST /users/{membership_id}/disable` 🔐, `POST …/enable` | `user.manage` | → utilisateur ; toutes ses sessions révoquées | `user.last_manager` | R-081 · F02 · P/T1 |
| `PUT /users/{membership_id}/unit-assignments` | `user.manage` | `{ unit_ids[] }` → affectations (agent terrain, gestionnaire) | | R-081 · F02 · P/T1 |
| `GET /role-settings`, `PUT /role-settings` | `user.manage` | `{ role, action, allowed }` pour `record_payment`, `import_data`, `export_data` (gestionnaire, collaborateur) → réglage ; audité | | F02 · P/T1 |
| `GET /support-grants` | `support.grant` | → accords et historique des accès (qui, quand, combien de temps) | | R-009 · F40 · P/T2 |
| `POST /support-grants` 🔐 | `support.grant` | `{ scope: "read" ou "import_assist", reason, duration_hours }` → accord ; **7 jours au plus**, 24 h par défaut, motif de 5 caractères au moins | `validation.failed` | R-009 · F40 · P/T2 |
| `POST /support-grants/{id}/revoke` 🔐 | `support.grant` | → accord révoqué | | R-009 · F40 · P/T2 |

### 7.4 Biens et lots

| Méthode et chemin | Droit | Corps → réponse | Erreurs propres | Règles · F · Prio |
|---|---|---|---|---|
| `GET /properties` | `catalog.read` | filtres `q`, `neighborhood`, `owner_id`, `archived` → liste de résumés (nom, quartier, propriétaire, nombre de lots, occupation) | | F04 · P/T1 |
| `POST /properties` | `catalog.write` | `{ name, address, neighborhood, city, default_owner_id, single_unit? }` → bien ; `single_unit` crée **1 bien + 1 lot** (maison seule) | | R-010 · F04 · P/T1 |
| `GET /properties/{id}`, `PATCH /properties/{id}` | `catalog.read`, `catalog.write` | fiche, `version` | `concurrency.*` | F04 · P/T1 |
| `POST /properties/{id}/archive` | `catalog.write` | → bien archivé ; l'historique reste visible | `unit.has_active_lease` si un lot a un bail en cours | R-005, R-006 · F04 · P/T1 |
| `GET /units` | `catalog.read` | filtres `property_id`, `neighborhood`, `usage`, `kind`, `status`, `q` → liste (référence, bien, statut dérivé, locataire, loyer, échéance du mois) | | F04 · P/T1 |
| `POST /units` | `catalog.write` | `{ property_id, reference, kind, usage, surface, reference_rent, allowed_activity? }` → lot. Les champs dépendent du type : `allowed_activity` pour un lot commercial (R-012, DP-41) | `unit.reference_duplicate` | R-011, R-012 · F04 · P/T1 |
| `GET /units/{id}`, `PATCH /units/{id}` | `catalog.read`, `catalog.write` | aperçu (6.5), `version` | `concurrency.*` | F04 · P/T1 |
| `POST /units/{id}/works` | `catalog.write` | `{ under_works: true ou false }` → lot ; audit `unit.works_set` | `unit.works_requires_vacant` | R-093 · F04 · P/T1 |
| `POST /units/{id}/archive` | `catalog.write` | → lot archivé | `unit.has_active_lease` | R-006 · F04 · P/T1 |
| `GET /units/{id}/leases`, `GET /units/{id}/interventions`, `GET /units/{id}/documents` | `catalog.read` | onglets de la fiche lot (6.5) | | F04, F07, F15, F16 · P/T1, T2 ; documents A/T3 |

Aucune suppression physique (R-005). La liste d'un agent terrain est filtrée sur ses lots.

### 7.5 Locataires et baux

| Méthode et chemin | Droit | Corps → réponse | Erreurs propres | Règles · F · Prio |
|---|---|---|---|---|
| `GET /tenants` | `catalog.read` | filtres `q`, `has_arrears`, `archived` → liste (nom, téléphone, lots, solde) | | F06 · P/T1 |
| `POST /tenants` | `catalog.write` | `{ kind, display_name, phones[], email, trade_activity, secondary_contact }` → locataire | `tenant.phone_duplicate` (avec `conflict`), `validation.phone_format` | R-017, R-018 · F06 · P/T1 |
| `GET /tenants/{id}`, `PATCH /tenants/{id}` | `catalog.read`, `catalog.write` | fiche, `version` ; champs masqués selon le rôle (4.5) | `concurrency.*`, `tenant.phone_duplicate` | F06 · P/T1 |
| `GET /tenants/{id}/leases`, `…/payments`, `…/reminders` | `catalog.read` (+ `payment.read`, `arrears.read`) | onglets de S09 | | F06 · P/T1 |
| `POST /tenants/{id}/merge` | `catalog.write` | `{ into_id }` → fiche conservée ; l'historique reste sur l'ancien identifiant (`merged_into_id`) ; audit `tenant.merged` | | R-017 · F06 · P/T1 |
| `POST /tenants/{id}/archive` | `catalog.write` | → locataire archivé | | R-005 · F06 · P/T1 |
| `GET /tenants/{id}/id-document` | `catalog.write` | → pièce d'identité **révélée**, action tracée (`tenant.id_document_viewed`) ; chiffrée en base | `feature.disabled` (champ désactivé au pilote, `ARCHITECTURE.md` §13.2) | R-019 · F06 · A |
| `GET /leases` | `catalog.read` | filtres `status`, `unit_id`, `tenant_id`, `ending_before` → liste (locataire, lot, statut, loyer, alertes) | | F07 · P/T1 |
| `POST /leases` | `catalog.write` | `{ unit_id, tenant_ids[], kind, start_date, end_date?, rent, charges, due_day, deposit_amount, vat_included, commercial_terms? }` → bail **brouillon** + `warnings[]` | `validation.failed` (`validation.rent_positive`, `validation.date_order`) ; `lease.deposit_above_two_months` en **avertissement** (habitation, R-024) | R-021, R-022, R-024, R-092 · F07 · P/T1 |
| `GET /leases/{id}`, `PATCH /leases/{id}` | `catalog.read`, `catalog.write` | fiche, `version` ; modification libre **seulement en brouillon** | `lease.not_draft`, `concurrency.*` | R-023 · F07 · P/T1 |
| `POST /leases/{id}/activate` | `catalog.write` | → bail `active` ; échéances générées, lot « occupé » (R-028, R-013) ; clé d'idempotence recommandée | `lease.overlap` (avec dates), `unit.under_works`, `lease.not_draft` | R-020, R-093 · F07, F08 · P/T1 |
| `POST /leases/{id}/amendments` | `catalog.write` | `{ effective_from, rent?, charges?, new_end_date? }` → bail ; **jamais rétroactif** ; une prolongation de la date de fin est un avenant (E-35) | `lease.amendment_retroactive` | R-023, R-091 · F07 · P/T1 |
| `POST /leases/{id}/terminate` | `catalog.write` | `{ terminated_on, reason }` → bail `ended` ; lot « vacant » | | F07 · P/T1 |
| `POST /leases/{id}/notice`, `POST /leases/{id}/notice/cancel` | `catalog.write` | `{ notice_given_on, departure_date }` → bail `notice` (reste en cours) | | R-090 · F23 · A/T3 |
| `GET /leases/{id}/installments` | `catalog.read` | → `Installment[]` avec statut calculé, y compris les lignes annulées | | R-027 à R-035 · F08 · P/T1 |
| `POST /leases/{id}/installment-lines` | `catalog.write` | `{ kind: "arrears" ou "exceptional", label, amount, due_date }` → ligne | | R-035, R-036 · F08 · P/T1 |
| `POST /installment-lines/{id}/void` | `catalog.write` | `{ reason }` → ligne annulée ; refusée si elle a reçu un paiement net | `installment.line_has_payments` (409) | R-036 · F08 · P/T1 |
| `GET /leases/{id}/payment-context` | `payment.record` | voir 6.1 | | R-043, R-087 · F09 · P/T1 |
| `GET /leases/{id}/deposit-events`, `POST …` | `catalog.write` | réception, retenue (motif obligatoire), restitution → événement en ajout seul | | R-066, R-067 · F20 · A/T3 |

**La génération des échéances n'a pas de route** : c'est une tâche planifiée idempotente (R-029). Sa règle pour les jours 29 à 31, le premier mois partiel et les baux à date de début passée n'est pas écrite dans `DOMAINE.md` (Questions ouvertes d'`ARCHITECTURE.md`). Le contrat ne l'expose pas avant qu'elle le soit.

### 7.6 Paiements et quittances

| Méthode et chemin | Droit | Corps → réponse | Erreurs propres | Règles · F · Prio |
|---|---|---|---|---|
| `GET /payments` | `payment.read` | filtres `from`, `to`, `method`, `tenant_id`, `unit_id`, `lease_id`, `status`, `received_by`, `q` ; tri `-recorded_at` → liste de `Payment` résumés | | F09 · P/T1 |
| `GET /payments/{id}` | `payment.read` | `Payment` complet (affectations, historique, quittance, demande de correction) | | F09 · P/T1 |
| `POST /payments/preview` | `payment.record` | `{ lease_id, amount, allocations? }` → répartition calculée, `surplus_amount`, `requires_*_confirmation`, `installments_after` ; sans écriture | | R-043, R-044, R-046 · F09 · P/T1 |
| `POST /payments` | `payment.record` | voir 6.1 | voir 6.1 | R-037 à R-048, R-088 · F09, F10 · P/T1 |
| `PUT, PATCH, DELETE /payments/{id}` | — | **toujours refusés** | `payment.immutable` (405) | R-047 · E-08 · P/T1 |
| `POST /payments/{id}/correction-requests` | `payment.request_correction` | `{ reason }` → demande `pending` ; notifie le gérant ; **ne modifie rien** | `correction.already_pending`, `payment.already_reversed` | R-048 · F09 · P/T1 |
| `GET /correction-requests` | `payment.reverse` | filtre `status` (`pending`, `resolved`, `rejected`) → file du gérant | | R-048 · F09 · P/T1 |
| `POST /correction-requests/{id}/reject` | `payment.reverse` | `{ note }` → demande `rejected` | | R-048 · P/T1 |
| `POST /payments/{id}/reversal` | `payment.reverse` 🔐 | `{ reason, correction_request_id?, replacement? }` + `Idempotency-Key`. Crée **une** contre-passation (montant opposé, affectations négatives, quittance d'origine `cancelled`). `replacement` (corps d'un paiement) crée **dans la même transaction** le paiement de remplacement, dont la quittance est **rectificative** et référence l'ancienne (R-096, E-38). → `{ reversal, replacement?, receipt_cancellation, receipts[], installments[] }` | `payment.already_reversed`, `payment.reversal_reason_required`, `auth.reauth_required` | R-047, R-048, R-051, R-096 · F09, F10 · P/T1 |
| `GET /receipts` | `receipt.issue` ou lecture | filtres `q` (numéro), `from`, `to`, `status` → liste | | F10 · P/T1 |
| `GET /receipts/{id}` | idem | `Receipt` complet, envois | | F10 · P/T1 |
| `GET /receipts/{id}/pdf?paper=a4\|a5` | `receipt.issue` ou lecture | PDF (5.5) | `receipt.pdf_not_ready` (409) | R-049, R-056 · F10 · P/T1 |
| `POST /receipts/{id}/sends` | `receipt.issue` | voir 6.1 | `receipt.recipient_has_no_phone` | R-054, R-055 · F10 · P/T1 |
| `GET /receipts/{id}/sends` | idem | historique d'envois (canal, qui, quand) | | R-055 · F10 · P/T1 |
| `POST /share-links/{id}/revoke` | `receipt.issue` | → lien révoqué | | R-054 · F10 · P/T1 |
| `POST /receipts/{id}/corrective` | — | **Réservé, non spécifié** : quittance rectificative sans effet sur le montant (nom, période). Règle absente (Questions ouvertes) | | R-051 · F10 · à valider |
| `POST /payment-sms/parse` | `payment.record` | `{ text }` → `{ amount, external_ref, method, paid_on }` en **suggestions** que l'utilisateur confirme ; lecture par règles ; aucune écriture | `payment_sms.unrecognized` (422) | F25 · A/T3 |
| `GET /cash-days`, `POST /cash-days/{date}/close` | `payment.read`, `cash.close` | espèces attendues (calculées), comptées, écart → journée verrouillée ; clé d'idempotence obligatoire | | R-079, R-089 · F19 · A/T3 |

**Pages publiques** (hors `/api`, sans session, sans JavaScript applicatif, `noindex`, `Referrer-Policy: no-referrer`) :

| Chemin | Rôle | Réponses |
|---|---|---|
| `GET /s/{jeton}` | page S26 : quittance ou compte rendu partagé ; un seul document | `200` HTML ; `410` lien expiré ou révoqué (texte de S26) ; `404` jeton inconnu ; limitée en débit par adresse |
| `GET /s/{jeton}/pdf` | PDF du même document | idem |
| `GET /v/{numéro}?code=…` | vérification d'un code de quittance : confirme émetteur, montant, date, **statut** (émise ou annulée), rien d'autre ; **n'expire pas** | décision n° 55 · A/T3 |

La lecture publique par jeton s'appuie sur une politique par empreinte (DB §6.7, décision n° 39) : aucun rôle contournant la sécurité par ligne.

### 7.7 Impayés et relances

| Méthode et chemin | Droit | Corps → réponse | Erreurs propres | Règles · F · Prio |
|---|---|---|---|---|
| `GET /home` | selon le rôle | tableau d'accueil adapté (S03) : gérant et propriétaire (encaissé, attendu, taux de recouvrement **formule à confirmer**, impayés par ancienneté) ; gestionnaire (promesses du jour et non tenues, rappels à J−3, échéances dues) ; caissier (encaissements du jour par mode) ; agent terrain (sa tournée) ; comptable (lecture) | | F11, F12, F22 · P/T1 |
| `GET /arrears` | `arrears.read` | voir 6.2 | | R-057 à R-059 · F11 · P/T1 |
| `GET /reminders` | `arrears.read` | filtres `lease_id`, `tenant_id`, `from`, `to` → journal | | R-062 · F12 · P/T1 |
| `GET /reminders/due` | `reminder.write` | `step_offset=-3` → échéances à rappeler | | R-097, DP-40 · F12 · P/T1 |
| `POST /reminders/draft`, `POST /reminders` | `reminder.write` | voir 6.2 | `reminder.recipient_has_no_phone`, `template.variable_missing` | R-060 à R-065 · F12 · P/T1 |
| `PUT, PATCH, DELETE /reminders/{id}` | — | **toujours refusés** | `reminder.immutable` (405) | R-062 · P/T1 |
| `GET /promises` | `arrears.read` | filtre `status` (`in_progress`, `kept`, `missed`) → promesses | | R-065 · F12 · P/T1 |
| `POST /promises/{id}/close` | `reminder.write` | `{ reason }` → promesse clôturée à la main | | F12 · P/T1 |
| `GET /message-templates` | `template.read` | → modèles par étape (J−3, J+1, J+5, J+10), canal, texte, `version` | | R-060, R-064 · F12 · P/T1 |
| `PUT /message-templates/{id}` | `settings.write` | `{ body, … }` → modèle ; **variables inconnues refusées** | `template.unknown_variable` | R-064 · F12 · P/T1 |
| `POST /message-templates/{id}/preview` | `settings.write` | `{ lease_id? }` → texte rendu (S17 « tester un modèle ») | `template.unknown_variable` | F12 · P/T1 |
| `GET /message-variables` | `template.read` | → variables autorisées (jeu à fixer : Questions ouvertes) | | R-064 · P/T1 |

### 7.8 Comptes rendus, mandants et mandats (plan Agence)

| Méthode et chemin | Droit | Corps → réponse | Erreurs propres | Règles · F · Prio |
|---|---|---|---|---|
| `GET /owners`, `POST /owners` | `statement.read`, `mandate.write` | `{ kind, display_name, phone, payout_method }` → mandant ; dans le plan Propriétaire, une fiche `is_self` est créée avec l'organisation | | R-016 · F05 · P/T2 |
| `GET /owners/{id}`, `PATCH /owners/{id}` | idem | fiche, `version` | `concurrency.*` | F05 · P/T2 |
| `GET /owners/{id}/payout-details` | `mandate.write` | → coordonnées de paiement **révélées**, action tracée ; chiffrées en base | | SECURITY §4.2 · F05 · P/T2 |
| `GET /mandates`, `POST /mandates` | `mandate.write` | `{ owner_id, rate_bp, commission_base, start_date, unit_ids[] }` → mandat | `mandate.overlap` (un lot, un mandat actif), `mandate.rate_out_of_range` | R-014, R-015 · F05 · P/T2 |
| `GET /mandates/{id}` | idem | mandat, avenants de taux, lots couverts | | F05 · P/T2 |
| `POST /mandates/{id}/rate-changes` | `mandate.write` | `{ effective_from, rate_bp }` → avenant ; date d'effet au 1er d'un mois **strictement futur** (proposé, DB §4.7) | `mandate.rate_change_retroactive` | R-069 · E-17 · P/T2 |
| `POST /mandates/{id}/close` | `mandate.write` | `{ end_date }` → mandat clôturé | | F05 · P/T2 |
| `PUT /mandates/{id}/units` | `mandate.write` | `{ unit_ids[] }` → transfert de lots | `mandate.overlap` | R-014 · F05 · P/T2 |
| `GET /statements` | `statement.read` | filtres `period`, `owner_id`, `status` (`to_produce`, `validated`, `paid`) | | F13 · P/T2 |
| `GET /statements/preview` | `statement.read` | voir 6.3 | `statement.units_without_mandate` | R-068 à R-075 · F13 · P/T2 |
| `POST /statements`, `POST /statements/{id}/versions`, `POST /statements/{id}/reopen`, `POST /statements/{id}/payout`, `POST /statements/{id}/shares` | `statement.validate`, `payout.record` | voir 6.3 | voir 6.3 | R-073, R-076, R-077, R-083 · F13 · P/T2 |
| `GET /statements/{id}`, `GET /statements/{id}/versions/{v}` | `statement.read` | statut dérivé, versions, instantané | | R-074 · F13 · P/T2 |
| `GET /statements/{id}/versions/{v}/pdf` | `statement.read` | PDF figé (5.5) | `receipt.pdf_not_ready` | R-074 · F13 · P/T2 |

### 7.9 Import

Détail des routes en 6.4. Complément :

| Méthode et chemin | Droit | Corps → réponse | Règles · F · Prio |
|---|---|---|---|
| `GET /import-batches` | `import.run` | historique : fichier, date, auteur, statut, compteurs, `cancellable` | R-080 · F14 · P/T2 |
| `GET /import-batches/{id}/rows` | `import.run` | lignes filtrées par `status`, pagination par curseur | R-080 · F14 · P/T2 |
| `GET /import-mappings` | `import.run` | correspondances mémorisées par organisation | F14 · P/T2 |
| `GET /import-template` | `import.run` | modèle Excel de l'éditeur (statique, versionné) | F14 · P/T2 |

### 7.10 Dépenses et travaux

| Méthode et chemin | Droit | Corps → réponse | Erreurs propres | Règles · F · Prio |
|---|---|---|---|---|
| `GET /interventions` | `catalog.read` | filtres `unit_id`, `owner_id`, `period`, `status`, `imputable` → liste | | F15 · P/T2 |
| `POST /units/{id}/interventions` | `expense.write` | voir 6.5 → intervention + `warnings[]` | | R-078, R-084 · F15 · P/T2 |
| `GET /interventions/{id}` | `catalog.read` | détail, prestataire, pièces, `statement_link` | | F15 · P/T2 |
| `PATCH /interventions/{id}` | `expense.write` | `{ label, description, cost, status, … }` → intervention ; `version` | `expense.frozen`, `concurrency.*` | R-074 · F15 · P/T2 |
| `POST /interventions/{id}/imputation` | `expense.impute` | `{ imputable, reason }` → imputation changée ; audit `expense.imputation_changed` | `expense.frozen` | R-084 · F13, F15 · P/T2 |
| `GET /providers`, `POST /providers`, `PATCH /providers/{id}` | `expense.write` | `{ display_name, trade, phone }` | `concurrency.*` | F15 · A/T3 |
| `POST /units/{id}/interventions` par l'agent terrain | `expense.write` (proposer) | **non spécifié** : le mécanisme de proposition n'existe pas dans le dossier | | F15 · à valider |

### 7.11 Documents [APRÈS, T3]

| Méthode et chemin | Droit | Corps → réponse | Erreurs propres | Règles · F · Prio |
|---|---|---|---|---|
| `POST /documents` | `catalog.write` | multipart (`file`, `doc_type`, `links[]` : bien, lot, locataire, bail, intervention) → document ; 10 Mo, PDF, JPEG, PNG ; réencodage des images | `upload.too_large`, `upload.type_not_allowed` | R-005 · F16 · A/T3 |
| `GET /documents` | `catalog.read` | filtres `linked_to`, `doc_type` → liste | | F16 · A/T3 |
| `GET /documents/{id}/download` | `catalog.read` | flux, `Content-Disposition: attachment`, `nosniff`, **jamais inline** | | SECURITY §7.4 · F16 · A/T3 |
| `POST /documents/{id}/archive` | `catalog.write` | → document archivé (pas de suppression) | | R-005 · F16 · A/T3 |

### 7.12 Journal d'audit et exports

| Méthode et chemin | Droit | Corps → réponse | Erreurs propres | Règles · F · Prio |
|---|---|---|---|---|
| `GET /audit` | `audit.read` | filtres `actor_id`, `action` (code `objet.verbe`), `object_type`, `object_id`, `from`, `to`, `actor_kind` (`user`, `system`, `support`) → lignes `{ id, occurred_at, actor, action, object, before, after, ip?, request_id }` ; l'adresse IP n'est visible que du gérant et du comptable | | R-007 · F03 · P/T1 |
| `PUT, PATCH, DELETE /audit/{id}` | — | **toujours refusés** | `audit.immutable` (405) | R-007 · F03 · P/T1 |
| `POST /exports` | `export.run` 🔐 | `{ kind: "payments" ou "arrears" ou "statements" ou "audit", format: "csv" ou "xlsx" ou "pdf", filters }` → `202` + opération ; `GET /exports/{id}/download` ; **cellules neutralisées** contre les formules ; l'export est tracé (`export.created`) | | R-007 · F18 · A/T3 |

L'export complet d'une organisation est une **commande** (`immo org export`), pas une route, au pilote (`ARCHITECTURE.md` §8.7) : **[PILOTE]**. Les valeurs avant et après de l'audit ne contiennent ni secret ni jeton ; un champ chiffré affiche « modifié ».

### 7.13 Paramètres

| Méthode et chemin | Droit | Corps → réponse | Erreurs propres | Règles · F · Prio |
|---|---|---|---|---|
| `GET /settings` | `settings.read` | `{ receipt_prefix, receipt_header, address, legal_ids, default_paper, share_link_ttl_days, payment_confirm_threshold, version }` ; fuseau, devise, langue en lecture seule (R-003) | | F01, F10 · P/T1 |
| `PATCH /settings` | `settings.write` | mêmes champs → paramètres ; `If-Match` ; audit `settings.updated` | `validation.failed` ; changement de préfixe en cours d'année : question ouverte (DB §11.1 n° 9) | R-038, R-054 · DP-37 · P/T1 |
| `PUT /settings/logo`, `PUT /settings/signature` | `settings.write` | multipart PNG ou JPEG 200 Ko → image **réencodée** | `upload.type_not_allowed` | F10 · P/T1 |
| `GET /settings/export-formats` | `export.run` | formats disponibles | | F18 · A/T3 |

### 7.14 Interrupteurs de fonctionnalités

- **Lecture** : `GET /me` renvoie `features` (clés actives pour l'organisation). Le front masque l'entrée ; le serveur **refuse** la route d'une fonction désactivée par `403 feature.disabled` (`ARCHITECTURE.md` §8.2).
- **Écriture** : uniquement par la **console HABEK** (7.18), journalisée (`feature.toggled`). Aucune route de l'application cliente ne les modifie.
- Interrupteurs prévus au pilote : clôture de caisse (F19), « Coller le SMS » (F25), documents (F16), agenda (F17), pièce d'identité du locataire. Un interrupteur ne change jamais une règle d'argent pour une seule organisation (règle d'or 12). Distinct du **plan** (`plan.feature_unavailable`, R-082).

### 7.15 Événements d'usage et erreurs du front

| Méthode et chemin | Droit | Corps → réponse | Règles · F · Prio |
|---|---|---|---|
| `POST /events` | `events.create` | `{ events: [{ name, properties, client_version }] }` (20 au plus, 8 Kio) → `204` | `ARCHITECTURE.md` §8.4 · décision n° 28 · P/T1 |
| `POST /client-errors` | `events.create` | `{ message, screen, client_version, request_ids[] }` → `204` ; **aucun texte de l'écran** | idem · P/T1 |

- Même domaine, **sans cookie supplémentaire, sans identifiant publicitaire, sans service tiers** (dossier §12).
- Les noms d'événement et leurs propriétés forment une **liste blanche** (`ARCHITECTURE.md` §8.4) : `screen_viewed`, `payment_form_opened`, `payment_form_abandoned`, `search_performed`, `validation_error`, `reminder_prepared`, `reminder_abandoned`, `statement_previewed`, `import_*`, `feature_used`, … Les événements d'argent (`payment_recorded`, `payment_idempotent_replay`) sont **écrits par le serveur**, pas par le front.
- **Aucune donnée personnelle** dans `properties` : ni nom, ni téléphone, ni texte libre ; des identifiants internes, durées, comptes, codes d'erreur. Un événement inconnu ou une propriété hors liste est **écarté et compté**, sans erreur renvoyée (réponse `204`) ; en essai il échoue bruyamment pour que l'erreur se voie.
- Conservation 13 mois puis agrégats ; organisations `demo` et `test` exclues des indicateurs.

### 7.16 Signalements de problèmes

| Méthode et chemin | Droit | Corps → réponse | Règles · F · Prio |
|---|---|---|---|
| `POST /feedback` | `feedback.create` | `{ category: "bug" ou "request" ou "question", description, can_call_back, context: { screen, path, client_version, viewport, last_request_ids[5], last_error_code } }` → `201 { id, reference }` ; notification immédiate à l'équipe | `ARCHITECTURE.md` §8.4 · décision n° 28 · P/T1 |

- **Pas de capture d'écran** au pilote. `path` ne porte **ni paramètre de requête ni jeton** ; `description` est du texte libre : il est traité comme du texte, stocké sans analyse et jamais journalisé.
- Le suivi (`new`, `in_progress`, `done`, `rejected`) se fait dans la console. Le nombre de clients concernés alimente le registre des demandes (dossier §21).

### 7.17 Recherche, opérations, santé

| Méthode et chemin | Droit | Corps → réponse | Règles · F · Prio |
|---|---|---|---|
| `GET /search` | `search.use` | voir 2.6 | R-081 · dossier §14.1 · P/T1 |
| `GET /operations/{id}` | utilisateur à l'origine | voir 5.4 ; **filtre explicite sur l'organisation** (la table `jobs` n'a pas de sécurité par ligne) | P/T1 |
| `GET /health` | public | `{ "status": "ok" }` seulement : aucune donnée, aucune version, aucun détail ; sert à la sonde externe | `ARCHITECTURE.md` §10.3 · P/T1 |
| `GET /ready` | réseau interne | vérifie base et stockage ; non exposé à Internet | P/T1 |

### 7.18 Console HABEK (réservée)

La console est servie sur un **nom d'hôte distinct**, avec un cookie distinct, une session plus courte (15 minutes d'inactivité, 8 heures au plus) et une double authentification obligatoire (`SECURITY.md` §3.7). Elle n'affiche jamais empreintes de mots de passe, secrets TOTP, empreintes de jetons ni valeurs chiffrées en clair. Les routes qui lisent des données d'une organisation exigent un **accord support valide** (R-009) et écrivent une ligne d'audit `support`. **[PILOTE]** (F40, T2). Le détail relèvera de `specs/F40-console.md` (à créer avant la tranche 6) ; sont réservés :

| Domaine | Routes réservées | Remarque |
|---|---|---|
| Organisations | `GET/POST /console/organizations`, `POST …/{id}/go-live`, `POST …/{id}/suspend` | création **par la console** (inscription fermée, décision n° 51) ; bascule `setup` → `live` irréversible |
| Interrupteurs | `PUT /console/organizations/{id}/features/{feature}` | `feature.toggled` audité |
| Import assisté | routes de 6.4 dans le contexte d'une organisation | portée `import_assist` |
| Observation | `GET /console/feedback`, `GET /console/errors`, `GET /console/usage` | requêtes SQL fixes sur vues agrégées |
| Récupération de compte | `POST /console/users/{id}/password-resets` | vérification renforcée, deux personnes (`SECURITY.md` §3.5) |

Au pilote, la console peut être une **ligne de commande** (`immo org create`, `immo org go-live`, `immo import`) ; l'interface web est spécifiée avant la tranche 6. La remise à zéro d'une organisation en phase `setup` et l'export complet sont des **commandes** (`immo org purge`, `immo org export`), jamais des routes (DB §8.3, §8.5).

### 7.19 Contrats préparés, non implémentés au pilote

Réserve d'emplacement **seulement** : aucune spécification détaillée, aucune route servie (404). Ces zones sont gelées pour qu'aucune route actuelle ne les utilise.

| Fonction | Espace réservé | Contraintes déjà décidées |
|---|---|---|
| Paiement en ligne par mobile money (F26, P2, DP-32) | `/api/online-payments/*`, `/api/webhooks/*` | HABEK ne détient **jamais** de fonds (règle d'or 3) ; compte marchand propre à chaque client ; webhooks idempotents ; un paiement en ligne entre dans le même livre que les autres (`payments`), donc avec les mêmes garanties |
| Envoi automatique par WhatsApp (F27, P2, DP-33) | `/api/messaging/*` | l'humain reste dans la boucle tant que la preuve de demande manque ; coût par message à répercuter |

**T3 : à spécifier avant chaque lot** [APRÈS] (étiquette « à spécifier », ni route servie ni test écrit) :

| Fonction | Routes à spécifier | Tables |
|---|---|---|
| Agenda (F17, S15) | `GET /api/tasks`, `POST /api/tasks`, `PATCH /api/tasks/{id}` | `tasks` (prévue, non créée) |
| Notifications (F22, S20) | `GET /api/notifications`, `POST /api/notifications/{id}/read` | `notification_reads` (prévue, non créée) |

Chaque lot de T3 reprend ces trois spécifications (route, table, test) avant le code : `TESTING_QA.md` §2.1 donne les cas à écrire avec la règle.

Toute valeur d'énumération ajoutée plus tard (`payments.method`, canaux) est un **ajout** au sens de 1.3.

## 8. Tests de contrat et cohérence entre le front et le back

Aucune de ces vérifications n'est facultative : chacune empêche un front et un back de diverger sans que personne ne le voie. Détail d'exécution et seuils dans `TESTING_QA.md`.

| # | Contrôle | Mécanisme | Quand | Bloquant |
|---|---|---|---|---|
| C1 | Le fichier `openapi.json` versionné est celui que génère le code | la CI régénère et compare ; différence = échec | chaque PR | **oui** [PILOTE] |
| C2 | Aucun changement incompatible (retrait, renommage, changement de type, champ de requête rendu obligatoire, resserrement) sans dépréciation datée | outil de comparaison OpenAPI entre la branche et la branche principale | chaque PR | **oui** [PILOTE] |
| C3 | Le client TypeScript généré est à jour et le front compile contre lui | génération + `tsc --noEmit` | chaque PR | **oui** [PILOTE] |
| C4 | Chaque exemple de ce document est valide | `tests/contract/examples/*.json` validés contre le schéma OpenAPI ; ils servent aussi de réponses simulées au front | chaque PR | **oui** [PILOTE] |
| C5 | Chaque réponse réelle des tests d'intégration est conforme au schéma | validation de la réponse contre OpenAPI dans les tests d'API | chaque PR | **oui** [PILOTE] |
| C6 | Chaque route a ses tests : sans session (401), sans jeton anti-falsification (403), un test par couple rôle × action, ressource d'une autre organisation (404 + audit), format d'erreur | registre des routes comparé aux tests ; **une route sans test d'isolation fait échouer le build** (CLAUDE.md règle d'or 4) | chaque PR | **oui** [PILOTE] |
| C7 | Les routes d'écriture immuables refusent `PUT`, `PATCH`, `DELETE` avec le bon code | test dédié sur paiements, quittances, reversements, relances, audit, versions | chaque PR | **oui** [PILOTE] |
| C8 | Idempotence : même clé deux fois, trois fois en parallèle, contenu différent, après perte de réponse | tests d'intégration (`TESTING_QA.md`) | chaque PR | **oui** [PILOTE] |
| C9 | En-têtes de sécurité et absence de cache sur chaque type de page | test d'en-têtes | chaque PR | **oui** [PILOTE] |
| C10 | Les codes d'erreur du serveur ont tous un texte dans le front, et le front n'affiche aucun code inconnu | test qui compare le catalogue de codes du serveur et le fichier de textes | chaque PR | **oui** [PILOTE] |
| C11 | Toute valeur d'énumération du serveur est connue du front (ou gérée par le repli) | génération + test de repli | chaque PR | non (alerte) |
| C12 | Le front n'appelle que des routes existantes | le client généré est la seule porte d'appel ; une recherche en CI interdit `fetch(` hors de `web/src/api/` | chaque PR | **oui** [PILOTE] |
| C13 | Le front en circulation reste servi après le retour arrière | test de compatibilité : le front de la version N parle au serveur de la version N−1 sur les routes de T1 | avant chaque mise en production | non [APRÈS], recommandé |

## 9. Journal des changements d'API

Tenu dans ce fichier jusqu'au premier changement après le pilote, puis déplacé dans `api/CHANGELOG.md` (emplacement à confirmer). **Une ligne par changement qui touche un contrat**, y compris un ajout. Les changements incompatibles passent par une dépréciation datée (1.3).

| Date | Version du front (build) | Type | Changement | Retrait prévu | Lien (PR, DP-xx) |
|---|---|---|---|---|---|
| 30/10/2026 | — | création | Première version du contrat (proposition, aucune route implémentée) | — | décisions n° 61 à 68 |
| 01/11/2026 | 0.0.0 | ajout | `POST /api/auth/login`, `POST /api/auth/logout`, `POST /api/auth/logout-all`, `GET /api/me`, `POST /api/me/organization`, `GET /api/organization` (étape 02) ; trois codes d'erreur `auth.no_active_membership`, `auth.organization_required`, `org.suspended` ; `X-Request-Id` au format `INC-XXXXXX` | — | F01, F02, F03, DP-85, DP-87 |

Colonne « Type » : `ajout`, `modification compatible`, `dépréciation`, `retrait`, `correction de document`.

## Questions ouvertes

Mise à jour du 30/10/2026 : les questions tranchées sont suivies de la décision qui les ferme ; les autres restent ouvertes.

1. **Doublon de référence externe (R-041)** : *tranchée par R-102 (DP-58) : avertissement qui bloque jusqu'à confirmation explicite (`acknowledge_external_ref_duplicate`) ; une référence d'un paiement contre-passé est réutilisable.*
2. **Verrouillage optimiste** : *tranchée par DP-56 (décision n° 64) : `row_version` et `If-Match` dès le pilote ; la colonne est dans `DATABASE_SCHEMA.md` §1.2.*
3. **Confirmation du mot de passe (5 minutes)** : *tranchée par DP-56 : colonne `sessions.reauth_until`.*
4. **Variables des modèles de relance** : *tranchée (glossaire de `DOMAINE.md`, R-064) : six variables `{nom}`, `{montant}`, `{mois}`, `{lot}`, `{date}`, `{lien_quittance}` ; le contrat renvoie la liste (`GET /message-variables`).*
5. **Promesse « tenue »** : *tranchée par R-101 (DP-58).*
6. **Quittance rectificative sans effet sur le montant** : *tranchée par DP-64 : hors pilote ; la route `POST /receipts/{id}/corrective` reste réservée (404).* La rectificative après contre-passation (DP-38) est conservée.
7. **Impression de la quittance** : à partir du PDF (reproductible, rendu unique) ou d'un aperçu HTML (plus rapide) ? Conditionne les tests visuels et l'endpoint d'aperçu (décision n° 7). **Réponse provisoire (DP-76, 31/10/2026) : à partir du PDF**, seule option imprimée et validée par Mimis (laser noir et blanc, A4 et A5) ; un PDF émis ne se régénère pas (DP-71) ; l'aperçu HTML imprimable n'a pas été comparé [À confirmer à l'étape 08].
8. **Rendu des deux formats** (A4 et A5) à chaque paiement : *tranchée par DP-64 : les deux formats sont rendus, A5 par défaut.*
9. **Agent terrain** : « Proposer » une dépense : *tranchée par DP-66 : route refusée au pilote.*
10. **Collaborateur (plan Propriétaire)** : *tranchée par DP-66 (droits de §4.6, colonne Col).*
11. **Compteurs d'import (E-20)** : *tranchée (audit C2-046) : « 197 lignes créables » = valides + avertissements ; les compteurs de la base vérifient `total = valid + warning + error` (`DOMAINE.md` E-20).*
12. **Génération des échéances** : *tranchée par R-098, R-099 et R-100 (DP-58).*
13. **Dépenses d'un compte rendu, net ou base négatifs, reversement partiel** : *tranchées par R-105 à R-108 (DP-59).*
14. **Année de numérotation** : *tranchée par DP-61 : année d'émission ; elle fixe `receipt.number` dans l'exemple de 6.1.*
15. **Chemins du dossier et d'`ARCHITECTURE.md`** : *tranchée par DP-56 : préfixe `/api/` pour toutes les routes sauf `/health`, `/ready`, `/s/…`, `/v/…`, `/console/…` (voir 7.1).*
16. **Réinitialisation du mot de passe** : *tranchée par DP-68 : lien généré ; la microcopy de `UI_UX_DESIGN.md` (S01) est alignée.*
17. **Fichiers de test fournis par les pilotes** : usage réel autorisé seulement après anonymisation et accord écrit (décision n° 15). Le contrat d'import n'en dépend pas. Ouverte.

## Décisions à enregistrer dans DECISIONS.md

Mise à jour du 30/10/2026 : les décisions n° 61, 62, 64, 67 (et 69 à 77, 79, 80, 87 des autres documents) sont prises par DP-56 ; les n° 63, 65, 66 et 68 restent proposées [À valider]. Aucune autre décision n'est présentée comme **prise** dans ce fichier : les conséquences techniques directes de règles déjà décidées (idempotence DP-49, montants entiers DP-45, immutabilité DP-47, isolation DP-48) sont reprises telles quelles. Les choix ci-dessous sont **ouverts** [À valider], ajoutés au tableau des décisions ouvertes à la suite de la n° 60.

| N° | Décision | Recommandation de ce document |
|---|---|---|
| 61 | Source de vérité du contrat : OpenAPI généré depuis le code Rust, fichier versionné, différence relue ; ce document écrit d'abord sur le papier | Code d'abord (1.2) ; précise la n° 20 |
| 62 | Versionnement : pas de numéro dans les adresses, évolution additive, `X-Client-Version` et `X-Min-Client-Version`, dépréciation datée (14 jours au moins) | Oui (1.3) ; précise `ARCHITECTURE.md` §3.7 |
| 63 | Pagination par curseur opaque et signé, 25 lignes par défaut, 100 au plus | Oui (2.5) |
| 64 | Verrouillage optimiste : `row_version` et `If-Match` sur les fiches modifiables **dès le pilote** | Oui (5.2) ; avance un point de `DATABASE_SCHEMA.md` de [APRÈS] à [PILOTE] |
| 65 | Opérations longues (import, PDF, export) : `202` et suivi par interrogation de `/operations/{id}` | Oui (5.4) |
| 66 | Erreurs : `problem+json`, `code` stable et `params` ; **textes affichés centralisés dans le front** | Oui (2.7, 3) |
| 67 | Champs JSON en `snake_case` anglais, alignés sur la base | Oui (2.1) ; ferme dans ce sens la n° 18 pour l'API |
| 68 | PDF et fichiers servis en flux par l'API (pas d'adresse signée du stockage au pilote) | Oui (5.5) |

