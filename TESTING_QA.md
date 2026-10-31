# Immo Suite — Tests et assurance qualité

| Champ | Valeur |
|---|---|
| Statut | Propriétaire des tests, des gravités S1 à S4, des délais de correction cibles et de la liste unique « prêt pour un pilote » (§12). Les outils de test n° 78 à 83 et 85 restent proposés [À valider] ; la n° 80 (tests sur PostgreSQL réel) est retenue par DP-56 |
| Date | 30/10/2026 |
| Dépend de | `DECISIONS.md` (état au 30/10/2026), dossier de cadrage v1.0 (§9, §14, §19, §20.2, §23), `ARCHITECTURE.md` v0.2 (§5.4, §7, §8.5, §9.3, §11.3, §12), `DATABASE_SCHEMA.md` v0.2 (§4, §6.8, §7.5, §9), `SECURITY.md` v0.2 (§10, §11), `API_CONTRACTS.md` v0.2, `CODING_RULES.md` v0.2, `DOMAINE.md` (R-001 à R-110, E-01 à E-48), `CONTEXTE.md`, `UI_UX_DESIGN.md` (quittance, accessibilité, performance), `MAQUETTE.md` (parcours, jeu de données, vérificateur) |
| Alimente | `DEPLOYMENT.md`, `CODING_RULES.md`, `CLAUDE.md` |
| Version | 0.2 |

Légende des marqueurs ([PILOTE], [APRÈS], [À valider], [Hypothèse], [À VÉRIFIER AVEC UN JURISTE]) : voir `CLAUDE.md`.

Hiérarchie en cas de conflit : voir `CLAUDE.md` (hiérarchie par sujet). Ce document ne contredit aucun autre.

Principe : une agence pilote saisit de vrais loyers. **Une erreur sur un montant ou une fuite entre organisations n'est jamais « mineure »** : elle est bloquante, et le système de tests existe d'abord pour qu'elle n'atteigne pas un pilote. Le reste (confort, mise en page, performance fine) se corrige en roulant.

Indispensable avant le premier pilote :

- **[PILOTE]** Tests d'or des règles financières (E-01 à E-48 applicables à T1 et T2) et tests de propriétés du livre des encaissements.
- **[PILOTE]** Tests de base de données : isolation sur toutes les tables, immutabilité, contraintes, migrations sur copie.
- **[PILOTE]** Un test d'isolation et un test de droits par route ; tests d'idempotence et de concurrence ; test de contrat.
- **[PILOTE]** Tests de bout en bout des cinq parcours (ordinateur et 390 px) ; tests visuels de la quittance ; tests d'import avec fichiers imparfaits.
- **[PILOTE]** Mesure du temps pour encaisser sur un téléphone d'entrée de gamme ; contrôles de poids et d'accessibilité.
- **[PILOTE]** Première restauration chronométrée, liste « prêt pour un pilote » entièrement vérifiée (section 12).
- **[APRÈS]** Tests de charge de début de mois, analyse dynamique des pages, test d'intrusion externe (avant la version commerciale), couverture de code mesurée.

## 1. Stratégie

### 1.1 Pyramide adaptée à un développeur

Peu de tests lents et fragiles, beaucoup de tests rapides sur ce qui coûte cher en cas d'erreur. Les règles d'argent vivent dans `crates/domain` (pures, sans base) : c'est là que la masse de tests se trouve.

| Niveau | Cible | Part de l'effort [Hypothèse] | Durée d'exécution | Outil (candidat) |
|---|---|---|---|---|
| 1. Tests d'or et unitaires (`domain`) | règles d'argent, montant en lettres, statut d'échéance, commission, numéro | ~35 % | secondes | `cargo test` |
| 2. Tests de propriétés (`domain`, livre) | invariants : somme des affectations, soldes, contre-passation | ~15 % | secondes à minutes | `proptest` (décision n° 79) |
| 3. Tests de base de données | isolation, immutabilité, contraintes, gardes, migrations | ~20 % | minutes | `#[sqlx::test]` sur PostgreSQL réel (décision n° 80) |
| 4. Tests d'API et de contrat | route par route : droits, isolation, idempotence, schéma | ~20 % | minutes | tests d'intégration Rust sur PostgreSQL réel |
| 5. Tests de bout en bout | cinq parcours, ordinateur et mobile ; authentification | ~5 % | 5 à 10 minutes | Playwright (décision n° 78) |
| 6. Tests visuels et de document | PDF de la quittance et du compte rendu | transversal | minutes | extraction de texte, rendu en image |
| 7. Performance et usage réel | poids, LCP, temps pour encaisser | transversal | minutes | contrôle de taille, Lighthouse, chronométrage |
| 8. Tests à la main | exploration, impression papier, lecteur d'écran, recette pilote | | | |

### 1.2 Ce qu'on automatise obligatoirement, ce qu'on teste à la main

| À automatiser (bloque la fusion ou la livraison) | À tester à la main |
|---|---|
| Règles d'argent (E-xx), propriétés du livre | Impression réelle d'une quittance A4 et A5 sur une imprimante laser ordinaire, en noir et blanc (mensuel et avant chaque changement de gabarit) |
| Isolation par route et par table, droits rôle × action, immutabilité | Parcours complet sur un **téléphone Android d'entrée de gamme réel**, au soleil si possible |
| Idempotence, concurrence, numérotation sans trou | Lecteur d'écran sur le parcours Encaisser (TalkBack) |
| Contrat d'API, format d'erreur, en-têtes de sécurité | Lecture critique des textes et de la microcopy en français (orthographe, ton) |
| Migrations sur base vide et sur copie anonymisée | Séances de recette avec les agences (section 13) |
| Cinq parcours de bout en bout, poids des pages, accessibilité automatique | Test d'intrusion par la personne qui n'a pas écrit le module (deux jours, avant le pilote) |
| Import avec fichiers imparfaits, fichiers piégés | Contrôle des contrastes avec un outil reconnu (les calculs de la maquette sont faits par formule : `maquette/NOTES.md`) |
| Journaux sans donnée personnelle | Lecture des alertes et du plan d'incident lors de l'exercice sur table |

### 1.3 Règles de conduite des tests

- **Les tests de base tournent avec le même rôle que la production** : `immo_app` (non propriétaire, sans `BYPASSRLS`). Un test exécuté avec un super-utilisateur ne prouve **rien** sur l'isolation (la sécurité au niveau des lignes serait contournée). Les trois rôles (`immo_owner`, `immo_app`, `immo_platform`) sont créés par la préparation des tests comme en production.
- **Aucun nouvel essai automatique sur les tests d'argent et d'isolation** : un test qui échoue de temps en temps sur ces sujets signale une vraie course critique, pas un « test instable ». Il est traité comme une anomalie bloquante (section 11).
- **Pas de simulation de la base** pour les tests d'intégration : PostgreSQL réel, en conteneur (le dossier §19 : « Base de test réelle »).
- **Horloge fixée** au vendredi 13/11/2026 09:15 (`Clock` et `app_now()`), sauf test de changement de jour.
- **Jamais de donnée réelle** (section 10).

## 2. Tests des règles financières

### 2.1 Tests d'or (golden)

Les cas de `DOMAINE.md` sont rejoués **à l'identique** : mêmes montants, mêmes résultats. Ils sont écrits **avant** le code (`CODING_RULES.md` §9). Chaque cas vit dans `tests/fixtures/golden/E-xx.json` (entrées, événements, état attendu) et est exécuté par le crate `domain` puis, quand c'est pertinent, par la base et l'API : un même cas, plusieurs niveaux.

Trois cas d'ancrage, toujours verts :

| Cas | Entrée | Résultat exigé |
|---|---|---|
| **E-05** paiement partiel | loyer dû 75 000 ; paiement de 50 000 en espèces | échéance `partial` avant J+1, 50 000 affectés, **25 000 restants** ; la quittance porte « solde restant 25 000 » ; à J+1, statut `late`, payé 50 000, reste 25 000, présente au tableau des impayés (E-25) |
| **E-06** avance | loyer 75 000 ; paiement de 100 000 | 75 000 affectés au loyer, **25 000 d'avance** affectés à l'échéance suivante ; sans échéance suivante générée, 25 000 de crédit du bail (E-31) |
| **E-18** compte rendu | 400 000 encaissés ; commission 8 % ; dépenses imputées 30 000 | commission **32 000**, net à reverser **338 000** ; le même relevé validé est **identique** à l'export PDF (R-074) |

```rust
// R-044 (E-05) : un paiement partiel laisse un reste dû
#[test]
fn partial_payment_leaves_remaining_balance_e05() {
    let lines = vec![open_rent_line("2026-11-13", Fcfa::new(75_000).unwrap())];
    let split = allocate(Fcfa::new(50_000).unwrap(), &lines);
    assert_eq!(split.allocated_total(), Fcfa::new(50_000).unwrap());
    assert_eq!(split.remaining_of(&lines[0]), Fcfa::new(25_000).unwrap());
}
```

*(Extrait d'exemple : les noms de fonctions sont indicatifs ; `unwrap` est permis dans les tests.)*

Correspondance complète des cas (niveaux : **U** unitaire `domain`, **P** propriété, **D** base de données, **A** API, **E** bout en bout) :

| Cas | Sujet | Niveaux | Tranche |
|---|---|---|---|
| E-01 | première échéance d'un bail activé | U, D | T1 |
| E-02 | avenant : mars à 75 000, avril à 80 000 | U, D | T1 |
| E-03 | génération idempotente de l'échéancier | D, A | T1 |
| E-04, E-37 | passage à « en retard » ; « due » le jour J0 (frontières 4, 5, 6) | U, P | T1 |
| E-05, E-25 | partiel, puis en retard (DP-13) | U, D, A, E | T1 |
| E-06, E-31 | avance, crédit du bail | U, D, A | T1 |
| E-07 | double clic : un seul paiement | A, D, E | T1 |
| E-08 | modification d'un paiement validé refusée, demande de correction | A, D | T1 |
| E-09 | confirmation au-delà du seuil | A, E | T1 |
| E-10, E-27 | numéros consécutifs ; remise à 1 au changement d'année | D, P | T1 |
| E-11 | montant en lettres | U, P | T1 |
| E-12 | lien de quittance expiré refusé | A | T1 |
| E-13, E-13b | tranches d'ancienneté (frontières 30, 31, 59, 60) | U, P | T1 |
| E-14 | texte de relance : nom, montant, mois exacts | U, A | T1 |
| E-15 | promesse non tenue : alerte visible dans l'accueil | U, A | T1 |
| E-16, E-26, E-28 | commission 8 %, arrondi 7,5 %, base loyer ou loyer+charges | U, P, D | T2 |
| E-17 | avenant de taux non rétroactif | D, A | T2 |
| E-18, E-19, E-29, E-30 | relevé ; paiement après validation ; dépense selon le payeur ; réouverture | U, D, A, E | T2 |
| E-20, E-21 | import de 200 lignes ; annulation | D, A, E | T2 |
| E-22 | dépôt de garantie > 2 mois : avertissement non bloquant | A | T1 |
| E-32 | ordre d'affectation à date égale (loyer, charges, exceptionnelles) | U, P | T1 |
| E-33 | paiement par un tiers : mention sur la quittance | A, PDF | T1 |
| E-34 | saisie après clôture de caisse | D, A | **[APRÈS] T3** |
| E-35 | renouvellement par avenant de prolongation | D, A | T1 |
| E-36 | bail refusé sur lot en travaux | A | T1 |
| E-38 | contre-passation, quittance annulée, rectificative suivante | D, A, E, PDF | T1 |
| E-39, E-40 | jour d'échéance 31 en mois court ; premier mois partiel (R-098, R-099) | U, D | T1 |
| E-41, E-43 | promesse tenue ou non ; taux de recouvrement (R-101, R-103) | U, P | T1 |
| E-42 | doublon de référence : avertissement bloquant, référence d'un paiement contre-passé réutilisable (R-102) | A, D | T1 |
| E-44 | avenant saisi après une échéance échue (R-104) | U, D | T1 |
| E-45, E-46 | charges sous base loyer ; reversement exact (R-105, R-108) | U, D, A | T2 |
| E-47, E-48 | remise à zéro en `setup` ; série d'essai puis série officielle (R-110) | D, A | T1 |
| E-23, E-24 | exemples de prix (agence de 60 lots, propriétaire de 15 lots) | — | **non testés** : arithmétique commerciale, hors produit |
| *F18, F20, F22, F23, F25, F17* | exports (neutralisation des formules, PP-19) ; préavis et restitution du dépôt (R-066, R-067) ; notifications ; renouvellement (R-090) ; lecture du SMS (fichier de cas réels) ; agenda | — | **[APRÈS] T3** : cas à écrire avec la règle, avant le code de chaque lot (`API_CONTRACTS.md` §7.19) |

**Couverture des règles.** La matrice « règle → test » (R-001 à R-110, avec trois états : test cité, couverte par famille, à écrire) est dans `TRACABILITE.md`, générée depuis les tables de ce document ; elle n'est pas recopiée ici pour n'avoir qu'une seule source.

### 2.2 Tests de propriétés sur le livre des encaissements

Générateurs de scénarios aléatoires (baux, échéances, paiements, contre-passations) ; chaque propriété doit tenir sur des milliers de cas. Le livre est celui des **encaissements locatifs**, pas un grand livre comptable (`ARCHITECTURE.md` §1.2).

| # | Propriété | Règles |
|---|---|---|
| P1 | La somme des affectations d'un paiement ne dépasse jamais son montant | R-044 |
| P2 | Pour tout paiement, `montant = somme des affectations + part non affectée (crédit du bail)`, et la part non affectée n'est jamais négative | R-046, R-086 |
| P3 | **Somme des écritures** : pour un bail, la somme des paiements nets (paiements + contre-passations) égale la somme des affectations nettes plus le crédit du bail | R-044, R-086 |
| P4 | Le solde d'une ligne (`dû − affecté`) n'est jamais négatif et jamais supérieur au dû | R-033, R-057 |
| P5 | **Contre-passation** : enregistrer un paiement puis sa contre-passation ramène chaque ligne d'échéance à l'état antérieur (affectations nettes nulles par ligne) | R-047, R-048 |
| P6 | Un paiement n'est contre-passé qu'une fois ; une contre-passation a un montant opposé | R-048 |
| P7 | L'ordre d'affectation est déterministe et suit la date, puis loyer, charges, exceptionnelles, quelle que soit la façon dont l'entrée est ordonnée | R-043, R-087 |
| P8 | Idempotence : rejouer N fois la même requête avec la même clé produit exactement un paiement | R-042 |
| P9 | Statut d'une échéance : fonction du seul triplet (dû, affecté, aujourd'hui) ; passe de `upcoming` à `due` à J0 puis `late` dès J+1 si le solde est non nul ; jamais `late` avec solde nul | R-032, R-033, R-095 |
| P10 | Tranches d'ancienneté : chaque retard positif appartient à exactement une tranche ; les bornes 7/8, 30/31, 59/60 se placent comme DP-12 | R-058 |
| P11 | Commission : pour toute base ≥ 0 et tout taux de 0 à 10 000, `commission = ⌊(base × taux + 5 000) / 10 000⌋` ; `0 ≤ commission ≤ base` ; croissante avec la base ; aucun débordement | R-068, DP-14 |
| P12 | `net = encaissé − commission − dépenses ± ajustement` pour tout compte rendu ; un paiement figure dans **un seul** compte rendu | R-070, R-072 |
| P13 | **Numérotation** : quelle que soit la suite d'opérations et les annulations forcées, les numéros d'une organisation, d'une série et d'une année sont exactement 1 à N, sans trou ni doublon ; **aucune quittance sans numéro** ; chaque paiement de type paiement a exactement une quittance | R-049, R-050 |
| P14 | Montant en lettres : relecture par un analyseur de référence indépendant égale le nombre, de 0 à 999 999 999 | R-053 |
| P15 | Génération d'échéances : relancée plusieurs fois, aucune période en double ; la date d'échéance tombe dans le mois de la période | R-027, R-029 |
| P16 | Aucune valeur de montant produite par l'API n'est autre qu'un entier dans les bornes | R-001 |

Le nombre de cas par propriété : quelques centaines en CI à chaque PR, 10 000 et plus en nocturne.

### 2.3 Concurrence et idempotence

| Test | Scénario | Résultat exigé | Où |
|---|---|---|---|
| Numérotation sous charge (prototype c) | 20 clients × 500 validations concurrentes, 3 organisations, 10 % d'annulations forcées après attribution | zéro trou, zéro doublon, numéros consécutifs dans l'ordre de validation ; p95 d'une validation mesuré ; passage d'année (E-27) | nocturne et avant livraison ; version réduite (4 × 50) à chaque PR |
| Idempotence en parallèle | même clé envoyée 3 fois **simultanément** | un seul paiement, trois réponses identiques | à chaque PR |
| Idempotence après coupure | réponse perdue après validation ; nouvel essai ; coupure avant validation ; nouvel essai | pas de doublon ; pas de paiement manquant | à chaque PR |
| Même clé, contenu différent | deux montants avec la même clé | `422 idempotency.key_reused_with_different_body`, aucun effet | à chaque PR |
| Deux caissiers, même locataire | deux paiements simultanés sur le même bail | sérialisation par bail ; somme des affectations ≤ chaque paiement ; pas de dépassement du dû | à chaque PR |
| Verrou et niveau d'isolation | passage en `REPEATABLE READ` (doit être refusé en revue) | un test vérifie que la connexion applicative est en `READ COMMITTED` | à chaque PR |

## 3. Tests de base de données

Exécutés sur un PostgreSQL réel avec les trois rôles de production (1.3). Reprennent les identifiants G-xx et ISO-x de `DATABASE_SCHEMA.md` (44 tests déjà rédigés pour le schéma, section 9), à **rejouer** sur le PostgreSQL retenu (prototype b) ; ceux de concurrence n'ont pas encore été joués.

| Famille | Tests | Statut |
|---|---|---|
| **Isolation (ISO-1 à ISO-9)** | sans contexte : zéro ligne lisible, insertion refusée ; une organisation ne voit jamais l'autre et ses `UPDATE` touchent zéro ligne ; clé composite : un bail de A ne peut pas viser un lot de B ; `immo_app` ni super-utilisateur ni `BYPASSRLS` ; **toutes** les tables métier ont `org_id`, RLS activée **et forcée**, politique présente (liste blanche des tables globales) ; le contexte ne survit pas à la transaction (la variante de session **doit échouer** : test négatif) ; accès support refusé sans accord, accordé, révoqué, expiré, durée maximale 7 jours ; jeton public : une ligne de lien, rien d'autre ; les vues de soldes ne montrent rien d'une autre organisation | [PILOTE] |
| **Isolation sur toutes les tables** | un test parcourt `pg_class` : une table du schéma métier sans `org_id`, sans RLS forcée ou sans politique **fait échouer le build** ; une seconde passe insère une ligne dans chaque table sous l'organisation A et vérifie qu'elle est invisible sous B | [PILOTE] |
| **Pool sous concurrence** | 50 tâches alternent deux organisations sur un pool de 2 connexions, 10 000 itérations : aucune lecture croisée (prototype b) | [PILOTE] |
| **Immutabilité (G-05, ISO-4)** | `UPDATE`, `DELETE`, `TRUNCATE` refusés sur les 25 tables d'écriture pour `immo_app`, `immo_platform` **et** `immo_owner` (déclencheur) ; l'altération du déclencheur est détectée par le contrôle nocturne | [PILOTE] |
| **Exceptions encadrées** | `cancel_import_batch` refusé si un paiement, une relance ou une promesse référence un bail du lot ; `purge_org_setup` refusée hors état `setup` ; suppression directe refusée (E-20, E-21 du schéma) | [PILOTE] |
| **Contraintes** | un seul bail en cours par lot (**G-01**, G-02, y compris contre les baux terminés : décision n° 41) ; montants positifs, entiers et bornés 1 à 999 999 999 (G-03) ; cohérence des dates et refus d'un paiement futur (G-04, G-11) ; affectations bornées et jamais entre deux baux (G-12, G-13) ; contre-passation exacte (G-14) ; avenants non rétroactifs (G-10, G-18) ; dernier gérant conservé (G-16) ; téléphone unique parmi les fiches non archivées ; mélange comptes démonstration et réels refusé | [PILOTE] |
| **Numérotation** | deux paiements → 1 et 2 ; annulation après attribution → le numéro est rendu ; numéro hors séquence refusé ; compteur ramené en arrière ou avancé de 5 refusé ; passage d'année (G-07, G-08, G-09) ; requête de contrôle des trous = zéro ligne | [PILOTE] |
| **Horloge** | `app_now()` redéfinie en essai et test ; parité avec `Clock` ; une valeur d'horodatage envoyée par le client est ignorée (G-06) | [PILOTE] |
| **Arithmétique** | contraintes `statement_versions_commission` et `_net` : une version fausse est refusée ; piège `sum(bigint)` : la commission calculée en SQL n'est pas 8 001 mais 8 000 | [PILOTE] |
| **Rôle de sauvegarde** | `pg_dump` et `pg_restore` sous RLS forcée avec le rôle choisi (décision n° 46) | [PILOTE] |

### 3.1 Migrations sur copie de données (décision liée à `DATABASE_SCHEMA.md` §7)

À chaque migration, en CI et sur l'essai :

1. appliquer sur une **base vide** ; appliquer sur la **dernière copie anonymisée** de la production ;
2. exécuter les trois contrôles de schéma : audit RLS (ISO-6), présence et activation des gardes d'écriture seule, `purge_order` complet ;
3. exécuter les **cinq requêtes de contrôle** de `DATABASE_SCHEMA.md` §7.5 avant **et** après (aucun trou de numérotation, aucune affectation hors bornes, aucune contre-passation inexacte, gardes actives, totaux par organisation : nombre de paiements et somme) ; les totaux ne bougent pas ;
4. vérifier que **l'ancienne version du binaire** démarre et réussit ses tests de fumée sur le nouveau schéma (condition du retour arrière) ;
5. une migration de **données** incompatible est livrée seule, avec sa procédure inverse écrite et jouée une fois sur copie.

## 4. Tests d'API et de contrat

Générés depuis le registre des routes et la matrice des droits (`API_CONTRACTS.md` §4.6, §8). **Une route sans ces tests fait échouer le build** (`CLAUDE.md`, règle d'or 4).

| # | Test | Pour chaque… | Résultat exigé |
|---|---|---|---|
| A1 | Sans session | route authentifiée | 401 |
| A2 | Sans jeton anti-falsification ou `Origin` incorrect | route qui écrit | 403 `auth.csrf_invalid` |
| A3 | **Droits** | couple rôle × action de la matrice (gérant, gestionnaire, caissier, agent terrain, comptable, propriétaire, collaborateur) | autorisé ou refusé conformément à la matrice ; refus côté serveur dans **100 %** des cas interdits ; droits réglables testés dans les deux états |
| A4 | **Isolation** | route avec identifiant de ressource | l'utilisateur de A appelle avec un identifiant de B : **404** `resource.not_found`, aucune donnée de B dans la réponse, ligne d'audit `security.cross_org_attempt`, alerte |
| A5 | Portée de l'agent terrain | lecture de lots, impayés, recherche | seuls ses lots, jeu de lots assignés et non assignés |
| A6 | Format d'erreur | réponse d'erreur | `problem+json`, `code` stable présent, `request_id` présent, aucun nom de table ni pile d'appels |
| A7 | Schéma | réponse de succès | conforme au fichier OpenAPI (validation de la réponse réelle) |
| A8 | Idempotence | route d'écriture d'argent | section 2.3 |
| A9 | Immutabilité | paiement, quittance, reversement, relance, audit | `PUT`, `PATCH`, `DELETE` : 405 avec le code `*.immutable` |
| A10 | Validation | corps de requête | champs inconnus refusés, montant non entier refusé, hors bornes refusé, date future refusée, téléphone invalide refusé, toutes les erreurs de champ renvoyées ensemble |
| A11 | Limitation du débit | connexion, réinitialisation, écritures d'argent | blocage progressif, 429 avec `Retry-After`, message de connexion identique que l'identifiant existe ou non |
| A12 | En-têtes de sécurité | chaque type de page | CSP stricte, HSTS, `nosniff`, `Referrer-Policy: no-referrer`, `Cache-Control: no-store`, `X-Robots-Tag` sur la page publique |
| A13 | Journaux | parcours complet | aucun numéro `+226`, jeton, mot de passe ni corps de requête |
| A14 | Fichiers | `GET` de PDF, documents, import | clé de stockage construite depuis l'organisation ; adresse d'un fichier de B demandée par A refusée ; `attachment` forcé pour un document téléversé |
| A15 | Opérations longues | `GET /operations/{id}` | un utilisateur de A ne lit pas l'opération de B (la table `jobs` n'a pas de sécurité par ligne) |

**Contrat front/back** (`API_CONTRACTS.md` §8, C1 à C13) : fichier OpenAPI régénéré identique ; aucune rupture de compatibilité sans dépréciation ; client TypeScript compile ; exemples valides ; tous les codes d'erreur ont un texte dans le front ; le front de la version N parle au serveur de la version N−1 [APRÈS, recommandé].

## 5. Tests de bout en bout

Playwright sur l'**essai** ou un environnement local identique, avec l'organisation de démonstration, horloge fixée au 13/11/2026 09:15, **deux tailles** : ordinateur 1440 px et mobile 390 px (profil téléphone d'entrée de gamme, section 8). Le premier chargement de chaque parcours est mesuré (poids).

| Parcours | Scénario nominal | Variantes testées | Priorité |
|---|---|---|---|
| **1 Encaisser et quittance** (caissière) | rechercher « Tapsoba » ; carte « Due aujourd'hui » 75 000 ; mode espèces par défaut ; « Encaisser et envoyer la quittance » ; aperçu avec numéro ; « Envoyer sur WhatsApp » (adresse `wa.me` construite, **non ouverte**) ; paiement en tête du journal, statut « Validé » | 50 000 (E-05) ; 100 000 avec avance (E-06) ; référence mobile money en doublon (R-041) ; **connexion perdue** puis nouvel essai sans doublon (R-042) ; montant au-dessus du seuil (R-038) ; paiement par un tiers (E-33) ; quittance dont le PDF échoue (le paiement reste enregistré, R-049) | [PILOTE] |
| **2 Impayés et relances** (gestionnaire) | accueil « À faire aujourd'hui » ; impayés, tranches, filtre M3 ; « Relancer » Pauline Yaméogo ; texte prêt ; WhatsApp ; « Avez-vous envoyé ? » oui ; promesse au 20/11 ; la ligne reste (R-059) | seconde relance le même jour (avertissement non bloquant, R-063) ; locataire sans WhatsApp ; variable inconnue dans un modèle (R-064) ; agent terrain : seuls ses lots ; état vide positif | [PILOTE] |
| **3 Compte rendu propriétaire** (gérante) | mandant M1, octobre ; 400 000 → commission 32 000 → net **338 000** ; impayé de 45 000 ; valider (modale, confirmation du mot de passe) ; PDF ; lien ; reversement ; statut « Payé » ; « Rouvrir » visible avant reversement et absent après | lot sans mandat (R-075) ; dépense sans pièce ; valeur obsolète entre aperçu et validation (`statement.preview_stale`) | [PILOTE] |
| **4 Import Excel** (gestionnaire) | téléverser ; colonnes ; aperçu 197 créables et 3 erreurs aux lignes 14, 87, 152 ; « continuer sans ces lignes » ; fusion de deux doublons ; confirmer ; rapport ; historique « Annulable » ; annuler : rien ne reste (E-21) | format invalide (colonnes attendues) ; fichier trop gros ; annulation refusée après un paiement | [PILOTE] |
| **5 Historique d'un lot** (gestionnaire) | filtre « Ouaga 2000 » ; Appartement A3 ; onglets Aperçu, Travaux (intervention du 22/10, 30 000, imputée au compte rendu d'octobre), Documents ; retour au compte rendu | ajouter une intervention ; agent terrain : lot non assigné introuvable | Aperçu et baux [PILOTE] ; Travaux T2 ; Documents [APRÈS] |
| **Transverses** | connexion, déconnexion, expiration de session, « déconnecter tous mes appareils » ; menu selon le rôle ; sélecteur d'organisation ; bandeau « Une mise à jour est disponible » ; bouton « Signaler un problème » | accès direct à une adresse interdite (« Accès refusé ») ; ressource d'une autre organisation (« Introuvable ») | [PILOTE] |

- Les assertions portent sur **les montants et numéros affichés** (ils viennent de l'API), les statuts, l'absence d'erreur console, l'absence de défilement horizontal de la page, et l'**absence** de tout repère propre à la maquette (« V1 », « À venir », « nom provisoire ») dans le produit.
- Les 28 contrôles du vérificateur de la maquette (`scripts/verifier-donnees.js`) deviennent des tests d'intégration sur l'organisation de démonstration (`ARCHITECTURE.md` §8.5).
- Les tests de bout en bout ne remplacent pas ceux des niveaux inférieurs : ils prouvent que les couches s'assemblent.

## 6. Tests visuels de la quittance et du compte rendu en PDF

**Quoi** : le PDF est un rendu de l'instantané ; ses défauts (accent cassé, montant tronqué, deuxième page) se voient d'abord sur un document remis à un locataire. Moteur non choisi (décision n° 7) : le jeu de cas est celui du prototype (a), puis rejoué en CI.

**Cas types (20, `UI_UX_DESIGN.md`)** : accents et cédilles (« Ouédraogo », « Yaméogo », « ç », « œ ») ; noms longs et composés (deux lignes au plus) ; société avec raison sociale longue ; adresse longue ; montant à 7 chiffres en lettres (« un million deux cent cinquante mille ») ; montant maximal 999 999 999 ; paiement partiel ; avance ; tiers payeur ; colocation (deux locataires) ; bail commercial « TVA incluse » ; **rectificative** ; **annulée** (filigrane « ANNULÉE — remplacée par la quittance n° … ») ; série d'essai (filigrane « ESSAI ») ; organisation de démonstration (filigrane « DÉMONSTRATION — sans valeur ») ; apostrophes et guillemets ; nom contenant du balisage ou une adresse (`<b>`, `https://…`) ; logo absent ; logo maximal (40 × 20 mm) ; format A4 et format A5.

**Vérifications automatiques par document** (décision n° 83) :

| Contrôle | Méthode |
|---|---|
| Une seule page en A4 et en A5, dans tous les cas | comptage des pages |
| Les textes attendus sont présents et corrects : numéro, noms accentués, montant en chiffres et **en lettres**, période, mode, solde restant, mentions, « TVA incluse » le cas échéant | extraction du texte du PDF et comparaison à l'instantané |
| Polices embarquées, accents complets | liste des polices du PDF |
| Aucune métadonnée d'auteur ; titre neutre ; aucune référence externe ; aucune requête sortante pendant le rendu | inspection du PDF et du bac à sable |
| Lisible en noir et blanc, aucun fond coloré plein, aucun texte clair sur fond foncé | rendu en niveaux de gris ; contraste mesuré |
| **Mise en page** | rendu en image à résolution fixe, comparaison à une **image de référence approuvée** avec tolérance ; toute différence demande l'approbation d'un humain dans la PR (images avant/après jointes) |
| Reproductibilité | deux rendus du même instantané donnent la même empreinte SHA-256, ou la même image. **Test de non-régression du moteur** : il ne prouve rien sur les documents émis, dont l'original fait foi (DP-71). Mesuré au prototype (a), DP-76 : identique pour Typst et le tracé natif, différent pour Chrome (dates de création) |
| Performance | rendu p95 inférieur à 1 s ; mémoire au pic compatible avec un serveur de 2 Go (prototype a) |

**Montant en lettres** : cas unitaires de 0 à 2 000 **tous** testés ; frontières : 20, 21, 70, 71, 80, 81, 90, 91, 99, 100, 101, 200, 201, 1 000, 1 100, 80 000, 100 000, 200 000, 1 000 000, 1 250 000, 2 000 000, 21 000 000, 999 999 999 ; accords de « vingt » et « cent » (« quatre-vingts », « deux cents » devant « mille » : règles à valider), « mille » invariable, « et un ». Orthographe traditionnelle ou rectifiée et formulation « francs CFA » à valider (décision n° 12) : les tests sont **paramétrés** par la décision.

**À la main** : impression A4 et A5 sur une imprimante laser ordinaire en noir et blanc, une fois avant le pilote et à chaque changement de gabarit.

## 7. Tests d'import avec des fichiers Excel imparfaits

Les fichiers sont **synthétiques**, construits à la main ou par script, imitant ce que produisent de vrais carnets : `tests/fixtures/import/`. Aucun fichier réel d'une agence n'y entre sans anonymisation et accord écrit (décision n° 15).

| Catégorie | Fichiers et défauts | Résultat exigé |
|---|---|---|
| **Référence** | 200 lignes, 3 erreurs aux lignes 14, 87, 152, 2 paires de doublons de téléphone (`importExemple` de la maquette) | rapport exact : 197 créables, 3 erreurs listées avec leur numéro ; doublons proposés à la fusion (E-20) |
| Structure | colonnes décalées ou dans un autre ordre ; titres de colonnes différents (« Tél », « Téléphone », « N° tel ») ; lignes de titre au-dessus de l'en-tête ; cellules fusionnées ; colonnes et lignes masquées ; lignes vides ; colonnes en trop ; colonne obligatoire absente | la correspondance propose la bonne association ou signale l'absence ; aucune ligne créée sans confirmation |
| Encodage | CSV en `;` et UTF-8 avec BOM ; CSV Windows-1252 (export Excel français) ; accents en forme composée et décomposée ; guillemets, retours à la ligne dans une cellule | accents conservés ; détection de l'encodage correcte |
| **Dates** | `13/11/2026`, `13/11/26`, `2026-11-13`, nombre de série Excel, `13 nov. 2026`, `31/02/2026` (impossible), `03/04/2026` (ambiguë), date de fin avant le début | formats reconnus selon la règle choisie ; date impossible ou ambiguë signalée avec le numéro de ligne (`import.date_invalid`, `import.date_order`) |
| **Montants en texte** | `75 000`, `75.000`, `75000 F`, `75 000 FCFA`, `1,250,000`, `75000,5` (décimale), `0`, `-5000`, `1 000 000 000` (hors bornes) | entiers reconnus ; décimale et hors bornes refusés (`import.amount_not_integer`, `import.amount_out_of_range`) ; **aucune troncature silencieuse** |
| Téléphones | `70123456`, `+226 70 12 34 56`, `0022670123456`, `70-12-34-56`, lettres, 7 chiffres, deux numéros dans une cellule | normalisation vers `+226XXXXXXXX` ; numéros invalides refusés |
| Énumérations | « Habitation », « HABITATION », « logement », « Appartement », « Boutique », valeur inconnue | reconnaissance insensible à la casse et aux accents ; inconnue : `import.enum_unknown` |
| **Doublons** | même téléphone sur deux locataires ; même référence de lot dans un bien ; deux baux chevauchants sur un lot dans le fichier | doublons proposés à la fusion (R-017) ; chevauchement refusé (`import.lease_overlap`) |
| Valeurs métier | loyer nul ou négatif ; jour d'échéance 31 ; dépôt supérieur à deux mois (avertissement) ; arriérés datés | refus ou avertissement ; arriérés créés comme lignes de type « arriéré » (R-035) |
| **Limites** | 5 000 lignes pile et 5 001 ; 5 Mo et plus ; très longues cellules ; émojis | 5 000 acceptées, 5 001 refusées (`import.too_many_rows`) ; limites respectées |
| **Fichiers piégés** (`SECURITY.md` §7.4, §11) | `.xls`, `.xlsm` ; archive « bombe » (ratio de compression) ; XML avec entité externe ; formules (`=HYPERLINK(…)`, `=cmd|…`) ; noms de cellule commençant par `=`, `+`, `-`, `@` ; faux XLSX (extension seule) | refusés ou neutralisés ; **aucune formule évaluée** ; rapport exporté avec cellules neutralisées ; aucune lecture de fichier du serveur |
| Garanties de processus | confirmation, annulation, idempotence | avant confirmation : **zéro** ligne créée (comptage en base) ; après annulation : rien ne reste du lot (E-21) ; annulation refusée après un paiement ; double confirmation sans effet ; 2 000 lignes analysées en moins de 5 s ; analyse à mémoire et temps bornés |

Chaque fichier a un fichier de résultat attendu (`*.expected.json`) : numéros de ligne, statut, code. Un changement de règle d'import met à jour les deux dans la même PR.

## 8. Performance et usage réel

### 8.1 Profil de test « Android d'entrée de gamme, réseau lent » (décision n° 81)

- **Émulation** (CI et local) : viewport 360 × 760 et 390 × 844, processeur ralenti **4×**, réseau « 4G lente » (de l'ordre de 1,6 Mbit/s descendant, 750 kbit/s montant, 150 ms d'aller-retour) et profil « 3G » (de l'ordre de 400 kbit/s, 400 ms) ; latence supplémentaire d'un hébergement en Europe mesurée par le prototype (e) [Hypothèse : 100 à 200 ms].
- **Appareil réel** : un téléphone Android d'entrée de gamme (mémoire de 2 Go ou moins, navigateur d'origine) est **acheté** pour l'équipe [coût de l'ordre de quelques dizaines d'euros, estimation à vérifier ; budget à valider, décision n° 81]. Il sert à chaque lot de tranche et à chaque séance de recette. Les mesures de l'émulation ne remplacent pas celles de l'appareil.

### 8.2 Temps pour encaisser un loyer (objectif : moins de 30 secondes)

Trois angles complémentaires, car le temps total est surtout du temps **humain** :

| Angle | Mesure | Seuil |
|---|---|---|
| **Temps machine** | scénario Playwright du parcours 1 sur le profil de 8.1 : somme des attentes réseau et de rendu (hors saisie) | **5 s au plus** sur « 4G lente » [Hypothèse] ; chemin nominal à **3 allers-retours** après l'ouverture (`API_CONTRACTS.md` §6.1) ; serveur : enregistrement p95 ≤ 300 ms, recherche p95 ≤ 150 ms |
| **Effort de saisie** | nombre d'appuis et de champs pour le chemin nominal | au plus : recherche (3 à 5 caractères), sélection, validation, soit moins de 5 gestes ; mode espèces et date du jour par défaut |
| **Temps réel d'usage** | événement `payment_recorded.duration_ms` (de l'ouverture du formulaire à la validation) pendant les essais et le pilote ; chronométrage en séance | **médiane sous 30 s** (KPI dossier §23) ; 20 paiements variés en moins de 10 minutes sans aide (critère de sortie de la phase 2) |

### 8.3 Poids des pages et temps d'affichage

| Mesure | Seuil | Contrôle |
|---|---|---|
| Premier chargement HTML + CSS + JS compressés | ≤ 200 Ko | `size-limit`, **bloquant** en CI |
| JavaScript / CSS | ≤ 150 Ko / ≤ 30 Ko | idem |
| Polices web | 0 | test sur le build |
| Images d'une page / logo | ≤ 100 Ko / ≤ 30 Ko | revue + test |
| Réponse d'une liste | ≤ 30 Ko compressés (25 lignes) | test d'API |
| LCP en « 4G lente », processeur 4× | ≤ 3 s | Lighthouse en nocturne sur l'essai, puis à la main sur l'appareil réel |
| Écran Encaisser | chargé en paquet séparé au premier accès du caissier | revue du découpage |
| Page publique de partage (S26) | HTML sans JavaScript applicatif | test d'en-têtes et de contenu |

### 8.4 Charge raisonnable [APRÈS]

Début de mois, 20 utilisateurs simultanés, pic d'encaissements d'une organisation de 500 lots : p95 des budgets de `ARCHITECTURE.md` §12.1 respectés. Joué avant la phase de stabilisation (dossier §19), pas avant le premier pilote (volumes pilote : moins de 20 utilisateurs, au plus 2 500 paiements par mois).

### 8.5 Accessibilité

| Contrôle | Méthode | Bloquant |
|---|---|---|
| Règles automatiques sur les écrans clés (14 écrans, ordinateur et mobile) | moteur d'analyse d'accessibilité intégré à Playwright : **zéro** violation « critique » ou « sérieuse » | oui [PILOTE] |
| Règles de lint | `jsx-a11y` en `error` | oui [PILOTE] |
| Clavier seul sur les cinq parcours : ordre de tabulation, focus visible, piège de focus dans les modales, Échap | à la main | avant le pilote |
| Lecteur d'écran (TalkBack sur l'appareil réel) sur le parcours Encaisser : montant lu en toutes lettres, erreurs liées au champ, toasts annoncés | à la main | avant le pilote |
| Contrastes (≥ 4,5:1 texte, ≥ 3:1 composants) et statuts jamais par la seule couleur | test sur les tokens + **contrôle avec un outil reconnu** (les valeurs de DP-44 ne sont calculées que par formule) | avant la diffusion publique |
| Cibles tactiles ≥ 44 × 44 px, zoom 200 % sans défilement horizontal, `prefers-reduced-motion`, `lang="fr"` | tests Playwright | oui [PILOTE] |

## 9. Données de test

- **Jeu aligné sur la maquette**, produit par le scénario `seeds/demo/` qui **rejoue les actions métier par les services** (inscription, biens, baux, paiements, relances, relevés) avec une horloge fixée, et non par des `INSERT` directs : les invariants (numérotation, affectations, audit) sont vrais par construction. Organisations « Agence Horizon Démo » (préfixe `HOR`, plan Agence : 40 lots, 31 locataires, 33 baux en cours dont 1 en préavis, 4 mandants, prochain numéro HOR-2026-000094) et « Patrimoine Sawadogo » (préfixe `SAW`, plan Propriétaire : 12 lots, 10 locataires) ; ancrages E-05, E-06, E-13, E-13b, E-15, E-16, E-18, E-20, E-25, E-37, E-38.
- **Jamais de donnée réelle** : numéros du bloc fictif `+226 00 00 01 NN`, `…02 NN` (organisation de Sawadogo), `…03 NN` et `…04 NN` (fichiers d'import) ; adresses électroniques en `example.test` ; noms combinant des prénoms et noms courants sans personne publique connue (à faire relire avant diffusion, `maquette/NOTES.md`). **Test de CI** : aucun numéro hors du bloc fictif dans `tests/`, `seeds/`, `maquette/`, captures et exemples ; fixtures modifiées = deuxième relecteur.
- **Copie d'essai anonymisée** de la production (`immo anonymize`, jamais en production) : uniquement avec l'**accord écrit** de l'agence (décision n° 15) ; contrôle automatique qu'aucun téléphone hors du bloc fictif, e-mail, pièce d'identité, note ni texte de relance ne subsiste ; montants, dates et structure conservés (utiles aux tests de migration et de performance).
- **Organisations d'essai** (`kind = test`) créées par les tests, détruites avec la base éphémère. L'organisation de démonstration (`kind = demo`) sert aussi de **témoin d'isolation** : aucune requête d'une organisation réelle ne la voit, et inversement.
- **Cas golden et fichiers de fixtures** : versionnés, relus, jamais générés à partir d'une base de production.
- **Remise à zéro** : `immo demo reset` crée une nouvelle organisation de démonstration et archive l'ancienne (aucune suppression).

## 10. Chaîne d'intégration continue (décision n° 82)

| Étape | Contenu | Seuils bloquants | Durée cible |
|---|---|---|---|
| **0. Avant validation locale** | `cargo fmt --check`, `clippy` rapide, détection de secrets | échec = commit refusé (hook local) | secondes |
| **1. PR rapide** | formatage, `clippy -D warnings`, tests unitaires et de propriétés (nombre de cas réduit), `cargo sqlx prepare --check`, `cargo audit`, `cargo deny`, `npm ci`, lint, `tsc --noEmit`, tests du front, génération OpenAPI et comparaison (C1 à C3), poids du front, détection de secrets sur tout l'historique | tout échec | ≤ 10 minutes |
| **2. PR intégration** | tests de base (rôles de production), isolation par table et par route, matrice de droits, tests d'API et de contrat, idempotence, numérotation (version réduite), migrations sur base vide, accessibilité automatique, parcours 1 de bout en bout (mobile et ordinateur), contrôles d'en-têtes, journaux | tout échec ; **un test d'isolation manquant fait échouer le build** | ≤ 15 minutes au total |
| **3. Nocturne** | propriétés à grand nombre de cas, numérotation sous charge (20 × 500), les cinq parcours × deux tailles, import avec tous les fichiers, tests visuels PDF, migrations sur la copie anonymisée de l'essai, Lighthouse sur le profil réduit, contrôle de trous de numérotation, `cargo audit` à jour | tout échec crée une alerte ; une régression de numérotation ou d'isolation **gèle les livraisons** | ≤ 60 minutes |
| **4. Avant livraison en production** | étape 2 complète sur le commit à livrer, liste de contrôle de la section 14.4 | liste complète cochée | manuel |

- **Durée acceptable** : l'étape 2 reste **sous 15 minutes** (90ᵉ centile) ; au-delà de 20 minutes, la CI est optimisée (découpage, cache) avant d'ajouter des tests. Une CI lente est contournée.
- **Tests instables** : aucun nouvel essai automatique sur les tests d'argent et d'isolation (un nouvel essai masquerait une course critique). Un test instable ailleurs est mis en quarantaine au plus 7 jours, avec un ticket, 3 en même temps au plus.
- **Pas de couverture imposée en pourcentage** au pilote : ce qui est exigé, ce sont les règles d'argent, les routes et les parcours listés. La couverture est mesurée [APRÈS] pour repérer les zones aveugles.
- La promotion en production reprend **la même image** que l'essai (`DEPLOYMENT.md`).

## 11. Gestion des anomalies

### 11.1 Niveaux de gravité (décision n° 84)

Échelle **unique** S1 à S4 de tout le dossier : `SECURITY.md` §9.2 et `DEPLOYMENT.md` §7.3 la reprennent (une quittance sans PDF depuis plus de 10 minutes est S2 partout).

| Niveau | Définition | Exemples | Gel des livraisons |
|---|---|---|---|
| **S1 — bloquante** | **Toute** erreur sur un montant, un solde, une quittance (numéro, contenu), une écriture ou un compte rendu ; **toute** fuite entre organisations ; perte ou altération de données ; secret exposé ; encaissement impossible pour une agence plus d'une heure | quittance de 75 000 pour un paiement de 57 000 ; trou ou doublon de numéro ; un caissier voit les locataires d'une autre agence ; sauvegarde corrompue | **oui, immédiat** |
| **S2 — majeure** | une fonction principale est dégradée ou impossible, avec contournement ; donnée non sensible exposée à tort dans une même organisation ; PDF absent plus de 10 minutes ; import qui échoue | relance impossible ; compte rendu qui ne se produit pas ; droits trop larges sur un rôle sans effet sur l'argent | non, sauf si elle touche la même zone que S1 |
| **S3 — mineure** | friction, texte, mise en page non bloquante, lenteur sans perte de saisie | libellé ambigu ; tableau mal aligné sur 360 px | non |
| **S4 — cosmétique** | souhait, amélioration | couleur, ordre de colonnes | non |

Une erreur de montant ou d'isolation est **S1 quelle que soit sa fréquence** : une seule occurrence suffit.

### 11.2 Délais de correction cibles [À valider avec l'agence pilote, heures de couverture à fixer]

| Niveau | Prise en compte | Contournement ou atténuation | Correctif livré |
|---|---|---|---|
| S1 | 1 heure (alerte immédiate ; information de l'agence concernée **dans l'heure** pour une interruption de l'encaissement, `SECURITY.md` §9) | 4 heures (cible de reprise, `ARCHITECTURE.md` §10.4) : coupure de la route, retour arrière, restauration | 24 heures |
| S2 | 4 heures ouvrées | 24 heures | **72 heures** (correctifs prioritaires du dossier, §20.2) |
| S3 | 2 jours ouvrés | — | prochain lot hebdomadaire (14 jours au plus) |
| S4 | tri hebdomadaire | — | selon la règle « sert-elle au moins trois clients ? » (dossier §20.3) |

La couverture réelle (jours, heures, personne de repli) est la décision n° 84, **ouverte** : à fixer par Mimis avant le premier pilote (PP-37). Le développeur n'a pas d'astreinte permanente (DP-54).

### 11.3 Politique de non-régression

1. Tout bogue corrigé reçoit d'abord un test qui **échoue** sur le bogue, nommé `regression_<identifiant>_<règle>` et qui cite la règle `R-xxx` ou le cas `E-xx` ; le test passe avec le correctif et reste.
2. Un S1 ajoute en plus : un cas golden ou une propriété, un point dans la liste de contrôle de livraison, un **retour d'expérience écrit sans recherche de coupable** dans les 5 jours (chronologie, cause, ce qui a marché, ce qui change).
3. Un bogue de données corrigé par migration ou par contre-passation exécute les cinq requêtes de contrôle avant et après (`DATABASE_SCHEMA.md` §7.5) et enregistre l'opération.
4. Un correctif S1 ou S2 est livré **seul** (pas groupé avec une fonction), avec le test de non-régression, après une sauvegarde à la demande.
5. Un test supprimé ou affaibli pour faire passer la CI est une faute de revue.

### 11.4 Liste de contrôle avant chaque livraison en production

- [ ] CI de l'étape 2 verte sur le commit exact à livrer ; nocturne de la veille verte
- [ ] Aucune alerte S1 ou S2 ouverte ; aucun gel actif
- [ ] Différence `openapi.json` relue ; aucun changement incompatible sans dépréciation datée
- [ ] Migrations : rétrocompatibles ; exécutées sur la dernière copie anonymisée ; contrôles de numérotation, affectations, gardes, totaux **avant et après** ; sauvegarde à la demande faite
- [ ] Retour arrière possible : l'image précédente fonctionne avec le nouveau schéma
- [ ] Parcours 1 vérifié à la main sur l'essai, sur le téléphone d'entrée de gamme
- [ ] Budgets de poids respectés ; accessibilité automatique verte
- [ ] Fonction nouvelle livrée **éteinte** derrière un interrupteur ; qui l'allume, quand, pour quelle agence
- [ ] Calendrier de livraison respecté (`DEPLOYMENT.md` §4)
- [ ] Information préalable de l'agence pilote si le changement est visible ou si une coupure est prévue
- [ ] Documents à jour (`API_CONTRACTS.md`, `DATABASE_SCHEMA.md`, `DECISIONS.md`)

## 12. Critères « prêt pour un pilote »

Liste **vérifiable** : chaque case a une **preuve** (sortie de CI, compte rendu, document signé). Rien n'est coché sans preuve consignée. C'est la **seule** liste « prêt pour un pilote » du dossier : `ARCHITECTURE.md` §9.3, `SECURITY.md` §10 et `DEPLOYMENT.md` §10 y renvoient et ne la recopient pas. Tous les points sont **[PILOTE]** et **ne se reportent pas**.

| # | Critère | Preuve |
|---|---|---|
| **Intégrité des écritures** | | |
| PP-01 | Paiements, affectations, quittances, reversements, versions de comptes rendus, audit, relances : ajout seul à deux barrières | tests G-05 et ISO-4 verts pour les trois rôles ; contrôle nocturne actif |
| PP-02 | Contre-passation et quittance rectificative opérationnelles (E-38) | tests D, A, E verts |
| PP-03 | Numérotation sans trou prouvée sous concurrence | prototype (c) rejoué en CI : 20 × 500, 3 organisations, 10 % d'annulations, zéro trou ; requête de contrôle à zéro |
| PP-04 | Idempotence : double clic, coupure avant et après validation, même clé en parallèle | tests de la section 2.3 verts |
| PP-05 | Cas E-01 à E-48 applicables à T1 et T2 automatisés ; trois cas d'ancrage verts | rapport de tests : tableau de 2.1 sans ligne rouge ni manquante |
| PP-06 | Propriétés P1 à P16 vertes, 10 000 cas par propriété | exécution nocturne |
| PP-07 | Les 28 contrôles de la maquette verts sur l'organisation de démonstration | exécution du vérificateur et tests d'intégration |
| **Isolation** | | |
| PP-08 | RLS activée et forcée sur toutes les tables métier ; rôle applicatif sans `BYPASSRLS`, non propriétaire | test de schéma (ISO-5, ISO-6) |
| PP-09 | Un test d'isolation par route, couverture 100 %, bloquante | comparaison automatique routes / registre de tests |
| PP-10 | Pool sous concurrence : aucune lecture croisée (50 tâches, 2 connexions, 10 000 itérations) | prototype (b) rejoué en CI |
| PP-11 | Matrice des droits testée pour tous les couples rôle × action ; agent terrain limité à ses lots | tests générés depuis la matrice |
| PP-12 | Accès support limité dans le temps, journalisé ; TOTP obligatoire pour les comptes HABEK | tests ISO-8 et test de bout en bout par la console |
| **Sauvegardes et reprise** | | |
| PP-13 | Sauvegarde continue, copie chiffrée chez un second fournisseur, clé hors hébergeur en deux exemplaires | revue de configuration ; tentative de suppression avec les identifiants du second fournisseur refusée |
| PP-14 | **Restauration complète faite sur une instance neuve, chronométrée**, comptages et totaux par organisation comparés, durée inférieure à 4 heures | compte rendu du premier test (modèle de `DEPLOYMENT.md`) |
| PP-15 | Alerte de sauvegarde manquante (24 h) | sauvegarde volontairement retardée en essai |
| PP-16 | Plan de réponse à incident écrit, rôles nommés, joué une fois sur table | compte rendu de l'exercice |
| **Sécurité et exploitation** | | |
| PP-17 | HTTPS, en-têtes de sécurité, limitation des tentatives, argon2id calibré, jeton anti-falsification, inscription publique fermée | tests A2, A11, A12 |
| PP-18 | Import durci : types, tailles, archives, entités XML, macros, formules | jeu de fichiers piégés (section 7) |
| PP-19 | Exports neutralisés ; rendu PDF isolé sans réseau ni secret | tests dédiés |
| PP-20 | `cargo audit`, `cargo deny`, `npm audit` verts ; aucun secret dans l'historique | sortie de CI |
| PP-21 | Journaux sans donnée personnelle ni jeton | test A13 |
| PP-22 | Mise en production et retour arrière exercés sur l'essai | procès-verbal de l'exercice |
| PP-23 | Export complet de l'organisation par commande testé ; purge en phase `setup` testée sur une organisation d'essai | tests de la section 3 et des commandes |
| PP-24 | Auto-test d'intrusion de deux jours par la personne qui n'a pas écrit le module : aucune faille critique ou haute ouverte | rapport |
| **Usage** | | |
| PP-25 | Encaisser en moins de 30 s (médiane) sur un Android d'entrée de gamme ; 20 paiements variés en moins de 10 minutes sans aide | chronométrage de la section 8.2 |
| PP-26 | Budgets de poids et de LCP respectés ; accessibilité automatique verte ; clavier et lecteur d'écran passés sur les cinq parcours | CI et compte rendu manuel |
| PP-27 | Quittance validée par l'agence (mise en page, mentions) ; mentions revues par un juriste ou risque accepté par écrit (DP-62) | validation écrite |
| PP-28 | Bouton « Signaler un problème » et événements d'usage en place et testés | test de bout en bout |
| **Cadre** | | |
| PP-29 | Accord écrit de l'agence : traitement des données, données de test anonymisées (n° 15), droit de purge en phase de reprise (n° 25) ; engagement de confidentialité signé | documents au dossier |
| PP-30 | Position sur l'hébergement hors du pays (Neon, DP-55) et la CIL tranchée, ou risque accepté par écrit (DP-62) | entrée `DECISIONS.md` |
| PP-31 | Cycle `setup` → `live` expliqué ; date de bascule fixée **hors** début de mois ; TOTP du gérant actif avant la bascule, la bascule sans TOTP étant refusée (DP-68) | note de bascule ; test de refus |
| PP-32 | Décisions bloquantes enregistrées : DP-52 (stratégie), DP-54 (capacité), DP-55 (base) et le reste de l'hébergement (n° 5), sauvegarde logique (n° 46), moteur PDF (n° 7), règles de T1 (DP-58), avis juridique (DP-62) | entrées `DP-xx` |
| PP-33 | Gouvernance des accès : comptes nominatifs, aucun accès de routine à la base de production, accès d'urgence par le développeur et le porteur du projet ; secrets distincts par environnement ; `payout_details_enc` et `totp_secret_enc` illisibles en base ; TLS `verify-full` | revue des droits et de l'inventaire ; lecture directe de la colonne |
| PP-34 | Liens de partage (128 bits, expiration, révocation) ; organisation de démonstration : mélange avec des comptes réels refusé par la base | tests dédiés |
| PP-35 | Copie d'essai anonymisée, contrôle automatique vert, accord écrit de l'agence | requête de contrôle des téléphones hors bloc fictif |
| PP-36 | Page d'état publique et canal d'alerte vers le développeur et le porteur du projet | capture de la page ; alerte d'essai reçue |
| PP-37 | Heures de couverture, personne de repli et délais S1 et S2 fixés (décision n° 84) et communiqués à l'agence | entrée `DECISIONS.md` ; engagement de l'agence |
| PP-38 | Engagement de confidentialité interne signé par chaque personne de l'équipe ; page des bons usages remise à l'agence (comptes nominatifs, téléphone perdu, lien de partage) | documents signés ; remise consignée |

## 13. Recette avec les agences pilotes

### 13.1 Principes (décision n° 85)

- **Observer, ne pas guider** : on teste le logiciel, pas la personne. Chaque hésitation est un défaut du produit.
- **Séance de 90 minutes, sur place**, avec **un animateur** (qui parle) et **un observateur-scribe** (qui note, ne parle pas), sur un **téléphone Android réel** de l'agence quand c'est possible, et l'ordinateur ensuite.
- **Données** : l'organisation de **démonstration** ou une organisation en phase `setup` ; jamais l'accord de faire une séance sur des données réelles sans accord écrit. **Aucun enregistrement vidéo ni capture d'écran** de données réelles ; notes manuscrites et chronométrage ; seules les captures d'une démonstration sont conservées.
- **Participants** : caissier, gestionnaire, gérant, et si possible un propriétaire direct pour le plan léger ; une séance par rôle clé.
- **Alignement avec le dossier** : les séances de recette et le pilote visent des **engagements**, pas des compliments (échelle 0 à 4, dossier §20.1) ; la séance note aussi le niveau d'engagement.

### 13.2 Scénarios d'essai (formulés pour l'utilisateur, sans indiquer où cliquer)

| N° | Rôle | Scénario | Ce qu'on observe | Parcours |
|---|---|---|---|---|
| R1 | Caissier | « Un locataire, M. Tapsoba, vient payer son loyer de novembre en espèces. Enregistrez-le et donnez-lui sa quittance. » | temps ; recherche ; compréhension du solde ; envoi WhatsApp ou impression | 1 |
| R2 | Caissier | « Il ne paie que 50 000 aujourd'hui. » puis « Un autre paie 100 000 pour un loyer de 75 000. » | compréhension du « partiel » et de l'« avance » | 1 |
| R3 | Caissier | « Vous vous êtes trompé de montant : le loyer était de 75 000, pas 57 000. Que faites-vous ? » | compréhension de la demande de correction (on ne modifie pas) | 1 |
| R4 | Caissier | « Votre connexion se coupe pendant l'enregistrement. » (réseau simulé) | confiance dans « il ne sera pas créé deux fois » | 1 |
| R5 | Gestionnaire | « Quels locataires sont en retard ? Relancez celui qui doit le plus ancien. » | lecture des tranches ; choix du ton ; noter la promesse | 2 |
| R6 | Gestionnaire | « Retrouvez l'historique des travaux de l'appartement A3. » | navigation lot, onglets | 5 |
| R7 | Gérant | « Produisez le compte rendu d'octobre de Mme Ouédraogo, vérifiez-le, validez-le. » | confiance dans le net ; compréhension de la commission et de la dépense ; modale de validation | 3 |
| R8 | Gérant | « Un reversement a été fait par virement. Enregistrez-le. » | compréhension du caractère définitif | 3 |
| R9 | Gestionnaire | « Voici votre fichier Excel de locataires. Importez-le. » (fichier imparfait de l'agence, anonymisé) | correspondance des colonnes ; lecture des erreurs ; peur de « casser » | 4 |
| R10 | Propriétaire direct | « Qui a payé ce mois, qui est en retard ? Envoyez un rappel. » (téléphone, 5 minutes sans aide) | critère du dossier (§9) : prise en main en 5 minutes | 2 |

### 13.3 Déroulé d'une séance (90 minutes)

| Durée | Étape | Contenu |
|---|---|---|
| 10 min | Accueil | présenter l'équipe ; « nous testons le logiciel, pas vous » ; rappeler la confidentialité et l'absence d'enregistrement ; recueillir le consentement à la prise de notes ; demander les outils actuels (carnet, Excel, téléphone) |
| 10 min | Contexte | comment l'agence encaisse un loyer aujourd'hui ; dernier litige de paiement ; combien de loyers en retard ce mois ; ce qu'elle perd chaque mois |
| 45 min | Scénarios | 4 à 5 scénarios de 8 à 10 minutes ; l'animateur lit l'énoncé et se tait ; l'aide n'est donnée qu'après trois relances silencieuses et est **notée** |
| 15 min | Débrief | « qu'est-ce qui vous a gêné ? qu'est-ce qui manque ? qu'est-ce que vous referiez autrement ? » ; ne pas défendre le produit |
| 10 min | Clôture | niveau d'engagement (0 à 4) ; prochain rendez-vous ; remerciements |

### 13.4 Grille d'observation (une par scénario)

| Champ | À noter |
|---|---|
| Participant, rôle, appareil, réseau, date | |
| Scénario, heure de début et de fin | **temps réel** |
| **Hésitations** | où (écran, élément), durée au-delà de 3 s, ce que la personne dit ou cherche |
| **Erreurs** | type : saisie (mauvais montant, mode), navigation (mauvais écran), compréhension (mauvaise interprétation d'un mot ou d'un statut) ; récupérée ou non |
| Aides demandées ou données | combien, à quel moment |
| Mots employés par la personne | vocabulaire spontané (« caution », « reçu », « arriéré ») pour ajuster les libellés |
| Réussite | complète, partielle avec aide, échec |
| Citations | phrases exactes, sans interprétation |
| Gravité proposée | S1 à S4 (11.1) ; une erreur de montant constatée est **toujours S1** |
| Photo | **jamais** de donnée réelle ; démonstration seulement |

### 13.5 Modèle de rapport de séance

```markdown
# Rapport de séance — {agence} — {JJ/MM/AAAA}
- Participants et rôles :  Observateur :  Appareil et réseau :  Organisation utilisée (démo / setup) :
## Résultats chiffrés
| Scénario | Temps (médiane) | Réussite sans aide | Erreurs | Aides |
## Ce qui a bien marché (3 points)
## Les 5 principaux points de friction (écran, ce qui s'est passé, citation, gravité S1–S4)
## Erreurs de compréhension de vocabulaire
## Bloquants (S1 et S2)
## Engagement observé (0 à 4) et suites demandées par l'agence
## Corrections proposées, priorisées (section 13.6) et décisions à enregistrer (DP-xx)
```

### 13.6 Transformer un retour en correction priorisée

1. **Consigner** dans le journal des retours (console, `feedback_reports`) avec l'agence, le rôle, l'écran, la citation. Un retour oral est écrit **le jour même**.
2. **Qualifier** : bogue, friction, demande, question ; **reproduire** sur l'essai ; chercher la règle `R-xxx` ou le cas `E-xx` concerné. Un retour qui contredit une règle d'argent est une question de **règle** (à enregistrer dans `DOMAINE.md`), jamais corrigé « au passage ».
3. **Évaluer la gravité** (S1 à S4, 11.1) puis la priorité : `priorité = gravité × fréquence (nombre de personnes et de fois) × portée (nombre d'agences concernées)` [proposition].
4. **Arbitrer avec la règle du dossier** (§20.3, §21) : si au moins **trois clients** en ont besoin, la demande devient un **paramètre** ; sinon elle est refusée ou facturée comme service. Les règles d'argent ne sont **jamais** personnalisées.
5. **Décider** : corriger, ajouter un paramètre, expliquer (texte), former, refuser, ou ouvrir une entrée `DP-xx`.
6. **Écrire le critère de vérification** (« une caissière encaisse un paiement partiel en moins de 30 s sans aide ») et le test de non-régression (11.3).
7. **Livrer** dans le calendrier (`DEPLOYMENT.md`), derrière un interrupteur si c'est une fonction ; **confirmer** avec l'agence que le point est résolu ; mesurer à J+7.

## 14. Récapitulatif : indispensable avant le pilote et ce qui peut venir pendant les essais

| Indispensable avant le premier pilote | Peut venir pendant les essais |
|---|---|
| Tests d'or E-01 à E-48 de T1 et T2, propriétés P1 à P16 | E-34 (clôture de caisse) et la propriété liée, avec F19 |
| Tests de base : isolation, immutabilité, contraintes, migrations sur copie | Rôle de sauvegarde et réinjection d'une seule organisation automatisés |
| Tests d'API : droits, isolation, idempotence, contrat, en-têtes, journaux | Test de compatibilité du front N avec le serveur N−1 |
| Parcours 1 à 4 de bout en bout (ordinateur et mobile), parcours 5 (aperçu) | Parcours 5 complet (travaux, documents), analyse dynamique des pages |
| Tests visuels de la quittance (20 cas, A4 et A5) ; compte rendu | Comparaison visuelle étendue, tests d'impression automatisés |
| Fichiers d'import imparfaits et piégés | Fichiers réels anonymisés des pilotes (avec accord écrit) |
| Mesure du temps pour encaisser, poids des pages, accessibilité (automatique, clavier, lecteur d'écran) | Test de charge de début de mois |
| Première restauration chronométrée, procédure d'incident jouée | Test d'intrusion externe (avant la version commerciale) |
| Auto-test d'intrusion de deux jours | Couverture de code mesurée, mutation |
| Séance de recette R1 à R4, R7, R9 avant le pilote | Cycle hebdomadaire de recette pendant le pilote |

## Questions ouvertes

1. **Capacité (n° 14, H11)** : *tranchée par DP-54 : un développeur, 25 heures par semaine.* Le calendrier des tests de la section 10 est établi pour une seule personne ; la roadmap en tire les durées.
2. **Couverture réelle des délais de correction** (11.2) : jours et heures de l'équipe, week-end ; l'agence accepte-t-elle ces délais ? Les agences ouvrent-elles le samedi ?
3. **Appareil réel d'entrée de gamme** : budget et modèle ; l'équipe en possède-t-elle déjà un ?
4. **Impression de la quittance** (question d'`API_CONTRACTS.md`) : à partir du PDF ou d'un aperçu HTML ? Conditionne les tests visuels (deux rendus à comparer).
5. **Orthographe du montant en lettres** : *tranchée par DP-64 (traditionnelle, « francs CFA »)* ; les mentions restent à faire relire par un juriste (DP-62).
6. **Définition de « valides » dans E-20** : *tranchée (audit C2-046) : 197 lignes créables (valides ou avec avertissement, dont 2 doublons de téléphone proposés à la fusion), 3 erreurs ; `DOMAINE.md` E-20.*
7. **Rejeu des 44 tests du schéma** : jusqu'ici exécutés sur PGlite (WebAssembly) ; les rejouer sur Neon (DP-55), en version 16 ou plus récente, et ajouter la concurrence (prototypes b et c).
8. **Fichiers réels des pilotes** : jusqu'où les utiliser pour les tests d'import (accord, anonymisation) ?
9. **Test d'intrusion externe avant le pilote** (n° 58) : budget ; sinon auto-test seulement, risque accepté par écrit.
10. **Séances sur place chez une agence** : consentement des personnes observées et information sur la prise de notes ; formulation du consentement [À VÉRIFIER AVEC UN JURISTE].
11. **Promesse « tenue », génération des échéances, doublon de référence (R-041)** : *tranchées par R-101, R-098 à R-100 et R-102 (DP-58) ; les tests d'or correspondent aux cas E-39 à E-44.* Les règles restent à confirmer avec l'agence pilote à la découverte.

## Décisions à enregistrer dans DECISIONS.md

Mise à jour du 30/10/2026 : la n° 80 est prise par DP-56 ; la n° 84 reste ouverte pour la couverture horaire. Le cadre des tests est celui du dossier (§19) et des documents techniques. Décisions de ce document, à la suite de la n° 77 :

| N° | Décision | Recommandation de ce document |
|---|---|---|
| 78 | Tests de bout en bout et d'accessibilité automatique : Playwright (Chromium), émulation mobile 390 px, moteur d'analyse d'accessibilité intégré | Oui (section 5, 8.5) ; Playwright est déjà installé dans l'environnement de la maquette |
| 79 | Tests de propriétés : `proptest` pour le crate `domain` et le livre | Oui (2.2) |
| 80 | Tests d'intégration sur un PostgreSQL réel avec base éphémère par test (`#[sqlx::test]`) et rôles de production, jamais de simulation de la base | Oui (1.3, 3) |
| 81 | Profil « Android d'entrée de gamme, réseau lent » (processeur 4×, « 4G lente » et 3G) et achat d'un appareil réel | Oui (8.1) ; budget de l'appareil à valider |
| 82 | Chaîne de CI en étapes (rapide, intégration, nocturne, avant livraison) ; PR sous 15 minutes ; pas de nouvel essai automatique sur les tests d'argent et d'isolation ; pas de seuil de couverture au pilote | Oui (section 10) |
| 83 | Tests visuels du PDF : texte extrait, une page, polices, métadonnées, image de référence approuvée par un humain | Oui (section 6) ; dépend du moteur PDF (n° 7) |
| 84 | Gravités S1 à S4 (erreur de montant ou d'isolation toujours S1) et délais de correction cibles | Oui (11) ; couverture horaire à valider |
| 85 | Protocole de recette : séances de 90 minutes sur place, animateur et observateur, démonstration ou `setup` seulement, aucun enregistrement de données réelles | Oui (13) |
