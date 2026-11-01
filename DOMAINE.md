# Immo Suite — Domaine (glossaire et règles métier)

| Champ | Valeur |
|---|---|
| Statut | Propriétaire des règles métier, du glossaire, des statuts et des cas chiffrés |
| Version | 0.3 |
| Date | 30/10/2026 |
| Dépend de | Dossier de cadrage v1.0 (§11, §15, §16), `DECISIONS.md` |
| Alimente | `DATABASE_SCHEMA.md`, `API_CONTRACTS.md`, `UI_UX_DESIGN.md`, `TESTING_QA.md`, `TRACABILITE.md` |

Source : dossier de cadrage v1.0 du 24/10/2026 (§11, §15, §16 surtout). Les références F01–F40 renvoient aux fonctionnalités du dossier. Tout ce qui est déduit et non écrit tel quel dans le dossier est signalé « déduit ».

## Glossaire

### Organisation et acteurs

| Terme | Définition |
|---|---|
| Organisation | Espace isolé d'un client (propriétaire ou agence). Racine de l'isolation des données. Cycle de vie : reprise (`setup`), exploitation (`live`), `suspended`, `closed` (R-110) |
| Plan | Périmètre fonctionnel de l'organisation : Propriétaire ou Agence |
| Propriétaire direct | Propriétaire qui gère lui-même ses biens avec Immo Suite |
| Mandant | Propriétaire pour qui une agence gère des lots. Reçoit un relevé, ne se connecte pas au MVP |
| Mandat | Contrat de gestion entre agence et mandant : taux de commission, base de calcul, dates, lots couverts |
| Commission | Part des loyers encaissés retenue par l'agence, selon le taux du mandat |
| Locataire | Personne ou société qui loue un lot. N'a pas de compte au MVP |
| Gérant | Rôle de direction : tous droits, valide contre-passations et reversements |
| Gestionnaire | Rôle qui suit les lots, les baux et les relances |
| Caissier | Rôle qui encaisse et émet les quittances |
| Comptable (lecture) | Rôle avec compte, en lecture seule, qui peut être un tiers de l'agence (DP-42) |
| Agent terrain | Rôle qui visite et relance, limité aux lots qui lui sont assignés |
| Prestataire | Artisan ou entreprise qui réalise une intervention. Pas de compte |

### Patrimoine

| Terme | Définition |
|---|---|
| Bien | Emplacement : immeuble, cour, site, maison. Archivable, jamais supprimé |
| Lot | Ce qu'on loue : maison, appartement, boutique, entrepôt, bureau. Appartient à un bien |
| Usage | Habitation, commercial ou mixte. Détermine les champs et les règles applicables au lot |
| Loyer de référence | Loyer indicatif du lot. Le loyer du bail seul fait foi pour les échéances |
| Vacant | Lot sans bail en cours |
| Occupé | Lot avec un bail en cours |
| Archivage | Retrait logique d'un bien, lot ou locataire. L'historique reste consultable. Rien ne se supprime |

### Location

| Terme | Définition |
|---|---|
| Bail | Relation lot–locataire : dates, loyer, charges, jour d'échéance, dépôt de garantie, type (habitation / commercial), statut |
| Bail en cours | Bail au statut actif ou en préavis (DP-39) |
| Avenant | Modification datée du loyer ou des charges. Ne touche jamais les échéances passées |
| Loyer | Montant mensuel de base fixé par le bail |
| Charges | Montant prévu au bail en plus du loyer |
| Jour d'échéance | Jour du mois où le loyer est dû |
| Période | Mois couvert par une échéance |
| Échéance | Montant dû pour un bail et une période. Une seule par bail et par période |
| Ligne d'échéance | Composante d'une échéance : loyer, charges, arriéré ou ligne exceptionnelle |
| Arriéré | Dette antérieure importée sous forme de ligne d'échéance de type « arriéré », datée |
| Retard | Échéance non soldée à partir du lendemain de sa date (J+1) |
| Solde | Montant dû moins montant affecté. Un solde impayé = somme des échéances en retard moins les affectations |
| Ancienneté | Nombre de jours de retard d'une échéance |
| Préavis | Période entre la notification d'un départ et la fin du bail. Statut « en préavis » du bail. Durée saisie au bail, sans défaut [À VÉRIFIER AVEC UN JURISTE] |
| Dépôt de garantie (caution) | Somme versée par le locataire à l'entrée. Suivie : réception, retenues motivées, restitution |
| Retenue | Part du dépôt de garantie conservée, avec motif tracé |

### Encaissement

| Terme | Définition |
|---|---|
| Paiement | Encaissement enregistré : locataire, bail, date, montant, mode, référence, auteur. Immuable |
| Mode de paiement | Espèces, mobile money, virement ou chèque |
| Référence externe | Identifiant du paiement hors Immo Suite (mobile money, virement). Unique par mode |
| Affectation | Part d'un paiement imputée à une échéance. La somme des affectations ne dépasse jamais le paiement |
| Paiement partiel | Paiement inférieur au dû. L'échéance devient « partielle » avant J+1, « en retard » ensuite (DP-13) |
| Avance | Excédent d'un paiement, affecté à l'échéance suivante |
| Crédit du bail | Part d'un paiement non encore affectée faute d'échéance générée. S'affecte dès la génération de l'échéance suivante |
| Contre-passation | Écriture inverse, liée au paiement d'origine, qui le corrige. Validée par le gérant |
| Demande de correction | Requête d'un caissier ou gestionnaire pour corriger un paiement. Ne modifie rien seule |
| Clé d'idempotence | Marque d'une soumission qui garantit qu'un double envoi crée un seul paiement |
| Quittance | Preuve de paiement PDF, numérotée sans trou par organisation. Immuable |
| Quittance rectificative | Quittance qui annule et remplace une quittance erronée |
| Journée de caisse | Ensemble des paiements en espèces d'un caissier pour un jour. Espèces attendues, comptées, écart |
| Clôture de caisse | Verrouillage de la journée de caisse après comptage |

### Recouvrement

| Terme | Définition |
|---|---|
| Relance | Contact avec un locataire au sujet d'une échéance, avant l'échéance (rappel à J−3) ou après retard : date, canal (WhatsApp, SMS, appel), modèle, texte envoyé, auteur, résultat. Non modifiable (DP-40) |
| Modèle de message | Texte avec variables rattaché à une étape : J−3, J+1, J+5, J+10. Variables autorisées : `{nom}`, `{montant}`, `{mois}`, `{lot}`, `{date}`, `{lien_quittance}` (R-064) |
| Promesse de paiement | Engagement daté d'un locataire à payer un montant. Non tenue : alerte au gestionnaire |
| Prochaine action | Rappel planifié après une relance |

### Reddition de comptes (plan Agence)

| Terme | Définition |
|---|---|
| Relevé propriétaire (compte rendu) | Document mensuel par mandant : loyers encaissés, commission, dépenses imputées, net à reverser, impayés. Figé une fois validé, sauf réouverture avant reversement (R-083) |
| Reddition de comptes | Fait de rendre compte au mandant de ce qui a été encaissé, retenu et dépensé |
| Net à reverser | Loyers encaissés − commission − dépenses imputées (± ajustements de la période précédente) |
| Reversement | Paiement du net au mandant : date, mode, référence, montant. Immuable |
| Intervention | Travail ou réparation sur un lot : date, nature, coût, prestataire, payeur, pièces jointes |
| Dépense imputable | Dépense d'un lot que le relevé déduit du net à reverser |

### Technique et pilotage

| Terme | Définition |
|---|---|
| Journal d'audit | Trace en écriture seule des actions sensibles : qui, quoi, sur quel objet, avant/après, quand, adresse IP |
| Lot d'import | Fichier importé, tracé, annulable tant que rien n'a été utilisé |
| Taux de recouvrement | Indicateur du tableau de bord : encaissé du mois divisé par attendu du mois, en pourcentage entier (R-103) |

## Statuts et cycles de vie

Le dossier liste certains états et en déduit d'autres. Les états et transitions marqués « déduit » viennent des règles des fonctionnalités, pas d'un diagramme du dossier.

### Lot

| De | Vers | Déclencheur | Source |
|---|---|---|---|
| (création) | Vacant | Création du lot | F04 |
| Vacant | Occupé | Activation d'un bail | F04, F07 |
| Occupé | Vacant | Fin du bail | F04 |
| Vacant | En travaux | Pose manuelle par le gérant ou le gestionnaire. Un lot occupé ne passe pas « en travaux » : il reçoit une intervention (F15) | §15.2, R-093, DP-25 |
| En travaux | Vacant | Retrait manuel à la fin des travaux. Un bail ne peut s'activer qu'après | R-093, DP-25 |
| Tout état sans bail en cours | Archivé | Archivage manuel. Refusé si bail en cours | F04, DP-39 |

### Bail

Statuts : brouillon, actif, en préavis, terminé (DP-24).

| De | Vers | Déclencheur | Source |
|---|---|---|---|
| (création) | Brouillon | Saisie lot, locataire, dates, loyer, jour d'échéance, dépôt | F07 |
| Brouillon | Actif | Activation : échéances générées, lot « occupé ». Refus si chevauchement ou si le lot est « en travaux » | F07, F08, DP-24 |
| Actif | Actif (nouvelle version) | Avenant daté : loyer, charges, ou prolongation de la date de fin (renouvellement). Les échéances passées ne bougent pas | F07, R-091 |
| Actif | En préavis | Saisie d'une date de départ prévue. Le bail reste en cours : les échéances continuent jusqu'au départ | F23, R-090, DP-39 |
| En préavis | Actif | Annulation du préavis, avec trace (déduit) | R-090 |
| Actif ou En préavis | Terminé | Résiliation, ou date de départ atteinte. Plus d'échéances. Lot « vacant » | F07, F23 |

### Échéance (statut recalculé, jamais saisi)

| De | Vers | Déclencheur | Source |
|---|---|---|---|
| (génération) | À venir | Échéance créée, date d'échéance non atteinte | F08 |
| À venir | Due | Jour de la date d'échéance (J0) | F08, R-095, DP-27 |
| À venir ou Due | Partielle | Montant affecté supérieur à 0 et inférieur au dû, avant J+1 | F09 |
| À venir, Due, Partielle ou En retard | Payée | Montant affecté égal au dû | F09 |
| Due ou Partielle | En retard | Date d'échéance + 1 jour atteinte, solde non nul. « En retard » prime sur « partielle » ; l'écran montre le payé et le reste dû | F08, DP-13 |
| Payée ou Partielle | Statut recalculé | Contre-passation d'un paiement affecté (déduit) | F08 |

### Paiement (déduit)

| De | Vers | Déclencheur | Source |
|---|---|---|---|
| (saisie) | Validé | Validation : affectations créées, quittance générée, paiement verrouillé | F09, §11.3 |
| Validé | Contre-passé | Contre-passation validée par le gérant, après une demande de correction | F09 |

Un paiement validé n'a aucune autre transition. La demande de correction est un objet séparé.

### Quittance (déduit)

| De | Vers | Déclencheur | Source |
|---|---|---|---|
| (génération) | Émise | Paiement validé | F10 |
| Émise | Annulée | Contre-passation du paiement (R-096), ou émission d'une quittance rectificative qui la remplace | F10, DP-38 |

### Relevé propriétaire

| De | Vers | Déclencheur | Source |
|---|---|---|---|
| (production) | Brouillon | Choix du propriétaire et de la période. Ajustement possible (ajout d'une dépense) | F13 |
| Brouillon | Validé | Validation par le gérant. Le relevé est figé | F13 |
| Validé | Payé | Enregistrement du reversement (date, mode, référence) | F13, §15.2 |
| Validé | Brouillon (nouvelle version) | Réouverture par le gérant avec motif tracé, seulement si aucun reversement n'est enregistré. Versions précédentes conservées. Impossible depuis « Payé » | F13, R-083, DP-17 |

### Promesse de paiement (déduit)

| De | Vers | Déclencheur | Source |
|---|---|---|---|
| (saisie) | En cours | Promesse notée avec date et montant | F12 |
| En cours | Tenue | Paiement reçu (déduit) | F12 |
| En cours | Non tenue | Lendemain de la date promise sans paiement : alerte au gestionnaire | F12 |

## Règles métier

« Portée » : Tous, Habitation, Commercial ou Agence (plan Agence). Une règle par ligne, testable.

### Fondamentaux

| ID | Règle | Portée | Source |
|---|---|---|---|
| R-001 | Tout montant est un entier de FCFA, sans décimale. Aucun nombre flottant. Bornes : de 1 à 999 999 999 pour un paiement, un loyer, un reversement ; de 0 à 999 999 999 pour les charges, un dépôt de garantie et une dépense (DP-56) | Tous | §12, §15.1 |
| R-002 | Toutes les dates métier suivent le fuseau Africa/Ouagadougou (UTC+0, sans changement d'heure) | Tous | §16, §18 |
| R-003 | Devise (FCFA, XOF) et langue (français) sont des paramètres d'organisation. Aucune autre valeur n'est activée en V1 | Tous | §10, §18 |
| R-004 | Toute donnée métier appartient à une organisation. Une demande sur une ressource d'une autre organisation répond « introuvable » et est journalisée | Tous | F01, §15.1 |
| R-005 | Rien ne se supprime. Un bien, lot ou locataire avec historique est archivé, et son historique reste visible. Seules exceptions : l'annulation d'un import non utilisé (R-080) et la remise à zéro d'une organisation en phase `setup` (R-110, DP-60) | Tous | F04, §15.1 |
| R-006 | Archiver un lot qui a un bail en cours est refusé, avec explication | Tous | F04, DP-39 |
| R-007 | Le journal d'audit est en écriture seule. Aucune modification ni suppression, même par le gérant ou HABEK, sauf avec l'organisation lors de sa remise à zéro en phase `setup` (R-110, DP-60) | Tous | F03 |
| R-008 | Si l'écriture dans le journal d'audit échoue, l'action métier est annulée | Tous | F03 |
| R-009 | Toute consultation d'un dossier client par le support HABEK crée une ligne d'audit « accès support ». L'accès expire sans action manuelle | Tous | F03, F40 |
| R-110 | Une organisation a un cycle de vie : `setup` (reprise : import, essais, double tenue avec l'ancien système), `live` (bascule), `suspended`, `closed`. En `setup`, les quittances sont de série d'essai et HABEK peut remettre l'organisation à zéro avec l'accord écrit de l'agence. Après la bascule, repartir de zéro est une nouvelle organisation | Tous | DP-60, DP-61 |

### Patrimoine, locataires, mandats

| ID | Règle | Portée | Source |
|---|---|---|---|
| R-010 | Une maison unique crée automatiquement 1 bien + 1 lot | Tous | F04 |
| R-011 | La référence d'un lot est unique dans son bien. Un doublon est refusé | Tous | F04 |
| R-012 | Le type de lot détermine les champs : un lot « boutique » propose « activité autorisée » ; un lot d'habitation non. L'indexation est portée par le bail (R-025) | Habitation, Commercial | F04, DP-41 |
| R-013 | Un lot passe à « occupé » à l'activation d'un bail et à « vacant » à la fin de son dernier bail en cours | Tous | F04, F07, DP-39 |
| R-014 | Un lot est couvert par un seul mandat actif. Un chevauchement de mandats sur un lot est refusé | Agence | F05 |
| R-015 | Le taux de commission est compris entre 0 % et 100 %. Hors plage, refusé | Agence | F05 |
| R-016 | Dans le plan Propriétaire, le propriétaire est créé automatiquement comme mandant. Mandats et commissions ne sont pas disponibles | Tous | §9.2, §15.2 |
| R-017 | Le téléphone d'un locataire est unique par organisation parmi les fiches non archivées ; deux fiches ne partagent pas un numéro au pilote. Un doublon propose d'ouvrir la fiche existante ou de fusionner (DP-58) [à confirmer avec l'agence pilote à la découverte] | Tous | F06 |
| R-018 | Un numéro de téléphone invalide (format +226 attendu) est refusé avec un message explicite | Tous | F06 |
| R-019 | La pièce d'identité d'un locataire est facultative, chiffrée et masquée par défaut | Tous | F06, §12.1 |
| R-093 | Le statut « en travaux » se pose à la main sur un lot vacant. Un bail ne peut pas s'activer sur un lot « en travaux ». Un lot occupé garde « occupé » et reçoit seulement une intervention | Tous | §15.2, F15, DP-25 |
| R-094 | Un lot d'usage « mixte » est accepté. Le type du bail (habitation ou commercial), choisi à sa création, détermine les règles et les champs applicables. Il n'existe pas de bail « mixte » | Habitation, Commercial | §15.2, DP-25 |

### Baux

| ID | Règle | Portée | Source |
|---|---|---|---|
| R-020 | Un lot a un seul bail en cours (actif ou en préavis) à une date donnée. Un second bail en cours ou un chevauchement est refusé | Tous | F04, F07, DP-39 |
| R-021 | La date de fin d'un bail ne précède pas sa date de début. Sinon refus | Tous | F07 |
| R-022 | Le loyer d'un bail est strictement positif. Zéro ou négatif refusé | Tous | F07 |
| R-023 | Un changement de loyer ou de charges est un avenant daté. Il n'est jamais rétroactif : les échéances passées ne changent pas | Tous | F07, §15.1 |
| R-024 | Dépôt de garantie supérieur à deux mois de loyer : avertissement non bloquant à la validation [À VÉRIFIER AVEC UN JURISTE] | Habitation | F07 |
| R-025 | Le bail commercial porte des champs en plus : indexation et durée. L'activité autorisée est portée par le lot (R-012). Le produit ne génère pas un bail commercial juridiquement complet [À VÉRIFIER AVEC UN JURISTE] | Commercial | F04, F07, §2.4, DP-41 |
| R-026 | Pas d'indexation ni de pénalité automatiques. Alerte « révision à étudier » pour l'indexation. Champ libre « frais de retard » saisi à la main | Tous | §10 |
| R-090 | Statuts du bail : brouillon, actif, en préavis, terminé. En préavis, le bail reste en cours (R-020) et les échéances continuent jusqu'à la date de départ. La durée du préavis est saisie au bail, sans valeur par défaut [À VÉRIFIER AVEC UN JURISTE] | Tous | F07, F23, DP-24 |
| R-091 | Renouveler un bail = avenant daté qui prolonge la date de fin. Même bail, même dépôt de garantie, même historique. Un changement de locataire est un nouveau bail | Tous | F07, F23, DP-24 |
| R-092 | Aucun calcul de TVA ni d'impôt. Le loyer est le montant dû tel que saisi. Un indicateur facultatif « TVA incluse » s'affiche sur le bail commercial et la quittance. Un loyer variable passe par des avenants datés ou une ligne exceptionnelle, sans échéance variable automatique [À valider, H7] | Commercial | §2.4, F07, DP-26 |
| R-104 | Un avenant s'applique aux échéances dont la date d'échéance est postérieure à la fois à sa date d'effet et à sa date de saisie ; les échéances échues non payées gardent l'ancien montant (précise R-023) [à confirmer avec l'agence pilote à la découverte] | Tous | F07, DP-58 |

### Échéances

| ID | Règle | Portée | Source |
|---|---|---|---|
| R-027 | Il existe une seule échéance par bail et par période | Tous | F08, §15.2 |
| R-028 | L'activation d'un bail génère ses échéances | Tous | F07 |
| R-029 | Une tâche planifiée génère chaque jour les échéances des 30 jours à venir. Elle est idempotente : l'exécuter deux fois ne crée aucun doublon | Tous | F08 |
| R-030 | Un bail terminé ne reçoit plus de nouvelle échéance | Tous | F08 |
| R-031 | Sans jour d'échéance au bail, la valeur par défaut est le 1er du mois, et elle est signalée | Tous | F08 |
| R-032 | Statuts d'une échéance : à venir, due, partielle, payée, en retard | Tous | F08 |
| R-033 | Une échéance au solde non nul est « en retard » dès le lendemain de sa date (J+1), qu'elle soit partiellement payée ou non. « En retard » prime sur « partielle » et l'écran montre le montant payé et le reste dû | Tous | F08, DP-13 |
| R-034 | Le statut d'une échéance est recalculé à chaque affectation de paiement. Aucun cache obsolète | Tous | F08, F11 |
| R-035 | Les arriérés importés sont des lignes d'échéance de type « arriéré », datées | Tous | F08, F14 |
| R-036 | Une ligne exceptionnelle (arriéré, régularisation) est ajoutée par le gérant ou le gestionnaire. Annuler une ligne exige un motif | Tous | F08 |
| R-095 | Une échéance est « à venir » avant sa date d'échéance, « due » le jour de cette date (J0), « en retard » dès J+1 si le solde n'est pas nul | Tous | F08, DP-27 |
| R-098 | Si le jour d'échéance du bail (29, 30 ou 31) n'existe pas dans le mois, la date d'échéance est le dernier jour du mois [à confirmer avec l'agence pilote à la découverte] | Tous | F08, DP-58 |
| R-099 | Le premier mois d'un bail activé en cours de mois est dû en entier, sans prorata [à confirmer avec l'agence pilote à la découverte] | Tous | F08, DP-58 |
| R-100 | Un bail activé avec une date de début passée génère ses échéances à partir du mois d'activation ; les périodes antérieures se saisissent comme lignes exceptionnelles (R-036) [à confirmer avec l'agence pilote à la découverte] | Tous | F07, F08, DP-58 |

### Paiements

| ID | Règle | Portée | Source |
|---|---|---|---|
| R-037 | Le montant d'un paiement est un entier strictement positif | Tous | F09 |
| R-038 | Un paiement supérieur à un seuil (ex. 1 000 000 FCFA) demande une confirmation [valeur à fixer] | Tous | F09 |
| R-039 | La date d'un paiement peut précéder la date de saisie mais jamais être future | Tous | F09 |
| R-040 | Modes de paiement acceptés : espèces, mobile money saisi, virement, chèque | Tous | F09 |
| R-041 | La référence externe est unique par mode. Un doublon est signalé : « Ce paiement semble déjà enregistré ». Comportement exact : R-102 | Tous | F09, §15.2 |
| R-042 | Deux soumissions identiques (double clic, reprise après coupure réseau) créent un seul paiement, via une clé d'idempotence | Tous | F09 |
| R-043 | À l'enregistrement, le paiement est affecté par défaut aux échéances les plus anciennes. L'utilisateur peut ajuster. Ordre précis : R-087 | Tous | F09, §11.3 |
| R-044 | La somme des affectations d'un paiement ne dépasse jamais son montant | Tous | §15.2, §19 |
| R-045 | Un paiement partiel est autorisé. Avant J+1, l'échéance passe à « partielle » avec le reste dû. Après J+1, elle reste « en retard » (R-033) | Tous | F09, DP-13 |
| R-046 | L'excédent d'un paiement est une avance, affectée à l'échéance suivante. Un montant supérieur au solde propose l'avance. Sans échéance suivante : R-086 | Tous | F09 |
| R-047 | Un paiement validé est immuable. Toute modification est refusée | Tous | F09, §12.1 |
| R-048 | Un caissier ou gestionnaire ne peut que demander une correction. Seul le gérant contre-passe, par une écriture inverse liée à l'original et tracée | Tous | F09, F02, §11.2 |
| R-086 | La part d'un paiement qu'aucune échéance existante ne peut recevoir est un crédit du bail, visible. Dès qu'une échéance est générée, le crédit s'y affecte automatiquement. Si le bail se termine, le crédit reste visible et se règle hors flux, avec trace | Tous | F09, F08, DP-20 |
| R-087 | L'affectation suit la date d'échéance (arriérés inclus, ils sont datés). À date égale : loyer, puis charges, puis lignes exceptionnelles. L'utilisateur peut ajuster | Tous | F09, DP-21 |
| R-088 | Un paiement est rattaché au bail, pas à un seul locataire. Le champ « payé par » désigne un locataire du bail ou un tiers (nom, téléphone facultatif). La quittance est au nom du ou des locataires du bail et mentionne le tiers s'il y en a un | Tous | F09, F10, DP-22 |
| R-089 | Un paiement en espèces saisi après la clôture de sa journée de caisse entre dans la journée ouverte du jour de saisie. Sa date de paiement réelle est conservée pour l'échéance et le relevé. Le caissier est averti de l'écart entre les deux dates | Tous | F19, F09, DP-23 |
| R-102 | Une référence externe déjà utilisée pour le même mode déclenche un avertissement qui bloque l'enregistrement tant que l'utilisateur ne confirme pas explicitement ; la référence d'un paiement contre-passé peut être réutilisée [à confirmer avec l'agence pilote à la découverte] | Tous | F09, R-041, DP-58 |

### Quittances

| ID | Règle | Portée | Source |
|---|---|---|---|
| R-049 | La validation d'un paiement génère une quittance, une seule par paiement. Si le PDF échoue, le paiement reste enregistré, la quittance est mise en file avec nouvel essai et alerte visible | Tous | F10, DP-15 |
| R-050 | La numérotation des quittances est continue et sans trou, par organisation et par année (préfixe + année + séquence) ; la numérotation est tenue par série (`trial` en phase `setup`, `official` après la bascule) et par année d'émission. La séquence repart à 1 chaque année. Deux paiements successifs d'une même série et d'une même année ont des numéros consécutifs (DP-61) | Tous | F10, §12.1, DP-15, DP-61 |
| R-051 | Une quittance est immuable. Une erreur se corrige par une quittance rectificative qui annule et remplace. La rectificative reçoit son propre numéro suivant et référence l'originale | Tous | F10, DP-15 |
| R-052 | Mentions de la quittance : identité du bailleur ou de l'agence, du locataire, du lot, période, montant, solde restant | Tous | F10 [À VÉRIFIER AVEC UN JURISTE, décision n° 12] |
| R-053 | Le montant de la quittance est écrit en chiffres et en lettres (orthographe traditionnelle, formulation « francs CFA », DP-64) [À VÉRIFIER AVEC UN JURISTE] | Tous | F10 |
| R-054 | Un lien de partage de quittance expire après une durée configurable et ne donne accès qu'à ce document | Tous | F10 |
| R-055 | Chaque envoi d'une quittance est consigné dans son historique | Tous | F10 |
| R-056 | La quittance est imprimable en A4 et A5 ; A5 par défaut, les deux formats sont rendus à chaque paiement (DP-64) | Tous | §14.3 |
| R-096 | La contre-passation d'un paiement passe sa quittance à « Annulée ». Le paiement de remplacement génère sa propre quittance, marquée rectificative, qui référence la quittance annulée [à confirmer avec l'agence pilote à la découverte] | Tous | F09, F10, R-048, R-051, DP-38 |

### Impayés et relances

| ID | Règle | Portée | Source |
|---|---|---|---|
| R-057 | Le solde impayé = somme des échéances en retard moins les affectations | Tous | F11 |
| R-058 | Tranches d'ancienneté du tableau des impayés : 1–7 j, 8–30 j, 31–59 j, 60 j et +. Un retard de 60 jours est dans la dernière tranche | Tous | F11, DP-12 |
| R-059 | Une promesse non échue est affichée mais ne masque pas la ligne d'impayé | Tous | F11 |
| R-060 | Étapes de relance : rappel courtois à J−3, puis modèles différents à J+1, J+5, J+10 | Tous | §11.4, §13.3 |
| R-061 | Au MVP, la relance est assistée : WhatsApp ou SMS s'ouvre avec le texte prêt, l'utilisateur confirme l'envoi | Tous | F12, §5 |
| R-062 | Chaque relance est journalisée (auteur, canal, texte envoyé) et non modifiable | Tous | F12, §15.2 |
| R-063 | Une seconde relance le même jour au même locataire déclenche un avertissement non bloquant | Tous | F12 |
| R-064 | Un modèle de message refuse les variables inconnues (liste autorisée : voir le glossaire « Modèle de message »). Une variable manquante bloque l'envoi avec un message précis | Tous | F12 |
| R-065 | Une promesse non tenue génère une alerte au gestionnaire dès le lendemain de la date promise, visible dans son accueil (S03) et dans la liste des impayés (S05) | Tous | F12, DP-36 |
| R-097 | Les échéances qui arrivent à J−3 sont proposées au gestionnaire pour un rappel assisté (même panneau que la relance). Aucun envoi automatique | Tous | §11.4, F12, DP-40 |
| R-101 | Une promesse de paiement est « tenue » quand un paiement au moins égal au montant promis est enregistré pour le bail à la date promise ou avant ; sinon elle est « non tenue » dès le lendemain (R-065). La clôture manuelle reste possible [à confirmer avec l'agence pilote à la découverte] | Tous | F12, DP-58 |
| R-103 | Taux de recouvrement = encaissé du mois divisé par attendu du mois, affiché en pourcentage entier arrondi à l'entier le plus proche [à confirmer avec l'agence pilote à la découverte] | Tous | F11, DP-58 |

### Dépôt de garantie

| ID | Règle | Portée | Source |
|---|---|---|---|
| R-066 | Le dépôt de garantie est suivi : montant, date de réception, retenues, date de restitution. Plafond d'habitation : voir R-024 | Tous | F07, F20, §15.2 |
| R-067 | Toute retenue sur le dépôt de garantie est motivée et tracée | Tous | §15.2 |

### Commission et reddition de comptes

| ID | Règle | Portée | Source |
|---|---|---|---|
| R-068 | Commission = taux du mandat × total des loyers encaissés du mandat sur la période, arrondie à l'entier de FCFA le plus proche (0,5 vers le haut). Le taux peut être décimal (ex. 7,5 %). Un seul arrondi par relevé et par mandant. Base par défaut : lignes de type loyer encaissées dans la période, charges exclues. Les arriérés et avances encaissés comptent à leur date d'encaissement. Le mandat peut choisir la base « loyer + charges » | Agence | F05, DP-14, DP-16 |
| R-069 | Un changement de taux n'est pas rétroactif. Un avenant de taux daté du 1er du mois suivant ne change pas le relevé du mois courant | Agence | F05 |
| R-070 | Net à reverser = loyers encaissés − commission − dépenses imputées (± ajustements de la période précédente) | Agence | F13 |
| R-071 | Un loyer entre dans le relevé de la période de sa date d'encaissement [à confirmer avec l'agence pilote à la découverte] | Agence | F13, décision n° 10, H8 |
| R-072 | Un loyer encaissé ne figure que dans un seul relevé | Agence | F13 |
| R-073 | Un relevé validé est figé. Un paiement saisi après sa validation, même daté de sa période, apparaît dans le relevé suivant. Seule exception : la réouverture (R-083) | Agence | F13, DP-17 |
| R-074 | L'export PDF d'un relevé est identique à sa dernière version validée | Agence | F13 |
| R-075 | Un lot qui a un encaissement dans la période et aucun mandat valide bloque la production du relevé, avec la liste des lots concernés (DP-59) | Agence | F13 |
| R-076 | Seul le gérant valide un relevé et enregistre un reversement | Agence | F13, §11.2 |
| R-077 | Un reversement est immuable : date, mode, référence, montant | Agence | §15.2 |
| R-078 | Une dépense porte un payeur (propriétaire, agence ou locataire) et un statut d'imputation au relevé. Sans pièce jointe : avertissement non bloquant. Voir R-084 pour l'imputation par défaut | Agence | F13, §15.2 |
| R-083 | Le gérant peut rouvrir un relevé validé tant qu'aucun reversement n'est enregistré. Motif obligatoire et tracé, nouvelle version, versions précédentes conservées. Après un reversement, la réouverture est refusée et toute correction va en période suivante | Agence | F13, DP-17 |
| R-084 | Une dépense payée par l'agence est imputable au relevé par défaut. Une dépense payée par le propriétaire ou le locataire est non imputable par défaut et figure à titre d'information. Le gérant peut changer l'imputation, avec trace | Agence | F13, §15.2, DP-18 |
| R-085 | Le dépôt de garantie n'entre ni dans les loyers encaissés, ni dans la base de commission. Il est suivi à part (F20) et peut figurer au relevé à titre d'information | Agence | F20, DP-19 [à confirmer avec l'agence pilote à la découverte] |
| R-105 | Sous un mandat à base « loyer », les charges encaissées sont reversées au mandant (comprises dans le net) ; seule la commission suit la base [à confirmer avec l'agence pilote à la découverte] | Agence | F13, DP-59 |
| R-106 | Une base de commission négative (contre-passations supérieures aux encaissements) refuse la production du relevé, avec un message. Un net négatif ne donne lieu à aucun reversement [à confirmer avec l'agence pilote à la découverte] | Agence | F13, DP-59 |
| R-107 | Un relevé retient les dépenses imputables (R-084) datées dans la période et de statut « terminée » [à confirmer avec l'agence pilote à la découverte] | Agence | F13, F15, DP-59 |
| R-108 | Un relevé ne reçoit qu'un reversement, du montant exact du net (précise R-077) [à confirmer avec l'agence pilote à la découverte] | Agence | F13, DP-59 |
| R-109 | Le relevé d'un mandant retient les lots couverts par un mandat qui recouvre la période, y compris un mandat clôturé en cours de mois [à confirmer avec l'agence pilote à la découverte] | Agence | F13, DP-59 |

### Caisse, import, accès

| ID | Règle | Portée | Source |
|---|---|---|---|
| R-079 | La clôture de caisse compare espèces attendues et comptées, calcule l'écart et verrouille la journée. Version simple dans le plan Propriétaire (P1) | Tous | F19, §15.2 |
| R-080 | Un import ne crée rien avant confirmation. Les doublons (même téléphone, même lot) sont proposés à la fusion. L'import est annulable tant que rien n'a été utilisé | Tous | F14 |
| R-081 | Les droits sont vérifiés côté serveur. Le dernier gérant actif ne peut pas être désactivé. Un agent terrain ne voit que ses lots assignés | Tous | F02 |
| R-082 | Le plan détermine les modules visibles : mandats, commissions et clôture de caisse complète sont réservés au plan Agence | Tous | F01 |

### Index des règles par identifiant

Les identifiants sont stables et cités dans tous les fichiers. Les règles ajoutées après coup se trouvent dans leur tableau thématique ; cet index donne les plages de chaque section.

| Section | Identifiants |
|---|---|
| Fondamentaux | R-001 à R-009, R-110 |
| Patrimoine, locataires, mandats | R-010 à R-019, R-093 à R-094 |
| Baux | R-020 à R-026, R-090 à R-092, R-104 |
| Échéances | R-027 à R-036, R-095, R-098 à R-100 |
| Paiements | R-037 à R-048, R-086 à R-089, R-102 |
| Quittances | R-049 à R-056, R-096 |
| Impayés et relances | R-057 à R-065, R-097, R-101, R-103 |
| Dépôt de garantie | R-066 à R-067 |
| Commission et reddition de comptes | R-068 à R-078, R-083 à R-085, R-105 à R-109 |
| Caisse, import, accès | R-079 à R-082 |

## Exemples chiffrés de référence

Cas de test à conserver tels quels. Montants en FCFA.

| ID | Cas | Données | Résultat attendu | Règles | Source |
|---|---|---|---|---|---|
| E-01 | Première échéance | Bail à 75 000, échéance le 5, activé le 1er mars | Échéance de mars au 5 mars pour 75 000 | R-028 | F07 |
| E-02 | Avenant | Avenant à 80 000 à partir d'avril | Mars reste à 75 000, avril passe à 80 000 | R-023 | F07 |
| E-03 | Génération idempotente | Tâche exécutée deux fois le même jour | Une seule échéance par période | R-027, R-029 | F08 |
| E-04 | Passage en retard | Échéance non payée, date d'échéance + 1 jour atteinte | Statut « en retard » | R-033 | F08 |
| E-05 | Paiement partiel | Loyer dû 75 000, paiement de 50 000 en espèces | Échéance « partielle », 25 000 restants | R-045 | F09 |
| E-06 | Avance | Loyer 75 000, paiement de 100 000 | 75 000 affectés au loyer, 25 000 d'avance sur l'échéance suivante | R-046 | F09 |
| E-07 | Double clic | Deux soumissions identiques | Un seul paiement créé | R-042 | F09 |
| E-08 | Paiement validé modifié | Un caissier tente de modifier un paiement validé | Action refusée, demande de correction proposée | R-047, R-048 | F09 |
| E-09 | Seuil de confirmation | Paiement supérieur à 1 000 000 | Confirmation demandée (seuil indicatif) | R-038 | F09 |
| E-10 | Numérotation | Deux paiements validés successivement | Quittances aux numéros consécutifs, sans trou | R-050 | F10 |
| E-11 | Montant en lettres | Paiement de 75 000 | Quittance avec le montant en chiffres et en lettres (« francs CFA », orthographe traditionnelle, DP-64) | R-053 | F10 |
| E-12 | Lien expiré | Lien de quittance ouvert après expiration | Accès refusé | R-054 | F10 |
| E-13 | Tranches d'ancienneté | Trois échéances en retard de 3, 20 et 70 jours | Trois tranches : 1–7 j, 8–30 j et 60 j et + | R-058 | F11 |
| E-13b | Frontière de tranche (dérivé, DP-12) | Retards de 30, 31, 59 et 60 jours | Tranches 8–30 j, 31–59 j, 31–59 j, 60 j et + | R-058 | DP-12 |
| E-14 | Relance courtoise | Impayé de 75 000, modèle « courtois » | Texte avec le nom, le montant et le mois exacts | R-060, R-064 | F12 |
| E-15 | Promesse non tenue | Promesse au 12, non tenue | Alerte visible dans l'accueil du gestionnaire (S03) le 13 | R-065 | F12, DP-36 |
| E-16 | Commission | Mandat à 8 %, loyer encaissé de 100 000 | Commission de 8 000 | R-068 | F05 |
| E-17 | Avenant de taux | Avenant de taux daté du 1er du mois suivant | Relevé du mois courant au taux ancien | R-069 | F05 |
| E-18 | Relevé propriétaire | 400 000 encaissés, commission 8 %, dépenses 30 000 | Commission 32 000, net à reverser 338 000 | R-070 | F13 |
| E-19 | Paiement après validation | Relevé validé, nouveau paiement daté de la même période | Absent de ce relevé, présent dans le suivant | R-073 | F13 |
| E-20 | Import | Fichier de 200 lignes dont 3 erreurs | 197 lignes créables (valides ou avec avertissement, dont 2 doublons de téléphone proposés à la fusion), 3 erreurs listées avec leur numéro | R-080 | F14 |
| E-21 | Import annulé | Import confirmé puis annulé avant tout paiement | Données importées disparues du patrimoine | R-080 | F14 |
| E-22 | Dépôt de garantie | Bail d'habitation, dépôt supérieur à deux mois de loyer | Avertissement non bloquant à la validation | R-024 | F07 |
| E-23 | Agence de 60 lots (fictif) | Loyer moyen 60 000, commission 8 %, abonnement 30 000 | 3 600 000 gérés par mois, 288 000 de revenu de gestion, abonnement ≈ 10 % | — | §22.2 |
| E-24 | Propriétaire de 15 lots (fictif) | Loyer moyen 60 000, abonnement 15 000 | 900 000 par mois, abonnement ≈ 1,7 % | — | §22.2 |
| E-25 | Partielle en retard (dérivé, DP-13) | Loyer 75 000, 50 000 payés, J+1 dépassé | Statut « en retard », payé 50 000, reste dû 25 000, présent dans le tableau des impayés | R-033, R-045 | DP-13 |
| E-26 | Arrondi de commission (dérivé, DP-14) | Taux 7,5 %, 333 333 encaissés | 24 999,975 arrondi à 25 000 | R-068 | DP-14 |
| E-27 | Numérotation annuelle (dérivé, DP-15) | Dernier paiement de 2026 numéroté 000812, premier de 2027 | Séquence 2027 repart à 000001 | R-050 | DP-15 |
| E-28 | Base de commission (dérivé, DP-16) | Mandat à 8 %, base par défaut. Encaissés : 100 000 de loyer et 10 000 de charges | Commission 8 000 (base 100 000). Avec la base « loyer + charges » : 8 800 | R-068 | DP-16 |
| E-29 | Dépense selon le payeur (dérivé, DP-18) | Deux dépenses de 30 000 : l'une payée par l'agence, l'autre par le propriétaire | Seule la première est déduite du net par défaut | R-084 | DP-18 |
| E-30 | Réouverture (dérivé, DP-17) | Relevé validé, aucun reversement ; puis reversement enregistré | Réouverture acceptée avec motif ; après reversement, réouverture refusée | R-083 | DP-17 |
| E-31 | Crédit du bail (dérivé, DP-20) | Loyer 75 000 dû, paiement de 100 000, aucune échéance suivante générée | 75 000 affectés, 25 000 de crédit du bail. À la génération de l'échéance suivante (75 000) : 25 000 affectés, reste dû 50 000 | R-086 | DP-20 |
| E-32 | Ordre d'affectation (dérivé, DP-21) | Échéance du 5 : loyer 75 000 et charges 10 000, paiement de 50 000 | 50 000 sur le loyer, reste 25 000 de loyer et 10 000 de charges. Base de commission de ce paiement : 50 000 | R-087, R-068 | DP-21 |
| E-33 | Paiement par un tiers (dérivé, DP-22) | Paiement de 75 000 fait par le frère du locataire | Paiement rattaché au bail, « payé par » = tiers, quittance au nom du locataire avec mention du tiers | R-088 | DP-22 |
| E-34 | Saisie après clôture (dérivé, DP-23) | Espèces reçues le 10, saisies le 11, journée du 10 clôturée | Paiement daté du 10, compté dans la journée de caisse du 11, avertissement affiché | R-089 | DP-23 |
| E-35 | Renouvellement (dérivé, DP-24) | Bail se terminant le 31 décembre, avenant prolongeant au 30 juin suivant | Même bail, échéances générées jusqu'en juin, dépôt de garantie inchangé | R-091 | DP-24 |
| E-36 | Bail sur lot en travaux (dérivé, DP-25) | Activation d'un bail sur un lot « en travaux » | Refus. Succès après retour du lot à « vacant » | R-093 | DP-25 |
| E-37 | Due / en retard (dérivé, DP-27) | Échéance du 5, solde non nul | Le 4 : à venir. Le 5 : due. Le 6 : en retard | R-095, R-033 | DP-27 |
| E-38 | Contre-passation et quittance (dérivé, DP-38) | Paiement de 57 000 saisi à tort pour un loyer de 75 000, quittance Q ; contre-passation ; nouveau paiement de 75 000 | Q passe à « Annulée ». Le nouveau paiement a sa quittance Q', rectificative, avec son propre numéro suivant, qui référence Q | R-096, R-051, R-050 | DP-38 |
| E-39 | Jour 31 en mois court (dérivé, DP-58) | Bail avec jour d'échéance 31, période de février 2027 | Échéance au 28/02/2027 | R-098 | DP-58 |
| E-40 | Premier mois partiel (dérivé, DP-58) | Bail activé le 20 mars, loyer 75 000 | Échéance de mars de 75 000, entière | R-099 | DP-58 |
| E-41 | Promesse tenue ou non (dérivé, DP-58) | Promesse de 35 000 au 20/11 ; paiement de 35 000 le 19/11, ou de 20 000 le 20/11 | Tenue dans le premier cas ; non tenue le 21/11 dans le second | R-101, R-065 | DP-58 |
| E-42 | Doublon de référence (dérivé, DP-58) | Deuxième paiement mobile money avec une référence déjà utilisée | Avertissement bloquant jusqu'à confirmation ; la référence d'un paiement contre-passé est réutilisable | R-102, R-041 | DP-58 |
| E-43 | Taux de recouvrement (dérivé, DP-58) | Attendu du mois 1 000 000, encaissé 860 000 | 86 % | R-103 | DP-58 |
| E-44 | Avenant et échéance échue (dérivé, DP-58) | Avenant à 80 000 d'effet au 01/12, saisi le 15/12 ; échéance du 05/12 non payée | Décembre reste à 75 000 ; l'échéance du 05/01 passe à 80 000 | R-104, R-023 | DP-58 |
| E-45 | Charges sous base loyer (dérivé, DP-59) | Mandat à 8 %, base loyer ; 100 000 de loyer et 10 000 de charges encaissés | Commission 8 000 ; 102 000 avant dépenses (110 000 − 8 000), charges reversées | R-105, R-068 | DP-59 |
| E-46 | Reversement exact (dérivé, DP-59) | Net à reverser 338 000 | Reversement de 338 000 accepté ; de 337 999 refusé | R-108 | DP-59 |
| E-47 | Remise à zéro (dérivé, DP-60) | Organisation en `setup`, puis en `live` | Remise à zéro acceptée en `setup` avec accord écrit ; refusée en `live` | R-005, R-110 | DP-60 |
| E-48 | Série d'essai (dérivé, DP-61) | Organisation en `setup` : quittances `ESSAI-…` ; bascule en `live` | La première quittance officielle de l'année porte le numéro 000001 | R-050, R-110 | DP-61 |

## Règles à confirmer avec l'agence pilote

Tout ce que le dossier marque [À valider] ou « à confirmer ». Depuis DP-52, ces points se confirment à la découverte de l'agence pilote (`DEPLOYMENT.md` §8.1 étape 2), non par une enquête préalable. Les règles R-098 à R-109 (DP-58, DP-59) sont des valeurs par défaut retenues en attendant.

- **Règle de reddition de comptes** : date d'encaissement (recommandation) ou date d'échéance. R-071, décision n° 10, H8.
- **Plafond du dépôt de garantie en habitation** : deux mois de loyer d'après la presse. R-024 [À VÉRIFIER AVEC UN JURISTE].
- **Bail d'habitation** : contrat écrit, mentions de la quittance, décret de 2018 sur les travaux et la révision du loyer [À VÉRIFIER AVEC UN JURISTE].
- **Bail commercial** (droit OHADA) : durée, indexation, caution, activité. R-025 [À valider].
- **Données personnelles** : déclaration ou autorisation CIL, hébergement hors du pays (H12) [À VÉRIFIER AVEC UN JURISTE] (CIL).
- **Fiscalité des revenus fonciers** : le MVP ne calcule rien [À valider].
- **Seuil de confirmation d'un paiement** : 1 000 000 FCFA est un exemple. R-038.
- **Commission** : le dossier suppose que les règles varient (base, plafond, minimum, barème). À confirmer avec l'agence pilote avant de figer R-068.
- **Étapes de relance** (J−3, J+1, J+5, J+10) et canal : issues d'un workflow reconstitué, non observé (§4, H9).
- **Format des SMS mobile money** (Orange Money, Moov) : des exemples réels sont nécessaires pour F25.
- **Plafond de lots du plan Propriétaire** [À valider].
- **Contre-passation et quittance** : annulation de la quittance d'origine et quittance rectificative du paiement de remplacement. R-096 [à confirmer avec l'agence pilote à la découverte].
- **Exceptions métier à observer** (checklist du dossier §20) :
  - Loyers payés en avance ou par tranches.
  - Paiements par un tiers.
  - Colocation.
  - Cautions de plusieurs mois.
  - Remises, réductions ou gratuités temporaires.
  - Baux verbaux.
  - Locaux commerciaux avec loyers variables ou charges à part.
  - Changement de propriétaire en cours de bail.
  - Loyers en devise.

## Questions ouvertes

- **Durée du préavis.** Habitation et commercial : durée légale [À VÉRIFIER AVEC UN JURISTE] (R-090).
- **Dépôt de garantie.** Qui détient la caution : l'agence ou le propriétaire ? Quel est le délai de restitution ? Y a-t-il un plafond en commercial ? Le relevé doit-il l'afficher ?
- **TVA et loyers variables en commercial.** R-092 ne calcule rien. À confronter aux locaux commerciaux réels (H7) : un loyer variable ou une TVA obligatoire changent-ils le besoin ?
- **Colocation : responsabilité.** Les colocataires sont-ils solidaires du loyer ? Le dossier ne le dit pas, et la relance cible quel locataire ?
- **Taux de recouvrement.** Formule retenue par défaut en R-103 (encaissé ÷ attendu du mois) ; à confirmer avec l'agence pilote.
- **Identité d'un lot à l'import (31/10/2026, DP-80).** Dans le fichier de référence de la maquette, 197 locataires créables se répartissent sur 36 lots, chacun avec 2 à 6 locataires simultanés et aucune date de sortie : si « Maison/Ref » identifie un lot, cela heurte « un lot, un seul bail actif » (R-020). Colocation, référence non unique ou donnée de démonstration ? Les données de la maquette sont synthétiques ; la réponse vient des fichiers réels de l'agence pilote (C-09). Piste, sans décision : ajouter une colonne « Bien » au modèle d'import. Ne pas inventer de règle de fusion de lots avant la réponse.
- **Date ambiguë à l'import (DP-80).** JJ/MM/AAAA est le format du produit ; une date comme `03/04/2026` est signalée, non refusée, quand aucune date de la colonne ne lève le doute. À confirmer avec l'agence pilote que les carnets suivent bien l'ordre JJ/MM.
- **Lignes masquées d'un tableur à l'import (DP-80).** Elles sont lues comme les autres ; une agence qui masque une ligne pour l'exclure la verrait importée. À confirmer avec l'agence pilote.
- **Valeurs par défaut des droits réglables (01/11/2026, F02).** La matrice dit « Réglable » pour : gestionnaire et collaborateur qui encaissent, gestionnaire qui exporte, gestionnaire qui importe, sans valeur par défaut. Proposition : « non » partout tant que le gérant ou le propriétaire n'a pas activé le droit. À trancher avant l'étape 03 (`role_settings`).
