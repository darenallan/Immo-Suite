# Inventaire des 44 tests du schéma (`DATABASE_SCHEMA.md` §9.4)

| Champ | Valeur |
|---|---|
| Statut | Inventaire établi le 31/10/2026, fin de l'étape 00 (`roadmap/etape-00-decisions-et-prototypes.md` §9) |
| Source | `DATABASE_SCHEMA.md` §9.4, 44 tests exécutés sur un banc PGlite (PostgreSQL 18 en WebAssembly), jamais encore sur PostgreSQL ni sur Neon |
| Rejeu | **Étape 01** (migration initiale) sur PostgreSQL local puis sur l'essai Neon ; tous doivent être recréés dans `migrations/` et `tests/` |

La colonne « Éprouvé à l'étape 00 » ne dit **pas** que le test est fait : elle dit que le **mécanisme** a été vérifié sur des tables jouets par un prototype (`spikes/*/NOTE.md`). Le test sur le vrai schéma reste à écrire.

| N° | Test | Résultat attendu (banc PGlite) | Éprouvé à l'étape 00 | Détail |
|---|---|---|---|---|
| 1 | ISO-1 sans contexte : zéro ligne visible, insertion refusée | lectures vides, écriture refusée | Éprouvé sur tables jouets | (b) T1 : 0 ligne sans contexte, `INSERT` refusé (42501), local et Neon |
| 2 | ISO-2 A ne voit pas B ; écrire dans B depuis A est refusé | lecture 0, insertion refusée, update 0 ligne | Éprouvé sur tables jouets | (b) T2 : 0 ligne lue, 0 ligne modifiée, insertion refusée |
| 3 | ISO-3 clé étrangère composite : un bail de A ne peut viser un lot de B | refusé par la FK (org_id, unit_id) | Éprouvé sur tables jouets | (b) T2 : bail de A sur un lot de B refusé (23503) |
| 4 | ISO-4 le rôle applicatif n’a ni UPDATE ni DELETE ni TRUNCATE sur les tables d’écriture | 10 tables vérifiées | Non | le prototype accorde `UPDATE` sur une table de test |
| 5 | ISO-5 attributs du rôle applicatif | ni super-utilisateur, ni BYPASSRLS, ne possède rien | Éprouvé | (b) : `bypassrls=false`, `super=false`, 0 objet possédé, local et Neon |
| 6 | ISO-6 audit de schéma : RLS activée, forcée et politique présente sur chaque table org_id | 54 tables protégées ; tables sans org_id (liste blanche) : login_attempts, organizations, password_resets, platform_events, platform_staff, purge_order, users | Partiel | (b) liste `ENABLE` et `FORCE` sur 5 tables jouets, pas l'audit complet |
| 7 | ISO-7 la variable de contexte ne survit pas à la transaction | local=true : disparaît ; local=false : persiste (c’est la fuite que OrgTx interdit) | Éprouvé | (b) T4 et T4b : `set_config(…, true)` disparaît, la variante de session fuit |
| 8 | ISO-8 accès support : refusé sans accord, accordé pour la durée, refusé après révocation | sans accord=0, avec accord>0, révoqué=0, expiré=0, durée max 7 j | Non |  |
| 9 | G-01 un seul bail en cours par lot (R-020) : chevauchement refusé, jour d’enchaînement accepté, brouillon ignoré | EXCLUDE gist (org_id, unit_id, daterange) | Non |  |
| 10 | G-02 bail en préavis : la fin effective est la date de départ (E-35 : prolongation seulement) | préavis, fin non raccourcissable, loyer figé | Non |  |
| 11 | G-03 montants : zéro, négatif, trop grand refusés ; décimal arrondi par la base (limite connue) | bornes 1..999 999 999 ; ATTENTION un décimal est arrondi : protection côté Rust (type Fcfa) | Partiel | (d) refuse les décimales avant la base ; le rejet par la base reste à tester |
| 12 | G-04 dates : fin avant début (R-021), paiement futur (R-039), journée de caisse avant le paiement (DP-23) | trois refus | Non |  |
| 13 | G-05 paiements en ajout seul : UPDATE/DELETE refusés au rôle applicatif ET par déclencheur pour le propriétaire du schéma | privilèges + déclencheur (deux barrières) | Non |  |
| 14 | G-06 horodatage non falsifiable : recorded_at / issued_at / occurred_at imposés par la base | recorded_at demandé 2001-01-01, enregistré 2026-11-13 09:15:00+00 | Non |  |
| 15 | G-07 numérotation (E-10) : consécutive, sans trou après annulation, rejet d’un numéro hors séquence | numéros 1,2,3 consécutifs ; rollback rend le 3 ; saut, recul et avance de +5 refusés ; contrôle de trous = 0 | Partiel | (c) : E-10, annulation sans trou, 19 110 quittances ; rejet d'un numéro hors séquence non testé |
| 16 | G-08 passage d’année (E-27) : la séquence 2027 repart à 1, la 2026 reste intacte | HOR-2026-000003 puis HOR-2027-000001 | Éprouvé sur tables jouets | (c) : E-27, séquence 2027 repart à 1, 2026 intacte |
| 17 | G-09 série d’essai (phase setup) séparée de la série officielle | trial autorisé en setup, official refusé en setup | Non |  |
| 18 | G-10 avenant (E-02) : valeurs par date d’effet ; non rétroactif si une échéance est déjà payée | mars 75 000, avril 80 000 ; avenant sur période payée refusé ; avenants immuables | Non |  |
| 19 | G-11 une échéance par bail et par période (R-027) ; génération idempotente (E-03) | unicité (bail, période) + ON CONFLICT DO NOTHING | Non |  |
| 20 | G-12 R-044 : somme des affectations ≤ paiement ; ligne jamais sur-payée (contrôle différé) | trop affecté : refusé à la validation ; 75 000 sur 100 000 : accepté | Non |  |
| 21 | G-13 affectation entre baux différents impossible (clés étrangères composites) | refusé : la ligne appartient à un autre bail | Non |  |
| 22 | G-14 contre-passation (E-38) : inverse exact, affectations annulées, un seul reversal, gérant seulement, quittance annulée, rectificative | Q=HOR-2026-000004 annulée ; Q′=HOR-2026-000005 rectificative ; net affecté 75 000 | Non |  |
| 23 | G-15 la rectificative doit référencer la quittance qu’elle remplace ; une seule rectificative par quittance | refusé | Non |  |
| 24 | ISO-9 liens publics : un jeton ne donne accès qu’à sa ligne ; sans jeton rien n’est visible | un jeton → une ligne de lien ; aucune autre table lisible sans contexte d’organisation | Non |  |
| 25 | G-16 R-081 : le dernier gérant actif ne peut pas être désactivé | refusé quand il est seul ; accepté s’il en reste un autre | Non |  |
| 26 | E-18 compte rendu M1 d’octobre : 400 000 − 32 000 − 30 000 = 338 000 ; commission arrondie (E-16, E-26, E-28) | encaissé 400 000, commission 32 000, dépenses 30 000, net 338 000 ; E-16=8 000, E-26=25 000, E-28=8 000 puis 8 800 | Non |  |
| 27 | G-17 R-072 : un encaissement ne figure que dans un seul relevé ; dépense figée une fois imputée (R-074) | doublon refusé ; montant de la dépense imputée figé | Non |  |
| 28 | E-30 réouverture avant reversement acceptée avec motif ; après reversement refusée ; reversement = net | v1 → réouverture motivée → v2 → reversement 338 000 → réouverture refusée | Non |  |
| 29 | G-18 R-014 : un lot, un seul mandat à la fois ; mandats réservés au plan Agence ; taux non rétroactif (R-069) | EXCLUDE sur mandate_units, déclencheur de plan, taux futurs au 1er du mois | Non |  |
| 30 | E-36 / R-093 / R-006 : pas de bail sur un lot en travaux ; pas de travaux sur lot occupé ; pas d’archivage avec bail en cours | trois refus, activation acceptée après remise à vacant | Non |  |
| 31 | G-19 R-017/R-018 : téléphone unique par organisation, format +226 | doublon refusé dans A, accepté dans B (unicité par organisation) | Non | le prototype (d) propose la fusion des téléphones en double ; l'unicité en base n'est pas testée |
| 32 | E-20 / E-21 : import de 200 lignes (197 valides, 3 erreurs) puis annulation exacte | 197/3 lignes (14, 87, 152) ; annulation : 0 bail, 0 locataire, 0 échéance ; import utilisé : refusé | Éprouvé sur tables jouets | (d) : 197 créables, 3 erreurs aux lignes 14, 87, 152, annulation exacte, refus après paiement, local et Neon |
| 33 | Cycle de vie : purge possible seulement en setup (décision n° 25) ; retour à setup interdit | purge OK en setup, refusée en live, autres organisations intactes, liste de purge complète | Non |  |
| 34 | G-20 limite connue : `create_organization` accepte n’importe quel utilisateur ; l’appelant doit être la couche `identity` | LIMITE : create_organization accepte n’importe quel p_user ; l’appelant doit être la couche identity (voir SECURITY.md) | Non |  |
| 35 | G-21 idempotence (R-042) : même clé deux fois = une ligne ; contenu différent détecté par request_hash | clé unique par organisation ; hash différent → l’application répond 422 | Éprouvé sur tables jouets | (c) : idempotence (i) à (iv), 422 sur contenu différent |
| 36 | E-05 / E-25 / E-37 : loyer 75 000, 50 000 payés → partielle avant J+1, en retard dès J+1 ; due à J0 | E-05: reste 25 000 ; E-25: en retard dès le 14/11 ; E-37: à venir / due / en retard (fonction de référence et requête SQL identiques) | Non |  |
| 37 | E-06 / E-31 : paiement de 100 000 pour 75 000 → 75 000 affectés, avance de 25 000 (crédit), puis affectée à l’échéance suivante | crédit 25 000 → affecté à décembre → reste dû 50 000 (E-31) | Non |  |
| 38 | E-13 / E-13b : tranches d’ancienneté 3, 20, 70 jours et 30, 31, 59, 60 jours | bornes DP-12 respectées par la fonction de référence (la source reste le crate domain) | Non |  |
| 39 | E-02 / E-35 : prolongation du bail par avenant, même bail, dépôt inchangé | 31/12/2026 → 30/06/2027 sur le même bail | Non |  |
| 40 | Vues : soldes dérivés identiques aux calculs directs (E-05, E-31) et isolées par organisation | reste dû 25 000 (E-05) ; décembre 25 000 affectés / 50 000 restants (E-31) ; E-38 net 75 000 ; B ne voit rien | Non |  |
| 41 | E-28 / E-29 / E-19 : base de commission, dépenses selon le payeur, paiement tardif reporté au relevé suivant | E-28 : 8 000 puis 8 800 ; E-29 : seule la dépense agence est déduite par défaut ; E-19 : absent du relevé d’octobre (non rattaché, hors lien), présent dans le relevé de novembre | Non |  |
| 42 | Démonstration : une organisation demo n’accepte que des comptes demo, et inversement | refus dans les deux sens | Non |  |
| 43 | R-076 : validation et reversement refusés à un non-gérant ; R-089 : saisie dans une journée de caisse clôturée refusée | non-gérant refusé ; journée clôturée refusée ; paiement du 12 saisi le 13 accepté dans la journée du 13 ; écart −1 000 | Non |  |
| 44 | Section 7.5 : les requêtes de contrôle s’exécutent et ne renvoient aucune anomalie | anomalies 0/0/0/0 ; totaux de l'organisation A : 20 lignes, somme 1141000 | Non |  |

Total : 44 tests ; éprouvés : 2 ; éprouvés sur tables jouets : 6 ; partiels : 3 ; non éprouvés : 33.
