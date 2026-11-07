# Étape 08 — Quittance PDF, impression et partage WhatsApp

| Champ | Valeur |
|---|---|
| Statut | En cours : fiche `specs/F10-quittance-pdf-et-partage.md` validée le 06/11/2026 (DP-98) ; code en cours |
| Version / Date | 0.1 / 30/10/2026 |
| Tranche | T1 (F10) |
| Effort | M, environ 10 JD |
| Dépend de | Étape 07 ; contenus C-04 (volet « mentions de la quittance »), C-08 |
| Jalon | Contribue à J2 et J3 |
| Lecture obligatoire | `CLAUDE.md`, ce fichier, puis les sections citées en 2 |

## 1. Objectif et valeur pour le pilote

Transformer l'instantané de la quittance en PDF A4 et A5 imprimable, le partager par un lien WhatsApp pré-rempli ou le remettre sur papier, avec historique d'envoi et lien qui expire. Un échec de rendu ne bloque jamais l'encaissement.
Valeur : la preuve remise au locataire, accroche d'usage du caissier (règle d'or 2) ; la quittance est validable par l'agence candidate avant le pilote (PP-27).

## 2. Périmètre, règles et documents à lire

| Sujet | Référence | Sections |
|---|---|---|
| Fonctionnalité | F10 | dossier §16 F10 |
| Règles | R-049, R-052, R-054, R-055, R-056 ; E-12 | `DOMAINE.md` « Quittances » |
| Décisions | DP-33 (lien pré-rempli), DP-64 (A5 par défaut, deux formats, « francs CFA »), DP-62 (mentions), n° 7 (moteur), 39 (lecture par jeton), 55 (code de contrôle : après le pilote), 68 (flux par l'API) | `DECISIONS.md` |
| Rendu | `ARCHITECTURE.md` §6, §3.9, §3.10 ; `SECURITY.md` §7.5, §7.8 | idem |
| Contenu et mise en page | `UI_UX_DESIGN.md` | « Quittance : mise en page imprimable » (contenu, format, 20 cas), S25, S26 ; `MAQUETTE.md` « Textes de démonstration » (quittance, message WhatsApp) |
| Contrat | `API_CONTRACTS.md` | §5.4, §5.5 (PDF), §6.1 (envoi), §7.6 (`/receipts/{id}/pdf`, `/sends`, `/share-links`), pages publiques `/s/{jeton}` |
| Schéma | `DATABASE_SCHEMA.md` | §3 `receipt_files`, `receipt_sends`, `share_links`, `org_settings` ; §6.7 (lecture publique par jeton) |
| Tests | `TESTING_QA.md` | §6 en entier, A12, A14 |

## 3. Prérequis

Étape 07 ; moteur PDF décidé à l'étape 00 (n° 7) ; **C-08** : mentions légales de la quittance rédigées par Mimis (identité du bailleur ou de l'agence, locataire, lot, période, montant en chiffres et en lettres, solde restant, mentions complémentaires) ; **C-04** : avis juridique, volet mentions et formulation du montant en lettres, ou risque accepté par écrit (DP-62) ; **C-06** : une agence candidate prête à valider la mise en page (PP-27). Police libre avec accents complets (IBM Plex Sans sous licence OFL [À valider]). Logo : copie du fichier livré le 26/10/2026 (`logo/`) réduit à 30 Ko pour l'affichage.

## 4. Fiche de spécification à produire ou valider

`specs/F10-quittance-pdf-et-partage.md` — **règles** : rendu en tâche de fond depuis l'instantané, deux formats rendus à chaque paiement, A5 par défaut (R-056, DP-64) ; le PDF échoué laisse le paiement enregistré et la quittance en file avec nouvel essai et alerte visible (R-049) ; contenu de haut en bas selon `UI_UX_DESIGN.md` ; mentions (R-052) soumises à C-08 et à l'avis juridique ; filigranes « ESSAI » (série `trial`), « DÉMONSTRATION — sans valeur » (organisation `demo`), « ANNULÉE — remplacée par la quittance n° … » (étape 09) ; lien de partage de 128 bits au moins, un seul document, expiration configurable (30 jours par défaut), révocation (R-054) ; chaque envoi consigné (R-055) ; message WhatsApp minimal (`SECURITY.md` §7.8) avec nom et deux derniers chiffres du numéro affichés avant l'ouverture. **Cas limites** : locataire sans WhatsApp (impression ou saisie du numéro), nom contenant du balisage ou une adresse, logo absent ou maximal, montant maximal, quittance annulée réimprimée. **Écrans** S25, S26, section quittance de S17. **Erreurs** : `receipt.pdf_not_ready` (409), `receipt.recipient_has_no_phone`, `share_link.expired`. **Cas chiffrés** : E-12 ; PDF de 75 000 FCFA avec « soixante-quinze mille francs CFA » ; les 20 cas types de `TESTING_QA.md` §6.

**S'arrêter pour la validation de Mimis après la fiche et les maquettes de la quittance imprimée.**

## 5. Tâches détaillées

### 5.1 Base de données
`receipt_files` (`receipt_id`, `paper` `a4|a5`, `state` `pending|ready|failed`, `sha256`, clé de stockage générée par le serveur `org/{org_id}/receipts/{receipt_id}.pdf`, ajout seul) ; `receipt_sends` (canal, auteur, date, ajout seul) ; `share_links` (empreinte du jeton, type et identifiant du document, `expires_at`, `revoked_at` ; colonnes modifiables limitées) avec **politique de lecture par empreinte** (n° 39, aucun rôle `BYPASSRLS` pour l'application) ; colonnes de `org_settings` : en-tête, adresse, identifiants légaux facultatifs, format par défaut, durée des liens, références du logo et de la signature.

### 5.2 Noyau métier
1. Gestionnaire de la tâche `render_receipt` : lit l'instantané, appelle `PdfRenderer` (processus isolé : utilisateur sans droits, sans réseau, sans secret, système de fichiers en lecture seule, limites de temps et de mémoire), écrit A4 et A5, enregistre l'empreinte SHA-256 ; nouvel essai avec attente croissante ; alerte S2 si aucun PDF après 10 minutes (R-049) ; rendu reproductible ; PDF sans métadonnée d'auteur, titre neutre, polices embarquées.
2. Gabarits fixes dans `templates/` avec échappement automatique de toute valeur utilisateur ; filigranes ; montant en lettres repris de l'instantané (jamais recalculé par le moteur).
3. `receipts::create_share_link`, `revoke`, `resolve_share_link(token)` ; `receipts::record_send` appelé **après confirmation de l'utilisateur** ; construction de l'adresse `wa.me` avec le texte minimal (nom, montant, mois, nom et téléphone de l'agence, lien de la quittance) ; refus si le locataire n'a pas de numéro.
4. Logo et signature : réencodage côté serveur (suppression des métadonnées dont la localisation), 200 Ko à l'envoi, 30 Ko à l'affichage.
5. Audit : `receipt.sent`, `receipt.link_created`, `receipt.link_revoked`, `receipt.pdf_failed`. Événements serveur : `receipt_pdf_rendered`, `receipt_pdf_failed`, `receipt_shared`, `share_link_opened`, `share_link_expired`.

### 5.3 API
`GET /receipts/{id}/pdf?paper=a4|a5` (flux, `Content-Disposition: attachment; filename="quittance-HOR-2026-000094.pdf"`, `nosniff`, `no-store`, 409 `receipt.pdf_not_ready` avec `Retry-After`), `POST /receipts/{id}/sends`, `GET /receipts/{id}/sends`, `POST /share-links/{id}/revoke`, `PUT /settings/logo`, `PUT /settings/signature`, `GET /operations/{id}` (filtre explicite sur l'organisation, la table `jobs` n'a pas de RLS). Pages publiques hors `/api` : `GET /s/{jeton}` (HTML rendu par Rust, sans JavaScript applicatif, `noindex`, `Referrer-Policy: no-referrer`, limitation de débit par adresse), `GET /s/{jeton}/pdf` ; `410` lien expiré ou révoqué, `404` jeton inconnu.

### 5.4 Interface
S25 : quittance en aperçu, numéro, statut, historique des envois, liens vers le paiement et vers la quittance remplacée ou remplaçante (étape 09), boutons « Envoyer sur WhatsApp » (panneau de contrôle du destinataire : nom et deux derniers chiffres), « Imprimer A4/A5 » (activés seulement si le PDF est `ready`), « Copier le lien », « Réimprimer » ; état « un nouvel essai est en cours » ; S26 (page publique : le document seul, texte « Ce lien a expiré. Demandez un nouveau lien à {organisation}. ») ; S17 : section quittance (en-tête, logo, signature, préfixe, format par défaut, durée des liens, aperçu de test) ; S11 : actions de quittance. Feuille d'impression sans navigation.

### 5.5 Documents PDF
Gabarit de quittance A4 et A5 : lisible en noir et blanc, aucun fond coloré plein, une seule page dans tous les cas, accents corrects, nom jusqu'à deux lignes, logo 40 × 20 mm au plus, mention « TVA incluse » si applicable, mention du tiers payeur, solde restant, avance.

### 5.6 Audit et événements
Voir 5.2 point 5.

### 5.7 Tests
- Visuels (`TESTING_QA.md` §6) : les 20 cas types ; une page en A4 et A5 ; texte extrait égal à l'instantané ; polices embarquées ; aucune métadonnée ; aucune référence externe ni requête sortante pendant le rendu ; rendu en niveaux de gris lisible ; image de référence approuvée par un humain dans la PR ; deux rendus du même instantané ⇒ même empreinte ; p95 ≤ 1 s ; mémoire compatible avec un serveur de 2 Go.
- Échec de rendu simulé : le paiement reste enregistré, la quittance en file, l'alerte part, le nouvel essai réussit (R-049).
- Liens : E-12 (accès refusé après expiration), révocation, jeton inconnu (404), un seul document par lien, aucune donnée de l'organisation sur la page, absence de JavaScript applicatif, en-têtes (A12), jeton absent des journaux de l'application et du proxy (A13), limitation de débit.
- Sécurité du rendu : nom `<b>` et adresse `https://…` restent inertes ; adresse d'un fichier d'une autre organisation refusée (A14).
- À la main : impression laser A4 et A5 en noir et blanc, TalkBack sur le parcours « Encaisser puis envoyer ».
- E2E : parcours 1 jusqu'à « Envoyer sur WhatsApp » (adresse `wa.me` construite, non ouverte).

## 6. Critères d'acceptation vérifiables

1. Après un paiement, le PDF A4 et A5 est prêt en moins de 10 secondes en essai ; sinon l'alerte « quittance sans PDF » part à 10 minutes.
2. Un échec de rendu ne supprime ni ne bloque le paiement ; la quittance est rendue au nouvel essai (R-049).
3. Le lien de partage ouvre le seul document, sans compte ; après expiration ou révocation il répond 410 avec le texte de S26 (E-12) ; avec un jeton inconnu, 404.
4. Les 20 cas types tiennent sur une page en A4 et en A5, avec accents corrects.
5. Chaque envoi (WhatsApp, impression, lien copié) figure dans l'historique de la quittance (R-055).
6. Le message WhatsApp ne contient que le nom, le montant, le mois, l'agence et le lien ; le panneau de contrôle du destinataire s'affiche avant l'ouverture.
7. Le PDF ne contient aucune métadonnée d'auteur et sa lecture n'effectue aucune requête sortante.
8. L'agence candidate a vu une quittance imprimée A4 et A5 et a validé par écrit la mise en page et les mentions, ou le risque est accepté par écrit (PP-27).

## 7. Sécurité et déploiement à vérifier

Processus de rendu sans réseau ni secret (PP-19) ; jetons de 128 bits au moins, seule l'empreinte en base ; adresses à jeton masquées dans les journaux du proxy ; `Cache-Control: no-store` sur PDF et pages publiques ; moteur déployé selon la décision n° 7 (conteneur séparé ou sous-processus restreint, `DEPLOYMENT.md` §2.1) ; taille de l'image et mémoire du serveur recalées avec les mesures du prototype (a).

## 8. Liste de démonstration (Mimis)

1. Encaisser 75 000 FCFA : ouvrir la quittance, attendre « PDF prêt », imprimer l'A5 puis l'A4 sur l'imprimante laser noir et blanc.
2. Envoyer sur WhatsApp : lire le panneau (« Issouf Tapsoba, numéro finissant par 01 »), ouvrir WhatsApp avec le texte prêt (sur son propre téléphone).
3. Ouvrir le lien reçu comme le ferait le locataire : voir la quittance seule, sans connexion.
4. Révoquer le lien : le rouvrir et lire « Ce lien a expiré ».
5. Le développeur coupe le moteur PDF en essai : encaisser, constater que le paiement existe et « un nouvel essai est en cours », rétablir et voir le PDF.
6. Sur une organisation de série d'essai : voir le filigrane « ESSAI ».

## 9. Définition de terminé, risques, hors étape

Terminé : critères 1 à 8 verts (le 8 peut être « risque accepté » consigné) ; images de référence approuvées versionnées ; budget de poids toujours respecté ; `API_CONTRACTS.md`, `DECISIONS.md` (n° 7, 83), `DEPLOYMENT.md` si l'image change, `CLAUDE.md` à jour.

| Risque | Mesure |
|---|---|
| Mentions refusées par le juriste ou l'agence | Gabarit paramétrable (en-tête, mentions complémentaires de `org_settings`), nouvelle version sans toucher aux quittances déjà émises (instantanés) |
| PDF trop lourd ou lent sur petit serveur | Mesures du prototype (a), rendu asynchrone, aperçu HTML immédiat |
| Locataire sans WhatsApp | Impression ou lien copié, numéro à saisir |

Hors étape : annulation, rectificative et filigrane « ANNULÉE » (étape 09), code de contrôle et page de vérification (après le pilote, DP-64, n° 55), envoi automatique par l'API WhatsApp (F27, hors MVP), quittance rectificative sans effet sur le montant (route réservée, 404).

## 10. Couverture (lue par `roadmap/verifier-couverture.py`)

- Étape : 08
- Dépend de : 07
- Tranche : T1
- Jalon : J2
- F : F10
- R : R-049, R-052, R-054, R-055, R-056
- E : E-12
- PP : PP-19, PP-27, PP-34
- Contenus : C-04, C-06, C-08
