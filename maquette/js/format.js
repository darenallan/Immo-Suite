/*
 * Formats d'affichage (DOMAINE.md, UI_UX_DESIGN.md « Conventions »).
 * Aucun calcul métier : uniquement la mise en forme de valeurs déjà présentes dans le jeu de données.
 */
(function () {
  var NBSP = ' ';
  var JOURS = ['dimanche', 'lundi', 'mardi', 'mercredi', 'jeudi', 'vendredi', 'samedi'];
  var MOIS = ['janvier', 'février', 'mars', 'avril', 'mai', 'juin', 'juillet', 'août', 'septembre', 'octobre', 'novembre', 'décembre'];
  var U = ['zéro', 'un', 'deux', 'trois', 'quatre', 'cinq', 'six', 'sept', 'huit', 'neuf', 'dix', 'onze', 'douze', 'treize', 'quatorze', 'quinze', 'seize'];

  function groupes(n) { return String(Math.abs(n)).replace(/\B(?=(\d{3})+(?!\d))/g, NBSP); }

  // Montant « 1 250 000 FCFA » (espace insécable), vrai signe moins pour une correction
  function montant(n, sansUnite) {
    return (n < 0 ? '−' + NBSP : '') + groupes(n) + (sansUnite ? '' : NBSP + 'FCFA');
  }
  function nombre(n) { return groupes(n); }

  // Lecture en toutes lettres pour les lecteurs d'écran (aria-label)
  function m100(n) {
    if (n < 17) return U[n];
    if (n < 20) return 'dix-' + U[n - 10];
    var dz = ['', '', 'vingt', 'trente', 'quarante', 'cinquante', 'soixante'];
    if (n < 70) { var d = Math.floor(n / 10), r = n % 10; return dz[d] + (r === 1 ? ' et un' : r ? '-' + U[r] : ''); }
    if (n < 80) { var r2 = n - 60; return 'soixante' + (r2 === 11 ? ' et onze' : '-' + m100(r2)); }
    var r3 = n - 80; return r3 === 0 ? 'quatre-vingts' : 'quatre-vingt-' + m100(r3);
  }
  function m1000(n, fin) {
    var h = Math.floor(n / 100), r = n % 100, s = '';
    if (h) s = (h === 1 ? 'cent' : U[h] + ' cent') + (h > 1 && r === 0 && fin ? 's' : '');
    if (r) { var t = m100(r); if (r === 80 && !fin) t = 'quatre-vingt'; s += (s ? ' ' : '') + t; }
    return s;
  }
  function enLettres(n) {
    if (n === 0) return 'zéro';
    var neg = n < 0; n = Math.abs(n);
    var p = [], mil = Math.floor(n / 1e6), mille = Math.floor((n % 1e6) / 1000), rest = n % 1000;
    if (mil) p.push((mil === 1 ? 'un' : m1000(mil, false)) + (mil > 1 ? ' millions' : ' million'));
    if (mille) p.push(mille === 1 ? 'mille' : m1000(mille, false).replace(/quatre-vingts$/, 'quatre-vingt') + ' mille');
    if (rest) p.push(m1000(rest, true));
    return (neg ? 'moins ' : '') + p.join(' ');
  }
  function montantSr(n) { return enLettres(n) + ' francs CFA'; }

  function date(iso) { return iso ? iso.slice(8, 10) + '/' + iso.slice(5, 7) + '/' + iso.slice(0, 4) : ''; }
  function dateLongue(iso) {
    var d = new Date(Date.UTC(+iso.slice(0, 4), +iso.slice(5, 7) - 1, +iso.slice(8, 10)));
    return JOURS[d.getUTCDay()] + ' ' + (+iso.slice(8, 10)) + ' ' + MOIS[+iso.slice(5, 7) - 1] + ' ' + iso.slice(0, 4);
  }
  function mois(periode) { return MOIS[+periode.slice(5, 7) - 1] + ' ' + periode.slice(0, 4); }
  function jours(n) { return n + NBSP + (n > 1 ? 'jours' : 'jour'); }
  function pourcent(x) { return String(x).replace('.', ',') + NBSP + '%'; }
  function majuscule(s) { return s ? s.charAt(0).toUpperCase() + s.slice(1) : s; }
  function initiales(nom) {
    var m = nom.replace(/^(M\.|Mme)\s+/, '').split(/\s+/).filter(Boolean);
    return ((m[0] || '').charAt(0) + (m.length > 1 ? m[m.length - 1].charAt(0) : '')).toUpperCase();
  }
  function echapper(s) {
    return String(s === null || s === undefined ? '' : s).replace(/[&<>"']/g, function (c) { return { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]; });
  }
  // Insensible aux accents et à la casse (recherche)
  function normaliser(s) { return String(s || '').normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase(); }

  window.Fmt = { montant: montant, nombre: nombre, montantSr: montantSr, enLettres: enLettres, date: date, dateLongue: dateLongue, mois: mois, jours: jours, pourcent: pourcent, majuscule: majuscule, initiales: initiales, e: echapper, norm: normaliser, NBSP: NBSP };
})();
