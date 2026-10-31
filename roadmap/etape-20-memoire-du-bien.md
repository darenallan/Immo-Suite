# Étape 20 — Mémoire du bien : travaux complets et documents

| Champ | Valeur |
|---|---|
| Statut | À faire (T3, pendant le pilote, derrière un interrupteur) |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T3 (F15 complet, F16) |
| Effort | L, environ 15 JD |
| Dépend de | Étape 16 ; critères d'entrée de l'étape 19 |
| Jalon | J6 |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Historique complet des travaux et documents (bail scanné, pièces) par bien, lot, locataire et bail.
Valeur : dossier retrouvable en cas de litige ; mémoire du bien sur dix ans.

## 2. Périmètre, règles et documents à lire

F15, F16 (dossier §7.3, §16) ; R-005, R-078 ; `API_CONTRACTS.md` §7.10, §7.11 ; `SECURITY.md` §7.4 (fichiers) ; `DATABASE_SCHEMA.md` §3 « Documents » (`documents`, `document_links`, créées vides) ; `UI_UX_DESIGN.md` S14, S07 ; parcours 5 de `TESTING_QA.md`.

## 3. Prérequis

Étapes 16 et 19 (critères d'entrée). Décision n° 68 (PDF et fichiers servis en flux). Analyse antivirus reportée : types restreints et téléchargement forcé (`SECURITY.md` §7.4).

## 4. Fiche de spécification à produire ou valider

`specs/F15-F16-travaux-documents.md` — prestataires, statuts d'intervention, pièces jointes, imputation reprise de l'étape 16 ; documents PDF, JPEG, PNG de 10 Mo au plus, réencodage des images, jamais affichés dans l'origine de l'application (`attachment`, `nosniff`), liés à plusieurs objets, archivés jamais supprimés. Cas limites : fichier piégé, type non autorisé, document d'une autre organisation. **S'arrêter pour validation de Mimis.**

## 5. Tâches détaillées

- 5.1 Base de données : activation de `documents` et `document_links` (RLS, clés composites), compléments de `interventions`.
- 5.2 Noyau métier : `receipts::documents` (envoi, lien, archivage), prestataires, interventions antérieures à l'usage d'Immo Suite.
- 5.3 API : `POST /documents`, `GET /documents`, `GET /documents/{id}/download`, `POST /documents/{id}/archive`, `GET/POST/PATCH /providers`, `GET /units/{id}/documents`.
- 5.4 Interface : S14 complet, onglets Travaux et Documents de S07, joindre un document, filtres ; interrupteur `documents`.
- 5.5 PDF : aucun. 5.6 Audit : `expense.imputation_changed` existant ; événement `feature_used`.
- 5.7 Tests : fichiers piégés et types, taille, isolation (A14), `Content-Disposition: attachment`, parcours 5 complet, accessibilité.

## 6. Critères d'acceptation vérifiables

1. Un document PDF de 10 Mo se téléverse, se lie à un lot et se télécharge en pièce jointe ; 10,1 Mo est refusé.
2. Un document d'une autre organisation est introuvable.
3. Une intervention imputée apparaît dans le compte rendu suivant sans modification des relevés validés.

## 7. Sécurité et déploiement à vérifier

Clé de stockage générée par le serveur ; seau privé ; interrupteur éteint par défaut, allumé pour une agence 2 à 3 jours d'observation.

## 8. Liste de démonstration (Mimis)

Ouvrir un lot, ajouter une intervention et un document, rechercher l'historique.

## 9. Définition de terminé, risques, hors étape

Terminé : critères verts ; interrupteur retiré à la date prévue.

| Risque | Mesure |
|---|---|
| Fichiers volumineux sur réseau lent | Limite 10 Mo, reprise manuelle |

Hors étape : états des lieux (F24), signature électronique (F34).

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 20
- Dépend de : 16
- Tranche : T3
- Jalon : J6
- F : F15, F16
- R : aucune
- E : aucun
- PP : aucun
- Contenus : aucun
