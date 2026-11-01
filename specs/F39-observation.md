# F39 — Observation des usages, erreurs et signalements (étape 04)

| Champ | Valeur |
|---|---|
| Statut | **Rédigée et validée par Mimis le 01/11/2026** (recommandations acceptées, DP-90) |
| Étape | 04, `roadmap/etape-04-observation-et-signalements.md` §4 |
| Dépend de | décisions n° 28 et 66 ; `ARCHITECTURE.md` §8.4 ; `API_CONTRACTS.md` §7.15, §7.16 ; `DATABASE_SCHEMA.md` tables `product_events`, `error_reports`, `feedback_reports` ; `SECURITY.md` §7.7, §8.1 |
| Règles | aucune règle métier (R-xxx) ; règles d'or 4 (isolation) et 6 (écriture seule) |
| Critères visés | critères 1 à 5 de l'étape 04 ; PP-28 ; A13 |
| Écarts d'implémentation | DP-91 (liste blanche en table, référence de session en texte, événements de connexion en tâche détachée, limites connues) |
| Hors fiche | tableau de bord riche, console des signalements (étape 11), événements d'argent, de relance et de compte rendu (étapes 07, 15, 17), agrégats après 13 mois (étape 19) |
| Nom | cette fiche porte le même numéro F39 que `F39-sauvegardes.md` : le numéro désigne la fonction « exploitation et mesure » du dossier ; les deux fiches sont distinctes |

## 1. Objet

Mesurer l'usage réel sans service tiers, voir les erreurs vécues, et laisser l'utilisateur signaler un problème en trois gestes. Tout est stocké dans notre base, sous isolation par organisation, et ne contient **aucune donnée personnelle** : ni nom, ni téléphone, ni texte libre dans les événements et les erreurs. Le seul texte libre est la description d'un signalement, que l'utilisateur écrit pour nous.

## 2. Règles

| N° | Règle |
|---|---|
| 1 | **Liste blanche fermée.** Chaque événement est une variante de l'énumération Rust `ProductEvent` : nom, origine (`server` ou `client`), propriétés typées. Aucune propriété n'est une chaîne libre : identifiants internes, énumérations fermées, entiers bornés, codes (`R-xxx`, codes d'erreur). Une propriété textuelle ne compile pas |
| 2 | **Origine.** Les événements d'argent et de sécurité sont écrits **par le serveur** ; le front n'envoie que `screen_viewed` et `validation_error` à cette étape. Un événement d'origine `server` reçu par `POST /events` est écarté comme un événement inconnu (le front ne peut pas fabriquer un `login_failed`) |
| 3 | **Événement inconnu, propriété hors liste ou mal typée, nom d'origine interdite** : l'événement entier est écarté (jamais à moitié) et **compté** : une ligne `event_discarded` par lot et par motif (`unknown_name`, `bad_property`, `server_origin`, `batch_overflow`), propriétés `{ reason, count }`, sans le nom reçu (il pourrait contenir une donnée). Réponse `204` |
| 4 | **Hors production** (`APP_ENV` différent de `production` : développement et essai), un événement écarté renvoie `422 validation.failed` au lieu de `204` : l'erreur se voit. En production, la même requête reste `204` |
| 5 | **Valeurs interdites, doublement vérifiées** : en plus du typage, le serveur refuse toute valeur de chaîne contenant `@`, une suite de 6 chiffres ou plus, ou `+226`. Un événement qui y échoue est écarté (motif `bad_property`) |
| 6 | **Temps** : `occurred_at` est imposé par la base (`app_now()`) ; le front n'envoie aucune date. `client_version` : motif `^\d+\.\d+\.\d+(-[a-z0-9.]{1,12})?$`, 20 caractères au plus, sinon l'événement est écarté |
| 7 | **Identité** : `org_id`, `user_id`, `role`, `session_id` viennent de la session, jamais du corps. Un événement n'existe que dans une session ouverte : aucun événement du front avant la connexion (l'écran de connexion n'émet rien ; `login_failed` est écrit par le serveur) |
| 8 | **Organisations** `demo` et `test` : leurs événements sont stockés (utiles pour les essais) mais **exclus des indicateurs** par `kind = 'real'` |
| 9 | **Ajout seul.** `product_events` n'a ni `UPDATE` ni `TRUNCATE`. `DELETE` n'est permis que sur les lignes de plus de 13 mois (politique de ligne, §3 point 3), pour la purge |
| 10 | **Conservation** : 13 mois [Hypothèse] pour `product_events` et pour `error_reports` (depuis `last_seen_at`). La purge est une tâche de fond (file de l'étape 01). Les agrégats qui doivent survivre sont conçus à l'étape 19 : rien ne tombe sous le seuil avant décembre 2027 |
| 11 | **Aucune requête n'attend l'observation.** Un échec d'écriture d'un événement ou d'un rapport d'erreur est journalisé (`tracing`, sans donnée) mais ne fait jamais échouer la requête de l'utilisateur. Seul le signalement (règle 14) est transactionnel |
| 12 | **Suivi d'erreurs serveur.** Toute erreur non prévue (réponse 500, panique rattrapée) est agrégée par **empreinte** = SHA-256 de (source, modèle de route, type d'erreur, 5 premiers cadres de la pile). Le message n'est pas dans l'empreinte. Écriture dans une transaction **séparée** (celle de la requête est annulée) : `INSERT … ON CONFLICT (org_id, fingerprint) DO UPDATE SET occurrences = occurrences + 1, last_seen_at = app_now()` |
| 13 | **Contenu de `sample`** (jsonb, 2 Kio) : identifiant de requête, modèle de route, méthode, statut, version du serveur, pile d'appels (noms de fonctions seulement), type d'erreur, et pour une erreur de base le code SQLSTATE et le nom de la contrainte. **Jamais** : corps de requête, valeurs de paramètres, message d'erreur de la bibliothèque (il peut citer une valeur), en-têtes. Seul le dernier exemple est gardé. Plafond : 200 empreintes distinctes par organisation et par jour ; au-delà, seules les occurrences de celles qui existent sont comptées |
| 14 | **Signalement.** Créé dans la transaction de la requête avec : une ligne `feedback_reports`, une tâche de notification dans la file (même transaction). Si l'un échoue, rien n'est créé (comme R-008). La description est traitée comme du texte inerte : stockée telle quelle (5 à 4 000 caractères après suppression des espaces de bord), **jamais journalisée**, jamais copiée dans un événement ni dans l'alerte |
| 15 | **Référence lisible** : `SIG-000042` (préfixe, six chiffres, séquence globale). Les trous sont sans conséquence (ce n'est pas une numérotation légale). La référence est affichée à l'utilisateur et donnée au support ; elle ne donne accès à rien |
| 16 | **Contexte du signalement** : `{ screen, path, client_version, viewport, last_request_ids[≤5], last_error_code }` ; `path` est le **modèle** d'adresse sans paramètre de requête, sans fragment, sans jeton (les adresses `/s/…`, `/invite/…`, `/reset/…` sont remplacées par leur modèle) ; le rôle vient de la session ; 2 Kio au plus ; pas de capture d'écran |
| 17 | **Alerte** : la tâche écrit dans `platform_events` une ligne `feedback.received` (rôle `immo_platform`, via `immo work`) et émet l'événement `tracing` `alert = "feedback.received"`. Le message ne contient que : référence, catégorie, écran, nature de l'organisation (`réelle`, `démo`, `test`), « rappel accepté » oui ou non. **Pas** de description, de nom d'agence, de nom d'utilisateur ni de numéro |
| 18 | **Accès** : tout utilisateur connecté, quel que soit son rôle, peut envoyer des événements, des erreurs de front et un signalement (droits `events.create` et `feedback.create` accordés à tous les rôles, ajoutés à la matrice de F02). L'utilisateur ne relit pas ses signalements dans l'application à cette étape |

## 3. Données (migration de l'étape)

1. **`product_events`**, **`error_reports`**, **`feedback_reports`** comme dans `DATABASE_SCHEMA.md`, avec les écarts suivants (la documentation propriétaire sera corrigée **avant** la migration, `CLAUDE.md`) :
   - `product_events` : ajout de `session_id uuid` (l'étape et `ARCHITECTURE.md` §8.4 le demandent, le schéma l'omet) ; clé étrangère de `session_id` **non posée** (les sessions sont purgées bien avant 13 mois) ;
   - `feedback_reports` : ajout de `reference text NOT NULL UNIQUE DEFAULT 'SIG-' || lpad(nextval('feedback_reference_seq')::text, 6, '0')` ; contrôles `category IN ('bug','request','question')`, `status IN ('new','in_progress','done','rejected')`, `pg_column_size(context) <= 2048` ; la clé étrangère `handled_by → platform_staff` est **posée avec la console** (étape 11), la table n'existant pas encore (même traitement que `support_grants.staff_id`, F40 §3 point 4) ;
   - noms de colonnes conservés : `body` (= `description` de l'API), `callback_ok` (= `can_call_back`).
2. **Droits** de `immo_app` : `product_events` SELECT, INSERT, DELETE ; `error_reports` SELECT, INSERT, UPDATE (`last_seen_at`, `occurrences`, `sample`), DELETE ; `feedback_reports` SELECT, INSERT ; `USAGE` sur la séquence de référence. `immo_platform` : SELECT sur les trois tables et UPDATE de `status`, `handled_by`, `handled_at` sur `feedback_reports`.
3. **Politique de suppression** de `product_events` : `FOR DELETE USING (occurred_at < app_now() - interval '13 months')` en plus de `org_isolation`. Sans elle, le droit `DELETE` du schéma permettrait d'effacer le journal d'usage.
4. **Lecture par `immo_platform` sans accord support** : politique `platform_read` sur les trois tables (voir question 1). Les autres tables à `org_id` gardent la politique `support_access` de F40 ; le test ISO-6 étendu de F40 est adapté (liste d'exceptions écrite dans le test, trois tables, motif : télémétrie du produit).
5. **Index** : `(org_id, name, occurred_at DESC)` du schéma, plus `(occurred_at)` pour la purge.

## 4. API

| Route | Corps → réponse | Particularités |
|---|---|---|
| `POST /events` | `{ events: [{ name, properties, client_version }] }` → `204` | 20 événements et 8 Kio au plus par lot |
| `POST /client-errors` | `{ message, screen, client_version, request_ids[≤5] }` → `204` | message de 200 caractères au plus, voir règle ci-dessous |
| `POST /feedback` | `{ category, description, can_call_back, context }` → `201 { id, reference }` | clé d'idempotence (double clic = un seul signalement) |

- Corps de plus de 8 Kio : `413 request.too_large`. Champ inconnu au niveau du corps : `400 request.unknown_field` (la souplesse de la règle 3 ne vaut qu'à l'intérieur de `events[]`).
- **Lot de plus de 20 événements** : les 20 premiers sont traités, les suivants écartés et comptés (`batch_overflow`), `204`.
- **Limites de débit par session** (en mémoire, comme DP-87) : `/events` 30 requêtes par minute, `/client-errors` 10 par minute, `/feedback` 5 par heure et par utilisateur. Dépassement : `429 rate_limit.exceeded` avec `Retry-After`. Le front ignore en silence un `429` sur `/events` et `/client-errors` (il espace ses envois) ; sur `/feedback` il affiche le bandeau « Trop de demandes. Réessayez dans {durée}. »
- **Erreur du front** : le serveur nettoie le message (suites de 6 chiffres, adresses électroniques, jetons de 20 caractères hexadécimaux ou plus remplacés par `…`), coupe à 200 caractères, ne garde **ni la pile ni l'adresse**. Empreinte = SHA-256 de (`client`, écran, message nettoyé, version). Écrit aussi un événement `client_error { screen, code }` (code = type JavaScript : `TypeError`, `RangeError`, `unhandled_rejection`, …).
- Toutes les routes exigent une session et l'en-tête anti-falsification habituel (`auth.csrf_invalid` sinon). Aucun cookie nouveau.
- Les routes sont ajoutées à `openapi.json` ; la **liste blanche** est exportée en `tests/contract/events-whitelist.json`, régénérée par un test et comparée en CI comme `openapi.json` : tout ajout d'événement apparaît dans une revue.
- **Route d'essai** `GET /api/_test/server-error` (provoque une erreur 500 réelle, pour le critère 4 et la démonstration) : montée seulement si `APP_ENV` n'est pas `production` **et** `ENABLE_TEST_ROUTES=1` ; absente de `openapi.json` ; un test vérifie le `404` avec la configuration de production.

## 5. Événements de l'étape

| Événement | Origine | Propriétés | Émis quand |
|---|---|---|---|
| `screen_viewed` | client | `screen` (S01 à S26), `width_class` (`mobile`, `tablet`, `desktop`) | changement d'écran (même écran répété dans la seconde : ignoré) |
| `validation_error` | client | `screen`, `field` (`^[a-z][a-z0-9_]{1,40}$`, constante déclarée dans le front), `rule_code` (`R-\d{3}` ou code d'erreur) | une erreur de saisie est affichée |
| `permission_denied` | server | `action` (code de droit), `role` | chaque `403 auth.forbidden` (en plus de la ligne d'audit existante) |
| `slow_response` | server | `route` (modèle), `method`, `status`, `duration_ms` | réponse de plus de 2 000 ms [Hypothèse] |
| `client_error` | server | `screen`, `code` | réception d'une erreur du front |
| `login_failed` | server | `reason` (`bad_credentials`, `rate_limited`, `totp_failed`, `reauth_failed`) | échec dont le compte est identifié (règle ci-dessous) |
| `session_expired` | server | `reason` (`idle`, `absolute`, `revoked`) | une requête arrive avec une session échue dont la ligne est retrouvée |
| `password_reset_requested` | server | `channel` (`admin_link`, `self_service`) | demande de réinitialisation |
| `feedback_submitted` | server | `screen`, `category` | signalement créé |
| `event_discarded` | server | `reason`, `count` | règle 3 |

- **`login_failed` et organisation** : `org_id` est obligatoire dans la table. L'événement n'est écrit que si le compte est identifié **et** appartient à une seule organisation active ; sinon il n'est pas écrit (la friction de connexion reste lisible dans `login_attempts`, déjà existante). Une identité inconnue n'écrit rien.
- **`password_reset_requested`** : la réponse à l'utilisateur reste identique que le compte existe ou non, et l'écriture de l'événement passe par la file de tâches (pas de différence de durée observable : critère de F01).
- Les événements de l'argent, des imports et des relances (liste d'`ARCHITECTURE.md` §8.4) sont ajoutés par les étapes concernées ; chaque étape étend l'énumération, le fichier de liste blanche et les indicateurs.

## 6. Indicateurs (`ops/sql/indicateurs.sql`)

Requêtes fixes, lecture seule, `kind = 'real'` partout : (1) activité par organisation et par jour ; (2) écrans les plus et les moins consultés ; (3) médiane et 90ᵉ centile du temps pour encaisser (`payment_recorded.duration_ms`) ; (4) part des paiements saisis sous 48 h (`payment_recorded` avec retard de saisie ≤ 2 jours) ; (5) abandons de formulaire de paiement ; (6) erreurs par type et par empreinte ; (7) signalements ouverts par statut et ancienneté. Les noms de propriétés des requêtes (3) à (5) sont **provisoires** jusqu'à l'étape 07, qui crée ces événements ; à l'étape 04, elles sont validées sur des lignes insérées par le test. Exécutées avec `immo_platform` (lecture seule, règle 4 de §3).

## 7. Interface

- **Client `track()`** (`web/src/observation/`) : file en mémoire (jamais écrite sur le disque du téléphone), 100 événements au plus (les plus anciens sont jetés), envoi au bout de 10 s ou à 20 événements, et à la mise en arrière-plan de la page (`fetch` avec `keepalive`, en-tête anti-falsification). Aucune attente de réponse, aucune relance après échec : une mesure perdue vaut mieux qu'un écran ralenti. Aucune bibliothèque tierce.
- **Erreurs** : `window.onerror` et `unhandledrejection` envoient vers `/client-errors`, au plus une erreur identique par minute et 5 par session.
- **« Signaler un problème »** : entrée du menu utilisateur et de la feuille « Plus », visible de tous les rôles. Feuille/modale accessible : catégorie (« Un bogue », « Une demande », « Une question »), description (zone de texte, compteur, 5 à 4 000 caractères), case « Vous pouvez me rappeler », résumé des erreurs en tête, étiquettes liées, 44 px. Texte d'avertissement : « Ce message sera lu par l'équipe HABEK. N'y écrivez aucun mot de passe. » Après envoi : toast « Merci. Votre référence : SIG-000042 » (référence copiable). Bandeau en cas de `429`. Textes dans `texts/fr.ts` (décision n° 66).
- États : initial, envoi, succès, erreur ; clavier et lecteur d'écran vérifiés à 390 et 1440 px.

## 8. Cas limites

| Cas | Comportement |
|---|---|
| Lot de 21 événements | 20 traités, le 21ᵉ écarté et compté, `204` |
| Corps de 8 Kio + 1 octet | `413 request.too_large` |
| Événement inconnu, production | `204`, ligne `event_discarded` |
| Événement inconnu, essai ou développement | `422 validation.failed` |
| Événement `login_failed` envoyé par le front | écarté (`server_origin`) |
| Propriété `"name": "Traoré"` ou `"phone": "+22670123456"` | l'événement est écarté (`bad_property`), rien d'autre n'est écrit |
| Navigateur hors connexion | la file se vide à 100 ; aucun écran bloqué ; rien n'est rejoué |
| Description de 4 caractères, ou de 5 espaces | `422 validation.failed` |
| Description contenant un numéro ou un jeton | stockée telle quelle (l'utilisateur écrit pour nous), absente des journaux, des événements et de l'alerte |
| Double clic sur « Envoyer » | une seule ligne (clé d'idempotence), même référence |
| 6ᵉ signalement en une heure | `429 rate_limit.exceeded`, bandeau |
| Même erreur serveur deux fois | une ligne, `occurrences = 2`, `last_seen_at` mis à jour |
| Erreur serveur hors session (pas d'organisation) | non agrégée dans `error_reports` (`org_id` obligatoire) : journal `tracing` avec empreinte et identifiant de requête seulement (question 3) |
| Écriture d'événement en échec (base lente) | la requête de l'utilisateur réussit ; `tracing` note l'échec |
| Signalement d'une organisation `demo` | stocké, alerte étiquetée `démo`, exclu des indicateurs |
| Signalement d'une autre organisation lu par une session | invisible (RLS) |
| Console | `immo_platform` lit les trois tables, ne peut ni insérer un événement ni modifier autre chose que le statut d'un signalement |

## 9. Tests

| Test | Contenu | Critère |
|---|---|---|
| Types | une propriété `String` libre dans `ProductEvent` ne compile pas (`trybuild` ou revue de la signature : `Properties` sans variante `String` ouverte) | 2 |
| Liste blanche | chaque variante : un lot valide est écrit ; un nom inconnu, une propriété en trop, un type faux, une valeur à `@` ou à 6 chiffres, un événement d'origine serveur : écarté, compté, `204` (production) puis `422` (essai) | 2, 3 |
| Limites | 20 événements acceptés, le 21ᵉ écarté ; 8 Kio acceptés, 8 Kio + 1 octet `413` ; propriétés de plus de 1 Kio écartées | 3 |
| Isolation (ISO-x) | un utilisateur de l'organisation A ne lit ni ne supprime les trois tables de B ; `INSERT` avec le `org_id` de B refusé | 2 |
| Ajout seul | `UPDATE` et `TRUNCATE` de `product_events` refusés ; `DELETE` d'une ligne récente refusé ; `DELETE` d'une ligne de 13 mois et 1 jour accepté | 2 |
| Droits (A3) | `events.create` et `feedback.create` : 3 routes × 7 rôles = 21 couples autorisés, 3 routes sans session = 401 | 1 |
| Erreur serveur | la route d'essai provoque une 500 : 1 ligne, `occurrences = 1`, puis `2` ; `sample` ne contient ni corps ni valeur de paramètre ; deux routes d'organisations différentes : deux lignes | 4 |
| Erreur du front | message contenant un numéro et une adresse électronique : nettoyé ; pile et adresse non gardées | 2 |
| Signalement | création, référence `SIG-` + 6 chiffres, contexte tronqué (`path` sans paramètre ni jeton), ligne `feedback_submitted`, tâche de notification dans la même transaction (échec de la tâche : pas de signalement) | 1 |
| Alerte | `immo work` écrit `feedback.received` dans `platform_events` en moins de 60 s ; le message ne contient pas la description | 1 |
| A13 | parcours complet (connexion, écrans, signalement dont la description contient `+22670123456` et un jeton, erreur du front) : le numéro, le jeton et la description sont absents des journaux, de `product_events`, de `error_reports`, de l'alerte et de `platform_events` ; recherche de motifs sur les lignes de la base | 2 |
| Indicateurs | `indicateurs.sql` s'exécute ; avec des lignes insérées par le test, résultats attendus ; l'organisation `demo` est exclue | 5 |
| Route d'essai | `404` quand `APP_ENV=production` | 4 |
| Front | `track()` envoie par lot, ne bloque pas hors connexion, jette au-delà de 100, n'attend jamais une réponse ; formulaire : validation, états, accessibilité ; E2E à 390 et 1440 px (signalement, erreur JavaScript provoquée, erreur serveur simulée) | 1 |

## 10. Ce qui sera vérifié sur le poste, et ce qui ne le sera pas

- Critère 1 (« alerte reçue sur le canal en moins d'une minute ») : vérifié jusqu'à `platform_events` et `tracing` seulement. **Le canal d'alerte réel (messagerie ou courriel de l'équipe) n'existe pas** : c'est la dette de l'étape 01 (critère 7, PP-15, C-01/C-02). Cela sera écrit tel quel dans le rapport.
- Critères 4 et 5 « sur l'essai » : l'essai n'existe pas ; vérifiés sur la pile locale.
- Téléphone réel : non (comme pour l'étape 03) ; 390 px dans Chrome.

## 11. Décisions et questions pour Mimis

1. **Lecture de la télémétrie par la console sans accord support.** Le schéma dit que `immo_platform` ne voit une organisation que sous accord valide (R-009). Pour lire les signalements et les indicateurs de **toutes** les agences sans demander un accord à chacune, il faut une exception : `immo_platform` lit `product_events` et `error_reports` (aucune donnée personnelle) et `feedback_reports` (texte que l'utilisateur envoie volontairement à HABEK, avec l'avertissement du formulaire). *Recommandation : accepter, exception écrite dans `DECISIONS.md`.* Contrepartie : la migration touche une garde, donc **second avis (C-19)** de plus. Autre option : aucune exception, et la console n'affiche que ce que l'agence a autorisé (la console des signalements deviendrait inutilisable).
2. **Alerte.** Le canal réel n'existe pas encore. *Recommandation : livrer l'alerte jusqu'à `platform_events` et `tracing` ; brancher le vrai canal quand C-01/C-02 seront fournis, sans changer ce code.* Souhaitez-vous un canal précis (courriel, WhatsApp, Slack) pour que je le prépare ?
3. **Erreurs serveur sans organisation** (route publique, connexion qui échoue en 500) : `error_reports.org_id` étant obligatoire, elles ne sont pas agrégées en base, seulement journalisées avec leur empreinte. *Recommandation : accepter* ; sinon rendre `org_id` nullable (RLS à adapter, plus risqué).
4. **`login_failed` et `session_expired` hors organisation identifiable** : non écrits dans `product_events` (lisibles dans `login_attempts`). *Recommandation : accepter.*
5. **Seuils proposés** : lot de 20 / 8 Kio (étape), réponse lente à 2 s, limites 30/10/5 par minute-minute-heure, 100 événements en mémoire du téléphone, 13 mois. *Recommandation : accepter ; à ajuster d'après le pilote.*
6. **Route d'essai** `GET /api/_test/server-error`, absente de la production par configuration (même image que la production, donc pas de « compilation différente »). *Recommandation : accepter.* Autre option : provoquer l'erreur seulement par un test automatisé, sans route (la démonstration 3 deviendrait une démonstration de test).
7. **Message du signalement lu par l'équipe seule** : l'utilisateur ne retrouve pas ses signalements dans l'application (hors fiche, étape 11 pour la console). *Recommandation : accepter.*
