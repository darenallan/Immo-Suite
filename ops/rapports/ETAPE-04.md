# Étape 04 — Rapport de tests et de démonstration (01/11/2026)

Étape : « Observation des usages, erreurs et signalements » (`roadmap/etape-04-observation-et-signalements.md`). Fiche `specs/F39-observation.md` **validée par Mimis le 01/11/2026** (recommandations acceptées, DP-90) ; DP-91 (écarts d'implémentation, **à relire**).

**Verdict : les 5 critères sont vérifiés sur le poste, avec trois réserves** (voir « Écarts et points à décider ») : l'alerte n'est vérifiée que jusqu'à `platform_events` et aux journaux (le canal réel n'existe pas), rien n'a tourné sur l'essai ni la production (inexistants, dette de l'étape 01), et la nouvelle politique `platform_read` attend le second avis C-19.

## Résultats réels (aucun chiffre estimé)

| Famille | Résultat | Commande |
|---|---|---|
| Rust : domaine, application, serveur, base | **270 réussis, 0 échec** (236 à l'étape 03) | `cargo test --workspace --locked` |
| Rust : clippy `-D warnings`, `cargo fmt` | verts | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| Front : typage, ESLint, Prettier, types d'API (`api:check`) | verts | `npm run typecheck`, `lint`, `format:check`, `api:check` |
| Front : tests unitaires et de composants | **51 réussis** (35 d'avant + 16 de l'étape) | `npm test --prefix web` |
| Front : poids (brotli) | JS 73,2 Ko / 150, CSS 2,3 Ko / 30, total 75,7 Ko / 200 (JS +2,2 Ko) | `npm run size --prefix web` |
| Navigateur sur la **pile réelle**, 390 et 1440 px | **12 réussis**, 2 ignorés (expiration d'inactivité, `-ShortIdle`) ; dont le parcours de démonstration de l'étape à chaque largeur | `powershell -File ops/dev/real-stack.ps1` |

Tests ajoutés (Rust) : `step04_observation.rs` (18 : ajout seul, isolation, lecture par la console, purge, lot valide, événement inconnu en production et en essai, limites 20 / 8 Kio / débit, 401 sans session, erreur du front, signalement, validation et idempotence, erreur serveur et occurrences, route d'essai absente en production, `permission_denied`, `login_failed`, `session_expired`, réinitialisation, indicateurs), `step04_logs.rs` (A13), 14 tests unitaires du module `observation`, `events_whitelist.rs` (liste blanche versionnée). La matrice A3 passe de 20 à 23 routes à droit (161 couples, 0 écart).

## Critères d'acceptation (étape 04, §6)

| # | Critère | État | Preuve |
|---|---|---|---|
| 1 | Un signalement crée une ligne avec référence, contexte et alerte en moins d'une minute | **vérifié, sauf le canal réel** | ligne `feedback_reports` avec `SIG-` + 6 chiffres, contexte sans paramètre ni jeton ; tâche créée dans la même transaction ; `immo work` écrit `feedback.received` dans `platform_events` en moins de 30 s (pile réelle, deux largeurs) ; **le canal d'équipe (courriel, messagerie) n'existe pas** : dette de l'étape 01 |
| 2 | Aucun nom, téléphone ni jeton dans les événements, signalements (hors texte libre) et journaux | **vérifié** | A13 : numéro, jeton, description et adresse électronique absents des journaux `tracing`, de `product_events`, `error_reports`, `platform_events`, du contexte et de `jobs` ; valeurs `@`, `+226`, 6 chiffres refusées par la validation ; la description, elle, est stockée (son seul lieu) |
| 3 | Événement inconnu écarté, compté, `204` ; en essai il échoue | **vérifié** | production : `204` et lignes `event_discarded` (jamais le nom reçu) ; essai : `422` ; événement d'origine serveur envoyé par le front : écarté ; 21ᵉ événement écarté et compté ; lot de plus de 8 Kio : `413` |
| 4 | Erreur serveur provoquée : une fois dans `error_reports`, avec son nombre d'occurrences | **vérifié sur la pile locale** | 1 ligne à « 1 occurrence » puis « 2 occurrences » ; exemple sans corps ni paramètre ; deux organisations, deux lignes ; navigateur réel : 4 occurrences sur une seule ligne après deux largeurs |
| 5 | `indicateurs.sql` s'exécute et exclut la démonstration | **vérifié sur la pile locale** | les huit requêtes avec le rôle de la console : médiane 30 000 ms et 2 sur 3 sous 48 h sur des lignes d'essai, organisation `demo` exclue, aucune colonne de nom ni de texte |

## Liste de démonstration (Mimis)

| # | Étape | État |
|---|---|---|
| 1 | Menu → « Signaler un problème », défaut fictif, référence affichée | **fait dans Chrome à 390 et 1440 px sur la pile locale** (résumé d'erreur si moins de 5 caractères, puis référence `SIG-00000x` copiable) ; **pas sur un vrai téléphone** |
| 2 | Alerte reçue sur le canal, ligne en base | **ligne en base et `platform_events` : faits** ; **canal réel : non** |
| 3 | Erreur de test : « 1 occurrence » puis « 2 occurrences » | **fait** par la route d'essai (`ENABLE_TEST_ROUTES=1`), pas sur l'essai |
| 4 | Requêtes d'indicateurs : seuls les écrans apparaissent, sans nom ni numéro | **fait** sur des lignes d'essai ; sur la pile réelle, `screen_viewed`, `login_failed`, `client_error` et `feedback_submitted` s'enregistrent |

Pour rejouer à la main : `powershell -File ops/dev/real-stack.ps1 -KeepStack` (route d'essai et travailleur d'alerte démarrés ; arrêt par `task.ps1 stop -Name real-api`, `real-work`, `real-web`), puis `psql -f ops/sql/indicateurs.sql`.

## Défauts trouvés par les tests et corrigés dans l'étape

- **Intergiciel d'observation non `Send`** : garder une référence à la réponse pendant un `await` empêchait la compilation ; les indices sont copiés avant.
- **Libellé du formulaire** : le champ « Décrivez ce qui s'est passé » englobait aussi l'aide et le compteur dans son nom accessible ; l'étiquette est désormais liée au champ seul.
- **Doublon d'écran** : la règle « même écran dans la seconde » persistait d'un test à l'autre ; elle est remise à zéro avec la file.
- **Matrice A3** : comptait 20 routes ; elle en compte 23.

## Écarts et points à décider

- **Canal d'alerte** : aucun canal réel (C-01, C-02). Le critère 1 est vérifié jusqu'à `platform_events` et au journal ; le branchement n'exige aucun changement de ce code.
- **Second avis (C-19)** : la migration `202611031000_observation.sql` ajoute `platform_read` (lecture de la télémétrie sans accord support, exception à R-009 voulue par DP-90). À désigner par Mimis avant toute promotion en production ; il couvre aussi les étapes 02 et 03 (dette ouverte).
- **Écarts d'implémentation** consignés dans DP-91 (à relire), en particulier : liste blanche en table, référence de session en texte de 16 caractères, événements de connexion écrits en tâche détachée, limites de débit qui comptent les tentatives.
- **Non couvert** : le code SQLSTATE d'une erreur de base n'est pas joint aux erreurs 500 (la cause est abandonnée avant `unexpected` dans le code existant) ; le nettoyage d'une erreur du front ne reconnaît pas un mot libre (le front n'envoie que des messages de son propre code).
- **Conservation de 13 mois** : fonction et tâche quotidienne testées avec des lignes vieillies ; aucun effet réel avant décembre 2027. Les agrégats qui doivent survivre restent à concevoir (étape 19).
- **Test de performance du journal (étape 03)** : il a échoué une fois pendant que la suite tournait en même temps que les outils du front (charge du poste) ; il passe en exécution normale. Même sensibilité à la charge que le test A1 de l'étape 02.
- **Incident d'outillage de cette session** : un `python -` nu lancé par erreur a rempli un journal temporaire de 127 Mo avant d'être arrêté (par sa tâche) ; le fichier, dans le dossier temporaire de la session, a été supprimé. Aucun fichier du dépôt ni de données touchés.
- **Rien n'est commité** : le dépôt local contient tout le travail des étapes 03 et 04 en modifications.
