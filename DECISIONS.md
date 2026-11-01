# Immo Suite — Journal de décisions

| Champ | Valeur |
|---|---|
| Statut | Document vivant, propriétaire des décisions prises et ouvertes et de la stratégie |
| Version | 0.4 |
| Date | 31/10/2026 |
| Dépend de | Dossier de cadrage v1.0 (§26), `AUDIT_COHERENCE.md`, `AUDIT_COHERENCE_2.md` |
| Alimente | Tous les documents (`CLAUDE.md` renvoie ici pour toute décision) |

Document vivant. Source : dossier de cadrage v1.0 du 24/10/2026 (section 26 pour les décisions ouvertes).

## Décisions prises

Décisions confirmées par Mimis le 24/10/2026. Statut de chacune : **Prise**.

### DP-01 — Cible : agences ET propriétaires directs, deux plans

- **Date** : 24/10/2026
- **Décision** : Immo Suite vise les agences et les propriétaires directs. Deux plans, « Propriétaire » et « Agence », sur une seule base de code.
- **Raison** : la valeur croît avec le nombre de lots, le nombre de personnes qui touchent à l'argent et la présence de propriétaires tiers (dossier §2.2). Même moteur, périmètre activé par plan (§1, §9.2).
- **Conséquence** :
  - F01 gère le type d'organisation et le plan.
  - Mandats, commissions et compte rendu propriétaire (F05, F13) sont réservés au plan Agence. Clôture de caisse complète aussi (F19).
  - Le plan Propriétaire a deux profils prédéfinis (propriétaire, collaborateur) et un plafond de lots à fixer [À valider].
  - Le premier segment de pilote reste à décider (décision ouverte n° 2). Les prix restent à tester par plan (n° 3).

### DP-02 — MVP large, historique des travaux inclus

- **Date** : 24/10/2026
- **Décision** : le MVP est large et inclut le journal des travaux et dépenses par lot (F15).
- **Raison** : valeur forte pour la confiance et l'historique du bien (dossier §6, JTBD « Should »).
- **Conséquence** :
  - F15, F16 (documents) et F17 (agenda) sont en V1.
  - Le dossier les livre après le cœur P0, en tranche T3. Seule une version minimale de F15 (dépense imputable) est livrée en T2, car F13 en dépend.
  - Risque identifié : le périmètre gonfle et retarde la première version (§24). Mitigations du dossier : livraison par tranches, pilote après T2, règle « une P2 à la fois ».

### DP-03 — Habitation ET commercial dès la V1

- **Date** : 24/10/2026
- **Décision** : les lots d'habitation et les locaux commerciaux sont gérés dès la V1.
- **Raison** : éviter une refonte du modèle de données. Le niveau Bien → Lot permet les deux usages dès le premier jour (§1, §8).
- **Conséquence** :
  - Le lot porte un usage et un type. Le bail porte un type (habitation / commercial).
  - Champs commerciaux prévus : activité, indexation, durée, TVA éventuelle.
  - Les règles juridiques du bail commercial (droit OHADA) ne sont pas automatisées [À valider].
  - L'indexation automatique (F30) et les pénalités automatiques (F21) restent hors MVP.

### DP-04 — Copropriété non retenue pour l'instant

- **Date** : 24/10/2026
- **Décision** : la copropriété (F35) n'est pas dans le périmètre actuel.
- **Raison** : autre métier. Millièmes, charges et appels de fonds n'ont presque rien de commun avec la location (§6, §8).
- **Conséquence** :
  - F35 est en P3, non planifiée.
  - Ne pas anticiper la copropriété dans le schéma de données.
  - « Pour l'instant » : à rouvrir seulement sur demande client documentée.

### DP-05 — Application web responsive (téléphone + ordinateur)

- **Date** : 24/10/2026
- **Décision** : Immo Suite est une application web responsive, utilisable sur téléphone et sur ordinateur.
- **Raison** : le web responsive suffit pour valider. Une application native ou hors-ligne ajoute une complexité de synchronisation (§10).
- **Conséquence** :
  - L'application mobile hors-ligne (F33) est hors MVP (P3).
  - Conception pour un Android d'entrée de gamme et une connexion lente (§14.3).
  - Navigation : menu latéral sur ordinateur, barre du bas à 4 entrées sur mobile.
  - Quittance imprimable en A4 et A5.

### DP-06 — Backend en Rust, sans réutiliser la stack de Scolo

- **Date** : 24/10/2026
- **Décision** : le backend est en Rust. La stack de Scolo n'est pas réutilisée.
- **Raison** : arguments du dossier (§12) : sécurité mémoire, faible consommation, un seul binaire, erreurs attrapées à la compilation sur du code financier. La raison du non-réemploi de Scolo n'est pas documentée (voir Questions ouvertes).
- **Conséquence** :
  - Monolithe modulaire Rust avec PostgreSQL. Pas de microservices.
  - Axum, Tokio, SQLx : recommandation du dossier, à figer dans ARCHITECTURE.
  - Risque : Rust ralentit l'itération (§24). Mitigations : monolithe simple, peu de dépendances, prototypes courts, Rust limité au backend.
  - Le frontend reste à décider (décision ouverte n° 6).

### DP-07 — Maquette interactive complète avant tout code, puis enquête terrain

- **Date** : 24/10/2026
- **Décision** : une maquette interactive complète est produite avant tout code. L'enquête terrain vient ensuite.
- **Raison** : valider avant de coder. Le dossier pose un seuil de validation fixé à l'avance et dit : « sans validation, on ne code pas » (§1, §17).
- **Conséquence** :
  - Aucun développement de l'application avant le Go de la phase 0 (portée précisée par DP-35).
  - Chaque écran de la maquette porte un repère « disponible en V1 / à venir » pour ne rien promettre à tort (§14).
  - Les seuils Go/No-Go doivent être fixés avant le premier entretien (fixés par DP-28, puis DP-34).
  - Les trois points de tension avec le dossier sont tranchés par DP-09, DP-10 et DP-11.
- **Statut** : Remplacée par DP-52 (portée « avant tout code »). La maquette complète reste faite et conservée.

### DP-08 — Paiements principalement en espèces et mobile money

- **Date** : 24/10/2026
- **Décision** : le produit part du principe que les loyers sont encaissés surtout en espèces et par mobile money.
- **Raison** : contexte local décrit par le dossier (§2.1, §5). La part exacte reste une hypothèse (H2) [À valider].
- **Conséquence** :
  - Saisie manuelle des paiements : espèces, mobile money saisi, virement, chèque (F09).
  - Lecture assistée d'un SMS mobile money collé (F25, P1).
  - Clôture de caisse pour les espèces (F19, P1).
  - Le paiement en ligne par le locataire (F26) reste hors MVP (P2). Décision n° 8 fermée par DP-32.
  - HABEK ne détient jamais les fonds des clients.

### DP-09 — Maquette complète (tous les écrans)

- **Date** : 24/10/2026
- **Contexte** : DP-07 dit « maquette complète ». Le dossier recommande cinq parcours (décision n° 4).
- **Décision** : la maquette couvre tous les écrans de la section 14.2 du dossier, et pas seulement cinq parcours.
- **Alternatives écartées** : cinq parcours (recommandation du dossier) ; cinq parcours puis extension pendant l'enquête.
- **Conséquences** :
  - Ferme la décision ouverte n° 4, avec une option différente de la recommandation du dossier.
  - Les cinq parcours restent prioritaires dans l'ordre de réalisation : encaisser et envoyer une quittance, impayés et relance, compte rendu propriétaire, import Excel, historique d'un lot.
  - Chaque écran porte un repère « disponible en V1 / à venir » (§14).
  - Risque noté par le dossier : une maquette trop ambitieuse retarde l'enquête (§17, phase 0). Mitigation : DP-10.
  - Liste précise des écrans de la première maquette : DP-31.
- **Statut** : Prise
- **Liens** : décision n° 4, DP-07.

### DP-10 — Enquête terrain en parallèle de la maquette

- **Date** : 24/10/2026
- **Contexte** : DP-07 dit « puis enquête terrain ». Le dossier prévoit des entretiens en parallèle de la maquette.
- **Décision** : la prise de rendez-vous et les premiers entretiens démarrent pendant la réalisation de la maquette. La maquette est montrée dans les entretiens suivants.
- **Alternatives écartées** : enquête strictement après la maquette terminée (allonge la phase 0 et retarde les premiers signaux de marché).
- **Conséquences** :
  - La phase 0 garde l'estimation du dossier : 4 à 6 semaines.
  - Les seuils Go/No-Go (décision n° 1) doivent être fixés avant le premier entretien, donc avant la fin de la maquette.
  - Les entretiens suivent le guide de la section 20 et l'échelle d'engagement.
- **Statut** : Remplacée par DP-52
- **Liens** : DP-07, décision n° 1.

### DP-11 — Prototype PDF jetable autorisé avant le Go

- **Date** : 24/10/2026
- **Contexte** : DP-07 dit « aucun code avant la maquette ». Le dossier inclut un prototype PDF d'une journée en phase 0.
- **Décision** : un prototype de quittance PDF est autorisé avant le Go. Il est jetable : il ne sert pas de base au produit.
- **Alternatives écartées** : aucun prototype avant le Go (choix du moteur PDF repoussé en phase 1).
- **Conséquences** :
  - Seule exception à « aucun code applicatif » avant le Go (portée précisée par DP-35).
  - Sert à préparer la décision n° 7 (moteur PDF), qui reste ouverte.
  - Peut être montré aux entretiens pour valider la mise en page.
- **Statut** : Prise (précisée par DP-52 : le prototype devient le prototype (a) d'`ARCHITECTURE.md` §7, de 2 à 3 jours)
- **Liens** : DP-07, décision n° 7.

### DP-12 — Tranches d'ancienneté : 1–7, 8–30, 31–59, 60 et +

- **Date** : 24/10/2026
- **Contexte** : le dossier (F11) liste « 31–60 j » et « 60 j et + », qui se chevauchent à 60 jours.
- **Décision** : tranches 1–7 j, 8–30 j, 31–59 j, 60 j et +. Un retard de 60 jours est dans la dernière tranche.
- **Alternatives écartées** : 1–7, 8–30, 31–60, 61 et + (change l'étiquette « 60 j et + » du dossier).
- **Conséquences** : R-058 et E-13b dans `DOMAINE.md`. Le tableau des impayés et les tests utilisent ces bornes.
- **Statut** : Prise
- **Liens** : F11, R-058.

### DP-13 — Échéance partielle en retard : « en retard » prime

- **Date** : 24/10/2026
- **Contexte** : le dossier donne un seul statut par échéance (à venir, due, partielle, payée, en retard), sans dire lequel s'applique à une partielle dépassée.
- **Décision** : à partir de J+1, une échéance au solde non nul est « en retard », même partiellement payée. « Partielle » ne s'applique qu'avant J+1. L'écran montre le montant payé et le reste dû.
- **Alternatives écartées** : « partielle » prime (risque de sortir du tableau des impayés) ; deux axes séparés (s'écarte de la liste de cinq statuts du dossier).
- **Conséquences** : R-033, R-045 et E-25 dans `DOMAINE.md`. Le tableau des impayés filtre sur le statut « en retard ».
- **Statut** : Prise
- **Liens** : F08, F09, F11.

### DP-14 — Commission : taux décimal, un arrondi par relevé

- **Date** : 24/10/2026
- **Contexte** : les montants sont des entiers FCFA (R-001). Un taux comme 7,5 % ne donne pas toujours un entier.
- **Décision** : le taux peut être décimal. La commission est calculée sur le total encaissé du mandat pour la période, arrondie à l'entier le plus proche (0,5 vers le haut). Un seul arrondi par relevé et par mandant.
- **Alternatives écartées** : arrondi paiement par paiement (l'écart s'accumule) ; taux entier uniquement.
- **Conséquences** : R-068 et E-26 dans `DOMAINE.md`. Le montant de la commission est un entier stocké au relevé. La base de calcul (loyer seul ou avec charges) reste une question ouverte.
- **Statut** : Prise
- **Liens** : F05, F13, décision n° 10.

### DP-15 — Quittances : séquence annuelle, une par paiement

- **Date** : 24/10/2026
- **Contexte** : le dossier exige une numérotation continue, sans trou, par organisation (préfixe + année + séquence), et dit « un paiement ou un lot de paiements ».
- **Décision** : la séquence repart à 1 chaque année, sans trou dans l'année. Une quittance par paiement. La rectificative reçoit son propre numéro suivant et référence l'originale.
- **Alternatives écartées** : séquence continue sans remise à zéro ; une quittance pour un lot de paiements.
- **Conséquences** : R-049, R-050, R-051 et E-27 dans `DOMAINE.md`. Le numéro est attribué dans la même transaction que la validation du paiement.
- **Statut** : Prise
- **Liens** : F10.

### DP-16 — Base de commission : loyer seul, paramétrable par mandat

- **Date** : 24/10/2026
- **Contexte** : le dossier dit que le taux s'applique sur les loyers encaissés et que la base reste paramétrable, sans préciser charges, arriérés ni avances.
- **Décision** : base par défaut = lignes de type loyer encaissées dans la période, charges exclues. Les arriérés et avances encaissés comptent à leur date d'encaissement. Chaque mandat peut choisir la base « loyer + charges ».
- **Alternatives écartées** : loyer + charges toujours ; loyer seul avec arriérés exclus.
- **Conséquences** : R-068 et E-28 dans `DOMAINE.md`. Le mandat porte un champ « base de calcul ». Dépend de R-071 (date d'encaissement), qui reste [À valider] (décision ouverte n° 10). Si R-071 change, cette décision est à revoir.
- **Statut** : Prise
- **Liens** : F05, F13, décisions n° 10 et DP-14.

### DP-17 — Réouverture d'un relevé avant reversement

- **Date** : 24/10/2026
- **Contexte** : F13 prévoit « rouvrir avec motif tracé », alors que le relevé validé est « figé ».
- **Décision** : le gérant peut rouvrir un relevé validé tant qu'aucun reversement n'est enregistré. Motif obligatoire et tracé. Une nouvelle version est créée, les versions précédentes sont conservées. Après un reversement, la réouverture est refusée et toute correction va en période suivante.
- **Alternatives écartées** : jamais de réouverture ; réouverture même après reversement.
- **Conséquences** : R-073, R-083, E-30 et le cycle de vie du relevé dans `DOMAINE.md`. Le relevé est versionné.
- **Statut** : Prise
- **Liens** : F13.

### DP-18 — Dépenses imputables selon le payeur

- **Date** : 24/10/2026
- **Contexte** : une dépense porte un payeur (propriétaire, agence ou locataire) et un statut d'imputation. Le dossier ne dit pas quelles dépenses se déduisent.
- **Décision** : une dépense payée par l'agence est imputable par défaut. Une dépense payée par le propriétaire ou le locataire est non imputable par défaut et figure à titre d'information. Le gérant peut changer l'imputation, avec trace.
- **Alternatives écartées** : imputation toujours manuelle ; toutes les dépenses déduites.
- **Conséquences** : R-078, R-084 et E-29 dans `DOMAINE.md`. Le changement d'imputation est une action d'audit.
- **Statut** : Prise
- **Liens** : F13, F15.

### DP-19 — Dépôt de garantie hors du relevé propriétaire

- **Date** : 24/10/2026
- **Contexte** : le dossier ne dit pas si le dépôt de garantie entre dans les loyers encaissés du relevé.
- **Décision** : le dépôt de garantie n'entre ni dans les loyers encaissés, ni dans la base de commission. Il est suivi à part (F20) et peut figurer au relevé à titre d'information.
- **Alternatives écartées** : ligne séparée reversée au propriétaire (suppose que l'agence reverse la caution) ; inclus dans les loyers encaissés.
- **Conséquences** : R-085 dans `DOMAINE.md`. Qui détient la caution (agence ou propriétaire) reste [À valider] sur le terrain.
- **Statut** : Prise
- **Liens** : F20, F13.

### DP-20 — Avance sans échéance suivante : crédit du bail

- **Date** : 24/10/2026
- **Contexte** : l'échéancier ne génère que les 30 jours à venir (R-029). Un excédent de paiement peut n'avoir aucune échéance où s'affecter.
- **Décision** : la part non affectée est un crédit du bail, visible. Elle s'affecte automatiquement dès qu'une échéance est générée. Si le bail se termine, le crédit reste visible et se règle hors flux, avec trace.
- **Alternatives écartées** : générer les échéances futures (contredit la fenêtre de 30 jours) ; refuser l'excédent.
- **Conséquences** : R-086 et E-31 dans `DOMAINE.md`. La génération d'échéances applique le crédit du bail. Le traitement exact d'un crédit sur bail terminé reste à préciser sur le terrain.
- **Statut** : Prise
- **Liens** : F08, F09, R-044.

### DP-21 — Ordre d'affectation : date, puis loyer, charges, exceptionnelles

- **Date** : 24/10/2026
- **Contexte** : le dossier affecte aux échéances les plus anciennes, sans ordre à date égale.
- **Décision** : tri par date d'échéance (arriérés inclus). À date égale : loyer, puis charges, puis lignes exceptionnelles. Ajustable par l'utilisateur.
- **Alternatives écartées** : prorata des lignes ; charges d'abord.
- **Conséquences** : R-087 et E-32 dans `DOMAINE.md`. Sur un paiement partiel, l'ordre augmente la base de commission (DP-16) par rapport à un prorata.
- **Statut** : Prise
- **Liens** : F09, DP-16.

### DP-22 — Paiement rattaché au bail, champ « payé par »

- **Date** : 24/10/2026
- **Contexte** : un bail peut avoir plusieurs locataires (colocation). Un tiers peut payer à la place du locataire.
- **Décision** : le paiement est rattaché au bail. Le champ « payé par » désigne un locataire du bail ou un tiers. La quittance est au nom du ou des locataires du bail et mentionne le tiers.
- **Alternatives écartées** : paiement rattaché à un colocataire ; tiers interdit.
- **Conséquences** : R-088 et E-33 dans `DOMAINE.md`. Le modèle de données lie le paiement au bail. La responsabilité des colocataires reste une question ouverte.
- **Statut** : Prise
- **Liens** : F09, F10.

### DP-23 — Saisie après clôture de caisse : journée de la saisie

- **Date** : 24/10/2026
- **Contexte** : la clôture verrouille la journée de caisse (F19). Un paiement en espèces peut être saisi le lendemain.
- **Décision** : le paiement entre dans la journée ouverte du jour de saisie. Sa date de paiement réelle est conservée. Le caissier est averti de l'écart.
- **Alternatives écartées** : rouvrir la journée clôturée ; refuser la saisie.
- **Conséquences** : R-089 et E-34 dans `DOMAINE.md`. Le paiement porte deux dates : date de paiement et journée de caisse.
- **Statut** : Prise
- **Liens** : F19, F09.

### DP-24 — Bail : quatre statuts, renouvellement par avenant

- **Date** : 24/10/2026
- **Contexte** : le dossier cite un « statut » du bail sans liste, parle d'un préavis en fin de bail (F23) et d'une action « renouveler » (F07).
- **Décision** : statuts brouillon, actif, en préavis, terminé. En préavis, le bail reste actif et les échéances continuent jusqu'au départ. Renouveler = avenant daté qui prolonge la date de fin. Un changement de locataire est un nouveau bail. La durée du préavis est saisie au bail, sans défaut.
- **Alternatives écartées** : renouvellement par nouveau bail lié ; trois statuts avec préavis réduit à une date.
- **Conséquences** : R-090, R-091, E-35 et le cycle de vie du bail dans `DOMAINE.md`. La durée légale du préavis reste [À valider par un juriste].
- **Statut** : Prise
- **Liens** : F07, F23, décision n° 12.

### DP-25 — Lot « en travaux » sur lot vacant ; lot « mixte » via le bail

- **Date** : 24/10/2026
- **Contexte** : le lot a un statut « en travaux » et un usage « mixte », sans règle associée. Le bail n'a que deux types.
- **Décision** : « en travaux » se pose à la main sur un lot vacant, et un bail ne peut pas s'activer tant que le lot est en travaux. Un lot occupé reçoit seulement une intervention. Un lot « mixte » est accepté ; le type du bail choisi à sa création détermine les règles. Pas de bail « mixte ».
- **Alternatives écartées** : retirer les deux de la V1 ; travaux compatibles avec un bail actif et bail « mixte ».
- **Conséquences** : R-093, R-094 et E-36 dans `DOMAINE.md`.
- **Statut** : Prise
- **Liens** : F04, F15, DP-03.

### DP-26 — Commercial : ni TVA ni impôt calculés

- **Date** : 24/10/2026
- **Contexte** : le dossier cite une TVA éventuelle et des loyers variables pour le commercial, sans règle. Il exclut tout calcul d'impôt du MVP (§2.4).
- **Décision** : aucun calcul de TVA. Le loyer est le montant dû tel que saisi. Un indicateur facultatif « TVA incluse » s'affiche sur le bail et la quittance. Un loyer variable passe par des avenants datés ou une ligne exceptionnelle.
- **Alternatives écartées** : loyer HT, taux de TVA et échéance TTC calculée ; exclusion complète du MVP.
- **Conséquences** : R-092 dans `DOMAINE.md`. À confronter au terrain (H7). Règle d'arbitrage du dossier : un besoin devient une option si trois clients au moins le demandent.
- **Statut** : Prise
- **Liens** : DP-03, F07, H7.

### DP-27 — Une échéance est « due » le jour de sa date

- **Date** : 24/10/2026
- **Contexte** : le dossier liste le statut « due » sans dire quand une échéance l'est.
- **Décision** : « à venir » avant la date d'échéance, « due » le jour de cette date (J0), « en retard » dès J+1 si le solde n'est pas nul.
- **Alternatives écartées** : « due » dès J−3 ; suppression du statut « due ».
- **Conséquences** : R-095 et E-37 dans `DOMAINE.md`. La relance courtoise à J−3 reste liée au modèle de message, pas au statut.
- **Statut** : Prise
- **Liens** : F08, R-033.

### DP-28 — Seuils Go/No-Go de la phase 0

- **Date** : 24/10/2026
- **Contexte** : DP-10 exige des seuils fixés avant le premier entretien. Le dossier les propose (§17) mais ne dit pas quoi décider à 2 seuils sur 4.
- **Décision** : sur 15 entretiens, quatre seuils :
  - au moins 8 confirment que le suivi des loyers et des impayés fait partie de leurs trois principaux soucis ;
  - au moins 4 acceptent de partager un échantillon réel de données (20 locataires) ;
  - au moins 3 signent une lettre d'intention de pilote ;
  - au moins 2 donnent un prix viable.
- **Règle de décision** : 4 seuils atteints = Go. 3 atteints = Go conditionnel : Mimis tranche par écrit, en nommant le seuil manqué, dans une entrée DP-xx. 2 atteints ou moins = révision de cap.
- **Échantillon** : 10 agences et 5 propriétaires directs. Au moins 2 lettres d'intention viennent d'agences.
- **Prix viable** : un prix mensuel cité comme acceptable (méthode trop cher / bon marché / limite) au moins égal au bas de la grille du plan : 5 000 FCFA pour le plan Propriétaire, 25 000 FCFA pour le plan Agence (dossier §22.2, hypothèses). Un prix cité sous 10 000 FCFA déclenche aussi la revue de cible prévue par le dossier (§20.1).
- **Alternatives écartées** : seuils 10/5/4/3 (risque de No-Go injustifié) ; seuils 6/3/2/1 (engagement trop faible) ; Go dès 3 sur 4 ; 4 sur 4 obligatoire ; échantillon sans quotas ou 7 agences / 8 propriétaires ; prix viable jugé au cas par cas ou plancher unique de 10 000 FCFA.
- **Conséquences** :
  - Ferme la décision ouverte n° 1. Les seuils ne changent plus après le premier entretien, sauf par une nouvelle entrée DP-xx qui remplace celle-ci.
  - À 2 sur 4, la règle est plus stricte que le dossier, qui parle de « moins de la moitié ».
  - Ne ferme pas la décision n° 2 (premier segment de pilote). L'échantillon de 10 agences et 5 propriétaires reflète seulement la recommandation du dossier.
  - `CONTEXTE.md` (critères Go/No-Go) est aligné sur cette entrée.
- **Statut** : Remplacée par DP-34
- **Liens** : décision n° 1, DP-10, H5, §17, §20.1.

Les entrées DP-29 à DP-43 appliquent les recommandations de l'audit `AUDIT_COHERENCE.md`, validées par Mimis le 24/10/2026.

### DP-29 — V1 = T1 + T2 + T3 ; F19, F23 et F25 rejoignent T3

- **Date** : 24/10/2026
- **Contexte** : « V1 » n'était défini nulle part. F19, F23 et F25 (P1) n'avaient pas de tranche (dossier §7.3 et §9.1). La règle « P1 → V1 » de la maquette les affichait comme livrées (audit C-001, C-003).
- **Décision** : V1 = T1 + T2 + T3, le découpage en tranches du dossier étant adopté. T3 est livrée pendant le pilote, qui démarre à la fin de T2 (dossier §9.1). F19 (clôture de caisse), F23 (fin de bail et changement de locataire) et F25 (saisie assistée depuis un SMS mobile money) entrent en T3. Dans la maquette, le repère suit la tranche : T1, T2 et T3 → « V1 », avec la mention « livré pendant le pilote » pour T3 ; fonction hors tranche → « À venir ».
- **Alternatives écartées** : repère « À confirmer » pour F19, F23, F25 ; parcours 5 réduit à la version minimale T2.
- **Conséquences** : tableau T3 de `CONTEXTE.md` ; règle des repères de `MAQUETTE.md` ; ligne « Tranche » de chaque écran de `UI_UX_DESIGN.md`. La question « tranches à confirmer » est close.
- **Statut** : Prise
- **Liens** : F19, F23, F25, DP-02, DP-07.

### DP-30 — Légende « À venir » et fonctions exclues non montrées

- **Date** : 24/10/2026
- **Contexte** : la légende « prévu plus tard » promettait des fonctions que le dossier déclare conditionnelles à la demande (§8, §17 phase 7). Le paiement en ligne et l'API WhatsApp sont exclus du MVP pour coût et risque (audit C-002).
- **Décision** : légende « À venir : envisagé selon la demande des clients, non planifié, sans engagement de date. » Le paiement en ligne (F26) et l'API WhatsApp Business (F27) ne sont pas montrés dans la maquette. Les autres éléments « À venir » restent : F29 (S09), F30 (S10), F28 (S12), F24 (S14), F31 (S17).
- **Alternatives écartées** : garder F26 et F27 visibles et désactivés pour mesurer la demande.
- **Conséquences** : `MAQUETTE.md` (légende, tableau des repères) et `UI_UX_DESIGN.md` (S05, S11).
- **Statut** : Prise
- **Liens** : DP-07, F26, F27, §8, §17.

### DP-31 — Périmètre de la première maquette (précise DP-09)

- **Date** : 24/10/2026
- **Contexte** : DP-09 vise les 12 écrans de §14.2. Le catalogue de `UI_UX_DESIGN.md` en compte 24, et deux écrans nécessaires aux parcours manquaient (audit C-022, C-011).
- **Décision** : la première maquette contient les 12 écrans du dossier §14.2, dont leurs sous-écrans (fiche lot S07, fiche locataire S09, utilisateurs et invitations S18), plus S01 (connexion), S02 (assistant de démarrage), S13 (mandants et mandats), S25 (quittance) et S26 (page de partage). Sont reportés après les premiers entretiens : S19 clôture de caisse, S20 notifications, S21 journal d'audit, S22 rapports avancés, S23 portail propriétaire mandant, S24 état des lieux.
- **Alternatives écartées** : valider les 24 écrans et accepter l'allongement de la phase 0.
- **Conséquences** : Niveau 2 de `MAQUETTE.md` réduit. Le catalogue de `UI_UX_DESIGN.md` reste la spécification du produit. La question de DP-09 sur « complète » est close.
- **Statut** : Prise
- **Liens** : DP-09, F10, F13, H11, §24.

### DP-32 — Paiement en ligne : P2, jamais de détention de fonds (ferme la décision n° 8)

- **Date** : 24/10/2026
- **Contexte** : la décision n° 8 était ouverte alors que DP-08, CONTEXTE, UI et MAQUETTE traitaient F26 comme hors MVP (audit C-016).
- **Décision** : le paiement en ligne par le locataire (F26) est en P2, hors MVP. HABEK ne détient jamais les fonds des clients.
- **Alternatives écartées** : MVP ; jamais.
- **Conséquences** : la contradiction de DP-08 est levée. F26 n'apparaît pas dans la maquette (DP-30).
- **Statut** : Prise
- **Liens** : décision n° 8, DP-08, F26.

### DP-33 — Relances : lien pré-rempli (ferme la décision n° 9)

- **Date** : 24/10/2026
- **Contexte** : la décision n° 9 était ouverte alors que CONTEXTE, UI et MAQUETTE construisaient les relances sur le lien pré-rempli (audit C-016).
- **Décision** : les relances et les quittances passent par un lien WhatsApp pré-rempli, sans API. L'API WhatsApp Business (F27) viendra après preuve de demande (dossier §17, phase 7).
- **Alternatives écartées** : API WhatsApp dès le MVP.
- **Conséquences** : la maquette simule le lien pré-rempli.
- **Statut** : Prise
- **Liens** : décision n° 9, F12, F27.

### DP-34 — Seuils Go/No-Go : « prix viable » précisé (remplace DP-28)

- **Date** : 24/10/2026
- **Contexte** : DP-28 se contredisait : un propriétaire citant 7 000 FCFA était « viable » (plancher 5 000) et déclenchait en même temps la revue de cible (« sous 10 000 »). Le plancher Agence de 25 000 écartait aussi 20 000–24 999, que H5 tient pour acceptable (audit C-018).
- **Décision** : tout DP-28 est repris sans changement (quatre seuils sur 15 entretiens : 8 / 4 / 3 / 2 ; règle 4 sur 4 = Go, 3 = Go conditionnel écrit, 2 ou moins = révision de cap ; échantillon de 10 agences et 5 propriétaires directs, dont au moins 2 lettres d'intention d'agences). Précisions sur le prix viable :
  - Planchers de la grille §22.2 : 5 000 FCFA/mois pour le plan Propriétaire, 25 000 FCFA/mois pour le plan Agence (hypothèses).
  - Un prix cité sous 10 000 FCFA est un signal consigné pour la revue de cible du dossier (§20.1). Il n'annule pas le seuil n° 4.
- **Alternatives écartées** : plancher Agence à 20 000 FCFA (bas de H5) et plancher Propriétaire à 10 000 FCFA.
- **Conséquences** : `CONTEXTE.md` renvoie à cette entrée pour les seuils (source unique). Ne ferme pas la décision n° 2.
- **Statut** : Remplacée par DP-52
- **Liens** : décision n° 1, DP-10, DP-28, H5, §20.1, §22.2.

### DP-35 — « Aucun code » : portée

- **Date** : 24/10/2026
- **Contexte** : DP-07 et DP-11 parlaient de « tout code » alors que la maquette est elle-même du code (audit C-024).
- **Décision** : « aucun code applicatif avant le Go ». Sont autorisés : la maquette (HTML, CSS, JavaScript statiques, script de vérification des données) et le prototype PDF jetable (DP-11).
- **Alternatives écartées** : laisser le texte en l'état.
- **Conséquences** : précise DP-07 et DP-11 sans les remplacer.
- **Statut** : Remplacée par DP-52
- **Liens** : DP-07, DP-11.

### DP-36 — Alerte « promesse non tenue » dès T1

- **Date** : 24/10/2026
- **Contexte** : le critère de F12 (T1) parle de « l'agenda du gestionnaire », or l'agenda F17 est en T3 (dossier F12, F17, §11.4 ; audit C-004).
- **Décision** : l'alerte de F12 est un indicateur visible dans l'accueil du gestionnaire (S03) et dans la liste des impayés (S05), livré en T1. L'agenda (F17) reste en T3.
- **Alternatives écartées** : déplacer F17 ou une version minimale en T1.
- **Conséquences** : exemple E-15 et R-065 de `DOMAINE.md` reformulés.
- **Statut** : Prise
- **Liens** : F12, F17, F22, R-065, E-15.

### DP-37 — Format du numéro de quittance

- **Date** : 24/10/2026
- **Contexte** : DP-15 fixe « préfixe + année + séquence » sans en préciser le format ; l'interface proposait « 6 chiffres » sans source (audit C-032).
- **Décision** : préfixe configurable par organisation (Paramètres), année sur 4 chiffres, séquence sur 6 chiffres, séparés par un tiret. La séquence repart à 1 chaque année (DP-15).
- **Alternatives écartées** : marquer ces valeurs « proposition » sans les décider.
- **Conséquences** : exemple de format dans `UI_UX_DESIGN.md`. La maquette utilise le placeholder `{numéro suivant}` dans ses textes. Ajustable par une nouvelle entrée.
- **Statut** : Prise
- **Liens** : DP-15, F10, R-050.

### DP-38 — Contre-passation d'un paiement et quittance

- **Date** : 24/10/2026
- **Contexte** : aucune règle ne disait ce que devient la quittance quand un paiement est contre-passé (audit C-009).
- **Décision** : la contre-passation d'un paiement passe sa quittance à « Annulée ». Le paiement de remplacement génère sa propre quittance, marquée rectificative, qui référence la quittance annulée. À valider en entretien avec les agences.
- **Alternatives écartées** : rectificative réservée aux erreurs sans effet sur le montant (nom, période).
- **Conséquences** : nouvelle règle R-096 et exemple E-38 dans `DOMAINE.md` ; le jeu de données de la maquette suit cette règle.
- **Statut** : Prise
- **Liens** : F09, F10, R-048, R-051, R-096.

### DP-39 — « Bail en cours » = actif ou en préavis

- **Date** : 24/10/2026
- **Contexte** : « actif » désignait un statut et un état ; un bail en préavis « reste actif » (audit C-008).
- **Décision** : « bail en cours » = bail au statut actif ou en préavis. R-006, R-013 et R-020 s'appliquent aux baux en cours.
- **Alternatives écartées** : faire de « en préavis » un indicateur du bail actif plutôt qu'un statut.
- **Conséquences** : R-006, R-013, R-020, R-090 et glossaire de `DOMAINE.md` ; décompte de la maquette « 33 baux en cours (dont 1 en préavis) ».
- **Statut** : Prise
- **Liens** : DP-24, R-020.

### DP-40 — Rappel à J−3 dans le MVP ; définition de « Relance »

- **Date** : 24/10/2026
- **Contexte** : le dossier prévoit un rappel courtois à J−3, « assisté au MVP » (§11.4), mais « Relance » était défini comme un contact pour un impayé et aucun écran ne listait les échéances à rappeler (audit C-012).
- **Décision** : le rappel à J−3 est dans le MVP, en mode assisté. « Relance » = contact avec un locataire au sujet d'une échéance, avant l'échéance (rappel à J−3) ou après retard. L'accueil du gestionnaire (S03) porte un bloc « Rappels à envoyer » qui ouvre le panneau « Relancer » de S05.
- **Alternatives écartées** : retirer J−3 du MVP.
- **Conséquences** : glossaire et R-097 de `DOMAINE.md` ; S03 de `UI_UX_DESIGN.md`.
- **Statut** : Prise
- **Liens** : F12, R-060, §11.4.

### DP-41 — Champs commerciaux : « activité autorisée » sur le lot, « indexation » et « durée » sur le bail

- **Date** : 24/10/2026
- **Contexte** : R-012 plaçait « activité » et « indexation » sur le lot, R-025 sur le bail (audit C-025).
- **Décision** : « activité autorisée » est portée par le lot (nature des locaux). « Indexation » et « durée » sont portées par le bail (conditions du contrat). Cette décision écarte la lecture du dossier (F04) qui place l'indexation sur le lot.
- **Alternatives écartées** : tous les champs sur le bail.
- **Conséquences** : R-012 et R-025 de `DOMAINE.md` ; écrans S06 et S10.
- **Statut** : Prise
- **Liens** : F04, F07, DP-03, R-012, R-025.

### DP-42 — « Comptable (lecture) » : rôle avec compte

- **Date** : 24/10/2026
- **Contexte** : « comptable » désignait un profil qui encaisse, un tiers qui reçoit des exports et un rôle en lecture (audit C-026).
- **Décision** : « Comptable (lecture) » est un rôle avec compte, en lecture seule, qui peut être un tiers de l'agence. La persona « Caissier / comptable » devient « Caissier ».
- **Alternatives écartées** : comptable sans compte, qui reçoit seulement des exports.
- **Conséquences** : glossaire de `DOMAINE.md` ; tableau des rôles de `CONTEXTE.md`.
- **Statut** : Prise
- **Liens** : F02, §11.2.

### DP-43 — Vocabulaire de l'interface

- **Date** : 24/10/2026
- **Contexte** : « propriétaire » avait trois sens ; l'état d'un paiement avait trois libellés ; « compte rendu » et « relevé » coexistaient (audit C-010, C-015, C-044).
- **Décision** :
  - « Propriétaire direct » pour le rôle et le plan ; « mandant » pour le bénéficiaire d'un compte rendu ; « compte rendu propriétaire » reste le nom du document (dossier §14.2). L'écran S13 devient « Mandants et mandats ».
  - Paiement : « Validé » et « Corrigé ». « Payé » est réservé à l'échéance.
  - « Journal de caisse » désigne la journée de caisse (S19) ; « Partiel » est le libellé du badge d'une échéance partielle.
- **Alternatives écartées** : renommer le document « compte rendu mandant » ; garder « Enregistré » pour un paiement.
- **Conséquences** : `UI_UX_DESIGN.md` (statuts, vocabulaire, S13) et `MAQUETTE.md` (sélecteur de rôle, textes affichés).
- **Statut** : Prise
- **Liens** : DP-24, F13, R-032.

### DP-44 — Palette : bleu nuit, blanc et turquoise

- **Date** : 25/10/2026
- **Contexte** : la décision ouverte n° 16 (palette définitive) restait en suspens ; la palette provisoire bronze/ocre dérivait de HABEK sans réconciliation avec la couleur du logo.
- **Décision** : palette « Bleu nuit, blanc et turquoise », proposée dans un comparatif de palettes et retenue en premier choix par la personne qui pilote le projet. Valeurs : bleu nuit `#142B4A`, blanc `#FFFFFF`, fond `#F3F6FA`, turquoise `#159D91`, texte `#263449`.
  - Le bleu nuit porte la marque : boutons primaires, menu latéral, soulignements.
  - Le turquoise est un accent : remplissage ou bordure seulement (3,3:1 sur blanc, insuffisant pour du texte).
  - Teintes dérivées pour tenir les contrastes AA : turquoise foncé `#0E6F66` pour les liens, texte `#0A1626` sur fond turquoise, turquoise clair `#5FD3C6` sur fond bleu nuit.
  - Les couleurs de statut financier (payé, partiel, en retard, à venir, due, avance, annulé) ne changent pas, hors le gris « Annulé » passé en gris froid. Un statut n'utilise jamais le turquoise.
- **Alternatives écartées** : palette bronze/ocre dérivée de HABEK (provisoire d'origine) ; couleur du logo ; palette hybride.
- **Conséquences** : `UI_UX_DESIGN.md` (tokens, tableau des contrastes, interdits), `MAQUETTE.md`, `CLAUDE.md`, `maquette/css/tokens.css` et le menu latéral sur fond bleu nuit. Le logo a été livré le 26/10/2026 (`logo/`, repris dans la maquette) ; à réconcilier avec la palette avant sa diffusion publique. Le contrôle des contrastes est un calcul WCAG sur les valeurs ; à refaire avec un outil avant la diffusion publique.
- **Statut** : Prise
- **Liens** : décision ouverte n° 16, DP-43.

Les entrées DP-45 à DP-49 viennent de `ARCHITECTURE.md` v0.1 (30/10/2026). Ce sont des conséquences techniques directes de règles déjà prises : aucune autre option ne respecte ces règles. Confirmées par Mimis le 30/10/2026 (validation en bloc des recommandations de `AUDIT_COHERENCE_2.md`). Les choix d'architecture qui demandent un arbitrage sont dans le tableau des décisions ouvertes (n° 19 à 29).

### DP-45 — Montants en entiers 64 bits, aucun flottant

- **Date** : 30/10/2026
- **Contexte** : R-001 impose des montants entiers en FCFA. DP-14 autorise un taux de commission décimal avec un seul arrondi par relevé.
- **Décision** : colonnes `BIGINT`, type Rust `Fcfa(i64)` sans multiplication par un flottant. Aucun `float`, `NUMERIC` ni `DECIMAL` pour un montant. Seul arrondi du système : la commission, calculée en arithmétique entière (0,5 vers le haut). Le front n'effectue aucun calcul d'argent.
- **Alternatives écartées** : `NUMERIC` (autorise des décimales que R-001 interdit) ; calcul côté front (deux implémentations des règles).
- **Conséquences** : `ARCHITECTURE.md` §4.1 et §4.2. La précision du taux est la décision ouverte n° 29.
- **Statut** : Prise (confirmée par Mimis le 30/10/2026)
- **Liens** : R-001, R-015, R-068, DP-14, E-26.

### DP-46 — Numérotation des quittances par compteur transactionnel

- **Date** : 30/10/2026
- **Contexte** : DP-15 et R-050 exigent une numérotation sans trou par organisation et par année, attribuée dans la transaction de validation. Une séquence PostgreSQL laisse des trous (`nextval` n'est pas annulé par un `ROLLBACK`).
- **Décision** : table de compteurs par organisation et par année, incrémentée par `INSERT … ON CONFLICT DO UPDATE … RETURNING` dans la transaction du paiement, prise après les contrôles métier. Contrainte d'unicité sur le numéro. Rendu PDF hors transaction, depuis un instantané de la quittance. Contrôle nocturne d'absence de trou.
- **Alternatives écartées** : séquence PostgreSQL (trous) ; numéro attribué au rendu du PDF (un échec de rendu bloquerait ou trouerait la numérotation, contraire à R-049).
- **Conséquences** : `ARCHITECTURE.md` §4.4. Validé sous concurrence par le prototype (c). L'année de la séquence est l'année d'émission (DP-61).
- **Statut** : Prise (confirmée par Mimis le 30/10/2026)
- **Liens** : R-049, R-050, DP-15, DP-37, E-10, E-27.

### DP-47 — Écritures en ajout seul garanties par la base

- **Date** : 30/10/2026
- **Contexte** : paiements, quittances, reversements, relances et audit sont immuables (R-007, R-047, R-051, R-062, R-077). Le dossier exige qu'une modification de l'audit soit rejetée par la base (F03).
- **Décision** : le rôle PostgreSQL de l'application n'a ni `UPDATE` ni `DELETE` sur les tables d'écriture. Les statuts « Corrigé » d'un paiement et « Annulée » d'une quittance sont dérivés de l'écriture liée (contre-passation, annulation). Quittances et versions validées de relevés sont stockées en instantané complet.
- **Alternatives écartées** : immuabilité seulement dans le code (contournable par une erreur ou une requête manuelle) ; colonnes de statut modifiables.
- **Conséquences** : `ARCHITECTURE.md` §4.3 et §4.9. Test d'intégration qui vérifie le refus des commandes.
- **Statut** : Prise (confirmée par Mimis le 30/10/2026)
- **Liens** : R-007, R-047, R-048, R-051, R-074, R-096, DP-38.

### DP-48 — Isolation par organisation à deux barrières

- **Date** : 30/10/2026
- **Contexte** : règle d'or 8, R-004 et dossier §12 : filtrage `org_id` dans le code et dans PostgreSQL. Avec un pool de connexions, une variable de session posée par `SET` survit d'une requête à l'autre.
- **Décision** : `org_id` sur toute table métier ; sécurité au niveau des lignes activée et forcée ; rôle applicatif sans `BYPASSRLS` et non propriétaire des tables ; contexte posé par `set_config('app.org_id', …, true)` au début de chaque transaction ; type Rust `OrgTx` seul moyen d'accéder à la base ; test d'isolation par route bloquant en CI.
- **Alternatives écartées** : filtrage dans le code seulement ; une base par client (lourd à exploiter, dossier §12).
- **Conséquences** : `ARCHITECTURE.md` §5. Validé par le prototype (b).
- **Statut** : Prise (confirmée par Mimis le 30/10/2026)
- **Liens** : R-004, R-009, F01, règle d'or 8.

### DP-49 — Idempotence des écritures d'argent par clé du client

- **Date** : 30/10/2026
- **Contexte** : R-042 et E-07 : un double clic ou une reprise après coupure ne crée qu'un paiement. `UI_UX_DESIGN.md` crée la clé à l'ouverture du formulaire.
- **Décision** : en-tête `Idempotency-Key` obligatoire sur les écritures d'argent ; clé enregistrée en base, unique par organisation, dans la même transaction que l'écriture, avec la réponse rendue ; même clé et contenu différent : refus.
- **Alternatives écartées** : déduplication par comparaison des champs (deux vrais paiements identiques seraient fusionnés) ; blocage du bouton seulement (inopérant après une coupure).
- **Conséquences** : `ARCHITECTURE.md` §4.5. Le doublon de référence externe (R-041) reste un contrôle distinct, non tranché.
- **Statut** : Prise (confirmée par Mimis le 30/10/2026)
- **Liens** : R-041, R-042, E-07, F09.

Les entrées DP-50 et DP-51 viennent de `DATABASE_SCHEMA.md` v0.1 (30/10/2026). Comme DP-45 à DP-49, ce sont des conséquences techniques directes de règles déjà prises. Confirmées par Mimis le 30/10/2026 (validation en bloc des recommandations de `AUDIT_COHERENCE_2.md`). Les choix du schéma qui demandent un arbitrage sont dans le tableau des décisions ouvertes (n° 30 à 47).

### DP-50 — Horloge unique en SQL, dates et instants

- **Date** : 30/10/2026
- **Contexte** : R-002 fixe le fuseau `Africa/Ouagadougou`. `ARCHITECTURE.md` §4.6 interdit `CURRENT_DATE` et l'heure du serveur : « aujourd'hui » vient du service `Clock`, injectable. La base a besoin de l'heure pour imposer l'horodatage des écritures et refuser un paiement daté du futur.
- **Décision** : dates métier en `date`, instants en `timestamptz`. En SQL, toute lecture de l'heure passe par une seule fonction `app_now()` (en production : `now()` ; en essai et en test : redéfinie par le propriétaire du schéma). La date du jour d'une organisation se calcule par `org_local_date(org_id)`. Jamais `CURRENT_DATE`, jamais un paramètre de session modifiable par le rôle applicatif.
- **Alternatives écartées** : `CURRENT_DATE` (heure du serveur, non testable) ; un paramètre de session `app.today` (le rôle applicatif pourrait antidater un paiement).
- **Conséquences** : `DATABASE_SCHEMA.md` §1.2. Les tests fixent l'horloge au 13/11/2026 09:15. Un test de parité vérifie que le `Clock` de l'application et `app_now()` donnent la même date.
- **Statut** : Prise (confirmée par Mimis le 30/10/2026)
- **Liens** : R-002, R-039, DP-48, `ARCHITECTURE.md` §4.6.

### DP-51 — Statut d'échéance et soldes jamais stockés

- **Date** : 30/10/2026
- **Contexte** : R-034 interdit tout cache obsolète. Le statut d'une échéance dépend de la date du jour (R-095, DP-13, DP-27) : un statut stocké serait faux le lendemain sans qu'aucune écriture n'ait eu lieu. Les écritures d'argent sont en ajout seul (DP-47) : on ne peut pas y mettre à jour un solde.
- **Décision** : le statut d'une échéance (à venir, due, partielle, payée, en retard), son ancienneté et son reste dû ne sont jamais stockés. Les montants (dû, affecté, reste) sont dérivés par des vues ; le statut est calculé à la lecture par une seule fonction du crate `domain`. Une table de synthèse éventuelle serait une projection recalculable, jamais la source de vérité (choix de forme : décision ouverte n° 33).
- **Alternatives écartées** : colonnes de solde mises à jour par déclencheur (contraire à DP-47, deux vérités) ; statut stocké et recalculé par tâche (faux entre deux exécutions).
- **Conséquences** : `DATABASE_SCHEMA.md` §5. Les vues `v_installment_balances`, `v_payment_credit` et `v_lease_credit` ne contiennent aucune règle de date.
- **Statut** : Prise (confirmée par Mimis le 30/10/2026)
- **Liens** : R-032, R-033, R-034, R-095, DP-13, DP-27, DP-47.

Les entrées DP-52 à DP-68 viennent de l'audit `AUDIT_COHERENCE_2.md` (30/10/2026). Mimis a tranché lui-même trois points (capacité, base de données, éditeur) et a validé en bloc les recommandations de l'audit pour tout le reste. Les points qui demandent encore une information de Mimis restent ouverts dans le tableau des décisions ouvertes.

### DP-52 — Pas d'enquête terrain avant le développement ; pilote avec données réelles

- **Date** : 30/10/2026
- **Contexte** : DP-07, DP-10, DP-28/DP-34 et DP-35 supposaient maquette, puis enquête terrain, puis Go, puis code. La stratégie a changé. La décision ouverte n° 19 la proposait.
- **Décision** : aucune enquête terrain ni seuil Go/No-Go avant le développement. Une première version (T1 puis T2, dossier §9.1) est construite et testée par des agences pilotes avec leurs vraies données. Ce qui manque s'ajoute pendant les essais (T3 et retours des pilotes). Les hypothèses H1 à H12 se valident pendant le pilote. La maquette est conservée comme support de démonstration. Le développement démarre par la tranche verticale (`ARCHITECTURE.md` §9.1) dès que les décisions bloquantes de `TESTING_QA.md` PP-32 sont enregistrées.
- **Alternatives écartées** : garder la phase 0 (retarde le premier retour réel) ; enquête allégée.
- **Conséquences** :
  - DP-07 (portée « avant tout code »), DP-10, DP-34 (et donc DP-28) et DP-35 sont remplacées. DP-09 et DP-31 (périmètre de la maquette) restent en vigueur. DP-11 reste prise : le prototype PDF devient le prototype (a) d'`ARCHITECTURE.md` §7.
  - Les seuils d'arrêt ou de continuation du pilote sont ceux de DP-53.
  - Les décisions n° 1 à 3, 7, 10, 12 et 15 perdent leur échéance « Phase 0 » (voir DP-57).
  - Écart assumé par rapport au dossier §17 (phase 0) et §26 n° 1.
- **Statut** : Prise
- **Liens** : décision n° 19, DP-07, DP-10, DP-11, DP-28, DP-34, DP-35.

### DP-53 — Critères de continuation, correction ou arrêt du pilote

- **Date** : 30/10/2026
- **Contexte** : DP-52 supprime le Go/No-Go de la phase 0. Il faut un critère de décision à la fin du pilote.
- **Décision** : reprendre les critères de sortie de la phase 4 du dossier (§17) : au moins 60 % des paiements des clients pilotes sont saisis dans Immo Suite dans les 48 heures ; au moins 2 clients acceptent de payer au tarif cible ; aucune anomalie S1 ouverte (`TESTING_QA.md` §11). L'échelle d'engagement de 0 à 4 (dossier §20.1) devient la grille d'observation du pilote, sans seuil.
- **Alternatives écartées** : seuils DP-34 (8/4/3/2 sur 15 entretiens), sans objet sans entretiens.
- **Conséquences** : `CONTEXTE.md` renvoie à cette entrée. Valeurs à confirmer avec le premier pilote ; une nouvelle entrée remplace celle-ci si elles changent.
- **Statut** : Prise
- **Liens** : DP-52, DP-34, dossier §17 (phase 4), §23.

### DP-54 — Capacité de développement : un développeur, 25 heures par semaine

- **Date** : 30/10/2026
- **Contexte** : le dossier (§17, H11) suppose un développeur à environ 25 heures par semaine ; six documents techniques supposaient deux personnes (décision n° 14).
- **Décision** : **un développeur, environ 25 heures par semaine**, assisté par Claude Code. Le porteur du projet est la seconde personne pour les accès partagés et le contact avec les agences.
- **Alternatives écartées** : deux développeurs (non retenu par Mimis).
- **Conséquences** :
  - Revue de code : relecture différée de 24 heures par l'auteur avec la liste de contrôle et la CI verte (`CODING_RULES.md` §8.3) ; un second avis externe est demandé pour les chemins sensibles et les migrations qui touchent une garde [personne à désigner].
  - Auto-test d'intrusion de deux jours et test de restauration : réalisés avec une personne externe désignée pour l'auto-test ; test de restauration par le développeur, observé par le porteur du projet.
  - Clés de chiffrement en deux exemplaires : développeur et porteur du projet. Accès d'urgence « à deux personnes » : développeur et porteur du projet.
  - Pas d'astreinte permanente (heures de couverture : décision n° 84, ouverte).
  - La roadmap fixe ses durées sur 25 heures par semaine. Les complexités du dossier (S, M, L) donnent 22 à 40 semaines-développeur pour T1 et T2 contre 14 à 19 semaines pour les phases du dossier : la roadmap arbitre le périmètre (T1 d'abord) plutôt que les délais.
- **Statut** : Prise
- **Liens** : décision n° 14, H11, dossier §17.

### DP-55 — PostgreSQL managé chez Neon

- **Date** : 30/10/2026
- **Contexte** : décisions n° 5, 21, 46 et 86 liées (hébergement, exploitation de PostgreSQL, sauvegarde logique sous RLS forcée, grille et coûts).
- **Décision** : la base de données est **PostgreSQL managé chez Neon** (option (a) de la décision n° 21). Décision conditionnelle sur la sauvegarde logique (n° 46) : option (a) (rôle de sauvegarde avec `BYPASSRLS`) si le prototype (b) la confirme sur Neon ; sinon option (b) (RLS activée mais non forcée).
- **Alternatives écartées** : PostgreSQL auto-hébergé ; hébergeur burkinabè (non évalué).
- **Conséquences** :
  - Points à vérifier au prototype (b) et avant la production [À vérifier] : (1) les rôles créés par la console ou l'API Neon sont membres de `neon_superuser`, qui porte `BYPASSRLS` pour les projets créés après août 2023 (documentation Neon) : un rôle `immo_backup` ne doit être créé que s'il permet `pg_dump` sous RLS forcée ; (2) la fenêtre d'historique pour la restauration à un instant donné dépend du plan Neon (de 6 heures à 30 jours) : l'objectif de 30 jours d'`ARCHITECTURE.md` §10.4 exige un plan qui l'offre, sinon seules les 30 copies logiques quotidiennes couvrent les erreurs découvertes tard ; (3) mise en veille automatique (scale to zero, 5 minutes par défaut) désactivée en production ; (4) région la plus proche de Ouagadougou disponible (Europe de l'Ouest, `eu-central-1` à confirmer) ; (5) connexion directe, non poolée, pour l'application au pilote (le contexte d'organisation est posé par transaction, compatible avec un pooler en mode transaction mais pas nécessaire) ; (6) version majeure 16 ou plus et extension `btree_gist` disponibles.
  - Neon est hébergé hors du Burkina Faso : la position CIL / juriste (n° 5, H12) reste à prendre (DP-62).
  - Restent ouverts (n° 5 en partie, n° 86) : serveur applicatif, stockage objet compatible S3, second fournisseur de sauvegardes (compte distinct de Neon), coûts. `DEPLOYMENT.md` §1.2 garde la grille pour ces trois ressources.
- **Statut** : Prise (n° 21) ; n° 46 prise sous la condition ci-dessus
- **Liens** : décisions n° 5, 21, 46, 86 ; `DATABASE_SCHEMA.md` §6.9.

### DP-56 — Pile technique de référence

- **Date** : 30/10/2026
- **Contexte** : 20 décisions techniques proposées par `ARCHITECTURE.md`, `DATABASE_SCHEMA.md`, `API_CONTRACTS.md`, `CODING_RULES.md` et `TESTING_QA.md` étaient traitées comme acquises sans entrée DP-xx (audit C2-005).
- **Décision** : les recommandations de ces documents sont retenues pour : n° 6 (front React + Vite + TypeScript, application monopage), 20 (Axum, SQLx, sessions serveur, file de tâches PostgreSQL, REST décrit par OpenAPI), 23 (monorepo), 30 (texte et `CHECK`), 31 (UUID v4), 32 (clés étrangères composites), 34 (écriture seule à deux barrières, 25 tables), 35 (bornes de 1 à 999 999 999), 41 (exclusion étendue aux baux terminés), 61 (contrat généré depuis le code), 62 (évolution additive, `X-Client-Version`), 64 (`row_version` et `If-Match` dès le pilote), 67 (`snake_case` anglais), 69 à 77, 79, 80 et 87 (qualité, tests et dépôt). Le préfixe des routes est `/api/` (sauf `/health`, `/ready`, `/s/…`, `/v/…`, `/console/…`).
- **Alternatives écartées** : voir chacune des décisions.
- **Conséquences** : `DATABASE_SCHEMA.md` ajoute `row_version` aux 11 tables modifiables et `sessions.reauth_until` (audit C2-027). Le moteur PDF (n° 7) reste ouvert jusqu'au prototype (a).
- **Statut** : Prise
- **Liens** : décisions n° 6, 20, 23, 30, 31, 32, 34, 35, 41, 61, 62, 64, 67, 69 à 77, 79, 80, 87.

### DP-57 — Premier segment de pilote, prix et ordre de construction de T2

- **Date** : 30/10/2026
- **Contexte** : les décisions n° 2, 3, 7, 10, 12 et 15 étaient rattachées à la phase 0 supprimée.
- **Décision** : le pilote principal est une **agence** (reddition de comptes, import, console : début à la fin de T2), plus un ou deux propriétaires directs pour valider le plan léger (recommandation du dossier §26 n° 2). Les prix se testent pendant le pilote (méthode « trop cher / bon marché / limite »). Nouvelles échéances : n° 2 fermée ; n° 3 et 12 et 15 « Avant le premier pilote » ; n° 7 « Prototype (a), tranche verticale » ; n° 10 « Avant la tranche 5 (T2) ». Ordre de construction de T2 : mandats et compte rendu, puis import et console (`ARCHITECTURE.md` §9.2) ; un import par ligne de commande réservé à HABEK est livré dès la tranche 2.
- **Alternatives écartées** : pilote propriétaire direct en premier (démarrage à la fin de T1) ; ordre du dossier §18 (import avant mandats).
- **Conséquences** : écart assumé par rapport au dossier §18. Prototype PDF de 2 à 3 jours (`ARCHITECTURE.md` §7).
- **Statut** : Prise
- **Liens** : décisions n° 2, 3, 7, 10, 12, 15 ; DP-52.

### DP-58 — Règles de T1 : échéances, promesse tenue, doublon de référence, taux de recouvrement, avenant, téléphone

- **Date** : 30/10/2026
- **Contexte** : sept règles de T1 n'étaient écrites dans aucun document propriétaire (audit C2-006). Les valeurs ci-dessous viennent des propositions de `ARCHITECTURE.md` §4.8 et de `DATABASE_SCHEMA.md` §11.1, ou de la maquette (taux de recouvrement, `maquette/NOTES.md`).
- **Décision** : R-098 (jour d'échéance 29 à 31 : dernier jour du mois), R-099 (premier mois partiel : loyer entier), R-100 (bail à date de début passée : échéances générées à partir du mois d'activation), R-101 (promesse tenue : paiement au moins égal au montant promis, à la date promise ou avant), R-102 (doublon de référence externe : avertissement qui bloque jusqu'à confirmation explicite ; une référence d'un paiement contre-passé est réutilisable), R-103 (taux de recouvrement : encaissé du mois divisé par attendu du mois, en pourcentage entier), R-104 (avenant : s'applique aux échéances postérieures à sa date d'effet et à sa date de saisie), R-017 précisée (unicité du téléphone parmi les fiches non archivées, pas de partage de numéro au pilote).
- **Alternatives écartées** : voir `DATABASE_SCHEMA.md` §11.1 et `ARCHITECTURE.md` Questions ouvertes.
- **Conséquences** : `DOMAINE.md` (règles R-098 à R-104, exemples E-39 à E-44). Chaque règle est marquée [à confirmer avec l'agence pilote à la découverte] jusqu'au premier pilote.
- **Statut** : Prise
- **Liens** : F07, F08, F09, F11, F12 ; décisions n° 43 ; audit C2-006.

### DP-59 — Règles du compte rendu propriétaire (T2) et règle de reddition R-071

- **Date** : 30/10/2026
- **Contexte** : `DATABASE_SCHEMA.md` §11.1 n° 2, 3, 4, 8, 10, 12 et la décision n° 10 étaient ouverts.
- **Décision** : R-071 confirmée comme règle par défaut (date d'encaissement) en attendant l'agence pilote. R-105 (charges encaissées sous un mandat à base « loyer » : reversées au mandant, comprises dans le net ; seule la commission suit la base), R-106 (base de commission négative : production refusée ; net négatif : aucun reversement), R-107 (dépenses d'un compte rendu : imputables, datées dans la période, statut « terminée »), R-108 (un seul reversement, du montant exact du net), R-109 (mandat clôturé en cours de mois : le compte rendu retient les lots couverts par un mandat qui recouvre la période ; un lot sans mandat valide avec un encaissement dans la période bloque la production, R-075).
- **Alternatives écartées** : base de reddition paramétrable par mandat (règle d'or 12).
- **Conséquences** : `DOMAINE.md` (R-105 à R-109, exemples E-45 et E-46). Marqueur [à confirmer avec l'agence pilote à la découverte].
- **Statut** : Prise
- **Liens** : F13, décision n° 10, DP-14, DP-16, DP-17 ; audit C2-008.

### DP-60 — Exceptions à R-005 et R-007 : annulation d'import et remise à zéro en phase setup

- **Date** : 30/10/2026
- **Contexte** : `DATABASE_SCHEMA.md` prévoit `cancel_import_batch` et `purge_org_setup`, qui contredisent à la lettre R-005 et R-007 (décisions n° 25 et 40).
- **Décision** : acceptées. Une organisation en phase `setup` peut être remise à zéro par HABEK, avec l'accord écrit de l'agence, une sauvegarde préalable, deux personnes (développeur et porteur du projet), une trace hors de l'organisation. Un import non utilisé est annulable (R-080). Après la bascule en `live`, repartir de zéro est une nouvelle organisation.
- **Alternatives écartées** : aucune purge (l'agence qui s'est trompée à l'import repart d'une nouvelle organisation).
- **Conséquences** : `DOMAINE.md` : R-005 et R-007 reçoivent leurs exceptions, R-110 (cycle de vie), E-47. `SECURITY.md` §6.2 corrigé.
- **Statut** : Prise
- **Liens** : décisions n° 25, 40 ; R-005, R-007, R-080 ; audit C2-009.

### DP-61 — Cycle setup puis live, séries de numérotation et année de la séquence

- **Date** : 30/10/2026
- **Contexte** : R-050 parlait d'un compteur par organisation et par année ; `DATABASE_SCHEMA.md` ajoute une série d'essai (décision n° 36) ; l'année de la séquence (DP-46) était à confirmer.
- **Décision** : en phase `setup`, les quittances sont de série `trial` (préfixe `ESSAI-`, filigrane « ESSAI ») ; à la bascule en `live`, la série `official` démarre à 000001. L'année de la séquence est l'année de la date d'émission, au fuseau de l'organisation.
- **Alternatives écartées** : série unique et purge des compteurs ; année de la date du paiement.
- **Conséquences** : `DOMAINE.md` : R-050 reformulée, R-110, E-48. `ARCHITECTURE.md` §4.4. Restent ouverts : valeur des quittances d'essai pour le locataire, et date de bascule par rapport au début de mois (`DEPLOYMENT.md` Questions ouvertes n° 7 et 8).
- **Statut** : Prise
- **Liens** : DP-15, DP-37, DP-46 ; décision n° 36 ; audit C2-010.

### DP-62 — Avis juridique avant le premier pilote

- **Date** : 30/10/2026
- **Contexte** : cinq sujets juridiques conditionnent le pilote ; la phase 0–1 qui devait les traiter n'existe plus.
- **Décision** : un avis juridique ponctuel est demandé **avant le premier pilote** sur : (1) mentions de la quittance et formulation du montant en lettres ; (2) plafond du dépôt de garantie (R-024) et bail commercial ; (3) déclaration ou autorisation CIL et hébergement hors du pays (Neon, DP-55) ; (4) durées de conservation et départ d'une agence (`SECURITY.md` §6.3 et §8.4) ; (5) engagement de confidentialité (`SECURITY.md` §8.6). Règle de repli : si l'avis n'est pas obtenu, le porteur du projet accepte le risque par écrit (modèle de `SECURITY.md` §8.6). Personne chargée et date cible : [à fixer par Mimis].
- **Alternatives écartées** : aucune revue (non retenu).
- **Conséquences** : le marqueur `[À VÉRIFIER AVEC UN JURISTE]` est utilisé partout (audit C2-049). Les décisions n° 12, 56, 57 et 94 restent ouvertes jusqu'à l'avis.
- **Statut** : Prise (plan) ; contenu de l'avis ouvert
- **Liens** : décisions n° 5, 12, 56, 57, 94 ; H12 ; audit C2-012.

### DP-63 — Éditeur

- **Date** : 30/10/2026
- **Contexte** : `CONTEXTE.md` hésitait entre « HABEK Software » et « HABEK Technology Company ».
- **Décision** : l'éditeur est **HABEK Technology Company**. Le nom du produit reste « Immo Suite » (provisoire) ; le domaine n'est pas choisi.
- **Alternatives écartées** : « HABEK Software ».
- **Conséquences** : `CONTEXTE.md`, engagement de confidentialité et accord de pilote utilisent ce nom. La décision n° 13 (nom, marque, domaine) reste ouverte.
- **Statut** : Prise
- **Liens** : décision n° 13 ; audit C2-014.

### DP-64 — Quittance : contenu, format et vérification

- **Date** : 30/10/2026
- **Contexte** : six choix sur la quittance n'étaient pas tranchés (audit C2-015).
- **Décision** : orthographe traditionnelle du montant en lettres, formulation « francs CFA » ; format par défaut A5 (A4 disponible), les deux formats rendus à chaque paiement ; quittance rectificative sans effet sur le montant : hors pilote (route réservée, 404) ; code de contrôle et page de vérification (décision n° 55) : après le pilote. Les mentions légales complémentaires restent `[À VÉRIFIER AVEC UN JURISTE]` (DP-62).
- **Alternatives écartées** : voir `API_CONTRACTS.md` Questions ouvertes.
- **Conséquences** : `DOMAINE.md` R-053 et R-056 ; `UI_UX_DESIGN.md` section Quittance.
- **Statut** : Prise
- **Liens** : décisions n° 12, 55 ; R-052, R-053, R-056.

### DP-65 — Plans et paliers commerciaux

- **Date** : 30/10/2026
- **Contexte** : le dossier parle de deux plans et d'une grille de quatre offres (Essentiel, Pro, Agence, Agence Plus).
- **Décision** : le logiciel n'a que deux plans, « Propriétaire » et « Agence ». Les paliers de prix sont commerciaux, sans effet dans le logiciel au pilote ; aucun plafond de lots ni d'utilisateurs n'est appliqué par le code.
- **Alternatives écartées** : plafonds dans `org_settings` (règle d'or 12).
- **Conséquences** : `CONTEXTE.md` ; la facturation reste manuelle (F31 hors MVP).
- **Statut** : Prise
- **Liens** : DP-01 ; F01, F31.

### DP-66 — Compléments à la matrice des permissions

- **Date** : 30/10/2026
- **Contexte** : la matrice du dossier §11.2 laisse des cases indéfinies (audit C2-030).
- **Décision** : le collaborateur du plan Propriétaire reprend les droits de `API_CONTRACTS.md` §4.6 (colonne Col) : enregistrer les paiements (droit réglable), relancer, voir biens, lots, locataires et impayés en lecture, demander une correction, émettre et réimprimer une quittance ; pas d'import, d'export, de dépense, de paramètres ni de journal d'audit. L'agent terrain ne propose pas de dépense au pilote (la route reste refusée). Les entrées de menu non définies (« ? ») sont masquées (choix de `maquette/NOTES.md`).
- **Alternatives écartées** : refus par défaut sans le préciser.
- **Conséquences** : `CONTEXTE.md`, `SECURITY.md` §4.1, `UI_UX_DESIGN.md` (colonne Collaborateur).
- **Statut** : Prise
- **Liens** : F02 ; `SECURITY.md` §4.2.

### DP-67 — Langue des identifiants de code

- **Date** : 30/10/2026
- **Contexte** : la décision n° 18 était ouverte alors que `ARCHITECTURE.md`, `DATABASE_SCHEMA.md`, `API_CONTRACTS.md` et `CODING_RULES.md` suivent déjà la convention.
- **Décision** : interface et documentation en français ; identifiants de code (variables, fonctions, tables, colonnes, routes, codes d'erreur, valeurs d'énumération) en anglais.
- **Alternatives écartées** : tout en français ; tout en anglais.
- **Conséquences** : `CLAUDE.md` ne la présente plus comme « proposée ». La maquette garde ses mots français (jetable).
- **Statut** : Prise
- **Liens** : décision n° 18.

### DP-68 — Authentification et accès au pilote

- **Date** : 30/10/2026
- **Contexte** : `ARCHITECTURE.md` et `SECURITY.md` divergeaient (décisions n° 27, 50, 51, 60).
- **Décision** : TOTP obligatoire pour tous les comptes de la console HABEK et pour le gérant (ou propriétaire) **avant la bascule de son organisation en `live`** ; inscription publique fermée au pilote, organisations créées par la console ; aucun e-mail ni SMS au premier pilote ; invitations par lien à usage unique de 7 jours ; réinitialisation du mot de passe par lien à usage unique de 15 minutes généré par le gérant ou le support.
- **Alternatives écartées** : TOTP du gérant seulement proposé ; code envoyé par SMS ou e-mail.
- **Conséquences** : écarts assumés par rapport au dossier F01 (inscription publique, code envoyé) et §12 (TOTP du gérant, option). `ARCHITECTURE.md` §3.8 et §13.2, `UI_UX_DESIGN.md` S01 et S17, `SECURITY.md` §10 et `TESTING_QA.md` PP-31 alignés.
- **Statut** : Prise
- **Liens** : décisions n° 27, 50, 51, 60 ; F01, F02 ; audit C2-013, C2-028.

### DP-69 — Neon en plan gratuit pendant le pilote ; objectif de 30 jours de restauration suspendu

- **Date** : 31/10/2026
- **Contexte** : DP-55 choisit Neon. L'étape 00 a relevé que le projet de prototypes est en plan gratuit (`free_v3`) : historique de restauration de 6 heures, mise en veille après 5 minutes non désactivable. Les objectifs de sauvegarde (décision n° 24, `ARCHITECTURE.md` §10.4, `DEPLOYMENT.md` §4.1) demandent 30 jours, ce qui suppose le plan Scale (environ 41 $ par mois en continu à 0,25 CU, tarifs neon.com du 31/10/2026 [À vérifier]) ; le plan Launch offre 7 jours (environ 19 $ par mois). Le pilote ne rapporte rien et la demande n'est pas démontrée.
- **Décision** : tout le logiciel (prototypes, essai, production du pilote) tourne sur **Neon en plan gratuit** jusqu'à l'arrivée d'abonnés payants. L'objectif de **30 jours** de restauration à un instant donné est **suspendu** ; la fenêtre réelle est de 6 heures. Le dossier est rouvert quand le logiciel a des abonnés payants.
- **Alternatives écartées** : plan Launch (7 jours) ; plan Scale (30 jours).
- **Conséquences** :
  - Limites du plan gratuit à surveiller [À vérifier à chaque palier] : 1 Go de stockage par projet (écritures bloquées au-delà), 100 CU-heures par projet et par mois (calcul suspendu jusqu'à la période suivante), 5 Go de transfert sortant, 10 branches, 1 instantané manuel, calcul plafonné à 2 CU, première requête plus lente après 5 minutes d'inactivité.
  - Les 30 jours de restauration à un instant donné ne sont plus fournis par Neon. La protection des données repose sur les copies logiques chiffrées chez un second fournisseur (`ARCHITECTURE.md` §10.4) ; leur fréquence devient le plafond de perte de données (une copie par jour : jusqu'à 24 heures) [À valider à l'étape 01, avec la décision n° 24]. Le test de restauration de l'étape 01 se fait à partir d'une copie logique.
  - Écarts à corriger dans les documents en fin d'étape 00 : `DEPLOYMENT.md` §4.1 et §10, `ARCHITECTURE.md` §10.4, `SECURITY.md` §5.3, `TESTING_QA.md` (objectifs de reprise), `ROADMAP.md` (jalon J0).
  - Bascule `live` avec les données réelles d'une agence : le risque de perte entre deux copies et la suspension du calcul en fin de quota sont à faire accepter par écrit par Mimis avant la bascule [À valider]. Contrat de l'hébergeur et plan gratuit : [À vérifier].
  - Réouverture : dès les premiers abonnés payants, comparer Launch et Scale ; décision n° 24 à fixer alors.
- **Statut** : Prise (Mimis, 31/10/2026)
- **Liens** : DP-55 ; décisions n° 5, 21, 24, 46, 86 ; `roadmap/etape-00-decisions-et-prototypes.md` ; C-02.

### DP-70 — Tous les fichiers dans le stockage objet, jamais dans PostgreSQL

- **Date** : 31/10/2026
- **Contexte** : diagnostic du 31/10/2026 : aucun mécanisme de stockage n'existe encore ; `ARCHITECTURE.md` §3.10 recommande le stockage objet compatible S3 [À valider] ; `DATABASE_SCHEMA.md` prévoit toutefois une pièce d'identité chiffrée dans une colonne `bytea` (`tenants.id_document_enc`).
- **Décision** : tout fichier (PDF émis, logos, signatures, documents T3, fichier d'origine d'un import, pièce d'identité éventuelle) est dans un **stockage objet privé compatible S3**, jamais en colonne PostgreSQL ni en objet volumineux. La base ne garde que les métadonnées : clé de stockage, SHA-256, taille, type, date. Interface `FileStore`, implémentation sur disque réservée au développement local.
- **Alternatives écartées** : fichiers en `bytea` ; disque du serveur en production.
- **Conséquences** :
  - La base Neon ne contient aucun fichier : elle reste dans la limite de 1 Go du plan gratuit (DP-69).
  - `tenants.id_document_enc` n'a plus d'objet : une pièce d'identité éventuelle passe par `documents` (`doc_type = 'id_document'`), chiffrée par l'application avant l'envoi [À valider à l'étape 05]. Le champ est désactivé au pilote : aucune migration à écrire avant. `import_rows.raw` (données de lignes, pas un fichier) reste en base et est purgé après confirmation.
  - Reste ouvert : le fournisseur du stockage objet et du second fournisseur de copies (décisions n° 5 et 86, C-02), à trancher avant l'étape 01.
- **Statut** : Prise (Mimis, 31/10/2026)
- **Liens** : décisions n° 5, 86 ; `ARCHITECTURE.md` §3.10 ; `DATABASE_SCHEMA.md` (`receipt_files`, `documents`, `import_batches`).

### DP-71 — Les PDF émis sont conservés : le fichier original fait foi

- **Date** : 31/10/2026
- **Contexte** : `ARCHITECTURE.md` §3.10 écrit que le PDF « peut toujours être reproduit à l'identique depuis l'instantané ». Six mois après l'émission, le propriétaire peut avoir changé des informations, le modèle de quittance avoir été modifié, le logiciel avoir évolué : un PDF régénéré ne serait pas le document remis au locataire. Une quittance est une preuve (`SECURITY.md` §2).
- **Décision** : le PDF **effectivement émis** est enregistré dans le stockage objet ; sa référence (clé de stockage), sa **date de génération** (`rendered_at`) et son **SHA-256** sont enregistrés en base (`receipt_files`). Il est **immuable** et n'est jamais régénéré pour le remplacer.
- **Alternatives écartées** : régénérer le PDF à la demande depuis l'instantané.
- **Conséquences** :
  - Garde-fous : (1) `receipt_files` reste en ajout seul (aucun UPDATE ni DELETE, déclencheur déjà prévu) ; (2) l'objet est écrit « créer seulement » : jamais réécrit, et les identifiants de l'application n'ont ni droit d'écrasement ni droit de suppression sur ce préfixe ; (3) le SHA-256 de l'objet est recalculé et comparé à celui de la base à chaque téléchargement et par un contrôle nocturne, tout écart est une alerte [gravité à fixer, `TESTING_QA.md` §11] ; (4) versionnage du seau, verrouillage d'objet si le fournisseur le permet (critère de choix du fournisseur), réplication quotidienne chez un second fournisseur (déjà prévue) ; (5) la version du gabarit et du moteur (`renderer_version`) est enregistrée avec le fichier.
  - `ARCHITECTURE.md` §3.10, dernière puce, est à corriger : la perte d'un PDF émis est une perte de preuve, pas seulement un fichier à refaire. L'instantané permet de produire une **copie** clairement indiquée comme telle, jamais l'original [règle à spécifier à l'étape 08 et à ajouter aux questions ouvertes de `DOMAINE.md`, ne pas l'inventer]. `TESTING_QA.md` §6 « Reproductibilité » devient un test de non-régression du moteur et ne prouve plus rien sur les documents émis.
  - Points à trancher aux étapes 08 et 09 [À valider] : la quittance **annulée** ou **rectificative** porte un filigrane (`UI_UX_DESIGN.md`) ; l'original reste intact et la version filigranée est un second fichier distinct ; l'unicité de `receipt_files` inclut `renderer_version`, ce qui doit être revue pour qu'un seul fichier soit « l'original ».
  - Prototype (a) : mesure aussi la reproductibilité (même instantané, même empreinte) ; le test « créer seulement » sur le stockage choisi se fait à l'étape 01.
  - Durée de conservation des PDF émis et départ d'une agence (n° 94) : [À VÉRIFIER AVEC UN JURISTE].
- **Statut** : Prise (Mimis, 31/10/2026)
- **Liens** : DP-64 ; R-049, R-056 ; F10 ; DP-70 ; `DATABASE_SCHEMA.md` (`receipt_files`).

### DP-72 — Pilote en deux temps et capacité

- **Date** : 31/10/2026
- **Contexte** : décisions n° 96, 97 et 103 (`ROADMAP.md` §1 et §5), proposées par la roadmap.
- **Décision** : (1) **pilote en deux temps** : lot 1 (T1 sans relances, plus moteur d'import et console en ligne de commande) puis lot 2 (relances, mandats, compte rendu, assistant d'import) avant la bascule `live` ; `TESTING_QA.md` PP-05 à PP-07 se lisent « pour le périmètre livré » ; **aucune bascule `live` avant l'étape 17** (n° 103). Écart assumé à DP-57. (2) **Capacité** : DP-54 est maintenue (un développeur, 25 heures par semaine, assisté par Claude Code) ; `ROADMAP.md` est recalé avec la vitesse mesurée à la fin des étapes 00 et 01.
- **Alternatives écartées** : ordre de DP-57 (pilote à la fin de T2) ; deux développeurs avec revue croisée.
- **Conséquences** : `ROADMAP.md` passe de « proposition » à plan retenu, sauf la grille de dates (recalage). Les décisions n° 98 à 102 restent ouvertes.
- **Statut** : Prise (Mimis, 31/10/2026)
- **Liens** : décisions n° 14, 96, 97, 103 ; DP-54, DP-57.

### DP-73 — Racine du dépôt Git

- **Date** : 31/10/2026
- **Contexte** : `CODING_RULES.md` question 4 : dossiers marketing et `.venv` dans la racine actuelle.
- **Décision** : le dépôt Git est créé dans `C:\Immo Suite\Immo Suite` ; un `.gitignore` exclut `film/`, `flyer/`, `infographie/`, `site/`, `logo/` (le logo nécessaire est copié en `templates/assets/`), tout `.venv`, `node_modules/`, `target/`, `maquette/_captures/` et les fichiers d'environnement locaux. Rien n'est supprimé.
- **Alternatives écartées** : dépôt dans un sous-dossier propre ne contenant que le code.
- **Conséquences** : le dossier de cadrage `.docx` est versé tel quel et ne doit jamais être modifié. L'hébergeur du code et de la CI reste à choisir (étape 01).
- **Statut** : Prise (Mimis, 31/10/2026)
- **Liens** : `CODING_RULES.md` §1.1 et question 4.

### DP-74 — Outils du front, observation et chaîne d'intégration

- **Date** : 31/10/2026
- **Décision** : confirmées telles que proposées par les documents : n° 28 (événements en base, sans traceur tiers ni cookie publicitaire, bouton « Signaler un problème »), n° 66 (erreurs `problem+json`, codes stables, textes centralisés dans le front), n° 71 (pas de bibliothèque de cache au pilote), n° 72 (ESLint avec `typescript-eslint` et `jsx-a11y`, Prettier), n° 78 (Playwright), n° 82 (CI par étapes, PR sous 15 minutes, pas de nouvel essai sur les tests d'argent et d'isolation, pas de seuil de couverture au pilote).
- **Alternatives écartées** : service d'observation tiers ; bibliothèque de cache ; Biome ; Cypress ; CI en une seule étape.
- **Statut** : Prise (Mimis, 31/10/2026)
- **Liens** : décisions n° 28, 66, 71, 72, 78, 82.

### DP-75 — Mesure du réseau à Ouagadougou (prototype e)

- **Date** : 31/10/2026
- **Contexte** : prototype (e), `ARCHITECTURE.md` §7. Aucune mesure ne peut se faire sans un serveur d'essai et une application assez avancée.
- **Décision** : **Mimis réalise les mesures** à Ouagadougou, avec **deux opérateurs mobiles, en 3G et en 4G, dans trois zones de la ville**, pour évaluer Immo Suite dans les conditions réelles de connexion des agences. Le **protocole est rédigé à l'étape 00** ; les **mesures sont reportées** au moment où l'application permet des tests réels. Calendrier proposé en deux vagues [À valider] : vague 1 à l'étape 01 ou 02, quand l'essai existe (aller-retour vers l'hébergement, front de démonstration de 150 Ko, réveil de la base après 5 minutes d'inactivité, DP-69) ; vague 2 à partir de J2 (parcours Encaisser réel).
- **Alternatives écartées** : mesure par un tiers ; mesure à l'étape 00.
- **Conséquences** : la région du serveur applicatif et les budgets de poids (`ARCHITECTURE.md` §12) restent provisoires jusqu'à la vague 1. Le téléphone Android d'entrée de gamme (C-13) est à fournir avant la vague 1.
- **Statut** : Prise (acteur, plan de mesure) ; calendrier en deux vagues [À valider]
- **Liens** : `ARCHITECTURE.md` §7 (e), §11 ; DP-69 ; C-13 ; `TESTING_QA.md` §8.1.

### DP-76 — Moteur PDF : Typst en bibliothèque (décision n° 7)

- **Date** : 31/10/2026
- **Contexte** : prototype (a) de l'étape 00 (`spikes/pdf/NOTE.md` §8). Trois moteurs comparés sur les 20 cas types en A4 et A5 (40 documents chacun) : Chrome 156 (un processus par rendu, et maintenu ouvert), tracé natif Rust (`pdf-writer`), Typst 0.15.1 (ligne de commande et bibliothèque). Les trois tiennent les 40 documents sur une page, avec le texte égal à l'instantané et sans couleur.
- **Décision** : le moteur est **Typst 0.15.x compilé comme bibliothèque Rust**, derrière l'interface `PdfRenderer` ; la version est **figée** ; polices **IBM Plex Sans statiques** (Regular et Bold, licence OFL) embarquées ; gabarits dans `templates/`. Le montant en lettres est produit par le crate `domain`, jamais par le moteur. L'impression papier sur laser noir et blanc depuis le PDF (A4 et A5) a été **validée par Mimis le 31/10/2026**.
- **Alternatives écartées** : Chrome sans interface (p95 de 1 228 ms en processus par rendu, au-dessus de l'objectif de 1 s ; 480 Mo de mémoire ; polices en Type3 ; PDF non reproductibles à cause des dates ; reste le repli) ; tracé natif (le plus léger, 7 Mo, mais environ le double d'effort et tout changement de mise en page demande du code Rust).
- **Conséquences** :
  - Mesures (PC i7-10510U, A4) : 3 ms par rendu et 17 Mo en bibliothèque, PDF de 31 Ko, même SHA-256 sur deux rendus ; binaire de 75,6 Mo. Valeurs absolues à remesurer sur le serveur d'essai (étape 01).
  - L'API de Typst change d'une version à l'autre (trois différences ont bloqué la première compilation) : figer la version et la monter volontairement. Licence de Typst (Apache-2.0) [À vérifier] ; licence de la police : OFL, texte joint.
  - `ARCHITECTURE.md` §6 : moteur choisi. `TESTING_QA.md` §6 « Reproductibilité » devient un test de non-régression du moteur (DP-71). La décision n° 83 (méthode des tests visuels) reste **ouverte** : le prototype a utilisé le texte extrait et une planche d'images relue à la main ; l'image de référence approuvée se décide à l'étape 08.
  - Défaut connu du gabarit du prototype : espacements irréguliers dans l'en-tête (blocs de mesure de la troncature) ; à reprendre à l'étape 08.
  - `API_CONTRACTS.md` question 7 : réponse **provisoire « à partir du PDF »**, seule option imprimée et validée ; l'impression depuis un aperçu HTML n'a pas été comparée [À confirmer à l'étape 08].
- **Statut** : Prise (Mimis, 31/10/2026)
- **Liens** : décisions n° 7, 83 ; DP-64, DP-70, DP-71 ; `spikes/pdf/NOTE.md`.

### DP-77 — Contexte d'organisation avec un pool de connexions : `OrgTx` seul mécanisme

- **Date** : 31/10/2026
- **Contexte** : prototype (b) (`spikes/isolation-pool/NOTE.md` §8), sur PostgreSQL local et sur Neon (connexion directe, plan gratuit), avec des rôles créés en SQL sans `BYPASSRLS`.
- **Décision** : tout accès à la base d'une organisation passe par `OrgTx::begin(&pool, org_id, user_id)`, qui pose `set_config('app.org_id', …, true)` et `set_config('app.user_id', …, true)` en première instruction d'une transaction explicite. C'est le **seul** mécanisme. Sont interdits, par règle de revue et de lint : `&PgPool` hors du module `platform`, `SET app.org_id` et `set_config(…, false)` (variable de session), `SET LOCAL` hors transaction. Les garde-fous `after_release` et `before_acquire` sont **facultatifs** : ils n'étaient pas nécessaires pour l'étanchéité mesurée.
- **Alternatives écartées** : variable de session (fuite mesurée) ; un pool par organisation ; confiance dans le filtre `WHERE org_id` seul.
- **Conséquences** :
  - Preuve : 10 000 itérations, 50 tâches, pool de 2 connexions, **0 lecture croisée** (local en 2,4 s, Neon en 4 467 s) ; la variante de session lit 25 100 lignes étrangères dans 502 itérations ; `SET LOCAL` hors transaction est sans effet et laisse fuir la valeur précédente ; sans contexte, 0 ligne et `INSERT` refusé (`42501`) ; la clé composite interdit un bail de A sur un lot de B (`23503`).
  - Surcoût : en local 0,03 ms ; sur Neon, `OrgTx` ajoute **4 allers-retours** (558 ms à 138,5 ms l'aller-retour), les garde-fous 1,2 de plus. Un serveur hébergé dans la région de la base paiera quelques millisecondes : **héberger le serveur applicatif dans la région de la base** (décision n° 5 et 86 restent ouvertes pour le choix).
  - À l'étape 01, à mesurer sur l'essai : envoyer `BEGIN` et `set_config` en un seul message ; réglage de la vérification de connexion de `sqlx` à la prise (un `SELECT 1` par le pool coûte 3,1 allers-retours) ; garder ou non les garde-fous.
  - `find_memberships` (adhésions avant contexte) accepte n'importe quel identifiant d'utilisateur : l'appelant doit être la couche `identity` (limite déjà notée G-20).
  - Un test de pool sous concurrence (50 tâches, 2 connexions) et un test négatif attendu en échec rejoignent la CI (`ARCHITECTURE.md` §5.4).
- **Statut** : Prise (Mimis, 31/10/2026)
- **Liens** : DP-55, DP-67 ; `ARCHITECTURE.md` §5.3 et §5.4 ; `DATABASE_SCHEMA.md` §6.

### DP-78 — Sauvegarde logique sous RLS forcée (décision n° 46) : confirmée

- **Date** : 31/10/2026
- **Contexte** : DP-55 prenait la décision n° 46 « sous condition » : rôle de sauvegarde avec `BYPASSRLS` si Neon le permet, sinon RLS non forcée. Prototype (b), §8.4 et §8.5.
- **Décision** : la condition est **remplie**. Rôle `immo_backup` avec `BYPASSRLS`, **créé par SQL**, utilisé seulement par le travail de sauvegarde et de restauration ; **RLS forcée conservée** ; pas de repli « RLS non forcée ». `immo_app` et `immo_platform` restent sans `BYPASSRLS`, non propriétaires.
- **Alternatives écartées** : RLS non forcée ; rôle sans `BYPASSRLS` avec une politique de lecture totale (échec mesuré : `pg_dump` fixe `row_security = off` et refuse toute table protégée, quelle que soit la politique) ; sauvegarde physique seule (la copie chez un second fournisseur doit être lisible sans le premier).
- **Conséquences** :
  - Mesuré sur Neon : `pg_dump -Fc` avec `immo_backup` : code 0, comptes identiques par table et par organisation ; `immo_owner` et `immo_app` : échec « query would be affected by row-level security policy » ; `pg_restore` vers un second projet : code 0 en 18,4 s, comptes identiques, RLS forcée conservée ; restauration à un instant donné : branche à T0 créée en 1,9 s, 100 lignes sur 150.
  - **Règle de migration** : chaque nouvelle table reçoit `GRANT SELECT … TO immo_backup` (`BYPASSRLS` ne donne aucun privilège de table : `pg_dump` échoue sans), vérifié par un test ; tout rôle cité par une politique doit exister avant une restauration.
  - Particularités de Neon : `neondb_owner` ne peut pas exécuter `ALTER ROLE … NOSUPERUSER` ; il ne peut accorder de droits que sur ce qu'il possède (tout `GRANT` de table passe par `immo_owner`) ; un rôle ne se supprime pas tant que des privilèges en dépendent.
  - Restauration à un instant donné : le CLI Neon crée une branche à un instant **ou** à partir d'une branche, pas les deux ; pour une branche autre que la branche par défaut, utiliser l'API (`parent_id` et `parent_timestamp`). Fenêtre de 6 heures au plan gratuit (DP-69). À consigner dans `DEPLOYMENT.md`.
  - CU-heures consommées par les prototypes : non relevées, à lire dans la console Neon.
- **Statut** : Prise (Mimis, 31/10/2026)
- **Liens** : DP-55, DP-69 ; décisions n° 46, 5, 21, 86 ; `DATABASE_SCHEMA.md` §6.9 ; `DEPLOYMENT.md` §5.

### DP-79 — Numérotation des quittances et idempotence : mécanisme confirmé

- **Date** : 31/10/2026
- **Contexte** : prototype (c) (`spikes/numerotation/NOTE.md` §8), configuration locale et Neon (branche `spike-c`), 19 110 à 19 113 quittances par configuration.
- **Décision** : `ARCHITECTURE.md` §4.4 et §4.5 sont **confirmés** : compteur `INSERT … ON CONFLICT DO UPDATE … RETURNING` dans la transaction de validation, verrou consultatif du bail pris **avant**, compteur pris **en dernier avant** la quittance, clé d'idempotence insérée **en premier** (`ON CONFLICT DO NOTHING`) puis mise à jour avec la réponse. Ordre des verrous : clé d'idempotence, verrou du bail, compteur.
- **Alternatives écartées** : séquence PostgreSQL (trous, ordre) ; clé d'idempotence insérée à la fin (non essayée).
- **Conséquences** :
  - Mesures : 0 trou, 0 doublon, 0 violation d'ordre, compteur égal au maximum, 0 interblocage ; 10 % d'annulations forcées après l'attribution du numéro sans trace ; E-10, E-27 et le cas DP-61 reproduits avec une horloge injectée ; idempotence (i) à (iv) : 100/100 réponses identiques, un seul paiement, 422 sur contenu différent, réponse perdue rejouée, coupure avant validation suivie d'une création normale. Un contrôle négatif (quittance supprimée à la main) est détecté.
  - `immo_app` reçoit `UPDATE` sur `receipt_counters` et `idempotency_keys` : écart à « jamais d'`UPDATE` sur une écriture », assumé (ce ne sont pas des tables d'écriture seule) ; à reporter dans `DATABASE_SCHEMA.md` [À vérifier à l'étape 07].
  - La durée du verrou du compteur est proportionnelle au nombre d'allers-retours après le compteur (4 : quittance, audit, réponse d'idempotence, validation) : 552 ms avec 138,5 ms d'aller-retour. **Serveur dans la région de la base** (DP-77). Le p95 de 300 ms côté serveur (`TESTING_QA.md` §8.2) est tenu en local (41 ms) et à remesurer à l'étape 01.
- **Statut** : Prise (Mimis, 31/10/2026)
- **Liens** : DP-15, DP-61 ; R-050 ; E-10, E-11, E-27 ; `ARCHITECTURE.md` §4.4 et §4.5 ; `TESTING_QA.md` §2.3 et §3.

### DP-80 — Import : bibliothèques, limites et règles de lecture

- **Date** : 31/10/2026
- **Contexte** : prototype (d) (`spikes/import/NOTE.md` §8) : 31 fichiers synthétiques (référence de 200 lignes tirée de la maquette, 22 variantes, 8 fichiers piégés), résultat attendu écrit à la construction.
- **Décision** :
  - **Bibliothèques** : `calamine` pour XLSX (lecture seule, aucune formule évaluée), `csv` et `encoding_rs` pour CSV (BOM UTF-8, UTF-8 strict, sinon Windows-1252 ; séparateur détecté), `zip` pour inspecter l'archive avant lecture.
  - **Limites** : 5 Mo, 5 000 lignes, 200 entrées d'archive, 100 Mo décompressés, ratio de compression au plus de 100, 500 caractères par cellule. Refus : `.xls`, `.xlsm` et toute macro, faux XLSX (signature), archive bombe, tout `<!DOCTYPE` ou `<!ENTITY` dans une partie XML. Rapport exporté avec les cellules commençant par `=`, `+`, `-` ou `@` préfixées d'une apostrophe.
  - **Lecture** (règles proposées, validées avec la décision) : colonnes reconnues par titres synonymes sans casse ni accents ; en-tête cherché dans les 15 premières lignes ; lignes vides ignorées ; lignes numérotées dans l'ordre des lignes de données ; **dates** : JJ/MM/AAAA, AAAA-MM-JJ, JJ/MM/AA (20xx, avertissement), « 13 nov. 2026 », cellule date, numéro de série entre 2000 et 2100 (avertissement) ; l'ordre JJ/MM est confirmé par une date de la colonne dont le premier élément dépasse 12, sinon les dates dont les deux premiers éléments diffèrent et sont inférieurs ou égaux à 12 sont **signalées** (`import.date_ambiguous`) ; **montants** entiers : un séparateur suivi d'exactement trois chiffres est un séparateur de milliers, toute autre forme à séparateur (même `75000,00`) est refusée comme décimale, bornes 1 à 999 999 999 ; **téléphones** vers `+226XXXXXXXX` (8 chiffres, préfixes `+226`, `00226` ou `226`), deux numéros dans une cellule refusés, téléphone absent en avertissement ; **jour d'échéance** : absent = 1er du mois avec avertissement (R-031), 1 à 31 accepté (R-098), autre valeur refusée ; **usage** : habitation = {habitation, logement, appartement, maison, villa, studio, chambre, résidence}, commercial = {commercial, boutique, magasin, bureau, local commercial, commerce}, inconnu refusé ; dépôt de plus de deux mois en avertissement ; arriéré daté (R-035), sans date refusé ; doublons de téléphone proposés à la fusion (R-017, R-080) ; chevauchement refusé seulement entre deux baux **fermés** d'un même lot.
  - **Processus** : zéro ligne créée avant confirmation ; la confirmation est unique (verrou de ligne, seconde confirmation sans effet) ; l'annulation supprime locataires, lots, baux et lignes d'import du lot, laisse l'en-tête du lot avec le statut « annulé » comme trace, et est **refusée** s'il existe un paiement sur un bail du lot (R-080, E-21, DP-60).
- **Alternatives écartées** : lecture d'un XLSX par évaluation des formules ; refus de toute date ambiguë ; analyse sans plafond d'archive.
- **Conséquences** :
  - Mesuré : fichier de référence **197 créables, 3 erreurs aux lignes 14, 87 et 152, 2 paires de doublons** (33 et 118, 61 et 140) ; 31 fichiers sur 31 conformes ; 0 formule évaluée, fichier témoin de l'entité externe jamais lu ; 2 000 lignes en 41 ms, 5 000 en 108 ms, 29 Mo au plus ; mêmes garanties de processus en local et sur Neon (`spike-d`). Les limites de 5 000 lignes et 5 Mo pourraient être relevées sans risque de performance.
  - **Non tranché, voir `DOMAINE.md` « Questions ouvertes »** : dans le fichier de référence, 197 locataires se répartissent sur 36 lots (2 à 6 locataires simultanés par lot, sans date de sortie), ce qui heurte « un lot, un bail actif » (R-020, `DATABASE_SCHEMA.md` G-01) si « Maison/Ref » identifie un lot. Une colonne « Bien » est à ajouter au modèle si l'agence pilote le confirme ; à traiter avec les fichiers réels (C-09, étape 12).
  - Lignes et colonnes masquées : lues comme les autres (`calamine` n'expose pas l'attribut) ; non testés : cellules fusionnées au milieu des données, CSV de 5 000 lignes, XLSX de LibreOffice ou Google Sheets, limite exacte de 5 Mo.
- **Statut** : Prise (Mimis, 31/10/2026), sauf le constat des lots, renvoyé aux questions ouvertes
- **Liens** : R-017, R-020, R-031, R-035, R-080, R-098 ; E-20, E-21 ; DP-56, DP-60, DP-70 ; `TESTING_QA.md` §7 ; `SECURITY.md` §7.4.

### DP-81 — Montant en lettres : règles d'accord (complète DP-64)

- **Date** : 31/10/2026
- **Contexte** : DP-64 fixe l'orthographe traditionnelle et la formulation « francs CFA », sans préciser chaque accord. Prototype (a), `spikes/pdf/NOTE.md` §8.3 : fonction pure `amount_in_words`, 22 011 valeurs (de 0 à 2 000, frontières de `TESTING_QA.md` §6, 999 999 999, 20 000 valeurs tirées) relues sans écart par un analyseur « mots vers nombre » indépendant et identiques à `num2words` ; 26 frontières comparées à un tableau écrit à la main.
- **Décision** : (1) trait d'union seulement entre dizaines et unités (« vingt-deux », « quatre-vingt-dix-sept ») ; (2) « et » sans trait d'union pour 21, 31, 41, 51, 61 et 71, mais pas 81 ni 91 (« quatre-vingt-un », « quatre-vingt-onze ») ; (3) « quatre-vingts » et « cents » prennent un « s » quand rien ne les suit, pas devant « mille » (« quatre-vingt mille », « deux cent mille »), un « s » devant « million » (« deux cents millions ») ; (4) « mille » invariable, jamais « un mille » ; (5) « de » devant la devise pour des millions entiers (« un million de francs CFA ») ; (6) « franc » au singulier pour 0 et 1 (« zéro franc CFA »). Le montant est écrit avec une majuscule initiale sur la quittance.
- **Alternatives écartées** : orthographe rectifiée de 1990 (écartée par DP-64) ; « un mille ».
- **Conséquences** : la fonction du prototype (`spikes/pdf/words/`, sans dépendance) est reprise dans le crate `domain` à l'étape 07 avec ses tests : tous les entiers de 0 à 2 000, les frontières, 999 999 999, et un test de relecture par propriété. L'accord avec `num2words` ne valide pas le libellé juridique : « francs CFA » et les mentions de la quittance restent **[À VÉRIFIER AVEC UN JURISTE]** (DP-62).
- **Statut** : Prise (Mimis, 31/10/2026)
- **Liens** : DP-62, DP-64 ; R-053 ; E-11 ; `TESTING_QA.md` §6.

### DP-82 — Sauvegardes du pilote : objectifs adaptés, chiffrement à clé publique, détenteurs de la clé (décisions n° 24, 53 et 95)

- **Date** : 31/10/2026
- **Contexte** : fiche `specs/F39-sauvegardes.md` (étape 01). DP-69 suspend les objectifs de 15 minutes et de 30 jours à cause du plan gratuit de Neon.
- **Décision** :
  - **Objectifs acceptés par Mimis pour le pilote** : récupération d'une erreur récente par restauration à un instant donné dans les **6 dernières heures** ; **perte maximale de 24 heures** si l'hébergeur de la base est perdu (une copie logique par jour) ; **reprise sous 4 heures** en journée ouvrée ; **30 copies quotidiennes et 12 copies mensuelles** ; **test de restauration chaque mois** à partir d'une copie logique.
  - **Chiffrement à clé publique** (proposition de `DEPLOYMENT.md` §5.2, retenue) : le serveur et le travail de sauvegarde ne portent que la clé publique ; la **clé privée** est hors ligne, en **deux exemplaires, détenus par Mimis (porteur du projet) et par le développeur**. Outil : `age` (crate Rust `age`), sauf blocage constaté à l'implémentation, qui serait alors noté ici.
  - **Droits** : identifiants du travail d'envoi en **écriture seule** ; verrouillage d'objet ou versionnage exigé du second fournisseur.
  - **Second fournisseur** : **choix reporté** par Mimis (« on verra plus tard »). Tant qu'il n'est pas choisi, **aucune donnée réelle** n'entre dans le système et la couche 2 est éprouvée avec un second fournisseur simulé (dossier local), ce qui ne satisfait pas PP-13.
- **Alternatives écartées** : clé symétrique présente sur le serveur ; clé privée sur le serveur.
- **Conséquences** : `specs/F39-sauvegardes.md` passe à « validée » sauf le second fournisseur. PP-13 (copie chiffrée chez un second fournisseur, suppression refusée) ne peut pas être atteint avant ce choix : à reprendre dans les contenus C-02. La bascule `live` reste soumise à l'acceptation écrite de DP-69.
- **Statut** : Prise (Mimis, 31/10/2026), sauf le choix du second fournisseur
- **Liens** : décisions n° 24, 53, 95 ; DP-69, DP-78 ; F39 ; PP-13, PP-14, PP-15.

### DP-83 — Outillage de l'étape 01 : décisions déléguées à Claude Code

- **Date** : 31/10/2026
- **Contexte** : Mimis a répondu « fais comme tu veux » aux points restants de la fiche F39 (question 5 et décisions n° 88, 89, 92). Le poste n'a pas Docker et aucun droit d'administrateur n'est disponible à distance (CLAUDE.md, « Poste réel »).
- **Décision** (suit les propositions des documents, **à relire par Mimis**) :
  - **Docker** : l'image est construite et testée **dans la CI** (Linux), jamais sur le poste ; le développement local utilise le PostgreSQL du port 5433 ou une branche Neon de développement.
  - **Orchestration (n° 88)** : Docker Compose et un proxy à certificats automatiques (Caddy), image identifiée par empreinte, promotion manuelle (`DEPLOYMENT.md` §2.3 et §2.6).
  - **Alertes (n° 89)** : canal unique vers le développeur et le porteur du projet, **sans donnée personnelle** dans le message ; l'outil concret (e-mail d'équipe, application de messagerie) est choisi quand les comptes existent [À confirmer par Mimis].
  - **Secrets (n° 92)** : variables d'environnement et gestionnaire de mots de passe ; pas de coffre dédié au pilote ; aucun secret dans le dépôt (DP-56).
  - **TLS vers la base** : `sqlx` avec `native-tls` (Schannel sous Windows, OpenSSL sous Linux), **provisoire** : `rustls` demande un compilateur C absent du poste. `verify-full` est exigé en production ; le choix est revu en CI Linux.
  - **Rust** : version figée à 1.99.1 (`rust-toolchain.toml`).
- **Alternatives écartées** : Docker Desktop sur le poste (droits d'administrateur) ; coffre de secrets dédié ; plateforme de conteneurs gérée.
- **Conséquences** : le Dockerfile, Compose et la CI sont écrits mais **non exécutables sur le poste** ; leur première exécution se fait quand la plateforme de CI existe (C-01).
- **Statut** : Prise (délégation de Mimis à Claude Code, 31/10/2026), à relire
- **Liens** : décisions n° 88, 89, 92 ; DP-55, DP-56 ; `DEPLOYMENT.md` §2.3, §2.4, §7.3.

### DP-84 — Outils du contrat d'API (`CODING_RULES.md` question 6)

- **Date** : 31/10/2026
- **Contexte** : étape 01 §5.3 : générer le contrat OpenAPI depuis le code, le versionner, détecter les ruptures, produire le client TypeScript dans le même commit.
- **Décision** (par délégation de Mimis, à relire) : **`utoipa`** génère `openapi.json` depuis les annotations des routes Rust ; un test de fraîcheur compare le fichier versionné à la sortie de `immo openapi` ; **`oasdiff`** (binaire unique, en CI) compare le contrat à celui de la branche principale et fait échouer la CI sur une rupture non annoncée ; **`openapi-typescript`** produit `web/src/api/generated/` (étape 5.4). Le journal d'accès utilise `tracing` en JSON avec une liste blanche de champs ; l'identifiant de requête est un UUID renvoyé dans `X-Request-Id`.
- **Alternatives écartées** : contrat écrit à la main (dérive du code) ; génération du code Rust depuis le contrat (inverse la source de vérité) ; `aide` (moins répandu).
- **Conséquences** : `openapi.json` à la racine, régénéré par `immo openapi openapi.json` ; une route sans annotation est invisible du contrat : le test de couverture de la liste des routes (étape suivante) compare les routes du routeur au contrat. `/ready` est annoncé au contrat mais réservé au réseau interne (404 hors réseau interne ou relayé).
- **Statut** : Prise (délégation de Mimis à Claude Code, 31/10/2026), à relire
- **Liens** : DP-67 ; `API_CONTRACTS.md` §7.17 ; `CODING_RULES.md` question 6.
### DP-85 — Authentification, sessions, rôles, en-têtes de l'étape 02 (décisions n° 22, 45, 48, 49, 59)

- **Date** : 01/11/2026
- **Contexte** : fiches `specs/F01-comptes-organisation.md`, `F02-roles-permissions.md` et `F03-journal-audit.md`, validées par Mimis le 01/11/2026 (« Oui vas y », avec les recommandations de chaque fiche). Les cinq décisions ci-dessous étaient ouvertes et conditionnent l'étape 02.
- **Décision** :
  - **n° 48, mots de passe** : argon2id, 19 Mio, 2 itérations, parallélisme 1 comme valeur de départ ; calibrage à 100–500 ms sur le serveur d'essai quand il existera (une mesure locale est consignée à l'étape 02) ; **plafond de 4 vérifications simultanées** ; 10 à 128 caractères, aucune règle de composition.
  - **n° 49, sessions côté serveur** : cookie `__Host-session` (`Secure`, `HttpOnly`, `SameSite=Lax`, sans `Domain`) ; identifiant de 256 bits, seule l'empreinte SHA-256 stockée ; inactivité 30 minutes (60 pour gérant et propriétaire), absolue 12 heures ; pas de « se souvenir de moi ».
  - **Jeton anti-falsification** : dérivé par HMAC-SHA256 de l'identifiant de session avec `SESSION_SECRET` ; son empreinte est stockée à la création (`csrf_hash`) ; changer `SESSION_SECRET` invalide tous les jetons.
  - **n° 45, rôles** : `owner` et `collaborator` sont des rôles distincts, réservés au plan Propriétaire ; `manager`, `property_manager`, `cashier`, `field_agent`, `accountant` au plan Agence. Contrôlé par le code.
  - **n° 59, en-têtes** : ceux de `SECURITY.md` §7.3 tels quels, `Cache-Control: no-store` sur toute l'API.
  - **n° 22, application installable** : aucun service worker à l'étape 02 ; décision reprise à l'étape où une coquille hors ligne sert.
  - **Matrice (F02)** : 25 actions × 7 rôles codées une seule fois dans `identity::authorize` ; « (L) » = autorisé pour une action de lecture, refusé pour une action d'écriture ; droits réglables **refusés tant que `role_settings` n'existe pas** (étape 03) ; valeurs par défaut des droits réglables : « non » (question ouverte dans `DOMAINE.md`).
  - **Limitation des tentatives** : 5ᵉ échec consécutif → 1 minute, 6ᵉ → 5 minutes, 7ᵉ et suivants → 15 minutes ; 20 échecs par adresse en 10 minutes → 15 minutes ; une requête pendant l'attente est refusée sans vérifier le mot de passe et ne compte pas.
  - **Codes d'erreur nouveaux** : `auth.no_active_membership` (403), `auth.organization_required` (403), `org.suspended` (403).
  - **Écarts de schéma** : `users.failed_logins` et `users.locked_until` restent inutilisées (retrait ultérieur) ; `audit_log.actor_staff_id` est créée **sans clé étrangère** vers `platform_staff` (ajoutée avec la console) ; `--prefix` est retiré de `immo org create` (le préfixe appartient aux séries de numérotation, DP-61).
  - **Audit des connexions** : l'échec d'écriture de l'audit fait échouer la connexion (R-008) ; `security.rate_limited` est écrit dans les organisations du compte quand l'identifiant correspond à un compte, nulle part sinon.
- **Alternatives écartées** : jetons JWT sans état (révocation impossible) ; `UPDATE(csrf_hash)` accordé à `immo_app` ; compteurs sur `users.failed_logins` (ils ne couvrent pas les identifiants inconnus).
- **Conséquences** : `API_CONTRACTS.md` §3.4 reçoit les trois codes ; `ROADMAP.md` note l'écart sur `--prefix` ; le calibrage argon2id de l'essai reste à faire (dette de l'étape 01).
- **Statut** : Prise (Mimis, 01/11/2026)
- **Liens** : F01, F02, F03 ; R-003, R-004, R-007, R-008 ; DP-48, DP-66, DP-68, DP-77 ; `SECURITY.md` §3, §4, §7.3.

### DP-86 — Détection d'une tentative d'accès à une autre organisation (`security.cross_org_attempt`)

- **Date** : 01/11/2026
- **Contexte** : les contrats exigent un 404, une ligne d'audit chez l'appelant et une alerte quand une ressource d'une autre organisation est demandée (R-004). La RLS forcée rend cette ressource invisible : le code ne peut pas distinguer « n'existe pas » de « existe chez B ». Fiche F03 §4, validée le 01/11/2026.
- **Décision** : une fonction SQL `SECURITY DEFINER` `exists_in_other_org(object_type, id)` qui ne renvoie qu'un **booléen**, ne lit que des tables d'une **liste blanche** (`organizations` à l'étape 02 ; chaque route ajoute son type), est appelée **uniquement** sur le chemin « introuvable » et n'est jamais exposée par l'API. Vrai → ligne `security.cross_org_attempt` dans l'audit de l'appelant et alerte journalisée (`alert=security.cross_org_attempt`, S1). Un identifiant qui n'existe nulle part n'est pas journalisé comme tentative.
- **Alternatives écartées** : journaliser tout 404 sur identifiant bien formé (bruit, fausses alertes S1) ; un rôle `BYPASSRLS` pour lire (DP-78 réserve ce droit à `immo_backup`).
- **Conséquences** : la fonction est testée sur une table jouet tant que les tables métier n'existent pas ; le canal d'alerte reste la dette (5) de l'étape 01.
- **Statut** : Prise (Mimis, 01/11/2026)
- **Liens** : R-004 ; DP-48, DP-77 ; F03 §4 ; `SECURITY.md` §6.1.

### DP-87 — Écarts d'implémentation de l'étape 02 (à relire par Mimis)

- **Date** : 01/11/2026
- **Contexte** : en écrivant le code de l'étape 02 (F01, F02, F03, DP-85, DP-86), plusieurs détails que les documents ne tranchent pas ou qu'ils contredisent sont apparus. Chacun est consigné ici, avec son motif, pour ne pas rester dans le code seul.
- **Décision** (choix de Claude Code, dans le cadre de la validation des fiches ; **à relire**) :
  - **Plafond général de débit en mémoire** (300 requêtes par minute par session, 600 par adresse) au lieu de compteurs PostgreSQL : valable pour **une** instance (le pilote en a une). Les compteurs de **connexion** restent dans PostgreSQL (`login_attempts`). À déplacer dans PostgreSQL avant d'exploiter plusieurs instances.
  - **`own_memberships`** (lecture de ses propres adhésions) ne s'applique que **sans organisation active** : avec l'écriture du `DATABASE_SCHEMA.md` §6.4, un membre de deux organisations verrait, en travaillant dans la première, ses adhésions de la seconde. Une fonction `my_organizations()` (SECURITY DEFINER, limitée à l'utilisateur du contexte) donne la liste de choix.
  - **`immo_app` n'a pas `INSERT` sur `organizations`** : la création passe par `create_organization()` (plus strict que le schéma). Politiques ajoutées pour le propriétaire (`org_owner_insert`, `own_memberships_owner`) et pour la console (`org_platform_update`), nécessaires aux fonctions SECURITY DEFINER sous RLS forcée.
  - **Liste blanche de `exists_in_other_org`** : une table (`cross_org_registry`) alimentée par les migrations, pas du code ; la fonction parcourt les autres organisations en posant leur contexte (comme les migrations de données, `DATABASE_SCHEMA.md` §6.3), puis restitue celui de l'appelant. Complexité proportionnelle au nombre d'organisations, sur le seul chemin « introuvable ».
  - **Rotation de session** à chaque changement d'organisation ; l'expiration absolue de 12 heures est conservée (changer d'organisation ne prolonge rien).
  - **Toute heure vient de `Clock`** (DP-50) et est écrite explicitement (`attempted_at`, `last_seen_at`, `idle_expires_at`…), pas par le défaut `app_now()` de la base ; un test vérifie la parité de la date locale entre `Clock` et la base.
  - **Identifiant de requête** : code d'incident `INC-XXXXXX` (alphabet sans caractères ambigus), pas un UUID.
  - **Profil de compilation** : informations de débogage réduites et pas de compilation incrémentale (disque presque plein du poste) ; `argon2` et `blake2` optimisés même en développement, pour que le calibrage et les tests de durée soient réalistes.
  - **Calibrage argon2id** : outil `immo password calibrate`. Mesure sur le PC de développement (Intel i7-10510U) : 19 Mio × 2 itérations = **19 ms** (hors de la cible de 100 à 500 ms de la décision n° 48) ; 128 Mio × 3 = 215 ms ; 256 Mio × 3 = 443 ms. La valeur de départ **n'est pas changée** : l'essai n'existe pas encore, et 4 vérifications simultanées de 128 Mio coûteraient 512 Mo de mémoire. À décider avec le serveur retenu (C-02).
- **Alternatives écartées** : compteurs PostgreSQL pour le plafond général (surcoût d'écritures à chaque requête, inutile avec une instance) ; liste blanche en code Rust (une migration de table ne pourrait pas l'étendre) ; `UPDATE(csrf_hash)` pour `immo_app`.
- **Conséquences** : `DATABASE_SCHEMA.md` §6.11 consigne les écarts de schéma ; le calibrage de l'essai reste ouvert.
- **Statut** : Prise (Claude Code dans le cadre de DP-85, 01/11/2026), **à relire par Mimis**
- **Liens** : DP-48, DP-50, DP-77, DP-85, DP-86 ; décision n° 48.

### DP-88 — Rôles, invitations, TOTP, journal lisible et accord d'accès support (étape 03)

- **Date** : 01/11/2026
- **Contexte** : fiches `specs/F01-comptes-organisation.md` §9, `F02-roles-permissions.md` §7, `F03-journal-audit.md` §8 et `F40-acces-support.md`, rédigées le 01/11/2026. Mimis a répondu « Applique tes recommandations et passe à la suite » : toutes les recommandations de ces fiches (F01 §9.12, F02 §7.9, F03 §8.7, F40 §7) sont **acceptées telles qu'écrites**. Elles sont résumées ici pour qu'elles ne restent pas dans les fiches seules.
- **Décision** :
  - **Écarts de schéma (F01 §9.1)** : `invitations.renewed_from` + index unique partiel (une invitation en attente par identifiant et organisation) ; `password_resets.revoked_at` ; table `totp_backup_codes` ; `users.totp_last_step` ; `organizations.row_version` ; `support_grants.staff_id` rendu nullable (clé étrangère ajoutée avec la console, étape 11).
  - **TOTP** : défi signé de 5 minutes à la place d'une « session partielle » ; secret chiffré AES-256-GCM, décalage d'un pas toléré, rejeu refusé ; 10 codes de secours par HMAC-SHA256 à clé (dépendance : la clé de chiffrement des champs sert aussi aux codes de secours) ; **pas** de désactivation du TOTP, de régénération des codes ni de QR code à l'étape 03.
  - **Invitations et réinitialisation** : jeton de 256 bits, seule l'empreinte est stockée ; invitation 7 jours ; lien de réinitialisation 15 minutes, usage unique ; même code `invitation.expired` (410) pour une invitation expirée, révoquée ou utilisée, texte « Cette invitation n'est plus valable. Demandez au gérant de la renvoyer. » ; `invitation.already_pending` (409) ; `{id}` de `POST /users/{id}/password-resets` = identifiant d'adhésion ; **la réinitialisation du mot de passe d'un gérant n'est jamais faite par un autre gérant** : support à deux personnes, opération manuelle notée dans `platform_events` jusqu'à l'étape 11.
  - **Mots de passe courants** : liste de 1 000 entrées. **Préfixe de quittance provisoire `REC`** à la création d'organisation.
  - **Droits réglables** : quatre seulement (gestionnaire × encaisser, importer, exporter ; collaborateur × encaisser), **défaut « non »** ; le collaborateur du plan Propriétaire n'encaisse donc pas tant que le propriétaire n'a pas activé le réglage (rappel affiché dans S18).
  - **Confirmation par mot de passe** : tout changement de rôle, toute réactivation d'un membre, toute révocation d'accord support (le contrat d'API fait foi ; la réactivation est un ajout).
  - **Garde « dernier gérant »** (G-16) étendue au `owner` du plan Propriétaire ; **interdiction de se retirer soi-même**.
  - **Sessions** : un changement de rôle ne révoque **rien** (le rôle est relu à chaque requête) ; une désactivation révoque les sessions de **cette organisation** seulement.
  - **Agent terrain** : `unit_assignments` sans clé vers `units` jusqu'à l'étape 05 ; la route n'accepte qu'une liste vide.
  - **Journal** : l'adresse IP est visible du gérant, du comptable **et du propriétaire direct** (correction de `SECURITY.md` §4.2) ; `meta.actors` ajouté à `GET /audit` ; actions de compte écrites dans chaque organisation où la personne est membre actif ; `PUT/PATCH/DELETE /audit` : 405 `audit.immutable` + ligne `security.permission_denied` ; cinq codes d'audit nouveaux (`invitation.renewed`, `invitation.revoked`, `user.enabled`, `user.updated`, `auth.mfa_backup_used`) ; bouton « Exporter » de S21 en repère « À venir » jusqu'à l'étape 23.
  - **Accès support (F40)** : accord donné à l'**équipe HABEK**, pas à une personne ; durée 24 h par défaut (choix 24 heures, 3 jours, 7 jours), 7 jours au plus [à confirmer avec une agence pilote] ; **portée appliquée par la base** (`read` = lecture seule ; `import_assist` = écriture) par la fonction `support_grant_active()` et la politique `support_access`, plus strict que `DATABASE_SCHEMA.md` §6.6 ; plusieurs accords simultanés permis ; la ligne d'audit visible dans S21 tient lieu de notification aux autres gérants jusqu'à l'étape 20 ; case de lecture des champs chiffrés reportée à l'arrivée du premier champ chiffré (pièce d'identité du locataire).
- **Alternatives écartées** : session partielle pour le TOTP ; argon2id pour les codes de secours (10 calculs coûteux par saisie) ; réinitialisation d'un gérant par un autre gérant ; création de `platform_staff` vide dès maintenant ; « oui » par défaut pour le collaborateur ; révocation des sessions à chaque changement de rôle ; QR code (même appareil pour le navigateur et l'application d'authentification).
- **Conséquences** : `API_CONTRACTS.md` (journal des changements : défi TOTP, `{id}` = adhésion, `invitation.already_pending`, texte de `invitation.expired`, `meta.actors`, ligne « sessions révoquées »), `SECURITY.md` (§3.2 sessions, §4.2 adresse IP, §6.1 codes d'audit) et `DATABASE_SCHEMA.md` (écarts de schéma) sont corrigés dans le même changement. La migration modifie des gardes (`support_grants`, invitations, dernier gérant) : **second avis C-19 requis** avant le critère de validation de l'étape.
- **Statut** : Prise (Mimis, 01/11/2026)
- **Liens** : F01 §9, F02 §7, F03 §8, F40 ; R-003, R-004, R-005, R-007, R-008 ; G-16 ; DP-85, DP-86, DP-87 ; `SECURITY.md` §3, §4, §5.6, §6.1 ; `API_CONTRACTS.md` §7.2.

### DP-89 — Écarts d'implémentation de l'étape 03 (à relire par Mimis)

- **Date** : 01/11/2026
- **Contexte** : en écrivant le code et les tests de l'étape 03 (F01 §9, F02 §7, F03 §8, F40, DP-88), des détails que les fiches ne tranchent pas, ou qu'elles formulent autrement que le code, sont apparus. Chacun est consigné ici avec son motif.
- **Décision** (choix de Claude Code dans le cadre de DP-88 ; **à relire**) :
  - **G-16 par verrou consultatif** (`pg_advisory_xact_lock` par organisation) au lieu de `FOR UPDATE` sur les lignes des gérants : avec `FOR UPDATE`, deux gérants qui se retirent l'un l'autre se bloquent mutuellement (interblocage, erreur `40P01`, 500) ; avec le verrou, le second attend et voit l'état validé du premier (`user.last_manager`). Testé à deux connexions.
  - **Invitation lue par son empreinte** : fonction `invitation_by_token(bytea)` (`SECURITY DEFINER`, politique de lecture réservée à `immo_owner` sur `app.token_hash` posé par la fonction elle-même) ; `immo_app` n'a aucun droit de lecture sur `invitations` hors contexte d'organisation (testé : poser `app.token_hash` soi-même ne donne rien). Même schéma que les liens publics de `DATABASE_SCHEMA.md` §6.7.
  - **Politiques de la console** : `apply_support_policies()` et `apply_org_policies()` (fonctions réservées à `immo_owner`) générées pour chaque table à `org_id`, comme le bloc `DO` de §6.5 mais rejouable. Exception : sur `audit_log`, la console **écrit** ses lignes `actor_kind = 'support'` avec n'importe quel accord actif (R-009 : « chaque consultation écrit une ligne d'audit ») mais ne peut pas se faire passer pour un utilisateur.
  - **Double authentification** : le défi est `base64url(charge).base64url(HMAC)` avec une clé dérivée de `SESSION_SECRET` ; la limite de 5 essais ratés par minute est comptée **par utilisateur** (clé `SHA-256("totp:" + identifiant d'utilisateur)` dans `login_attempts`) et non par identifiant de connexion, car le défi ne porte pas le texte saisi ; HMAC-SHA1 (TOTP) et HMAC-SHA256 écrits à la main, vérifiés sur les vecteurs RFC 4226, 6238 et 4231 (aucune erreur possible sur la longueur de clé, donc pas de `unwrap`) ; secret chiffré par `aes-gcm` 0.11, codes de secours par HMAC-SHA256 d'une sous-clé de la clé de champs, liés à l'utilisateur.
  - **Clé de chiffrement des champs** : variable `FIELD_ENCRYPTION_KEY` (64 caractères hexadécimaux), **obligatoire en production**, facultative en développement (sans elle, `POST /me/totp` répond 500 `server.unexpected`). Identifiant de clé `1` en tête de chaque valeur : la rotation viendra avec un second identifiant.
  - **Dates du journal** : `from` et `to` sont au format `AAAA-MM-JJ` sur le fil (règle des dates métier de `API_CONTRACTS.md` §1), affichées `JJ/MM/AAAA` à l'écran ; la fiche F03 §8.1 écrivait `JJ/MM/AAAA` pour l'API.
  - **Corps de requête lu après les droits** : une route à droit refuse d'abord (`403 auth.forbidden`), puis lit le corps (`400`/`422`). Révélé par le test A3 : sans cela, un rôle sans droit recevait `400` sur un corps mal formé.
  - **Registre des routes** : nouvelle valeur `Access::Token` pour les quatre routes publiques « par jeton » (`/auth/totp/verify`, `/auth/password-reset/complete`, `/invitations/accept`, `/invitations/{token}`) ; la liste fermée des routes sans session de `API_CONTRACTS.md` §4.1 reste `login` et `logout`. Un test relit les deux listes.
  - **Réponses** : `POST /auth/login` renvoie `user` et `csrf_token` **nuls** quand `mfa_required` (aucune session avant le code) et un `challenge` ; `POST /invitations/accept` renvoie le même corps (sans `user` ni `csrf_token` pour un compte existant, qui garde sa session) ; `GET /organization` porte `version` ; `GET /me` renvoie les vrais réglages ; `GET /audit` renvoie `{ items[], next_cursor, meta: { actors[] } }`.
  - **Actions de compte et audit** : la ligne de l'organisation **active** est écrite dans la transaction de l'action (R-008) ; celles des **autres** organisations de la personne le sont juste après, une transaction chacune, en meilleur effort (un échec est journalisé). Une panne de l'audit d'une autre organisation n'annule donc pas un changement de mot de passe.
  - **Réinitialisation** : l'utilisation d'un lien écrit `auth.reset_used` **et** `auth.password_changed` (F03 §8.2 : « ou réinitialisation »).
  - **Limites de débit** : invitations 5 par heure et par identifiant (renvois compris, compte en base) ; réinitialisation 5 par heure et par personne visée (compte en base) et 10 essais par heure et par adresse (compteur en mémoire, une instance : même réserve que DP-87).
  - **`unit_assignments`** : sans clé vers `units` ; seule la liste vide est acceptée jusqu'à l'étape 05 ; `scope_of` est livrée et testée sur 25 lots dont 10 assignés.
  - **Création d'organisation** : la ligne `org_settings` (préfixe `REC`) est créée par un **déclencheur** sur `organizations`, pour que `create_organization()` et les insertions directes des essais donnent le même résultat ; les organisations existantes sont complétées par la migration.
- **Alternatives écartées** : `FOR UPDATE` seul pour G-16 ; lecture des invitations par un rôle `BYPASSRLS` (DP-78) ; défi stocké en base (une table de plus pour 5 minutes de vie).
- **Conséquences** : `DATABASE_SCHEMA.md` §6.12 consigne les écarts de schéma ; `API_CONTRACTS.md` (catalogue des erreurs, journal des changements) ; le second avis C-19 doit porter aussi sur G-16 (verrou), `invitation_by_token()` et les politiques de la console.
- **Statut** : Prise (Claude Code dans le cadre de DP-88, 01/11/2026), **à relire par Mimis**
- **Liens** : DP-77, DP-78, DP-85, DP-86, DP-87, DP-88 ; F01 §9, F02 §7, F03 §8, F40.

### DP-90 — Observation des usages, erreurs et signalements (étape 04)

- **Date** : 01/11/2026
- **Contexte** : fiche `specs/F39-observation.md` (rédigée le 01/11/2026), étape 04, décision n° 28 (événements en base, sans service tiers) et n° 66 (textes centralisés). Mimis a répondu « Applique tes recommandations » : les sept recommandations de F39-observation §11 sont **acceptées telles qu'écrites**. Résumé pour qu'elles ne restent pas dans la fiche seule.
- **Décision** :
  - **Liste blanche fermée** : énumération Rust `ProductEvent` (propriétés typées, aucune chaîne libre), export versionné `tests/contract/events-whitelist.json`. À l'étape 04 le front n'envoie que `screen_viewed` et `validation_error` ; les autres événements sont d'origine serveur et un événement d'origine serveur reçu du front est écarté. Un événement écarté est **compté** (`event_discarded` : motif et nombre, jamais le nom reçu), réponse `204` en production, `422 validation.failed` hors production (essai, développement).
  - **Valeurs refusées** quel que soit le type : `@`, suite de 6 chiffres ou plus, `+226`.
  - **Écarts de schéma** : `product_events.session_id` ajouté (sans clé étrangère) ; `feedback_reports.reference` (`SIG-` + six chiffres, séquence globale) ; contrôles de catégorie, de statut et de taille de contexte ; `feedback_reports.handled_by` sans clé étrangère jusqu'à la console (étape 11) ; `DELETE` de `product_events` limité par une politique aux lignes de plus de 13 mois.
  - **Exception à R-009 (question 1)** : `immo_platform` lit `product_events`, `error_reports` et `feedback_reports` de toutes les organisations **sans accord support** (télémétrie sans donnée personnelle ; le signalement est un texte envoyé volontairement à HABEK, avec avertissement dans le formulaire) et ne modifie que `status`, `handled_by`, `handled_at` des signalements. Politique `platform_read`. **Garde modifiée : second avis C-19 requis** (en plus de ceux des étapes 02 et 03).
  - **Alerte** : tâche dans la file (même transaction que le signalement) qui écrit `feedback.received` dans `platform_events` et émet l'événement `tracing` `alert = "feedback.received"`, sans description ni nom ; le canal réel (courriel, messagerie) vient avec C-01/C-02.
  - **Erreurs serveur** : agrégées par empreinte dans `error_reports` (transaction séparée, jamais de corps ni de message de bibliothèque) ; **sans organisation : journal `tracing` seulement**. `login_failed` et `session_expired` ne sont écrits que si l'organisation est identifiable.
  - **Seuils** : 20 événements et 8 Kio par lot, 2 000 ms pour `slow_response`, limites par session 30/min (`/events`), 10/min (`/client-errors`), 5/h (`/feedback`), 100 événements en mémoire dans le téléphone, 13 mois [Hypothèse].
  - **Route d'essai** `GET /api/_test/server-error`, montée seulement si `APP_ENV` n'est pas `production` **et** `ENABLE_TEST_ROUTES=1` ; absente d'`openapi.json`.
  - L'utilisateur ne relit pas ses signalements dans l'application à l'étape 04.
- **Alternatives écartées** : service tiers de suivi d'erreurs (données hors de notre base) ; aucune exception à R-009 (console des signalements inutilisable) ; `org_id` nullable dans `error_reports` (RLS plus risquée) ; routes d'essai compilées à part (même image pour l'essai et la production).
- **Conséquences** : `DATABASE_SCHEMA.md`, `API_CONTRACTS.md` §7.15 et §7.16 et le journal des changements, `SECURITY.md` §8.1 corrigés dans le même changement ; matrice de F02 complétée (`events.create`, `feedback.create` pour tous les rôles) ; le second avis C-19 doit porter sur `platform_read`.
- **Statut** : Prise (Mimis, 01/11/2026)
- **Liens** : F39-observation ; décisions n° 28 et 66 ; R-009 ; DP-86, DP-87, DP-88 ; `ARCHITECTURE.md` §8.4 ; `SECURITY.md` §7.7, §8.1.

### DP-91 — Écarts d'implémentation de l'étape 04 (à relire par Mimis)

- **Date** : 01/11/2026
- **Contexte** : en écrivant le code et les tests de l'étape 04 (F39-observation, DP-90), des détails que la fiche ne tranche pas, ou qu'elle formule autrement que le code, sont apparus. Chacun est consigné ici avec son motif.
- **Décision** (choix de Claude Code dans le cadre de DP-90 ; **à relire**) :
  - **Liste blanche en table plutôt qu'en énumération de variantes** : `SPECS` (nom, origine, propriétés et leur nature) pilote la validation, les constructeurs typés du serveur et l'export `tests/contract/events-whitelist.json`. Aucune nature de propriété n'est une chaîne libre (écran de la liste, choix fermé, entier borné, identifiant de champ, code, modèle de route) ; un test le vérifie. La fiche parlait d'une énumération « qui ne compile pas » ; l'équivalent est une validation unique par laquelle passent tous les événements.
  - **Référence de session** : `product_events.session_id` est un texte de 16 hexadécimaux (début de l'empreinte du jeton), et non un uuid : `sessions` n'a pas d'identifiant uuid.
  - **Événements de connexion écrits en tâche détachée** (`login_failed`, `rate_limited`) pour que la durée d'une connexion refusée ne distingue pas un compte connu d'un compte inconnu ; les autres événements du serveur s'écrivent dans la transaction ou juste après la réponse (au plus 2 s d'attente).
  - **`session_expired`** : écrit à la première requête qui présente un cookie échu, puis la session est révoquée pour ne l'écrire qu'une fois.
  - **`permission_denied`, `slow_response` et les erreurs 500** sont tirés de la réponse par l'intergiciel d'accès (indices posés sur la réponse), sans modifier chaque gestionnaire. L'authentification de la requête est refaite à cette occasion, **seulement** sur ces trois cas.
  - **Pile d'une erreur 500** : capturée à la création de l'erreur (`ApiError::unexpected`), noms de fonctions de notre code seulement. **Non couvert** : le code SQLSTATE d'une erreur de base (la cause est abandonnée avant `unexpected` dans le code existant) ; à ajouter quand les erreurs porteront leur cause.
  - **Limite connue du nettoyage des erreurs du front** : suites de 6 chiffres, adresses, `+226`, jetons hexadécimaux et mots de 32 caractères sont remplacés ; **un mot libre ne se reconnaît pas**. Le front n'envoie que des messages issus de son propre code, jamais un texte saisi ; à revoir si un message cite une saisie.
  - **Limites de débit** : elles comptent les tentatives (une description refusée consomme une des 5 créations de l'heure) ; en mémoire, une instance (même réserve que DP-87).
  - **`Idempotency-Key` de `/feedback`** : colonne `idempotency_key` sur la table (pas de table `idempotency_keys`, qui n'existe pas encore) ; un rejeu répond `201` avec la même référence.
  - **Alerte** : tâche `feedback.notify` traitée par `immo work` ; `PLATFORM_DATABASE_URL` facultative (sans elle l'alerte reste dans le journal). Tâche quotidienne `observation.purge` (13 mois).
  - **Dimensions du front** : file de 100 événements au plus (jamais écrite sur le téléphone), lots de 20, envoi à 10 s ou à la mise en arrière-plan, erreurs JavaScript : 5 par session, une identique par minute.
- **Alternatives écartées** : `trybuild` (une dépendance de plus pour prouver ce que la validation unique et un test de liste prouvent déjà) ; écrire les événements de connexion dans la requête (écart de durée).
- **Conséquences** : le second avis C-19 porte aussi sur la politique `platform_read` ; `SECURITY.md` §8.1 mentionne la limite du nettoyage.
- **Statut** : Prise (Claude Code dans le cadre de DP-90, 01/11/2026), **à relire par Mimis**
- **Liens** : DP-86, DP-87, DP-90 ; F39-observation ; `ARCHITECTURE.md` §8.4.

## Décisions ouvertes

Les 15 décisions de la section 26 du dossier, plus trois ajoutées hors dossier (n° 16 à 18). La colonne « Recommandation du dossier » n'est **pas** une décision. Sont prises : n° 1 (DP-28, puis DP-34, puis DP-52 et DP-53), 2, 4, 6, 7 (DP-76), 8, 9, 10, 14, 16, 18, 19, 20, 21, 23, 25, 27, 30, 31, 32, 34, 35, 36, 40, 41, 46 (confirmée, DP-78), 50, 51, 55, 60, 61, 62, 64, 67, 69, 70, 73 à 77, 79, 80 et 87 (voir la colonne Statut). Les autres sont **Ouvertes**. Prises ensuite (étape 01) : n° 24, 53, 95 (DP-82) et n° 88, 89, 92 (DP-83).

| N° | Question | Options | Recommandation du dossier | Échéance | Statut |
|---|---|---|---|---|---|
| 1 | Seuils de Go/No-Go de la phase 0 | Seuils chiffrés ; « on verra » | Fixer les seuils de la section 17 avant le premier entretien | Avant la maquette | **Prise le 24/10/2026 (DP-28, remplacée par DP-34)** |
| 2 | Premier segment de pilote | Agences ; propriétaires ; les deux | Agences comme pilote principal (la reddition de comptes est le plus gros levier), plus 1–2 propriétaires directs pour valider le plan léger | Phase 0 | **Prise le 30/10/2026 (DP-57)** : agence en pilote principal, plus 1 à 2 propriétaires directs |
| 3 | Grille de prix à tester | Prix unique ; paliers par plan | Paliers de la section 22. Méthode de questionnement, pas un prix annoncé | Avant le premier pilote | Ouverte |
| 4 | Périmètre de la maquette | Tous les écrans ; cinq parcours | Cinq parcours de la section 14, avec repères « V1 / à venir » | Avant de lancer Claude Code | **Prise le 24/10/2026 : tous les écrans (DP-09)** |
| 5 | Hébergement et conformité | Serveur hors du pays ; local | Hébergement fiable à coût maîtrisé. Contrat avec clause de restitution et chiffrement. Échange avec la CIL ou un juriste | Phase 1 | Ouverte, en partie tranchée par DP-55 (base chez Neon) ; serveur, stockage objet, CIL restent ouverts |
| 6 | Frontend | TypeScript (React/Svelte) ; Rust full-stack | TypeScript au départ ; Rust pour le backend | Phase 1 | **Prise le 30/10/2026 (DP-56)** |
| 7 | Moteur de génération PDF | Bibliothèque Rust ; moteur de mise en page ; HTML→PDF | Prototype de 2 à 3 jours avant de décider (`ARCHITECTURE.md` §7) | Prototype (a), tranche verticale | **Prise le 31/10/2026 (DP-76)** : Typst en bibliothèque |
| 8 | Paiement en ligne | MVP ; P2 ; jamais | P2, et jamais de détention de fonds par HABEK | Avant la phase 6 | **Prise le 24/10/2026 : P2 (DP-32)** |
| 9 | Relances | Lien pré-rempli ; API WhatsApp | Lien pré-rempli. API après preuve de demande | Phase 2 | **Prise le 24/10/2026 : lien pré-rempli (DP-33)** |
| 10 | Règle de reddition de comptes | Date d'encaissement ; date d'échéance | Date d'encaissement (à confirmer par entretiens) | Avant la tranche 5 (T2) | **Prise le 30/10/2026 (DP-59)** : date d'encaissement par défaut, à confirmer avec l'agence pilote |
| 11 | Modèle d'onboarding | Libre-service ; accompagné payant ; inclus | Import accompagné, facturé ou inclus dans l'annuel | Phase 3 | Ouverte |
| 12 | Validation juridique | Aucune ; revue ponctuelle | Revue ponctuelle : mentions de quittance, plafonds du dépôt de garantie, bail commercial, CGU et clauses de données | Avant le premier pilote (DP-62) | Ouverte |
| 13 | Nom et marque | Garder « Immo Suite » ; vérifier ; changer | Vérifier la disponibilité du nom, du domaine et d'éventuelles marques existantes avant la vidéo de démonstration | Avant publication | Ouverte ; éditeur tranché par DP-63 (HABEK Technology Company), nom du produit et domaine ouverts |
| 14 | Capacité de développement | Dédiée ; partagée | Définir noir sur blanc les heures par semaine. Recalibrer la roadmap en conséquence | Phase 1 | **Prise le 30/10/2026 (DP-54)** : un développeur, 25 heures par semaine |
| 15 | Données de test | Données fictives ; données réelles anonymisées | Anonymisées, avec accord écrit du client | Avant le premier pilote | Ouverte |
| 16 | Palette définitive (ajoutée hors dossier le 24/10/2026) | Palette bronze/ocre dérivée de HABEK ; couleur du logo ; palette hybride | Aucune dans le dossier. Palette bleu nuit, blanc et turquoise retenue (DP-44). Le logo, non fourni, reste à réconcilier avec elle | Avant la diffusion publique des vidéos | **Prise** (DP-44, 25/10/2026) |
| 17 | Vouvoiement dans l'interface (ajoutée hors dossier le 24/10/2026) | Vouvoiement ; tutoiement ; au choix de l'organisation | Aucune dans le dossier. Vouvoiement provisoire dans `UI_UX_DESIGN.md`, à tester en entretien | Avant la diffusion publique des vidéos | Ouverte |
| 18 | Langue des identifiants de code (ajoutée hors dossier le 24/10/2026) | Interface et documentation en français, identifiants de code en anglais ; tout en français ; tout en anglais | Aucune dans le dossier. Convention proposée dans `CLAUDE.md` : interface et documentation en français, identifiants de code en anglais | Phase 1 | **Prise le 30/10/2026 (DP-67)** : identifiants en anglais |
| 19 | Stratégie : pilote réel sans enquête préalable (ajoutée le 30/10/2026) | Remplacer DP-07, DP-10 et DP-34 pour la phase 0 ; garder la phase 0 | Aucune dans le dossier. Annoncée par la consigne d'`ARCHITECTURE.md`, contraire à DP-07 : une entrée DP-xx doit la trancher | Avant tout code applicatif | **Prise le 30/10/2026 (DP-52, DP-53)** |
| 20 | Pile backend (ajoutée le 30/10/2026) | Axum + SQLx + sessions serveur + file de tâches PostgreSQL + REST/OpenAPI ; Actix Web ; ORM ; jetons JWT | `ARCHITECTURE.md` §3 : Axum, SQLx, sessions en base, file de tâches maison, REST décrit par OpenAPI | Avant la tranche verticale | **Prise le 30/10/2026 (DP-56)** |
| 21 | Exploitation de PostgreSQL (ajoutée le 30/10/2026) | Managé avec restauration à un instant donné ; installé sur le serveur | `ARCHITECTURE.md` §3.3 : managé, plus une copie logique quotidienne chez un second fournisseur | Avec la décision n° 5 | **Prise le 30/10/2026 (DP-55)** : PostgreSQL managé chez Neon |
| 22 | Application installable (PWA) (ajoutée le 30/10/2026) | PWA avec cache de la coquille seulement ; site sans service worker | `ARCHITECTURE.md` §3.6 : PWA, jamais de cache d'API ni d'écriture hors-ligne | Tranche 1 | **Prise le 01/11/2026 (DP-85)** |
| 23 | Dépôt et structure (ajoutée le 30/10/2026) | Monorepo ; deux dépôts | `ARCHITECTURE.md` §3.4 : monorepo (crates `domain`, `app`, `server`, dossier `web`) | Avant la tranche verticale | **Prise le 30/10/2026 (DP-56)** |
| 24 | Objectifs de sauvegarde (ajoutée le 30/10/2026) | Valeurs à fixer | `ARCHITECTURE.md` §10.4 : perte maximale 15 min, reprise 4 h, test de restauration mensuel | Avant le premier pilote | **Prise le 31/10/2026 (DP-82)** : objectifs adaptés par DP-69 (6 heures, perte maximale de 24 heures, reprise 4 heures) ; 15 minutes et 30 jours suspendus |
| 25 | Remise à zéro d'une organisation pilote (ajoutée le 30/10/2026) | Purge possible avant bascule ; jamais de purge (nouvelle organisation) | `ARCHITECTURE.md` §8.6 : états `setup` puis `live`, purge seulement en `setup`, accord écrit. Touche R-005 | Avant le premier pilote | **Prise le 30/10/2026 (DP-60)** |
| 26 | Organisation de démonstration (ajoutée le 30/10/2026) | En production ; en essai seulement | `ARCHITECTURE.md` §8.5 : en production, `kind = demo`, filigrane, exclue des indicateurs | Tranche 1 | Ouverte |
| 27 | E-mail et SMS au premier pilote (ajoutée le 30/10/2026) | Aucun, liens copiables ; passerelle SMS ; e-mail transactionnel | `ARCHITECTURE.md` §3.8 et §3.11 : aucun au premier pilote, réinitialisation par lien généré par le gérant ou le support | Tranche 1 | **Prise le 30/10/2026 (DP-68)** |
| 28 | Observation des usages (ajoutée le 30/10/2026) | Événements en base et suivi d'erreurs maison ; service tiers | `ARCHITECTURE.md` §8.4 : en base, sans traceur tiers ni cookie publicitaire, bouton « Signaler un problème » | Tranche verticale | **Prise le 31/10/2026 (DP-74)** |
| 29 | Précision du taux de commission (ajoutée le 30/10/2026) | Centièmes de point (750 = 7,5 %) ; millièmes | `ARCHITECTURE.md` §4.1 : centièmes de point, si deux décimales suffisent | Tranche 5 | Ouverte |
| 30 | Énumérations de la base (ajoutée le 30/10/2026) | Types `ENUM` ; texte et `CHECK` ; tables de référence | `DATABASE_SCHEMA.md` §1.2 : texte et `CHECK`, valeurs en anglais | Avant la migration initiale | **Prise le 30/10/2026 (DP-56)** |
| 31 | Identifiants de la base (ajoutée le 30/10/2026) | UUID v4 ; UUID v7 ; entier séquentiel | `DATABASE_SCHEMA.md` §1.2 : UUID v4 générés par la base ; `bigint` pour les journaux | Avant la migration initiale | **Prise le 30/10/2026 (DP-56)** |
| 32 | Clés étrangères composites (ajoutée le 30/10/2026) | Clé simple sur `id` ; clé composite `(org_id, id)` | `DATABASE_SCHEMA.md` §1.2 : composite entre toutes les tables métier | Avant la migration initiale | **Prise le 30/10/2026 (DP-56)** |
| 33 | Soldes (ajoutée le 30/10/2026) | Vues dérivées ; colonnes stockées ; table de synthèse | `DATABASE_SCHEMA.md` §5 : vues ; table de synthèse seulement si une mesure l'impose (DP-51 fixe que le statut n'est jamais stocké) | Tranche 3 | Ouverte |
| 34 | Écriture seule à deux barrières (ajoutée le 30/10/2026) | Privilèges seuls ; privilèges et déclencheur | `DATABASE_SCHEMA.md` §4.2 : privilèges et déclencheur sur 25 tables, exceptions par deux fonctions. Précise DP-47 | Avant la migration initiale | **Prise le 30/10/2026 (DP-56)** |
| 35 | Bornes des montants (ajoutée le 30/10/2026) | Sans borne ; 1 à 999 999 999 | `DATABASE_SCHEMA.md` §1.2 : borne alignée sur le montant en lettres testé | Avant la migration initiale | **Prise le 30/10/2026 (DP-56)** |
| 36 | Numérotation : séries d'essai et officielle, année (ajoutée le 30/10/2026) | Série unique et purge ; deux séries ; année d'émission ou année du paiement | `DATABASE_SCHEMA.md` §4.3 : deux séries (`trial`, `official`), année d'émission. Lié à la n° 25 | Avant le premier pilote | **Prise le 30/10/2026 (DP-61)** |
| 37 | Contrôles différés des affectations (ajoutée le 30/10/2026) | Contrôle dans le code seulement ; contrôles différés sous verrou du bail | `DATABASE_SCHEMA.md` §4.5 : contrôles différés, niveau `READ COMMITTED` exigé | Avant le premier pilote | Ouverte |
| 38 | Accès support : politique d'isolation dédiée (ajoutée le 30/10/2026) | Contrôle dans la console seulement ; politique de la base sous accord du client | `DATABASE_SCHEMA.md` §6.6 : politique dédiée, accord de 7 jours au plus | Avant le premier pilote | Ouverte |
| 39 | Lecture publique par jeton (ajoutée le 30/10/2026) | Politique par empreinte de jeton ; fonction à rôle contournant la RLS | `DATABASE_SCHEMA.md` §6.7 : politique par jeton, aucun rôle `BYPASSRLS` pour l'application | Tranche 3 | Ouverte |
| 40 | Annulation d'import et purge par fonctions (ajoutée le 30/10/2026) | Fonctions à droits du propriétaire ; `DELETE` accordé à l'application ; archivage seulement | `DATABASE_SCHEMA.md` §4.2 : deux fonctions `SECURITY DEFINER`, exception encadrée à R-005 | Avant le premier pilote | **Prise le 30/10/2026 (DP-60)** |
| 41 | Chevauchement de baux avec les baux terminés (ajoutée le 30/10/2026) | Exclusion sur actifs et préavis ; exclusion étendue aux terminés | `DATABASE_SCHEMA.md` §4.1 : étendue aux baux terminés | Avant la migration initiale | **Prise le 30/10/2026 (DP-56)** |
| 42 | Avenants : garde mécanique (ajoutée le 30/10/2026) | Contrôle dans le code ; garde de la base ; règle complète | `DATABASE_SCHEMA.md` §4.4 : garde minimale (aucune échéance payée à partir de la date d'effet) ; la règle complète dépend de la question 1 de §11.1 | Avant la tranche 2 | Ouverte |
| 43 | Téléphone du locataire unique parmi les fiches non archivées (ajoutée le 30/10/2026) | Unique sur toutes les fiches ; unique parmi les non archivées | `DATABASE_SCHEMA.md` §4.7 : non archivées | Avant la tranche 2 | Ouverte |
| 44 | Comptes rendus : aperçu calculé, version écrite à la validation (ajoutée le 30/10/2026) | Brouillon stocké ; aperçu calculé | `DATABASE_SCHEMA.md` §3 : aperçu calculé, `statement_versions` à la validation | Tranche 5 | Ouverte |
| 45 | Rôles du plan Propriétaire (ajoutée le 30/10/2026) | Rôles distincts `owner`, `collaborator` ; profils de droits sur les rôles de l'agence | `DATABASE_SCHEMA.md` §3 : valeurs distinctes | Tranche 1 | **Prise le 01/11/2026 (DP-85)** |
| 46 | Sauvegarde logique sous RLS forcée (ajoutée le 30/10/2026) | Rôle de sauvegarde avec `BYPASSRLS` ; RLS non forcée ; sauvegarde physique seulement | `DATABASE_SCHEMA.md` §6.9 : rôle de sauvegarde si l'hébergeur le permet. À vérifier avec la n° 5 et la n° 21 | Avant de choisir l'hébergeur | **Prise le 30/10/2026 (DP-55)** : rôle de sauvegarde avec `BYPASSRLS` si Neon le permet (prototype b), sinon RLS non forcée ; **condition levée le 31/10/2026 (DP-78)** : rôle `immo_backup` avec `BYPASSRLS` confirmé sur Neon, RLS forcée conservée |
| 47 | Procédure d'exception pour réinjecter des données (ajoutée le 30/10/2026) | Gardes désactivées par le propriétaire en maintenance ; aucune réinjection | `DATABASE_SCHEMA.md` §8.4 | Avant le premier pilote | Ouverte |
| 48 | Mots de passe (ajoutée le 30/10/2026) | argon2id ; bcrypt ; scrypt | `SECURITY.md` §3.1 : argon2id, 19 Mio, 2 itérations, parallélisme 1 (à calibrer), 10 caractères au moins | Tranche 1 | **Prise le 01/11/2026 (DP-85)** |
| 49 | Sessions côté serveur : cookie et durées (ajoutée le 30/10/2026) | Sessions ; jetons JWT | `SECURITY.md` §3.2 : sessions, cookie `__Host-session`, inactivité 30 minutes (60 pour gérant et propriétaire), absolue 12 heures. Confirme `ARCHITECTURE.md` §3.8 | Tranche 1 | **Prise le 01/11/2026 (DP-85)** |
| 50 | Double authentification (ajoutée le 30/10/2026) | TOTP ; SMS ; clés d'accès ; mot de passe seul et confirmation | `SECURITY.md` §3.4 : TOTP obligatoire pour la console ; confirmation par mot de passe avant les actions à enjeu ; TOTP du gérant obligatoire avant la bascule en `live`. Avance le TOTP du gérant, reporté par `ARCHITECTURE.md` | Avant le premier pilote | **Prise le 30/10/2026 (DP-68)** |
| 51 | Inscription publique (ajoutée le 30/10/2026) | Ouverte ; fermée au pilote | `SECURITY.md` §5.7 : fermée au pilote, organisations créées par la console (écart avec le texte de F01) | Avant le premier pilote | **Prise le 30/10/2026 (DP-68)** |
| 52 | Chiffrement des champs sensibles (ajoutée le 30/10/2026) | Dans l'application ; extension de la base ; aucun | `SECURITY.md` §5.2 : application, AES-256-GCM, clé hors base | Avant le premier pilote | Ouverte |
| 53 | Sauvegardes : droits et clé (ajoutée le 30/10/2026) | Identifiants en écriture seule et verrouillage ; identifiants complets | `SECURITY.md` §5.3. Complète la n° 24 | Avant le premier pilote | **Prise le 31/10/2026 (DP-82)** : écriture seule et verrouillage d'objet ; second fournisseur à choisir |
| 54 | Accès du personnel HABEK aux données (ajoutée le 30/10/2026) | Accord du client et durée limitée ; accès libre tracé | `SECURITY.md` §5.6 : accord du client, 24 heures par défaut, aucun accès de routine, accès d'urgence à deux | Tranche 6 | Ouverte |
| 55 | Code de contrôle de la quittance (ajoutée le 30/10/2026) | Aucun ; code et page publique de vérification | `SECURITY.md` §7.5 : code de 10 caractères et page qui n'expire pas. Répond à la question ouverte de `UI_UX_DESIGN.md` | Tranche 3 | **Prise le 30/10/2026 (DP-64)** : après le pilote |
| 56 | Durées de conservation (ajoutée le 30/10/2026) | Voir `SECURITY.md` §6.3 | Table proposée ; à faire valider par un juriste | Avant le premier pilote (avis juridique, DP-62) | Ouverte |
| 57 | Engagement de confidentialité des agences pilotes (ajoutée le 30/10/2026) | Aucun ; engagement simple ; contrat complet | `SECURITY.md` §8.6 : contenu proposé, à faire relire par un juriste | Avant le premier pilote (avis juridique, DP-62) | Ouverte |
| 58 | Tests de sécurité (ajoutée le 30/10/2026) | Voir `SECURITY.md` §11 | Obligatoires avant des données réelles : revue ciblée, isolation, permissions, fichiers, auto-test d'intrusion, restauration. Test externe avant la version commerciale | Avant le premier pilote | Ouverte |
| 59 | En-têtes de sécurité et journaux (ajoutée le 30/10/2026) | Voir `SECURITY.md` §7.3, §7.7 | CSP stricte, `Referrer-Policy: no-referrer`, adresses à jeton masquées | Tranche 1 | **Prise le 01/11/2026 (DP-85)** |
| 60 | Récupération de compte (ajoutée le 30/10/2026) | Lien généré de 15 minutes ; code par SMS | `SECURITY.md` §3.5 : lien généré, vérification renforcée pour le gérant. Cohérent avec la n° 27 | Tranche 1 | **Prise le 30/10/2026 (DP-68)** |
| 61 | Source de vérité du contrat d'API (ajoutée le 30/10/2026) | Contrat généré depuis le code Rust, fichier OpenAPI versionné ; contrat écrit à la main d'abord | `API_CONTRACTS.md` §1.2 : code d'abord, différence relue en revue, document écrit d'abord sur le papier. Précise la n° 20 | Avant la tranche verticale | **Prise le 30/10/2026 (DP-56)** |
| 62 | Versionnement et évolution de l'API (ajoutée le 30/10/2026) | Sans numéro dans les adresses, évolution additive ; préfixe `/api/v1` | `API_CONTRACTS.md` §1.3 : additive, `X-Client-Version` et `X-Min-Client-Version`, dépréciation datée (14 jours au moins). Précise `ARCHITECTURE.md` §3.7 | Avant la tranche verticale | **Prise le 30/10/2026 (DP-56)** |
| 63 | Pagination (ajoutée le 30/10/2026) | Curseur opaque signé ; `offset` et page | `API_CONTRACTS.md` §2.5 : curseur, 25 lignes par défaut, 100 au plus | Tranche 1 | Ouverte |
| 64 | Verrouillage optimiste des fiches (ajoutée le 30/10/2026) | `row_version` et `If-Match` dès le pilote ; dernier écrit l'emporte ; comparaison de champs | `API_CONTRACTS.md` §5.2 : `row_version` dès le pilote. Avance un point de `DATABASE_SCHEMA.md` (question 15) de [APRÈS] à [PILOTE] | Avant la migration initiale | **Prise le 30/10/2026 (DP-56)** |
| 65 | Opérations longues (import, PDF, export) (ajoutée le 30/10/2026) | `202` et interrogation de `/operations/{id}` ; flux d'événements ; requête qui attend | `API_CONTRACTS.md` §5.4 : interrogation | Tranche 3 | Ouverte |
| 66 | Erreurs de l'API et textes affichés (ajoutée le 30/10/2026) | `problem+json`, code stable, textes centralisés dans le front ; texte du serveur affiché tel quel | `API_CONTRACTS.md` §2.7 et §3 : code et paramètres du serveur, textes du front | Tranche verticale | **Prise le 31/10/2026 (DP-74)** |
| 67 | Casse des champs JSON (ajoutée le 30/10/2026) | `snake_case` aligné sur la base ; `camelCase` | `API_CONTRACTS.md` §2.1 : `snake_case` anglais. Va dans le sens de la n° 18 | Avant la tranche verticale | **Prise le 30/10/2026 (DP-56)** |
| 68 | Téléchargement des PDF et des fichiers (ajoutée le 30/10/2026) | Flux par l'API ; adresse signée du stockage objet | `API_CONTRACTS.md` §5.5 : flux par l'API au pilote | Tranche 3 | Ouverte |
| 69 | Gestion des erreurs et interdits Rust (ajoutée le 30/10/2026) | `thiserror` dans les bibliothèques et `anyhow` dans le binaire ; `anyhow` partout | `CODING_RULES.md` §4.1 et §4.7 : types d'erreur propres, `unwrap`, `expect`, `panic` et `unsafe` interdits | Avant la tranche verticale | **Prise le 30/10/2026 (DP-56)** |
| 70 | Bibliothèque de dates (ajoutée le 30/10/2026) | `chrono` avec `chrono-tz` ; `jiff` | `CODING_RULES.md` §4.3 : `chrono` + `chrono-tz` (SQLx), prise en charge de `jiff` à vérifier | Avant la tranche verticale | **Prise le 30/10/2026 (DP-56)** |
| 71 | Données du front (ajoutée le 30/10/2026) | Client généré et enveloppe fine ; bibliothèque de cache | `CODING_RULES.md` §5.1 : sans bibliothèque de cache au pilote | Tranche verticale | **Prise le 31/10/2026 (DP-74)** |
| 72 | Outils du front (ajoutée le 30/10/2026) | ESLint, Prettier et `size-limit` ; Biome | `CODING_RULES.md` §3.2 : ESLint (`typescript-eslint`, `jsx-a11y`) et Prettier | Tranche verticale | **Prise le 31/10/2026 (DP-74)** |
| 73 | Branches et commits (ajoutée le 30/10/2026) | Tronc commun, branches courtes et PR ; branches longues par fonctionnalité | `CODING_RULES.md` §8.1 : tronc commun, `main` protégée, squash, commits en français avec identifiants `F`, `R`, `DP` | Avant la tranche verticale | **Prise le 30/10/2026 (DP-56)** |
| 74 | Revue et taille des changements (ajoutée le 30/10/2026) | Revue par l'autre développeur avec chemins sensibles ; revue facultative | `CODING_RULES.md` §8.2 et §8.3 : relecture obligatoire, PR de moins de 400 lignes, liste de contrôle. Dépend de la n° 14 | Avant la tranche verticale | **Prise le 30/10/2026 (DP-56)** |
| 75 | Outillage de la règle « jamais `&PgPool` hors `platform` » (ajoutée le 30/10/2026) | Lints `disallowed-types` et `disallowed-methods` ; revue seule ; crate dédié | `CODING_RULES.md` §3.1 : lints dès le pilote. Avance l'outil d'[APRÈS] à [PILOTE] (`ARCHITECTURE.md` §5.3) | Avant la tranche verticale | **Prise le 30/10/2026 (DP-56)** |
| 76 | Détection de secrets (ajoutée le 30/10/2026) | Outil dédié en CI et en pré-validation ; revue seule | `CODING_RULES.md` §3.4 : outil à binaire unique sur tout l'historique. Produit à choisir | Avant la tranche verticale | **Prise le 30/10/2026 (DP-56)** |
| 77 | Licences des dépendances (ajoutée le 30/10/2026) | Liste blanche dans `cargo deny` ; aucune règle | `CODING_RULES.md` §4.6 : liste blanche de licences permissives, copyleft fort refusé | Avant la tranche verticale | **Prise le 30/10/2026 (DP-56)** |
| 78 | Tests de bout en bout et accessibilité automatique (ajoutée le 30/10/2026) | Playwright ; Cypress | `TESTING_QA.md` §5 et §8.5 : Playwright (déjà installé pour la maquette) | Tranche verticale | **Prise le 31/10/2026 (DP-74)** |
| 79 | Tests de propriétés (ajoutée le 30/10/2026) | `proptest` ; `quickcheck` | `TESTING_QA.md` §2.2 : `proptest` | Avant la tranche verticale | **Prise le 30/10/2026 (DP-56)** |
| 80 | Base des tests d'intégration (ajoutée le 30/10/2026) | PostgreSQL réel, base éphémère, rôles de production ; simulation | `TESTING_QA.md` §1.3 : PostgreSQL réel et rôles de production | Avant la tranche verticale | **Prise le 30/10/2026 (DP-56)** |
| 81 | Profil de test « Android d'entrée de gamme, réseau lent » (ajoutée le 30/10/2026) | Émulation seule ; émulation et appareil réel acheté | `TESTING_QA.md` §8.1 : émulation et un appareil réel. Budget à valider | Tranche verticale | Ouverte |
| 82 | Chaîne d'intégration continue (ajoutée le 30/10/2026) | Étapes (rapide, intégration, nocturne, avant livraison) ; une seule étape | `TESTING_QA.md` §10 : PR sous 15 minutes, pas de nouvel essai sur les tests d'argent et d'isolation, pas de seuil de couverture au pilote | Tranche verticale | **Prise le 31/10/2026 (DP-74)** |
| 83 | Tests visuels du PDF (ajoutée le 30/10/2026) | Texte extrait et image de référence approuvée ; revue manuelle seule | `TESTING_QA.md` §6. Dépend de la n° 7 | Après le prototype PDF | Ouverte ; méthode du prototype (texte extrait, planche relue) notée en DP-76, image de référence à décider à l'étape 08 |
| 84 | Gravités et délais de correction (ajoutée le 30/10/2026) | S1 à S4, erreur de montant ou d'isolation toujours S1 ; autres échelles | `TESTING_QA.md` §11 : S1 correctif en 24 h, S2 en 72 h (dossier §20.2). Heures de couverture à fixer | Avant le premier pilote | Ouverte |
| 85 | Protocole de recette avec les agences (ajoutée le 30/10/2026) | Séances de 90 minutes sur place, animateur et observateur, sans enregistrement de données réelles ; séances à distance | `TESTING_QA.md` §13 | Avant le premier pilote | Ouverte |
| 86 | Hébergement, grille de choix et coûts (ajoutée le 30/10/2026) | Europe avec PostgreSQL managé ; grand cloud en Afrique ; hébergeur au Burkina Faso | `DEPLOYMENT.md` §1.2 et §1.3 : option (a) sous trois conditions, coûts estimés de 50 à 120 € par mois. Précise les n° 5, 21 et 46 | Avec la n° 5 | Ouverte, en partie tranchée par DP-55 (base) ; serveur, stockage objet, second fournisseur et coûts restent ouverts |
| 87 | Dépôt de code et CI (ajoutée le 30/10/2026) | Plateforme grand public ; plateforme concurrente ; forge auto-hébergée | `DEPLOYMENT.md` §2.2 : plateforme avec CI sur compte d'organisation, double authentification, protection de branche | Avant la tranche verticale | **Prise le 30/10/2026 (DP-56)** |
| 88 | Orchestration et mise en production (ajoutée le 30/10/2026) | Docker Compose et proxy à certificats automatiques ; plateforme gérée de conteneurs ; Kubernetes | `DEPLOYMENT.md` §2.3 et §2.6 : Compose, image par empreinte, promotion manuelle, coupure de quelques secondes hors heures d'encaissement | Tranche verticale | **Prise le 31/10/2026 (DP-83, délégation de Mimis)** : Compose et proxy à certificats automatiques ; à relire |
| 89 | Supervision, alertes et canal (ajoutée le 30/10/2026) | Sonde externe et canal d'équipe sans donnée personnelle ; service d'astreinte payant | `DEPLOYMENT.md` §7.3. Précise `ARCHITECTURE.md` §10.3 | Avant le premier pilote | **Prise le 31/10/2026 (DP-83, délégation de Mimis)** : canal unique sans donnée personnelle ; outil à confirmer |
| 90 | Noms d'hôte, domaine et DNS (ajoutée le 30/10/2026) | `app.`, `essai.`, `console.`, `www.`, `status.` ; hôte unique | `DEPLOYMENT.md` §3.1. Dépend de la n° 13 | Avant le premier pilote | Ouverte |
| 91 | Calendrier de livraison pendant le pilote (ajoutée le 30/10/2026) | Zones rouge, orange, verte du mois ; aucune règle | `DEPLOYMENT.md` §4 : gel du 1 au 7, mardi à jeudi, 06 h 00 à 07 h 30. À recalibrer après le premier mois | Avant le premier pilote | Ouverte |
| 92 | Secrets de production (ajoutée le 30/10/2026) | Variables d'environnement et gestionnaire de mots de passe ; coffre dédié | `DEPLOYMENT.md` §2.4 : sans coffre dédié au pilote | Avant le premier pilote | **Prise le 31/10/2026 (DP-83, délégation de Mimis)** : variables d'environnement et gestionnaire de mots de passe ; à relire |
| 93 | Page d'état pour les clients (ajoutée le 30/10/2026) | Page statique hors du serveur principal ; service tiers ; aucune | `DEPLOYMENT.md` §7.5 : page statique, trois états | Avant le premier pilote | Ouverte |
| 94 | Départ d'une agence : export, purge et preuve (ajoutée le 30/10/2026) | Purge sur instruction écrite après délai de grâce ; conservation 10 ans ; conservation limitée puis purge | `DEPLOYMENT.md` §8.2 : purge sur instruction écrite, certificat de destruction. Incohérence entre `SECURITY.md` §6.3 et §8.4, et aucune fonction de purge pour `live`. Juriste | Avant le premier départ (avis juridique, DP-62) | Ouverte |
| 95 | Chiffrement des sauvegardes (ajoutée le 30/10/2026) | Clé publique (clé privée hors ligne) ; clé symétrique | `DEPLOYMENT.md` §5.2 : clé publique. Précise la n° 53 et la variable `BACKUP_ENCRYPTION_KEY` de `CLAUDE.md` | Avant le premier pilote | **Prise le 31/10/2026 (DP-82)** : clé publique, clé privée en deux exemplaires (Mimis et le développeur) |
| 96 | Capacité révisée (ajoutée le 30/10/2026) | Un développeur à 25 h par semaine (DP-54) ; deux développeurs, avec revue croisée à la place de la relecture différée | `ROADMAP.md` §5 : estimation en trois scénarios. La consigne de la roadmap suppose deux développeurs ; DP-54 reste en vigueur tant qu'elle n'est pas remplacée | Avant l'étape 00 | **Prise le 31/10/2026 (DP-72)** : DP-54 maintenue |
| 97 | Pilote en deux temps (ajoutée le 30/10/2026) | Pilote après T1 + T2 (DP-57) ; lot 1 (T1 sans relances, plus import et console en ligne de commande) puis lot 2 avant la bascule `live` ; PP-05 à PP-07 lus « pour le périmètre livré » | `ROADMAP.md` §1 : lot 1 recommandé (premières données réelles environ 7 semaines plus tôt à deux développeurs) | Avant l'étape 00 | **Prise le 31/10/2026 (DP-72)** |
| 98 | Console du pilote en ligne de commande (ajoutée le 30/10/2026) | Ligne de commande (`immo org …`) ; interface web | `AUDIT_COHERENCE_2.md` C2-033 : ligne de commande, `specs/F40-console.md` à l'étape 11 | Étape 11 | Ouverte |
| 99 | Migrations incrémentales par étape (ajoutée le 30/10/2026) | Une migration initiale pour les 63 tables ; migrations par domaine à chaque étape | `ROADMAP.md` : par étape (chaque étape reste vérifiable), `cash_days` créée vide avec `payments` | Étape 01 | Ouverte |
| 100 | Domaine technique provisoire (ajoutée le 30/10/2026) | Attendre le choix du nom (n° 13) ; domaine technique provisoire | Domaine provisoire sous contrôle de l'organisation, cookies sans attribut `Domain` | Avant l'étape 01 | Ouverte |
| 101 | Conditions du pilote (ajoutée le 30/10/2026) | Durée, gratuité, acompte de fin de pilote, critères de réussite | Double tenue de 2 semaines puis suivi à J+30, J+60, J+90 ; gratuité pendant le pilote ; critères de DP-53 | Avant l'étape 11 | Ouverte |
| 102 | Boucle d'apprentissage : grille de priorité et seuils d'entrée d'une tranche (ajoutée le 30/10/2026) | Grille gravité × fréquence × portée ; seuils d'entrée de T3 | `roadmap/etape-19-boucle-apprentissage.md` | Avant l'étape 19 | Ouverte |
| 103 | Bascule `live` conditionnée à la livraison du compte rendu (ajoutée le 30/10/2026) | Bascule dès la fin de la double tenue ; pas de `live` avant l'étape 17 | `ROADMAP.md` §1 : pas de `live` avant l'étape 17 | Avant l'étape 14 | **Prise le 31/10/2026 (DP-72)** |

Les décisions 16, 17 et 18 ne viennent pas de la section 26 du dossier. Les n° 16 et 18 sont prises (DP-44, DP-67) ; la n° 17 est ouverte.

Les décisions 19 à 29 viennent d'`ARCHITECTURE.md` v0.1 (30/10/2026). Pour les décisions n° 5, 6, 7, 14 et 18, `ARCHITECTURE.md` donne une recommandation sans les trancher.

Les décisions 30 à 47 viennent de `DATABASE_SCHEMA.md` v0.1 et les décisions 48 à 60 de `SECURITY.md` v0.1 (30/10/2026). Elles étaient toutes **Ouvertes** à l'origine : leur colonne « Recommandation du dossier » reprend la recommandation du fichier d'origine, qui n'est pas une décision. Celles qui sont prises le sont par une entrée DP-52 à DP-68 (colonne Statut). La décision n° 18 (langue des identifiants) est prise dans le sens « identifiants en anglais » (DP-67).

Les décisions 61 à 68 viennent d'`API_CONTRACTS.md`, les décisions 69 à 77 de `CODING_RULES.md`, les décisions 78 à 85 de `TESTING_QA.md` et les décisions 86 à 95 de `DEPLOYMENT.md` (tous en v0.1, 30/10/2026). Elles étaient toutes **Ouvertes** à l'origine. Aucune n'est prise tant qu'une entrée DP-xx n'existe pas ; les entrées DP-52 à DP-68 du 30/10/2026 en ont pris une partie (colonne Statut).

Les décisions 96 à 103 viennent de `ROADMAP.md` v0.1 (30/10/2026). Elles sont toutes **Ouvertes** : la roadmap est une proposition tant que Mimis ne les a pas tranchées par une entrée DP-xx.

Liens avec les décisions prises :

- n° 2 ↔ DP-01 (cible), DP-57 (premier segment).
- n° 19 ↔ DP-52, DP-53 (stratégie : pilote réel sans enquête). n° 14 ↔ DP-54 (capacité). n° 21 et 46 ↔ DP-55 (Neon).
- n° 4 ↔ DP-09, DP-31 (maquette complète et son périmètre). n° 1 ↔ DP-07, DP-10, DP-28, DP-34 (seuils avant le premier entretien, remplacés par DP-52 et DP-53).
- n° 7 ↔ DP-11 (prototype PDF jetable, moteur encore à choisir).
- n° 6 ↔ DP-06 (backend Rust).
- n° 8 ↔ DP-08, DP-32 (paiements). n° 9 ↔ DP-33 (relances).

## Écarts assumés par rapport au dossier de cadrage

Le dossier de cadrage (`Immo-Suite_Dossier-de-cadrage.docx`) n'est jamais modifié. Tout écart voulu est listé ici avec l'entrée qui le décide. Règle : un écart nouveau est ajouté à cette table avec son entrée DP-xx.

| Écart | Entrée | Section du dossier |
|---|---|---|
| Pas de phase 0 ni de Go/No-Go avant le développement ; pilote réel directement | DP-52, DP-53 | §17 (phase 0), §26 n° 1 |
| Tranches d'ancienneté 1–7, 8–30, 31–59, 60 et + | DP-12 | F11 |
| F19, F23 et F25 en tranche T3 | DP-29 | §9.1, §7.3 |
| Alerte de promesse non tenue dans l'accueil (T1) et non dans l'agenda | DP-36 | F12, F17 |
| Indexation portée par le bail, non par le lot | DP-41 | F04 |
| « Comptable (lecture) » est un rôle avec compte | DP-42 | §3, §11.2 |
| Ordre de construction de T2 : mandats et compte rendu avant l'import | DP-57 | §18 |
| Inscription publique fermée au pilote ; organisations créées par la console | DP-68 | F01 |
| Réinitialisation du mot de passe par lien généré et non par code envoyé | DP-68 | F01 |
| TOTP du gérant obligatoire avant la bascule en `live` (le dossier le range en option P1) | DP-68 | §12, F02 |
| Quittance : format A5 par défaut, deux formats rendus ; rectificative sans effet et code de contrôle hors pilote | DP-64 | F10 |

Écarts proposés non encore décidés : accès support de 24 heures par défaut et 7 jours au plus (n° 54), objectifs de sauvegarde de 15 minutes et 4 heures (n° 24), pilote en deux temps avec démarrage en `setup` avant la fin de T2 (n° 97, écart à DP-57 et à la lecture de PP-05 à PP-07 de `TESTING_QA.md`).

## Modèle d'entrée

À copier pour toute nouvelle décision.

```markdown
### DP-XX — Titre court

- **Date** : JJ/MM/AAAA
- **Contexte** : pourquoi la question se pose, ce qui la déclenche.
- **Décision** : ce qui est décidé, en une ou deux phrases.
- **Alternatives écartées** : option A (pourquoi non) ; option B (pourquoi non).
- **Conséquences** : ce qui change dans le produit, le code, la maquette, les autres documents.
- **Statut** : Prise | Remplacée par DP-YY | Abandonnée
- **Liens** : n° de décision ouverte, identifiants F, H, R concernés.
```

## Règle de mise à jour

- Toute décision structurante prise pendant le projet est ajoutée ici **avant** d'être appliquée dans le code.
- Une décision structurante touche au périmètre, aux règles d'argent, au modèle de données, à la stack, aux prix, au légal ou à l'hébergement.
- Une décision ouverte devient « Prise » seulement quand une entrée DP-XX existe. Mettre alors à jour son statut dans le tableau, avec la date.
- Une recommandation du dossier n'est jamais une décision tant qu'elle n'a pas d'entrée DP-XX.
- Ne jamais supprimer une entrée. Une décision changée est marquée « Remplacée par DP-YY », et la nouvelle entrée cite l'ancienne.
- Le code, les commits et les autres documents citent l'identifiant DP-XX concerné.
- Numérotation : DP-69 et suivants.

## Questions ouvertes

- **Pourquoi ne pas réutiliser la stack de Scolo ?** La raison n'est pas dans le dossier. À documenter dans DP-06.
- **Qui valide les décisions ?** Le dossier ne nomme pas le décideur. Les décisions du 24/10/2026 sont attribuées à Mimis.
