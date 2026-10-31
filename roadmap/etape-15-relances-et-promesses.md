# Étape 15 — Relances assistées, promesses et rappels à J−3

| Champ | Valeur |
|---|---|
| Statut | À faire |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T1 (F12), livré dans le lot 2 |
| Effort | M, environ 9 JD |
| Dépend de | Étape 10 ; contenu C-10 |
| Jalon | Lot 2 (avant la bascule `live`) |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Relancer au bon moment avec un message prêt (WhatsApp ou SMS), journaliser chaque contact, noter une promesse et voir l'alerte quand elle n'est pas tenue.
Valeur : le gestionnaire agit sur les impayés sans quitter l'outil, sans aucun envoi automatique (règle d'or 11).

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections |
|---|---|---|
| Fonctionnalité | F12 | dossier §16 F12 |
| Règles | R-060 à R-065, R-097, R-101 ; E-14, E-15, E-41 | `DOMAINE.md` « Impayés et relances » |
| Décisions | DP-33, DP-36, DP-40, DP-58 | `DECISIONS.md` |
| Sécurité | `SECURITY.md` | §7.8 (WhatsApp), T17 |
| Contrat | `API_CONTRACTS.md` | §6.2, §7.7 |
| Écrans | `UI_UX_DESIGN.md` | S03, S05 (panneau Relancer), S17 (modèles), « Modèles de relance » |
| Schéma | `DATABASE_SCHEMA.md` §3 | « Relances et promesses » |

## 3. Prérequis

Étape 10 ; **C-10** : textes des quatre modèles (J−3, J+1, J+5, J+10 ; courtois, ferme, dernier avis). Les six variables sont `{nom}`, `{montant}`, `{mois}`, `{lot}`, `{date}`, `{lien_quittance}`. Création d'organisation complétée par les modèles par défaut (migration de rattrapage pour les organisations existantes).

## 4. Fiche de spécification à produire ou valider

`specs/F12-relances.md` — modèles refusant toute variable inconnue et bloquant l'envoi si une variable manque (R-064, E-14) ; relance assistée : WhatsApp ou SMS s'ouvre avec le texte, l'utilisateur confirme, la relance n'est journalisée qu'après confirmation (R-061, R-062) ; second contact le même jour : avertissement non bloquant (R-063) ; rappel à J−3 proposé sans envoi (R-097) ; promesse « tenue » = paiement au moins égal au promis, à la date promise ou avant (R-101, E-41) ; non tenue dès le lendemain, alerte dans S03 et S05 (R-065, E-15). Cas limites : locataire sans numéro, colocation (destinataire à confirmer), plusieurs échéances. Cas chiffrés : E-14, E-15, E-41. **S'arrêter pour validation de Mimis.**

## 5. Tâches détaillées

### 5.1 Base de données
`message_templates` (`row_version`), `reminders` et `reminder_installments` (ajout seul), `promises` (colonnes modifiables limitées : clôture manuelle) ; modèles par défaut à la création d'organisation.

### 5.2 Noyau métier
`render_template` (variables autorisées), `reminders_due(today)` (J−3), `log_reminder`, `record_promise`, `close_promise`, statut de promesse calculé ; audit et événements `reminder_prepared`, `reminder_confirmed`, `reminder_abandoned`, `promise_recorded`, `promise_missed`.

### 5.3 API
`GET /reminders`, `GET /reminders/due`, `POST /reminders/draft`, `POST /reminders`, refus `PUT|PATCH|DELETE` (`reminder.immutable`), `GET /promises`, `POST /promises/{id}/close`, `GET/PUT /message-templates`, `POST /message-templates/{id}/preview`, `GET /message-variables`.

### 5.4 Interface
S05 : colonnes dernière relance, promesse, prochaine action ; panneau « Relancer » (côté latéral sur ordinateur, feuille du bas sur mobile) avec nom et deux derniers chiffres du numéro avant ouverture ; S03 gestionnaire : blocs « Rappels à envoyer » et promesses du jour/non tenues ; S17 : édition et test des modèles.

### 5.5 Documents PDF
Aucun.

### 5.6 Audit et événements
Voir 5.2.

### 5.7 Tests
E-14, E-15, E-41 ; variable inconnue refusée, variable manquante bloquante ; seconde relance du jour ; relance immuable (405) ; locataire sans numéro ; A3, A4, A5 (agent terrain : ses lots) ; E2E parcours 2 complet (relance, WhatsApp non ouvert, promesse au 20/11, ligne qui reste) ; journaux sans numéro.

## 6. Critères d'acceptation vérifiables

1. Impayé de 75 000 FCFA et modèle courtois : le texte contient le nom, le montant et le mois exacts (E-14).
2. Promesse au 12 non tenue : alerte visible dans l'accueil le 13 (E-15) ; promesse de 35 000 au 20/11 : tenue par 35 000 le 19/11, non tenue par 20 000 le 20/11 (E-41).
3. Aucune relance n'est journalisée sans confirmation ; une seconde relance le même jour avertit sans bloquer.
4. Une promesse non échue n'efface pas la ligne d'impayé (R-059).

## 7. Sécurité et déploiement à vérifier

Texte minimal du message ; confirmation du destinataire ; aucun envoi côté serveur ; modèles modifiables seulement par le gérant ; livraison derrière un interrupteur allumé pour l'agence pilote.

## 8. Liste de démonstration (Mimis)

Ouvrir un impayé, choisir « courtois », lire le texte, ouvrir WhatsApp sur son téléphone, confirmer, noter une promesse au 20, voir l'accueil le lendemain de la date.

## 9. Définition de terminé, risques, hors étape

Terminé : critères 1 à 4 verts ; textes C-10 intégrés ; documents à jour.

| Risque | Mesure |
|---|---|
| Ton ou étapes différents du terrain | Modèles modifiables par l'agence, mesure `reminder_*` |
| Locataires refusant WhatsApp | Canal SMS ou appel tracé |

Hors étape : API WhatsApp Business (F27), notifications internes (étape 23).

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 15
- Dépend de : 10
- Tranche : T1
- Jalon : Lot 2
- F : F12
- R : R-060, R-061, R-062, R-063, R-064, R-065, R-097, R-101
- E : E-14, E-15, E-41
- PP : aucun
- Contenus : C-10
