# F10 — Quittance PDF, impression et partage WhatsApp (étape 08)

| Champ | Valeur |
|---|---|
| Statut | **Rédigée le 06/11/2026, validée par Mimis le 06/11/2026** (recommandations du §10 acceptées, DP-98) |
| Étape | 08, `roadmap/etape-08-quittance-pdf-et-partage.md` §4 |
| Dépend de | dossier §16 F10 ; `specs/F10-numerotation-et-instantane.md` (étape 07) ; DP-33, DP-61, DP-62, DP-64, DP-76 (moteur : Typst en bibliothèque), DP-96, DP-97 ; décisions n° 39 et 68 (**ouvertes**), 55 (après le pilote), 83 (**ouverte**) ; `ARCHITECTURE.md` §6, §3.9, §3.10 ; `SECURITY.md` §7.5, §7.8 ; `DATABASE_SCHEMA.md` `receipt_files`, `receipt_sends`, `share_links`, `org_settings`, §6.7 ; `API_CONTRACTS.md` §5.4, §5.5, §6.1, §7.6 ; `UI_UX_DESIGN.md` « Quittance : mise en page imprimable », S25, S26, S17 ; `TESTING_QA.md` §6, A12 à A14 ; `spikes/pdf/NOTE.md` §8 |
| Règles | R-049, R-052 [À VÉRIFIER AVEC UN JURISTE, décision n° 12], R-053, R-054, R-055, R-056, R-088, R-092 |
| Cas chiffrés | E-12 ; 75 000 FCFA → « soixante-quinze mille francs CFA » ; les 20 cas types de `TESTING_QA.md` §6 |
| Contenus attendus | **C-08** (mentions de la quittance, à rédiger par Mimis), **C-04** (avis juridique ou risque accepté par écrit, DP-62), **C-06** (agence candidate qui valide la mise en page, PP-27) : **aucun n'est encore livré** (question 1) |
| Hors fiche | annulation, rectificative et filigrane « ANNULÉE » (étape 09) ; code de contrôle et page de vérification (après le pilote, n° 55) ; envoi automatique par l'API WhatsApp (F27) ; compte rendu partagé (étape 13) |

## 1. Objet

Rendre l'instantané de chaque quittance en PDF A4 et A5, en tâche de fond, sans jamais bloquer l'encaissement ; l'imprimer ; le partager par un lien WhatsApp pré-rempli ou un lien copié, vers une page publique qui ne montre que ce document et expire ; consigner chaque envoi.

## 2. Règles

| N° | Règle | Source |
|---|---|---|
| 1 | **Rendu en tâche de fond** : chaque quittance émise crée, dans la transaction du paiement, une tâche `render_receipt` (identifiants seulement). Le rendu lit **l'instantané** (`receipts.snapshot`), jamais les tables vivantes ; le montant en lettres est celui de l'instantané (jamais recalculé par le moteur) | R-049, DP-76, F10-numérotation règle 9 |
| 2 | **Deux formats à chaque quittance** : A5 (par défaut, `org_settings.default_paper`) et A4 ; une page, toujours ; mise en page de `UI_UX_DESIGN.md` (contenu de haut en bas, marges, corps, titre, total) ; noir et blanc lisible, aucun fond plein | R-056, DP-64 |
| 3 | **Moteur** : Typst 0.15.x figé, compilé dans le binaire `immo`, exécuté dans le **processus isolé** `immo pdf-worker` (environnement vidé, sans réseau ni secret, délai 10 s, sortie bornée à 5 Mo), polices IBM Plex Sans statiques embarquées (OFL), gabarit `templates/quittance.typ`. Toute valeur venue de l'instantané est passée en **données** (jamais concaténée dans le source du gabarit) : un nom `<b>` ou une adresse `https://…` restent du texte | DP-76, `SECURITY.md` §7.5, A14 |
| 4 | **Fichiers** : `receipt_files` (une ligne par format et version du moteur, empreinte SHA-256, taille), contenu dans le stockage de fichiers sous la clé générée par le serveur `org/{org_id}/receipts/{receipt_id}-{a4\|a5}.pdf` ; ajout seul. PDF sans métadonnée d'auteur, titre neutre (« Quittance »), même empreinte pour deux rendus du même instantané | DB `receipt_files`, `TESTING_QA.md` §6 |
| 5 | **Échec de rendu** : le paiement et la quittance restent ; la tâche est réessayée (file de l'étape 01 : 30 s, 1 min, 2 min…) ; `pdf.state` = `pending` tant qu'aucun fichier, `failed` après le cinquième échec ; **alerte S2** `receipt.pdf_missing` si une quittance n'a pas ses deux PDF 10 minutes après son émission (contrôle périodique du travailleur) [question 9] | R-049, étape §5.2 point 1 |
| 6 | **Filigranes** : « ESSAI » (série `trial`, déjà dans l'instantané) ; « DÉMONSTRATION — sans valeur » (organisation `demo`) [question 6] ; « ANNULÉE — remplacée par la quittance n° … » à l'étape 09 | DP-61, `SECURITY.md` §7.5 |
| 7 | **Téléchargement** : `GET /receipts/{id}/pdf?paper=a4\|a5` sert le fichier **en flux par l'API** après contrôle d'organisation et de droit ; `409 receipt.pdf_not_ready` avec `Retry-After: 5` tant qu'il n'existe pas ; en-têtes `Content-Type: application/pdf`, `Content-Disposition: attachment; filename="quittance-{numéro}.pdf"` (`inline` accepté pour un PDF produit par le système), `X-Content-Type-Options: nosniff`, `Cache-Control: no-store` [question 4] | `API_CONTRACTS.md` §5.5, n° 68 |
| 8 | **Lien de partage** : jeton aléatoire de **256 bits** (générateur du système, base64url) [question 10], seule son empreinte SHA-256 est stockée (`share_links.token_hash`) ; un seul document ; expiration `org_settings.share_link_ttl_days` (30 jours par défaut, 1 à 365) ; révocation par `revoked_at`. « Copier le lien » crée un nouveau lien (le jeton n'est montré qu'une fois) | R-054, DB §6.7 |
| 9 | **Page publique S26** `GET /s/{jeton}` (hors `/api`, sans session) : HTML rendu par le serveur, **sans JavaScript applicatif**, en-têtes `noindex`, `Referrer-Policy: no-referrer`, `Cache-Control: no-store`, CSP stricte ; montre le document seul (aperçu HTML de l'instantané, boutons « Télécharger » et « Imprimer » vers `GET /s/{jeton}/pdf`) ; **410** avec « Ce lien a expiré. Demandez un nouveau lien à {organisation}. » si expiré ou révoqué ; **404** « Introuvable » si inconnu ; débit limité par adresse (30 requêtes par minute) ; le jeton n'apparaît dans aucun journal (chemin masqué en `/s/:token`) | R-054, E-12, A12, A13, `SECURITY.md` §7.8 |
| 10 | **Lecture publique par empreinte** : la page pose `app.token_hash` ; une politique de `share_links` ne rend que la ligne qui porte cette empreinte ; le gestionnaire lit ensuite la quittance dans une seconde transaction ouverte au nom de l'organisation du lien. Aucun rôle `BYPASSRLS` [question 3] | DB §6.7, n° 39 |
| 11 | **Envoi WhatsApp** (`POST /receipts/{id}/sends`, canal `whatsapp`) : le serveur crée le lien, compose le message **minimal** et l'adresse `https://wa.me/{numéro sans +}?text=…` ; il renvoie le destinataire affichable (nom, **deux derniers chiffres**) ; le front montre le panneau de contrôle **avant** d'ouvrir WhatsApp ; l'envoi est consigné **au clic de confirmation** (une ligne `receipt_sends`). Destinataire : le locataire principal du bail, son téléphone ; sans numéro : `422 receipt.recipient_has_no_phone` (le front propose l'impression ou le lien copié) [question 5] | DP-33, R-055, `SECURITY.md` §7.8 |
| 12 | **Message** (texte constant, aucune variable libre) : « Bonjour {nom}, voici votre quittance n° {numéro} pour {montant} (loyer de {mois}) : {lien} — {agence}, {téléphone de l'agence} » [question 5] | `MAQUETTE.md`, `SECURITY.md` §7.8 |
| 13 | **Autres canaux consignés** : `print_a4`, `print_a5` (au clic « Imprimer »), `link_copied` (au clic « Copier le lien ») ; historique `GET /receipts/{id}/sends` (canal, qui, quand) | R-055 |
| 14 | **Paramètres de la quittance (S17)** : préfixe, format par défaut, durée des liens, en-tête (nom affiché, adresse, téléphone), identifiants légaux, mentions complémentaires ; **logo** et **signature** (`PUT /settings/logo`, `PUT /settings/signature`, PNG ou JPEG, 200 Ko au plus, réencodés côté serveur sans métadonnées, 30 Ko à l'affichage). Un changement ne touche **aucune** quittance déjà émise (instantané) [question 7] | étape §5.2 point 4 |
| 15 | **Reprise** : les quittances émises à l'étape 07 sans fichier reçoivent leur tâche de rendu par une reprise unique au démarrage du travailleur | DP-96 |
| 16 | **Droits** : PDF et historique : `payment.read` ; envoi, lien, révocation : `receipt.issue` ; paramètres : `settings.write`. Matrice A3 mise à jour | F02 |
| 17 | **Audit** : `receipt.sent`, `receipt.link_created`, `receipt.link_revoked`, `receipt.pdf_failed`. **Événements serveur** (liste blanche) : `receipt_pdf_rendered` (format, durée), `receipt_pdf_failed`, `receipt_shared` (canal), `share_link_opened`, `share_link_expired` | étape §5.2 point 5 |

## 3. Données (migration de l'étape)

1. `receipt_files`, `receipt_sends`, `share_links` **comme dans `DATABASE_SCHEMA.md`** ; `receipt_files` et `receipt_sends` en ajout seul ; `share_links` : SELECT, INSERT, UPDATE (`revoked_at`) pour `immo_app`.
2. Politique de lecture publique de `share_links` par `app.token_hash` (DB §6.7), test ISO-9.
3. `org_settings` : colonnes de référence du logo et de la signature déjà présentes (`logo_storage_key`, `signature_storage_key`) ; l'en-tête tient dans `letterhead` (JSON : nom affiché, adresse, téléphone).
4. Registres (`cross_org_registry`, `purge_order`) et second avis C-19 sur la politique publique.

## 4. Gabarit et rendu

- `templates/quittance.typ` (repris du prototype (a), défaut d'espacement de l'en-tête corrigé) ; `templates/fonts/` : IBM Plex Sans Regular et Bold **statiques** (instances de la police variable du prototype) et `OFL.txt`.
- Entrée du gabarit : l'instantané plus le format, le filigrane et le logo et la signature éventuels (octets passés au moteur, jamais une adresse).
- Troncature : nom sur deux lignes au plus avec « … », jamais de seconde page ; logo 40 × 20 mm au plus.
- Contrôles automatiques (décision n° 83) sur les 20 cas types en A4 et A5 : une page, texte extrait égal à l'instantané, polices embarquées, aucune métadonnée d'auteur, aucune référence externe (`/URI`, `/Launch`), aucun pixel coloré, même empreinte sur deux rendus, p95 ≤ 1 s [question 2].

## 5. API

| Route | Droit | Remarque |
|---|---|---|
| `GET /receipts/{id}/pdf?paper=a4\|a5` | `payment.read` | flux, `409 receipt.pdf_not_ready` + `Retry-After` |
| `POST /receipts/{id}/sends` | `receipt.issue` | `{ channel: whatsapp\|print_a4\|print_a5\|link_copied }` → `{ send, share_link?, recipient?, message? }` (§6.1) ; `Idempotency-Key` acceptée |
| `GET /receipts/{id}/sends` | `payment.read` | historique |
| `POST /share-links/{id}/revoke` | `receipt.issue` | → lien révoqué |
| `PUT /settings/logo`, `PUT /settings/signature` | `settings.write` | multipart, PNG ou JPEG, 200 Ko |
| `GET /s/{jeton}`, `GET /s/{jeton}/pdf` | public | S26, 410 et 404 |

`GET /receipts/{id}` porte désormais `pdf: { state, papers, sha256 }` réel. Erreurs : `receipt.pdf_not_ready` (409), `receipt.recipient_has_no_phone` (422), `share_link.expired` (410, page publique), `upload.too_large` (413), `upload.type_not_allowed` (415).

## 6. Écrans

- **S25** (aperçu de la quittance, après S04 et depuis S11) : état du PDF (« PDF prêt » ou « Un nouvel essai est en cours »), « Envoyer sur WhatsApp » (panneau : « Issouf Tapsoba, numéro finissant par 01 » puis « Ouvrir WhatsApp »), « Imprimer A5 » et « Imprimer A4 » (ouvrent le PDF ; actifs seulement si prêt), « Copier le lien », historique des envois.
- **S26** : page publique sans menu ni autre donnée de l'organisation.
- **S17** : section « Quittance » (champs de la règle 14, logo, signature, aperçu de test).

## 7. Critères (étape §6) et tests

- Rendu réel des 20 cas types A4 et A5 avec les contrôles du §4 ; échec simulé (moteur coupé) : paiement présent, `pending`, nouvel essai réussi après rétablissement (R-049) ; alerte à 10 minutes.
- Liens : E-12 (410 après expiration, horloge déplacée), révocation, jeton inconnu (404), un seul document, aucune donnée de l'organisation hors document, pas de JavaScript, en-têtes (A12), jeton absent des journaux (A13), débit limité.
- Envoi : message minimal exact, `wa.me` construite et **non ouverte** en E2E, envoi consigné, locataire sans numéro.
- Sécurité du rendu : balisage et adresse inertes ; fichier d'une autre organisation refusé (A14).
- Pile réelle (390 et 1440 px) : démonstration §8 de l'étape (sauf l'impression laser et WhatsApp sur téléphone, à la main par Mimis).

## 8. Cas limites

| Cas | Comportement |
|---|---|
| Locataire sans numéro | `422 receipt.recipient_has_no_phone` ; impression ou lien copié proposés |
| Nom avec balisage ou adresse web | texte inerte dans le PDF et sur S26 |
| Logo absent | en-tête sans logo ; logo maximal : 40 × 20 mm |
| Montant maximal (999 999 999) | une page, montant en lettres sur deux lignes au plus |
| Quittance d'essai réimprimée | filigrane « ESSAI » conservé |
| PDF demandé avant le rendu | `409 receipt.pdf_not_ready`, bouton inactif, nouvel essai automatique côté écran |

## 9. Ce qui reste hors de cette étape

Rectificative et annulée (étape 09) ; code de contrôle (n° 55) ; compte rendu partagé (étape 13) ; stockage objet distant (S3) : le stockage sur disque suffit en local.

## 10. Questions pour Mimis (avec recommandation)

1. **Contenus C-08, C-04, C-06 non livrés** : *recommandation : développer avec les mentions de R-052 (bailleur ou agence, locataire, lot, période, montant en chiffres et en lettres, solde restant) et les mentions complémentaires de Paramètres ; garder le critère 8 (validation de l'agence candidate, PP-27) **ouvert** jusqu'à vos contenus, ou à votre acceptation écrite du risque (DP-62)*. L'étape peut être faite, pas déclarée « prête pour un pilote ».
2. **Décision n° 83 (tests visuels)** : *recommandation : contrôles automatiques du §4 sur les 20 cas, plus une planche PNG de référence par format relue et approuvée par vous une fois, puis protégée par l'empreinte SHA-256 du PDF* (le rendu est reproductible) ; une différence d'empreinte oblige à régénérer et à faire relire la planche.
3. **Décision n° 39 (lecture publique)** : *recommandation : politique par empreinte de jeton (`app.token_hash`), sans rôle `BYPASSRLS`* (DB §6.7, déjà retenue par le schéma). À enregistrer en DP-98.
4. **Décision n° 68 (téléchargement)** : *recommandation : flux par l'API au pilote* (contrat §5.5 (a)). À enregistrer dans DP-98.
5. **Message WhatsApp** : `MAQUETTE.md` donne le numéro de quittance mais pas le téléphone de l'agence ; `SECURITY.md` §7.8 demande le téléphone de l'agence. *Recommandation : le texte de la règle 12 (numéro, montant, mois, lien, agence et son téléphone), destinataire = locataire principal, son téléphone.*
6. **Filigrane « DÉMONSTRATION »** : l'instantané v1 ne porte pas la nature de l'organisation. *Recommandation : version 2 de l'instantané avec `issuer.kind` pour les nouvelles quittances ; les quittances déjà émises gardent leur instantané v1 (aucune organisation de démonstration n'en a encore).*
7. **Logo et signature** : *recommandation : les faire à cette étape (réencodage côté serveur par la bibliothèque `image`, PNG ou JPEG seulement)* ; sinon l'en-tête reste textuel jusqu'à une étape ultérieure.
8. **Impression** : *recommandation : imprimer depuis le PDF (seule voie validée sur papier, DP-76)* ; l'aperçu HTML reste pour l'écran et la page S26.
9. **Alerte « quittance sans PDF »** : *recommandation : contrôle toutes les 5 minutes par le travailleur, alerte S2 au journal et dans `platform_events` pour toute quittance sans ses deux fichiers 10 minutes après émission (une alerte par quittance)*.
10. **Longueur du jeton** : *recommandation : 256 bits (au-delà des 128 exigés), encodage base64url de 43 caractères*.
