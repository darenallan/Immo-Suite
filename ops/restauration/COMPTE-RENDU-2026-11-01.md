# Compte rendu du test de restauration — 01/11/2026 (essai à blanc, poste de développement)

> **Ce n'est PAS le test de l'exigence PP-14.** C'est une répétition sur le poste de développement : base de données neuve sur le PostgreSQL 18
> local jetable (port 5433), jeu synthétique, second fournisseur simulé par un dossier. L'exigence (instance neuve chez l'hébergeur, copie réelle chez
> le second fournisseur, application déployée) reste **à faire** quand C-01 et C-02 seront fournis. Aucune donnée réelle, aucun secret.

- Réalisé par : Claude Code (poste de Mimis, commande à distance). Observateur : aucun (Mimis à distance). Source restaurée : copie quotidienne de test, générée à l'instant.
- Instance cible : base neuve sur PostgreSQL 18 local (même version majeure que la cible prévue, Neon étant à 17 ou 18 selon le projet [À vérifier]). Rôles et extensions identiques : **oui** (`ops/sql/roles.sql`, `btree_gist`).

## Chronométrage (jeu : 3 organisations fictives, 1 200 lignes, tests `tests/db/tests/backup.rs`)

| Étape | Durée mesurée | Remarque |
|---|---|---|
| Retrouver la clé | non chronométrée | clé de test engendrée par `immo backup keygen` ; l'exercice réel exige la clé hors ligne et une seconde personne |
| Créer l'instance | non chronométrée | `CREATE DATABASE` + `roles.sql` : moins d'une seconde en local ; chez l'hébergeur : à mesurer |
| Restaurer la base | déchiffrement 81 ms + `pg_restore` 704 ms ; total 1 243 ms | la base restaurée refuse de s'écraser si elle n'est pas vide |
| Restaurer les fichiers | **non fait** | couche 3 (réplique du seau) inexistante : aucun stockage objet choisi (DP-70, C-02) |
| Contrôles de cohérence | inclus dans le test | voir ci-dessous |
| **Total mesuré** | **1,2 s** pour la base seule | cible : moins de 4 h. Sans valeur de prévision : volume réel et réseau absents |

Sur le scénario en ligne de commande (3 lignes, `immo backup restore`) : déchiffrement 29 ms, restauration 146 ms.

## Résultats

- Lignes par table : écarts avec la source : **aucun** (400 lignes par organisation, 3 organisations).
- Somme des montants par organisation : **identiques**.
- Cinq requêtes de contrôle de `DATABASE_SCHEMA.md` §7.5 : **non applicables** (les tables métier n'existent pas encore) — à jouer à l'étape 02 et suivantes.
- RLS activée et forcée sur les tables, rôle applicatif sans `BYPASSRLS` : **oui** (audit de schéma sur la base restaurée).
- Gardes d'écriture seule restaurées et actives (`platform_events`) : **oui**. Droits de `immo_backup` restaurés : **oui**.
- Lecture de PDF restaurés : **non applicable** (aucun PDF).

## Contrôles de sécurité de la copie (mêmes tests)

- Copie chiffrée (`age`, clé publique) : aucun clair dans l'objet ; **mauvaise clé refusée** ; **un octet modifié détecté** (empreinte) ; copie tronquée détectée (taille).
- Écriture seule : **une copie existante n'est jamais réécrite** (tentative journalisée `backup.failed`). Le verrouillage d'objet et le refus de suppression par les identifiants du travail d'envoi **ne peuvent pas être essayés** sans le second fournisseur réel (critère 5).
- Alerte de sauvegarde manquante : déclenchée à 25 h, silencieuse à 23 h (PP-15, en test ; l'envoi sur le canal reste à faire).

## Perte de données mesurée (RPO) : non mesurable à blanc. Reprise mesurée (RTO) : non mesurable à blanc.

## Anomalies, décisions et corrections
- Les copies logiques quotidiennes ne couvrent que 24 h de perte au pire ; la couche 1 (Neon, 6 h de retour dans le temps au plan gratuit) couvre le reste (DP-69, DP-82).
- À jouer pour de vrai : voir `ops/BILAN-ETAPE-01.md`, critères 5 et 6. Responsable : développeur ; échéance : dès réception de C-01 et C-02.

## Prochaine échéance de test : dès que le second fournisseur et l'hébergeur existent (puis mensuel, 3ᵉ semaine).
