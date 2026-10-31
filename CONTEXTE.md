# Immo Suite — Contexte (boussole métier)

| Champ | Valeur |
|---|---|
| Statut | Propriétaire du périmètre, des tranches, des rôles et des exclusions |
| Version | 0.3 |
| Date | 30/10/2026 |
| Dépend de | Dossier de cadrage v1.0 (§1 à §10, §22 à §26), `DECISIONS.md` |
| Alimente | `DOMAINE.md`, `UI_UX_DESIGN.md`, `ARCHITECTURE.md`, `TESTING_QA.md`, `CLAUDE.md` |

Source : dossier de cadrage v1.0 du 24/10/2026 (`Immo-Suite_Dossier-de-cadrage.docx`). Aucune enquête terrain n'a eu lieu et aucune n'est prévue avant le développement (DP-52). Le « workflow actuel », les prix et les volumes sont des hypothèses [À valider], validées pendant le pilote.

## Identité

| Élément | Valeur |
|---|---|
| Nom | Immo Suite |
| Éditeur | HABEK Technology Company (DP-63). Le nom « Immo Suite » est provisoire (décision n° 13) |
| Nature | Logiciel web de gestion locative, utilisable sur téléphone et ordinateur |
| Marché | Burkina Faso. Villes citées : Ouagadougou, Bobo-Dioulasso |
| Usages | Habitation et commercial (boutiques, entrepôts, bureaux) |
| Devise / langue | FCFA (XOF), sans décimale / français |
| Statut | Cadrage terminé (v1.0). Première version en construction (T1 puis T2), testée par des agences pilotes avec leurs données réelles (DP-52). La maquette est conservée comme démonstration |
| Décisions | Voir `DECISIONS.md` |

## Problème, solution, promesse

**Problème.** Les loyers sont encaissés de façon dispersée : espèces, mobile money sur des numéros personnels, virements, chèques. Le suivi se fait sur carnet, Excel ou WhatsApp [À valider]. Conséquences :

- Impayés découverts tard.
- Relances non tracées.
- Preuves fragiles : quittances manuscrites ou absentes, captures d'écran perdues.
- Comptes rendus aux propriétaires longs et contestables.
- Mémoire du bien dispersée.

**Solution.** Un livre de comptes locatifs. On enregistre un encaissement, on émet la quittance, on voit les impayés, on relance, on rend compte au propriétaire.

**Promesse.** Savoir en une minute qui doit quoi, qui a payé et quelle preuve a été remise. Le caissier vise moins de 30 secondes à l'encaissement (cible mesurée au pilote, `TESTING_QA.md` PP-25). L'agence produit le compte rendu d'un propriétaire en deux clics.

## Cibles et plans

Deux plans sur une seule base de code. Même moteur, périmètre activé par plan.

| | Plan Propriétaire | Plan Agence |
|---|---|---|
| Pour qui | Propriétaire direct, quelques lots | Agence de gestion, plusieurs utilisateurs |
| Utilisateurs | 1 propriétaire + 1 collaborateur de confiance | Plusieurs, avec rôles (gérant, gestionnaire, caissier…) |
| Lots | Aucun plafond appliqué par le logiciel au pilote (DP-65) | Aucun plafond appliqué par le logiciel au pilote (DP-65) |
| Mandants et commissions | Non (le propriétaire est lui-même le mandant) | Oui (F05, F13) |
| Clôture de caisse, audit détaillé | Version simple | Complet |
| Quittances, relances, impayés, travaux, documents | Oui | Oui |

- Le locataire ne paie jamais Immo Suite.
- Le dossier cible les agences et les propriétaires à partir d'une dizaine de lots, avec un plan d'entrée léger pour les plus petits.
- Prix : hypothèses à tester pendant le pilote (H5), jamais annoncées comme définitives. Les paliers sont commerciaux, sans effet dans le logiciel (DP-65). Grille de test du dossier (FCFA/mois) :
  - Propriétaire Essentiel : 5 000–10 000.
  - Propriétaire Pro : 10 000–20 000.
  - Agence : 25 000–40 000.
  - Agence Plus : 50 000 et plus.

## Règles d'or métier

Principes non négociables. Toute fonctionnalité, tout écran et tout code les respectent.

1. **Livre de comptes, pas outil de relance.** La valeur est la vérité sur les encaissements, la preuve (quittance) et la reddition de comptes. La relance est une fonction.
2. **La quittance est l'accroche d'usage.** Le caissier saisit chaque paiement parce que la quittance en sort immédiatement (PDF, impression, WhatsApp). Si la saisie n'est pas un geste de comptoir, le tableau des impayés est faux.
3. **Jamais de détention de fonds de clients.** Les paiements en ligne iraient directement sur le compte marchand de l'agence ou du propriétaire. HABEK n'encaisse ni ne reverse l'argent des clients.
4. **Paiements immuables et traçables.** Un paiement n'est jamais modifié ni supprimé. Une correction crée une écriture inverse (contre-passation), liée à l'originale et tracée.
5. **Bien / Lot.** Le Bien est un emplacement (immeuble, cour, site). Le Lot est ce qu'on loue (appartement, boutique, entrepôt). Une maison seule = 1 bien + 1 lot.
6. **Montants en entiers FCFA.** Jamais de décimaux ni de flottants.
7. **Quittances numérotées sans trou.** Numérotation continue et sans trou, par organisation et par année (DP-15, R-050). Une erreur se corrige par une quittance rectificative.
8. **Isolation par organisation.** Une organisation ne voit jamais les données d'une autre. Filtrage `org_id` dans le code et dans PostgreSQL.
9. **Journal d'audit en écriture seule.** Pas d'action sensible sans trace.
10. **Rien ne se supprime.** Archivage logique.
11. **L'humain reste dans la boucle.** Relances assistées (WhatsApp s'ouvre avec le texte prêt), pas automatiques. Aucune IA au MVP : les règles d'argent sont des règles exactes.
12. **Règles d'argent jamais sur mesure.** Une demande client devient une option paramétrable si au moins trois clients la veulent. Sinon elle est refusée ou facturée comme service.

## Périmètre

### MVP en trois tranches

Définition de « terminé » du MVP : une agence pilote clôture un mois complet (encaisser, quittancer, relancer, produire les comptes rendus propriétaires) sans ouvrir Excel. Un propriétaire direct suit ses 10 locataires depuis son téléphone sans aide.

Le découpage en tranches vient du dossier et est adopté par DP-29. V1 = T1 + T2 + T3. Le pilote démarre à la fin de T2. T3 se construit pendant le pilote.

**T1 — Cœur d'encaissement.** Résultat testable : le caissier encaisse et remet la quittance ; le gestionnaire voit les impayés et relance.

| ID | Fonctionnalité |
|---|---|
| F01 | Comptes, organisation et plans |
| F02 | Rôles et permissions |
| F03 | Journal d'audit |
| F04 | Biens et lots (habitation / commercial) |
| F06 | Fiche locataire |
| F07 | Baux (avenants, dépôt de garantie simple) |
| F08 | Échéancier automatique |
| F09 | Enregistrement des paiements (partiel, avance, contre-passation) |
| F10 | Quittances PDF numérotées et partage |
| F11 | Tableau de bord des impayés |
| F12 | Relances assistées et journal |
| F39 | Sauvegardes chiffrées, restauration testée |

**T2 — Reddition et migration.** Résultat testable : une agence pilote migre ses données et sort ses comptes rendus du mois.

| ID | Fonctionnalité |
|---|---|
| F05 | Propriétaires mandants et mandats (plan Agence) |
| F13 | Compte rendu propriétaire (plan Agence) |
| F14 | Import guidé Excel / CSV |
| F40 | Console HABEK (support, import assisté) |
| F15 (minimal) | Dépense imputable |

**T3 — Mémoire et confort.** Résultat testable : historique du bien, dossiers complets, alertes de fin de bail.

| ID | Fonctionnalité |
|---|---|
| F15 | Journal des travaux et dépenses par lot (complet) |
| F16 | Documents par bien, lot, locataire, bail |
| F17 | Agenda et alertes |
| F18 | Exports CSV / Excel / PDF pour le comptable |
| F20 | Dépôt de garantie : suivi, retenues, restitution |
| F22 | Notifications internes |
| F19 | Clôture de caisse journalière (rattachée à T3 par DP-29) |
| F23 | Fin de bail et changement de locataire (rattachée à T3 par DP-29) |
| F25 | Saisie assistée depuis un SMS mobile money collé (rattachée à T3 par DP-29) |

**Priorités** : P0 = F01–F14, F39, F40. P1 = F15–F20, F22, F23, F25. P2 = F21, F24, F26–F28, F30–F32. P3 = F29, F33–F37.

### Exclusions du MVP

| Exclu | Raison | Alternative au démarrage |
|---|---|---|
| Paiement en ligne par le locataire (F26), P2 (DP-32) | Frais d'agrégateur, conformité, risque de détenir des fonds, support des échecs | Saisie du paiement mobile money + lecture assistée du SMS (F25) |
| WhatsApp Business API (F27), lien pré-rempli retenu (DP-33) | Approbation de modèles, coût par message (tarifs Burkina à vérifier), mise en production plus lourde | Lien WhatsApp pré-rempli, sans coût |
| États des lieux complets, signature électronique (F24, F34) | Photos, hors-ligne, valeur juridique de la signature | Pièce jointe dans les documents du bail (F16) |
| Portails propriétaire mandant et locataire (F28, F29) | Comptes, mots de passe, support, sans preuve de besoin | Compte rendu PDF et lien de partage |
| Application mobile native / hors-ligne (F33) | Complexité de synchronisation | Web responsive |
| Copropriété, annonces, vitrine (F35, F36) | Autres métiers, risque de dispersion | — |
| Comptabilité générale | Dérive vers un logiciel de comptabilité | Exports pour le comptable (F18) |
| Pénalités de retard automatiques (F21) | Sensible légalement et socialement | Champ libre « frais de retard » saisi à la main |
| Indexation automatique du loyer (F30) | Règles à vérifier (bail et décret) | Alerte « révision à étudier » (F17) |
| Intelligence artificielle (dont F37) | Aucun cas où elle bat une règle simple | Règles et modèles de texte |
| Multi-pays, multi-devises, multilingue | Un pays, une devise, une langue au début | Devise et langue en paramètres, rien d'activé |
| API publique, intégrations comptables | Aucun client ne la demandera avant d'avoir des données | Exports CSV |

Hors MVP aussi : abonnement Immo Suite en ligne (F31, facturé à la main au début) et rapports avancés (F32).

## Utilisateurs et rôles

| Rôle | Ce qu'il veut | Fréquence |
|---|---|---|
| Gérant / directeur d'agence (décideur, acheteur, admin) | Sécuriser les revenus, contrôler le personnel, valider les reversements | Quotidien, 10–20 min |
| Gestionnaire locatif | Suivre ses lots, relancer au bon moment | Quotidien, plusieurs heures |
| Caissier | Encaisser vite, remettre une quittance, clôturer la caisse | Quotidien, pics en début de mois |
| Comptable (lecture) | Consulter les paiements et les comptes rendus, exporter pour la comptabilité. Rôle avec compte, en lecture seule, qui peut être un tiers de l'agence (DP-42) | Mensuel |
| Agent terrain / recouvreur | Voir sa tournée, noter une visite ou une promesse. Android d'entrée de gamme, réseau instable | Quotidien, mobile |
| Propriétaire direct | Savoir qui a payé, ne pas oublier. Gère seul ou délègue | Hebdomadaire à quotidien |
| Propriétaire mandant (client d'une agence) | Lire son compte rendu mensuel. Lecture indirecte | Mensuel |
| Locataire | Reçoit quittance et rappels par WhatsApp ou SMS. Pas de compte au MVP | Mensuel |
| Équipe HABEK (support) | Importer, corriger, dépanner. Admin plateforme | Quotidien |

Autres utilisateurs indirects : artisans et prestataires (ordre d'intervention, pas de compte), notaire / huissier (hors MVP).

Personas de travail fictifs, à remplacer par des personnes réelles :

- **A — Mme Kaboré**, gérante d'agence à Ouagadougou [À valider]. 3 employés, une quinzaine de propriétaires mandants. Elle achète si le compte rendu propriétaire sort en deux clics.
- **B — M. Sawadogo**, propriétaire direct. Une quinzaine de logements et deux boutiques, collecte déléguée à un parent. Il achète si la prise en main tient en cinq minutes sur son téléphone.

### Matrice de permissions (organisation Agence)

Appliquée côté serveur, jamais seulement masquée dans l'interface. Un test par couple rôle × action.

| Action | Gérant | Gestionnaire | Caissier | Agent terrain | Comptable (lecture) |
|---|---|---|---|---|---|
| Créer / modifier biens, lots, baux | Oui | Oui | Non | Non | Non |
| Voir biens, lots, locataires | Oui | Oui | Lecture | Ses lots assignés | Lecture |
| Enregistrer un paiement | Oui | Réglable | Oui | Non (P1 : sur place avec validation) | Non |
| Corriger / annuler un paiement | Oui | Demande | Demande | Non | Non |
| Émettre / réimprimer une quittance | Oui | Oui | Oui | Non | Lecture |
| Voir le tableau des impayés | Oui | Oui | Lecture | Ses lots | Lecture |
| Envoyer une relance, noter une promesse | Oui | Oui | Non | Oui (ses lots) | Non |
| Voir / produire le compte rendu propriétaire | Oui | Voir | Non | Non | Voir |
| Valider un reversement | Oui | Non | Non | Non | Non |
| Saisir un travail / une dépense | Oui | Oui | Non | Proposer | Non |
| Exporter les données | Oui | Réglable | Non | Non | Oui |
| Gérer utilisateurs et rôles | Oui | Non | Non | Non | Non |
| Voir le journal d'audit | Oui | Non | Non | Non | Lecture |

Plan Propriétaire : mêmes rôles, deux profils prédéfinis. Propriétaire = tous droits. Collaborateur = enregistrer les paiements et relancer.

Compléments à la matrice (DP-66) : le collaborateur du plan Propriétaire enregistre les paiements (droit réglable), relance, voit biens, lots, locataires et impayés en lecture, demande une correction, émet et réimprime une quittance ; il n'importe pas, n'exporte pas, ne saisit pas de dépense et n'accède ni aux paramètres ni au journal d'audit. L'agent terrain ne propose pas de dépense au pilote (la route reste refusée). Les entrées de menu que la matrice ne définit pas sont masquées. La matrice ci-dessus est la source unique : `SECURITY.md` §4.1 et `API_CONTRACTS.md` §4.6 la reprennent.

## Contexte local

- **Espèces.** Encaissement au bureau, quittancier papier, écarts de caisse. D'où la quittance numérotée et la clôture de caisse.
- **Mobile money.** Orange Money et Moov Money, souvent vers des numéros personnels. Capture d'écran envoyée sur WhatsApp. Au MVP : saisie guidée et lecture du SMS, pas d'intégration. Frais indicatifs d'un agrégateur : 2 à 3,5 % [À valider].
- **WhatsApp.** Canal principal des relances et des quittances. Au MVP : lien pré-rempli ouvert depuis l'application. Hypothèse : cela suffit pour démarrer (H9). Les locataires qui refusent WhatsApp seraient relancés par SMS ou appel tracé.
- **SMS.** Passerelle à choisir, coût par SMS à obtenir.
- **Terminal.** Android d'entrée de gamme, connexion lente, soleil. Indicatif +226. Quittance imprimable en A4 et A5.
- **Cadre légal à garder en tête** (toutes ces règles restent à faire valider, décision n° 12) :
  - Bail d'habitation : loi n° 103-2015/CNT. D'après la presse, contrat écrit et garantie plafonnée à deux mois. Décret de 2018 sur les travaux et la révision du loyer. [À VÉRIFIER AVEC UN JURISTE]
  - Bail commercial ou professionnel : régime distinct (droit OHADA). Le produit saisit des conditions différentes sans générer un bail complet. [À VÉRIFIER AVEC UN JURISTE]
  - Données personnelles : loi n° 001-2021/AN et CIL. Déclaration ou autorisation, hébergement hors du pays encadré. [À VÉRIFIER AVEC UN JURISTE] (CIL)
  - Fiscalité des revenus fonciers : le MVP ne calcule ni ne déclare aucun impôt. Il exporte des états propres. [À VÉRIFIER AVEC UN JURISTE]

## Concurrence

Aucun logiciel local dominant trouvé pour le Burkina Faso. Cela ne prouve pas son absence (H10), car les solutions de niche vivent sur WhatsApp et Facebook.

| Acteur | Zone | Enseignement |
|---|---|---|
| Locapay | Bénin | Gestion locative, annonces et paiement mobile money ; plus de 6 000 utilisateurs et plus de 100 mandats revendiqués. Vise la place de marché, pas le créneau « registre + reddition » |
| Kanimmo | Sénégal | Même cible qu'Immo Suite. Un entrant régional est plausible |
| YOFATI IMMO | Sénégal | CRM + loyers pour agences, tableau d'impayés. Le tableau d'impayés seul est banalisé |
| RentBeta, Reemio | Ouganda, Kenya | Confirment la tendance panafricaine |

Le vrai concurrent est le cahier + Excel + WhatsApp, et le coût de changement (ressaisir 200 locataires). D'où l'import Excel (F14) et l'accompagnement.

Différenciation visée : ancrage local, qualité de l'import, fiabilité des chiffres (audit, immutabilité), compte rendu propriétaire.

## Hypothèses à valider

| # | Hypothèse | Comment la valider |
|---|---|---|
| H1 | Le suivi sur carnet/Excel/WhatsApp est la norme chez les agences et gros propriétaires de Ouagadougou et Bobo-Dioulasso | Pilote : observation chez l'agence pilote (section 20) |
| H2 | Une grande part des paiements se fait en espèces au bureau et par mobile money | Pilote : relevé des paiements de l'agence pilote |
| H3 | La reddition mensuelle de comptes aux propriétaires est pénible et source de litiges | Pilote : observer la préparation d'un compte rendu |
| H4 | Les agences accepteraient de saisir chaque paiement au comptoir | Pilote : taux de saisie sous 48 h |
| H5 | Le prix de 20–50 000 FCFA/mois est accepté pour les agences ; 5–20 000 pour les propriétaires | Pilote : questions de prix et acompte (méthode « trop cher / bon marché / limite ») |
| H6 | Les propriétaires directs (10+ lots) forment un segment assez grand et accessible | Comptage des cibles accessibles via réseaux et bouche-à-oreille |
| H7 | Le commercial (boutiques, entrepôts) nécessite des champs spécifiques mais pas de moteur différent | Pilote : observation avec des propriétaires de locaux commerciaux |
| H8 | La règle de reddition « date d'encaissement » est la plus courante | Pilote : découverte de l'agence (`DEPLOYMENT.md` §8.1 étape 2) |
| H9 | Les relances WhatsApp assistées suffisent pour démarrer | Retours des gestionnaires en pilote |
| H10 | Aucun concurrent local significatif n'existe | Recherche complémentaire (Facebook, WhatsApp, salons, agences) |
| H11 | La capacité de développement est d'environ 25 heures par semaine | Confirmation et planification explicite |
| H12 | Un hébergement hors du pays est acceptable pour la CIL avec les garanties contractuelles | Avis de la CIL ou d'un juriste |

### Critères du pilote

Il n'y a plus de Go/No-Go avant le développement (DP-52). Les critères de continuation, de correction ou d'arrêt du pilote sont fixés dans `DECISIONS.md`, entrée **DP-53** (source unique : ne pas recopier les chiffres ici). Le critère « prêt pour un pilote » est dans `TESTING_QA.md` §12.

Échelle d'engagement notée par personne, comme grille d'observation du pilote (sans seuil) : 0 poli mais évasif ; 1 décrit ses douleurs et montre ses outils ; 2 partage un échantillon de données ; 3 signe une lettre d'intention et désigne un référent ; 4 accepte un prix et prépaye.

Livrables avant le démarrage du pilote : tranches T1 et T2, prototype PDF de quittance (prototype (a)), documents de pilote (`contenus/`), avis juridique succinct ou risque accepté par écrit (DP-62). Durée : fixée par la roadmap sur 25 heures par semaine (DP-54).

## Index des documents

La liste des documents, leur statut (créé ou à créer) et le moment où les lire sont tenus dans un seul endroit : le tableau « Quel fichier lire et quand » de `CLAUDE.md`. Source du projet : `Immo-Suite_Dossier-de-cadrage.docx`, jamais modifié.

## Questions ouvertes

- **Cadre légal.** Les quatre points marqués [À valider] du « Contexte local » (bail d'habitation, bail commercial, données personnelles, fiscalité) sont traités par la décision n° 12 et par la section « Règles à confirmer sur le terrain » de `DOMAINE.md`.
- **Frais d'agrégateur mobile money.** « 2 à 3,5 % » vient d'un fournisseur local [À valider] : à vérifier au moment du choix d'un agrégateur (F26, P2).
- **Persona A.** Une agence « de taille moyenne à Ouagadougou » est une hypothèse [À valider] : à remplacer par une personne réelle après le premier pilote.
- **F38.** Le dossier définit F01 à F37, F39 et F40. F38 n'existe pas (trou dans la numérotation).
- **Décideur.** Le dossier s'adresse à un « tu » non nommé. Confirmer que Mimis valide les critères du pilote (DP-53) et les prix.
