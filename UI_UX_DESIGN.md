# Immo Suite — UI/UX et design system

| Champ | Valeur |
|---|---|
| Statut | Propriétaire de l'interface : écrans, composants, textes, tokens, accessibilité et performance |
| Version | 0.2 |
| Date | 30/10/2026 |
| Dépend de | Dossier de cadrage v1.0 (§14), `CONTEXTE.md`, `DOMAINE.md`, `DECISIONS.md`, `SECURITY.md` (filigranes, §7.5), `ARCHITECTURE.md` (§13.2) |
| Alimente | `MAQUETTE.md`, `maquette/`, `API_CONTRACTS.md`, `TESTING_QA.md`, `CLAUDE.md` |

Sources : dossier de cadrage v1.0 (§14 surtout, §5, §11, §12.1, §16), `CONTEXTE.md`, `DOMAINE.md`. Ce document sert à la maquette (`MAQUETTE.md`) et au vrai produit. La palette (bleu nuit, blanc, turquoise) est **arrêtée** (décision n° 16, DP-44 dans `DECISIONS.md`). Le vouvoiement reste **provisoire** (décision ouverte n° 17).

## Principes de design

1. **Encaisser en moins de 30 secondes, sans formation** (médiane). Quatre gestes : rechercher, saisir le montant, vérifier le mode (« Espèces » par défaut), valider.
2. **Comprendre l'accueil en 5 secondes.** Trois chiffres au plus au-dessus de la ligne de flottaison : encaissé, attendu, en retard.
3. **Android d'entrée de gamme, connexion lente, plein soleil.** Écran de 360 px de large minimum [hypothèse]. Pages légères, pas d'animation lourde, texte à fort contraste.
4. **L'argent est sacré.** Pas de bouton « Supprimer » sur un paiement ou une quittance. Une erreur se corrige par une demande de correction puis une contre-passation (R-047, R-048).
5. **Jamais la couleur seule.** Chaque statut a une icône, un texte et une forme.
6. **Une action principale par écran.** Un seul bouton primaire visible.
7. **Confirmer seulement ce qui a un enjeu.** Pas de modale pour encaisser ni pour relancer.
8. **Ne jamais perdre une saisie.** Coupure réseau : l'écran garde les données et propose de réessayer sans doublon (R-042).
9. **Le vocabulaire du métier.** Quittance, bail, arriéré, caution. Aucun jargon logiciel.
10. **Dire la vérité sur ce qui existe.** Dans la maquette, chaque écran porte un repère « V1 » ou « À venir ».

Choix de non-faire en V1 : pas de thème sombre (usage en plein soleil), pas d'animation décorative, pas de défilement infini, pas d'action importante portée par une icône seule.

## Design tokens

Bloc source des tokens. La maquette le copie tel quel dans `maquette/css/tokens.css`. Le vrai produit le réutilise.

```css
:root {
  /* Marque : bleu nuit, blanc, turquoise (décision n° 16, palette retenue le 25/10/2026) */
  --color-brand-600: #142B4A;      /* primaire : boutons, fond du menu latéral, soulignements (bleu nuit) */
  --color-brand-700: #0D1E36;      /* survol, appui, texte sur fond teinté */
  --color-brand-100: #DCE6F3;      /* fond teinté */
  --color-brand-050: #EAF0F8;
  --color-accent-500: #159D91;     /* accent turquoise : remplissage ou bordure UNIQUEMENT, jamais en texte (3,3:1 sur blanc) */
  --color-accent-300: #5FD3C6;     /* turquoise clair : accent sur fond bleu nuit (menu latéral, anneau de focus) */
  --color-accent-100: #DDF3F0;

  /* Neutres froids */
  --color-bg: #F3F6FA;
  --color-surface: #FFFFFF;
  --color-surface-sunken: #E9EEF5;
  --color-border: #D5DDE8;          /* séparateurs décoratifs */
  --color-border-input: #6F7F97;    /* contours de champs et de contrôles (≥ 3:1) */
  --color-text: #263449;
  --color-text-secondary: #44546C;
  --color-text-muted: #5A6B84;
  --color-text-on-brand: #FFFFFF;
  --color-text-on-accent: #0A1626;
  --color-link: #0E6F66;            /* turquoise foncé : liens sur fond clair (6,0:1 sur blanc) */
  --color-focus: #142B4A;
  --color-overlay: rgba(20, 43, 74, 0.55);

  /* Menu latéral sur fond bleu nuit (textes ≥ 7:1 sur --color-brand-600) */
  --nav-text: #DCE6F3;
  --nav-text-muted: #B8C7DC;
  --nav-active-bg: #1E3A63;

  /* Statuts financiers (fond / texte). Toujours avec icône + libellé */
  --status-paid-bg: #E3F2E8;     --status-paid-fg: #1F5E3A;
  --status-partial-bg: #FCE8D8;  --status-partial-fg: #8A3A00;
  --status-late-bg: #FBE4E1;     --status-late-fg: #9B1C1C;
  --status-upcoming-bg: #E4ECF4; --status-upcoming-fg: #2F4B66;
  --status-due-bg: #FDF1D5;      --status-due-fg: #6B4600;
  --status-credit-bg: #ECE8F8;   --status-credit-fg: #4B3A8C;
  --status-void-bg: #E8ECF1;     --status-void-fg: #4A5568;
  --status-due-mark: #B7791F;    /* repère graphique « due » (barres), jamais en texte */

  /* Badges neutres : lot, bail, quittance, usage (aucune couleur de statut financier) */
  --neutral-badge-bg: var(--color-surface-sunken);
  --neutral-badge-fg: var(--color-text-secondary);

  /* Retours (réutilisent les paires de statuts) */
  --feedback-success-bg: var(--status-paid-bg);     --feedback-success-fg: var(--status-paid-fg);
  --feedback-warning-bg: var(--status-due-bg);      --feedback-warning-fg: var(--status-due-fg);
  --feedback-error-bg: var(--status-late-bg);       --feedback-error-fg: var(--status-late-fg);
  --feedback-info-bg: var(--status-upcoming-bg);    --feedback-info-fg: var(--status-upcoming-fg);

  /* Typographie : polices système, aucun téléchargement */
  --font-sans: "Segoe UI", Roboto, "Helvetica Neue", Arial, system-ui, sans-serif;
  --font-numeric: tabular-nums;
  --text-sm: 0.875rem;    /* 14 px : minimum pour tout texte utile */
  --text-base: 1rem;      /* 16 px : corps et champs (évite le zoom iOS) */
  --text-lg: 1.125rem;
  --text-xl: 1.25rem;
  --text-2xl: 1.5rem;
  --text-3xl: 1.875rem;   /* chiffres clés */
  --leading-body: 1.5;
  --leading-title: 1.25;
  --weight-regular: 400;
  --weight-semibold: 600;
  --weight-bold: 700;

  /* Espacements (base 4 px) */
  --space-1: 4px;  --space-2: 8px;  --space-3: 12px; --space-4: 16px;
  --space-5: 20px; --space-6: 24px; --space-8: 32px; --space-10: 40px;
  --space-12: 48px; --space-16: 64px;

  /* Rayons */
  --radius-sm: 6px;  --radius-md: 10px;  --radius-lg: 16px;  --radius-pill: 999px;

  /* Ombres (légères : écrans bon marché, plein soleil) */
  --shadow-1: 0 1px 2px rgba(20, 43, 74, 0.08);
  --shadow-2: 0 4px 12px rgba(20, 43, 74, 0.12);
  --shadow-3: 0 12px 32px rgba(20, 43, 74, 0.20);

  /* Mise en page */
  --target-min: 44px;       /* cible tactile minimale */
  --target-primary: 52px;   /* bouton primaire sur mobile */
  --topbar-height: 56px;
  --bottombar-height: 64px;
  --sidebar-width: 248px;
  --content-max: 1200px;
  --gutter-mobile: 16px;
  --gutter-desktop: 24px;

  /* Mouvement */
  --motion-fast: 120ms;
  --motion-base: 200ms;

  /* Couches */
  --z-sticky: 10; --z-nav: 20; --z-sheet: 30; --z-modal: 40; --z-toast: 50;
}

@media (prefers-reduced-motion: reduce) {
  :root { --motion-fast: 0ms; --motion-base: 0ms; }
}
```

Les points de rupture ne se déclarent pas en variables CSS : 600 px et 1024 px (voir Responsive).

### Contraste des paires principales

Ratios calculés avec la formule WCAG 2.1, valeurs arrondies. À revérifier avec un outil avant le tournage des vidéos.

| Usage | Texte sur fond | Ratio | Seuil |
|---|---|---|---|
| Texte courant | `--color-text` sur `--color-bg` | 11,6:1 | AA (4,5) et AAA (7) |
| Texte secondaire | `--color-text-secondary` sur `--color-bg` | 7,1:1 | AA et AAA |
| Texte discret | `--color-text-muted` sur `--color-surface-sunken` | 4,7:1 | AA |
| Bouton primaire | blanc sur `--color-brand-600` | 14,3:1 | AA et AAA |
| Lien | `--color-link` sur blanc | 6,0:1 | AA |
| Bouton accent | `--color-text-on-accent` sur `--color-accent-500` | 5,4:1 | AA |
| Contour de champ | `--color-border-input` sur blanc | 4,1:1 | 3:1 (composants) |
| Payé | `--status-paid-fg` sur `--status-paid-bg` | 6,7:1 | AA |
| Partiel | `--status-partial-fg` sur `--status-partial-bg` | 6,6:1 | AA |
| En retard | `--status-late-fg` sur `--status-late-bg` | 6,7:1 | AA |
| À venir | `--status-upcoming-fg` sur `--status-upcoming-bg` | 7,6:1 | AA |
| Due | `--status-due-fg` sur `--status-due-bg` | 7,5:1 | AA |
| Avance | `--status-credit-fg` sur `--status-credit-bg` | 7,6:1 | AA |
| Annulé | `--status-void-fg` sur `--status-void-bg` | 6,3:1 | AA |
| Badge neutre | `--neutral-badge-fg` sur `--neutral-badge-bg` | 6,6:1 | AA |
| Repère « V1 » | `--color-brand-700` sur `--color-brand-100` | 13,3:1 | AA et AAA |
| Repère « À venir » | `--color-brand-700` sur `--color-accent-100` | 14,5:1 | AA et AAA |
| Soulignement d'onglet actif | `--color-brand-600` sur `--color-bg` | 13,1:1 | 3:1 (composants) |
| Menu latéral | `--nav-text` sur `--color-brand-600` | 11,3:1 | AA et AAA |
| Menu latéral, texte discret | `--nav-text-muted` sur `--color-brand-600` | 8,3:1 | AA et AAA |
| Menu latéral, entrée active | blanc sur `--nav-active-bg` | 11,4:1 | AA et AAA |
| Accent clair sur bleu nuit | `--color-accent-300` sur `--color-brand-600` | 7,9:1 | 3:1 (composants) |
| Anneau de focus | `--color-focus` sur `--color-bg` | 13,1:1 | 3:1 (composants) |

Interdits :

- Le turquoise `--color-accent-500` n'est jamais une couleur de texte sur fond clair (3,3:1 sur blanc, 3,1:1 sur `--color-bg`). Pour un texte turquoise, utiliser `--color-link`.
- Le texte blanc n'est jamais posé sur `--color-accent-500` (3,3:1). Utiliser `--color-text-on-accent`.
- Le turquoise et le vert « Payé » ne se remplacent pas l'un l'autre : le turquoise est une couleur de marque, le vert porte uniquement le statut « Payé ». Un statut n'utilise jamais `--color-accent-500`.
- `--color-border` ne porte jamais le contour d'un champ.

### Statuts financiers

Chaque badge = icône + libellé + couleur. Statuts alignés sur DOMAINE.md (R-032, R-033, R-095, DP-13, DP-27).

| Statut | Libellé | Icône (forme) | Fond / texte | Précision d'affichage |
|---|---|---|---|---|
| Payé | Payé | cercle coché | `--status-paid-*` | Montant payé. Statut d'une échéance |
| Validé | Validé | cercle coché | `--status-paid-*` | Statut d'un paiement enregistré (S11). « Payé » est réservé à l'échéance (DP-43) |
| Partiel | Partiel | cercle à moitié plein | `--status-partial-*` | « Payé 50 000 FCFA · reste 25 000 FCFA ». Seulement avant J+1 |
| En retard | En retard | triangle d'alerte | `--status-late-*` | « 7 jours » et reste dû. Prime sur « partiel » (DP-13) |
| À venir | À venir | calendrier | `--status-upcoming-*` | Date d'échéance |
| Due | Due | cloche | `--status-due-*` | « Due aujourd'hui » (J0) |
| Avance | Avance | flèche vers le haut dans un cercle | `--status-credit-*` | « Avance 25 000 FCFA » ou « Crédit du bail » (DP-20) |
| Annulé | Annulé | cercle barré | `--status-void-*` | Texte barré. Paiement contre-passé : libellé « Corrigé » avec infobulle |

Autres statuts : style neutre (`--neutral-badge-bg` et `--neutral-badge-fg`, avec icône et libellé). Les couleurs de statut financier sont réservées à l'échéance, au paiement et au relevé « Payé » pour ne pas être confondues avec l'argent :

| Objet | Statuts (libellé) | Style |
|---|---|---|
| Lot | Vacant, Occupé, En travaux, Archivé | Pastille neutre contour plein + icône |
| Bail | Brouillon (contour pointillé), Actif, En préavis (icône horloge), Terminé (texte barré) | Pastille neutre |
| Relevé | Brouillon (contour pointillé), Validé (fond `--color-brand-100`, texte `--color-brand-700`), Payé (reversement enregistré, famille « Payé ») | Neutre, sauf les deux cas cités |
| Quittance | Émise, Annulée (texte barré) | Pastille neutre |
| Usage | Habitation, Commercial, Mixte | Pastille neutre + icône (maison, boutique, deux pièces) |

## Composants

Format : variantes, états, règles. Tous les états interactifs : défaut, survol (ordinateur), focus visible, appui, désactivé.

### Bouton

- **Variantes** : primaire (`--color-brand-600`, texte blanc), secondaire (contour `--color-brand-600`, fond blanc), tertiaire (lien, texte `--color-link`), accent (rare : une mise en avant par écran), icône + libellé. Tailles : normale 44 px, grande 52 px (mobile, action principale).
- **États** : défaut, survol (`--color-brand-700`), focus (anneau 2 px `--color-focus`, décalage 2 px), appui, désactivé (avec la raison en texte sous le bouton), chargement (« Enregistrement… », double clic bloqué).
- **Règles** : un seul primaire par écran ou modale. Libellé = verbe + objet (« Encaisser et envoyer la quittance »). Jamais de bouton d'icône seule pour une action importante. Jamais d'action « Supprimer » sur paiement, quittance, relevé validé.

### Champs et formulaires

- **Variantes** : texte, montant, téléphone, date, liste, choix segmenté, case à cocher, zone de texte, téléversement.
  - Montant : clavier numérique (`inputmode="numeric"`), séparateur de milliers automatique, suffixe « FCFA », entier seulement (R-001, R-037).
  - Téléphone : préfixe « +226 » fixe, saisie en groupes de deux chiffres.
  - Date : saisie JJ/MM/AAAA, bouton calendrier, date du jour par défaut. Une date future est refusée pour un paiement (R-039).
  - Mode de paiement : choix segmenté « Espèces · Mobile money · Virement · Chèque ». « Espèces » par défaut (R-040).
- **États** : défaut, focus, rempli, erreur (icône + message sous le champ), désactivé, lecture seule.
- **Règles** : étiquette toujours visible, jamais le seul « placeholder ». Champ obligatoire noté « (obligatoire) » en texte. Validation à la sortie du champ et à l'envoi. Message d'erreur lié au champ (`aria-describedby`). Valeurs par défaut intelligentes. Les données saisies survivent à une erreur ou une coupure. Une clé d'idempotence est créée à l'ouverture du formulaire de paiement (R-042).

### Tableau responsive

- **Variantes** : tableau complet (≥ 1024 px), tableau à colonnes essentielles (600–1023 px), liste de cartes (< 600 px) avec détail au clic.
- **États** : normal, ligne survolée, ligne sélectionnée, chargement (squelette), vide, erreur.
- **Règles** :
  - Montants alignés à droite, chiffres tabulaires. Total en pied.
  - En-tête collant. Ligne cliquable en entier. Pagination de 25 lignes avec « Afficher plus ».
  - Tri et filtres mémorisés par utilisateur.
  - Sur mobile : trois informations par carte au plus (nom, montant, statut). Le reste s'ouvre au clic.
  - Colonnes essentielles :

| Tableau | Colonnes essentielles (mobile) | Colonnes complètes (ordinateur) |
|---|---|---|
| Impayés (S05) | Locataire, reste dû, ancienneté | + lot, bien, dernière relance, promesse, prochaine action |
| Paiements (S11) | Locataire, montant, statut | + date, lot, mode, référence, reçu par, n° de quittance |
| Locataires (S08) | Nom, solde, statut | + téléphone, lot(s), bail |
| Lots (S06) | Référence, statut du lot, locataire | + bien, usage, loyer, échéance du mois |
| Baux (S10) | Locataire, lot, statut | + dates, loyer, jour d'échéance, alertes |

### Carte locataire et carte lot

- **Locataire** : nom, téléphone « +226 70 12 34 56 », lot(s), solde avec badge, dernière relance. Actions : Encaisser, Relancer, Appeler, WhatsApp. Variantes : compacte (résultat de recherche) et complète (en-tête de fiche). Pièce d'identité masquée par défaut (R-019).
- **Lot** : référence, bien, quartier, usage (badge neutre), statut du lot, locataire actuel, loyer, état de l'échéance du mois. Variantes : compacte (liste) et complète (en-tête de fiche).
- **États** : normal, sélectionné, archivé (texte atténué + libellé « Archivé »), chargement (squelette).
- **Règles** : jamais d'action destructive sur une carte. Les actions respectent les droits du rôle.

### Badge de statut

- Voir « Statuts financiers » pour le détail.
- **États** : défaut, avec précision (« 7 jours »), compact (icône + libellé court).
- **Règles** : icône, libellé et couleur sont toujours ensemble. Largeur minimale pour garder la forme. Libellé jamais tronqué. L'infobulle explique un statut non évident (« Corrigé »).

### Onglets

- **Variantes** : horizontaux. Défilement latéral sur mobile avec bord dégradé visible.
- **Utilisation** : fiche lot (Aperçu, Baux, Paiements, Travaux, Documents). Fiche locataire (Aperçu, Baux, Paiements, Relances, Documents).
- **États** : actif (soulignement `--color-brand-600` + texte gras), inactif, focus, avec compteur.
- **Règles** : six onglets au plus. Libellé en texte. L'onglet actif figure dans l'adresse pour pouvoir être partagé.

### Filtres

- **Variantes** : barre de puces (ordinateur), feuille du bas « Filtrer » (mobile).
- **États** : aucun filtre, filtres actifs (puces avec « × » et compteur), « Effacer tout ».
- **Règles** : filtres de l'écran Impayés : bien, propriétaire, gestionnaire, usage (F11). Mémorisés par utilisateur. Le nombre de résultats est annoncé (`aria-live`).

### Recherche unique

- **Portée** : nom, téléphone, lot, n° de quittance (§14.1 étape 8). Un seul champ, toujours accessible : en haut de page sur ordinateur, icône dans la barre du haut sur mobile, « / » comme raccourci clavier.
- **Comportement** :
  - Suggestions dès 2 caractères, délai de 200 ms.
  - Insensible aux accents et à la casse.
  - Téléphone reconnu avec ou sans « +226 » et avec espaces.
  - Un n° de quittance (ex. « HOR-2026-000094 ») est reconnu par son préfixe.
  - Résultats groupés : Locataires, Lots, Quittances. Huit résultats au plus par groupe.
- **États** : initial (recherches récentes), chargement, résultats, aucun résultat (propose « Créer un locataire »), erreur.
- **Règles** : le résultat ouvre l'action la plus probable. Locataire → carte avec « Encaisser ». Quittance → son détail. Un agent terrain ne trouve que ses lots (R-081).

### Modale de confirmation

- **Variantes** : standard, avec motif obligatoire (zone de texte), feuille du bas (mobile).
- **Usage réservé** aux actions à enjeu :
  - paiement supérieur au seuil (R-038) ;
  - contre-passation ;
  - validation d'un compte rendu ;
  - enregistrement d'un reversement ;
  - réouverture d'un relevé (R-083) ;
  - clôture de caisse ;
  - confirmation d'un import ;
  - désactivation d'un utilisateur.
- **États** : ouverte, en cours (bouton en chargement), erreur.
- **Règles** : titre = la question. Le corps dit la conséquence concrète. Boutons : [Annuler] puis [Verbe]. Pour une action irréversible, le focus initial est sur [Annuler]. La touche Échap ferme. Le focus reste piégé dans la modale. Aucune modale pour encaisser, relancer ou naviguer.

### Toast

- **Variantes** : succès, information, avertissement, erreur.
- **États** : apparu, visible, fermé. Succès et information : 4 secondes. Avertissement et erreur : restent jusqu'à fermeture.
- **Règles** : position en bas au-dessus de la barre du bas (mobile), en haut à droite (ordinateur). Zone `role="status"` (ou `role="alert"` pour une erreur). Un toast ne porte jamais seul une information critique. Pas d'action « Annuler » sur un paiement : l'action proposée est « Voir la quittance ».

### État vide

- **Structure** : icône sobre, titre, une phrase, une action principale.
- **Variantes** : premier usage (propose d'importer ou de créer), aucun résultat (propose d'effacer les filtres), état positif (« Aucun impayé. Bravo »).
- **Règles** : toujours une action proposée. Ton encourageant et sobre. Pas d'illustration lourde.

### Squelette de chargement

- **Variantes** : tableau (5 lignes), carte, ligne de chiffres clés.
- **États** : chargement ; chargement long (au-delà de 10 secondes, le message « La connexion est lente… » et un bouton [Réessayer] apparaissent) ; terminé.
- **Règles** : jamais de rond de chargement plein écran. Pulsation d'opacité uniquement, supprimée si l'utilisateur réduit les animations. Les dimensions du squelette correspondent au contenu final pour éviter les sauts de mise en page.

### État d'erreur

- **Variantes** : erreur de champ ; bandeau de page (« Connexion perdue ») ; page d'erreur (« Introuvable », « Accès refusé », « Un problème est survenu ») ; erreur de génération de quittance (le paiement est enregistré, R-049) ; conflit (référence déjà utilisée).
- **Règles** : dire quoi faire. Jamais de code technique. Bouton [Réessayer] quand cela a un sens. Un code court d'incident est affiché en petit pour le support. Une ressource d'une autre organisation répond « Introuvable » (R-004).

### Composants complémentaires

- **Montant** : « 1 250 000 FCFA », chiffres tabulaires, aligné à droite, signe moins « − 25 000 FCFA » pour une correction. Lecteur d'écran : « un million deux cent cinquante mille francs CFA ».
- **Bandeau d'alerte** : quatre niveaux (retours), icône + texte, fermable seulement s'il n'est pas critique.
- **Pas-à-pas** : création de bail en 3 étapes (S10), import (S16), assistant de démarrage (S02). Étape courante annoncée « Étape 2 sur 3 ».
- **Feuille du bas** : menu « Plus », filtres, relance sur mobile. Poignée de fermeture, bouton [Fermer] visible.
- **Carte de chiffre clé** : valeur (`--text-3xl`), libellé, lien « Voir le détail ».
- **Chronologie** : historique d'un lot, journal des relances. Date à gauche, événement, auteur.
- **Visite guidée du produit** : onboarding de 3 écrans pour le caissier et le gestionnaire (dossier §14.1), lancée depuis S02 et rejouable depuis le menu « Plus ». Ne pas confondre avec la « Démonstration guidée » de la maquette (`MAQUETTE.md`).

## Navigation

- **Ordinateur (≥ 1024 px)** : menu latéral fixe de 248 px. En haut : recherche unique, cloche, menu utilisateur (nom, rôle, organisation, déconnexion). Sélecteur d'organisation si l'utilisateur en a plusieurs (F01).
  - Au quotidien : Accueil, Encaisser, Impayés, Paiements et quittances.
  - Patrimoine : Biens et lots, Locataires, Baux.
  - Gestion : Comptes rendus, Mandants et mandats, Travaux et documents, Agenda.
  - Outils : Import, Clôture de caisse, Paramètres, Utilisateurs, Journal d'audit.
- **Mobile et tablette (< 1024 px)** : barre du haut (titre, recherche, cloche) et barre du bas **toujours à 4 entrées**, dont la dernière est « Plus ». « Plus » ouvre une feuille avec les autres entrées permises au rôle. Les entrées de la barre dépendent du rôle :

| Rôle | Barre du bas (4 entrées) |
|---|---|
| Gérant, caissier, propriétaire direct | Accueil, Encaisser, Impayés, Plus (barre du dossier §14.3) |
| Gestionnaire | Accueil, Encaisser, Impayés, Plus. Si le droit d'encaisser est retiré, « Encaisser » est remplacé par « Locataires » |
| Agent terrain | Accueil, Impayés, Locataires, Plus |
| Comptable (lecture) | Accueil, Impayés, Paiements, Plus |

- **Arrivée par rôle** : caissier → Encaisser. Gestionnaire → « À faire aujourd'hui ». Gérant et propriétaire direct → tableau de bord. Agent terrain → sa tournée (déduit).
- **Droits** : une entrée interdite est masquée. Le serveur applique le droit quoi qu'il arrive (CONTEXTE, matrice de permissions).

Entrées de menu par rôle. É = voir et agir. L = lecture seule. — = absente. Ce tableau dérive de la matrice de permissions de `CONTEXTE.md`, qui reste la source unique. Une entrée que la matrice ne définit pas est **masquée** (DP-66). La colonne « Collaborateur » est celle du plan Propriétaire (compléments de `CONTEXTE.md`, DP-66).

| Entrée | Gérant | Gestionnaire | Caissier | Agent terrain | Comptable | Propriétaire direct (plan Propriétaire) | Collaborateur (plan Propriétaire) |
|---|---|---|---|---|---|---|---|
| Accueil | É | É | É | É (ses lots) | L | É | É |
| Encaisser | É | É (réglable) | É | — | — | É | É (réglable) |
| Impayés | É | É | L | É (ses lots) | L | É | É (relance) |
| Paiements et quittances | É | É | É | — | L | É | É |
| Biens et lots | É | É | L | L (ses lots) | L | É | L |
| Locataires | É | É | L | L (ses lots) | L | É | L |
| Baux | É | É | — | — | — | É | — |
| Comptes rendus | É | L | — | — | L | — | — |
| Mandants et mandats | É | — | — | — | — | — | — |
| Travaux et documents | É | É | — | Proposer (refusé au pilote) | — | É | — |
| Agenda | É | É | — | — | — | É | — |
| Import | É | É (si autorisé) | — | — | — | É | — |
| Clôture de caisse | É | — | É | — | — | É (version simple) | — |
| Notifications | É | É | — | — | — | É | — |
| Paramètres | É | — | — | — | — | É | — |
| Utilisateurs | É | — | — | — | — | É (2 profils) | — |
| Journal d'audit | L | — | — | — | L | L | — |

## Parcours par rôle

Base : tableau du dossier §14.1. Les identifiants S-xx renvoient au catalogue des écrans. Le parcours de l'agent terrain n'existe pas dans le dossier : il est déduit des personas, de la matrice de permissions et du JTBD « Could », et reste à valider.

### Caissier

| Étape | Parcours |
|---|---|
| 1. Arrivée | Connexion par téléphone (S01). Arrive sur « Encaisser » (S04) |
| 2. Onboarding | Visite guidée de 3 écrans |
| 3. Première action | Rechercher un locataire, encaisser |
| 4. Parcours principal | Recherche → montant → mode → valider → quittance (S04) |
| 5. Actions fréquentes | Réimprimer une quittance, voir l'historique d'un locataire (S11, S09) |
| 6. Notifications | Rien d'intrusif |
| 7. Erreurs | Message clair. Correction par demande, puis contre-passation par le gérant |
| 8. Recherche | Recherche unique : nom, téléphone, lot, n° de quittance |
| 9. Reporting | Journal de caisse du jour (S11, S19) |
| 10. Sortie | Clôture de caisse (S19), puis déconnexion |

### Gestionnaire

| Étape | Parcours |
|---|---|
| 1. Arrivée | Arrive sur « À faire aujourd'hui » (S03, variante gestionnaire) |
| 2. Onboarding | Visite guidée + import de données (S16) |
| 3. Première action | Ouvrir le tableau des impayés (S05) |
| 4. Parcours principal | Impayé → relancer → noter une promesse (S05) |
| 5. Actions fréquentes | Ajouter un bail, noter un travail (S10, S14) |
| 6. Notifications | Promesses non tenues, fins de bail (S20) |
| 7. Erreurs | Message clair. Correction par demande |
| 8. Recherche | Recherche unique + filtres |
| 9. Reporting | Liste des impayés, relances faites (S05) |
| 10. Sortie | Déconnexion automatique après inactivité |

### Gérant et propriétaire direct

| Étape | Parcours |
|---|---|
| 1. Arrivée | Tableau de bord : encaissé, attendu, impayés (S03) |
| 2. Onboarding | Assistant : organisation → premier bien → premier bail ou import (S02) |
| 3. Première action | Créer ou importer ses lots (S06, S16) |
| 4. Parcours principal | Vérifier les impayés → clôturer le mois → compte rendu (S05, S12) |
| 5. Actions fréquentes | Valider un reversement, inviter un collaborateur (S12, S18) |
| 6. Notifications | Résumé quotidien, demandes de correction (S20) |
| 7. Erreurs | Message clair. Contre-passation validée par le gérant |
| 8. Recherche | Recherche unique |
| 9. Reporting | Encaissé vs attendu, retard moyen, comptes rendus, exports (S03, S12) |
| 10. Sortie | Déconnexion. Accès support révocable |

Propriétaire direct (plan Propriétaire) : accueil simplifié « Ce mois : X payés sur Y, Z en retard », rappel en un clic (persona B).

### Agent terrain (déduit, à valider)

| Étape | Parcours |
|---|---|
| 1. Arrivée | Connexion par téléphone. Arrive sur la liste de ses lots en impayé (S05, filtrée) |
| 2. Onboarding | Visite guidée de 3 écrans |
| 3. Première action | Ouvrir sa tournée (liste de ses locataires à visiter) |
| 4. Parcours principal | Locataire → appeler ou WhatsApp → noter la visite ou la promesse (S05, S09) |
| 5. Actions fréquentes | Proposer une dépense ou un travail (matrice : « Proposer ») |
| 6. Notifications | Promesses de ses lots |
| 7. Erreurs | Réseau instable : saisie conservée, message « Réessayez quand la connexion revient » |
| 8. Recherche | Recherche unique, limitée à ses lots |
| 9. Reporting | Liste de ses visites et promesses |
| 10. Sortie | Déconnexion automatique après inactivité |

## Catalogue des écrans

Légende : É = voit et agit. L = lecture. Priorité = priorité de la fonctionnalité principale (P0 à P3). Tranche = tranche de livraison (T1, T2, T3), voir DP-29. Dans la maquette, le repère suit la tranche : T1, T2 et T3 → « V1 » (T3 : mention « livré pendant le pilote »), fonction hors tranche → « À venir ». Un élément hors tranche dans un écran V1 porte son propre repère « À venir ».

Écrans au-delà des 12 du dossier §14.2 : S01, S02, S13, S19, S20, S21, S25, S26 (déduits de §14.1, F03, F05, F10, F13, F19, F22) et S22 à S24 (repère « À venir »). S07 (fiche lot), S09 (fiche locataire) et S18 (utilisateurs et invitations) sont des sous-écrans d'écrans du dossier. La première maquette suit DP-31 : S19 à S24 sont reportés après le démarrage du pilote.

### S01 — Connexion, invitation et réinitialisation

- **Objectif** : se connecter, accepter une invitation, réinitialiser son mot de passe.
- **Rôles** : public.
- **Contenu** : quatre modes. (1) Connexion : identifiant (téléphone ou e-mail), mot de passe ; double authentification si elle est activée. (2) Accepter l'invitation : nom affiché, mot de passe, à partir du lien reçu (valable 7 jours). (3) Mot de passe oublié : message « Demandez un lien de réinitialisation à votre gérant ». (4) Choisir un nouveau mot de passe : à partir du lien généré par le gérant ou le support (à usage unique, valable 15 minutes). **Pas d'inscription publique au pilote** : les organisations sont créées par la console (DP-68, écart assumé par rapport au dossier F01) ; l'inscription publique viendra après le pilote.
- **Actions** : se connecter, accepter l'invitation, choisir un nouveau mot de passe.
- **États** : vide (n/a) · chargement (bouton) · erreur (identifiant déjà utilisé → proposer la connexion ; trop de tentatives → blocage temporaire ; invitation expirée après 7 jours) · succès (arrivée sur l'écran du rôle).
- **Fonctionnalités** : F01. **Priorité** : P0. **Tranche** : T1.

### S02 — Assistant de démarrage

- **Objectif** : amener un nouveau client à sa première donnée utile.
- **Rôles** : gérant, propriétaire direct.
- **Contenu** : pas-à-pas : organisation (nom, logo, en-tête de quittance) → premier bien ou import → premier bail → invitations.
- **Actions** : suivant, passer, importer (→ S16).
- **États** : en cours (étape X sur Y) · erreur de champ · succès (accueil avec premier état vide positif).
- **Fonctionnalités** : F01, F14. **Priorité** : P0. **Tranche** : T1 (import : T2). Lance la visite guidée du produit (3 écrans).

### S03 — Accueil et tableau de bord

- **Objectif** : voir en 5 secondes l'état des encaissements et les alertes.
- **Rôles** : tous, contenu adapté.
- **Contenu** :
  - Gérant et propriétaire direct : encaissé ce mois, attendu, taux de recouvrement [formule à confirmer], impayés par ancienneté (1–7, 8–30, 31–59, 60 et +), alertes du jour.
  - Gestionnaire, « À faire aujourd'hui » (déduit de F12, F17, F22) : promesses du jour et promesses non tenues (indicateur de T1, DP-36), relances planifiées, bloc « Rappels à envoyer » (échéances à J−3, R-097, DP-40) qui ouvre le panneau « Relancer » de S05, échéances dues aujourd'hui, fins de bail à 60 jours (T3).
  - Caissier, « Aujourd'hui » : encaissements du jour (montant, nombre, par mode), dernier paiement saisi, bouton « Encaisser ». L'arrivée du caissier reste S04.
  - Agent terrain, « Ma tournée » : liste de ses lots en retard avec reste dû, promesses de ses lots, boutons Appeler et WhatsApp (déduit).
  - Comptable (lecture) : encaissé, attendu et impayés en lecture seule.
  - Plan Propriétaire : « X payés sur Y, Z en retard », rappel en un clic.
- **Actions** : ouvrir les impayés, encaisser, relancer.
- **États** : vide (premier usage : « Importez vos baux pour commencer ») · chargement (squelette des chiffres) · erreur (bandeau + Réessayer) · succès.
- **Fonctionnalités** : F11, F09, F12, F17. **Priorité** : P0. **Tranche** : T1 (fins de bail : T3).

### S04 — Encaisser

- **Objectif** : enregistrer un paiement et remettre la quittance en moins de 30 secondes.
- **Rôles** : gérant É · gestionnaire É (réglable) · caissier É · propriétaire direct É.
- **Contenu** (§14.2) : recherche rapide · carte du locataire avec solde · montant · date (aujourd'hui par défaut) · mode (« Espèces » par défaut) · référence (mobile money, virement, chèque) · « payé par » (locataire du bail ou tiers, R-088) · note · aperçu de l'affectation (échéances les plus anciennes d'abord, modifiable, R-043, R-087) · bouton « Encaisser et envoyer la quittance ». Zone repliable « Coller le SMS mobile money » (F25).
- **Garde-fous** : référence déjà utilisée (R-041) · date future refusée (R-039) · surplus proposé en avance (R-046) · confirmation au-delà du seuil (R-038) · avertissement si la date de paiement tombe dans une journée de caisse clôturée (R-089).
- **Actions** : encaisser, envoyer sur WhatsApp, imprimer (A4/A5), copier le lien, « Encaisser un autre loyer ».
- **États** : vide (aucun locataire choisi : récents) · chargement (bouton) · erreur (réseau : saisie conservée ; PDF en échec : paiement enregistré, quittance en file, R-049) · succès (aperçu de la quittance avec son n°, S25).
- **Fonctionnalités** : F09, F10, F25. **Priorité** : P0 (F25 : P1). **Tranche** : T1 (zone « Coller le SMS », F25 : T3). **Mobile** : une colonne, clavier numérique, bouton primaire collé en bas de l'écran.

### S05 — Impayés

- **Objectif** : voir ce qui est dû, classé par urgence, et relancer.
- **Rôles** : gérant É · gestionnaire É · caissier L · agent terrain É (ses lots) · comptable L.
- **Contenu** : total en retard · répartition par ancienneté · tableau filtrable (bien, mandant, gestionnaire, usage) · colonnes : locataire, lot, ancienneté, reste dû, dernière relance, promesse, prochaine action · panneau « Relancer » (modèle courtois, ferme, dernier avis ; aperçu du texte ; canal WhatsApp ou SMS ; confirmation d'envoi ; résultat ; promesse date + montant ; prochaine action).
- **Règles d'affichage** : une promesse non échue ne masque pas la ligne (R-059). Avertissement si une relance a déjà été faite aujourd'hui (R-063).
- **Actions** : relancer, encaisser, filtrer, trier, exporter.
- **États** : vide positif (« Aucun impayé. Bravo ») · chargement · erreur · succès (relance enregistrée).
- **Fonctionnalités** : F11, F12. **Priorité** : P0. **Tranche** : T1. L'envoi automatique par WhatsApp Business (F27) n'est pas montré (DP-30).

### S06 — Biens et lots

- **Objectif** : gérer le patrimoine.
- **Rôles** : gérant É · gestionnaire É · caissier L · agent terrain L (ses lots) · comptable L.
- **Contenu** : liste des biens (nom, adresse, quartier, propriétaire, nombre de lots, occupation) · fiche bien (lots, statuts, travaux, documents) · création bien + lots (les champs dépendent du type de lot ; une maison seule crée 1 bien + 1 lot).
- **Actions** : créer, modifier, archiver, filtrer.
- **États** : vide (« Créez votre premier bien ou importez votre portefeuille ») · chargement · erreur (référence de lot déjà utilisée) · succès.
- **Fonctionnalités** : F04. **Priorité** : P0. **Tranche** : T1.

### S07 — Fiche lot

- **Objectif** : tout savoir sur un lot et son historique.
- **Rôles** : comme S06.
- **Contenu** : en-tête (référence, bien, usage, type, statut) · onglets : Aperçu (bail actif, loyer, échéance du mois, solde), Baux, Paiements, Travaux (F15), Documents (F16).
- **Actions** : créer un bail, mettre en travaux (lot vacant seulement, R-093), ajouter une intervention, archiver (refusé si bail actif, R-006).
- **États** : vide par onglet · chargement · erreur · succès.
- **Fonctionnalités** : F04, F07, F15, F16. **Priorité** : P0 (onglets Travaux et Documents : P1). **Tranche** : T1 (onglets Travaux et Documents : T3).

### S08 — Locataires

- **Objectif** : retrouver un locataire, créer une fiche.
- **Rôles** : gérant É · gestionnaire É · caissier L · agent terrain L (ses lots) · comptable L.
- **Contenu** : liste (nom, téléphone, lot, solde) · création (type personne ou société, téléphones dont WhatsApp, e-mail, contact de secours ; pièce d'identité : champ désactivé au pilote, `ARCHITECTURE.md` §13.2).
- **Actions** : créer, fusionner les doublons, archiver.
- **États** : vide · chargement · erreur (numéro invalide, doublon probable) · succès.
- **Fonctionnalités** : F06. **Priorité** : P0. **Tranche** : T1.

### S09 — Fiche locataire

- **Objectif** : voir l'historique complet d'un locataire.
- **Rôles** : comme S08.
- **Contenu** : coordonnées · baux · paiements · relances · documents · solde. Carte avec Encaisser, Relancer, Appeler, WhatsApp. Pièce d'identité masquée.
- **Actions** : encaisser, relancer, modifier, fusionner.
- **États** : vide par onglet · chargement · erreur · succès.
- **Fonctionnalités** : F06, F09, F12. **Priorité** : P0. **Tranche** : T1. Élément « Espace locataire » (F29) : repère « À venir ».

### S10 — Baux

- **Objectif** : créer et suivre les baux, leur échéancier, leur fin.
- **Rôles** : gérant É · gestionnaire É · propriétaire direct É · autres : à confirmer.
- **Contenu** : liste (échéances et alertes) · création en 3 étapes (lot et locataires → dates, loyer, charges, jour d'échéance, dépôt de garantie, type → vérification et pièce jointe, activation) · fiche bail : statuts (brouillon, actif, en préavis, terminé), échéancier avec badges, avenants, préavis et fin de bail, dépôt de garantie (retenues, restitution).
- **Garde-fous** : chevauchement refusé (R-020) · lot en travaux refusé (R-093) · dépôt supérieur à deux mois : avertissement non bloquant (R-024).
- **Actions** : créer, activer, ajouter un avenant, renouveler, résilier, imprimer une fiche de bail.
- **États** : vide · chargement · erreur · succès.
- **Fonctionnalités** : F07, F08, F20, F23. **Priorité** : P0 (F20, F23 : P1). **Tranche** : T1 (F20 et F23 : T3). Élément « Indexation automatique » (F30) : repère « À venir ».

### S11 — Paiements et quittances

- **Objectif** : retrouver un paiement, une quittance, traiter les corrections.
- **Rôles** : gérant É · gestionnaire É · caissier É · comptable L · propriétaire direct É.
- **Contenu** : journal des paiements (date, locataire, lot, montant, mode, référence, reçu par, n° de quittance, statut) · détail d'un paiement (affectations, historique) · file des demandes de correction (gérant) · réimpression.
- **Actions** : réimprimer, envoyer, demander une correction, contre-passer (gérant), émettre une quittance rectificative.
- **États** : vide (« Aucun paiement sur cette période ») · chargement · erreur · succès.
- **Fonctionnalités** : F09, F10, F18. **Priorité** : P0 (exports : P1). **Tranche** : T1 (exports : T3). Le lien de paiement en ligne (F26) n'est pas montré (DP-30). Le détail d'un paiement ouvre l'aperçu de sa quittance (S25).

### S12 — Comptes rendus propriétaires

- **Objectif** : produire, valider et partager le compte rendu mensuel d'un mandant.
- **Rôles** : gérant É · gestionnaire L · comptable L. Plan Agence.
- **Contenu** : liste des comptes rendus (mandant, période, statut) · production : mandant + période → loyers encaissés (liste), commission (taux, base), dépenses imputées, net à reverser, impayés · ajustement (ajouter une dépense) · validation · PDF · lien de partage · reversement (date, mode, référence) · réouverture avec motif tant qu'aucun reversement (R-083).
- **Garde-fous** : un lot sans mandat bloque la production (R-075). Dépense sans pièce : avertissement.
- **Actions** : prévisualiser, valider, exporter, enregistrer un reversement, rouvrir.
- **États** : vide · chargement · erreur · succès.
- **Fonctionnalités** : F13, F05, F15 (minimal). **Priorité** : P0. **Tranche** : T2. Élément « Portail mandant » (F28) : repère « À venir ». Le lien de partage ouvre la page publique S26.

### S13 — Mandants et mandats

- **Objectif** : savoir pour qui on gère et à quelles conditions.
- **Rôles** : gérant É. Plan Agence.
- **Contenu** : liste des mandants · fiche (contacts, mode de reversement, mandats : taux, base, dates, lots couverts).
- **Actions** : créer, avenant de taux, clôturer un mandat, transférer des lots.
- **États** : vide · chargement · erreur (chevauchement, taux hors plage) · succès.
- **Fonctionnalités** : F05. **Priorité** : P0. **Tranche** : T2.

### S14 — Travaux et documents

- **Objectif** : garder la mémoire du bien et les pièces d'un dossier.
- **Rôles** : gérant É · gestionnaire É · agent terrain (proposer) · propriétaire direct É.
- **Contenu** : journal d'interventions (date, nature, description, coût, prestataire, payeur, statut, imputable au relevé, pièces jointes) · documents par bien, lot, locataire, bail · prestataires.
- **Actions** : ajouter une intervention, joindre un document, filtrer.
- **États** : vide · chargement · erreur · succès.
- **Fonctionnalités** : F15, F16. **Priorité** : P1. **Tranche** : T2 (dépense imputable, version minimale de F15) ; journal complet et documents : T3. État des lieux complet (F24) : repère « À venir ».

### S15 — Agenda

- **Objectif** : ne rater aucune échéance non financière.
- **Rôles** : gérant É · gestionnaire É · propriétaire direct É.
- **Contenu** : tâches et alertes triées par date : fin de bail à 60 jours, promesses non tenues, rappels de visite, « révision à étudier ».
- **Actions** : ajouter une tâche, assigner, marquer comme faite.
- **États** : vide (« Rien de prévu ») · chargement · erreur · succès.
- **Fonctionnalités** : F17. **Priorité** : P1. **Tranche** : T3.

### S16 — Import

- **Objectif** : faire entrer un portefeuille existant sans ressaisie.
- **Rôles** : gérant É · gestionnaire É (si autorisé) · propriétaire direct É.
- **Contenu** : pas-à-pas : téléverser (modèle Excel Immo Suite ou fichier du client) → associer les colonnes (propositions automatiques) → aperçu (valides, avertissements, erreurs avec n° de ligne, doublons à fusionner) → confirmer → rapport téléchargeable. Historique des imports.
- **Actions** : téléverser, associer, corriger, confirmer, annuler (tant que rien n'est utilisé, R-080).
- **États** : vide · chargement (analyse) · erreur (format invalide : colonnes attendues) · succès (rapport).
- **Fonctionnalités** : F14. **Priorité** : P0. **Tranche** : T2.

### S17 — Paramètres

- **Objectif** : configurer l'organisation.
- **Rôles** : gérant É · propriétaire direct É.
- **Contenu** : organisation (nom, ville, logo, en-tête de quittance, identifiants légaux facultatifs, signature ou cachet, préfixe de numérotation, durée d'expiration des liens) · modèles de messages (étapes J−3, J+1, J+5, J+10 ; variables) · droits réglables · plan (changement par HABEK) · sécurité (double authentification du gérant : obligatoire avant `live`, DP-68) · exports (F18) · **accès support** : autoriser l'équipe HABEK pour une durée choisie, révoquer à tout moment, historique des accès (consentement du client, dossier §12.1, R-009).
- **Actions** : modifier, tester un modèle, exporter, autoriser ou révoquer l'accès support.
- **États** : chargement · erreur de champ · succès (toast).
- **Fonctionnalités** : F01, F10, F12, F02, F18, F40, F03. **Priorité** : P0 (exports : P1). **Tranche** : T1 (exports : T3 ; accès support : T2 avec F40 ; double authentification du gérant : T1, obligatoire avant `live`, DP-68). Élément « Abonnement en ligne » (F31) : repère « À venir ».

### S18 — Utilisateurs et invitations (console d'invitation)

- **Objectif** : donner l'accès à son équipe, au bon niveau.
- **Rôles** : gérant É · propriétaire direct É.
- **Contenu** : liste (nom, identifiant, rôle, statut : actif, invité, expiré) · formulaire d'invitation (téléphone ou e-mail, rôle, lots assignés pour un agent terrain) · droits réglables.
- **Actions** : inviter, renvoyer (invitation valable 7 jours), changer le rôle (effet immédiat), désactiver. Le dernier gérant actif ne peut pas être retiré (R-081).
- **États** : vide (« Invitez votre premier collaborateur ») · chargement · erreur (identifiant déjà utilisé, invitation expirée) · succès.
- **Fonctionnalités** : F01, F02. **Priorité** : P0. **Tranche** : T1.

### S19 — Clôture de caisse

- **Objectif** : comparer les espèces attendues et comptées, verrouiller la journée.
- **Rôles** : caissier É · gérant É. Version simple dans le plan Propriétaire.
- **Contenu** : paiements en espèces du jour · espèces attendues · espèces comptées (saisie) · écart · motif si écart · historique des journées.
- **Actions** : clôturer (modale), consulter une journée.
- **États** : vide (« Aucun encaissement en espèces aujourd'hui ») · chargement · erreur · succès (journée verrouillée).
- **Fonctionnalités** : F19. **Priorité** : P1. **Tranche** : T3 (DP-29).

### S20 — Notifications

- **Objectif** : savoir quoi faire à l'ouverture.
- **Rôles** : gérant, gestionnaire (autres : à confirmer).
- **Contenu** : cloche + liste : promesse non tenue, demande de correction, compte rendu prêt, bail à échéance dans 60 jours, quittance en échec de génération. Résumé quotidien des impayés (gérant).
- **Actions** : ouvrir l'objet lié, marquer comme lu.
- **États** : vide (« Aucune notification ») · chargement · erreur · succès.
- **Fonctionnalités** : F22. **Priorité** : P1. **Tranche** : T3.

### S21 — Journal d'audit

- **Objectif** : retrouver qui a fait quoi, quand.
- **Rôles** : gérant L · comptable L · propriétaire L.
- **Contenu** : filtres (utilisateur, objet, période) · lignes : qui, quoi, objet, avant/après, quand, adresse IP. Lecture seule, aucune modification possible (R-007).
- **Actions** : filtrer, rechercher, exporter (gérant).
- **États** : vide · chargement · erreur · succès.
- **Fonctionnalités** : F03. **Priorité** : P0. **Tranche** : T1.

### S25 — Quittance (aperçu, détail, impression)

- **Objectif** : montrer, imprimer et partager une quittance, après un paiement ou depuis le journal.
- **Rôles** : gérant É · gestionnaire É · caissier É · comptable L · propriétaire direct É.
- **Contenu** : la quittance mise en page selon « Quittance : mise en page imprimable » (A4 ou A5) · son numéro, son statut (émise, annulée) · historique des envois (R-055) · lien vers le paiement d'origine, vers la quittance qu'elle remplace ou qui la remplace (R-051, R-096).
- **Actions** : envoyer sur WhatsApp, imprimer (A4/A5), copier le lien, réimprimer, émettre une quittance rectificative.
- **États** : chargement (squelette de la page) · erreur (PDF non prêt : « un nouvel essai est en cours », R-049) · succès.
- **Fonctionnalités** : F10. **Priorité** : P0. **Tranche** : T1.

### S26 — Page de partage publique et lien expiré

- **Objectif** : ce que voit le destinataire (locataire, mandant) qui ouvre un lien de partage.
- **Rôles** : destinataire du lien, sans compte.
- **Contenu** : le document seul (quittance ou compte rendu), sans menu ni autre donnée de l'organisation (R-054). Si le lien a expiré : « Ce lien a expiré. Demandez un nouveau lien à {organisation}. »
- **Actions** : télécharger ou imprimer le document.
- **États** : chargement · lien expiré (accès refusé, E-12) · lien invalide (« Introuvable ») · succès.
- **Fonctionnalités** : F10, F13. **Priorité** : P0. **Tranche** : T1 (compte rendu : T2).

### Écrans « À venir » (repère obligatoire dans la maquette ; reportés après le démarrage du pilote, DP-31)

| ID | Écran | Contenu | Fonctionnalité | Priorité |
|---|---|---|---|---|
| S22 | Rapports avancés | Rendement, vacance, prévisionnel | F32 | P2 |
| S23 | Portail mandant | Lecture de son compte rendu en ligne | F28 | P2 |
| S24 | État des lieux | Checklist, photos, signature | F24 | P2 |

Hors catalogue : la console HABEK (F40) est un outil interne de l'équipe HABEK, absent de la maquette. Au pilote, elle peut être une ligne de commande (`API_CONTRACTS.md` §7.18) ; ses écrans seront spécifiés dans `specs/F40-console.md` avant la tranche 6.

## Conventions

### Formats

| Élément | Format | Exemple |
|---|---|---|
| Montant | Entier, espace insécable comme séparateur de milliers, « FCFA » en toutes lettres | 1 250 000 FCFA |
| Montant négatif | Vrai signe moins | − 25 000 FCFA |
| Pourcentage | Virgule décimale, espace avant « % » | 7,5 % |
| Date | JJ/MM/AAAA | 13/11/2026 |
| Date longue (titres) | Jour en toutes lettres | vendredi 13 novembre 2026 |
| Heure | 24 h, fuseau Africa/Ouagadougou (R-002) | 09:15 |
| Téléphone | +226 puis 4 groupes de 2 chiffres | +226 70 12 34 56 |
| Durée | Chiffres + unité accordée | 1 jour, 7 jours |
| N° de quittance | Préfixe configurable + année sur 4 chiffres + séquence sur 6 chiffres (R-050, DP-37) | HOR-2026-000094 |

Jamais d'abréviation d'un montant (« 1,25 M »). Jamais de décimale sur un montant (R-001).

### Voix et ton (vouvoiement provisoire)

- Vouvoiement. Phrases courtes. Verbes d'action à l'infinitif sur les boutons.
- Sobre et non agressif. Pas de point d'exclamation d'alerte, pas de majuscules criardes, pas de culpabilisation.
- Les erreurs expliquent la solution. Elles ne désignent pas l'utilisateur comme fautif.
- Les relances restent courtoises. Elles s'adaptent à l'étape, et le ton se durcit par la précision, pas par l'agressivité. Aucun texte ne menace.

### Vocabulaire

| À l'écran | Terme du domaine | Remarque |
|---|---|---|
| Impayé | Échéance en retard | |
| Reste dû | Solde | |
| Avance | Avance, crédit du bail | « Crédit du bail » seulement si aucune échéance n'existe encore |
| Correction | Contre-passation | Libellé « Corrigé » sur un paiement contre-passé |
| Compte rendu propriétaire | Relevé propriétaire | |
| Caution | Dépôt de garantie | Libellé principal : « Dépôt de garantie ». Le mot « caution » est recherchable |
| Tournée | Liste de lots de l'agent terrain | |
| Compte rendu (écran, texte) | Relevé propriétaire (donnée) | « Compte rendu » partout à l'écran (DP-43) |
| Propriétaire direct | Propriétaire direct (rôle, plan) | Le bénéficiaire d'un compte rendu s'appelle « mandant » |
| Mandant | Mandant | Titre de S13 : « Mandants et mandats » |
| Journal de caisse | Journée de caisse | Écran S19 |
| Partiel | Échéance partielle | Libellé court du badge |
| Validé, Corrigé | Paiement validé, contre-passé | « Payé » est réservé à l'échéance |

Mots interdits à l'écran : identifiant technique, « null », « erreur 500 », « token », « supprimer » appliqué à un paiement ou une quittance.

### Modèles de relance (squelette)

Variables autorisées (R-064, `DOMAINE.md`) : `{nom}`, `{montant}`, `{mois}`, `{lot}`, `{date}`, `{lien_quittance}`. Étapes du domaine : J−3, J+1, J+5, J+10 (R-060).

| Étape | Ton | Contenu attendu |
|---|---|---|
| J−3 | Courtois (rappel) | Rappel de l'échéance à venir, montant, date |
| J+1 | Courtois | Constat du retard, montant, demande d'envoyer la référence si déjà payé |
| J+5 | Ferme | Retard précisé, invitation à régulariser ou à proposer une date |
| J+10 | Dernier avis | Récapitulatif du montant dû, invitation à contacter l'agence |

La correspondance étape → ton est une proposition (voir Questions ouvertes).

## Quittance : mise en page imprimable

Objet central du produit. Mentions de R-052 et R-053 [À VÉRIFIER AVEC UN JURISTE] (DP-62). Format par défaut : A5 (A4 disponible) ; les deux formats sont rendus à chaque paiement (DP-64). Le moteur de génération PDF n'est pas encore choisi (décision ouverte n° 7) : cette spécification est indépendante du moteur.

### Contenu, de haut en bas

1. **En-tête** : logo (40 × 20 mm au plus), nom de l'agence ou du propriétaire, adresse, téléphone, identifiants légaux si renseignés.
2. **Titre et numéro** : « QUITTANCE DE LOYER », « N° HOR-2026-000094 » en gras, date d'émission.
3. **Parties** : « Reçu de » (locataire ou locataires du bail ; si un tiers a payé : « Payé par : … », R-088) · « Pour le lot » (référence, bien, quartier, usage).
4. **Paiement** : période (« Loyer de novembre 2026 », échéance du …), date du paiement, mode, référence externe si applicable.
5. **Ventilation** : lignes loyer, charges, arriéré ou autre, avec le montant imputé. Ligne « Total reçu » en chiffres : « 75 000 FCFA ».
6. **Montant en lettres** : « Soixante-quinze mille francs CFA » [formulation de la devise à valider].
7. **Situation** : « Solde restant dû après ce paiement » (« 0 FCFA, échéance soldée » ou le reste). Si avance : « Avance de … FCFA, déduite de l'échéance du … ».
8. **Signature et cachet** : image configurée dans Paramètres, « Reçu par : nom, fonction ».
9. **Pied** : mention « TVA incluse » pour un bail commercial qui l'indique (R-092) · mentions légales complémentaires [à valider] · « Document généré par Immo Suite ».

Variantes :

- **Quittance rectificative** : bandeau « Quittance rectificative — annule et remplace la quittance n° … » (R-051).
- **Quittance annulée** (copie réimprimée) : filigrane « ANNULÉE — remplacée par la quittance n° … ».
- **Série d'essai** (organisation en phase `setup`, numéro `ESSAI-…`) : filigrane « ESSAI » (DP-61).
- **Organisation de démonstration** : filigrane « DÉMONSTRATION — sans valeur » (`SECURITY.md` §7.5).

### Format

| Élément | A4 (210 × 297 mm) | A5 (148 × 210 mm) |
|---|---|---|
| Orientation | Portrait | Portrait |
| Marges | 15 mm | 10 mm |
| Corps | 11 pt | 10,5 pt minimum |
| Montant total | 14 pt gras | 13 pt gras |
| Titre | 16 pt gras | 14 pt gras |
| Pages | 1 | 1 |

- Une seule page. Si un contenu déborde, il est tronqué proprement (nom sur deux lignes au plus) et jamais reporté sur une seconde page.
- Lisible en noir et blanc : les filets bleu nuit `#142B4A` restent foncés et lisibles ; le turquoise `#159D91` devient un gris moyen, donc il ne porte jamais seul une information. Logo fourni en version monochrome.
- Aucun fond coloré plein. Aucun texte clair sur fond foncé.
- Police : système pour la maquette. La police embarquée dans le PDF réel suit le moteur choisi.

### Cas de test de mise en page (§19 : 20 cas types)

- Accents et cédilles : « Ouédraogo », « Yaméogo ».
- Noms longs et composés (deux lignes au plus).
- Société avec raison sociale longue.
- Adresse longue.
- Montants à 7 chiffres en lettres : « un million deux cent cinquante mille ».
- Paiement partiel, avance, tiers payeur, colocation (deux locataires), bail commercial « TVA incluse », rectificative, annulée.

## Microcopy

Placeholders entre accolades. Vouvoiement provisoire. Les valeurs entre crochets sont des exemples.

### Erreurs de saisie

| Contexte | Message | Action proposée |
|---|---|---|
| Montant vide ou nul | « Saisissez un montant supérieur à 0. » | Focus sur le champ |
| Date future | « La date du paiement ne peut pas être dans le futur. Choisissez aujourd'hui ou une date passée. » | |
| Référence déjà utilisée (R-041) | « Ce paiement semble déjà enregistré (quittance n° {n°}). Vérifiez la référence avant de continuer. » | [Voir le paiement] |
| Surplus (R-046) | « Le montant dépasse le solde de {25 000 FCFA}. Enregistrer le surplus comme avance ? Il sera déduit de l'échéance suivante. » | [Enregistrer l'avance] [Corriger le montant] |
| Téléphone invalide | « Numéro invalide. Format attendu : +226 suivi du numéro (exemple : +226 70 12 34 56). » | |
| Doublon de locataire (R-017) | « Un locataire avec ce numéro existe déjà : {nom}. » | [Ouvrir la fiche] [Fusionner] |
| Bail qui chevauche (R-020) | « Ce lot a déjà un bail actif du {date} au {date}. Terminez-le ou choisissez d'autres dates. » | |
| Lot en travaux (R-093) | « Ce lot est en travaux. Remettez-le à « vacant » pour activer un bail. » | [Retirer « en travaux »] |
| Archivage refusé (R-006) | « Ce lot a un bail actif. Terminez le bail avant d'archiver le lot. » | |
| Loyer nul (R-022) | « Le loyer doit être supérieur à 0 FCFA. » | |
| Variable inconnue (R-064) | « La variable {variable} n'existe pas. Variables disponibles : {nom}, {montant}, {mois}, {lot}, {date}, {lien_quittance}. » | |

### Avertissements non bloquants

| Contexte | Message |
|---|---|
| Dépôt de garantie > 2 mois (R-024) | « Le dépôt de garantie dépasse deux mois de loyer. Vérifiez qu'il respecte la loi sur le bail d'habitation. Vous pouvez continuer. » |
| Relance déjà faite aujourd'hui (R-063) | « Une relance a déjà été envoyée à {nom} aujourd'hui à {heure}. Voulez-vous en envoyer une autre ? » |
| Dépense sans pièce | « Aucune pièce jointe. Vous pouvez enregistrer, mais le propriétaire pourra la demander. » |
| Date dans une journée clôturée (R-089) | « La journée du {date} est clôturée. Ce paiement sera compté dans la caisse d'aujourd'hui, avec la date du {date}. » |

### Droits, réseau, système

| Contexte | Message | Action |
|---|---|---|
| Droit refusé | « Vous n'avez pas le droit de {action}. Rôle requis : {rôle}. » | [Demander une correction] si pertinent |
| Paiement validé modifié (R-047) | « Un paiement validé ne peut pas être modifié. Vous pouvez demander une correction au gérant. » | [Demander une correction] |
| Connexion perdue | « Connexion perdue. Vos informations sont conservées. Réessayez dès que le réseau revient. Si le paiement a déjà été enregistré, il ne sera pas créé deux fois. » | [Réessayer] |
| Quittance en échec (R-049) | « Le paiement est enregistré. La quittance n'est pas encore prête : un nouvel essai est en cours. » | [Actualiser] |
| Ressource introuvable (R-004) | « Introuvable. Cette page n'existe pas ou vous n'y avez pas accès. » | [Retour à l'accueil] |
| Connexion lente | « La connexion est lente… » | [Réessayer] |
| Dernier gérant (R-081) | « Vous ne pouvez pas retirer le dernier gérant. Nommez d'abord un autre gérant. » | |
| Identifiant déjà utilisé | « Cet identifiant a déjà un compte. Voulez-vous vous connecter ? » | [Se connecter] |
| Trop de tentatives | « Trop de tentatives. Réessayez dans {durée} ou réinitialisez votre mot de passe. » | [Réinitialiser] |
| Invitation expirée | « Cette invitation a expiré (validité : 7 jours). Demandez au gérant de la renvoyer. » | |
| Lien de réinitialisation | « Votre gérant vous a transmis un lien de réinitialisation. Il est valable 15 minutes. » | |
| Compte rendu bloqué (R-075) | « Impossible de produire le compte rendu : {n} lots n'ont pas de mandat ({liste}). » | [Attribuer un mandat] |
| Import avec erreurs (E-20) | « {197} lignes valides, {3} erreurs. Corrigez les erreurs dans votre fichier (lignes {liste}) ou continuez sans ces lignes. » | [Télécharger le rapport] |

### Confirmations (toasts)

- « Paiement de {75 000 FCFA} enregistré. Quittance n° {n°} prête. » [Envoyer sur WhatsApp] [Imprimer]
- « Relance enregistrée. Prochaine action : {date}. »
- « Promesse de paiement notée pour le {date}. »
- « Compte rendu de {mandant} validé. »
- « Reversement de {338 000 FCFA} enregistré. »
- « Import terminé : {197} lignes créées. »
- « Demande de correction envoyée au gérant. »

### Modales de confirmation

| Action | Titre | Corps | Boutons |
|---|---|---|---|
| Paiement > seuil (R-038) | Confirmer un paiement de {1 250 000 FCFA} ? | Ce montant est élevé. Vérifiez le montant et le mode avant de valider. | [Corriger] [Confirmer le paiement] |
| Contre-passation | Corriger le paiement n° {n°} ? | Une écriture inverse de {75 000 FCFA} sera créée. Le paiement d'origine reste visible. Motif obligatoire. | [Annuler] [Corriger le paiement] |
| Validation d'un compte rendu | Valider le compte rendu de {mandant} pour {octobre 2026} ? | Il sera figé. Net à reverser : {338 000 FCFA}. Vous pourrez le rouvrir tant qu'aucun reversement n'est enregistré. | [Annuler] [Valider] |
| Reversement (DP-17) | Enregistrer le reversement de {338 000 FCFA} à {mandant} ? | Cette action est définitive : le compte rendu ne pourra plus être rouvert. | [Annuler] [Enregistrer le reversement] |
| Réouverture (R-083) | Rouvrir le compte rendu ? | Une nouvelle version sera créée. Indiquez le motif. | [Annuler] [Rouvrir] |
| Clôture de caisse | Clôturer la journée du {date} ? | Attendu : {…} · Compté : {…} · Écart : {…}. Après la clôture, la journée est verrouillée. | [Annuler] [Clôturer la journée] |
| Désactiver un utilisateur | Désactiver {nom} ? | Cette personne ne pourra plus se connecter. | [Annuler] [Désactiver] |

### États vides

| Écran | Message | Action |
|---|---|---|
| Impayés (aucun) | « Aucun impayé. Bravo, tout est à jour. » | [Voir l'historique des paiements] |
| Impayés (premier usage) | « Aucun impayé pour le moment. Importez vos baux pour commencer. » | [Importer mes baux] |
| Recherche sans résultat | « Aucun résultat pour « {terme} ». Vérifiez l'orthographe ou cherchez par numéro de téléphone. » | [Créer un locataire] |
| Biens et lots | « Aucun bien pour le moment. Créez votre premier bien ou importez votre portefeuille. » | [Créer un bien] [Importer] |
| Travaux d'un lot | « Aucune intervention enregistrée sur ce lot. Ajoutez la première pour garder l'historique du bien. » | [Ajouter une intervention] |
| Documents | « Aucun document. Joignez le bail signé pour retrouver le dossier en cas de litige. » | [Joindre un document] |
| Agenda | « Rien de prévu. Les alertes de fin de bail apparaîtront ici. » | |
| Notifications | « Aucune notification. » | |
| Encaissements du jour | « Aucun encaissement aujourd'hui. Recherchez un locataire pour encaisser. » | [Encaisser] |
| Utilisateurs | « Vous êtes seul(e) pour le moment. Invitez votre premier collaborateur. » | [Inviter] |

## Accessibilité

Cible : WCAG 2.1 niveau AA.

- Contrastes de texte ≥ 4,5:1 et contrastes de composants ≥ 3:1. Texte principal ≥ 7:1 (soleil).
- Couleur jamais seule : icône et texte sur chaque statut, erreur et retour.
- Cibles tactiles ≥ 44 × 44 px. Espacement de 8 px entre cibles voisines.
- Tout est utilisable au clavier. Ordre de tabulation logique. Anneau de focus visible. Aucun piège, sauf dans une modale qui piège volontairement le focus.
- `lang="fr"`. Titres hiérarchisés. Un seul `h1` par écran. Repères de page (`header`, `nav`, `main`).
- Formulaires : étiquette liée à chaque champ, erreurs liées au champ, résumé des erreurs en tête de formulaire.
- Toasts et résultats dynamiques annoncés (`aria-live`).
- Montants lus en toutes lettres par les lecteurs d'écran (« un million deux cent cinquante mille francs CFA »).
- Zoom jusqu'à 200 % sans perte de contenu ni défilement horizontal de la page.
- `prefers-reduced-motion` respecté.
- Tableaux sémantiques sur ordinateur. Listes sur mobile.
- Plein soleil : texte principal quasi noir sur fond clair, pas de gris clair sous 4,5:1, taille ≥ 14 px pour tout texte utile.

## Performance

Budgets proposés [à valider par le prototype] :

| Mesure | Budget |
|---|---|
| Premier chargement, HTML + CSS + JS | ≤ 200 Ko compressés |
| CSS | ≤ 30 Ko compressés |
| JavaScript | ≤ 150 Ko compressés |
| Images d'une page | ≤ 100 Ko. Logo ≤ 30 Ko |
| Icônes | Une seule feuille SVG ≤ 15 Ko |
| Polices web | 0 (polices système) |
| Plus grand contenu affiché (LCP) | ≤ 3 s en « 4G lente », processeur ralenti 4× |
| Lignes affichées d'un tableau | 25 par page |
| Résultats de recherche | 8 par groupe |

Autres règles :

- Pas d'animation de plus de 200 ms. Aucune animation décorative.
- Reprise sur coupure : saisies conservées dans l'écran, clé d'idempotence réutilisée au nouvel essai.
- Pas de défilement infini. Pas de carte, pas de vidéo.
- Les budgets de la maquette sont dans `MAQUETTE.md`.

## Responsive

| Point de rupture | Largeur | Référence de test | Navigation | Tableaux | Colonnes de mise en page |
|---|---|---|---|---|---|
| Mobile | 360 à 599 px | 390 px | Barre du bas à 4 entrées | Liste de cartes | 4 |
| Tablette | 600 à 1023 px | 768 px | Barre du bas | Colonnes essentielles | 8 |
| Ordinateur | 1024 px et plus | 1440 px | Menu latéral de 248 px | Complets | 12, contenu 1200 px au plus, centré |

- Marges latérales : 16 px sur mobile, 24 px dès la tablette.
- Barre du bas : zone de sécurité de l'appareil respectée (`env(safe-area-inset-bottom)`). Le contenu garde une marge égale à la hauteur de la barre.
- Bouton primaire mobile : pleine largeur, 52 px, collé en bas de l'écran dans les formulaires de saisie (Encaisser).
- Panneaux : « Relancer » est un panneau latéral sur ordinateur et une feuille du bas sur mobile.
- Impression : A4 et A5 pour la quittance. Les autres écrans ont une feuille d'impression simple (sans navigation).
- Orientation paysage sur mobile : mise en page mobile conservée.

## Questions ouvertes

- **Palette (décision n° 16, DP-44).** Palette bleu nuit, blanc et turquoise arrêtée. Le logo a été livré le 26/10/2026 (`logo/`, repris dans la maquette) ; à réconcilier avec la palette avant sa diffusion publique. Contrastes calculés par formule WCAG : à revérifier avec un outil avant la diffusion publique.
- **Vouvoiement (décision n° 17).** Le dossier ne fixe pas la forme d'adresse. Le vouvoiement est un choix provisoire.
- **Entrées de menu non définies par la matrice** : *tranchée par DP-66 : elles sont masquées* (voir le tableau « Entrées de menu par rôle »).
- **Parcours de l'agent terrain.** Déduit, non décrit par le dossier (JTBD « Could », F33 en P3). À valider.
- **2FA du gérant.** *Tranchée par DP-68 : T1, obligatoire avant `live`.*
- **Propositions non sourcées dans le dossier** : filigrane « ANNULÉE » d'une quittance annulée, 25 lignes par page de tableau, 8 résultats de recherche par groupe, contenu des variantes d'accueil du caissier, de l'agent terrain et du comptable (S03). À valider par le pilote.
- **Correspondance étape → ton** des relances : proposition. Les six variables de modèles sont arrêtées (R-064).
- **Mentions de la quittance** : à faire relire par un juriste (DP-62). La formulation « francs CFA » est arrêtée (DP-64).
- **Empreinte ou code de contrôle sur la quittance.** L'entité Quittance a une « empreinte » (§15.2). Faut-il l'imprimer ou ajouter un code de vérification ?
- **Durées non définies par le dossier** : durée de blocage après tentatives, durée d'expiration des liens de partage, seuil de confirmation d'un paiement (R-038).
- **Taux de recouvrement.** *Tranchée par R-103 (DP-58) : encaissé du mois divisé par attendu du mois.*
- **Application installable (PWA).** Recommandée par le dossier, non décidée (DECISIONS).
- **Thème sombre.** Non prévu en V1, à confirmer.
- **Budgets de performance et largeur minimale de 360 px.** Propositions, à valider par le prototype.
- **« Caution » ou « dépôt de garantie ».** Libellé principal proposé : « Dépôt de garantie ».
