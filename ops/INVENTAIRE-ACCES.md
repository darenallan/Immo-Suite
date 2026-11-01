# Inventaire des accès (DEPLOYMENT.md §3.3) — daté du 01/11/2026

Aucun secret, aucun mot de passe, aucun code de secours dans ce fichier. Il dit **qui doit détenir quoi** et **où on en est**.
État au 01/11/2026 : **aucun compte d'organisation n'existe encore** (C-01, C-02, C-03 non fournis). Titulaire proposé : HABEK Technology Company (DP-63).

| Ressource | Administrateurs prévus | État au 01/11/2026 | Qui doit agir |
|---|---|---|---|
| Nom de domaine (registrar) | développeur + porteur du projet | **non choisi** (C-03, décision n° 13) | Mimis |
| DNS (CAA, HSTS sans preload, SPF/DMARC « aucun expéditeur ») | idem | non créé | après C-03 |
| Hébergeur (serveur, stockage objet) | développeur ; facturation : porteur du projet | **non choisi** (C-02) | Mimis |
| Base de données | développeur | Neon, plan gratuit, projets d'essai `immo-spikes` et `immo-spikes-restore` ; **clé d'API `napi_…` à révoquer** (collée dans le chat) | Mimis |
| Second fournisseur de sauvegardes (compte distinct, écriture seule, verrouillage d'objet) | développeur + porteur du projet | **non choisi** (DP-82) | Mimis |
| Dépôt, CI, registre d'images (compte d'organisation, double authentification) | développeur + porteur du projet (propriétaire de secours) | dépôt **local seulement**, aucune plateforme reliée (C-01) | Mimis |
| Gestionnaire de mots de passe de l'équipe | développeur ; accès d'urgence : porteur du projet | à confirmer | Mimis |
| Clé privée des sauvegardes (deux exemplaires, hors hébergeur) | développeur + porteur du projet | **non générée** : `immo backup keygen <fichier>` ; décision DP-82 : deux exemplaires détenus par « nous » | Mimis et développeur ensemble |
| Clé de chiffrement des champs | idem | non créée (étape ultérieure) | — |
| Sonde externe, page d'état, canal d'alerte | développeur + porteur du projet | **canal non choisi** (décision n° 89 : e-mail d'équipe et messagerie [À valider]) | Mimis |
| Comptes de la console HABEK (TOTP, nominatifs) | — | étape ultérieure | — |
| Jetons AI Gateway (collés dans le chat) | — | **à révoquer** | Mimis |

## Rappels de procédure
- Deux appareils d'authentification par personne ; codes de secours imprimés, **deux endroits physiques distincts**.
- Revue de cet inventaire chaque trimestre. **Exercice « personne indisponible » : à planifier avec Mimis** (aucune date inventée ; premier trimestre après la mise en place des comptes).
- Départ d'une personne : désactivation le jour même, rotation des secrets, revue des accès de l'hébergeur.
