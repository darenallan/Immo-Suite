# Immo Suite — Audit de cohérence, deuxième vague

**Mise à jour du 30/10/2026 (phase B).** Les sections A à E décrivent l'état des documents **avant** correction. Les décisions de Mimis sont en section F, les corrections appliquées en section G, les constats restés ouverts en section H et les vérifications refaites en section I.

## A. Synthèse

**Verdict.** Non, la roadmap ne peut pas être lancée en l'état. Trois blocages : la nouvelle stratégie n'est enregistrée nulle part (décision n° 19 ouverte, `CLAUDE.md` interdit encore tout code avant un Go de phase 0), la capacité de développement et l'hébergement ne sont pas décidés, et sept règles de T1 manquent dans `DOMAINE.md`. Les arithmétiques (40 cas chiffrés recalculés), les renvois (426 références de section, R, E, DP, F) et le jeu de données de la maquette (28 contrôles sur 28) sont sains. La majorité des 56 constats sont des corrections de texte (30 de nature MÉCANIQUE) ; 26 demandent une décision de Mimis.

Audit précédent : `AUDIT_COHERENCE.md` (constats traités par DP-29 à DP-43). Limite : l'auditeur est le même modèle que le rédacteur de ces documents ; une relecture humaine des constats « BLOQUE » est recommandée.

### Nombre de constats par gravité, par type et par nature

| Gravité | Contradiction | Incohérence | Manque | Ambiguïté | Doublon | Terme obsolète | Invention non sourcée | MÉCANIQUE | DÉCISION | Total |
|---|---|---|---|---|---|---|---|---|---|---|
| BLOQUE LA ROADMAP | 2 | 0 | 3 | 2 | 0 | 0 | 0 | 1 | 6 | **7** |
| BLOQUE LE PILOTE | 2 | 0 | 5 | 1 | 0 | 0 | 0 | 0 | 8 | **8** |
| IMPORTANT | 4 | 8 | 7 | 1 | 2 | 6 | 0 | 17 | 11 | **28** |
| MINEUR | 0 | 8 | 2 | 1 | 0 | 1 | 1 | 12 | 1 | **13** |
| **Total** | **8** | **16** | **17** | **5** | **2** | **7** | **1** | **30** | **26** | **56** |

### Top 10 des points à trancher avant la roadmap

1. **C2-001** : Enregistrer la stratégie (DP-52) et les critères de continuation du pilote.
2. **C2-003** : Nombre de personnes et d'heures par semaine (décision n° 14).
3. **C2-007** : Premier segment de pilote : agence (fin de T2) ou propriétaire direct (fin de T1).
4. **C2-004** : Hébergement, PostgreSQL managé, rôle de sauvegarde (n° 5, 21, 46, 86) et budget.
5. **C2-005** : Valider en bloc les 32 décisions « avant la tranche verticale » (dont 20 techniques).
6. **C2-006** : Sept règles de T1 : jours 29–31, mois partiel, bail à date passée, promesse tenue, doublon de référence, taux de recouvrement, année de numérotation.
7. **C2-013** : TOTP du gérant obligatoire avant la bascule en live ? (n° 50)
8. **C2-009** : Accepter la remise à zéro d'une organisation en phase setup et les exceptions à R-005 / R-007 (n° 25, 40).
9. **C2-012** : Avis juridique et CIL : qui, quand, risque accepté par écrit ou non.
10. **C2-014** : Éditeur, nom du produit, domaine (n° 13).

### Fichiers lus (date de dernière modification) et fichiers attendus mais absents

| Fichier | Dernière modification | Lignes | Taille |
|---|---|---|---|
| `Immo-Suite_Dossier-de-cadrage.docx` | 24/10/2026 21:41 | — | 94 Ko |
| `DECISIONS.md` | 30/10/2026 19:59 | 714 | 71 Ko |
| `CONTEXTE.md` | 24/10/2026 21:41 | 249 | 18 Ko |
| `DOMAINE.md` | 24/10/2026 21:41 | 424 | 37 Ko |
| `UI_UX_DESIGN.md` | 25/10/2026 11:59 | 914 | 65 Ko |
| `MAQUETTE.md` | 25/10/2026 11:59 | 602 | 43 Ko |
| `CLAUDE.md` | 30/10/2026 19:59 | 96 | 8 Ko |
| `AUDIT_COHERENCE.md` | 24/10/2026 21:41 | 625 | 65 Ko |
| `ARCHITECTURE.md` | 30/10/2026 16:33 | 1020 | 101 Ko |
| `DATABASE_SCHEMA.md` | 30/10/2026 19:23 | 3174 | 219 Ko |
| `SECURITY.md` | 30/10/2026 19:21 | 757 | 91 Ko |
| `API_CONTRACTS.md` | 30/10/2026 19:48 | 1423 | 129 Ko |
| `CODING_RULES.md` | 30/10/2026 19:51 | 423 | 51 Ko |
| `TESTING_QA.md` | 30/10/2026 19:55 | 554 | 62 Ko |
| `DEPLOYMENT.md` | 30/10/2026 19:58 | 613 | 72 Ko |
| `maquette/NOTES.md` | 25/10/2026 19:11 | 109 | 19 Ko |
| `maquette/README.md` | 25/10/2026 19:11 | 90 | 6 Ko |
| `site/NOTES.md` | 25/10/2026 21:39 | 31 | 4 Ko |
| `site/README.md` | 25/10/2026 21:39 | 92 | 7 Ko |
| `film/SCENARIO.md` | 25/10/2026 13:40 | 144 | 16 Ko |
| `film/VOIX_OFF.md` | 25/10/2026 12:43 | 96 | 5 Ko |
| `film/NOTES.md` | 25/10/2026 14:09 | 83 | 8 Ko |
| `flyer/README.md` | 25/10/2026 19:42 | 81 | 7 Ko |
| `infographie/README.md` | 30/10/2026 12:12 | 38 | 2 Ko |
| `maquette/` (index.html, css, js, data, scripts, assets ; `_captures/` non ouvert) | 25/10/2026 | 384 fichiers | — |
| `site/immo-suite/index.html`, `flyer/build/*.html`, `film/SCENARIO.md`, `film/VOIX_OFF.md`, `infographie/` (textes seulement) | 25–30/10/2026 | — | — |
| Dossier de cadrage converti en Markdown (hors du dossier du projet) : 1 276 lignes, 26 sections + annexe | 24/10/2026 | 1 276 | 127 Ko |

**Lecture.** Les documents `.md` racine ont été lus en entier, sauf `DATABASE_SCHEMA.md` §3 « Référence des tables » (l.296-2255, 63 fiches de tables, générées depuis le catalogue) et `API_CONTRACTS.md` §6 (l.494-1078, exemples JSON des cinq parcours) : ces deux parties ont été traitées par script (extraction des tables, fonctionnalités, règles, montants) et lues par échantillon (`payments`, `leases`, `sessions`, parcours 1). Les dossiers marketing ont été contrôlés sur leurs textes visibles, sans visionner les vidéos ni les PDF.

**Absents ou non créés.** `ROADMAP.md`, `TRACABILITE.md`, `specs/`, `contenus/` (donc `GUIDE_ENTRETIEN.md`, `LETTRE_INTENTION.md` cités par `CLAUDE.md`), `TERRAIN.md` (jamais annoncé : aucune occurrence de `TERRAIN.md` ni de `[DÉPEND DU TERRAIN]` dans les documents), `donnees/verifier.html` et `donnees/verifier.js` (cités par `CLAUDE.md` et `MAQUETTE.md` ; le fichier réel est `maquette/scripts/verifier-donnees.js`), `api/CHANGELOG.md` (futur), dépôt de code (le dossier n'est pas un dépôt Git). Pandoc n'est pas installé : le `.docx` a été converti par `python-docx` (paragraphes, titres et tableaux dans l'ordre du document), sans modification de l'original.

### Vérifications mécaniques faites par script, et résultat

| # | Vérification | Résultat |
|---|---|---|
| 1 | Identifiants F-xx : définis dans le dossier, référencés dans les 14 documents | 39 définis (F38 absent du dossier, noté par `CONTEXTE.md`) ; aucune référence orpheline |
| 2 | Identifiants R-xxx : définis dans `DOMAINE.md`, référencés partout | 97 définis, R-001 à R-097 continus ; 0 référence orpheline ; 0 doublon |
| 3 | Identifiants E-xx | 39 définis (E-01 à E-38 et E-13b) ; 0 référence orpheline |
| 4 | Décisions DP-xx | DP-01 à DP-51 continues, sans doublon ; 0 référence à un DP inexistant ; DP-52 seulement cité comme prochain numéro |
| 5 | Décisions ouvertes n° 1 à 95 | Continues, sans doublon : 5 prises (1, 4, 8, 9, 16), 90 ouvertes ; les statuts du tableau concordent avec DP-28/34, 09, 32, 33, 44 |
| 6 | Renvois « `FICHIER.md` §x » | 426 références de section vérifiées contre les titres numérotés : 0 cassée |
| 7 | Fichiers et dossiers cités et absents du disque | 7 : `GUIDE_ENTRETIEN.md`, `LETTRE_INTENTION.md`, `ROADMAP.md`, `api/CHANGELOG.md`, `donnees/verifier.html`, `donnees/verifier.js`, `tests/fixtures/golden/E-xx.json` (`ROADMAP.md`, `api/CHANGELOG.md` et `tests/fixtures/golden/E-xx.json` sont futurs ; `GUIDE_ENTRETIEN.md` et `LETTRE_INTENTION.md` sont obsolètes ou à remplacer : C2-002, C2-011 ; `donnees/verifier.*` est erroné : C2-036) |
| 8 | En-têtes « Dépend de » (plages DP et n°) | 7 documents techniques avec plage périmée (ex. ARCHITECTURE : DP-01 à DP-44, n° 1 à 18 ; réel : DP-51, n° 95) : C2-047 |
| 9 | Cas chiffrés de `DOMAINE.md` et chiffres répétés (40 calculs : E-05, E-06, E-13, E-13b, E-16, E-18, E-23, E-24, E-26, E-28, E-31, E-32, retards en jours, volumes de la maquette, durées des vidéos, conversions €/FCFA) | 40 sur 40 exacts. Formule entière `(base × taux_bp + 5 000) / 10 000` d'ARCHITECTURE §4.2 : E-16 = 8 000, E-26 = 25 000, E-18 = 32 000 ; 13/11/2026 est un vendredi |
| 10 | Coûts de `DEPLOYMENT.md` §1.3 | Production 26 à 80 € (annoncé 30 à 75), essai 16 à 31 € (annoncé 15 à 30), ensemble 42 à 111 € (annoncé 50 à 120) : C2-023 |
| 11 | Contrastes de `UI_UX_DESIGN.md` (25 paires recalculées, formule WCAG 2.1) | 24 exacts à ± 0,1 ; 1 divergence de paire avec `MAQUETTE.md` (repère « À venir ») : C2-045 |
| 12 | Matrice de permissions : `CONTEXTE.md` contre `SECURITY.md` et dossier §11.2 | 13 lignes × 5 rôles : 0 écart avec SECURITY ; 1 faute de frappe du dossier (« travaux ») ; `API_CONTRACTS.md` §4.6 relue à la main : cohérente, plus 2 colonnes (P, Col) et 4 trous (C2-030) |
| 13 | Termes obsolètes (enquête, Go/No-Go, phase 0, entretien, avant le Go, lettre d'intention, GUIDE_ENTRETIEN) | Présents dans CLAUDE (7 lignes), CONTEXTE (9), DECISIONS (≈ 40), MAQUETTE (≈ 15), DOMAINE (3), UI (3), TESTING (1), DEPLOYMENT (3), SECURITY (1), ARCHITECTURE (2), site, flyer, film. Marqueurs `[DÉPEND DU TERRAIN]` et `TERRAIN.md` : 0 occurrence |
| 14 | Maquette : `node scripts/verifier-donnees.js` (exécution en lecture seule) | 28 contrôles réussis sur 28. Volumes : 40 lots, 4 mandants, 31 locataires, 33 baux, 94 paiements (dont 1 contre-passation), 93 quittances, prochain numéro HOR-2026-000094 ; `importExemple` : 200 lignes, 197 valides, 3 erreurs |
| 15 | `DATABASE_SCHEMA.md` : décompte des objets | 63 tables (conforme à l'en-tête), 5 vues ; 25 tables en écriture seule (liste de §4.2 comptée : 25) ; 73 déclencheurs et 115 politiques non vérifiables (extraits SQL partiels) : C2-038 |
| 16 | `ARCHITECTURE.md` §2.3 contre `DATABASE_SCHEMA.md` §3 (noms de tables) | 3 noms d'ARCHITECTURE absents du schéma (`lease_notices`, `security_deposits`, `tasks`) : C2-026 |
| 17 | Matrice F01–F40 (écran, route, table, test) | Annexe 1 ; 12 fonctionnalités de V1 sans test direct par identifiant, F17 et F22 sans route : C2-031 |
| 18 | Matrice R-xxx (base, API, UI, test) | Annexe 2 ; 38 règles sans test cité par identifiant : C2-032 |
| 19 | Promesses commerciales (site, flyer, film, infographie) : F26, F27, hors-connexion, prix, chiffres | 1 écart (site : « Prévu plus tard ») ; aucun prix ni gain chiffré dans le film, le flyer ni l'infographie ; 1 phrase chiffrée dans `MAQUETTE.md` : C2-024, C2-055 |
| 20 | `CLAUDE.md` : longueur, statuts, structure | 96 lignes (< 100) ; statuts « Créé » exacts sauf l'index (ARCHITECTURE « à créer ») : C2-036 |

## B. Constats

### C2-001 — Le changement de stratégie n'est enregistré nulle part et quatre décisions « Prise » disent l'inverse

- **Gravité** : BLOQUE LA ROADMAP
- **Type** : Contradiction
- **Nature** : DÉCISION
- **Où** :
  - `DECISIONS.md`, tableau des décisions ouvertes, n° 19 : « Stratégie : pilote réel sans enquête préalable (ajoutée le 30/10/2026) | Remplacer DP-07, DP-10 et DP-34 pour la phase 0 ; garder la phase 0 | … | Avant tout code applicatif | Ouverte ».
  - `DECISIONS.md`, DP-07 : « Aucun développement de l'application avant le Go de la phase 0 (portée précisée par DP-35) » ; DP-10, DP-34 et DP-35 (« aucun code applicatif avant le Go ») ont le statut **Prise**.
  - `ARCHITECTURE.md` Questions ouvertes n° 1, `DEPLOYMENT.md` Questions ouvertes n° 16, `TESTING_QA.md` PP-32 : tous renvoient à « une entrée DP-xx » qui n'existe pas.
- **Constat** : La stratégie a changé (plus d'enquête avant le développement ; pilote avec données réelles ; ajouts pendant les essais), mais aucune entrée DP-xx ne l'enregistre. Quatre décisions « Prise » (DP-07, DP-10, DP-34, DP-35) disent l'inverse et restent en vigueur. Le dossier de cadrage dit aussi : « Le chemin critique est la phase 0 : sans validation, on ne code pas » (§17).
- **Impact** : Tant que DP-52 n'existe pas, `CLAUDE.md` interdit tout code applicatif (voir C2-002), la roadmap n'a pas de point de départ défini et aucun critère ne dit comment le pilote décide de continuer ou d'arrêter.
- **Document propriétaire** : `DECISIONS.md`
- **Recommandation** :
  `DECISIONS.md` : ajouter **DP-52** (date du jour de validation), fermer la décision n° 19 et marquer « Remplacée par DP-52 » les entrées DP-07 (portée « avant tout code »), DP-10, DP-34 (et donc DP-28 déjà remplacée) et DP-35. DP-11 reste prise et est précisée (le prototype PDF devient le prototype (a) d'`ARCHITECTURE.md` §7).

  Texte proposé :

  ```markdown
  ### DP-52 — Pas d'enquête terrain avant le développement ; pilote avec données réelles

  - **Date** : JJ/MM/AAAA
  - **Contexte** : DP-07, DP-10, DP-28/DP-34 et DP-35 supposaient maquette, puis enquête, puis Go, puis code. La stratégie a changé.
  - **Décision** : aucune enquête terrain ni seuil Go/No-Go avant le développement. Une première version (T1 puis T2, dossier §9.1) est construite et testée par des agences pilotes avec leurs vraies données. Les ajouts se font pendant les essais (T3 et retours). Les hypothèses H1 à H12 se valident pendant le pilote. La maquette est conservée comme support de démonstration.
  - **Alternatives écartées** : garder la phase 0 (retarde le premier retour réel) ; enquête allégée (non retenue par la personne qui pilote le projet).
  - **Conséquences** : DP-07, DP-10, DP-34 et DP-35 sont remplacées. Les seuils d'arrêt ou de continuation du pilote sont ceux de l'entrée DP-53 [à fixer]. Écart assumé par rapport au dossier §17 (phase 0) et §26 n° 1.
  - **Statut** : Prise
  - **Liens** : décision n° 19, DP-07, DP-10, DP-11, DP-28, DP-34, DP-35.
  ```
- **Alternative** : Conserver la phase 0 sous une forme réduite (maquette + prototype PDF seulement) : écartée par la décision de Mimis ; à ne retenir que si Mimis la rétablit.
- **Question pour Mimis** : Quels critères décident, à la fin du pilote, de continuer, corriger ou arrêter ? Proposition à confirmer : reprendre les critères de sortie de la phase 4 du dossier (au moins 60 % des paiements saisis sous 48 h ; au moins 2 clients acceptent de payer au tarif cible). Le premier segment de pilote est-il bien « agences » (voir C2-007) ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-002 — CLAUDE.md interdit encore tout code applicatif avant un Go de phase 0

- **Gravité** : BLOQUE LA ROADMAP
- **Type** : Contradiction
- **Nature** : MÉCANIQUE
- **Où** :
  - `CLAUDE.md` l.14 : « **Bloc 1 (= phase 0, dossier §17) : maquette interactive** … puis enquête terrain en parallèle (DP-07, DP-10, DP-31). Aucun code applicatif avant le Go de la phase 0 (DP-35). »
  - `CLAUDE.md` l.10 : « Aucune enquête terrain n'a eu lieu. Les prix, volumes et workflows sont des hypothèses [À valider]. »
  - `CLAUDE.md` l.49 : `contenus/` « Contient `GUIDE_ENTRETIEN.md` … et `LETTRE_INTENTION.md` ».
- **Constat** : Le fichier chargé à chaque session par Claude Code interdit encore tout code applicatif et décrit une phase 0 abandonnée. Il liste deux fichiers (`GUIDE_ENTRETIEN.md`, `LETTRE_INTENTION.md`) qui n'existent pas et dont le premier n'a plus d'objet. La règle « Aucun code applicatif avant le Go » est la première chose qu'un agent lira.
- **Impact** : Un agent suit la règle écrite et refuse de démarrer la tranche verticale, ou, s'il la contourne, enfreint une règle de `CLAUDE.md`.
- **Document propriétaire** : `CLAUDE.md` (après DP-52)
- **Recommandation** :
  `CLAUDE.md`, section « Phase actuelle », remplacer par : « **Construction de la première version (T1 puis T2)**, testée par des agences pilotes avec leurs données réelles (DP-52). Pas d'enquête avant le développement. Le développement démarre par la tranche verticale (`ARCHITECTURE.md` §9.1) dès que les décisions bloquantes de `TESTING_QA.md` PP-32 sont enregistrées. La maquette (`maquette/`) reste une démonstration. » ; ligne 10 : « Aucune enquête terrain n'a eu lieu et aucune n'est prévue avant le développement : les prix, volumes et workflows sont des hypothèses validées pendant le pilote [À valider]. » ; ligne 49 : retirer `GUIDE_ENTRETIEN.md`, garder `contenus/` pour l'engagement de confidentialité, la convention de pilote et les guides d'accueil (voir C2-011).
- **Alternative** : Aucune.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-003 — Un développeur à 25 h/semaine ou deux personnes : décision n° 14 ouverte, effort calculé supérieur aux phases du dossier

- **Gravité** : BLOQUE LA ROADMAP
- **Type** : Ambiguïté
- **Nature** : DÉCISION
- **Où** :
  - Dossier §17 : « Capacité : un seul développeur, assisté par Claude Code, environ 25 heures par semaine » ; CONTEXTE H11 : « environ 25 heures par semaine ».
  - `ARCHITECTURE.md` §1.3 : « Simplicité d'exploitation pour deux personnes » ; §3.4 : « deux développeurs » ; `CODING_RULES.md` §8.3, `TESTING_QA.md` §1.1, `DEPLOYMENT.md` principe : « deux personnes exploitent le service » ; `SECURITY.md` H-S3.
  - `DECISIONS.md` n° 14 (« Capacité de développement ») : Ouverte. `ARCHITECTURE.md` Questions ouvertes n° 2, `CODING_RULES.md` n° 1, `TESTING_QA.md` n° 1, `DEPLOYMENT.md` n° 15 le signalent.
- **Constat** : Cinq documents techniques supposent deux personnes (revue croisée, test de restauration alterné, double contrôle des gardes, accès d'urgence à deux) ; le dossier et CONTEXTE parlent d'un développeur à 25 h/semaine. Calcul fait sur les complexités du dossier (§7, S = 0,5 semaine, M = 1 à 2, L = 3 à 5, hypothèse du calcul) : T1 ≈ 15,5 à 28,5 semaines-développeur, T2 ≈ 6,5 à 11,5, T3 ≈ 7,5 à 15. Les phases du dossier (3–4 + 6–8 + 5–7 = 14 à 19 semaines pour fondations, T1 et T2) sont donc inférieures à la somme de ses propres complexités (22 à 40).
- **Impact** : Sans nombre de personnes ni d'heures, la roadmap ne peut pas fixer de dates, et plusieurs règles de processus (revue obligatoire par l'autre développeur, PP-14, PP-24 « par la personne qui n'a pas écrit le module ») sont inapplicables.
- **Document propriétaire** : `DECISIONS.md` (n° 14), `ARCHITECTURE.md` pour l'exploitation
- **Recommandation** :
  `DECISIONS.md` : DP-54 « Capacité : N personnes, H heures par semaine chacune, période de référence ». `ROADMAP.md` : dériver les durées de ce nombre. Si un seul développeur : appliquer l'exception de `CODING_RULES.md` §8.3 (relecture différée) et remplacer dans `TESTING_QA.md` PP-24 et `DEPLOYMENT.md` §5.6 « l'autre développeur » par une personne externe nommée.
- **Alternative** : Rester sur deux personnes et rédiger la roadmap en supposant les heures de chacune (à écrire dans DP-54).
- **Question pour Mimis** : Combien de personnes écrivent du code, et combien d'heures par semaine chacune, pendant les 6 prochains mois ? Qui est la deuxième personne pour les revues et la restauration ?
- **Décision de Mimis** : [ ] OK recommandation  [ ] OK alternative  [x] Autre : « un développeur, 25 h/semaine » (30/10/2026)

### C2-004 — Hébergement, exploitation de PostgreSQL et sauvegarde logique : quatre décisions liées, aucune prise

- **Gravité** : BLOQUE LA ROADMAP
- **Type** : Manque
- **Nature** : DÉCISION
- **Où** :
  - `DECISIONS.md` n° 5 (hébergement), n° 21 (PostgreSQL managé ou non), n° 46 (sauvegarde logique sous RLS forcée), n° 86 (grille et coûts) : toutes Ouvertes, échéances « Phase 1 », « Avec la décision n° 5 », « Avant de choisir l'hébergeur ».
  - `DATABASE_SCHEMA.md` §6.9 : « `pg_dump` fixe `row_security = off` … La copie logique quotidienne d'`ARCHITECTURE.md` §10.4 ne peut donc pas s'exécuter avec `immo_owner` ni avec `immo_app`. »
  - `DEPLOYMENT.md` §1.2, grille : « Un rôle d'administration avec `BYPASSRLS` utilisable pour la copie logique … sinon, on ne force pas la RLS (décision n° 46) | oui, à trancher **avant** de choisir ».
- **Constat** : Le choix de l'hébergeur conditionne la migration initiale (rôles, extension `btree_gist`, RLS forcée ou non), la sauvegarde (copie logique chez un second fournisseur), le moteur PDF (taille du serveur) et le budget. `ARCHITECTURE.md` a recommandé une option sous trois conditions dont la première est une mesure (prototype e) qui n'a pas été faite.
- **Impact** : La tranche verticale (`ARCHITECTURE.md` §9.1 : « environnement d'essai déployé par la CI, nom de domaine et HTTPS ») ne peut pas démarrer, et la roadmap ne peut pas chiffrer l'infrastructure.
- **Document propriétaire** : `ARCHITECTURE.md` §11 et `DEPLOYMENT.md` §1
- **Recommandation** :
  Séquencer dans la roadmap : (1) prototype (e) de latence réelle depuis Ouagadougou (1 jour) ; (2) vérification auprès de 2 ou 3 hébergeurs candidats de la grille `DEPLOYMENT.md` §1.2 (surtout `BYPASSRLS` pour la sauvegarde et la restauration à un instant donné) ; (3) DP-55 « Hébergement et exploitation de PostgreSQL » qui ferme n° 5, 21, 46 et 86 ensemble. Ne pas écrire la migration initiale avant DP-55.
- **Alternative** : Si le budget l'impose : PostgreSQL auto-hébergé (n° 21, option b) avec outil de sauvegarde continue, au prix d'une charge d'exploitation (`DEPLOYMENT.md` §1.2).
- **Question pour Mimis** : Quel budget mensuel d'infrastructure est acceptable (`DEPLOYMENT.md` Questions ouvertes n° 18 : 50 à 120 € estimés) ? L'hébergement en Europe de l'Ouest est-il acceptable sans avis préalable de la CIL (voir C2-012) ?
- **Décision de Mimis** : [ ] OK recommandation  [ ] OK alternative  [x] Autre : « on utilisera Neon pour la base de données » (30/10/2026)

### C2-005 — 32 décisions « avant la tranche verticale » (dont 20 techniques) déjà traitées comme acquises, à valider en bloc

- **Gravité** : BLOQUE LA ROADMAP
- **Type** : Manque
- **Nature** : DÉCISION
- **Où** :
  - `DECISIONS.md` : 32 décisions ouvertes ont une échéance « Avant la migration initiale », « Avant la tranche verticale », « Tranche verticale » ou « Phase 1 » : 12 de portée (n° 2, 5, 6, 7, 14, 19, 20, 21, 23, 46, 86, 87), traitées par les constats sur la stratégie, la capacité et l'hébergement, et 20 techniques (n° 18, 30, 31, 32, 34, 35, 41, 61, 62, 64, 67, 69, 70, 73, 74, 75, 76, 77, 79, 80). Liste en annexe 4.
  - Chaque document technique répète : « Aucune décision technique de ce document n'est prise tant qu'une entrée DP-xx n'existe pas ».
- **Constat** : 90 décisions sont ouvertes sur 95 ; 5 sont prises. Les documents techniques traitent déjà leurs recommandations comme acquises (monorepo, Axum, sessions serveur, UUID v4, texte + CHECK, bornes 1 à 999 999 999, `row_version`, etc.), alors que `CODING_RULES.md` §8.5 interdit d'appliquer une recommandation sans entrée DP-xx.
- **Impact** : Le premier commit de code contredit les règles du projet (`CLAUDE.md` : « Mettre à jour DECISIONS.md à chaque décision structurante, **avant** de l'appliquer dans le code »).
- **Document propriétaire** : `DECISIONS.md`
- **Recommandation** :
  Organiser **une séance de validation en bloc** avant la tranche verticale : pour chacune des 20 décisions techniques (et les 12 de portée, voir les constats liés), Mimis répond « recommandation » ou « autre » dans une grille (annexe 4). Enregistrer le résultat en une entrée par thème (pile backend n° 20/61/62/67 ; base n° 30 à 35, 41 ; qualité n° 69 à 77, 79, 80 ; dépôt n° 23, 87). Points à examiner séparément car ils changent le schéma : n° 41 (chevauchement avec les baux terminés), n° 64 (`row_version` dès le pilote), n° 35 (bornes des montants), n° 46 (voir C2-004).
- **Alternative** : Valider toutes les recommandations des documents techniques sauf celles que Mimis marque, en une seule entrée « DP-56 — pile technique de référence ».
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-006 — Sept règles de T1 absentes de DOMAINE.md

- **Gravité** : BLOQUE LA ROADMAP
- **Type** : Manque
- **Nature** : DÉCISION
- **Où** :
  - `ARCHITECTURE.md` §4.8 : « Points non définis par `DOMAINE.md`, à trancher avant le code de T1 : jour 29 à 31 dans un mois court ; premier mois partiel (prorata ou non) ; bail activé avec une date de début passée (génère-t-on les échéances passées ?) ».
  - `DATABASE_SCHEMA.md` §11.1 n° 1 (avenant daté dans le passé ou sur une échéance impayée), n° 6 (téléphone partagé), n° 7 (« Quand une promesse est-elle « tenue » »), n° 11 ; `ARCHITECTURE.md` Q4 et `API_CONTRACTS.md` Q1 (doublon de référence R-041 : bloquant ou non) ; Q5/Q14 (année de la numérotation) ; `DOMAINE.md` Questions ouvertes n° « Taux de recouvrement » : « Formule exacte … non précisée ».
  - `TESTING_QA.md` Q11 : « sans règle écrite, les tests d'or correspondants ne peuvent pas être écrits ; ils bloquent la fin de T1 ».
  - `DOMAINE.md`, section « Questions ouvertes » (5 entrées, l.420-424) : seule la formule du taux de recouvrement y figure ; les six autres questions n'y sont pas.
- **Constat** : Sept règles de T1 n'existent dans aucun document propriétaire : génération des échéances (jour 29–31, premier mois partiel, bail à date passée), promesse « tenue », doublon de référence externe, taux de recouvrement, année de numérotation, avenant sur échéance générée, téléphone partagé. `DOMAINE.md` est le propriétaire des règles métier et ne mentionne que la formule du taux de recouvrement, alors que `CODING_RULES.md` §9.4 impose d'y ajouter la question.
- **Impact** : F08, F09, F10, F11 et F12 (T1) ne sont pas spécifiables ; les tests d'or E-xx correspondants ne s'écrivent pas ; un agent risque de coder un comportement « raisonnable » sur de l'argent, ce que `CLAUDE.md` interdit.
- **Document propriétaire** : `DOMAINE.md`
- **Recommandation** :
  `DOMAINE.md` : (1) ajouter les 7 questions à « Questions ouvertes » maintenant (mécanique) ; (2) après réponse de Mimis, ajouter une règle R-098 et suivantes et un exemple E-39 et suivants par sujet, puis décision DP-58. Propositions des documents techniques à valider : jour 29–31 → dernier jour du mois ; premier mois partiel → loyer entier ; bail à date passée → échéances générées depuis le mois d'activation seulement ; promesse tenue → paiement ≥ montant promis à la date promise ou avant ; année de numérotation → année d'émission ; avenant → s'applique aux échéances postérieures à la fois à sa date d'effet et à sa date de saisie (`DATABASE_SCHEMA.md` §11.1 n° 1).
- **Alternative** : Coder les propositions comme options paramétrables par organisation : refusé par la règle d'or 12 (règles d'argent jamais sur mesure).
- **Question pour Mimis** : Pour chacun des sept sujets, la proposition ci-dessus est-elle acceptée ? Le doublon de référence externe est-il bloquant ou seulement un avertissement ? Deux locataires peuvent-ils partager le même téléphone (couple, famille) ? Quelle est la formule du taux de recouvrement ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-007 — Les décisions rattachées à la phase 0 (segment de pilote, prix, reddition, juridique) n'ont plus d'échéance

- **Gravité** : BLOQUE LA ROADMAP
- **Type** : Ambiguïté
- **Nature** : DÉCISION
- **Où** :
  - `DECISIONS.md` n° 2 : « Premier segment de pilote », échéance « Phase 0 », Ouverte ; n° 3 (grille de prix, « Phase 0 »), n° 10 (règle de reddition, « Phase 0 »), n° 12 (validation juridique, « Phase 0–1 »), n° 15 (données de test, « Phase 2 »), n° 7 (moteur PDF, « Phase 0 »).
  - Dossier §26 n° 2 : « Agences comme pilote principal … + 1–2 propriétaires directs ».
- **Constat** : Ces sept décisions étaient rattachées à une phase 0 qui n'existe plus. Leur nouvelle échéance n'est écrite nulle part. Le premier segment de pilote change le calendrier : un pilote « propriétaire direct » pourrait démarrer à la fin de T1, un pilote « agence » seulement à la fin de T2 (DP-29, ARCHITECTURE §9.2).
- **Impact** : La roadmap ignore à quelle date le premier client réel entre et quelles fonctions il doit avoir (reddition, import, console).
- **Document propriétaire** : `DECISIONS.md`
- **Recommandation** :
  `DECISIONS.md` : dans le tableau des décisions ouvertes, remplacer les échéances « Phase 0 / 1 / 2 / 3 » par des échéances de la nouvelle séquence : n° 2 → « Avant la roadmap » ; n° 3 → « Avant le premier pilote » (prix testés pendant le pilote) ; n° 7 → « Prototype (a), tranche verticale » ; n° 10 → « Avant la tranche 5 (T2) » ; n° 12 → « Avant le premier pilote » ; n° 15 → « Avant le premier pilote ». Enregistrer le premier segment dans DP-57.
- **Alternative** : Garder les échéances actuelles : écarté, elles désignent une phase supprimée.
- **Question pour Mimis** : Le premier pilote est-il une agence (reddition de comptes, import, console : fin de T2) ou un propriétaire direct (fin de T1) ? Combien de pilotes visés (le dossier §17 dit 3 à 5) ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-008 — Règles du compte rendu propriétaire (T2) non tranchées, dont R-071

- **Gravité** : BLOQUE LE PILOTE
- **Type** : Manque
- **Nature** : DÉCISION
- **Où** :
  - `DATABASE_SCHEMA.md` §11.1 n° 2 : « Sous un mandat à base « loyer », les charges encaissées sont-elles reversées au mandant (dans le net) ou conservées par l'agence ? E-28 calcule la commission (8 000) mais ne dit pas ce que devient la part de 10 000 » ; n° 3 (net et base négatifs) ; n° 4 (quelles dépenses entrent dans un compte rendu) ; n° 8 (reversement partiel) ; n° 10 (mandat clôturé en cours de mois) ; n° 12 (quels lots bloquent R-075).
  - `DOMAINE.md` R-071 : « [règle à confirmer sur le terrain] » ; `DECISIONS.md` n° 10 : Ouverte, « Phase 0 » ; DP-16 : « Si R-071 change, cette décision est à revoir ».
  - `API_CONTRACTS.md` Q13, `ARCHITECTURE.md` Q7 : mêmes questions.
- **Constat** : Le compte rendu propriétaire (F13, T2) repose sur une règle de reddition non confirmée (R-071) et sur six points que `DOMAINE.md` ne tranche pas. La confirmation « sur le terrain » n'a plus de moment défini depuis la suppression de la phase 0 : elle ne peut avoir lieu que pendant la découverte de l'agence pilote (`DEPLOYMENT.md` §8.1 étape 2) ou être décidée maintenant.
- **Impact** : Le calcul du net à reverser, le plus sensible pour une agence, peut être faux au premier mois réel ; la règle d'or 12 interdit de la rendre « sur mesure ».
- **Document propriétaire** : `DOMAINE.md`
- **Recommandation** :
  `DOMAINE.md` : ajouter les six questions à « Questions ouvertes » (mécanique) puis les trancher avant la tranche 5. Pour R-071 : remplacer « [règle à confirmer sur le terrain] » par « [à confirmer avec l'agence pilote à la découverte, avant la première production de compte rendu] » et garder la date d'encaissement comme règle par défaut (DP-16). Les propositions de `DATABASE_SCHEMA.md` §11.1 (reversement unique du montant exact ; net négatif refusé ; lots avec encaissement dans la période pour R-075) sont reprises comme valeurs par défaut en attendant.
- **Alternative** : Rendre la base de reddition (date d'encaissement ou d'échéance) paramétrable par mandat : contraire à la règle d'or 12 tant que trois clients ne la demandent pas.
- **Question pour Mimis** : Les charges encaissées (base « loyer » seul) sont-elles reversées au mandant ou conservées par l'agence ? Un reversement partiel est-il possible ? Un compte rendu à net négatif est-il refusé ou autorisé ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-009 — R-005 et R-007 contredits par la purge d'une organisation en phase setup et l'annulation d'import

- **Gravité** : BLOQUE LE PILOTE
- **Type** : Contradiction
- **Nature** : DÉCISION
- **Où** :
  - `DOMAINE.md` R-005 : « Rien ne se supprime. Un bien, lot ou locataire avec historique est archivé » ; R-007 : « Le journal d'audit est en écriture seule. Aucune modification ni suppression, même par le gérant ou HABEK ».
  - `DATABASE_SCHEMA.md` §4.2 : fonctions `cancel_import_batch` (supprime baux, locataires, lots, biens, mandants) et `purge_org_setup` ; §8.3 : « Elle supprime les données métier de cette organisation seule … (compteurs de numérotation et journal d'audit compris) ».
  - `ARCHITECTURE.md` §8.6 : « Cette purge contredit à la lettre R-005 … décision à enregistrer avant le premier pilote » ; `DECISIONS.md` n° 25 et n° 40 : Ouvertes, « Avant le premier pilote ».
- **Constat** : Deux exceptions à R-005 (annulation d'import, purge en phase `setup`) et une à R-007 (l'audit est purgé avec l'organisation) sont conçues et testées, mais `DOMAINE.md` ne les prévoit pas. R-080 autorise déjà « annulable tant que rien n'a été utilisé » sans lever la contradiction avec R-005.
- **Impact** : Une règle absolue (« Rien ne se supprime ») est violée par le produit prévu ; un auditeur ou une agence peut le constater.
- **Document propriétaire** : `DOMAINE.md`
- **Recommandation** :
  Après décision de Mimis sur n° 25 et n° 40, `DOMAINE.md` : R-005 devient « Rien ne se supprime, sauf : (a) l'annulation d'un import non utilisé (R-080) ; (b) la remise à zéro d'une organisation en phase `setup`, avec accord écrit de l'agence, tracée hors de l'organisation (DP-60). » ; R-007 : ajouter « … sauf avec l'organisation lors de la remise à zéro en phase `setup` ». Ajouter un exemple E-39 (purge en `setup` refusée en `live`).
- **Alternative** : Interdire toute purge : l'agence qui s'est trompée à l'import repart d'une nouvelle organisation (la seule voie après `live`).
- **Question pour Mimis** : Acceptez-vous que HABEK puisse remettre à zéro une organisation tant qu'elle est en phase de reprise (`setup`), avec l'accord écrit de l'agence ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-010 — Numérotation « sans trou » : séries d'essai, année d'émission et premier compte rendu non décidés

- **Gravité** : BLOQUE LE PILOTE
- **Type** : Ambiguïté
- **Nature** : DÉCISION
- **Où** :
  - `ARCHITECTURE.md` §8.6 et `DATABASE_SCHEMA.md` §4.3 : quittances de série `trial` en phase `setup`, série `official` à 1 à la bascule ; `DECISIONS.md` n° 36 : Ouverte.
  - `ARCHITECTURE.md` §4.4 : « L'année de la séquence est l'année de l'émission … À confirmer » ; `DATABASE_SCHEMA.md` Q8 et `API_CONTRACTS.md` Q14 : cas du paiement du 31/12 saisi le 02/01.
  - `DEPLOYMENT.md` Q7 : « la bascule `live` hors du début de mois laisse un premier mois mixte … Comment produire le premier compte rendu mensuel d'une agence sans mélange ? » ; Q8 : quittances d'essai pendant la double tenue.
  - `DOMAINE.md` R-050 : « par organisation et par année », sans série ni année d'émission.
- **Constat** : La numérotation « sans trou » (règle d'or 7) est définie différemment selon le document : un compteur par organisation et année (DOMAINE, ARCHITECTURE §4.4) ou par organisation, série et année (DATABASE_SCHEMA). La mention « essai » sur des quittances remises à de vrais locataires pendant la double tenue n'est pas validée par une agence. Le premier compte rendu réel mélange deux séries.
- **Impact** : Les premières quittances remises à de vrais locataires pourraient être sans valeur ou renumérotées ; la contrainte d'unicité et le contrôle nocturne de `DATABASE_SCHEMA.md` §4.3 s'appliqueraient à un modèle non décidé.
- **Document propriétaire** : `DOMAINE.md` (R-050) puis `DATABASE_SCHEMA.md`
- **Recommandation** :
  `DECISIONS.md` : DP-61 « Cycle `setup` → `live`, séries de numérotation et année » qui reprend n° 25, 36 et la proposition d'année d'émission. `DOMAINE.md` : R-050 reformulée (« … par organisation, par série (`trial` ou `official`) et par année d'émission ») et E-27/E-40 pour le passage d'année et la série d'essai. `ARCHITECTURE.md` §4.4 : ajouter la série (voir C2-026).
- **Alternative** : Pas de série d'essai : une seule série, la remise à zéro avant la bascule effaçant les numéros (purge des compteurs) ; plus simple, mais nécessite la purge (C2-009).
- **Question pour Mimis** : Les quittances émises pendant la double tenue (deux semaines) ont-elles une valeur pour le locataire ? L'agence continue-t-elle son carnet pendant cette période ? La bascule se fait-elle en début de mois (zone rouge) ou en milieu de mois avec un premier compte rendu partiel ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-011 — Sept documents de pilote (accord, engagement, bons usages…) ni rédigés ni planifiés

- **Gravité** : BLOQUE LE PILOTE
- **Type** : Manque
- **Nature** : DÉCISION
- **Où** :
  - `SECURITY.md` §8.6 : « Contenu **proposé** d'un document simple … signé par chaque agence pilote avant la première donnée réelle » ; §9.3 : « La réponse est écrite dans le plan **avant** le pilote ».
  - `DEPLOYMENT.md` §8.1 : prérequis « lettre d'intention signée et référent désigné ; engagement de confidentialité signé … accord écrit sur les données de test anonymisées (n° 15) et sur le droit de purge … page des bons usages » ; §8.2 : modèle de certificat de destruction.
  - `CLAUDE.md` : `contenus/` « à créer » avec `LETTRE_INTENTION.md` ; aucun des documents n'existe ; aucun ne figure dans une liste de tâches.
- **Constat** : Sept documents contractuels ou d'accueil sont des prérequis du premier pilote (`TESTING_QA.md` PP-29, `DEPLOYMENT.md` §10 « Cadre et usage ») et aucun n'est rédigé ni planifié : engagement de confidentialité, accord écrit (traitement des données, données de test anonymisées, droit de purge), lettre d'intention / convention de pilote, page des « bons usages », certificat de destruction, plan de réponse à incident, procédure d'accueil. Plusieurs exigent un juriste.
- **Impact** : Le premier pilote ne peut pas démarrer à la date du code : les agences ne signent pas, et HABEK n'a pas de base légale pour traiter des données réelles.
- **Document propriétaire** : `DEPLOYMENT.md` (accueil), `SECURITY.md` (engagement), `TESTING_QA.md` (liste « prêt »)
- **Recommandation** :
  Créer `contenus/` avec : `ENGAGEMENT_CONFIDENTIALITE.md` (reprend `SECURITY.md` §8.6, relu par un juriste), `ACCORD_PILOTE.md` (convention : données réelles, données de test anonymisées n° 15, droit de purge n° 25, durée, sortie), `BONS_USAGES.md` (une page : comptes nominatifs, téléphone perdu, lien de partage), `CERTIFICAT_DESTRUCTION.md` (modèle de `DEPLOYMENT.md` §8.2), `PLAN_INCIDENT.md` (`SECURITY.md` §9 + `DEPLOYMENT.md` §6.1). Les inscrire dans `CLAUDE.md` et dans la roadmap comme tâches de la phase « prérequis du pilote ». Remplacer `GUIDE_ENTRETIEN.md` (sans objet) par `GUIDE_ACCUEIL_AGENCE.md` tiré de `DEPLOYMENT.md` §8.1.
- **Alternative** : Réduire au minimum légal pour le premier pilote (engagement + accord écrit) et reporter les autres aux pilotes suivants.
- **Question pour Mimis** : Qui rédige ces documents (Mimis, un juriste) et sous quel délai ? Le pilote exige-t-il une lettre d'intention signée (état actuel : oui, `SECURITY.md` §5.7 et `DEPLOYMENT.md` §8.1) ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-012 — Avis juridique et CIL : cinq sujets bloquants, ni personne ni date

- **Gravité** : BLOQUE LE PILOTE
- **Type** : Manque
- **Nature** : DÉCISION
- **Où** :
  - `DECISIONS.md` n° 12 (validation juridique, « Phase 0–1 », Ouverte) ; n° 5 et H12 (CIL, hébergement hors du pays) ; n° 56 (conservation) ; n° 57 (engagement de confidentialité) : Ouvertes.
  - `TESTING_QA.md` PP-27 : « mentions revues par un juriste ou risque accepté par écrit » ; PP-30 : « position sur l'hébergement hors du pays et la CIL tranchée, ou risque accepté par écrit ».
  - `SECURITY.md` §8 : « ce document ne les reprend pas comme acquis » (loi n° 001-2021/AN, CIL).
- **Constat** : Cinq sujets juridiques conditionnent le pilote : mentions de la quittance et formulation du montant en lettres, plafond du dépôt de garantie (R-024), déclaration ou autorisation CIL et hébergement hors du pays, durées de conservation, engagement de confidentialité. Aucune date ni personne n'est désignée. Le dossier de cadrage prévoyait cet avis « Phase 0–1 » ; cette phase n'existe plus.
- **Impact** : Soit le pilote démarre avec un « risque accepté par écrit » que personne n'a rédigé, soit il attend un avis dont la date est inconnue.
- **Document propriétaire** : `DECISIONS.md` (n° 12) / `SECURITY.md` (§8)
- **Recommandation** :
  `DECISIONS.md` : DP-62 « Avis juridique avant le premier pilote » : liste des cinq sujets, personne désignée, date cible, et règle de repli (risque accepté par écrit par Mimis, modèle de `SECURITY.md` §8.6). Tous les passages juridiques gardent le marqueur `[À VÉRIFIER AVEC UN JURISTE]` (voir C2-049).
- **Alternative** : Demander un avis « rapide » limité à : mentions de quittance, dépôt de garantie et CIL/hébergement, et reporter conservation et CGU à la version commerciale.
- **Question pour Mimis** : Qui consulte la CIL ou un juriste, avec quel budget, et avant quelle date ? Acceptez-vous de démarrer le premier pilote avec un risque accepté par écrit sur l'hébergement hors du pays ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-013 — Double authentification du gérant : [APRÈS] dans l'architecture, [PILOTE] dans quatre autres documents

- **Gravité** : BLOQUE LE PILOTE
- **Type** : Contradiction
- **Nature** : DÉCISION
- **Où** :
  - `ARCHITECTURE.md` §3.8 : « [APRÈS] Double authentification du gérant (P1, tranche non définie) » ; §13.2 : « Pas de double authentification pour le gérant (seulement pour HABEK) | TOTP du gérant (P1) ».
  - `SECURITY.md` §3.4 : « TOTP … **obligatoire avant la bascule de son organisation en `live`** **[PILOTE]** » ; Questions ouvertes n° 2 ; l.757 : « `ARCHITECTURE.md` §3.8 … est à réviser selon la décision n° 50 ».
  - `TESTING_QA.md` PP-31 : « TOTP du gérant actif avant la bascule » ; `DEPLOYMENT.md` §8.1 étape 5 : « TOTP du gérant activé (obligatoire avant `live`, décision n° 50) ».
  - `SECURITY.md` §10 (liste de contrôle) : seule la console HABEK est citée (« TOTP obligatoire pour **tous** les comptes de la console HABEK »).
  - `UI_UX_DESIGN.md` S17 : « 2FA : tranche non définie par le dossier ».
- **Constat** : Quatre documents disent que le TOTP du gérant est une exigence du pilote, un dit qu'il est reporté, un le met en question ouverte, et la liste de contrôle de sécurité elle-même ne le contient pas. La décision n° 50 est ouverte.
- **Impact** : T1 contient ou non un parcours d'activation TOTP pour le gérant, ses écrans (S17), ses routes (`/me/totp`, `/auth/totp/verify`) et ses tests. La bascule `live` est bloquée ou non.
- **Document propriétaire** : `SECURITY.md` (authentification)
- **Recommandation** :
  Décider n° 50 (recommandation : TOTP obligatoire avant `live`). Puis : `ARCHITECTURE.md` §3.8 et §13.2 alignés sur SECURITY ; `SECURITY.md` §10 ajoute la case « TOTP du gérant actif avant la bascule `live` (test de refus de la bascule sans TOTP) » ; `UI_UX_DESIGN.md` S17 : « 2FA du gérant : T1, obligatoire avant `live` » ; `CONTEXTE.md` : rattacher à F01/F02 en T1.
- **Alternative** : TOTP du gérant seulement proposé au pilote, obligatoire à la version commerciale : alléger T1 mais accepter le risque T5 (vol de compte du gérant, `SECURITY.md` §2).
- **Question pour Mimis** : Le TOTP du gérant est-il obligatoire avant la bascule en `live` ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-014 — Éditeur, nom du produit et domaine non fixés

- **Gravité** : BLOQUE LE PILOTE
- **Type** : Manque
- **Nature** : DÉCISION
- **Où** :
  - `CONTEXTE.md` Questions ouvertes : « **Éditeur.** « HABEK Software » (en-tête et demande) ou « HABEK Technology Company » (champ Éditeur du dossier) ? À trancher pour les documents et les quittances. »
  - `site/NOTES.md` « À valider » n° 8 et le site : « Immo Suite, par HABEK Technology Company » ; `DECISIONS.md` n° 13 (nom et marque, « Avant publication », Ouverte) ; `DEPLOYMENT.md` §3.1 : « Le nom de domaine n'est pas choisi : la décision n° 13 … est ouverte » ; `maquette/NOTES.md` : bandeau « nom provisoire ».
- **Constat** : La raison sociale de l'éditeur, le nom du produit et le domaine ne sont pas fixés. Or l'engagement de confidentialité, l'accord de pilote, le domaine de l'essai et de la production (certificats, cookies `__Host-`, adresses des liens de quittance), les sous-domaines (`DEPLOYMENT.md` §3.1) et le pied de quittance en dépendent.
- **Impact** : Impossible de signer avec les agences ni de créer l'environnement d'essai avec un domaine définitif ; changer le domaine après l'envoi de liens de quittance casserait les liens déjà remis (jetons dans l'adresse).
- **Document propriétaire** : `DECISIONS.md` (n° 13) / `CONTEXTE.md` (Identité)
- **Recommandation** :
  `DECISIONS.md` : DP-63 « Éditeur, nom du produit et domaine » ; `CONTEXTE.md` tableau Identité : une seule valeur d'éditeur ; `DEPLOYMENT.md` §3.1 : remplacer `<domaine>` par la valeur décidée. Faire la vérification de disponibilité du nom et du domaine (recommandation du dossier §26 n° 13) avant de communiquer le domaine à une agence.
- **Alternative** : Garder « Immo Suite » comme nom de travail et utiliser un sous-domaine du domaine de l'éditeur pour le pilote (ex. `immosuite.<domaine HABEK>`).
- **Question pour Mimis** : Quel est le nom légal de l'éditeur à faire figurer dans les contrats et les documents ? Le nom « Immo Suite » et le domaine retenu sont-ils vérifiés ?
- **Décision de Mimis** : [ ] OK recommandation  [ ] OK alternative  [x] Autre : « HABEK Technology » (HABEK Technology Company) (30/10/2026)

### C2-015 — Quittance : mentions légales, montant en lettres, rectificative sans effet, format et code de contrôle

- **Gravité** : BLOQUE LE PILOTE
- **Type** : Manque
- **Nature** : DÉCISION
- **Où** :
  - `UI_UX_DESIGN.md` Quittance, pied : « mentions légales complémentaires [à valider] » ; `DOMAINE.md` E-11 : « formulation de la devise à valider » ; `DECISIONS.md` n° 12 ; `ARCHITECTURE.md` §4.2 : « orthographe traditionnelle ou rectifications de 1990 ; formulation « francs CFA » ».
  - `API_CONTRACTS.md` §7.6 : `POST /receipts/{id}/corrective` « **Réservé, non spécifié** : quittance rectificative sans effet sur le montant (nom, période). Règle absente » ; Q6, Q8 (A4 et A5 rendus à chaque paiement), `org_settings.default_paper` « A5 par défaut au pilote ».
  - `SECURITY.md` §7.5 / `DECISIONS.md` n° 55 : code de contrôle et page de vérification (Ouverte).
- **Constat** : La quittance est « le cœur du produit » (dossier §12) et six de ses choix ne sont pas tranchés : mentions légales, orthographe et libellé du montant en lettres, quittance rectificative sans effet sur le montant, format par défaut (A5 ?), rendu simultané A4 + A5, code de contrôle imprimé. `TESTING_QA.md` PP-27 exige que l'agence valide la quittance (mise en page, mentions).
- **Impact** : Le prototype PDF (a) ne peut pas comparer les moteurs sur « la vraie quittance » (`ARCHITECTURE.md` §7) ; la première quittance remise à un locataire pourrait contenir une mention contestable.
- **Document propriétaire** : `UI_UX_DESIGN.md` (contenu) / `DOMAINE.md` (R-052, R-053)
- **Recommandation** :
  Réunir les six points dans DP-64 « Quittance : contenu, format et vérification ». Propositions déjà dans les documents : orthographe traditionnelle et « francs CFA » ; A5 par défaut, A4 disponible ; rendu des deux formats à chaque paiement ; rectificative sans effet sur le montant = hors pilote (route réservée, 404) ; code de contrôle = après le pilote. Reporter le résultat dans `DOMAINE.md` (R-052, R-053, R-056) et `UI_UX_DESIGN.md` (section Quittance).
- **Alternative** : Livrer au pilote la quittance « telle que la maquette » et la faire valider par la première agence en séance de recette (`TESTING_QA.md` PP-27) ; toute évolution suit comme paramètre d'organisation.
- **Question pour Mimis** : Acceptez-vous les six propositions ci-dessus pour le pilote ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-016 — CONTEXTE.md décrit encore « maquette, puis enquête, puis développement »

- **Gravité** : IMPORTANT
- **Type** : Terme obsolète
- **Nature** : MÉCANIQUE
- **Où** :
  - `CONTEXTE.md` l.15 : « Statut | Cadrage terminé (v1.0). Phase 0 (Discovery) : maquette interactive puis enquête terrain. Aucun code ».
  - `CONTEXTE.md` l.228-234 : section « Critères Go/No-Go de la phase 0 » (renvoie à DP-34), « Échelle d'engagement » et « Livrables de la phase 0 : maquette cliquable, vidéos de démonstration, 15 entretiens documentés, grille d'engagement, prototype PDF de quittance, avis juridique succinct. Durée estimée : 4 à 6 semaines. »
  - `CONTEXTE.md` l.211-226 : colonne « Comment la valider » des hypothèses H1 à H12 : « Entretiens et observation (section 20) », « Relevé des 20 derniers paiements d'un client », etc.
  - `CONTEXTE.md` Questions ouvertes l.247 : « à remplacer par une personne réelle après les premiers entretiens » ; l.249 : « Confirmer que Mimis valide les seuils Go/No-Go et les prix ».
- **Constat** : Le propriétaire du périmètre décrit encore le processus « maquette, puis enquête, puis développement ». Les hypothèses sont validées « par entretiens » alors que le moment de validation devient le pilote.
- **Impact** : Tout agent qui lit `CONTEXTE.md` (« Au début de chaque tâche », `CLAUDE.md`) reçoit l'ancienne séquence.
- **Document propriétaire** : `CONTEXTE.md` (après DP-52)
- **Recommandation** :
  `CONTEXTE.md` : (1) Statut → « Cadrage terminé (v1.0). Première version en construction (T1 puis T2), testée par des agences pilotes avec leurs données réelles (DP-52). » ; (2) remplacer la section Go/No-Go par « Critères du pilote » qui renvoie à l'entrée DP-53 et à `TESTING_QA.md` §12, en gardant l'échelle d'engagement comme grille d'observation du pilote ; (3) supprimer « Livrables de la phase 0 » ; (4) colonne « Comment la valider » : H1, H2, H3, H4, H8, H9 → « Pilote : relevé des paiements, observation de la préparation d'un compte rendu » ; H5 → « Pilote : questions de prix et acompte » ; H6, H10, H11, H12 → inchangées (hors produit) ; (5) Questions ouvertes : « après les premiers entretiens » → « après le premier pilote ».
- **Alternative** : Aucune.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-017 — DECISIONS.md : phrases périmées, doublon PWA, n° 18 applicable mais ouverte

- **Gravité** : IMPORTANT
- **Type** : Terme obsolète
- **Nature** : MÉCANIQUE
- **Où** :
  - `DECISIONS.md` l.681 : « Priorité immédiate selon le dossier : valider les décisions 1, 3, 4 et 7, puis lancer la maquette et préparer le guide d'entretien (section 20). Restent ouvertes : 3 et 7 (moteur PDF). »
  - `DECISIONS.md` Questions ouvertes l.713 : « Application des seuils Go/No-Go (DP-34). Une même organisation compte-t-elle pour un seul entretien … Que décider si moins de 15 entretiens … ? »
  - `DECISIONS.md` Questions ouvertes l.712 : « **Application installable (PWA).** Recommandée par le dossier … non confirmée » alors que la décision n° 22 la porte.
  - `DECISIONS.md` l.669 : « La décision n° 18 … peut être fermée dans le sens « identifiants en anglais » » ; n° 18 reste Ouverte.
  - `DECISIONS.md` l.681 demande de « valider les décisions 1, 3, 4 et 7 » alors que les n° 1 et 4 sont déjà prises.
- **Constat** : Le journal des décisions contient des phrases périmées (priorité immédiate, questions sur des seuils supprimés), un doublon (PWA est à la fois une question et la décision n° 22), et une décision (n° 18) que tous les documents techniques appliquent déjà sans qu'elle soit prise.
- **Impact** : Le journal devient peu fiable : un lecteur ne sait pas ce qui est en vigueur.
- **Document propriétaire** : `DECISIONS.md`
- **Recommandation** :
  `DECISIONS.md`, après DP-52 : supprimer le paragraphe « Priorité immédiate » (l.681) ; supprimer la question « seuils Go/No-Go (DP-34) » (l.713) ; retirer la question PWA (couverte par n° 22) ; garder « Pourquoi ne pas réutiliser la stack de Scolo » et « Qui valide les décisions » ; DP-67 « Langue des identifiants » (fermer n° 18 : interface et documentation en français, identifiants de code en anglais, `CODING_RULES.md` §2) ; ajouter dans la section « Règle de mise à jour » : « Numérotation : DP-52 et suivants » → le numéro qui suit la dernière entrée créée.
- **Alternative** : Aucune.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-018 — MAQUETTE.md : la maquette sert encore à « mener l'enquête terrain »

- **Gravité** : IMPORTANT
- **Type** : Terme obsolète
- **Nature** : MÉCANIQUE
- **Où** :
  - `MAQUETTE.md` l.9-10 : « Mener l'enquête terrain de la phase 0 (15 entretiens, échelle d'engagement, seuils Go/No-Go). »
  - l.16 : « L'enquête démarre en parallèle (DP-10). » ; l.15 et l.41 : « S19 à S24 sont reportés après les premiers entretiens » ; l.62.
  - l.460 : « La clôture invite à une conversation (entretien, partage d'un échantillon de données), pas à un achat. »
  - l.462-475, section « Usage en entretien » (tableau de six hypothèses à tester) ; l.475 : « Le modèle de lettre d'intention et le guide d'entretien sont à créer (`contenus/`, voir `CLAUDE.md`) » ; l.598 : « **Seuils Go/No-Go.** Fixés par DP-34. La maquette ne les affiche pas. »
  - `maquette/README.md` l.3 : « pour les démonstrations vidéo et les entretiens de terrain ».
- **Constat** : Le brief de la maquette fait de l'enquête son usage principal. Sous la nouvelle stratégie, la maquette sert à présenter le produit à des agences pilotes potentielles ; la section « Usage en entretien » n'a plus de commanditaire.
- **Impact** : Confusion sur ce que la maquette doit prouver ; un agent pourrait préparer un guide d'entretien inutile.
- **Document propriétaire** : `MAQUETTE.md`
- **Recommandation** :
  `MAQUETTE.md` : objectif → « Montrer Immo Suite dans sa version finale à des agences et propriétaires pilotes potentiels, par des vidéos et des démonstrations. » ; supprimer l.16 ; remplacer « après les premiers entretiens » par « après le démarrage du pilote » (S19 à S24 sont de toute façon T3 ou À venir) ; la section « Usage en entretien » devient « Usage en démonstration » sans échelle d'engagement (garder uniquement la colonne « hypothèse testée » comme repère pour le pilote) ; supprimer les mentions des seuils Go/No-Go (l.598) ; `maquette/README.md` l.3 : remplacer « entretiens de terrain » par « démonstrations aux agences pilotes ».
- **Alternative** : Garder la section « Usage en entretien » comme support de démonstrations commerciales, sans la relier à DP-10 ni à DP-34.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-019 — Marqueurs « [À valider sur le terrain / en entretien] » et Go/No-Go de mise en production

- **Gravité** : IMPORTANT
- **Type** : Terme obsolète
- **Nature** : MÉCANIQUE
- **Où** :
  - `DOMAINE.md` R-071 : « [règle à confirmer sur le terrain] » ; R-085 : « [À valider sur le terrain] » ; R-096 : « [À valider en entretien] » ; section « Règles à confirmer sur le terrain » (l.391) ; l.402 : « À confirmer en entretien avant de figer R-068 » ; l.406 : « R-096 [À valider en entretien] ».
  - `UI_UX_DESIGN.md` l.426 et l.645 : « reportés après les premiers entretiens, DP-31 » ; l.905 : « À valider par les entretiens ».
  - `TESTING_QA.md` §13.1 : « les entretiens et le pilote visent des **engagements**, pas des compliments (échelle 0 à 4, dossier §20.1) ».
  - `DEPLOYMENT.md` §10 : « Revue finale par les deux développeurs et le porteur du projet (Go/No-Go écrit) » ; §8.1 : « lettre d'intention signée et référent désigné ».
  - `SECURITY.md` §5.7 : « créées par la console, après la lettre d'intention ».
- **Constat** : Les marqueurs [À valider sur le terrain] / [en entretien] n'ont plus de moment défini. Le « Go/No-Go » de `DEPLOYMENT.md` §10 est un autre objet (décision de mise en production du premier pilote) que le Go/No-Go de phase 0, mais porte le même nom. La lettre d'intention reste un prérequis de pilote (DEPLOYMENT, SECURITY) ; son modèle n'existe pas (C2-011).
- **Impact** : Ambiguïté sur la date et la personne qui valide : les règles « à confirmer » n'ont aucun propriétaire.
- **Document propriétaire** : `DOMAINE.md` (marqueurs), `DEPLOYMENT.md` (Go/No-Go)
- **Recommandation** :
  Remplacer partout « [à confirmer sur le terrain] / [À valider sur le terrain] / [À valider en entretien] » par « [à confirmer avec l'agence pilote à la découverte] » (`DOMAINE.md` R-071, R-085, R-096, l.391-406). `UI_UX_DESIGN.md` : « après les premiers entretiens » → « après le démarrage du pilote » ; « par les entretiens » → « par le pilote ». `DEPLOYMENT.md` §10 : « (Go/No-Go écrit) » → « (décision écrite de mise en production du pilote) » pour éviter la confusion avec DP-28/DP-34. `TESTING_QA.md` §13.1 : « les séances de recette visent des engagements … » (retirer « les entretiens »).
- **Alternative** : Aucune.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-020 — DP-45 à DP-51 « Prise » sans validation explicite

- **Gravité** : IMPORTANT
- **Type** : Manque
- **Nature** : DÉCISION
- **Où** :
  - `DECISIONS.md` DP-45 à DP-51 : « Statut : **Prise** » ; l.489 : « Ce sont des conséquences techniques directes de règles déjà prises : aucune autre option ne respecte ces règles. À relire par Mimis. »
  - Les mêmes documents d'origine disent : « Aucune décision technique de ce document n'est prise tant qu'une entrée DP-xx n'existe pas ».
- **Constat** : Sept décisions sont enregistrées « Prise » par l'auteur des documents sans validation explicite de Mimis (« À relire par Mimis »). Certaines contiennent des points ouverts (DP-46 : « L'année de la séquence … reste à confirmer » ; DP-49 : « Le doublon de référence externe (R-041) reste un contrôle distinct, non tranché »).
- **Impact** : Un journal de décisions où des entrées « Prise » n'ont pas été vues par le décideur n'a plus de valeur de référence.
- **Document propriétaire** : `DECISIONS.md`
- **Recommandation** :
  Mimis confirme chacune des entrées DP-45 à DP-51 (case ci-dessous) ; celles que Mimis ne confirme pas passent au statut « Proposée » et sont ajoutées au tableau des décisions ouvertes. Séparer dans DP-46 la partie prise (compteur transactionnel) de la partie ouverte (année), qui rejoint DP-61 (C2-010).
- **Alternative** : Confirmer en bloc DP-45 à DP-51 sans changement.
- **Question pour Mimis** : Confirmez-vous DP-45 à DP-51 (montants en entiers 64 bits, compteur transactionnel, écriture seule, isolation à deux barrières, idempotence, horloge unique, soldes jamais stockés) ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-021 — Trois hiérarchies de vérité différentes dans sept documents

- **Gravité** : IMPORTANT
- **Type** : Contradiction
- **Nature** : MÉCANIQUE
- **Où** :
  Trois formulations de la hiérarchie de vérité dans sept documents :
  - (1) `CLAUDE.md` l.25 et `maquette/NOTES.md` l.3 : « `DECISIONS.md` > dossier de cadrage > `DOMAINE.md` > `CONTEXTE.md` > `UI_UX_DESIGN.md` > `MAQUETTE.md` > `CLAUDE.md` » (aucun document technique).
  - (2) `TESTING_QA.md` l.18, `DEPLOYMENT.md` l.19, `API_CONTRACTS.md` l.19, `CODING_RULES.md` l.18 : « `DECISIONS.md` > dossier de cadrage > `ARCHITECTURE.md` > `DATABASE_SCHEMA.md` > `SECURITY.md` > `DOMAINE.md` > `CONTEXTE.md` > `UI_UX_DESIGN.md` » (DOMAINE après ARCHITECTURE, DATABASE_SCHEMA et SECURITY).
  - (3) `DATABASE_SCHEMA.md` l.19 : « `DECISIONS.md` > dossier de cadrage > `ARCHITECTURE.md` > `DOMAINE.md` > `CONTEXTE.md` > `UI_UX_DESIGN.md` » (ni SECURITY, ni lui-même).
  - `ARCHITECTURE.md` et `SECURITY.md` n'en donnent aucune.
- **Constat** : Les règles métier (DOMAINE) sont placées au-dessus des documents techniques dans CLAUDE.md et en dessous dans cinq autres documents ; en cas de conflit sur une règle d'argent, la réponse dépend du fichier lu. La consigne de cet audit définit un propriétaire **par sujet**, ce qui supprime le besoin d'un classement général.
- **Impact** : Un conflit entre DOMAINE et DATABASE_SCHEMA (par exemple le chevauchement avec les baux terminés, DB §9.3) reçoit deux réponses opposées selon le document consulté.
- **Document propriétaire** : `CLAUDE.md` (index unique) — contenu défini par le tableau des propriétaires par sujet
- **Recommandation** :
  `CLAUDE.md` : remplacer la ligne « Hiérarchie de vérité » par la table « Document propriétaire par sujet » de cet audit (12 lignes) et la phrase « En cas de conflit, le document propriétaire du sujet fait foi ; si deux propriétaires s'opposent, signaler le conflit, ne pas trancher ». Dans `TESTING_QA.md`, `DEPLOYMENT.md`, `DATABASE_SCHEMA.md`, `API_CONTRACTS.md`, `CODING_RULES.md` et `maquette/NOTES.md`, remplacer la ligne « Source de vérité en cas de conflit : … » par « Propriétaire par sujet : voir `CLAUDE.md`. Ce document fait foi pour : <son sujet>. »
- **Alternative** : Garder une hiérarchie générale unique dans CLAUDE.md et la reprendre à l'identique dans les autres documents : écarté, un classement général ne dit pas qui fait foi sur une règle précise.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-022 — Deux échelles de gravité (1–3 et S1–S4) sans correspondance

- **Gravité** : IMPORTANT
- **Type** : Incohérence
- **Nature** : MÉCANIQUE
- **Où** :
  - `SECURITY.md` §9.2 : gravités « **1 — critique** / **2 — élevée** / **3 — moyenne** ».
  - `TESTING_QA.md` §11.1 : « S1 — bloquante / S2 — majeure / S3 — mineure / S4 — cosmétique ».
  - `DEPLOYMENT.md` §7.3 : colonne Gravité des alertes en S1, S2, S3 (sans S4) ; §6.1 : « qualifier la gravité S1 à S3 (`SECURITY.md` §9.2, `TESTING_QA.md` §11.1) ».
  - Exemples : « Compte de gérant … compromis » est gravité **2** pour SECURITY ; « Quittance sans PDF depuis plus de 10 minutes » est **3** pour SECURITY, **S2** pour TESTING_QA (« PDF absent plus de 10 minutes ») et **S3 (S2 si répété)** pour DEPLOYMENT.
- **Constat** : Deux échelles de gravité coexistent, sans correspondance, et trois documents donnent des gravités différentes au même événement (quittance sans PDF).
- **Impact** : Le plan d'incident (qui décide d'un gel des livraisons, d'un appel à l'agence dans l'heure) applique une gravité ambiguë.
- **Document propriétaire** : `TESTING_QA.md` (gravités S1–S4, §11)
- **Recommandation** :
  Adopter l'échelle S1 à S4 de `TESTING_QA.md` §11.1 partout. `SECURITY.md` §9.2 : colonne « Gravité » → « S1 (critique) / S2 (élevée) / S3 (moyenne) », avec renvoi à `TESTING_QA.md` §11.1. `DEPLOYMENT.md` §7.3 : harmoniser « Quittance sans PDF » en **S2** (cohérent avec TESTING) ; garder S3 pour le premier échec isolé uniquement si TESTING le précise.
- **Alternative** : Garder deux échelles (sécurité d'une part, qualité de l'autre) avec une table de correspondance : plus de maintenance.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-023 — Coûts d'hébergement : le total ne se déduit pas du tableau, trois fourchettes

- **Gravité** : IMPORTANT
- **Type** : Incohérence
- **Nature** : MÉCANIQUE
- **Où** :
  - `DEPLOYMENT.md` §1.3, tableau : colonne Production : 8–20 + 15–40 + 1–3 + 1–5 + 1–2 + 0–10 = **26 à 80 €**, annoncé « **30 à 75 €** » ; colonne Essai : 5–10 + 10–15 + 1 + 0–5 = **16 à 31 €**, annoncé « 15 à 30 € » ; ligne « Ensemble » : « **50 à 120 € par mois, soit environ 33 000 à 79 000 FCFA** » alors que 15–30 + 30–75 = **45 à 105 €** (calcul par script).
  - `ARCHITECTURE.md` §11.1 : « de l'ordre de 30 à 80 € par mois au total, soit 20 000 à 52 000 FCFA » ; `DECISIONS.md` n° 86 : « 50 à 120 € par mois » ; dossier §12 et §22.3 : « quelques milliers à quelques dizaines de milliers de FCFA par mois » ; dossier §17 : « budget de lancement … quelques centaines de milliers de FCFA [à confirmer] ».
  - Conversions (1 € = 655,957 FCFA) correctes : 50 € = 32 798 FCFA, 120 € = 78 715 FCFA, 30 € = 19 679, 80 € = 52 477.
- **Constat** : Le total « 50 à 120 € » ne se déduit pas du tableau qui le justifie. Trois fourchettes coexistent (production 30–75, production 30–80, ensemble 50–120). 12 mois à 33 000–79 000 FCFA représentent 0,4 à 0,95 million de FCFA par an, soit le double ou plus d'un budget de lancement « de quelques centaines de milliers de FCFA ». Les coûts ponctuels (téléphone de test, avis juridique, test d'intrusion externe, gestionnaire de mots de passe) ne sont pas chiffrés.
- **Impact** : Un budget faux ou incomplet fausse la décision sur l'hébergement (C2-004) et la roadmap.
- **Document propriétaire** : `DEPLOYMENT.md` (§1.3, coûts) ; `ARCHITECTURE.md` renvoie
- **Recommandation** :
  `DEPLOYMENT.md` §1.3 : recalculer les totaux (Essai 16 à 31 €, Production 26 à 80 €, Ensemble 42 à 111 €, soit 27 600 à 72 800 FCFA) ou corriger les lignes ; `ARCHITECTURE.md` §11.1 : remplacer le chiffre par « voir `DEPLOYMENT.md` §1.3 » ; `DECISIONS.md` n° 86 : même renvoi. Ajouter dans `DEPLOYMENT.md` une ligne « coûts ponctuels du pilote » (appareil Android, avis juridique, test d'intrusion) avec « à chiffrer ».
- **Alternative** : Aucune.
- **Question pour Mimis** : Quel est le budget de lancement réellement disponible (le dossier dit « quelques centaines de milliers de FCFA [à confirmer] ») ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-024 — Le site annonce comme « prévus plus tard » le paiement en ligne, l'envoi automatique WhatsApp et le hors-connexion

- **Gravité** : IMPORTANT
- **Type** : Contradiction
- **Nature** : DÉCISION
- **Où** :
  - `site/immo-suite/index.html`, bloc « Où en est le produit » : « Disponible aujourd'hui : Maquette interactive … En construction : Version d'essai … **Prévu plus tard : Paiement en ligne par mobile money, envoi automatique WhatsApp et fonctionnement sans connexion** ». FAQ : « Faut-il une connexion ? Oui, aujourd'hui. Le fonctionnement sans connexion est prévu plus tard. »
  - `DECISIONS.md` DP-30 : « Le paiement en ligne (F26) et l'API WhatsApp Business (F27) ne sont pas montrés dans la maquette » ; légende « À venir : envisagé selon la demande des clients, non planifié, sans engagement de date » ; DP-32 : F26 « P2, hors MVP » ; DP-05 : hors-ligne = P3 ; `CONTEXTE.md` Exclusions.
  - `site/NOTES.md` « À valider par vous » n° 1 reconnaît le conflit : « À confirmer que vous voulez bien les annoncer sur une page publique. »
- **Constat** : La page publique annonce trois fonctions exclues comme « prévues plus tard », alors que la décision du projet est de ne les montrer nulle part et de ne promettre aucune date ; « prévu » est plus fort qu'« envisagé selon la demande ». Le point est signalé par l'auteur de la page, sans réponse.
- **Impact** : Promesse commerciale non engagée par le plan produit ; elle contredit DP-30 et la légende de la maquette.
- **Document propriétaire** : `DECISIONS.md` (DP-30)
- **Recommandation** :
  Choisir : (a) retirer la ligne « Prévu plus tard » et la phrase « Le fonctionnement sans connexion est prévu plus tard » de `site/immo-suite/index.html` (et `config.json`), garder « Oui, aujourd'hui. » ; ou (b) inscrire dans `DECISIONS.md` (DP-30) que ces trois fonctions peuvent être citées comme « À venir : envisagé selon la demande, sans date », puis remplacer « Prévu plus tard » par « À venir (envisagé selon la demande, sans date) ».
- **Alternative** : Option (b) ci-dessus.
- **Question pour Mimis** : Voulez-vous annoncer publiquement le paiement en ligne, l'envoi automatique WhatsApp et le fonctionnement sans connexion ? Si oui, sous quelle formulation ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-025 — Site, prospectus et film : appel à un « entretien de 20 minutes » et sondage hérités de l'enquête

- **Gravité** : IMPORTANT
- **Type** : Terme obsolète
- **Nature** : DÉCISION
- **Où** :
  - `site/immo-suite/index.html` : « Demander un entretien de 20 minutes » (trois emplacements), « Nous recherchons des agences pilotes. », sondage « 3 questions, 1 minute » (nombre de logements, outil de suivi, tâche la plus longue).
  - `flyer/build/a5.html`, `guide.html`, `reperes.html` : « Version de démonstration : nous recherchons des agences pilotes. Un entretien de 20 minutes ? » ; `film/VOIX_OFF.md` l.77-79 : carte de fin « agences pilotes recherchées … question ouverte » ; `film/NOTES.md` : le film montre « l'ancienne barre du haut ».
  - `site/README.md` : mentions des entretiens (3 occurrences).
- **Constat** : Le site et le prospectus invitent à un « entretien de 20 minutes » et à un sondage : outils de la phase d'enquête. Avec la nouvelle stratégie, l'appel à l'action devient le recrutement d'agences pilotes. Le texte « agences pilotes » est déjà présent ; seuls le mot « entretien » et le sondage sont à décider. Le film (v1/v2) n'a pas été régénéré après le logo (`maquette/NOTES.md` « Non fait »).
- **Impact** : Message commercial incohérent avec la stratégie (une agence qui répond au sondage s'attend à un entretien d'étude, pas à un pilote).
- **Document propriétaire** : `DECISIONS.md` (stratégie) puis marketing
- **Recommandation** :
  Après DP-52, choisir le texte de l'appel : proposition « Devenir agence pilote : un premier échange de 20 minutes ». Mettre à jour `site/immo-suite/index.html` et `config.json`, `flyer/build/*.html` (puis regénérer les PDF), `film/VOIX_OFF.md` ; garder ou retirer le sondage (il collecte une taille de portefeuille : utile au recrutement) ; régénérer le film avec le logo actuel.
- **Alternative** : Garder « entretien de 20 minutes » : l'échange initial avec une agence reste un entretien de recrutement ; seul le sondage change.
- **Question pour Mimis** : Quel appel à l'action voulez-vous sur le site, le prospectus et le film (« Devenir agence pilote » ?), et le sondage reste-t-il ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-026 — ARCHITECTURE.md n'a jamais été aligné sur les 14 écarts relevés par DATABASE_SCHEMA.md §11.2

- **Gravité** : IMPORTANT
- **Type** : Incohérence
- **Nature** : MÉCANIQUE
- **Où** :
  - `DATABASE_SCHEMA.md` §11.2 : points A1 à A14 « à signaler sur `ARCHITECTURE.md` (non modifié) » : A1 (rôle `BYPASSRLS` pour `pg_dump`), A3 (« Retirer `pgcrypto` des extensions requises »), A4 (25 tables en ajout seul, pas huit), A5 (exclusion sur `ended`), A6 (séries `trial`/`official`), A7 (`lease_notices` inutile), A9 (tables globales), A10 (bornes), A11 (`route` dans `idempotency_keys`), A12 (`immo_readonly`), A14 (pooler).
  - `ARCHITECTURE.md` (modifié le 30/10/2026 16:33, avant `DATABASE_SCHEMA.md` 19:23) : §3.3 liste `pgcrypto` ; §4.3 « huit » tables ; §4.4 compteur sans série ; §4.7 exclusion `active`/`notice` seulement ; §2.3 liste `lease_notices` et `security_deposits` (DB : `security_deposit_events`, non créée).
- **Constat** : Quatorze écarts connus entre le propriétaire de l'architecture et le propriétaire des tables n'ont jamais été reportés. Certains sont des corrections pures (`pgcrypto`, `lease_notices`, liste des tables globales) ; d'autres dépendent de décisions (n° 34, 36, 41, 46).
- **Impact** : Deux versions de la numérotation, de l'exclusion des baux et des tables en écriture seule coexistent ; un développeur suit l'une ou l'autre.
- **Document propriétaire** : `ARCHITECTURE.md` (stack, règles) / `DATABASE_SCHEMA.md` (tables, contraintes)
- **Recommandation** :
  `ARCHITECTURE.md` : appliquer immédiatement A3, A7, A9, A10, A11, A12, A14 (corrections factuelles, renvoyer à `DATABASE_SCHEMA.md` pour les listes) ; appliquer A1, A4, A5, A6 dès que n° 34, 36, 41, 46 sont pris (C2-004, C2-010, C2-005). §4.3 : remplacer la liste de huit tables par « voir `DATABASE_SCHEMA.md` §4.2 (25 tables) ». §2.3 : corriger `security_deposits` → `security_deposit_events` et retirer `lease_notices`.
- **Alternative** : Corriger `DATABASE_SCHEMA.md` pour suivre `ARCHITECTURE.md` : écarté, DB est plus précis et testé.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-027 — L'API expose la version des fiches et la confirmation du mot de passe, le schéma ne les porte pas

- **Gravité** : IMPORTANT
- **Type** : Incohérence
- **Nature** : DÉCISION
- **Où** :
  - `API_CONTRACTS.md` §5.2 : `row_version` / `version` et `If-Match` sur `properties`, `units`, `tenants`, `owners`, `mandates`, `leases`, `interventions`, `providers`, `message_templates`, `org_settings`, `role_settings` ; schémas `Lease.version`, `Tenant.version`.
  - `DATABASE_SCHEMA.md` §1.2 « Champs communs » : « `updated_at`, `updated_by` : **absents** … Un `row_version` pour l'édition concurrente viendra [APRÈS] » ; le mot `row_version` n'apparaît que dans le texte (3 occurrences), jamais comme colonne des 63 tables ; Q15 de DB.
  - `API_CONTRACTS.md` §4.6 : confirmation du mot de passe valable 5 minutes ; Q3 : « `sessions` n'a pas de colonne pour la mémoriser » ; `DATABASE_SCHEMA.md` : aucune occurrence de `reauth`.
  - `CODING_RULES.md` §2 : exemple de migration `202611021030_add_row_version.sql`.
- **Constat** : Le contrat d'API expose deux mécanismes que le schéma ne contient pas : le numéro de version des fiches et la mémorisation de la confirmation du mot de passe. Les décisions n° 64 (version dès le pilote) et la question 3 d'API sont ouvertes.
- **Impact** : Une API qui renvoie `version` et exige `If-Match` sur 11 tables sans colonne pour le stocker ; la migration initiale doit contenir ces colonnes avant le premier paiement, car une colonne ajoutée plus tard sur une table en écriture seule n'est pas remplissable.
- **Document propriétaire** : `DATABASE_SCHEMA.md` (colonnes) ; `API_CONTRACTS.md` (contrat) selon n° 64
- **Recommandation** :
  Décider n° 64 dans le lot technique (C2-005). Recommandation : accepter `row_version` dès T1 sur les 11 tables modifiables (aucune table en écriture seule n'est concernée) et ajouter `sessions.reauth_until timestamptz`. Puis `DATABASE_SCHEMA.md` §1.2 : retirer « [APRÈS] » et ajouter la colonne `row_version integer NOT NULL DEFAULT 1` aux 11 tables et `reauth_until` à `sessions`.
- **Alternative** : « Le dernier écrit l'emporte » au pilote (n° 64 option b) : retirer `version`, `If-Match` et les codes 412/428 de l'API.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-028 — UI S01 spécifie une inscription publique et un code envoyé que le pilote n'a pas

- **Gravité** : IMPORTANT
- **Type** : Incohérence
- **Nature** : DÉCISION
- **Où** :
  - `UI_UX_DESIGN.md` S01 « Connexion et inscription » : « Inscription : nom, identifiant, mot de passe, type de compte … Réinitialisation : code à usage unique valable 15 minutes » ; microcopy : « Un code à usage unique vous a été envoyé. Il est valable 15 minutes. » ; `maquette/NOTES.md` : S01 « 4 modes ».
  - `SECURITY.md` §5.7 : « L'inscription publique de F01 est désactivée au pilote » ; §3.5 : lien généré de 15 minutes ; `API_CONTRACTS.md` §4.1 : « `POST /api/auth/register` n'existe pas au pilote (404) » ; Q16 : la microcopy « suppose un envoi que le pilote n'a pas » ; `DECISIONS.md` n° 27, 51, 60 : Ouvertes.
  - Dossier F01 : « Création publique ».
- **Constat** : L'écran S01, T1, spécifie une inscription publique et un code envoyé que la sécurité et l'architecture ont retirés du pilote. L'écart est signalé par SECURITY et API, jamais reporté dans UI.
- **Impact** : Un écran à construire (inscription, code à usage unique) pour une route qui répond 404 ; une microcopy fausse affichée au pilote.
- **Document propriétaire** : `SECURITY.md` (authentification) puis `UI_UX_DESIGN.md`
- **Recommandation** :
  Après n° 27, 51, 60 : `UI_UX_DESIGN.md` S01 → « Connexion, acceptation d'invitation, demande de réinitialisation (lien fourni par le gérant). Inscription publique : après le pilote » ; microcopy « Code de réinitialisation » → « Le gérant vous a transmis un lien de réinitialisation. Il est valable 15 minutes. » ; ajouter les écrans « Accepter l'invitation » et « Choisir un nouveau mot de passe » qui n'existent pas au catalogue. Noter l'écart par rapport au dossier F01 dans `DECISIONS.md`.
- **Alternative** : Garder l'inscription publique dans l'interface mais derrière un interrupteur éteint (`ARCHITECTURE.md` §8.2).
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-029 — Deux plans ou quatre offres : aucun palier ni plafond dans le schéma

- **Gravité** : IMPORTANT
- **Type** : Incohérence
- **Nature** : DÉCISION
- **Où** :
  - `CONTEXTE.md` Questions ouvertes : « **Deux plans ou quatre offres ?** Le dossier parle de deux plans, mais sa grille de prix liste quatre offres : Propriétaire Essentiel, Propriétaire Pro, Agence, Agence Plus. Ce sont des paliers de prix ou des plans distincts ? » ; « **Plafond de lots du plan Propriétaire.** Exemple de 10–30 seulement [À valider]. »
  - Dossier §22.2 : Essentiel « Jusqu'à 10 lots, 1–2 utilisateurs » ; Agence « Jusqu'à 100 lots, 5 utilisateurs » ; `DATABASE_SCHEMA.md` `organizations.plan` : `CHECK (plan IN ('owner', 'agency'))`, aucun plafond de lots ni d'utilisateurs ; `API_CONTRACTS.md` : `plan.feature_unavailable` seulement.
  - `DECISIONS.md` DP-01 : « un plafond de lots à fixer [À valider] ».
- **Constat** : Le schéma ne porte ni palier de prix ni plafond. Le dossier mêle deux notions : le plan (périmètre fonctionnel) et le palier (taille et prix). La facturation est manuelle au début (F31 hors MVP), donc le sujet ne bloque pas le code, mais il bloque l'accueil du pilote (que promet-on à l'agence ?).
- **Impact** : Sans règle, une agence peut dépasser 100 lots ou 5 utilisateurs sans que le produit ne réagisse ; rien ne distingue Essentiel de Pro.
- **Document propriétaire** : `CONTEXTE.md` (plans) / `DECISIONS.md`
- **Recommandation** :
  `DECISIONS.md` : DP-65 « Plans et paliers » : deux plans seulement dans le produit ; les paliers sont commerciaux, sans effet dans le logiciel au pilote ; aucun plafond appliqué par le code. `CONTEXTE.md` : retirer la question « Deux plans ou quatre offres ? » et noter « paliers commerciaux ».
- **Alternative** : Ajouter `org_settings.max_units` et `max_users` appliqués par le code : refusé au pilote (règle d'or 12 : pas de règle sur mesure).
- **Question pour Mimis** : Les paliers de prix (Essentiel, Pro, Agence, Agence Plus) sont-ils uniquement commerciaux, sans effet dans le logiciel au pilote ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-030 — Matrice des permissions incomplète : collaborateur, agent terrain « proposer », entrées de menu « ? »

- **Gravité** : IMPORTANT
- **Type** : Manque
- **Nature** : DÉCISION
- **Où** :
  - `API_CONTRACTS.md` Q9 : « **Agent terrain** : « Proposer » une dépense — quel statut, qui valide ? … Route refusée en attendant. » ; Q10 : « **Collaborateur (plan Propriétaire)** : voit-il les impayés et peut-il demander une correction ? La matrice dit seulement « enregistrer les paiements et relancer ». »
  - `UI_UX_DESIGN.md` « Entrées de menu par rôle » : « ? = non défini par la matrice » pour Baux, Travaux et documents, Agenda, Notifications (caissier, agent terrain, comptable) ; la table n'a pas de colonne **Collaborateur** alors que `DATABASE_SCHEMA.md` (`memberships.role`), `API_CONTRACTS.md` §4.6 et `maquette/data` (« collaborateur ») l'utilisent.
  - `MAQUETTE.md` l.89, l.596 : agent terrain et comptable absents du sélecteur de rôle.
- **Constat** : La matrice du dossier (§11.2) est complète pour les cinq rôles d'agence mais laisse quatre trous : mécanisme de « proposer » de l'agent terrain, droits exacts du collaborateur, entrées de menu du caissier, de l'agent terrain et du comptable (Baux, Travaux, Agenda, Notifications), droits sur les actions déduites (accord support, import, paramètres : `SECURITY.md` §4.1). `maquette/NOTES.md` a masqué par prudence les entrées « ? ».
- **Impact** : Les tests « un par couple rôle × action » (TESTING A3, PP-11) ne peuvent pas être générés tant que la matrice a des cases « ? » ; le produit refuse par défaut (`SECURITY.md` §4.2) et un rôle peut se retrouver sans accès à un écran dont il a besoin.
- **Document propriétaire** : `CONTEXTE.md` (matrice, rôles)
- **Recommandation** :
  `CONTEXTE.md` : compléter la matrice (une ligne par action de `API_CONTRACTS.md` §4.6, une colonne Collaborateur) avec la proposition de `maquette/NOTES.md` (« masquées ») comme valeur par défaut ; DP-66 « Compléments à la matrice des permissions ». `UI_UX_DESIGN.md` : ajouter la colonne Collaborateur au tableau « Entrées de menu par rôle » et remplacer chaque « ? » par la valeur décidée.
- **Alternative** : Garder le refus par défaut pour tout ce qui n'est pas dans la matrice (état actuel de SECURITY §4.2) et ne rien décider avant le pilote.
- **Question pour Mimis** : Le collaborateur du plan Propriétaire voit-il les impayés et peut-il demander une correction ? L'agent terrain peut-il proposer une dépense, et qui la valide ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-031 — Fonctions de T3 sans route (F17, F22), sans table créée ni test

- **Gravité** : IMPORTANT
- **Type** : Manque
- **Nature** : MÉCANIQUE
- **Où** :
  - Analyse par script de la matrice F01–F40 (annexe 1) : **F17** (agenda, S15, T3) n'a aucune route et aucune table créée (`tasks` « prévue, non créée ») ; **F22** (notifications, S20, T3) n'a aucune route propre (seulement `GET /api/home`) ni table créée (`notification_reads`) ; `API_CONTRACTS.md` l.27 liste pourtant « agenda, notifications » parmi les points d'accès [APRÈS] de T3.
  - Aucun cas E-xx rattaché à **F18, F20, F22, F25** (seul F19 a E-34 ; F23 n'est couvert que par E-35, renouvellement) : `TESTING_QA.md` §2.1 « Correspondance complète des cas » ; R-066, R-067 et R-090 n'ont pas de test cité ; F25 (lecture du SMS) n'a ni règle R-xxx ni test.
  - `DATABASE_SCHEMA.md` §3 : `security_deposit_events`, `tasks`, `notification_reads`, `subscriptions` « prévues, non créées ».
- **Constat** : Les fonctions de T3 sont décrites au niveau écran mais pas au niveau route, table et test. C'est acceptable tant qu'elles sont livrées pendant le pilote, mais chaque tranche T3 devra reprendre les trois spécifications.
- **Impact** : Une fonction de T3 démarre sans contrat d'API ni cas de test : les tests d'or se rédigent après le code, contrairement à `CODING_RULES.md` §9.2.
- **Document propriétaire** : `API_CONTRACTS.md` (routes), `DATABASE_SCHEMA.md` (tables), `TESTING_QA.md` (cas)
- **Recommandation** :
  Ajouter un paragraphe « T3 : à spécifier avant chaque lot » dans `API_CONTRACTS.md` §7.19 (F17 : `GET/POST /api/tasks`, F22 : `GET /api/notifications`, `POST …/read`) avec l'étiquette « à spécifier » ; ajouter dans `TESTING_QA.md` §2.1 les lignes F18 (neutralisation des formules, PP-19), F20 (R-066, R-067), F23 (R-090), F25 (lecture du SMS, fichier de cas réels), F22, F17 avec « cas à écrire avec la règle ».
- **Alternative** : Attendre le début de la tranche concernée.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-032 — 38 règles sur 97 sans test cité par identifiant

- **Gravité** : IMPORTANT
- **Type** : Manque
- **Nature** : MÉCANIQUE
- **Où** :
  - Analyse par script (annexe 2) : sur 97 règles R-xxx, **38** n'ont aucun test cité dans `TESTING_QA.md` ni dans les 44 tests de `DATABASE_SCHEMA.md` §9 (R-002, R-003, R-004, R-005, R-007, R-008, R-009, R-010, R-011, R-012, R-013, R-015, R-016, R-019, R-022, R-025, R-026, R-030, R-031, R-034, R-036, R-037, R-040, R-052, R-055, R-056, R-061, R-062, R-066, R-067, R-071, R-078, R-079, R-082, R-085, R-090, R-092, R-094) ; 12 ne sont dans aucune route ni code d'erreur de l'API (R-008, R-025, R-026, R-030, R-031, R-032, R-034, R-045, R-050, R-052, R-094, R-095) ; 40 ne figurent ni dans `UI_UX_DESIGN.md` ni dans `MAQUETTE.md`.
  - Beaucoup sont couvertes **par famille** (par exemple R-004 par les tests ISO et A4 ; R-007 par G-05 et ISO-4 ; R-022 par une contrainte `CHECK` de `leases`), mais pas **par identifiant**.
  - `DOMAINE.md` l.179 : « Une règle par ligne, testable. »
- **Constat** : `TESTING_QA.md` trace ses tests par cas E-xx et par famille, pas par règle. Certaines règles n'ont pas de mécanisme créé : R-079 (clôture de caisse : `cash_days` créée vide), R-066 et R-067 (table `security_deposit_events` non créée), R-090 (préavis : colonnes de `leases`, route [APRÈS]).
- **Impact** : On ne peut pas démontrer que chaque règle de `DOMAINE.md` est testée ; une règle peut être oubliée sans que la liste « prêt pour un pilote » (PP-05) ne le montre.
- **Document propriétaire** : `TESTING_QA.md` (couverture des règles)
- **Recommandation** :
  `TESTING_QA.md` : ajouter en §2 un tableau « Règle → test » (97 lignes, généré depuis `TRACABILITE.md`) avec trois états : « test cité », « couverte par famille X », « à écrire (tranche T3 ou règle ouverte) ». Phase B : générer cette matrice dans `TRACABILITE.md` (annexe 2 de cet audit en est la base).
- **Alternative** : Se contenter des cas E-xx et des familles : ne répond pas à la règle « une règle, un test » de `CODING_RULES.md` §9.2.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-033 — Console HABEK (F40, T2) sans écran ni spécification

- **Gravité** : IMPORTANT
- **Type** : Manque
- **Nature** : DÉCISION
- **Où** :
  - `UI_UX_DESIGN.md` : « F40 Console HABEK » absente du catalogue d'écrans (`UI_UX_DESIGN.md` : « Hors catalogue : la console HABEK (F40) est un outil interne … absent de la maquette ») ; `API_CONTRACTS.md` §7.18 : « Le détail relèvera de `specs/` » (n'existe pas) ; routes `GET/POST /console/organizations`, `/go-live`, `/suspend`, `/features/{feature}`, `/console/users/{id}/password-resets`.
  - `SECURITY.md` §5.7 : inscription fermée, « les organisations sont créées par la console » ; `DEPLOYMENT.md` §8.1 étape 1 : « par la **console** » ; `ARCHITECTURE.md` §9.1 : « Connexion d'un utilisateur créé par commande ».
- **Constat** : La console est l'unique voie de création d'une organisation et de bascule en `live`, et le seul endroit où activer les interrupteurs, générer les liens de réinitialisation d'un gérant et consulter les signalements. Elle est T2 (F40, P0) mais n'a ni écrans, ni spécification, ni test (annexe 1 : F40 « sans test » direct).
- **Impact** : Le premier pilote ne peut être accueilli (création, bascule) sans console ; l'effort de F40 n'est pas estimé.
- **Document propriétaire** : `API_CONTRACTS.md` (routes) + `specs/F40.md`
- **Recommandation** :
  Créer `specs/F40-console.md` avant la tranche 6 : écrans (liste des organisations, création, bascule `setup`/`live`, accès support, interrupteurs, signalements, tableau de bord interne), routes détaillées, tests. En attendant, la console peut être une **ligne de commande** (`immo org create`, `immo org go-live`) comme `ARCHITECTURE.md` §9.2 le propose pour l'import.
- **Alternative** : Faire de la ligne de commande la console du pilote et reporter l'interface web de la console à la version commerciale.
- **Question pour Mimis** : La console du pilote est-elle une interface web ou une ligne de commande ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-034 — Prototypes techniques (10 jours) hors planification ; durée du prototype PDF incohérente

- **Gravité** : IMPORTANT
- **Type** : Incohérence
- **Nature** : MÉCANIQUE
- **Où** :
  - `ARCHITECTURE.md` §7 : prototype (a) PDF « 3 jours (dossier §17 : 2 à 3 jours) ».
  - `DECISIONS.md` n° 7 : « Prototype d'une journée avant de décider » ; DP-11 : « Le dossier inclut un prototype PDF d'une journée en phase 0 » ; dossier §17 Phase 0 : « 2–3 jours pour le prototype PDF » ; §26 n° 7 : « Prototype d'une journée ».
  - `ARCHITECTURE.md` §7 : cinq prototypes (a) à (e) pour 10 jours-personne, ordre « (b) et (c) d'abord, ensemble » ; `ARCHITECTURE.md` §9.1 : tranche verticale « 2 semaines [Hypothèse] ».
- **Constat** : La durée du prototype PDF vaut 1 jour, 2–3 jours ou 3 jours selon le document. Les cinq prototypes sont des prérequis techniques de la tranche verticale (`DATABASE_SCHEMA.md` §9.1 : « La concurrence … n'a pas été jouée », « `pg_dump` … n'a pas pu être testé ») mais ne sont dans aucune planification.
- **Impact** : La roadmap sous-estime ou oublie les 10 jours de prototypes qui valident le schéma et la numérotation.
- **Document propriétaire** : `ARCHITECTURE.md` §7
- **Recommandation** :
  `DECISIONS.md` n° 7 et DP-11 : « prototype de 2 à 3 jours (`ARCHITECTURE.md` §7) » ; `ROADMAP.md` : première phase = prototypes (b) puis (c) puis (a) puis (e), (d) avant l'import, 10 jours-personne au total, avant la migration initiale.
- **Alternative** : Aucune.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-035 — Le critère « prêt pour un pilote » existe en quatre listes différentes

- **Gravité** : IMPORTANT
- **Type** : Doublon
- **Nature** : MÉCANIQUE
- **Où** :
  Le critère « prêt pour un pilote » existe en **quatre listes** : `ARCHITECTURE.md` §9.3 (24 cases), `SECURITY.md` §10 (37 cases), `TESTING_QA.md` §12 (PP-01 à PP-32, qui dit « reprend `ARCHITECTURE.md` §9.3 et `SECURITY.md` §10, sans les modifier »), `DEPLOYMENT.md` §10 (27 cases, « regroupe » les trois).
  Écarts mesurés : auto-test d'intrusion (TESTING PP-24, DEPLOYMENT §10, SECURITY §11) absent d'ARCHITECTURE §9.3 ; TOTP du gérant (TESTING PP-31, DEPLOYMENT) absent de SECURITY §10 et déclaré [APRÈS] par ARCHITECTURE (C2-013) ; gouvernance des accès, page d'état, heures de couverture, organisation de démonstration : DEPLOYMENT seulement ; engagement de confidentialité et accord écrit : TESTING PP-29, SECURITY §10, DEPLOYMENT, absent d'ARCHITECTURE ; `CLAUDE.md` : aucune mention.
- **Constat** : Quatre listes de 24 à 37 cases décrivent la même porte avec des contenus différents ; chacune prétend reprendre les autres. Le propriétaire désigné par la consigne est `TESTING_QA.md`.
- **Impact** : Une case ajoutée dans une liste mais pas dans les autres : le pilote démarre sur la foi d'une liste incomplète.
- **Document propriétaire** : `TESTING_QA.md` §12
- **Recommandation** :
  `TESTING_QA.md` §12 : devient la **seule** liste. Y ajouter les cases manquantes : gouvernance des accès (`DEPLOYMENT.md` §3.3), page d'état, heures de couverture et personne de repli, organisation de démonstration (mélange refusé), engagement interne de l'équipe (`SECURITY.md` §10), TOTP du gérant (selon C2-013). `ARCHITECTURE.md` §9.3, `SECURITY.md` §10 et `DEPLOYMENT.md` §10 : remplacer la liste par « voir `TESTING_QA.md` §12 » et ne garder que ce qui est spécifique au document (le *comment vérifier* de SECURITY §10 devient des renvois aux tests). `CLAUDE.md` : ajouter une ligne « Critère « prêt pour un pilote » : `TESTING_QA.md` §12 ».
- **Alternative** : Garder DEPLOYMENT §10 comme liste de **mise en production** (opérationnelle) distincte de la liste de qualité de TESTING_QA §12, avec renvoi explicite.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-036 — CLAUDE.md : statut d'ARCHITECTURE, vérificateur de la maquette, variables d'environnement

- **Gravité** : IMPORTANT
- **Type** : Terme obsolète
- **Nature** : MÉCANIQUE
- **Où** :
  - `CLAUDE.md` l.40 : `ARCHITECTURE.md` « (à créer) » (alors que l.20 : « Créé, v0.1 »).
  - l.73 : « jeu de données vérifié par `donnees/verifier.html` » ; `MAQUETTE.md` l.131-132 : `donnees/verifier.js` · `verifier.html` ; fichier réel : `maquette/scripts/verifier-donnees.js` (`maquette/NOTES.md` : « Node 24 est maintenant présent … `node scripts/verifier-donnees.js` »).
  - l.9 : « Frontend non décidé (décision ouverte n° 6) » et l.89 : « front React + Vite + TypeScript … » ; l.94 : variables `DATABASE_URL`, `MIGRATION_DATABASE_URL`, `S3_*`, `SESSION_SECRET`, `BACKUP_ENCRYPTION_KEY` (hors hébergeur) alors que `DEPLOYMENT.md` §2.4 ajoute `APP_ENV`, `PUBLIC_BASE_URL`, la clé de chiffrement des champs, et signale « Écart signalé : `CLAUDE.md` liste `BACKUP_ENCRYPTION_KEY` … » (n° 95 : clé publique) ; l.93 : structure sans `tests/contract/`, `spikes/`, `rust-toolchain.toml` (`CODING_RULES.md` §1.1).
  - l.6 : « Éditeur : HABEK (nom exact à trancher, voir CONTEXTE.md) » ; l.25 : hiérarchie (C2-021).
  - `CLAUDE.md` : 96 lignes (< 100).
- **Constat** : `CLAUDE.md` est à jour sur les statuts (« Créé » pour six documents et les documents techniques) mais en retard sur trois faits (ARCHITECTURE créé, vérificateur de la maquette, variables d'environnement) et contient des affirmations contradictoires sur le front.
- **Impact** : Un agent exécute une commande qui n'existe pas (`verifier.html`) ou configure un secret de sauvegarde différent de celui que DEPLOYMENT recommande.
- **Document propriétaire** : `CLAUDE.md`
- **Recommandation** :
  `CLAUDE.md` : l.40 → « `ARCHITECTURE.md` | Avant d'écrire du code applicatif | … » (retirer « (à créer) ») ; l.73 → « `node maquette/scripts/verifier-donnees.js` (28 contrôles) » ; l.9 → « Frontend : React + Vite + TypeScript proposé (`ARCHITECTURE.md` §3.5, n° 6 ouverte) » ; l.94 → « Variables : `APP_ENV`, `PUBLIC_BASE_URL`, `DATABASE_URL`, `MIGRATION_DATABASE_URL`, `S3_*`, `SESSION_SECRET`, clé de chiffrement des champs (copie hors ligne). Clé de sauvegarde : voir `DEPLOYMENT.md` §5.2 (n° 95) » ; l.93 : ajouter `tests/contract/`, `spikes/`. Rester sous 100 lignes en supprimant les phrases qui répètent `CODING_RULES.md` §9.
- **Alternative** : Aucune.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-037 — Dix familles de contenus recopiés dans plusieurs documents

- **Gravité** : IMPORTANT
- **Type** : Doublon
- **Nature** : MÉCANIQUE
- **Où** :
  Contenus répétés qui divergeront (la valeur propriétaire est entre parenthèses) :
  - Légende `[PILOTE] / [APRÈS] / [À valider] / [Hypothèse]` : 7 documents (propriétaire : aucun ; à définir dans `CLAUDE.md`).
  - Matrice de permissions : `CONTEXTE.md` (propriétaire), dossier §11.2, `SECURITY.md` §4.1 (identique, vérifié par script : 0 écart), `API_CONTRACTS.md` §4.6, `UI_UX_DESIGN.md` « Entrées de menu ».
  - Budgets de performance 200 / 150 / 30 Ko, LCP 3 s : `UI_UX_DESIGN.md` (propriétaire), `ARCHITECTURE.md` §12.1, `CODING_RULES.md` §3.2 et §5.4, `TESTING_QA.md` §8.3.
  - Objectifs de sauvegarde 15 min / 4 h / 30 j / 12 copies : `ARCHITECTURE.md` §10.4 (propriétaire), `SECURITY.md` §5.3, `DEPLOYMENT.md` §5, `TESTING_QA.md` PP-14. `DEPLOYMENT.md` §5.1 ajoute « 30 copies quotidiennes » absentes des autres.
  - Alertes : `ARCHITECTURE.md` §10.3 (10 lignes), `DEPLOYMENT.md` §7.3 (15 lignes, propriétaire).
  - Limitation du débit : `SECURITY.md` §3.3 (propriétaire), `API_CONTRACTS.md` §4.7 (+ 300 requêtes/minute).
  - Jeu de données de démonstration (40 lots, 31 locataires, 33 baux, HOR-2026-000094) : `MAQUETTE.md` (propriétaire), `ARCHITECTURE.md` §8.5, `TESTING_QA.md` §9, `DEPLOYMENT.md` §9.1.
  - Textes d'erreur : `API_CONTRACTS.md` §3.4 (✱ = absents d'UI) et `UI_UX_DESIGN.md` Microcopy.
  - Durées de conservation : `SECURITY.md` §6.3 (propriétaire), `ARCHITECTURE.md` §8.4 (13 mois), `DATABASE_SCHEMA.md`.
  - Liste des tables : `ARCHITECTURE.md` §2.3 et `DATABASE_SCHEMA.md` §3 (propriétaire).
- **Constat** : Dix familles de valeurs sont recopiées dans deux à quatre documents. Aujourd'hui elles concordent, sauf les écarts déjà relevés (coûts, calendrier, gravités, TOTP, tables).
- **Impact** : Chaque changement futur exige quatre modifications ; l'oubli d'une seule crée la prochaine contradiction.
- **Document propriétaire** : Voir « Document propriétaire » de chaque ligne
- **Recommandation** :
  Pour chaque famille, le document non propriétaire remplace la valeur par « voir <propriétaire> §x » (sauf budgets et RPO/RTO qui sont des chiffres d'objectif à garder en une phrase avec renvoi). Phase B : appliquer pour les alertes (ARCHITECTURE §10.3 → DEPLOYMENT §7.3), le calendrier (C2-044), les coûts (C2-023), la hiérarchie (C2-021), les listes « prêt pour un pilote » (C2-035). Ajouter dans `CLAUDE.md` une ligne « Légende des marqueurs » et la retirer des sept documents.
- **Alternative** : Laisser les doublons et ajouter « à tenir en phase avec » dans chaque fichier : plus lourd à maintenir.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-038 — DATABASE_SCHEMA.md : « PostgreSQL réel » est PGlite, scripts et tests non livrés

- **Gravité** : IMPORTANT
- **Type** : Ambiguïté
- **Nature** : MÉCANIQUE
- **Où** :
  - `DATABASE_SCHEMA.md` l.5 : « Le schéma a été exécuté et testé sur un **PostgreSQL réel** de test (section 9) » ; §9.1 : « PostgreSQL 18 réel (**PGlite, PostgreSQL compilé en WebAssembly**, en mémoire) », « 44 tests, 44 réussis. Les scripts de test **ne sont pas livrés dans ce dossier** (consigne : aucun code hors extraits SQL) » ; chiffres « 63 tables, 73 déclencheurs, 115 politiques, 5 vues » (vérifiés partiellement par script : 63 tables, 5 vues ; 73 et 115 non vérifiables sans le schéma complet).
  - `TESTING_QA.md` §3 : « 44 tests déjà rédigés pour le schéma, section 9 » ; Q7 : « jusqu'ici exécutés sur PGlite (WebAssembly) ».
  - `TESTING_QA.md` §1.3 : « Pas de simulation de la base pour les tests d'intégration : PostgreSQL réel ».
- **Constat** : L'en-tête présente PGlite comme « un PostgreSQL réel », ce que `TESTING_QA.md` §1.3 et Q7 distinguent explicitement. Les scripts SQL complets du schéma (les 63 tables, les déclencheurs) et des 44 tests ne sont pas dans le dossier : le schéma de §3 est « généré à partir du catalogue de la base de test » mais la source exécutable est absente.
- **Impact** : Les 73 déclencheurs, 115 politiques et 44 tests sont des affirmations invérifiables ; la migration initiale doit être réécrite à partir de la documentation.
- **Document propriétaire** : `DATABASE_SCHEMA.md`
- **Recommandation** :
  `DATABASE_SCHEMA.md` l.5 : « exécuté et testé sur PostgreSQL 18 en mémoire (PGlite, WebAssembly), pas encore sur le PostgreSQL de production (prototype b) ». Ajouter en §9.1 : « Le script complet et les 44 tests seront recréés dans `migrations/` et `tests/` à la tranche verticale ; ce document est leur spécification ». Retirer « consigne » (C2-048).
- **Alternative** : Livrer les scripts dans un dossier `spikes/schema/` : à décider avec la place des fichiers jetables (n° 23).
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-039 — Écarts assumés par rapport au dossier dispersés

- **Gravité** : IMPORTANT
- **Type** : Manque
- **Nature** : MÉCANIQUE
- **Où** :
  Écarts assumés par rapport au dossier de cadrage (non modifiable) dispersés dans des documents sans liste :
  - DP-12 (tranches 31–59 et non 31–60), DP-29 (F19, F23, F25 en T3), DP-36 (alerte F12 dans l'accueil, non dans l'agenda), DP-41 (indexation sur le bail), DP-42 (rôle comptable avec compte) — enregistrés ;
  - `SECURITY.md` §5.7 / n° 51 (inscription publique fermée, « écart avec le texte de F01 »), n° 27/60 (réinitialisation par lien et non par code envoyé, F01), n° 50 (TOTP du gérant avant `live`, dossier §12 « option, P1 »), n° 54 (accès support : 24 h par défaut, 7 jours au plus, dossier : « limité dans le temps »), n° 64 (version des fiches dès le pilote), n° 24 (RPO 15 min, dossier : « sauvegardes quotidiennes ») ;
  - DP-52 (phase 0 supprimée, dossier §17 et §26 n° 1), capacité deux développeurs (dossier §17).
- **Constat** : Aucun fichier ne regroupe les écarts avec le dossier. Une partie est dans des décisions DP-xx, une partie dans des documents techniques marqués « Ouverte ».
- **Impact** : On ne sait plus où le produit s'écarte volontairement du dossier : risque de « corriger » un écart voulu, ou d'oublier de documenter un écart nouveau.
- **Document propriétaire** : `DECISIONS.md`
- **Recommandation** :
  `DECISIONS.md` : ajouter une section « Écarts assumés par rapport au dossier de cadrage » (tableau : écart, entrée DP-xx, section du dossier), alimentée par les 12 lignes ci-dessus. Règle de mise à jour : tout écart nouveau y est ajouté avec l'entrée DP-xx qui le décide.
- **Alternative** : Aucune.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-040 — Variables des modèles de relance : quatre, six ou non définies

- **Gravité** : IMPORTANT
- **Type** : Incohérence
- **Nature** : DÉCISION
- **Où** :
  - `UI_UX_DESIGN.md` Microcopy « Variable inconnue » : « Variables disponibles : {nom}, {montant}, {mois}, {lien_quittance} » ; section « Modèles de relance » : « Variables : `{nom}`, `{montant}`, `{mois}`, `{lien_quittance}`. Noms des variables : proposition à confirmer ».
  - `maquette/NOTES.md` : « Les modèles utilisent `{nom}`, `{montant}`, `{mois}`, `{lot}`, `{date}`, `{lien_quittance}` … `{lot}` et `{date}` sont nécessaires au texte de référence de `MAQUETTE.md` » ; `MAQUETTE.md` message courtois : « pour la {Maison … Gounghin}, échu le {10/11/2026} » (6 variables).
  - `ARCHITECTURE.md` Q16, `DATABASE_SCHEMA.md` §10.2 n° 6, `API_CONTRACTS.md` Q4 : « La liste autorisée par R-064 est à fixer ».
  - `DOMAINE.md` R-064 : « refuse les variables inconnues » sans en donner la liste.
- **Constat** : La liste des variables autorisées (R-064) est définie quatre fois ou pas du tout. Le texte de référence de la maquette en utilise six.
- **Impact** : `GET /api/message-variables`, la validation des modèles et le message d'erreur de la microcopy ne peuvent pas être implémentés de façon cohérente.
- **Document propriétaire** : `DOMAINE.md` (glossaire « Modèle de message ») 
- **Recommandation** :
  `DOMAINE.md` : glossaire « Modèle de message » : « Variables autorisées : `{nom}`, `{montant}`, `{mois}`, `{lot}`, `{date}`, `{lien_quittance}` (R-064) » ; `UI_UX_DESIGN.md` : mettre la même liste dans la microcopy et dans « Modèles de relance » ; `API_CONTRACTS.md` Q4 close.
- **Alternative** : Réduire la maquette à quatre variables : exige de réécrire le texte de référence de la relance.
- **Question pour Mimis** : Les six variables (nom, montant, mois, lot, date, lien_quittance) sont-elles acceptées ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-041 — Conservation 10 ans ou purge à 30 jours après le départ d'une agence

- **Gravité** : IMPORTANT
- **Type** : Contradiction
- **Nature** : DÉCISION
- **Où** :
  - `SECURITY.md` §6.3 : « `audit_log`, écritures financières, quittances, comptes rendus : Durée du contrat, **plus 10 ans** après la sortie » ; §8.4 : « Après l'export et la fin du contrat : délai de grâce de 30 jours, puis **purge** de l'organisation (suppression des données de production) ».
  - `DATABASE_SCHEMA.md` §4.2 / §8.3 : `purge_org_setup` « jamais à une organisation `live` » ; aucune fonction de purge pour une organisation `closed`.
  - `DEPLOYMENT.md` §8.2 et `DECISIONS.md` n° 94 : « Incohérence entre `SECURITY.md` §6.3 et §8.4, et aucune fonction de purge pour `live` ».
- **Constat** : Le départ d'une agence est décrit de deux façons incompatibles (conservation 10 ans ou purge à 30 jours) et le mécanisme technique de la seconde n'existe pas. Le sujet touche la loi sur les données personnelles et l'obligation de conservation, donc un juriste.
- **Impact** : Au premier départ, ni l'engagement de confidentialité (§8.6 : « destruction ensuite ») ni la base ne permettent d'agir.
- **Document propriétaire** : `SECURITY.md` (conservation) ; `DATABASE_SCHEMA.md` (fonction de purge)
- **Recommandation** :
  `SECURITY.md` : une seule règle écrite après l'avis juridique (DP-62) ; en attendant, §8.4 est la règle de l'engagement (option (a) de `DEPLOYMENT.md` §8.2) et §6.3 précise « pendant la durée du contrat et jusqu'à la purge sur instruction écrite ». `DATABASE_SCHEMA.md` : fonction `purge_org_closed` avant le premier départ (pas avant le premier pilote).
- **Alternative** : Option (b) de `DEPLOYMENT.md` §8.2 : conservation 10 ans après la sortie, accès fermé, sans purge.
- **Question pour Mimis** : Après le départ d'une agence, HABEK conserve-t-il ses données (10 ans proposés) ou les détruit-il sur instruction écrite après 30 jours ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-042 — SECURITY.md : la purge de setup n'est pas la seule suppression

- **Gravité** : IMPORTANT
- **Type** : Contradiction
- **Nature** : MÉCANIQUE
- **Où** :
  - `SECURITY.md` §6.2 : « La purge d'une organisation en phase `setup` (DB §8.3) est la **seule** suppression possible ».
  - `DATABASE_SCHEMA.md` §4.2 : `cancel_import_batch` supprime aussi baux, locataires, lots, biens et mandants ; `DOMAINE.md` R-080 : « L'import est annulable tant que rien n'a été utilisé ».
- **Constat** : Deux suppressions autorisées (annulation d'import et purge de `setup`) alors que SECURITY affirme qu'il n'y en a qu'une.
- **Impact** : Une affirmation fausse dans le document de sécurité, qui sert de base à la revue.
- **Document propriétaire** : `SECURITY.md`
- **Recommandation** :
  `SECURITY.md` §6.2 : « La purge d'une organisation en phase `setup` et l'annulation d'un import non utilisé (R-080) sont les **seules** suppressions possibles ; elles sont tracées dans `platform_events` ».
- **Alternative** : Aucune.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-043 — Heures de couverture et personne de repli non fixées

- **Gravité** : IMPORTANT
- **Type** : Manque
- **Nature** : DÉCISION
- **Où** :
  - `DECISIONS.md` n° 84 (délais de correction S1 24 h, S2 72 h, « Heures de couverture à fixer »), n° 89 (canal d'alerte) : Ouvertes.
  - `DEPLOYMENT.md` Questions ouvertes n° 5 : « Heures de couverture et repli … Pas d'astreinte 24 h sur 24 avec deux personnes » ; `TESTING_QA.md` §11.2 : « L'équipe n'a pas d'astreinte permanente (question ouverte) » ; `DEPLOYMENT.md` §7.3 : « Heures de couverture : à fixer ».
- **Constat** : Les engagements de service du pilote (prise en compte S1 en 1 h, correctif S1 en 24 h, information de l'agence « dans l'heure ») n'ont ni heures de couverture ni personne de repli. Ils figurent dans l'engagement de confidentialité (`SECURITY.md` §8.6) comme délais proposés.
- **Impact** : Une promesse d'intervention « dans l'heure » signée par l'agence sans que personne ne soit joignable le samedi.
- **Document propriétaire** : `TESTING_QA.md` (§11.2 délais) / `DEPLOYMENT.md` (§6)
- **Recommandation** :
  `DECISIONS.md` : une entrée « Couverture du pilote » : jours et heures, personne de repli, numéro d'urgence ; puis aligner `TESTING_QA.md` §11.2, `DEPLOYMENT.md` §6.1 et §7.3, `SECURITY.md` §8.6.
- **Alternative** : Ne promettre aucun délai dans l'engagement (« au mieux ») : réduit la confiance de l'agence.
- **Question pour Mimis** : Quels jours et heures les développeurs sont-ils joignables pendant le pilote (samedi ? soirs ?), et qui est la personne de repli ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-044 — Calendrier de livraison formulé de trois façons

- **Gravité** : MINEUR
- **Type** : Incohérence
- **Nature** : MÉCANIQUE
- **Où** :
  - `ARCHITECTURE.md` §10.5 : « Fréquence visée pendant le pilote : plusieurs livraisons par semaine, jamais le premier ni le cinq du mois … ni le vendredi soir ».
  - `TESTING_QA.md` §11.4 : « Calendrier respecté : pas le 1ᵉʳ au 7 du mois, pas de vendredi après-midi, pas la veille d'un jour férié ni d'une bascule d'agence ».
  - `DEPLOYMENT.md` §4 : zone rouge 1 à 7, orange 8–10 et 25–31, verte 11–24 ; « mardi à jeudi » ; « de 06 h 00 à 07 h 30 » ; `DECISIONS.md` n° 91 : même règle.
- **Constat** : Trois formulations : le propriétaire (DEPLOYMENT, qui reprend la citation d'ARCHITECTURE) est le plus détaillé ; les deux autres sont des versions courtes qui divergent (« le premier ni le cinq » contre « 1 à 7 » ; « vendredi soir » contre « vendredi après-midi »).
- **Impact** : Une livraison faite « le 6 » est autorisée par ARCHITECTURE et interdite par DEPLOYMENT.
- **Document propriétaire** : `DEPLOYMENT.md` (§4)
- **Recommandation** :
  `ARCHITECTURE.md` §10.5 et `TESTING_QA.md` §11.4 : remplacer la règle par « Calendrier de livraison : voir `DEPLOYMENT.md` §4 » ; `DEPLOYMENT.md` §4 : retirer la citation d'ARCHITECTURE (« jamais le premier ni le cinq ») de la colonne « Pourquoi ».
- **Alternative** : Aucune.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-045 — UI_UX_DESIGN.md en retard sur SECURITY, DATABASE_SCHEMA et TESTING_QA

- **Gravité** : MINEUR
- **Type** : Incohérence
- **Nature** : MÉCANIQUE
- **Où** :
  - `UI_UX_DESIGN.md` « Quittance » : variantes « rectificative » et « annulée » seulement (filigrane « ANNULÉE — remplacée par la quittance n° … ») ; S08 : « pièce d'identité facultative » ; S17 : « sécurité (2FA du gérant, P1) ».
  - `SECURITY.md` §7.5 : filigranes « DÉMONSTRATION — sans valeur » (organisation `demo`), « ESSAI » (série `trial`), « ANNULÉE » ; `ARCHITECTURE.md` §13.2, `SECURITY.md` §8.1, `DATABASE_SCHEMA.md` §1.2 : pièce d'identité « Champ désactivé au pilote » ; `TESTING_QA.md` §6 : 20 cas types dont « série d'essai (filigrane « ESSAI »), organisation de démonstration (filigrane « DÉMONSTRATION — sans valeur »).
  - `UI_UX_DESIGN.md` contraste « Repère « À venir » : `--color-brand-700` sur `--color-accent-100` (14,5:1) » alors que `MAQUETTE.md` l.49 : texte `--status-due-fg` sur `--color-accent-100` (7,3:1 recalculé).
- **Constat** : Trois éléments de `SECURITY` / `DB` / `TESTING` ne sont pas dans le document de design : deux filigranes de quittance, le champ pièce d'identité désactivé au pilote, la couleur du repère « À venir ». Les contrastes de la table UI sont tous exacts à ±0,1 près (recalcul par script, voir la liste des vérifications), sauf cette divergence de paire.
- **Impact** : Le PDF de la quittance d'essai est produit sans spécification de mise en page de son filigrane ; le formulaire locataire propose un champ qui n'existe pas au pilote.
- **Document propriétaire** : `UI_UX_DESIGN.md`
- **Recommandation** :
  `UI_UX_DESIGN.md` : section Quittance → ajouter « Série d'essai : filigrane « ESSAI » ; organisation de démonstration : filigrane « DÉMONSTRATION — sans valeur » » ; S08/S09 → « pièce d'identité (désactivée au pilote, `ARCHITECTURE.md` §13.2) » ; tableau de contraste → paire du repère « À venir » = celle de `MAQUETTE.md` (`--status-due-fg` sur `--color-accent-100`, 7,3:1) ou l'inverse selon le fichier `tokens.css` réel.
- **Alternative** : Aucune.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-046 — E-20 : que veut dire « 197 lignes valides » ?

- **Gravité** : MINEUR
- **Type** : Ambiguïté
- **Nature** : MÉCANIQUE
- **Où** :
  - Dossier F14 : « 197 lignes sont valides et 3 erreurs » ; `DOMAINE.md` E-20 ; `maquette/data` : `importExemple` `total 200, valides 197, erreurs 3` (vérifié par script) avec deux paires de doublons de téléphone (lignes 33/118 et 61/140) qui sont proposées à la fusion.
  - `TESTING_QA.md` Q6 : « les compteurs de la base (`total = valid + warning + error`) diffèrent de la lecture « 197 valides » (195 valides et 2 avertissements) » ; `API_CONTRACTS.md` Q11 : « 197 lignes valides = valides + avertissements (proposé) ».
- **Constat** : La signification de « valide » dans E-20 n'est pas fixée : valides stricts (195) ou valides + avertissements (197). Le cas est un cas d'ancrage de la maquette, du test d'import (prototype d) et de la vidéo 4.
- **Impact** : Le test de référence de l'import peut être écrit avec deux résultats attendus différents.
- **Document propriétaire** : `DOMAINE.md` (E-20)
- **Recommandation** :
  `DOMAINE.md` E-20 : « 197 lignes **créables** (valides ou avec avertissement), 3 erreurs listées avec leur numéro » et ajouter la ligne « dont 2 doublons de téléphone proposés à la fusion » ; `TESTING_QA.md` Q6 close.
- **Alternative** : Compter 195 valides et 2 avertissements : modifier la maquette, la vidéo 4 et DOMAINE.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-047 — En-têtes « Dépend de » périmés ; six fichiers fondateurs sans en-tête

- **Gravité** : MINEUR
- **Type** : Incohérence
- **Nature** : MÉCANIQUE
- **Où** :
  Les en-têtes « Dépend de » mentionnent des plages périmées (vérifié par script) : `ARCHITECTURE.md` « DP-01 à DP-44, décisions ouvertes n° 1 à 18 » (existent : DP-01 à DP-51 et n° 1 à 95) ; `DATABASE_SCHEMA.md` « DP-01 à DP-49, n° 1 à 29 » ; `SECURITY.md` « DP-01 à DP-51, n° 1 à 47 » ; `API_CONTRACTS.md` « n° 1 à 60 » ; `CODING_RULES.md` « n° 1 à 68 » ; `TESTING_QA.md` « n° 1 à 77 » ; `DEPLOYMENT.md` « n° 1 à 85 » (il ajoute lui-même n° 86–95). `DECISIONS.md`, `CONTEXTE.md`, `DOMAINE.md`, `UI_UX_DESIGN.md`, `MAQUETTE.md` et `CLAUDE.md` n'ont pas d'en-tête (Version, Statut, Dépend de, Alimente).
- **Constat** : Les en-têtes techniques décrivent le journal au moment de leur écriture. Les six fichiers fondateurs n'ont pas de méta-données homogènes.
- **Impact** : Une personne ne sait pas si un document a été relu après telle décision.
- **Document propriétaire** : Chaque document
- **Recommandation** :
  Remplacer dans chaque en-tête technique par « `DECISIONS.md` (état au JJ/MM/AAAA) » sans plage ; ajouter aux six fichiers fondateurs un en-tête `| Champ | Valeur |` (Statut, Version, Date, Dépend de, Alimente).
- **Alternative** : Aucune.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-048 — Quatre documents citent « la consigne »

- **Gravité** : MINEUR
- **Type** : Incohérence
- **Nature** : MÉCANIQUE
- **Où** :
  `ARCHITECTURE.md` Questions ouvertes n° 1, 2, 14, 19 (« La consigne de ce document annonce … ») ; `DATABASE_SCHEMA.md` §9.1 (« consigne : aucun code hors extraits SQL ») ; `CODING_RULES.md` Questions ouvertes n° 1 (« consigne de ce travail ») ; `DEPLOYMENT.md` Questions ouvertes n° 16 (« consigne d'`ARCHITECTURE.md` »).
- **Constat** : Quatre documents citent « la consigne », un message de commande qui n'est pas dans le dossier. La phrase est incompréhensible sans elle.
- **Impact** : Un lecteur ne peut pas vérifier ce que « la consigne » imposait.
- **Document propriétaire** : Chaque document
- **Recommandation** :
  Remplacer « la consigne » par la décision qu'elle représente : « décision de Mimis : pas d'enquête avant le développement (DP-52) », « hypothèse de deux développeurs (n° 14) », « identifiants en anglais (n° 18) ».
- **Alternative** : Aucune.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-049 — Sept formes de marqueur juridique

- **Gravité** : MINEUR
- **Type** : Incohérence
- **Nature** : MÉCANIQUE
- **Où** :
  Marqueurs juridiques : `[À VÉRIFIER AVEC UN JURISTE]` (SECURITY, DEPLOYMENT) ; `[À faire valider par un juriste]` (CONTEXTE l.191, dossier §2.4) ; `[règle à valider avec un juriste]` (DOMAINE R-024) ; `[À valider par un juriste]` (DOMAINE R-090) ; `[mentions à valider, décision n° 12]` (DOMAINE R-052) ; `[à valider juridiquement]` (UI, MAQUETTE) ; `[À valider]` seul (CONTEXTE l.192-193).
- **Constat** : Sept formes pour marquer un élément juridique. `SECURITY.md` n'en affirme aucun comme acquis ; la consigne de l'audit demande le marqueur unique `[À VÉRIFIER AVEC UN JURISTE]`.
- **Impact** : Une recherche du marqueur oublie une partie des éléments juridiques.
- **Document propriétaire** : Chaque document
- **Recommandation** :
  Normaliser en `[À VÉRIFIER AVEC UN JURISTE]` tout élément juridique (R-024, R-052, R-053, R-090, R-092, CONTEXTE « Cadre légal », UI Quittance) ; garder `[À valider]` pour les choix non juridiques.
- **Alternative** : Aucune.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-050 — MAQUETTE.md : faits périmés (maquette « pas encore créée », Node, logo, volumes)

- **Gravité** : MINEUR
- **Type** : Terme obsolète
- **Nature** : MÉCANIQUE
- **Où** :
  - `MAQUETTE.md` l.3 : « La maquette elle-même n'est pas encore créée » (`maquette/` existe : 384 fichiers) ; l.1 : « Fichier temporaire : à archiver après les vidéos ».
  - l.109 : « Node n'est pas installé sur le poste de travail (constaté) » ; `maquette/NOTES.md` : « Node 24 est maintenant présent » (vérifié : `node --version` → v24.20.0 ; `node scripts/verifier-donnees.js` → 28 contrôles réussis sur 28).
  - l.121-133 : structure prévue (`donnees/`, `css/components.css`, `js/ecrans/`) différente de la structure réelle (`data/`, `css/composants.css`, `scripts/verifier-donnees.js`).
  - l.185 : jours d'échéance « 1, 4, 5, 10, 13, 14, 15, 20, 24, 25 et 30 » ; données réelles : + le jour 16 (`maquette/NOTES.md`) ; l.226 : « Environ 100 paiements » ; données : 94 paiements (dont 1 contre-passation) et 93 quittances.
  - l.602 : « Aucun fichier de logo n'a été fourni » ; `DECISIONS.md` DP-44 : « Le logo n'est pas fourni » ; `UI_UX_DESIGN.md` Questions ouvertes : même phrase ; `maquette/NOTES.md` « Logo (26/10/2026) » : logo livré.
- **Constat** : Le brief de la maquette décrit un état antérieur à sa réalisation. Ces faits sont corrigés dans `maquette/NOTES.md` et `maquette/README.md` mais pas dans le brief, ni dans DP-44 et UI pour le logo.
- **Impact** : Faible : le brief est « temporaire » ; mais DP-44 est une décision enregistrée dont une phrase est fausse.
- **Document propriétaire** : `MAQUETTE.md` ; `DECISIONS.md` (DP-44)
- **Recommandation** :
  `MAQUETTE.md` : corriger ces points ; DP-44 : ajouter « Logo livré le 26/10/2026 (`logo/`) » ; `UI_UX_DESIGN.md` Questions ouvertes : retirer la phrase sur le logo ; `CLAUDE.md` : renvoyer à `maquette/NOTES.md`.
- **Alternative** : Archiver `MAQUETTE.md` (« fichier temporaire ») et ne garder que `maquette/NOTES.md` : à décider quand les vidéos sont finies.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-051 — Préfixe /api écrit de deux façons dans API_CONTRACTS.md

- **Gravité** : MINEUR
- **Type** : Incohérence
- **Nature** : MÉCANIQUE
- **Où** :
  `API_CONTRACTS.md` Q15 : « le texte cite `POST /payments` et `POST /api/events` ; ce document normalise sur le préfixe `/api/` » ; §7.1 : « Tous les chemins sont préfixés par `/api` » alors que les tables de §7 écrivent `POST /auth/login`, `GET /me`, `GET /payments` sans préfixe ; `ARCHITECTURE.md` §8.4 : `POST /api/events`.
- **Constat** : Les chemins sont écrits sans préfixe dans les tables de §7 et avec préfixe dans §2 à §6 et dans ARCHITECTURE. §7.1 explique la convention, mais `openapi.json` devra la fixer.
- **Impact** : Risque de routes dupliquées ou incohérentes dans le premier dépôt.
- **Document propriétaire** : `API_CONTRACTS.md`
- **Recommandation** :
  `API_CONTRACTS.md` Q15 close dans le lot technique (DP-56) : préfixe `/api/` pour toutes les routes sauf `/health`, `/ready`, `/s/…`, `/v/…`, `/console/…`.
- **Alternative** : Aucune.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-052 — Trois seuils de taille de PR

- **Gravité** : MINEUR
- **Type** : Incohérence
- **Nature** : MÉCANIQUE
- **Où** :
  `CODING_RULES.md` §6 : « une PR de moins de 150 lignes qui passe la CI peut être fusionnée sur une relecture rapide » ; §8.2 : « Cible : moins de 400 lignes modifiées … au-delà de 800, la PR est découpée » ; §9.3 : « moins de 400 lignes » ; `DECISIONS.md` n° 74 : « PR de moins de 400 lignes ».
- **Constat** : Trois seuils (150, 400, 800) sans lien explicite.
- **Impact** : Confusion à la revue.
- **Document propriétaire** : `CODING_RULES.md`
- **Recommandation** :
  §6 : « … hors chemins sensibles, une PR de moins de 150 lignes qui passe la CI peut être fusionnée après une relecture rapide ; la cible générale est de 400 lignes (§8.2) ».
- **Alternative** : Aucune.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-053 — Logo : 200 Ko à l'envoi, 30 Ko au budget d'affichage

- **Gravité** : MINEUR
- **Type** : Incohérence
- **Nature** : MÉCANIQUE
- **Où** :
  `SECURITY.md` §7.4 : « Logos et signatures de l'organisation : PNG ou JPEG, **200 Ko** au plus ; réencodés » ; `UI_UX_DESIGN.md` Performance : « Logo ≤ 30 Ko » ; `TESTING_QA.md` §8.3 : « Images d'une page / logo ≤ 100 Ko / ≤ 30 Ko » ; quittance : logo 40 × 20 mm.
- **Constat** : Le budget d'affichage est 30 Ko, le plafond d'envoi 200 Ko, et le réencodage n'est pas dit réduire à 30 Ko.
- **Impact** : Un logo de 200 Ko accepté dépasse le budget de poids de chaque page qui l'affiche.
- **Document propriétaire** : `UI_UX_DESIGN.md` (Performance) ; `SECURITY.md` (envoi)
- **Recommandation** :
  `SECURITY.md` §7.4 : « … 200 Ko au plus à l'envoi, réencodé et réduit à 30 Ko pour l'affichage (`UI_UX_DESIGN.md`, Performance) ».
- **Alternative** : Ramener le plafond d'envoi à 100 Ko.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-054 — Le premier audit n'a pas de section « Corrections appliquées »

- **Gravité** : MINEUR
- **Type** : Manque
- **Nature** : MÉCANIQUE
- **Où** :
  `ARCHITECTURE.md` Questions ouvertes n° 15 : « `AUDIT_COHERENCE.md` n'a pas de section « Corrections appliquées » » ; `maquette/NOTES.md` : « Il manque toujours une vraie section ». `AUDIT_COHERENCE.md` l.7 : paragraphe « Suivi (24/10/2026) » seulement.
- **Constat** : Le premier audit n'a pas de relevé de ce qui a été appliqué. Le présent audit en aura un (Phase B).
- **Impact** : Traçabilité de la première vague incomplète.
- **Document propriétaire** : `AUDIT_COHERENCE.md` (historique)
- **Recommandation** :
  Ne pas modifier `AUDIT_COHERENCE.md` (historique). Ajouter en tête de `AUDIT_COHERENCE_2.md` une ligne « Audit précédent : `AUDIT_COHERENCE.md` (constats traités par DP-29 à DP-43) » et fermer la question n° 15 d'`ARCHITECTURE.md`.
- **Alternative** : Aucune.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-055 — « En trente secondes » : promesse chiffrée avant mesure

- **Gravité** : MINEUR
- **Type** : Invention non sourcée
- **Nature** : MÉCANIQUE
- **Où** :
  - `MAQUETTE.md` Vidéo 1 : « **Phrase-clé** : « Un paiement, une quittance, en trente secondes. » » ; `CONTEXTE.md` Promesse : « Le caissier encaisse en moins de 30 secondes » ; dossier §14 et KPI §23 : « Moins de 30 secondes (médiane) » ; `TESTING_QA.md` PP-25 : mesure à faire sur un Android d'entrée de gamme.
  - `site/NOTES.md` n° 2 : « « Retrouvables en quelques secondes » (bénéfice 2) : formulation de votre texte, non chiffrée mais non mesurée non plus ».
- **Constat** : La cible de 30 secondes est un objectif produit mesuré plus tard (PP-25), pas un résultat. `film/VOIX_OFF.md` s'impose « aucune promesse chiffrée sur un gain » ; la phrase-clé de la vidéo 1 de `MAQUETTE.md` en contient une.
- **Impact** : Promesse chiffrée non démontrée dans une communication publique.
- **Document propriétaire** : `MAQUETTE.md` / `film/VOIX_OFF.md`
- **Recommandation** :
  `MAQUETTE.md` Vidéo 1 : « Un paiement, une quittance, en quelques secondes. » ; `CONTEXTE.md` Promesse : « Le caissier vise moins de 30 secondes (cible mesurée au pilote, `TESTING_QA.md` PP-25) ».
- **Alternative** : Garder « trente secondes » une fois PP-25 mesuré.
- **Question pour Mimis** : —
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

### C2-056 — Ordre de construction de T2 : import avant ou après les mandats

- **Gravité** : MINEUR
- **Type** : Manque
- **Nature** : DÉCISION
- **Où** :
  `ARCHITECTURE.md` §9.2, remarque : « Le dossier (§18) place l'import (n° 13) avant les mandats (n° 14). L'ordre demandé place l'import en dernier. … un import par ligne de commande réservé à HABEK peut être livré dès la tranche 2, l'assistant S16 venant en tranche 6 [À valider] ».
- **Constat** : Le dossier (§18) et `ARCHITECTURE.md` donnent deux ordres pour T2. Le dossier justifie l'import « avant le pilote » car il dépend du schéma des baux.
- **Impact** : Ordre de construction de T2 à fixer pour la roadmap.
- **Document propriétaire** : `ARCHITECTURE.md` §9.2
- **Recommandation** :
  Valider l'ordre d'`ARCHITECTURE.md` §9.2 (mandats et relevé, puis import et console) avec l'import par ligne de commande dès la tranche 2, pour que l'agence pilote charge ses données réelles pendant le développement. Ajouter l'écart au dossier §18 dans la liste des écarts assumés.
- **Alternative** : Suivre l'ordre du dossier (import avant mandats).
- **Question pour Mimis** : L'import par ligne de commande réservé à HABEK dès la tranche 2 vous convient-il ?
- **Décision de Mimis** : [x] OK recommandation  [ ] OK alternative  [ ] Autre : ________ (30/10/2026, validation en bloc : « le reste, applique tes recommandations »)

## C. Plan de correction

Règle générale : décision de Mimis d'abord, puis le document propriétaire, puis les documents qui en dépendent. Les décisions DP proposées sont numérotées dans l'ordre d'application (la première, DP-52, est le prochain numéro libre).

| Étape | Fichier | Constats | Dépend de |
|---|---|---|---|
| 0 | (décisions de Mimis) | Réponses aux questions de C2-001, C2-003, C2-007, C2-004, C2-006, C2-008, C2-009, C2-010, C2-012, C2-013, C2-014, C2-015, C2-041, C2-043, C2-029, C2-030, C2-040, C2-020, C2-033, C2-024, C2-025, C2-028, C2-056, C2-005 | — |
| 1 | `DECISIONS.md` | Entrées DP-52, DP-53, DP-54, DP-55, DP-56, DP-57, DP-58, DP-59, DP-60, DP-61, DP-62, DP-63, DP-64, DP-65, DP-66, DP-67 ; statuts des DP remplacées ; section « Écarts assumés par rapport au dossier » ; nettoyage ; échéances des décisions de la phase 0 : C2-001, C2-003, C2-007, C2-005, C2-020, C2-017, C2-039, C2-029 | Étape 0 |
| 2 | `DOMAINE.md` | Règles manquantes et marqueurs : C2-006, C2-008, C2-009, C2-010, C2-040, C2-046, C2-019, C2-049 | Étapes 0 et 1 |
| 3 | `CONTEXTE.md` | Statut, Go/No-Go, hypothèses, éditeur, plans, matrice : C2-016, C2-014, C2-029, C2-030, C2-049 | Étapes 1 et 2 |
| 4 | `SECURITY.md` | TOTP, gravités, conservation, suppressions, logo : C2-013, C2-022, C2-041, C2-042, C2-053, C2-043 | Étapes 0 et 1 |
| 5 | `DATABASE_SCHEMA.md` | Colonnes `version` / `reauth_until` selon n° 64, PGlite, en-tête, fonction de purge : C2-027, C2-038, C2-047, C2-041 | Étapes 1 et 4 |
| 6 | `ARCHITECTURE.md` | Alignement sur DB et SECURITY, coûts, calendrier, consigne, en-tête : C2-026, C2-013, C2-023, C2-044, C2-048, C2-047, C2-035, C2-056, C2-004 | Étapes 1, 4 et 5 |
| 7 | `API_CONTRACTS.md` | Chemins, T3, en-tête : C2-051, C2-031, C2-033, C2-047 | Étapes 5 et 6 |
| 8 | `CODING_RULES.md` | Seuils de PR, consigne, en-tête : C2-052, C2-048, C2-047 | Étape 6 |
| 9 | `TESTING_QA.md` | Liste unique « prêt pour un pilote », matrice règle → test, T3, gravités, couverture : C2-035, C2-032, C2-031, C2-019, C2-022, C2-044, C2-043, C2-047 | Étapes 2, 4 et 6 |
| 10 | `DEPLOYMENT.md` | Coûts, calendrier, Go/No-Go, domaine, couverture, renvoi « prêt » : C2-023, C2-044, C2-019, C2-035, C2-014, C2-043, C2-047, C2-011 | Étapes 1, 4 et 9 |
| 11 | `UI_UX_DESIGN.md` | S01, filigranes, variables, matrice : C2-028, C2-045, C2-040, C2-030, C2-019, C2-050 | Étapes 1 à 4 |
| 12 | `MAQUETTE.md`, `maquette/README.md` | Objectif, usage, faits, promesse : C2-018, C2-050, C2-055 | Étapes 1 et 11 |
| 13 | `site/`, `flyer/`, `film/` | Promesses et appel à l'action : C2-024, C2-025, C2-055 | Étape 1 |
| 14 | `CLAUDE.md` | Phase actuelle, index, hiérarchie, variables, légende : C2-002, C2-036, C2-021, C2-037, C2-011 | Toutes les étapes précédentes |
| 15 | `TRACABILITE.md` (nouveau) | Matrices corrigées F01–F40 et R-001–R-097 : C2-031, C2-032 | Étapes 2, 5, 7, 9, 11 |

**Dépendances clés.**

- C2-001 précède C2-002, C2-016, C2-018, C2-019, C2-017 et C2-025.
- C2-013 (n° 50) précède l'alignement d'`ARCHITECTURE.md` et la liste unique de `TESTING_QA.md` (C2-035).
- C2-004 précède la migration initiale ; C2-027 (n° 64) précède toute migration.
- C2-006 précède les tests d'or de T1 ; C2-008 précède la tranche 5 (compte rendu).
- C2-010 précède C2-026 (séries de numérotation) et la rédaction des quittances d'essai.

**Numérotation proposée des décisions.**

| DP | Titre | Constat |
|---|---|---|
| DP-52 | Pas d'enquête terrain avant le développement ; pilote avec données réelles | C2-001 |
| DP-53 | Critères de continuation, correction ou arrêt du pilote | C2-001 |
| DP-54 | Capacité de développement (personnes, heures) | C2-003 |
| DP-55 | Hébergement et exploitation de PostgreSQL (ferme n° 5, 21, 46, 86) | C2-004 |
| DP-56 | Pile technique de référence (lot de validation en bloc) | C2-005 |
| DP-57 | Premier segment de pilote, prix testés pendant le pilote, échéances des décisions de l'ancienne phase 0 | C2-007 |
| DP-58 | Règles de T1 : échéances, promesse tenue, doublon de référence, taux de recouvrement, avenant, téléphone | C2-006 |
| DP-59 | Règles du compte rendu propriétaire (T2) et règle de reddition R-071 | C2-008 |
| DP-60 | Exceptions à R-005 et R-007 (annulation d'import, remise à zéro en phase setup) | C2-009 |
| DP-61 | Cycle setup puis live, séries de numérotation et année de la séquence | C2-010 |
| DP-62 | Avis juridique avant le premier pilote (CIL, mentions, conservation) | C2-012 |
| DP-63 | Éditeur, nom du produit et domaine | C2-014 |
| DP-64 | Quittance : contenu, format et vérification | C2-015 |
| DP-65 | Plans et paliers commerciaux | C2-029 |
| DP-66 | Compléments à la matrice des permissions | C2-030 |
| DP-67 | Langue des identifiants de code (ferme n° 18) | C2-017 |

## D. Ce qui est cohérent

- Les 40 calculs de référence (cas E-xx, volumes de la maquette, retards en jours, conversions, durées des vidéos) sont exacts ; le jeu de données de la maquette passe ses 28 contrôles.
- Montants : entiers de FCFA, `BIGINT`, aucune décimale, un seul arrondi (commission, 0,5 vers le haut) décrit de la même façon dans DOMAINE, ARCHITECTURE §4.2, DATABASE_SCHEMA §4.6, API_CONTRACTS §2.2, CODING_RULES §4.2 et TESTING_QA P11.
- Tranches T1, T2, T3 et priorités P0 à P3 : CONTEXTE, ARCHITECTURE §9.2, UI (catalogue), API (étiquettes P/T1…) et TESTING_QA §2.1 concordent ; le dossier §9.1 est étendu par DP-29 sans contradiction.
- Les identifiants F, R, E, DP, S et les 426 renvois de section sont tous valides ; les décisions n° 1 à 95 sont continues et rangées par document d'origine.
- La matrice de permissions de `SECURITY.md` §4.1 est identique à celle de `CONTEXTE.md` ; `API_CONTRACTS.md` §4.6 la reprend et y ajoute le plan Propriétaire.
- Idempotence (clé de 30 jours, même transaction), isolation (`OrgTx`, RLS forcée, 404 hors organisation), écriture seule (25 tables), sessions serveur et limitation du débit : décrites de la même façon dans ARCHITECTURE, DATABASE_SCHEMA, SECURITY, API_CONTRACTS et CODING_RULES.
- Objectifs de sauvegarde (15 min, 4 h, 30 jours, 12 copies mensuelles, test mensuel) et alertes de 24 h identiques dans ARCHITECTURE, SECURITY, DEPLOYMENT et TESTING_QA.
- Budgets de poids et de performance (200 / 150 / 30 Ko, LCP 3 s, 25 lignes, p95 serveur) identiques dans UI, ARCHITECTURE, CODING_RULES et TESTING_QA.
- Exclusions (copropriété, paiement en ligne, envoi automatique WhatsApp, hors-connexion) absentes de tout écran, route, table et vidéo : routes `/api/online-payments` et `/api/messaging` réservées (404) ; seule exception, le site (C2-024).
- Vocabulaire : « quittance » et « reçu » ne sont pas confondus (« reçu » n'apparaît que dans « Total reçu » et « reçu par »), mandant, relevé / compte rendu, bien / lot et impayé / arriéré sont utilisés conformément au glossaire de `DOMAINE.md` et au tableau « Vocabulaire » d'`UI_UX_DESIGN.md`.

## E. Annexes

### Annexe 1 — Matrice de traçabilité F01 à F40

Générée par script à partir du dossier (priorité), de `CONTEXTE.md` (tranche), du catalogue d'`UI_UX_DESIGN.md` (écrans), du catalogue d'`API_CONTRACTS.md` §7 (routes ; étiquette entre crochets), des fiches de `DATABASE_SCHEMA.md` §3 (tables) et de `TESTING_QA.md` (cas E-xx cités, parcours, familles de tests). « Statut au pilote » : **Au démarrage** = T1 et T2 (le pilote démarre à la fin de T2, DP-29) ; **Pendant** = T3, livré pendant le pilote ; **Hors V1** = non planifié.

| F | Fonctionnalité | Prio | Tranche | Écran(s) | Endpoint(s) | Table(s) | Test(s) | Statut au pilote | Manque |
|---|---|---|---|---|---|---|---|---|---|
| F01 | Comptes, organisation et plans | P0 | T1 | S01, S02, S17, S18 | POST /auth/login [P/T1]; POST /auth/totp/verify [P/T1]; POST /auth/logout (+1) [P/T1]; GET /me [P/T1] … (+11 routes) | organizations, users, memberships, invitations, sessions, login_attempts, password_resets, org_settings | ISO-1 à ISO-9, A1, A2, A11, PP-08 à PP-10, PP-17, E2E transverses | Au démarrage |  |
| F02 | Rôles et permissions | P0 | T1 | S17, S18 | GET /me [P/T1]; GET /users [P/T1]; PATCH /users/{membership_id} [P/T1]; POST /users/{membership_id}/disable (+1) [P/T1] … (+2 routes) | memberships, role_settings, unit_assignments | A3, A5, G-16, PP-11 | Au démarrage |  |
| F03 | Journal d'audit | P0 | T1 | S17, S21 | GET /audit [P/T1]; PUT, PATCH, DELETE /audit/{id} [P/T1] | support_grants, audit_log | G-05, ISO-4, PP-01, règle CODING §7 n° 9 | Au démarrage |  |
| F04 | Biens et lots | P0 | T1 | S06, S07 | GET /properties [P/T1]; POST /properties [P/T1]; GET /properties/{id} (+1) [P/T1]; POST /properties/{id}/archive [P/T1] … (+6 routes) | properties, units | E2E-P5 | Au démarrage |  |
| F05 | Propriétaires mandants et mandats | P0 (Agence) | T2 | S12, S13 | GET /owners (+1) [P/T2]; GET /owners/{id} (+1) [P/T2]; GET /owners/{id}/payout-details [P/T2]; GET /mandates (+1) [P/T2] … (+3 routes) | owners, mandates, mandate_units, mandate_rate_changes | E-16, E-17, E2E-P3 | Au démarrage |  |
| F06 | Fiche locataire | P0 | T1 | S08, S09 | GET /tenants [P/T1]; POST /tenants [P/T1]; GET /tenants/{id} (+1) [P/T1]; GET /tenants/{id}/leases (+2) [P/T1] … (+3 routes) | tenants | G-19 (R-017, R-018), A10, E2E-P1 | Au démarrage |  |
| F07 | Baux | P0 | T1 | S07, S10 | GET /units/{id}/leases (+2) [P/T1]; GET /leases [P/T1]; POST /leases [P/T1]; GET /leases/{id} (+1) [P/T1] … (+3 routes) | leases, lease_tenants, lease_commercial_terms, lease_amendments | E-01, E-02, E-22, E2E-P5 | Au démarrage |  |
| F08 | Échéancier automatique | P0 | T1 | S10 | POST /leases/{id}/activate [P/T1]; GET /leases/{id}/installments [P/T1]; POST /leases/{id}/installment-lines [P/T1]; POST /installment-lines/{id}/void [P/T1] | installments, installment_lines, installment_line_voids, jobs | E-03, E-04 | Au démarrage |  |
| F09 | Enregistrement des paiements | P0 | T1 | S03, S04, S09, S11 | GET /leases/{id}/payment-context [P/T1]; GET /payments [P/T1]; GET /payments/{id} [P/T1]; POST /payments/preview [P/T1] … (+4 routes) | payments, allocations, correction_requests, idempotency_keys | E-05, E-06, E-07, E-08, E-09, E2E-P1 | Au démarrage |  |
| F10 | Quittances PDF numérotées | P0 | T1 | S04, S11, S17, S25, S26 | POST /payments [P/T1]; POST /payments/{id}/reversal [P/T1]; GET /receipts [P/T1]; GET /receipts/{id} [P/T1] … (+6 routes) | payments, receipt_counters, receipts, receipt_cancellations, receipt_files, receipt_sends, share_links, org_settings, jobs | E-10, E-11, E-12, E2E-P1 | Au démarrage |  |
| F11 | Tableau de bord des impayés | P0 | T1 | S03, S05 | GET /home [P/T1]; GET /arrears [P/T1] | vues `v_installment_*` (calcul, pas de table) | E-13, E-13b, E-25, P9, P10, E2E-P2 | Au démarrage |  |
| F12 | Relances assistées | P0 | T1 | S03, S05, S09, S17 | GET /home [P/T1]; GET /reminders [P/T1]; GET /reminders/due [P/T1]; POST /reminders/draft (+1) [P/T1] … (+5 routes) | message_templates, reminders, reminder_installments, promises | E-14, E-15, E2E-P2 | Au démarrage |  |
| F13 | Compte rendu propriétaire | P0 (Agence) | T2 | S12, S26 | GET /statements [P/T2]; GET /statements/preview [P/T2]; POST /statements (+4) [P/T2]; GET /statements/{id} (+1) [P/T2] … (+2 routes) | share_links, statements, statement_versions, statement_reopenings, statement_payment_links, statement_expense_links, payouts, expense_imputation_changes | E-18, E-19, E2E-P3 | Au démarrage |  |
| F14 | Import guidé Excel / CSV | P0 | T2 | S02, S16 | GET /import-batches [P/T2]; GET /import-batches/{id}/rows [P/T2]; GET /import-mappings [P/T2]; GET /import-template [P/T2] | import_batches, import_rows, import_mappings | E-20, E-21, E2E-P4 | Au démarrage |  |
| F15 | Journal des travaux et dépenses par lot | P1 (V1) | T3 (min. T2) | S07, S12, S14 | GET /units/{id}/leases (+2) [P/T1]; GET /interventions [P/T2]; POST /units/{id}/interventions [P/T2]; GET /interventions/{id} [P/T2] … (+4 routes) | statement_expense_links, providers, interventions, expense_imputation_changes | E2E-P3, E2E-P5 | Minimal au démarrage ; complet pendant |  |
| F16 | Documents par bien, lot, locataire, bail | P1 (V1) | T3 | S07, S14 | GET /units/{id}/leases (+2) [P/T1]; POST /documents [A/T3]; GET /documents [A/T3]; GET /documents/{id}/download [A/T3] … (+1 routes) | documents, document_links | E2E-P5 | Pendant |  |
| F17 | Agenda et alertes | P1 (V1) | T3 | S03, S15 | — (aucune route) | `tasks` (prévue, non créée) | — | Pendant | route, test, table créée |
| F18 | Exports CSV / Excel / PDF pour le comptable | P1 | T3 | S11, S17 (pas d’écran propre) | POST /exports [A/T3]; GET /settings/export-formats [A/T3] | — (aucune) | PP-19 (neutralisation des formules) ; aucun cas E-xx | Pendant |  |
| F19 | Clôture de caisse journalière | P1 | T3 | S19 | GET /cash-days (+1) [A/T3] | cash_days | TESTING:F19 | Pendant |  |
| F20 | Dépôt de garantie | P1 | T3 | S10 | GET /leases/{id}/deposit-events (+1) [A/T3] | `security_deposit_events` (prévue, non créée) | — (R-066, R-067 sans test) | Pendant | test, table créée |
| F21 | Pénalités / frais de retard configurables | P2 | — | — | — | — | — | Hors V1 |  |
| F22 | Notifications internes | P1 | T3 | S20 | GET /home (partiel) ; aucune route propre | `notification_reads` (prévue, non créée) | — | Pendant | test, table créée |
| F23 | Fin de bail et changement de locataire | P1 | T3 | S10 | POST /leases/{id}/notice (+1) [A/T3] | leases | E-35 (renouvellement, R-091) ; R-090 sans test | Pendant |  |
| F24 | États des lieux | P2 | — | S24 | — | — | — | Hors V1 |  |
| F25 | Saisie assistée depuis un SMS mobile money collé | P1 | T3 | S04 | POST /payment-sms/parse [A/T3] | — (aucune) | — (aucun cas ; exemples réels de SMS à fournir) | Pendant | test |
| F26 | Paiement en ligne par le locataire | P2 | — | — | — | — | — | Hors V1 |  |
| F27 | WhatsApp Business API | P2 | — | — | — | — | — | Hors V1 |  |
| F28 | Portail propriétaire mandant | P2 | — | S23 | — | — | — | Hors V1 |  |
| F29 | Espace locataire | P3 | — | — | — | — | — | Hors V1 |  |
| F30 | Révision / indexation du loyer | P2 | — | — | — | — | — | Hors V1 |  |
| F31 | Abonnement Immo Suite en ligne | P2 | — | — | — | subscriptions (prévue, non créée) | — | Hors V1 |  |
| F32 | Rapports avancés | P2 | — | S22 | — | — | — | Hors V1 |  |
| F33 | Application mobile hors-ligne pour agents terrain | P3 | — | — | — | — | — | Hors V1 |  |
| F34 | Signature électronique | P3 | — | — | — | — | — | Hors V1 |  |
| F35 | Copropriété | P3 | — | — | — | — | — | Hors V1 |  |
| F36 | Vitrine et annonces de biens vacants | P3 | — | — | — | — | — | Hors V1 |  |
| F37 | Assistant IA conversationnel | P3 | — | — | — | — | — | Hors V1 |  |
| F38 | (n'existe pas dans le dossier) | — | — | — | — | — | — | — | Trou de numérotation |
| F39 | Sauvegardes chiffrées, restauration testée, journalisation technique | P0 | T1 | — (opération) | commandes `immo …` (hors API) | — (opération) | PP-13 à PP-16 ; test mensuel de restauration (DEPLOYMENT §5.4) | Au démarrage |  |
| F40 | Console HABEK | P0 | T2 | S17 (accès support) ; console hors catalogue | /console/… réservées (7.18) ; /support-grants [P/T2] | support_grants, import_batches, platform_staff, platform_events, org_features | ISO-8, PP-12 ; console non spécifiée | Au démarrage |  |

**Éléments techniques sans fonctionnalité source** (relevé manuel) : tables `product_events`, `error_reports`, `feedback_reports`, `org_features`, `org_settings`, `jobs`, `purge_order` (observation du pilote et interrupteurs : `ARCHITECTURE.md` §8.2 à §8.4, sans F-xx) ; routes `POST /events`, `POST /client-errors`, `POST /feedback`, `GET /search`, `GET /health` ; écrans S25 et S26 (rattachés à F10 et F13) ; commandes `immo org export`, `immo org purge`, `immo anonymize`, `immo demo reset`. Aucune n'est dans le dossier ; elles sont justifiées par `ARCHITECTURE.md` §8 mais n'ont pas de F-xx : à rattacher à F39/F40 ou à une ligne « Exploitation du pilote » dans `CONTEXTE.md`.

### Annexe 2 — Matrice R-xxx → base → API → écran → test

Générée par script par **citation explicite** de l'identifiant (le fait qu'une règle soit couverte « par famille » sans être citée n'est pas compté). Base : tables dont la fiche cite la règle, `§4` = garantie de `DATABASE_SCHEMA.md` §4. API : code d'erreur ou route qui cite la règle ; « mention » = citée seulement en texte. Tests : cas E-xx ou règle citée dans `TESTING_QA.md` ; « famille » = couverture relevée à la main dans une famille de tests (`TESTING_QA.md`, `DATABASE_SCHEMA.md` §9) qui ne cite pas l'identifiant. « Calcul » : règle portée par le crate `domain` (DP-51), sans mécanisme en base.

| Règle | Portée | Base | API | UI | Maquette | Tests cités | Écart |
|---|---|---|---|---|---|---|---|
| R-001 | Tous | payments +§4 | mention | UI | MAQ | TESTING(R) |  |
| R-002 | Tous | organizations | mention | UI | — | famille : TESTING §3 Horloge (parité Clock / app_now), G-04 |  |
| R-003 | Tous | organizations | mention | — | — | — | écran, test |
| R-004 | Tous | organizations +§4 | resource.not_found | UI | — | famille : A4, ISO-1 à ISO-9, PP-08 à PP-10 |  |
| R-005 | Tous | properties, units | route | — | — | — | écran, test |
| R-006 | Tous | units +§4 | route, unit.has_active_lease | UI | — | — | test |
| R-007 | Tous | audit_log +§4 | route | UI | — | famille : G-05, ISO-4, A9 |  |
| R-008 | Tous | audit_log | — | — | — | famille : CODING §7 n° 9 (l'action échoue si l'audit échoue) | API, écran |
| R-009 | Tous | audit_log, platform_events, platform_staff … +§4 | route | UI | — | famille : ISO-8, PP-12 |  |
| R-010 | Tous | properties | route | — | — | — | écran, test |
| R-011 | Tous | units | route, unit.reference_duplicate | — | — | — | écran, test |
| R-012 | Habitation, Commercial | units | route | — | — | — | écran, test |
| R-013 | Tous | Calcul | mention | — | MAQ | — | test |
| R-014 | Agence | mandate_units, mandates +§4 | mandate.overlap, route | — | MAQ | — | test |
| R-015 | Agence | mandates | mandate.rate_out_of_range, route | — | — | — | écran, test |
| R-016 | Tous | owners | route | — | — | — | écran, test |
| R-017 | Tous | tenants +§4 | route, tenant.phone_duplicate | UI | — | TESTING(R) |  |
| R-018 | Tous | tenants, users +§4 | route | — | — | — | écran, test |
| R-019 | Tous | tenants | route | UI | — | — | test |
| R-020 | Tous | leases +§4 | lease.overlap, payment.lease_not_in_progress, route | UI | MAQ | — | test |
| R-021 | Tous | — | route | — | MAQ | — | base, test |
| R-022 | Tous | — | route | UI | MAQ | famille : G-03, A10 | base |
| R-023 | Tous | lease_amendments, leases +§4 | lease.amendment_retroactive, lease.not_draft, route | — | MAQ | E-02 |  |
| R-024 | Habitation | Calcul | route | UI | MAQ | E-22 |  |
| R-025 | Commercial | lease_commercial_terms, leases | — | — | — | — | API, écran, test |
| R-026 | Tous | lease_commercial_terms | — | — | — | — | API, écran, test |
| R-027 | Tous | installments | route | — | MAQ | E-03, TESTING(R) |  |
| R-028 | Tous | Calcul | mention | — | — | E-01 | écran |
| R-029 | Tous | installments, jobs | mention | — | — | E-03, TESTING(R) | écran |
| R-030 | Tous | installments | — | — | — | — | API, écran, test |
| R-031 | Tous | leases | — | — | — | — | API, écran, test |
| R-032 | Tous | Calcul | — | UI | MAQ | TESTING(R) | API |
| R-033 | Tous | Calcul | mention | UI | MAQ | E-04, E-25, E-37, TESTING(R) |  |
| R-034 | Tous | installments | — | — | — | — | API, écran, test |
| R-035 | Tous | installment_lines | route | — | — | TESTING(R) | écran |
| R-036 | Tous | installment_line_voids, installment_lines +§4 | route | — | — | — | écran, test |
| R-037 | Tous | payments +§4 | route | UI | — | famille : G-03, A10 |  |
| R-038 | Tous | org_settings | payment.confirmation_required, route | UI | MAQ | E-09, TESTING(R) |  |
| R-039 | Tous | payments +§4 | mention | UI | MAQ | — | test |
| R-040 | Tous | payments | mention | UI | — | — | test |
| R-041 | Tous | payments | payment.external_ref_duplicate | UI | MAQ | TESTING(R) |  |
| R-042 | Tous | idempotency_keys, payments +§4 | mention | UI | MAQ | E-07, TESTING(R) |  |
| R-043 | Tous | allocations | route | UI | — | TESTING(R) |  |
| R-044 | Tous | allocations +§4 | payment.allocation_invalid, route | — | MAQ | TESTING(R) |  |
| R-045 | Tous | Calcul | — | — | — | E-05, E-25 | API, écran |
| R-046 | Tous | allocations | payment.surplus_not_confirmed, route | UI | — | E-06, TESTING(R) |  |
| R-047 | Tous | payments +§4 | payment.immutable, route | UI | MAQ | E-08, TESTING(R) |  |
| R-048 | Tous | correction_requests, payments +§4 | payment.already_reversed, payment.reversal_reason_required, route | UI | MAQ | E-08, TESTING(R) |  |
| R-049 | Tous | jobs, receipt_files, receipts | receipt.pdf_not_ready, route | UI | MAQ | TESTING(R) |  |
| R-050 | Tous | receipt_counters, receipts +§4 | — | UI | MAQ | E-10, E-27, E-38, TESTING(R) | API |
| R-051 | Tous | receipt_cancellations, receipts | receipt.immutable, route | UI | MAQ | E-38 |  |
| R-052 | Tous | receipts | — | UI | — | famille : TESTING §6 (textes attendus du PDF) | API |
| R-053 | Tous | receipts | mention | UI | — | E-11, TESTING(R) |  |
| R-054 | Tous | org_settings, share_links | route, share_link.expired | UI | — | E-12 |  |
| R-055 | Tous | receipt_sends | route | UI | — | — | test |
| R-056 | Tous | receipt_files | route | — | — | famille : TESTING §6 (une page en A4 et en A5) | écran |
| R-057 | Tous | Calcul | route | — | — | TESTING(R) | écran |
| R-058 | Tous | Calcul | mention | — | MAQ | E-13, E-13b, TESTING(R) |  |
| R-059 | Tous | promises | route | UI | MAQ | TESTING(R) |  |
| R-060 | Tous | message_templates | route | UI | — | E-14 |  |
| R-061 | Tous | reminders | mention | — | — | famille : E2E parcours 2 (WhatsApp assisté) | écran |
| R-062 | Tous | reminder_installments, reminders | route | — | — | famille : A9 (reminder.immutable) | écran |
| R-063 | Tous | reminders | mention | UI | MAQ | TESTING(R) |  |
| R-064 | Tous | message_templates | route, template.unknown_variable, template.variable_missing | UI | MAQ | E-14, TESTING(R) |  |
| R-065 | Tous | promises | route | — | — | E-15 | écran |
| R-066 | Tous | — | route | — | — | — | base, écran, test |
| R-067 | Tous | — | route | — | — | — | base, écran, test |
| R-068 | Agence | mandates, statement_versions +§4 | route | — | MAQ | E-16, E-26, E-28, E-32 |  |
| R-069 | Agence | mandate_rate_changes +§4 | mandate.rate_change_retroactive, route | — | — | E-17 | écran |
| R-070 | Agence | statement_versions +§4 | mention | — | MAQ | E-18, TESTING(R) |  |
| R-071 | Agence | Calcul | mention | — | MAQ | — | test |
| R-072 | Agence | statement_payment_links, statements +§4 | statement.preview_stale | — | — | TESTING(R) | écran |
| R-073 | Agence | statement_payment_links, statement_versions | route, statement.already_validated, statement.preview_stale | — | — | E-19 | écran |
| R-074 | Agence | statement_expense_links, statement_versions +§4 | expense.frozen, route | — | — | TESTING(R) | écran |
| R-075 | Agence | Calcul | route, statement.units_without_mandate | UI | MAQ | TESTING(R) |  |
| R-076 | Agence | payouts, statement_versions +§4 | route | — | — | — | écran, test |
| R-077 | Agence | payouts +§4 | payout.amount_mismatch, route | — | — | — | écran, test |
| R-078 | Agence | interventions | route | — | — | — | écran, test |
| R-079 | Tous | cash_days | route | — | — | — | écran, test |
| R-080 | Tous | import_batches, import_rows, owners | import.not_cancellable, import.wrong_state, route | UI | MAQ | E-20, E-21 |  |
| R-081 | Tous | memberships, role_settings, unit_assignments +§4 | auth.forbidden, route, user.last_manager | UI | — | — | test |
| R-082 | Tous | mandates, organizations +§4 | feature.disabled | — | — | — | écran, test |
| R-083 | Agence | payouts, statement_reopenings, statement_versions +§4 | route, statement.reopen_forbidden | UI | — | E-30 |  |
| R-084 | Agence | expense_imputation_changes, interventions, statement_expense_links | route | — | MAQ | E-29 |  |
| R-085 | Agence | Calcul | mention | — | — | — | écran, test |
| R-086 | Tous | allocations | mention | — | — | E-31, TESTING(R) | écran |
| R-087 | Tous | allocations, installment_lines | route | UI | MAQ | E-32, TESTING(R) |  |
| R-088 | Tous | lease_tenants, payments | route | UI | — | E-33 |  |
| R-089 | Tous | cash_days, payments | route | UI | — | E-34 |  |
| R-090 | Tous | leases | route | — | — | — | écran, test |
| R-091 | Tous | lease_amendments, leases +§4 | route | — | — | E-35 | écran |
| R-092 | Commercial | leases | route | UI | — | famille : TESTING §6 (cas « TVA incluse ») |  |
| R-093 | Tous | leases, units +§4 | route, unit.under_works, unit.works_requires_vacant | UI | MAQ | E-36 |  |
| R-094 | Habitation, Commercial | units | — | — | — | — | API, écran, test |
| R-095 | Tous | Calcul | — | UI | MAQ | E-37, TESTING(R) | API |
| R-096 | Tous | payments, receipt_cancellations, receipts +§4 | route | UI | MAQ | E-38 |  |
| R-097 | Tous | reminders | route | UI | — | TESTING(R) |  |

**Règles techniques sans règle métier source** : les gardes de `DATABASE_SCHEMA.md` §4 citent toutes une règle R-xxx ou une décision DP-xx, sauf le contrôle `payer_third_party_phone` (R-088 partielle) et la borne 1 à 999 999 999 (n° 35, décision ouverte, pas de R-xxx : R-001 n'en parle pas). Ajouter la borne à R-001 ou à une nouvelle règle (C2-006).

### Annexe 3 — Questions ouvertes consolidées (sans doublon)

**Bloque la roadmap**

1. Quels critères décident, à la fin du pilote, de continuer, corriger ou arrêter ? (`DECISIONS.md` n° 19, dossier §17 phase 4) — C2-001
2. Combien de personnes et d'heures par semaine pendant les 6 prochains mois ? Qui est la deuxième personne pour revues et restauration ? (n° 14 ; `ARCHITECTURE.md` Q2, `CODING_RULES.md` Q1, `TESTING_QA.md` Q1, `DEPLOYMENT.md` Q15) — C2-003
3. Premier pilote : agence ou propriétaire direct ? Combien de pilotes ? (n° 2) — C2-007
4. Budget mensuel d'infrastructure et budget de lancement ? (`DEPLOYMENT.md` Q18, dossier §17) — C2-023
5. Hébergement en Europe de l'Ouest acceptable ? Mesure de latence (prototype e) avant ? (n° 5 ; `ARCHITECTURE.md` Q12 ; `DEPLOYMENT.md` Q2) — C2-004
6. L'hébergeur retenu donne-t-il un rôle avec `BYPASSRLS` pour `pg_dump`, sinon on ne force pas la RLS ? (n° 46 ; `SECURITY.md` Q11 ; `DATABASE_SCHEMA.md` Q12) — C2-004
7. Lot de 32 décisions « avant la tranche verticale » (dont 20 techniques) : recommandations acceptées ? (annexe 4) — C2-005
8. Règles de T1 : jour d'échéance 29–31 en mois court ; premier mois partiel ; bail à date de début passée ; promesse « tenue » ; doublon de référence bloquant ou non ; taux de recouvrement ; année de numérotation (31/12 saisi le 02/01) ; avenant sur échéance générée ; téléphone partagé. (`ARCHITECTURE.md` Q3 à Q5 ; `DATABASE_SCHEMA.md` 11.1 n° 1, 6, 7, 11 ; `API_CONTRACTS.md` Q1, Q5, Q12, Q14 ; `DOMAINE.md` « Taux de recouvrement ») — C2-006
9. Hébergeur du code et de la CI (plateforme, compte d'organisation) ? (`ARCHITECTURE.md` Q13 ; n° 87) — C2-005
10. Les dossiers `film/`, `flyer/`, `infographie/`, `logo/`, `site/` et leurs `.venv` sortent-ils du dépôt de code ? (`ARCHITECTURE.md` Q13 ; `CODING_RULES.md` Q4 ; `DEPLOYMENT.md` Q14) — C2-005

**Bloque le premier pilote**

11. Charges encaissées sous un mandat à base « loyer » : reversées ou conservées ? (`DATABASE_SCHEMA.md` 11.1 n° 2) — C2-008
12. Net ou base de commission négatifs : refus ou autorisation ? (`DATABASE_SCHEMA.md` 11.1 n° 3 ; `ARCHITECTURE.md` Q7) — C2-008
13. Dépenses d'un compte rendu : par date de la dépense ou du paiement, statut « terminée » seul ? (`DATABASE_SCHEMA.md` 11.1 n° 4 ; `API_CONTRACTS.md` Q13) — C2-008
14. Reversement partiel ou en plusieurs fois ? (`DATABASE_SCHEMA.md` 11.1 n° 8) — C2-008
15. Mandat clôturé en cours de mois ; lots qui déclenchent R-075. (`DATABASE_SCHEMA.md` 11.1 n° 10, 12) — C2-008
16. Règle de reddition : date d'encaissement ou d'échéance ? (n° 10, R-071, H8) — C2-008
17. Remise à zéro d'une organisation en phase setup : acceptée ? (n° 25, 40 ; `ARCHITECTURE.md` Q6) — C2-009
18. Quittances de la double tenue : valeur pour le locataire ? L'agence continue-t-elle son carnet ? (`DEPLOYMENT.md` Q8 ; n° 36) — C2-010
19. Bascule en live : début de mois (zone rouge) ou milieu de mois avec premier compte rendu partiel ? (`DEPLOYMENT.md` Q7) — C2-010
20. Préfixe de quittance modifiable après la première émission de l'année ? (`DATABASE_SCHEMA.md` 11.1 n° 9) — C2-010
21. Mentions légales de la quittance ; orthographe et formulation du montant en lettres (« francs CFA ») ? (n° 12 ; `ARCHITECTURE.md` Q9 ; `TESTING_QA.md` Q5) — C2-015
22. Quittance rectificative sans effet sur le montant : règle, champs corrigeables, qui ? (`API_CONTRACTS.md` Q6) — C2-015
23. Format par défaut (A5), rendu simultané A4 + A5, impression depuis le PDF ou un aperçu HTML ? (`API_CONTRACTS.md` Q7, Q8 ; `TESTING_QA.md` Q4) — C2-015
24. Code de contrôle imprimé et page de vérification qui n'expire pas ? (n° 55 ; `SECURITY.md` Q8) — C2-015
25. TOTP du gérant obligatoire avant la bascule en live ? (n° 50 ; `SECURITY.md` Q2) — C2-013
26. Durées de session (30 min caissier, 60 min gérant) supportables au comptoir ? (n° 49 ; `SECURITY.md` Q3) — à tester avec une agence
27. Inscription publique fermée au pilote ; réinitialisation par lien généré. (n° 27, 51, 60 ; `SECURITY.md` Q1 ; `ARCHITECTURE.md` Q10 ; `API_CONTRACTS.md` Q16) — C2-028
28. Agence avec un seul gérant : récupération de compte par vérification HABEK acceptable ? (`SECURITY.md` Q15)
29. Le support peut-il lire les champs chiffrés (mention explicite dans l'accord) ? (`SECURITY.md` Q9)
30. Second fournisseur de sauvegardes : écriture seule et verrouillage d'objet ? Sinon copie hors ligne mensuelle. (`SECURITY.md` Q10 ; `DEPLOYMENT.md` Q10)
31. Chiffrement des sauvegardes : clé publique ou symétrique ? (n° 95 ; `DEPLOYMENT.md` Q9)
32. Restriction de la console par adresse IP ? (`SECURITY.md` Q13)
33. Responsable du traitement (agence ou HABEK) ; notification d'une fuite à une autorité ; durées de conservation. (`SECURITY.md` Q4, Q6, Q7 ; n° 56) — C2-012
34. Déclaration ou autorisation CIL avant le pilote ; hébergement hors du pays. (n° 5, H12 ; `SECURITY.md` Q5 ; `DEPLOYMENT.md` Q3) — C2-012
35. Départ d'une agence : conservation 10 ans ou purge à 30 jours ? (n° 94 ; `DEPLOYMENT.md` Q4) — C2-041
36. Jours et heures de couverture, personne de repli, numéro d'urgence. (`DEPLOYMENT.md` Q5 ; `TESTING_QA.md` Q2) — C2-043
37. Calendrier de livraison à confirmer avec une agence (samedi ? jours de pic ?). (`DEPLOYMENT.md` Q6)
38. Gouvernance des accès : titulaire légal des comptes, messagerie d'organisation, troisième détenteur des codes de secours, gestionnaire de mots de passe. (`DEPLOYMENT.md` Q1)
39. Rendu PDF isolé : conteneur séparé ou sous-processus restreint ? (`DEPLOYMENT.md` Q11 ; dépend du moteur, n° 7)
40. Page d'état : hébergement et mise à jour. (`DEPLOYMENT.md` Q12 ; n° 93)
41. Éditeur légal, nom du produit, domaine. (`CONTEXTE.md` « Éditeur » ; n° 13 ; `DEPLOYMENT.md` Q13) — C2-014
42. Qui rédige l'engagement de confidentialité, l'accord de pilote et la lettre d'intention ; exige-t-on la lettre d'intention ? — C2-011
43. Test d'intrusion externe avant le pilote : budget et prestataire. (n° 58 ; `SECURITY.md` Q12 ; `TESTING_QA.md` Q9 ; `DEPLOYMENT.md` Q17)
44. Séances de recette sur place : consentement des personnes observées. (`TESTING_QA.md` Q10)
45. Appareil Android d'entrée de gamme : budget, modèle. (n° 81 ; `TESTING_QA.md` Q3)
46. Fichiers réels des pilotes pour les tests d'import : jusqu'où, avec quel accord ? (n° 15 ; `TESTING_QA.md` Q8 ; `API_CONTRACTS.md` Q17)
47. Console HABEK : interface web ou ligne de commande au pilote ? — C2-033
48. Droits du collaborateur ; agent terrain « proposer » ; entrées de menu du caissier, de l'agent terrain, du comptable. (`API_CONTRACTS.md` Q9, Q10 ; `UI_UX_DESIGN.md`) — C2-030
49. Variables des modèles de relance (six proposées). (`ARCHITECTURE.md` Q16 ; `API_CONTRACTS.md` Q4) — C2-040
50. Un lot a-t-il un seul agent terrain ou plusieurs ? (`ARCHITECTURE.md` Q17)
51. Colocation : solidarité des colocataires, destinataire de la relance. (`ARCHITECTURE.md` Q18 ; `DOMAINE.md`)
52. Paliers de prix (Essentiel, Pro, Agence, Agence Plus) uniquement commerciaux ? Plafond de lots du plan Propriétaire. (`CONTEXTE.md`) — C2-029
53. Confirmation de DP-45 à DP-51. — C2-020
54. « Valides » dans E-20 : 197 ou 195 ? (`TESTING_QA.md` Q6 ; `API_CONTRACTS.md` Q11) — C2-046
55. Annonces publiques : paiement en ligne, envoi automatique WhatsApp, hors-connexion ; appel à l'action du site. — C2-024, C2-025

**Peut attendre**

56. Précision du taux de commission (deux décimales). (n° 29)
57. Vouvoiement (n° 17) ; thème sombre ; application installable (n° 22).
58. Dépôt de garantie : qui détient la caution, délai de restitution, plafond en commercial, affichage au relevé. (`DOMAINE.md`)
59. TVA et loyers variables en commercial (H7). (`DOMAINE.md`)
60. Durée légale du préavis (habitation, commercial). (`DOMAINE.md` R-090)
61. Format réel des SMS Orange Money et Moov Money pour F25. (`DOMAINE.md`)
62. Fiscalité des revenus fonciers. (`DOMAINE.md`)
63. Pièce d'identité du locataire : l'activer un jour ? (`SECURITY.md` Q14)
64. Message type à remettre aux locataires pour reconnaître la communication officielle. (`SECURITY.md` Q16)
65. Pourquoi ne pas réutiliser la pile de Scolo ? Qui valide les décisions ? (`DECISIONS.md` « Questions ouvertes »)
66. Persona A (agence « de taille moyenne ») à remplacer par une personne réelle. (`CONTEXTE.md`)
67. Frais d'agrégateur mobile money, tarifs SMS et WhatsApp (P2). (`CONTEXTE.md`)
68. Nom de la « Visite guidée » et de la « Démonstration guidée » (vocabulaire). (`UI_UX_DESIGN.md`)

### Annexe 4 — Décisions ouvertes consolidées (tableau `DECISIONS.md` n° 1 à 95)

**BLOQUE LA ROADMAP — portée, séquence, effort, équipe** (12)

| N° | Question | Échéance indiquée |
|---|---|---|
| 2 | Premier segment de pilote | Phase 0 |
| 5 | Hébergement et conformité | Phase 1 |
| 6 | Frontend | Phase 1 |
| 7 | Moteur de génération PDF | Phase 0 |
| 14 | Capacité de développement | Phase 1 |
| 19 | Stratégie : pilote réel sans enquête préalable | Avant tout code applicatif |
| 20 | Pile backend | Avant la tranche verticale |
| 21 | Exploitation de PostgreSQL | Avec la décision n° 5 |
| 23 | Dépôt et structure | Avant la tranche verticale |
| 46 | Sauvegarde logique sous RLS forcée | Avant de choisir l'hébergeur |
| 86 | Hébergement, grille de choix et coûts | Avec la n° 5 |
| 87 | Dépôt de code et CI | Avant la tranche verticale |

**BLOQUE LA ROADMAP — lot technique à valider en bloc avant la tranche verticale** (20)

| N° | Question | Échéance indiquée |
|---|---|---|
| 18 | Langue des identifiants de code | Phase 1 |
| 30 | Énumérations de la base | Avant la migration initiale |
| 31 | Identifiants de la base | Avant la migration initiale |
| 32 | Clés étrangères composites | Avant la migration initiale |
| 34 | Écriture seule à deux barrières | Avant la migration initiale |
| 35 | Bornes des montants | Avant la migration initiale |
| 41 | Chevauchement de baux avec les baux terminés | Avant la migration initiale |
| 61 | Source de vérité du contrat d'API | Avant la tranche verticale |
| 62 | Versionnement et évolution de l'API | Avant la tranche verticale |
| 64 | Verrouillage optimiste des fiches | Avant la migration initiale |
| 67 | Casse des champs JSON | Avant la tranche verticale |
| 69 | Gestion des erreurs et interdits Rust | Avant la tranche verticale |
| 70 | Bibliothèque de dates | Avant la tranche verticale |
| 73 | Branches et commits | Avant la tranche verticale |
| 74 | Revue et taille des changements | Avant la tranche verticale |
| 75 | Outillage de la règle « jamais `&PgPool` hors `platform` » | Avant la tranche verticale |
| 76 | Détection de secrets | Avant la tranche verticale |
| 77 | Licences des dépendances | Avant la tranche verticale |
| 79 | Tests de propriétés | Avant la tranche verticale |
| 80 | Base des tests d'intégration | Avant la tranche verticale |

**BLOQUE LE PREMIER PILOTE** (32)

| N° | Question | Échéance indiquée |
|---|---|---|
| 3 | Grille de prix à tester | Phase 0 |
| 10 | Règle de reddition de comptes | Phase 0 |
| 11 | Modèle d'onboarding | Phase 3 |
| 12 | Validation juridique | Phase 0–1 |
| 13 | Nom et marque | Avant publication |
| 15 | Données de test | Phase 2 |
| 24 | Objectifs de sauvegarde | Avant le premier pilote |
| 25 | Remise à zéro d'une organisation pilote | Avant le premier pilote |
| 36 | Numérotation : séries d'essai et officielle, année | Avant le premier pilote |
| 37 | Contrôles différés des affectations | Avant le premier pilote |
| 38 | Accès support : politique d'isolation dédiée | Avant le premier pilote |
| 40 | Annulation d'import et purge par fonctions | Avant le premier pilote |
| 42 | Avenants : garde mécanique | Avant la tranche 2 |
| 43 | Téléphone du locataire unique parmi les fiches non archivées | Avant la tranche 2 |
| 47 | Procédure d'exception pour réinjecter des données | Avant le premier pilote |
| 50 | Double authentification | Avant le premier pilote |
| 51 | Inscription publique | Avant le premier pilote |
| 52 | Chiffrement des champs sensibles | Avant le premier pilote |
| 53 | Sauvegardes : droits et clé | Avant le premier pilote |
| 56 | Durées de conservation | Avant le premier pilote |
| 57 | Engagement de confidentialité des agences pilotes | Avant le premier pilote |
| 58 | Tests de sécurité | Avant le premier pilote |
| 83 | Tests visuels du PDF | Après le prototype PDF |
| 84 | Gravités et délais de correction | Avant le premier pilote |
| 85 | Protocole de recette avec les agences | Avant le premier pilote |
| 89 | Supervision, alertes et canal | Avant le premier pilote |
| 90 | Noms d'hôte, domaine et DNS | Avant le premier pilote |
| 91 | Calendrier de livraison pendant le pilote | Avant le premier pilote |
| 92 | Secrets de production | Avant le premier pilote |
| 93 | Page d'état pour les clients | Avant le premier pilote |
| 94 | Départ d'une agence : export, purge et preuve | Avant le premier départ |
| 95 | Chiffrement des sauvegardes | Avant le premier pilote |

**PEUT ATTENDRE — à valider au début de la tranche concernée** (26)

| N° | Question | Échéance indiquée |
|---|---|---|
| 17 | Vouvoiement dans l'interface | Avant la diffusion publique des vidéos |
| 22 | Application installable (PWA) | Tranche 1 |
| 26 | Organisation de démonstration | Tranche 1 |
| 27 | E-mail et SMS au premier pilote | Tranche 1 |
| 28 | Observation des usages | Tranche verticale |
| 29 | Précision du taux de commission | Tranche 5 |
| 33 | Soldes | Tranche 3 |
| 39 | Lecture publique par jeton | Tranche 3 |
| 44 | Comptes rendus : aperçu calculé, version écrite à la validation | Tranche 5 |
| 45 | Rôles du plan Propriétaire | Tranche 1 |
| 48 | Mots de passe | Tranche 1 |
| 49 | Sessions côté serveur : cookie et durées | Tranche 1 |
| 54 | Accès du personnel HABEK aux données | Tranche 6 |
| 55 | Code de contrôle de la quittance | Tranche 3 |
| 59 | En-têtes de sécurité et journaux | Tranche 1 |
| 60 | Récupération de compte | Tranche 1 |
| 63 | Pagination | Tranche 1 |
| 65 | Opérations longues (import, PDF, export) | Tranche 3 |
| 66 | Erreurs de l'API et textes affichés | Tranche verticale |
| 68 | Téléchargement des PDF et des fichiers | Tranche 3 |
| 71 | Données du front | Tranche verticale |
| 72 | Outils du front | Tranche verticale |
| 78 | Tests de bout en bout et accessibilité automatique | Tranche verticale |
| 81 | Profil de test « Android d'entrée de gamme, réseau lent » | Tranche verticale |
| 82 | Chaîne d'intégration continue | Tranche verticale |
| 88 | Orchestration et mise en production | Tranche verticale |

**Prises (5)** : n° 1, 4, 8, 9, 16. Total : 95 décisions.

## F. Décisions de Mimis (30/10/2026)

Message de Mimis : « C2-004 > on utilisera Neon pour la base de données / C2-014 > HABEK Technology / C2-003 > un développeur 25 h/semaine / Le reste applique tes recommandations ».

| Constat | Réponse | Entrée enregistrée |
|---|---|---|
| C2-003 | Un développeur, environ 25 heures par semaine (réponse propre) | DP-54 |
| C2-004 | PostgreSQL managé chez Neon (réponse propre ; la sauvegarde logique reste conditionnelle, point à vérifier au prototype b) | DP-55 |
| C2-014 | Éditeur : HABEK Technology Company (réponse propre ; le nom du produit et le domaine restent ouverts) | DP-63 |
| Les 53 autres constats | Recommandation de l'audit appliquée (validation en bloc) | DP-52, DP-53, DP-56 à DP-62, DP-64 à DP-68 et les corrections de la section G |

Les cases « Décision de Mimis » de la section B sont cochées en conséquence.

## G. Corrections appliquées (phase B, 30/10/2026)

Règles suivies : décision d'abord (`DECISIONS.md`), puis le document propriétaire, puis les documents qui en dépendent ; le dossier de cadrage `.docx` n'est pas modifié (les écarts sont dans `DECISIONS.md`, section « Écarts assumés ») ; aucun fichier nouveau, sauf `TRACABILITE.md` ; `ROADMAP.md` n'est pas écrit. Les sauvegardes des fichiers d'avant la phase B sont dans le dossier de travail de l'auditeur. Statut : **Appliqué**, **Partiel** (le reste est en section H) ou **Ouvert** (rien d'applicable sans information de Mimis).

| Constat | Statut | Fichiers modifiés | Ce qui a changé |
|---|---|---|---|
| C2-001 | Appliqué | `DECISIONS.md`, `CONTEXTE.md` | DP-52 (pas d'enquête avant le développement, pilote réel) et DP-53 (critères du pilote) ; DP-07, DP-10, DP-34, DP-35 passées à « Remplacée » ; section « Critères du pilote » à la place du Go/No-Go dans `CONTEXTE.md`. |
| C2-002 | Appliqué | `CLAUDE.md` | Phase actuelle réécrite (construction de T1 puis T2, pilote réel) ; mention des enquêtes retirée ; fichier réécrit en 78 lignes. |
| C2-003 | Appliqué | `DECISIONS.md`, `ARCHITECTURE.md`, `CODING_RULES.md`, `DEPLOYMENT.md`, `SECURITY.md`, `DATABASE_SCHEMA.md`, `API_CONTRACTS.md`, `CLAUDE.md` | DP-54 : un développeur à 25 h/semaine. « Deux développeurs/deux personnes » remplacé partout par « le développeur et le porteur du projet » ; revue de code par relecture différée de 24 h plus second avis externe sur les chemins sensibles (`CODING_RULES.md` §8.3) ; tests de restauration par le développeur observé ; durées de la tranche verticale renvoyées à la roadmap. |
| C2-004 | Partiel | `DECISIONS.md`, `ARCHITECTURE.md`, `DATABASE_SCHEMA.md`, `DEPLOYMENT.md`, `SECURITY.md`, `TESTING_QA.md` | DP-55 : base chez Neon ; n° 21 prise, n° 46 sous condition, n° 5 et 86 en partie. Points propres à Neon notés [À vérifier] : rôles de la console avec `BYPASSRLS` (donc `immo_app` créé en SQL), historique du plan, région, veille. Serveur, stockage objet et second fournisseur restent ouverts. |
| C2-005 | Appliqué | `DECISIONS.md`, `DATABASE_SCHEMA.md`, `API_CONTRACTS.md`, `CODING_RULES.md`, `TESTING_QA.md` | DP-56 : 32 décisions techniques « avant la tranche verticale » prises en bloc (n° 6, 20, 23, 30–32, 34, 35, 41, 61, 62, 64, 67, 69–77, 79, 80, 87) ; les documents indiquent leur nouveau statut. |
| C2-006 | Appliqué | `DECISIONS.md`, `DOMAINE.md`, `ARCHITECTURE.md`, `API_CONTRACTS.md`, `DATABASE_SCHEMA.md`, `TESTING_QA.md` | DP-58 ; R-098 à R-104 ajoutées, R-017 précisée, exemples E-39 à E-44 ; questions d'`ARCHITECTURE.md`, d'`API_CONTRACTS.md`, de `DATABASE_SCHEMA.md` et de `TESTING_QA.md` closes. Marqueur [à confirmer avec l'agence pilote à la découverte]. |
| C2-007 | Appliqué | `DECISIONS.md`, `ARCHITECTURE.md` | DP-57 : pilote principal = une agence, plus un ou deux propriétaires ; prix testés pendant le pilote ; échéances des n° 2, 3, 7, 10, 12, 15 recalées ; ordre de T2 (mandats puis import) assumé comme écart au dossier §18. |
| C2-008 | Appliqué | `DECISIONS.md`, `DOMAINE.md`, `DATABASE_SCHEMA.md`, `API_CONTRACTS.md` | DP-59 ; R-071 confirmée par défaut, R-105 à R-109, exemples E-45 et E-46. |
| C2-009 | Appliqué | `DECISIONS.md`, `DOMAINE.md`, `SECURITY.md`, `DATABASE_SCHEMA.md`, `ARCHITECTURE.md` | DP-60 : exceptions à R-005 et R-007 (annulation d'import, remise à zéro en `setup`) ; R-110 (cycle de vie), E-47. |
| C2-010 | Appliqué | `DECISIONS.md`, `DOMAINE.md`, `ARCHITECTURE.md`, `DATABASE_SCHEMA.md`, `UI_UX_DESIGN.md`, `API_CONTRACTS.md` | DP-61 : séries `trial` et `official`, année d'émission ; R-050 reformulée, E-48 ; compteur par (organisation, série, année) dans `ARCHITECTURE.md` §4.4 ; filigrane « ESSAI » dans la quittance. |
| C2-011 | Ouvert | `CLAUDE.md`, `TESTING_QA.md` | Les documents du pilote ne sont pas rédigés (aucun fichier nouveau autorisé). `CLAUDE.md` nomme maintenant ceux à créer dans `contenus/` (sans le guide d'entretien) ; `TESTING_QA.md` PP-38 les exige. |
| C2-012 | Partiel | `DECISIONS.md`, `TESTING_QA.md`, `DEPLOYMENT.md` | DP-62 : avis juridique ponctuel avant le premier pilote sur cinq sujets, règle de repli (risque accepté par écrit). Personne chargée et date : à fixer par Mimis. |
| C2-013 | Appliqué | `DECISIONS.md`, `ARCHITECTURE.md`, `SECURITY.md`, `UI_UX_DESIGN.md`, `TESTING_QA.md`, `DEPLOYMENT.md` | DP-68 : TOTP du gérant obligatoire avant `live` ; inscription publique fermée ; réinitialisation par lien ; `ARCHITECTURE.md` §3.8, §9.3, §13.2 alignés ; test de refus de la bascule sans TOTP (PP-31). |
| C2-014 | Partiel | `DECISIONS.md`, `CONTEXTE.md`, `CLAUDE.md` | DP-63 : éditeur HABEK Technology Company. Nom du produit et domaine : décision n° 13, toujours ouverte. |
| C2-015 | Appliqué | `DECISIONS.md`, `DOMAINE.md`, `UI_UX_DESIGN.md`, `API_CONTRACTS.md`, `TESTING_QA.md` | DP-64 : « francs CFA », orthographe traditionnelle, A5 par défaut avec les deux formats rendus, rectificative sans effet et code de contrôle hors pilote. Les mentions légales restent [À VÉRIFIER AVEC UN JURISTE]. |
| C2-016 | Appliqué | `CONTEXTE.md` | Statut, hypothèses (« Pilote : … »), critères du pilote, questions closes. |
| C2-017 | Appliqué | `DECISIONS.md`, `UI_UX_DESIGN.md` | Priorité immédiate périmée retirée, doublon PWA retiré, questions closes retirées, numérotation à DP-69 ; n° 18 prise (DP-67) ; logo livré noté dans DP-44 et dans les questions d'`UI_UX_DESIGN.md`. |
| C2-018 | Appliqué | `MAQUETTE.md`, `maquette/README.md` | Objectif = démonstrations aux agences pilotes potentielles ; « Usage en entretien » devenu « Usage en démonstration » sans échelle ni seuil ; mentions de l'enquête et des Go/No-Go retirées. |
| C2-019 | Appliqué | `DOMAINE.md`, `UI_UX_DESIGN.md`, `MAQUETTE.md`, `TESTING_QA.md`, `DEPLOYMENT.md` | Marqueur [à confirmer avec l'agence pilote à la découverte] ; « après le démarrage du pilote » ; « décision écrite de mise en production du pilote » à la place de « Go/No-Go » dans `DEPLOYMENT.md` §10 ; séances de recette dans `TESTING_QA.md` §13.1. |
| C2-020 | Appliqué | `DECISIONS.md` | DP-45 à DP-51 : « Prise (confirmée par Mimis le 30/10/2026) ». |
| C2-021 | Appliqué | `CLAUDE.md`, `DATABASE_SCHEMA.md`, `ARCHITECTURE.md`, `API_CONTRACTS.md`, `CODING_RULES.md`, `TESTING_QA.md`, `DEPLOYMENT.md`, `SECURITY.md`, `maquette/NOTES.md` | Table « propriétaire par sujet » (12 lignes) dans `CLAUDE.md` ; les autres documents renvoient à `CLAUDE.md` à la place de leur ancienne hiérarchie. |
| C2-022 | Appliqué | `SECURITY.md`, `TESTING_QA.md`, `DEPLOYMENT.md` | Échelle unique S1 à S4 ; « quittance sans PDF depuis plus de 10 minutes » = S2 dans les trois documents. |
| C2-023 | Partiel | `DEPLOYMENT.md`, `ARCHITECTURE.md`, `DECISIONS.md` | Totaux recalculés (essai 16–31 €, production 26–80 €, ensemble 42–111 €, soit 27 600 à 72 800 FCFA par mois) ; coûts ponctuels listés (« à chiffrer ») ; `ARCHITECTURE.md` et n° 86 renvoient à `DEPLOYMENT.md` §1.3. Budget disponible : question de Mimis restée ouverte. |
| C2-024 | Partiel | `site/src/index.src.html`, `site/immo-suite/index.html`, `site/outils/verifier.py`, `site/NOTES.md` | Option (a) : ligne « Prévu plus tard » et phrase de la FAQ retirées. L'archive `site/immo-suite.zip` n'est pas régénérée. |
| C2-025 | Partiel | `site/` (index, `config.json`, `js/immo-suite.js`, `README.md`, `NOTES.md`, `outils/verifier.py`), `flyer/src/verso.html`, `flyer/build/*.html`, `flyer/scripts/controle.py`, `film/VOIX_OFF.md` | Appel à l'action « Devenir agence pilote : un premier échange de 20 minutes » (site) et « Un premier échange de 20 minutes ? » (prospectus) ; sondage conservé ; carte de fin du film : appel à devenir agence pilote. PDF du prospectus, archive du site et film non régénérés. |
| C2-026 | Appliqué | `ARCHITECTURE.md` | A1 à A14 de `DATABASE_SCHEMA.md` §11.2 repris : `pgcrypto` retiré, 25 tables en écriture seule, exclusion étendue aux baux terminés, séries de numérotation, `lease_notices`/`security_deposits` corrigés, tables globales, bornes, `route` des clés d'idempotence, `immo_readonly`, pooler, rôle de sauvegarde sous Neon. |
| C2-027 | Appliqué | `DATABASE_SCHEMA.md`, `API_CONTRACTS.md` | `row_version` ajouté aux 11 tables modifiables, `sessions.reauth_until` ajouté ; questions 2 et 3 d'`API_CONTRACTS.md` closes. |
| C2-028 | Appliqué | `UI_UX_DESIGN.md` | S01 devient « Connexion, invitation et réinitialisation » (quatre modes, pas d'inscription publique) ; microcopy « lien de réinitialisation ». |
| C2-029 | Appliqué | `DECISIONS.md`, `CONTEXTE.md` | DP-65 : deux plans, paliers commerciaux sans effet dans le logiciel ; questions « deux plans ou quatre offres » et « plafond » closes. |
| C2-030 | Appliqué | `DECISIONS.md`, `CONTEXTE.md`, `SECURITY.md`, `UI_UX_DESIGN.md` | DP-66 : compléments à la matrice (collaborateur, agent terrain, entrées « ? » masquées) ; colonne « Collaborateur » ajoutée au tableau des menus. |
| C2-031 | Appliqué | `API_CONTRACTS.md`, `TESTING_QA.md`, `TRACABILITE.md` | Paragraphe « T3 : à spécifier avant chaque lot » (F17, F22) ; lignes de T3 dans `TESTING_QA.md` §2.1 ; les tests eux-mêmes restent à écrire avec chaque règle. |
| C2-032 | Appliqué | `TRACABILITE.md`, `TESTING_QA.md` | Matrice règle → test (R-001 à R-110) générée dans `TRACABILITE.md` ; `TESTING_QA.md` §2.1 y renvoie au lieu de la recopier (écart à la recommandation, pour éviter un doublon). 36 règles restent sans test cité (section H). |
| C2-033 | Ouvert | `API_CONTRACTS.md`, `UI_UX_DESIGN.md` | `specs/F40-console.md` ne peut pas être créé (fichier nouveau interdit). La console du pilote peut être une ligne de commande (`API_CONTRACTS.md` §7.18), noté dans l'UI. |
| C2-034 | Partiel | `DECISIONS.md`, `ARCHITECTURE.md` | Prototype PDF = 2 à 3 jours partout (n° 7, DP-11, §7). Mise en roadmap des prototypes (b), (c), (a), (e), (d) : à faire dans `ROADMAP.md`. |
| C2-035 | Appliqué | `TESTING_QA.md`, `ARCHITECTURE.md`, `SECURITY.md`, `DEPLOYMENT.md`, `CLAUDE.md` | `TESTING_QA.md` §12 devient la seule liste (PP-01 à PP-38, six cases ajoutées) ; `ARCHITECTURE.md` §9.3, `SECURITY.md` §10 et la liste de `DEPLOYMENT.md` §10 (qui devient la liste de mise en production) y renvoient. |
| C2-036 | Appliqué | `CLAUDE.md` | ARCHITECTURE sans « (à créer) » ; vérificateur réel `node maquette/scripts/verifier-donnees.js` ; pile de référence ; variables d'environnement alignées sur `DEPLOYMENT.md` ; structure du dépôt complétée. |
| C2-037 | Partiel | `CLAUDE.md`, `DEPLOYMENT.md`, `ARCHITECTURE.md`, `SECURITY.md`, `TESTING_QA.md` | Légende dans `CLAUDE.md` seul ; hiérarchie, calendrier, coûts, listes « prêt pour un pilote » dédupliqués. Alertes (`ARCHITECTURE.md` §10.3 contre `DEPLOYMENT.md` §7.3), limitation du débit et budgets de performance non encore réduits à des renvois. |
| C2-038 | Appliqué | `DATABASE_SCHEMA.md` | « PostgreSQL réel » remplacé par PGlite ; §9.1 dit que le script et les 44 tests seront recréés dans `migrations/` et `tests/`. |
| C2-039 | Appliqué | `DECISIONS.md` | Section « Écarts assumés par rapport au dossier de cadrage » (11 lignes) et règle de mise à jour. |
| C2-040 | Appliqué | `DOMAINE.md`, `UI_UX_DESIGN.md`, `API_CONTRACTS.md`, `ARCHITECTURE.md`, `DATABASE_SCHEMA.md` | Six variables `{nom}`, `{montant}`, `{mois}`, `{lot}`, `{date}`, `{lien_quittance}` partout ; questions closes. |
| C2-041 | Partiel | `SECURITY.md`, `DATABASE_SCHEMA.md`, `DEPLOYMENT.md` | §6.3 : « durée du contrat, puis jusqu'à la purge sur instruction écrite » ; fonction `purge_org_closed` prévue avant le premier départ. La règle définitive attend l'avis juridique (DP-62). |
| C2-042 | Appliqué | `SECURITY.md` | §6.2 : deux suppressions possibles, tracées dans `platform_events` (DP-60). |
| C2-043 | Ouvert | `TESTING_QA.md`, `DEPLOYMENT.md` | Heures de couverture, personne de repli et numéro d'urgence : données de Mimis manquantes. Case PP-37 ajoutée ; décision n° 84 reste ouverte. |
| C2-044 | Appliqué | `DEPLOYMENT.md`, `ARCHITECTURE.md`, `TESTING_QA.md` | Calendrier de livraison : source unique dans `DEPLOYMENT.md` §4 ; les deux autres renvoient. |
| C2-045 | Appliqué | `UI_UX_DESIGN.md`, `MAQUETTE.md` | Filigranes « ESSAI » et « DÉMONSTRATION — sans valeur » ; pièce d'identité désactivée au pilote ; 2FA du gérant en T1 ; paire de contraste du repère « À venir » alignée sur `tokens.css` (brand-700 sur accent-100, 14,5:1). |
| C2-046 | Appliqué | `DOMAINE.md`, `TESTING_QA.md`, `API_CONTRACTS.md` | E-20 : « 197 lignes créables (valides ou avec avertissement) » et 3 erreurs ; questions closes. |
| C2-047 | Appliqué | les 12 documents | « `DECISIONS.md` (état au 30/10/2026) » sans plage ; versions des documents liés à jour ; en-tête (Statut, Version, Date, Dépend de, Alimente) ajouté à `DECISIONS.md`, `CONTEXTE.md`, `DOMAINE.md`, `UI_UX_DESIGN.md`, `MAQUETTE.md`, `CLAUDE.md`. |
| C2-048 | Appliqué | `ARCHITECTURE.md`, `DATABASE_SCHEMA.md`, `CODING_RULES.md`, `DEPLOYMENT.md`, `maquette/NOTES.md` | « La consigne » remplacée par la décision qu'elle représentait (DP-52, DP-54, DP-67) ou supprimée. |
| C2-049 | Appliqué | `CONTEXTE.md`, `DOMAINE.md`, `SECURITY.md`, `DEPLOYMENT.md`, `TESTING_QA.md`, `UI_UX_DESIGN.md` | Une seule forme : [À VÉRIFIER AVEC UN JURISTE] ; définie dans la légende de `CLAUDE.md`. |
| C2-050 | Appliqué | `MAQUETTE.md`, `DECISIONS.md`, `UI_UX_DESIGN.md` | Maquette réalisée, Node 24 présent, structure réelle, jour 16, 94 paiements et 93 quittances, logo livré ; DP-44 corrigée. |
| C2-051 | Appliqué | `API_CONTRACTS.md` | Préfixe `/api/` fixé (sauf `/health`, `/ready`, `/s/…`, `/v/…`, `/console/…`) ; les tables de §7 l'omettent pour la lisibilité, dit explicitement en §7.1 ; question 15 close. |
| C2-052 | Appliqué | `CODING_RULES.md` | §6 : 150 lignes hors chemins sensibles, cible générale de 400 lignes (§8.2). |
| C2-053 | Appliqué | `SECURITY.md` | §7.4 : 200 Ko à l'envoi, réduit à 30 Ko pour l'affichage. |
| C2-054 | Appliqué | `AUDIT_COHERENCE_2.md`, `ARCHITECTURE.md` | `AUDIT_COHERENCE.md` non modifié ; ligne « Audit précédent » présente en tête ; question 15 d'`ARCHITECTURE.md` fermée. |
| C2-055 | Appliqué | `MAQUETTE.md`, `CONTEXTE.md`, `maquette/js/ecrans/connexion.js`, `site/immo-suite/maquette/js/ecrans/connexion.js` | « En quelques secondes » ; cible de 30 secondes présentée comme cible mesurée au pilote (PP-25) ; promesse chiffrée retirée de l'écran de présentation de la maquette. |
| C2-056 | Appliqué | `DECISIONS.md`, `ARCHITECTURE.md` | DP-57 : ordre de T2 (mandats puis import) avec import par ligne de commande dès la tranche 2 ; ajouté aux écarts assumés. |

Bilan : 44 constats appliqués, 9 partiellement, 3 restés ouverts, sur 56.

**Écarts à la lettre des recommandations.** (1) Matrice règle → test : un renvoi vers `TRACABILITE.md` remplace le tableau de 97 lignes que l'audit proposait d'ajouter dans `TESTING_QA.md`, pour n'avoir qu'une source. (2) `DEPLOYMENT.md` §10 est conservé comme liste de mise en production (variante proposée par l'audit pour C2-035), réduite à ses points d'exploitation. (3) `DECISIONS.md` : DP-52 à DP-68 prennent les numéros annoncés en section C, avec DP-56 pour le lot technique et DP-67 pour la langue des identifiants ; DP-54 à DP-55 et DP-63 enregistrent les réponses propres de Mimis.

## H. Constats restés ouverts

**Ce que Mimis doit décider ou fournir avant la roadmap**

1. Heures de couverture du pilote (jours, heures, samedi ?), personne de repli et numéro d'urgence (C2-043, décision n° 84).
2. Qui rédige les documents du pilote (engagement de confidentialité, accord de pilote, bons usages, certificat de destruction, plan d'incident, guide d'accueil, lettre d'intention) et pour quelle date (C2-011).
3. Avis juridique : qui, pour quelle date, sur les cinq sujets de DP-62 ; sinon, accepter le risque par écrit (C2-012, C2-041, C2-015).
4. Budget mensuel d'infrastructure et budget de lancement (C2-023, `DEPLOYMENT.md` question 18).
5. Nom du produit, domaine et vérification de disponibilité (C2-014, décision n° 13).
6. Console du pilote : ligne de commande seulement, ou interface web ? Spécification `specs/F40-console.md` à écrire (C2-033).
7. Personne à désigner pour le second avis externe sur les chemins sensibles et pour l'auto-test d'intrusion (DP-54, `CODING_RULES.md` question 9).
8. Plan Neon (fenêtre d'historique, prix) et choix du serveur applicatif, du stockage objet et du second fournisseur de sauvegardes (C2-004, décisions n° 5 et 86).

**Constats appliqués en partie**

- C2-004 : `pg_dump` sous RLS forcée sur Neon, région, veille et historique du plan à vérifier au prototype (b) [À vérifier].
- C2-024 et C2-025 : sources du site, du prospectus et de la voix off corrigées, mais l'archive `site/immo-suite.zip`, les PDF du prospectus et le film (audio et vidéo) doivent être régénérés avec leurs outils (Python, navigateur, synthèse vocale) avant toute diffusion.
- C2-034 : les prototypes (b), (c), (a), (e), (d) sont décrits ; leur ordre et leurs dix jours-personne sont à inscrire dans `ROADMAP.md`.
- C2-037 : alertes (`ARCHITECTURE.md` §10.3 / `DEPLOYMENT.md` §7.3), limitation du débit et budgets de performance encore recopiés dans plusieurs documents.
- C2-041 : règle de conservation et de purge à fixer après l'avis juridique ; fonction `purge_org_closed` à créer avant le premier départ.

**Restes de la matrice de traçabilité** (`TRACABILITE.md` section 3)

- Fonctions de T3 sans route servie ni table créée : F17 (agenda), F22 (notifications), F20 (dépôt de garantie). Tests à écrire avec les règles : F17, F18, F20, F22, F25.
- Règles sans test cité ni famille : voir `TRACABILITE.md` section 3 (dont R-100, R-106, R-107, R-109 « à écrire » avec leurs cas d'or de T1 et T2).
- Règles absentes de l'interface et de la maquette : surtout les règles de calcul récentes (R-098 à R-110) ; l'interface les applique sans les afficher.

**Questions ouvertes de fond** (toutes marquées dans leur document) : valeur des quittances d'essai pour le locataire et date de bascule par rapport au début de mois (`DEPLOYMENT.md` questions 7 et 8) ; préfixe de quittance modifiable (`DATABASE_SCHEMA.md` 11.1 n° 9) ; journée de caisse (n° 5, T3) ; précision du taux de commission (n° 29) ; organisation de démonstration en production (n° 26) ; hébergeur du code et de la CI (question 13) ; agent terrain à un ou plusieurs lots ; colocation ; impression de la quittance à partir du PDF ou d'un aperçu HTML (attend le prototype (a)).

**Hypothèses de règles à confirmer avec l'agence pilote à la découverte** : R-098 à R-109 (DP-58, DP-59), R-071 (date d'encaissement) et les valeurs de DP-53.

## I. Vérifications après correction

Refaites par script le 30/10/2026 sur les documents corrigés.

| # | Vérification | Résultat |
|---|---|---|
| 1 | Identifiants F-xx | Réussi : 39 définis dans le dossier (F38 absent, noté par CONTEXTE) ; références hors F01–F40 : aucune |
| 2 | Identifiants R-xxx | Réussi : 110 définis, R-001 à R-110 ; manquants aucun ; doublons aucun ; références orphelines aucune |
| 3 | Identifiants E-xx | Réussi : 49 définis (E-01 à E-48 et E-13b) ; manquants aucun ; références orphelines aucune |
| 4 | Décisions DP-xx | Réussi : 68 entrées DP-01 à DP-68 ; manquantes aucune ; doublons aucun ; références orphelines aucune (DP-69 = prochain numéro) |
| 5 | Décisions n° 1 à 95 | Réussi : 95 lignes, continues, sans doublon : True |
| 6 | Écrans S-xx | Réussi : 26 définis ; références hors catalogue : aucune |
| 7 | Cases PP-xx de TESTING_QA §12 | Réussi : 38 cases PP-01 à PP-38 ; références orphelines aucune |
| 8 | Tables de DATABASE_SCHEMA §3 | Réussi : 63 fiches de tables (63 annoncées) |
| 9 | row_version et reauth_until (DP-56) | Réussi : row_version présent dans 11 tables sur 11 ; reauth_until dans sessions : True |
| 10 | Tables en écriture seule (§4.2) | Réussi : 25 tables listées en §4.2 (25 annoncées dans les documents) |
| 11 | CLAUDE.md sous 100 lignes | Réussi : 78 lignes |
| 12 | En-têtes (Statut, Version, Dépend de) | Réussi : manquants : aucun |
| 13 | Plages et versions périmées | Réussi : 0 occurrence(s) |
| 14 | Hiérarchie : une seule (CLAUDE.md) | Réussi : ancienne formule restante dans : aucun document |
| 15 | Termes de l’ancienne stratégie hors DECISIONS.md | Réussi : 1 occurrence(s) : CONTEXTE: "s du pilote\n\nIl n'y a plus de Go/No-Go avant le développement (DP-52" |
| 16 | Coûts d’hébergement (DEPLOYMENT §1.3) | Réussi : essai 16–31 €, production 26–80 €, ensemble 42–111 € = 27600–72800 FCFA ; texte conforme : True |
| 17 | Dossier de cadrage non modifié | Réussi : ['AUDIT_COHERENCE.docx', 'Immo-Suite_Dossier-de-cadrage.docx'] dernière modification antérieure au 30/10/2026 |
| 18 | Renvois de section (`DOC.md` §x.y) vers un titre existant | Réussi : 428 renvois contrôlés, 0 invalide ; renvois ajoutés à la main aux sections 1.3, 4, 5.2, 6.9, 7.18, 7.19, 8.3, 9.1, 12 et d'autres : tous trouvés |
| 19 | Fichiers cités et absents du disque | `ROADMAP.md` (interdit d'écriture dans cette phase), `api/CHANGELOG.md` (futur), `specs/F40-console.md` (à créer), `tests/fixtures/golden/E-xx.json` (modèle de nom). `GUIDE_ENTRETIEN.md`, `LETTRE_INTENTION.md`, `donnees/verifier.html`, `donnees/verifier.js` ne sont plus cités |
| 20 | Calculs de référence (40 : retards, lots, conversions, durées) | Réussi : 0 écart sur 40 |
| 21 | Contrastes de `UI_UX_DESIGN.md` (formule WCAG) | Réussi : tous à ±0,1 près ; la paire du repère « À venir » (brand-700 sur accent-100, 14,45) est maintenant la même dans `UI_UX_DESIGN.md`, `MAQUETTE.md` et `tokens.css` |
| 22 | `node maquette/scripts/verifier-donnees.js` | Réussi : 28 contrôles sur 28 |
| 23 | Matrices `TRACABILITE.md` | Générées : 40 fonctionnalités et 110 règles ; écarts restants listés en section 3 du fichier |

