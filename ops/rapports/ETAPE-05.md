# Étape 05 — Rapport de tests et de démonstration (04/11/2026)

Étape : « Biens, lots, locataires et fiches mandants » (`roadmap/etape-05-biens-lots-locataires.md`). Fiches `specs/F04-biens-lots.md`, `specs/F06-fiche-locataire.md` et `specs/F05-mandants.md` **validées par Mimis le 03/11/2026** (« go la suite » : recommandations acceptées, DP-92) ; DP-93 (écarts d'implémentation, **à relire**).

**Verdict : les 8 critères sont vérifiés sur le poste, avec trois réserves** (voir « Écarts et points à décider ») : le budget de 150 ms de la recherche n'est pas mesuré ; rien n'a tourné sur l'essai ni la production (inexistants, dette de l'étape 01) ; la garde `organizations_defaults()` modifiée attend le second avis C-19.

## Résultats réels (aucun chiffre estimé)

| Famille | Résultat | Commande |
|---|---|---|
| Rust : domaine, application, serveur, base | **295 réussis, 0 échec** (270 à l'étape 04) | `cargo test --workspace --locked --jobs 2` (base jetable du port 5435) |
| Rust : clippy `-D warnings`, `cargo fmt` | verts | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| Front : typage, ESLint | verts | `npm run typecheck`, `npm run lint` |
| Front : tests unitaires et de composants | **60 réussis** (51 d'avant + 9 de l'étape) | `npm test --prefix web` |
| Front : poids (brotli) | JS 78,8 Ko / 150, CSS 2,4 Ko / 30, total 81,4 Ko / 200 (JS +5,6 Ko) | `npm run size --prefix web` |
| Navigateur sur la **pile réelle**, 390 et 1440 px | **14 réussis**, 2 ignorés (expiration d'inactivité, `-ShortIdle`) ; dont le parcours de démonstration de l'étape à chaque largeur | `powershell -File ops/dev/real-stack.ps1 -Port 5435` |

Tests ajoutés : `crates/domain/tests/catalog.rs` (11 : téléphone R-018 dont deux propriétés, référence R-011, champs selon l'usage R-012, loyer de référence P16 en propriété, surface décimale) ; `tests/db/tests/step05_catalog.rs` (11 : critères 1 à 8, fusion, mandants, recherche, isolation A4 et ISO-3) ; 3 tests unitaires du module `catalog` ; `web/src/Step05.test.tsx` (8) ; `web/e2e-real/etape-05.spec.ts` (1 parcours, deux largeurs). La matrice A3 passe de 23 à 46 routes à droit (322 couples, 0 écart). Deux jeux d'essai de l'étape 03 créent désormais de vrais lots : la nouvelle clé `unit_assignments → units` refusait leurs affectations fictives (comportement voulu).

## Critères d'acceptation (étape 05, §6)

| # | Critère | État | Preuve |
|---|---|---|---|
| 1 | Maison seule : exactement 1 bien et 1 lot ; deux lots de même référence refusés | **vérifié** | `c1_…` : 1 et 1, référence `1` ; un lot invalide n'ajoute aucun bien ; « A1 », « a1 », « A1  » refusés (`unit.reference_duplicate`) ; « A1 » dans un autre bien accepté ; navigateur réel aux deux largeurs |
| 2 | « Activité autorisée » pour une boutique, pas pour l'habitation | **vérifié** | `c2_…` : `validation.not_applicable` sur un lot d'habitation, accepté en commercial et en mixte ; montants décimaux, nuls, hors bornes ou en texte refusés ; front : le champ n'apparaît qu'en commercial ou mixte |
| 3 | Doublon de téléphone : proposition d'ouvrir la fiche ; trois formes, même fiche ; 7 chiffres refusé | **vérifié** | `c3_…` : `409 tenant.phone_duplicate` avec la fiche existante, rien de créé ; `70123456`, `+226 70 12 34 56`, `0022670123456` donnent `+22670123456` ; 7 chiffres : `validation.phone_format` ; fiche archivée : numéro libéré ; front : [Ouvrir la fiche] seulement (DP-92) |
| 4 | Pièce d'identité ni saisissable ni lisible | **vérifié** | `c4_c8_…` : `403 feature.disabled` ; `id_document` dans un corps : `422 request.unknown_field` ; aucun champ dans le formulaire |
| 5 | Agent terrain assigné à 2 lots sur 5 : ne voit que ces 2 | **vérifié** | `c5_…` : liste, fiche (`404` pour les 3 autres) et recherche ; biens limités à ceux qui contiennent ses lots ; aucun locataire avant les baux ; navigateur réel : le gérant coche un lot dans S18, l'agent terrain ne voit que ce lot |
| 6 | Deux gestionnaires sur la même fiche : conflit de version | **vérifié** | `c6_…` sur bien, lot et locataire : `428` sans `If-Match`, `412` périmé, rien d'écrasé |
| 7 | Lot ou locataire archivé : consultable, rien de supprimé | **vérifié** | `c7_…` : bien et lots archivés ensemble, lisibles, hors des listes par défaut, non modifiables ; `DELETE` refusé au rôle `immo_app` sur les quatre tables |
| 8 | Caissier : fiches en lecture, sans contact de secours ni notes | **vérifié** | `c4_c8_…` : champs absents (pas `null`) pour le caissier et le comptable ; écriture refusée ; front : rien de masqué n'est affiché |

## Liste de démonstration (Mimis)

Préparer : `powershell -File ops/dev/real-stack.ps1 -Port 5435 -KeepStack`, puis http://127.0.0.1:4173, compte `+226 70 12 34 56`, mot de passe `un mot de passe correct` (jetable). La démonstration 3 suit DP-92 (le doublon n'est pas fusionnable).

1. Créer « Zogona » avec « A1 » (habitation) et « B1 » (boutique) : le champ « Activité autorisée » n'apparaît que pour la boutique. **Joué par le parcours réel.**
2. Créer la maison seule « Villa Kaboré » : un bien, un lot. **Joué.**
3. Créer « Issouf Tapsoba » (`+226 00 00 01 01`), puis un second locataire avec ce numéro : message et [Ouvrir la fiche]. Créer la seconde fiche sans numéro, puis « Fusionner avec une autre fiche ». **Joué.**
4. Taper « tapsoba » puis « 00000101 » dans la recherche : la fiche est trouvée les deux fois. **Joué.**
5. Dans « Utilisateurs », « Lots assignés » de l'agent terrain (`+226 70 12 34 62`) : cocher « B1 », enregistrer ; se connecter avec ce compte : un seul bien, un seul lot. **Joué.**

## Défauts trouvés par les tests et corrigés dans l'étape

- Les fiches F04 et F06 écrivaient `400 request.unknown_field` ; le contrat (propriétaire) et le code disent `422` : fiches corrigées.
- Sélecteur ambigu dans le parcours réel (`Référence` trouvait aussi « Loyer de référence ») : rendu exact.
- Quatre identifiants passés tels quels à l'audit (sérialisation impossible) ; trois remarques de clippy ; espaces insécables littéraux dans deux expressions régulières du front.

## Écarts et points à décider

- **DP-93 à relire** : fiche `is_self` par déclencheur ; clé des affectations `NOT VALID` ; fiche `is_self` non modifiable par l'API au plan Propriétaire ; curseur non signé (dette commune avec le journal) ; tri unique par nom.
- **Livré après le premier rapport** (04/11/2026) : sélecteur de lots assignés dans S18 (`GET /users` porte `unit_ids`), compte d'agent terrain dans la pile de démonstration. **Non livré** : mesure du budget de 150 ms sur 500 lots.
- **Second avis C-19** : s'ajoute `organizations_defaults()` (création de la fiche `is_self`).
- **Poste** : le test de délai du rendu PDF (`pdf_process`) a échoué deux fois sur trois pendant l'étape (3,06 s et 3,5 s pour un seuil de 3 s), puis a réussi au passage final ; il mesure la machine chargée, pas le code de l'étape. Un passage des parcours a été faussé par une mise en veille nocturne (base arrêtée, enveloppe `vite preview` restée vivante, arrêtée par `task.ps1`).
- **Essai et production** : inexistants (C-01, C-02, C-03).
