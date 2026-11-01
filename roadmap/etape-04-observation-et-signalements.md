# Étape 04 — Observation des usages, erreurs et signalements

| Champ | Valeur |
|---|---|
| Statut | Faite et éprouvée en local le 01/11/2026, en attente de la validation de Mimis (`ops/rapports/ETAPE-04.md`) |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T1 (journalisation technique de F39 ; exigence d'`ARCHITECTURE.md` §8.4) |
| Effort | M, environ 7 JD |
| Dépend de | Étape 02 (peut se faire en parallèle de l'étape 03) |
| Jalon | Contribue à J3 (PP-28) |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Mettre en place, **avant** les écrans d'argent, la mesure des usages sans traceur tiers, le suivi des erreurs et le bouton « Signaler un problème ». Chaque étape suivante n'a plus qu'à émettre ses événements.
Valeur : le pilote sert à apprendre ; sans cette étape, le temps pour encaisser, les abandons et les erreurs d'import ne seraient mesurables qu'à la main et trop tard (DP-53 : 60 % des paiements saisis sous 48 h).

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections |
|---|---|---|
| Événements, erreurs, signalements | n° 28 ; F39 | `ARCHITECTURE.md` §8.4 en entier |
| Contrat | `API_CONTRACTS.md` | §7.15 (`POST /events`, `POST /client-errors`), §7.16 (`POST /feedback`) |
| Tables | `DATABASE_SCHEMA.md` §3 | « Paramètres, interrupteurs et observation » : `product_events`, `error_reports`, `feedback_reports` |
| Minimisation | `SECURITY.md` | §7.7, §8.1 (ligne « Événements d'usage » et « Signalements ») |
| Écran | `UI_UX_DESIGN.md` | « Navigation » (menu utilisateur, feuille « Plus »), « Microcopy » (erreurs, confirmations) |
| Critère | `TESTING_QA.md` | PP-28, §8.2 (temps pour encaisser), A13 |
| Alertes | `DEPLOYMENT.md` §7.3 (ligne « Signalement d'un pilote ») | idem |

## 3. Prérequis

Étape 02 (sessions, `OrgTx`, intergiciels, canal d'alerte de l'étape 01). Décisions n° 28 (événements en base, sans tiers) et n° 66 (textes d'erreur centralisés dans le front) confirmées. Durée de conservation de 13 mois [Hypothèse] à confirmer dans `specs/`.

## 4. Fiche de spécification à produire

`specs/F39-observation.md` — **règles** : liste blanche des noms d'événements et de leurs propriétés (identifiants internes, durées, comptes, codes d'erreur, jamais de nom, téléphone ni texte libre) ; événements d'argent écrits par le serveur, événements d'écran envoyés par le front en lots ; organisations `demo` et `test` exclues des indicateurs ; conservation 13 mois puis agrégats. **Cas limites** : événement inconnu (écarté, compté, réponse 204 ; en essai il échoue bruyamment) ; lot de plus de 20 événements ; texte libre dans un signalement (traité comme du texte, jamais journalisé) ; navigateur hors connexion. **Écrans** : bouton « Signaler un problème » (formulaire : description, catégorie bogue/demande/question, case « vous pouvez me rappeler »). **Erreurs** : `request.*`, `rate_limit.exceeded`. **Cas chiffrés** : 20 événements par lot, 8 Kio par lot, 5 derniers identifiants de requête joints. **S'arrêter pour validation de Mimis.**

## 5. Tâches détaillées

### 5.1 Base de données
`product_events` (bigint, `org_id`, `user_id`, `role`, `session_id`, `name`, `properties jsonb`, `client_version`, `occurred_at` imposé par la base) sous RLS forcée, ajout seul ; `error_reports` (empreinte, route, version, nombre, première et dernière occurrence, sans corps de requête) ; `feedback_reports` (catégorie, description, `can_call_back`, contexte JSON limité, statut `new|in_progress|done|rejected`, `org_id`, auteur). Index `(org_id, name, occurred_at)`. Contraintes : `properties` limité en taille ; `name` dans la liste de la migration (`CHECK`) ou validé par le serveur selon le choix noté dans la fiche.

### 5.2 Noyau métier
1. Module `platform::events` : énumération Rust `ProductEvent` (une variante par événement de la liste blanche d'`ARCHITECTURE.md` §8.4) ; propriétés typées, aucun champ texte libre ; fonction `emit(tx, event)` utilisée par les gestionnaires des étapes suivantes.
2. Événements de cette étape : `screen_viewed`, `validation_error`, `permission_denied`, `client_error`, `slow_response`, `login_failed`, `session_expired`, `password_reset_requested`, `feedback_submitted`. Chaque étape suivante ajoute les siens (règle ajoutée à la définition de terminé).
3. Suivi d'erreurs : toute erreur serveur non prévue est agrégée par empreinte avec l'identifiant de requête, l'organisation, la route, la version, la pile d'appels, sans corps.
4. Signalement : stockage, référence lisible, notification immédiate sur le canal d'alerte (sans donnée personnelle dans le message).

### 5.3 API
`POST /events` (20 événements et 8 Kio au plus, `204` même si certains sont écartés), `POST /client-errors` (`204`, aucun texte d'écran), `POST /feedback` (`201 { id, reference }`, contexte : écran, chemin **sans paramètre de requête ni jeton**, version du front, taille d'écran, cinq derniers identifiants de requête, dernier code d'erreur ; pas de capture d'écran). Limites de débit par session ; droits `events.create` et `feedback.create` pour tout utilisateur connecté.

### 5.4 Interface
Client `track()` qui envoie les événements en petits lots à `POST /events` sans cookie supplémentaire ; capture de `window.onerror` et des rejets non traités vers `POST /client-errors` ; bouton « Signaler un problème » dans le menu utilisateur et la feuille « Plus » ; formulaire accessible (étiquettes liées, résumé des erreurs) ; toast de confirmation avec la référence ; bandeau si la réponse est refusée (limite). Aucune bibliothèque tierce de mesure.

### 5.5 Documents PDF
Aucun.

### 5.6 Audit et événements
Les signalements ne passent pas par l'audit ; le contenu libre n'est jamais journalisé. `ops/sql/indicateurs.sql` : requêtes fixes (temps médian pour encaisser, part des paiements saisis sous 48 h, abandons, erreurs par type, signalements ouverts) sur les événements, prêtes pour les étapes suivantes ; les organisations `demo` et `test` sont exclues par `kind = 'real'`.

### 5.7 Tests
- Une propriété textuelle (chaîne libre, numéro `+226`, nom) dans un événement est refusée à la compilation (types) et, côté API, l'événement est écarté et compté.
- A13 : aucun numéro, jeton ni description de signalement dans les journaux d'un parcours complet.
- Isolation de `product_events`, `feedback_reports` (ISO-x) ; refus de `UPDATE`/`DELETE` sur `product_events`.
- E2E : formulaire de signalement sur 390 px, erreur serveur simulée, erreur JavaScript provoquée en essai.
- Limites : 21ᵉ événement d'un lot écarté ; corps de plus de 8 Kio refusé.

## 6. Critères d'acceptation vérifiables

1. Un signalement envoyé depuis un téléphone crée une ligne avec référence, contexte joint et alerte reçue sur le canal en moins d'une minute.
2. Aucun événement ni signalement ne contient de nom, de téléphone ni de jeton (test de recherche sur les lignes de la base et sur les journaux).
3. Un événement inconnu est écarté, compté, et la réponse reste `204` ; en essai, il fait échouer le test.
4. Une erreur serveur provoquée en essai apparaît une fois dans `error_reports` avec son nombre d'occurrences.
5. `ops/sql/indicateurs.sql` s'exécute sur l'essai sans erreur et exclut l'organisation de démonstration.

## 7. Sécurité et déploiement à vérifier

Aucun service tiers, aucun cookie publicitaire ; contenu libre jamais journalisé ni exporté dans les événements ; conservation et accès documentés (développeur et porteur du projet) ; mention de la mesure d'usage prévue dans l'engagement de confidentialité (C-05, `SECURITY.md` §8.1).

## 8. Liste de démonstration (Mimis)

1. Sur un téléphone, ouvrir le menu, toucher « Signaler un problème », décrire un défaut fictif, envoyer : lire la référence affichée.
2. Voir l'alerte arriver sur le canal de l'équipe, et la ligne dans la base (le développeur la montre).
3. Provoquer une erreur de test sur l'essai : voir une seule ligne « 1 occurrence », puis « 2 occurrences » au deuxième essai.
4. Ouvrir le résultat des requêtes d'indicateurs : voir que seuls les écrans consultés apparaissent, sans nom ni numéro.

## 9. Définition de terminé, risques, hors étape

Terminé : critères 1 à 5 verts ; `API_CONTRACTS.md` §9 mis à jour ; liste blanche versionnée ; `DECISIONS.md` (n° 28, 66) mis à jour ; règle « événements de l'écran » ajoutée à la liste de contrôle de PR.

| Risque | Mesure |
|---|---|
| Données personnelles glissées dans les propriétés | Types fermés, test de recherche, revue de toute nouvelle variante |
| Volume d'événements sur un réseau lent | Lots de 20 au plus, envoi différé, aucune attente de réponse |
| Signalements non traités | Alerte immédiate et revue hebdomadaire (étape 19) |

Hors étape : tableau de bord riche (requêtes SQL fixes suffisent, `ARCHITECTURE.md` §8.4), console des signalements (étape 11, commande), événements des écrans d'argent, de relance et de compte rendu (étapes 07, 15, 17).

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 04
- Dépend de : 02
- Tranche : T1
- Jalon : J3
- F : F39
- R : aucune
- E : aucun
- PP : PP-28
- Contenus : aucun
