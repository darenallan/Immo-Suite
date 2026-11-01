# Étape 02 — Rapport de tests et de démonstration (01/11/2026)

Étape : « Organisations, connexion et squelette déployé » (`roadmap/etape-02-organisations-connexion-squelette.md`). Fiches F01, F02, F03 validées par Mimis le 01/11/2026 ; décisions DP-85, DP-86, DP-87.

**Verdict : tout ce qui se vérifie sur le poste est vert. Un critère (n° 10, mise en production puis retour arrière) et une partie de la démonstration exigent l'essai et la production déployés : ils ne sont PAS vérifiés** (dette de l'étape 01 : C-01, C-02, C-03).

## Résultats réels (aucun chiffre estimé)

| Famille | Résultat | Commande |
|---|---|---|
| Rust : domaine, application, serveur, base | **174 réussis, 0 échec** | `cargo test --workspace --locked` |
| Rust : clippy `-D warnings`, `cargo fmt --check` | verts | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| `cargo audit`, `cargo deny check` | verts (deux exceptions datées, DP-87 n'y change rien) | voir `deny.toml` |
| Front : typage, ESLint (`jsx-a11y`, `react-hooks`), Prettier | verts | `npm run typecheck`, `lint`, `format:check` |
| Front : tests unitaires et de composants | **21 réussis** | `npm test --prefix web` |
| Front : poids (compressé) | JS 63,0 Ko / 150, CSS 2,0 Ko / 30, total 65,2 Ko / 200 | `npm run size --prefix web` |
| Navigateur, API simulée, 390 px et 1440 px | **12 réussis** | `npm run test:e2e --prefix web` |
| Navigateur sur la **pile réelle** (serveur Rust, PostgreSQL, front), 390 px et 1440 px | **10 réussis** (dont l'expiration d'inactivité réelle) | `powershell -File ops/dev/real-stack.ps1 -ShortIdle` |

## Critères d'acceptation (étape 02, §6)

| # | Critère | État | Preuve |
|---|---|---|---|
| 1 | Identifiant inconnu et mot de passe faux : même réponse, temps comparable | **vérifié** | A1 : corps identiques, rapport des durées médianes < 1,6 (`tests/db/tests/api.rs`) |
| 2 | 5 échecs, la 6ᵉ reçoit 429 avec `Retry-After` | **vérifié** | A2 + `login.rs` (attentes 1, 5, 15 minutes) + pile réelle (message de limitation du vrai serveur) |
| 3 | Cookie `__Host-session`, `Secure`, `HttpOnly`, `SameSite=Lax`, sans `Domain` | **vérifié** | test d'attributs ; sur la pile réelle, lu dans le navigateur (`context.cookies()`), invisible de `document.cookie` |
| 4 | Autre organisation : 404, aucune donnée, ligne d'audit chez l'appelant, alerte | **vérifié** | A4 (HTTP) et `login.rs` ; alerte = ligne de journal `alert=security.cross_org_attempt` (le canal reste la dette n° 5 de l'étape 01) |
| 5 | `UPDATE`, `DELETE`, `TRUNCATE` sur `audit_log` échouent pour les 3 rôles | **vérifié** | G-05 : 9 cas (`immo_app`, `immo_platform` par les droits ; `immo_owner` par le déclencheur) |
| 6 | Écriture d'audit en échec : action annulée (R-008) | **vérifié** | test de connexion avec `INSERT` retiré : ni session, ni tentative réussie enregistrées |
| 7 | Test rôle × action : 100 % des couples, refus par défaut | **vérifié** | **175 couples sur 175, 0 écart**, lus dans le tableau de `API_CONTRACTS.md` §4.6 ; 17 entrées de menu × 7 rôles = 119 cellules lues dans `UI_UX_DESIGN.md` |
| 8 | Sans contexte : 0 ligne ; 0 lecture croisée sur 10 000 itérations de pool | **vérifié** | ISO-1 ; PP-10 : 50 tâches × 200 itérations sur 2 connexions, tables réelles, 0 violation |
| 9 | En-têtes de sécurité sur chaque type de réponse | **vérifié (local)** | A12 : réponses 200, 401, 404, 422, 429, sondes ; 9 en-têtes + CSP + `X-Request-Id` |
| 10 | Mise en production de la même image puis retour arrière, procès-verbal | **NON VÉRIFIÉ** | exige l'essai et la production (C-01, C-02, C-03) ; scripts `ops/deploy/*.sh` écrits, jamais exécutés |

Autres exigences de l'étape : A13 (aucun numéro `+226`, mot de passe, jeton ni e-mail dans les journaux, à tous les niveaux, parcours complet) **vérifié** ; parité `Clock` / base de la date locale (DP-50) **vérifiée** ; registre des routes égal au contrat OpenAPI et toute route protégée refuse sans session **vérifié** ; aucune route d'inscription (A6) **vérifié** ; `immo user create` refusé en production **vérifié** (test et pile réelle).

## Liste de démonstration (Mimis)

| # | Étape | État |
|---|---|---|
| 1 | Créer une organisation et un gérant (`immo org create`, `immo user create`) | **fait en local** (`ops/dev/real-stack.ps1`) ; **pas sur l'essai** (inexistant) |
| 2 | Téléphone Android sur l'adresse de l'essai : mauvais mot de passe, puis bon : accueil du gérant | **fait dans Chrome à 390 px sur la pile locale** ; **pas sur un téléphone ni sur l'essai** |
| 3 | Déconnexion, reconnexion, expiration d'inactivité raccourcie | **fait en local** (expiration réelle après 1 minute) |
| 4 | Rapport d'isolation et de droits « 7 rôles × 25 actions, 0 écart » | **ce document** (175 couples, 0 écart) |
| 5 | Procès-verbal de mise en production puis de retour arrière | **NON FAIT** (essai et production inexistants) |

Pour rejouer la démonstration à la main : `powershell -File ops/dev/real-stack.ps1 -KeepStack` laisse la pile en marche (`http://127.0.0.1:4173`, comptes `70 12 34 56` gérant et `70 12 34 57` caissier, mot de passe de démonstration `un mot de passe correct`, base jetable) ; l'arrêt se fait par `task.ps1 stop -Name real-api` puis `-Name real-web`.

## Écarts et points à décider

- **Calibrage argon2id (décision n° 48)** : 19 ms avec le réglage de départ sur ce PC, hors de la cible de 100 à 500 ms ; réglages plus coûteux mesurés. Valeur **non changée** en attendant le serveur retenu (DP-87).
- **Écarts d'implémentation** consignés dans DP-87 et `DATABASE_SCHEMA.md` §6.11 (plafond général en mémoire, `own_memberships` sans organisation active, etc.).
- **`--prefix`** de `immo org create` retiré (DP-85) : écart à noter dans `ROADMAP.md`.
- **Second avis sur la migration de garde (`audit_log`)** : `ROADMAP.md` (C-19) demande une personne désignée tant qu'un seul développeur écrit le code. **Non fait** : à désigner par Mimis avant la promotion en production.
- **Incidents d'outillage** : (1) disque du poste plein pendant une compilation (cache de compilation supprimé, profil allégé) ; (2) `task.ps1 stop` plantait quand le numéro d'un superviseur était réutilisé par un processus protégé : corrigé (`Alive`, identité non prouvée = on ne touche à rien).
