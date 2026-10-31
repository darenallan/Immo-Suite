# Immo Suite — CLAUDE.md

| Champ | Valeur |
|---|---|
| Statut | Point d'entrée de tout agent : index, propriétaires par sujet, légende, règles d'or, commandes. Version 0.3, 31/10/2026 |
| Dépend de | `DECISIONS.md` (état au 31/10/2026), `CONTEXTE.md`, `ARCHITECTURE.md`, `CODING_RULES.md` |

## Projet

- Immo Suite : logiciel web de gestion locative (téléphone et ordinateur) pour propriétaires et agences du Burkina Faso. Habitation et commercial. Devise FCFA, langue française.
- Éditeur : HABEK Technology Company (DP-63). Nom du produit provisoire, domaine non choisi (décision n° 13).
- Deux plans, une base de code : « Propriétaire » et « Agence ». Le produit est un livre de comptes locatifs : encaisser, quittancer, relancer, rendre compte au mandant.
- Capacité : un développeur, environ 25 heures par semaine, assisté par Claude Code (DP-54). Base de données : PostgreSQL chez Neon (DP-55).
- Aucune enquête terrain n'a eu lieu et aucune n'est prévue avant le développement : prix, volumes et workflows sont des hypothèses validées pendant le pilote [À valider].

## Phase actuelle

**Construction de la première version (T1 puis T2)**, testée par des agences pilotes avec leurs données réelles (DP-52, DP-53). Le développement démarre par la tranche verticale (`ARCHITECTURE.md` §9.1) dès que les décisions bloquantes de `TESTING_QA.md` PP-32 sont enregistrées. La maquette (`maquette/`) reste une démonstration. Il n'y a pas encore de code applicatif. Le dépôt Git existe en local (DP-73) ; les prototypes jetables de l'étape 00 sont dans `spikes/` (hors build). Critère « prêt pour un pilote » : `TESTING_QA.md` §12 (PP-01 à PP-38), liste unique.

## Roadmap (ROADMAP.md, proposition à valider)

- Plan en 24 étapes : `ROADMAP.md` (jalons, dépendances, estimation, avancement) et `roadmap/etape-NN-*.md` (autoportants). **Étape 00 terminée le 31/10/2026** (décisions DP-76 à DP-81 ; la mesure réseau (e) est reportée, DP-75). **Étape courante : 01** (tableau d'avancement de `ROADMAP.md` §4 ; le mettre à jour en fin d'étape).
- Résultats de l'étape 00 : `spikes/*/NOTE.md` (section 8 de chaque note) et `DECISIONS.md` (DP-69 à DP-81). `spikes/REPRISE.md` garde l'état du poste. **Outillage des tâches : `spikes/tools/README.md`** (toute commande longue passe par `spikes/tools/task.ps1`, jamais `python -` ni `python` nu).
- Démarrer une étape : « Démarre l'étape NN. Lis CLAUDE.md, puis `roadmap/etape-NN-<nom>.md`, puis uniquement les sections qu'il cite. Applique la procédure de `ROADMAP.md` §8 » : fiches `specs/` d'abord, arrêt pour validation, tests d'argent avant le code, démonstration, validation avant l'étape suivante.
- Les décisions n° 96, 97 et 103 sont prises (DP-72) ; les n° 98 à 102 restent ouvertes : ne pas les appliquer avant leur entrée DP-xx.

## Qui fait foi (propriétaire par sujet)

En cas de conflit, le document propriétaire du sujet fait foi. `DECISIONS.md` et le dossier de cadrage prévalent sur tous. Si deux propriétaires s'opposent, signaler le conflit, ne pas trancher.

| Sujet | Propriétaire | Quand le lire |
|---|---|---|
| Décisions prises et ouvertes, stratégie, écarts au dossier | `DECISIONS.md` | Avant toute décision structurante |
| Périmètre, tranches, rôles, matrice de permissions, règles d'or | `CONTEXTE.md` | Au début de chaque tâche |
| Règles métier R-xxx, cas E-xx, glossaire | `DOMAINE.md` | Avant un calcul, un statut, une règle d'argent |
| Écrans, composants, textes, tokens, accessibilité, performance | `UI_UX_DESIGN.md` | Avant un écran ou un texte |
| Jeu de données et usage de la maquette | `MAQUETTE.md`, `maquette/NOTES.md` | Pour la maquette seulement |
| Pile, modules, règles d'architecture, prototypes, ordre de construction | `ARCHITECTURE.md` | Avant d'écrire du code applicatif |
| Tables, colonnes, contraintes, déclencheurs, droits SQL | `DATABASE_SCHEMA.md` | Avant une migration ou une requête |
| Authentification, menaces, données sensibles, conservation | `SECURITY.md` | Avant les droits ou les données sensibles |
| Routes, formats d'erreur, en-têtes | `API_CONTRACTS.md` | Avant d'ajouter une route |
| Règles de code, revue, dépôt, travail avec Claude Code | `CODING_RULES.md` (§9 : Claude Code) | Avant d'écrire du code |
| Tests, gravités S1 à S4, liste « prêt pour un pilote » | `TESTING_QA.md` | Avant d'écrire ou lancer des tests |
| Environnements, coûts, calendrier de livraison, sauvegardes, accueil d'une agence | `DEPLOYMENT.md` | Avant un déploiement ou un incident |

Autres fichiers : `Immo-Suite_Dossier-de-cadrage.docx` (périmètre d'origine, **ne jamais le modifier**), `AUDIT_COHERENCE.md` et `AUDIT_COHERENCE_2.md` (audits), `TRACABILITE.md` (matrices fonction → écran → route → table → test et règle → test, générées), `ROADMAP.md` et `roadmap/` (créés le 30/10/2026), `specs/` (à créer, une fiche par fonctionnalité avant son code), `contenus/` (à créer : engagement de confidentialité, accord de pilote, bons usages, certificat de destruction, plan d'incident, guide d'accueil d'une agence, lettre d'intention).

Légende des marqueurs : **[PILOTE]** indispensable avant de confier de vraies données à une agence ; **[APRÈS]** peut venir pendant ou après le pilote ; **[À valider]** choix que le porteur du projet doit trancher ; **[Hypothèse]** valeur supposée, à mesurer ; **[À VÉRIFIER AVEC UN JURISTE]** point juridique (DP-62) ; **[à confirmer avec l'agence pilote à la découverte]** règle métier à vérifier au premier pilote ; **[À vérifier]** fait technique à confirmer.

## Règles d'or

Les 12 règles complètes sont dans `CONTEXTE.md` (source unique). Rappel des plus critiques :

1. Montants en entiers FCFA. Jamais de décimale ni de flottant (règle 6, R-001).
2. Un paiement est immuable. On le corrige par contre-passation, jamais par modification ni suppression (règle 4, R-047, R-048).
3. Quittances numérotées sans trou, par organisation, série et année (règle 7, R-050).
4. Isolation par organisation : `org_id` partout (règle 8, R-004). Un test d'isolation manquant bloque le build.
5. Jamais de détention de fonds de clients (règle 3).
6. Journal d'audit en écriture seule. Rien ne se supprime : archivage (règles 9 et 10, R-005, R-007), sauf les exceptions de DP-60.
7. Ne jamais inventer une règle métier. La demander, ou l'ajouter aux « Questions ouvertes » de `DOMAINE.md`.
8. Pas de fonction hors du périmètre de `CONTEXTE.md`, pas d'IA au MVP (règle 11).

## Comportement attendu

- Citer les identifiants R-xxx, E-xx, DP-xx et F-xx dans le code, les tests et les commits. Mettre à jour `DECISIONS.md` **avant** d'appliquer une décision structurante.
- Une recommandation du dossier de cadrage n'est pas une décision tant qu'elle n'a pas d'entrée DP-xx. Conserver les marqueurs partout où ils s'appliquent.
- En cas de doute sur un périmètre, une priorité ou une tranche, demander. Ne pas écraser un fichier existant sans demander.
- Pour la maquette : aucune logique réelle, aucune ressource externe ; jeu de données vérifié par `node maquette/scripts/verifier-donnees.js` (28 contrôles).

## Conventions

- Interface et documentation : français, vouvoiement (provisoire, décision n° 17). Identifiants de code (variables, fonctions, tables, colonnes, routes, codes d'erreur) : anglais (DP-67).
- Formats : « 1 250 000 FCFA », JJ/MM/AAAA, +226 70 12 34 56, fuseau Africa/Ouagadougou. Palette bleu nuit / blanc / turquoise (DP-44) : tokens dans `UI_UX_DESIGN.md`.
- Commits : un sujet clair avec l'identifiant DP-xx ou F-xx (`CODING_RULES.md` §8.1).

## Pile et commandes (ARCHITECTURE.md, DP-55, DP-56)

- **Stack** : monolithe Rust (Axum, Tokio, SQLx) + PostgreSQL chez Neon (RLS forcée, rôles créés en SQL sans `BYPASSRLS`) + stockage S3 ; front React + Vite + TypeScript (PWA, sans cache d'API) ; PDF : moteur non choisi, prototype (a) d'abord (n° 7).
- **Poste prévu** : Rust stable, Docker (PostgreSQL local), `sqlx-cli`, Node LTS (Node 24 constaté). **Poste réel au 31/10/2026** : Rust 1.99.1 en toolchain **GNU** (pas de MSVC), `sqlx-cli` 0.9, Typst 0.15.1, Node 24, Chrome ; **ni Docker ni WSL** : un PostgreSQL 18 local jetable écoute sur le **port 5433** (lancé et arrêté par `task.ps1`, voir `spikes/REPRISE.md`) ; le PostgreSQL de Mimis (port 5432) n'est **jamais** touché ; Neon (plan gratuit, DP-69) sert aux essais en ligne. Les commandes `docker compose` ci-dessous sont à remplacer par ce PostgreSQL local tant que Docker est absent.
- **Commandes prévues** : `cargo build`, `npm ci --prefix web` · `docker compose up -d db`, `cargo run -p server -- serve`, `npm run dev --prefix web` · tests `cargo test` (règles d'argent : `cargo test -p domain`), `npm test --prefix web`, `npm run test:e2e --prefix web` (Playwright, 390 et 1440 px) · lint `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `npm run lint --prefix web` · dépendances `cargo audit`, `cargo deny check`, `npm audit --prefix web` · migrations `sqlx migrate run`, `cargo sqlx prepare --check` · contrat d'API : `openapi.json` régénéré et comparé en CI · poids `npm run size --prefix web` · démo `cargo run -p server -- demo reset`.
- **Exploitation** (`DEPLOYMENT.md`) : `immo org create`, `immo org go-live`, `immo org export <org>`, `immo org purge <org>` (phase `setup` seulement), `immo anonymize` (jamais en production), `immo demo reset`. Cinq requêtes de contrôle avant toute migration : `DATABASE_SCHEMA.md` §7.5.
- **Dépôt** : `crates/domain` (règles pures), `crates/app`, `crates/server`, `migrations/`, `web/`, `templates/`, `seeds/demo/`, `tests/fixtures/`, `tests/contract/`, `spikes/`, `ops/` (`ARCHITECTURE.md` §3.4, `CODING_RULES.md` §1.1).
- **Variables** : `APP_ENV`, `PUBLIC_BASE_URL`, `DATABASE_URL` (rôle `immo_app`), `MIGRATION_DATABASE_URL` (rôle `immo_owner`), `S3_*`, `SESSION_SECRET`, clé de chiffrement des champs (copie hors ligne). Clé de sauvegarde : `DEPLOYMENT.md` §5.2. Aucun secret dans le dépôt.
- **Environnements** : développement (local), essai (données anonymisées, n° 15), production (pilotes + organisation de démonstration, n° 26).
- **Non négociable** : jamais `&PgPool` hors `platform` (toujours `OrgTx`), jamais d'`UPDATE`/`DELETE` sur une écriture, jamais de séquence pour les quittances, migrations en avant seulement et rétrocompatibles.
