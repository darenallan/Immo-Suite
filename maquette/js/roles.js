/*
 * Rôles du sélecteur et menu de navigation.
 * Source : MAQUETTE.md (tableau des rôles) et UI_UX_DESIGN.md (« Entrées de menu par rôle »).
 * É = voit et agit. L = lecture seule. Absent = entrée masquée.
 * Les cases « ? » (non définies par la matrice du dossier) sont masquées par prudence : voir NOTES.md.
 */
(function () {
  var ROLES = {
    caissier: { id: 'caissier', libelle: 'Caissier', fonction: 'Caissière', userId: 'U3', org: 'hor', arrivee: '/encaisser' },
    gestionnaire: { id: 'gestionnaire', libelle: 'Gestionnaire', fonction: 'Gestionnaire', userId: 'U2', org: 'hor', arrivee: '/accueil' },
    gerant: { id: 'gerant', libelle: 'Gérant', fonction: 'Gérante', userId: 'U1', org: 'hor', arrivee: '/accueil' },
    proprietaire: { id: 'proprietaire', libelle: 'Propriétaire direct', fonction: 'Propriétaire direct', userId: 'S1', org: 'saw', arrivee: '/accueil' }
  };
  var ORDRE = ['caissier', 'gestionnaire', 'gerant', 'proprietaire'];

  var SECTIONS = ['Au quotidien', 'Patrimoine', 'Gestion', 'Outils'];
  // droits : g = gérant, s = gestionnaire, c = caissier, p = propriétaire direct
  var NAV = [
    { id: 'accueil', lib: 'Accueil', ic: 'home', sec: 0, d: { g: 'E', s: 'E', c: 'E', p: 'E' } },
    { id: 'encaisser', lib: 'Encaisser', ic: 'banknote', sec: 0, d: { g: 'E', s: 'E', c: 'E', p: 'E' } },
    { id: 'impayes', lib: 'Impayés', ic: 'alert-triangle', sec: 0, d: { g: 'E', s: 'E', c: 'L', p: 'E' } },
    { id: 'paiements', lib: 'Paiements et quittances', ic: 'receipt', sec: 0, d: { g: 'E', s: 'E', c: 'E', p: 'E' } },
    { id: 'lots', lib: 'Biens et lots', ic: 'building', sec: 1, d: { g: 'E', s: 'E', c: 'L', p: 'E' } },
    { id: 'locataires', lib: 'Locataires', ic: 'users', sec: 1, d: { g: 'E', s: 'E', c: 'L', p: 'E' } },
    { id: 'baux', lib: 'Baux', ic: 'file-text', sec: 1, d: { g: 'E', s: 'E', p: 'E' } },
    { id: 'comptes-rendus', lib: 'Comptes rendus', ic: 'chart', sec: 2, d: { g: 'E', s: 'L' } },
    { id: 'mandants', lib: 'Mandants et mandats', ic: 'briefcase', sec: 2, d: { g: 'E' } },
    { id: 'travaux', lib: 'Travaux et documents', ic: 'wrench', sec: 2, d: { g: 'E', s: 'E', p: 'E' } },
    { id: 'agenda', lib: 'Agenda', ic: 'calendar', sec: 2, d: { g: 'E', s: 'E', p: 'E' } },
    { id: 'import', lib: 'Import', ic: 'upload', sec: 3, d: { g: 'E', s: 'E', p: 'E' } },
    { id: 'parametres', lib: 'Paramètres', ic: 'sliders', sec: 3, d: { g: 'E', p: 'E' } },
    { id: 'utilisateurs', lib: 'Utilisateurs', ic: 'user-plus', sec: 3, d: { g: 'E', p: 'E' } }
  ];
  // Écrans qui se rattachent à une entrée de menu pour le contrôle des droits
  var RATTACHE = { lot: 'lots', locataire: 'locataires', quittance: 'paiements', bail: 'baux' };
  var PUBLICS = { connexion: 1, partage: 1 };
  var CODE = { gerant: 'g', gestionnaire: 's', caissier: 'c', proprietaire: 'p' };

  var Roles = {
    ordre: ORDRE, sections: SECTIONS, nav: NAV,
    def: function (id) { return ROLES[id]; },
    tous: function () { return ORDRE.map(function (i) { return ROLES[i]; }); },
    utilisateur: function (id) { var d = ROLES[id]; return window.DEMO[d.org].utilisateurs.filter(function (u) { return u.id === d.userId; })[0]; },
    // Droit du rôle sur une entrée : 'E', 'L' ou null
    droit: function (roleId, entreeId) {
      var e = NAV.filter(function (n) { return n.id === entreeId; })[0];
      return e ? (e.d[CODE[roleId]] || null) : null;
    },
    // Droit d'ouvrir un écran (entrée de menu, écran rattaché ou écran public)
    peutOuvrir: function (roleId, nomEcran) {
      if (PUBLICS[nomEcran]) return 'E';
      if (nomEcran === 'assistant') return Roles.droit(roleId, 'parametres');
      return Roles.droit(roleId, RATTACHE[nomEcran] || nomEcran);
    },
    entrees: function (roleId) { return NAV.filter(function (n) { return Roles.droit(roleId, n.id); }); },
    // Barre du bas : toujours 4 entrées dont « Plus » (UI_UX_DESIGN.md, Navigation)
    barreBas: function (roleId) {
      return ['accueil', 'encaisser', 'impayes'].filter(function (i) { return Roles.droit(roleId, i); });
    }
  };
  window.Roles = Roles;
})();
