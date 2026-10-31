# Étape 11 — Console HABEK en ligne de commande et cycle de vie d'une organisation

| Champ | Valeur |
|---|---|
| Statut | À faire |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T2 (F40) et T1 (F39 : export, purge) |
| Effort | M, environ 12 JD |
| Dépend de | Étapes 03 (accord support) et 08 (série d'essai, filigranes) |
| Jalon | Contribue à J3 |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Permettre à HABEK de créer une agence, la faire passer de `setup` à `live`, exporter ses données, la remettre à zéro en phase `setup`, produire une organisation de démonstration et une copie anonymisée — sans jamais lire une base de production à la main.
Valeur : l'accueil d'une agence (`DEPLOYMENT.md` §8.1) devient une procédure répétable et auditée.

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections |
|---|---|---|
| Fonctionnalités | F40 (console), F39 (export, purge) | dossier §16 F39/F40 |
| Règles | R-009 (accès support tracé, expirant), R-110 (cycle de vie) ; E-47, E-48 | `DOMAINE.md` |
| Décisions | DP-60, DP-61, DP-68 ; n° 15, 25, 26, 38, 40, 47, 54, 98 | `DECISIONS.md` |
| Exploitation | `DEPLOYMENT.md` | §8.1 (étapes 1, 9), §8.2, §9, `CLAUDE.md` « Exploitation » |
| Schéma | `DATABASE_SCHEMA.md` | §3 `platform_staff`, `platform_events`, `org_features` ; §4.2 (`purge_org_setup`), §6.6, §7.6, §8.3 à §8.5 |
| Sécurité | `SECURITY.md` | §3.7, §5.6, §8.4 |
| Contrat | `API_CONTRACTS.md` | §7.14, §7.18 (routes réservées, console en ligne de commande au pilote) |

## 3. Prérequis

Étapes 03 et 08. **C-07** (conditions du pilote) pour fixer ce que le support peut faire. Décision n° 98 : console du pilote = ligne de commande, interface web reportée [À valider]. Comptes HABEK nominatifs avec TOTP.

## 4. Fiche de spécification à produire ou valider

`specs/F40-console.md` — commandes : `immo org create|list|show|go-live|suspend|purge|export`, `immo feature set`, `immo demo reset`, `immo anonymize`, `immo report usage|feedback|errors`, `immo password-reset` ; authentification des comptes HABEK (nominatifs, TOTP, session de 15 min) ; accès aux données d'une organisation seulement sous accord valide (R-009) avec ligne d'audit `support.record_viewed` ; **bascule `go-live`** : refusée sans TOTP actif du gérant (PP-31), irréversible, numérotation officielle à 000001 (E-48) ; **purge** : `setup` seulement, accord écrit, sauvegarde préalable, deux personnes, trace dans `platform_events` (R-110, E-47) ; **export** : un CSV par table, PDF, manifeste SHA-256, cellules neutralisées (`=`, `+`, `-`, `@`). Cas limites : organisation `live` purgée (refus), export d'une organisation `demo` (exclue), accord support expiré en cours de commande. Cas chiffrés : E-47, E-48. **S'arrêter pour validation de Mimis.**

## 5. Tâches détaillées

### 5.1 Base de données
`platform_staff` (comptes nominatifs, `totp_required` vrai par contrainte), `org_features` (clé, état, auteur, note), fonctions `SECURITY DEFINER` `purge_org_setup` (refusée hors `setup`, ordre `purge_order`, trace hors organisation) et `create_organization` complétée (modèles par défaut à l'étape 15) ; déclencheur interdisant le retour à `setup` y compris pour la console ; garde « comptes démonstration / réels » ; propriété du compteur : série `official` créée à la bascule.

### 5.2 Noyau métier
`platform::org_lifecycle` (create, go_live, suspend), `platform::export` (dans un `OrgTx`, champs chiffrés selon la décision de `SECURITY.md`), `platform::anonymize` (jamais en production : refus si `APP_ENV=production`), `seeds/demo/` rejouant les actions métier par les services (organisations « Agence Horizon Démo » préfixe `HOR` et « Patrimoine Sawadogo » préfixe `SAW`, horloge fixée au 13/11/2026) pour le périmètre déjà livré ; audit `feature.toggled`, `org.purged`, `org.go_live`.

### 5.3 API
Aucune route publique ; routes `/console/…` non servies. Lecture de `features` par `GET /me` (déjà livrée) vérifiée ; refus `403 feature.disabled` d'une fonction éteinte.

### 5.4 Interface
Aucun écran (console en ligne de commande). Section « Accès support » de S17 consolidée (historique).

### 5.5 Documents PDF
Filigranes « DÉMONSTRATION — sans valeur » vérifiés sur l'organisation `demo`.

### 5.6 Audit et événements
Tout accès aux données d'une organisation écrit `support.record_viewed` visible dans S17 et S21.

### 5.7 Tests
E-47, E-48 ; ISO-8 ; purge refusée en `live` ; export puis comptage de lignes par table ; cellules d'export neutralisées (PP-19) ; `anonymize` : aucun numéro hors du bloc fictif `+226 00 00 …`, aucun e-mail, aucune note (PP-35) ; mélange démonstration/réel refusé (PP-34) ; bascule sans TOTP refusée (PP-31) ; interrupteur éteint ⇒ route refusée.

## 6. Critères d'acceptation vérifiables

1. `immo org go-live` sans TOTP actif du gérant est refusé ; avec, la première quittance officielle porte le numéro 000001 (E-48).
2. `immo org purge` réussit en `setup` (paiements 0, quittances 0, compteurs 0, adhésions conservées) et échoue en `live` (E-47).
3. `immo org export` produit CSV, PDF, manifeste ; le nombre de lignes et les empreintes correspondent.
4. Une consultation sans accord support valide est refusée ; avec accord, elle écrit une ligne d'audit visible du client.
5. Après `immo anonymize`, le contrôle automatique est vert.
6. `immo demo reset` crée une nouvelle organisation de démonstration et archive l'ancienne.

## 7. Sécurité et déploiement à vérifier

TOTP obligatoire des comptes HABEK (PP-12) ; aucun accès de routine à la production ; commandes destructrices avec double validation ; secrets distincts ; migration de `purge_org_setup` relue par un second avis.

## 8. Liste de démonstration (Mimis)

1. Créer l'organisation d'essai « Agence Test » par `immo org create` et voir le lien d'invitation du gérant.
2. Y encaisser un paiement : voir la quittance `ESSAI-…` avec filigrane.
3. Tenter `go-live` sans TOTP : lire le refus ; activer le TOTP ; réussir ; encaisser : numéro officiel 000001.
4. Purger une autre organisation en `setup`, puis échouer sur une organisation `live`.
5. Lancer l'export et ouvrir le CSV dans un tableur.

## 9. Définition de terminé, risques, hors étape

Terminé : critères 1 à 6 verts ; `specs/F40-console.md` validée ; `DEPLOYMENT.md`, `CLAUDE.md`, `DECISIONS.md` (n° 15, 25, 26, 98) à jour.

| Risque | Mesure |
|---|---|
| Purge mal ciblée | Fonction unique filtrée par `org_id`, `setup` exigé, double validation, sauvegarde préalable |
| Console web demandée tôt | Commande suffisante au pilote ; écrans après le pilote |

Hors étape : écrans web de la console, `purge_org_closed` (avant le premier départ), tableau de bord riche.

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 11
- Dépend de : 03, 08
- Tranche : T2
- Jalon : J3
- F : F40, F39
- R : R-009, R-110
- E : E-47, E-48
- PP : PP-12, PP-23, PP-31, PP-35
- Contenus : C-07
