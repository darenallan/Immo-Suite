# Étape 18 — Assistant d'import guidé (écran S16)

| Champ | Valeur |
|---|---|
| Statut | À faire |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T2 (F14 : interface) |
| Effort | M, environ 8 JD |
| Dépend de | Étape 12 |
| Jalon | Lot 2 |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Permettre au gestionnaire ou au gérant de téléverser son fichier, associer les colonnes, lire l'aperçu, confirmer et annuler, sans passer par HABEK.
Valeur : réimports pendant la double tenue et agences suivantes sans coût de support.

## 2. Périmètre, règles et documents à lire

F14 ; R-080 ; E-20, E-21 (`DOMAINE.md`) ; `API_CONTRACTS.md` §6.4, §7.9, §5.4 ; `UI_UX_DESIGN.md` S16, S02 ; `TESTING_QA.md` §5 (parcours 4), §7 ; moteur de l'étape 12.

## 3. Prérequis

Étape 12 ; textes d'erreur d'import validés (`UI_UX_DESIGN.md` « Microcopy »).

## 4. Fiche de spécification à produire ou valider

`specs/F14-assistant-import.md` — pas-à-pas téléverser → associer (propositions automatiques, mémorisation) → aperçu (valides, avertissements, erreurs avec numéro de ligne, doublons à fusionner) → confirmer → rapport ; historique avec « Annulable » ; polling 1 s puis 2 s puis 5 s ; message « La connexion est lente… » après 10 s ; droit réglable `import_data` du gestionnaire. **S'arrêter pour validation de Mimis.**

## 5. Tâches détaillées

- 5.1 Base de données : aucune (colonnes de l'étape 12).
- 5.2 Noyau métier : aucune règle nouvelle ; droit `import.run` appliqué (gérant, propriétaire, gestionnaire autorisé).
- 5.3 API : routes de l'étape 12 consommées par le client généré.
- 5.4 Interface : S16 complet, étape d'import de l'assistant S02, historique des imports, rapport téléchargeable, états vide/chargement (analyse)/erreur (format invalide : colonnes attendues)/succès.
- 5.5 PDF : aucun. 5.6 Événements : `import_uploaded`, `import_mapping_edited`, `import_previewed`, `import_confirmed`, `import_cancelled`, `import_abandoned`.
- 5.7 Tests : E2E parcours 4 (197 créables, 3 erreurs, continuer sans ces lignes, fusion de deux doublons, confirmer, annuler, E-21) à 390 et 1440 px ; accessibilité ; annulation refusée après un paiement ; fichier trop gros.

## 6. Critères d'acceptation vérifiables

1. Le parcours 4 se déroule sans intervention du développeur sur le fichier de référence.
2. Les erreurs sont lues avec leur numéro de ligne et une consigne.
3. Après annulation sans paiement, plus aucune donnée du lot.
4. Un gestionnaire sans droit `import_data` ne voit pas l'entrée de menu et l'API répond 403.

## 7. Sécurité et déploiement à vérifier

Mêmes limites de fichiers qu'à l'étape 12 ; interrupteur éteint puis allumé pour l'agence pilote.

## 8. Liste de démonstration (Mimis)

Importer le fichier de 200 lignes depuis un téléphone ou un ordinateur, corriger, confirmer, annuler un second import.

## 9. Définition de terminé, risques, hors étape

Terminé : critères 1 à 4 verts ; `UI_UX_DESIGN.md` inchangé ou mis à jour.

| Risque | Mesure |
|---|---|
| Utilisateur qui craint de « casser » | Rien n'est créé avant confirmation, annulation visible |

Hors étape : import de paiements historiques.

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 18
- Dépend de : 12
- Tranche : T2
- Jalon : Lot 2
- F : F14
- R : R-080
- E : E-20, E-21
- PP : PP-18
- Contenus : aucun
