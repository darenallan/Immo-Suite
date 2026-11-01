// Tous les textes de l'interface (UI_UX_DESIGN.md, vouvoiement provisoire : décision n° 17).
// Les messages d'erreur reprennent le catalogue de API_CONTRACTS.md §3.4 ; un texte marqué ✱ y est une proposition à valider.
export const fr = {
  app: {
    name: 'Immo Suite',
    tagline: 'Gestion locative pour le Burkina Faso',
  },
  common: {
    loading: 'Chargement…',
    retry: 'Réessayer',
    skipToContent: 'Aller au contenu',
  },
  login: {
    title: 'Connexion',
    identifier: 'Téléphone ou e-mail',
    identifierHint: 'Exemple : 70 12 34 56',
    password: 'Mot de passe',
    submit: 'Se connecter',
    submitting: 'Connexion…',
    forgot: 'Mot de passe oublié ?',
    forgotHelp: 'Demandez un lien de réinitialisation à votre gérant.',
  },
  chooser: {
    title: 'Choisissez une organisation',
    intro: 'Votre compte donne accès à plusieurs organisations.',
    roleLabel: 'Rôle',
  },
  home: {
    title: 'Accueil',
    welcome: 'Bonjour',
    organization: 'Organisation',
    role: 'Rôle',
    comingSoon: 'Les écrans de gestion arrivent aux étapes suivantes.',
  },
  denied: {
    title: 'Accès refusé',
    body: 'Votre rôle ne donne pas accès à cette page.',
    back: 'Retour à l’accueil',
  },
  notFound: {
    title: 'Introuvable',
    body: 'Cette page n’existe pas ou vous n’y avez pas accès.',
    back: 'Retour à l’accueil',
  },
  nav: {
    label: 'Navigation principale',
    more: 'Plus',
    moreTitle: 'Autres entrées',
    close: 'Fermer',
    comingSoon: 'À venir',
    entries: {
      home: 'Accueil',
      collect: 'Encaisser',
      arrears: 'Impayés',
      payments: 'Paiements et quittances',
      properties: 'Biens et lots',
      tenants: 'Locataires',
      leases: 'Baux',
      statements: 'Comptes rendus',
      mandates: 'Mandants et mandats',
      works: 'Travaux et documents',
      agenda: 'Agenda',
      import: 'Import',
      cashClose: 'Clôture de caisse',
      notifications: 'Notifications',
      settings: 'Paramètres',
      users: 'Utilisateurs',
      audit: 'Journal d’audit',
    },
  },
  roles: {
    manager: 'Gérant',
    property_manager: 'Gestionnaire',
    cashier: 'Caissier',
    field_agent: 'Agent terrain',
    accountant: 'Comptable',
    owner: 'Propriétaire',
    collaborator: 'Collaborateur',
  },
  menu: {
    user: 'Menu utilisateur',
    switchOrganization: 'Changer d’organisation',
    logout: 'Déconnexion',
    logoutAll: 'Déconnecter tous mes appareils',
  },
  update: {
    banner: 'Une mise à jour est disponible.',
    reload: 'Actualiser',
  },
  errors: {
    'auth.invalid_credentials': 'Identifiant ou mot de passe incorrect.',
    'auth.session_expired': 'Votre session a expiré. Reconnectez-vous.',
    'auth.unauthenticated': 'Votre session a expiré. Reconnectez-vous.',
    'auth.csrf_invalid': 'Votre session n’est plus valide pour cette action. Rechargez la page.',
    'auth.no_active_membership': 'Votre compte n’a accès à aucune organisation. Contactez votre gérant.',
    'auth.organization_required': 'Choisissez une organisation pour continuer.',
    'org.suspended': 'Cette organisation est suspendue. Contactez le support.',
    'resource.not_found': 'Introuvable. Cette page n’existe pas ou vous n’y avez pas accès.',
    'network.unreachable': 'Connexion impossible. Vérifiez votre réseau et réessayez.',
    'server.unexpected': 'Un problème est survenu. Réessayez.',
  },
  errorWithIncident: (code: string) =>
    `Un problème est survenu. Réessayez. Si le problème continue, communiquez le code ${code} au support.`,
  rateLimited: (duration: string) =>
    `Trop de tentatives. Réessayez dans ${duration} ou réinitialisez votre mot de passe.`,
  duration: {
    seconds: (n: number) => `${String(n)} seconde${n > 1 ? 's' : ''}`,
    minutes: (n: number) => `${String(n)} minute${n > 1 ? 's' : ''}`,
  },
} as const;

export type RoleKey = keyof typeof fr.roles;
