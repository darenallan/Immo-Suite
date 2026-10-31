/*
 * Composants communs : icônes, badges de statut, repères V1 / À venir, modales, toasts, menus.
 * Les fonctions retournent du HTML (chaînes) ; les données saisies sont toujours échappées avec Fmt.e.
 */
(function () {
  var F = window.Fmt;

  function ic(nom, cls) {
    return '<svg class="ic' + (cls ? ' ' + cls : '') + '" aria-hidden="true" focusable="false"><use href="#i-' + nom + '"/></svg>';
  }

  // Statuts financiers : icône + libellé + couleur, toujours ensemble (UI_UX_DESIGN.md)
  var STATUTS = {
    payee: { lib: 'Payé', ic: 'check-circle', cls: 'paid' },
    valide: { lib: 'Validé', ic: 'check-circle', cls: 'paid' },
    partielle: { lib: 'Partiel', ic: 'half-circle', cls: 'partial' },
    en_retard: { lib: 'En retard', ic: 'alert-triangle', cls: 'late' },
    a_venir: { lib: 'À venir', ic: 'calendar', cls: 'upcoming' },
    due: { lib: 'Due', ic: 'bell', cls: 'due' },
    avance: { lib: 'Avance', ic: 'arrow-up-circle', cls: 'credit' },
    annule: { lib: 'Annulé', ic: 'ban', cls: 'void' },
    corrige: { lib: 'Corrigé', ic: 'ban', cls: 'void' }
  };
  function statut(cle, detail) {
    var s = STATUTS[cle];
    return '<span class="badge badge-' + s.cls + '">' + ic(s.ic) + '<span class="badge-texte">' + s.lib + '</span>' + (detail ? '<span>· ' + F.e(detail) + '</span>' : '') + '</span>';
  }
  function neutre(lib, icone, extra) {
    return '<span class="badge badge-neutre' + (extra ? ' ' + extra : '') + '">' + (icone ? ic(icone) : '') + F.e(lib) + '</span>';
  }
  function usage(u) {
    var m = { habitation: ['Habitation', 'home'], commercial: ['Commercial', 'store'], mixte: ['Mixte', 'layers'] }[u];
    return neutre(m[0], m[1]);
  }

  // Repère « V1 » / « À venir » : pastille de 24 px, le texte porte le sens
  function repere(o) {
    o = o || {};
    var mini = o.mini ? ' repere-mini' : '';
    if (o.type === 'avenir') {
      return '<span class="repere repere-avenir' + mini + '" title="À venir : envisagé selon la demande des clients, non planifié, sans engagement de date.">' + ic('hourglass') + 'À venir</span>';
    }
    return '<span class="repere repere-v1' + mini + '" title="' + (o.pilote ? 'V1, livré pendant le pilote' : 'Prévu dans la première version') + '">' + ic('check') + 'V1' + (o.pilote ? ' · livré pendant le pilote' : '') + '</span>';
  }

  function montant(n, opt) {
    opt = opt || {};
    return '<span class="num' + (opt.cls ? ' ' + opt.cls : '') + '" aria-label="' + F.e(F.montantSr(n)) + '">' + F.montant(n) + '</span>';
  }

  function btn(lib, o) {
    o = o || {};
    var cls = 'btn btn-' + (o.v || 'secondaire') + (o.grand ? ' btn-grand' : '') + (o.petit ? ' btn-petit' : '') + (o.cls ? ' ' + o.cls : '');
    var attrs = (o.action ? ' data-action="' + o.action + '"' : '') + (o.data || '') + (o.id ? ' id="' + o.id + '"' : '');
    var contenu = (o.ic ? ic(o.ic) : '') + '<span>' + lib + '</span>';
    if (o.href) return '<a class="' + cls + '" href="' + o.href + '"' + attrs + '>' + contenu + '</a>';
    return '<button type="button" class="' + cls + '"' + attrs + (o.disabled ? ' disabled' : '') + '>' + contenu + '</button>';
  }

  function etatVide(o) {
    return '<div class="etat-vide">' + ic(o.ic || 'info') + '<h2>' + o.titre + '</h2><p class="secondaire">' + o.texte + '</p>' + (o.action || '') + '</div>';
  }

  /* ---------- Toasts ---------- */
  function toast(texte, o) {
    o = o || {};
    var zone = document.getElementById('toasts'); if (!zone) return;
    var type = o.type || 'info';
    var el = document.createElement('div');
    el.className = 'toast' + (type === 'succes' ? ' toast-succes' : type === 'avert' ? ' toast-avert' : type === 'erreur' ? ' toast-erreur' : '');
    el.setAttribute('role', type === 'erreur' ? 'alert' : 'status');
    el.innerHTML = ic(type === 'succes' ? 'check-circle' : type === 'erreur' || type === 'avert' ? 'alert-circle' : 'info') +
      '<span class="toast-texte">' + texte + '</span>' + (o.lien ? '<button type="button" data-toast-lien>' + o.lien.lib + '</button>' : '') +
      '<button type="button" aria-label="Fermer le message" data-toast-fermer>' + ic('x', 'ic-sm') + '</button>';
    zone.appendChild(el);
    function fermer() { if (el.parentNode) el.parentNode.removeChild(el); }
    el.querySelector('[data-toast-fermer]').addEventListener('click', fermer);
    if (o.lien) el.querySelector('[data-toast-lien]').addEventListener('click', function () { o.lien.fn(); fermer(); });
    // Succès et information : 4 s. Avertissement et erreur : restent jusqu'à fermeture (UI_UX_DESIGN.md)
    if (type === 'succes' || type === 'info') setTimeout(fermer, o.duree || 4000);
  }

  /* ---------- Couches (modales, feuilles, menus) ---------- */
  var pile = []; // couches ouvertes, la dernière est au-dessus
  function piegeFocus(couche) {
    return function (ev) {
      if (ev.key !== 'Tab') return;
      var f = couche.querySelectorAll('a[href], button:not([disabled]), input:not([disabled]), select, textarea, [tabindex]:not([tabindex="-1"])');
      if (!f.length) return;
      var premier = f[0], dernier = f[f.length - 1];
      if (ev.shiftKey && document.activeElement === premier) { dernier.focus(); ev.preventDefault(); }
      else if (!ev.shiftKey && document.activeElement === dernier) { premier.focus(); ev.preventDefault(); }
    };
  }
  function ouvrirCouche(fond, o) {
    o = o || {};
    var retour = document.activeElement;
    document.getElementById('calques').appendChild(fond);
    var piege = piegeFocus(fond);
    fond.addEventListener('keydown', piege);
    var entree = { fond: fond, retour: retour, fermer: null };
    function fermer() {
      if (fond.parentNode) fond.parentNode.removeChild(fond);
      pile = pile.filter(function (x) { return x !== entree; });
      if (retour && retour.focus && document.contains(retour)) retour.focus();
      if (o.surFermeture) o.surFermeture();
    }
    entree.fermer = fermer;
    pile.push(entree);
    return fermer;
  }
  // Fermeture de toutes les couches ouvertes (changement d'écran)
  function fermerTout() { pile.slice().reverse().forEach(function (e) { e.fermer(); }); }
  document.addEventListener('keydown', function (ev) {
    if (ev.key === 'Escape' && pile.length) { pile[pile.length - 1].fermer(); ev.stopPropagation(); }
  });

  // Modale de confirmation : titre = la question, corps = la conséquence, [Annuler] puis [Verbe]
  function modale(o) {
    var fond = document.createElement('div');
    fond.className = 'modale-fond';
    var acts = (o.actions || []).map(function (a, i) {
      return '<button type="button" class="btn btn-' + (a.v || 'secondaire') + '" data-i="' + i + '"' + (a.focus ? ' data-focus="1"' : '') + '>' + a.lib + '</button>';
    }).join('');
    fond.innerHTML = '<div class="modale" role="dialog" aria-modal="true" aria-labelledby="modale-titre"><div class="modale-poignee" aria-hidden="true"></div><h2 id="modale-titre">' + o.titre + '</h2><div class="modale-corps">' + (o.corps || '') + '</div><div class="modale-actions">' + acts + '</div></div>';
    var fermer = ouvrirCouche(fond);
    fond.addEventListener('mousedown', function (ev) { if (ev.target === fond) fermer(); });
    (o.actions || []).forEach(function (a, i) {
      fond.querySelector('[data-i="' + i + '"]').addEventListener('click', function () { if (a.fn) a.fn(fermer); else fermer(); });
    });
    var premier = fond.querySelector('[data-focus="1"]') || fond.querySelector('button[data-i]');
    if (premier) premier.focus();
    return { fermer: fermer, el: fond };
  }

  // Menu déroulant ancré sous la barre du haut (rôles, notifications)
  function popover(html, o) {
    o = o || {};
    var fond = document.createElement('div');
    fond.className = 'popover-fond';
    fond.innerHTML = '<div class="popover" role="dialog" aria-label="' + F.e(o.titre || 'Menu') + '">' + html + '</div>';
    var fermer = ouvrirCouche(fond, o);
    fond.addEventListener('mousedown', function (ev) { if (ev.target === fond) fermer(); });
    var premier = fond.querySelector('[aria-checked="true"]') || fond.querySelector('button, a');
    if (premier) premier.focus();
    return { fermer: fermer, el: fond };
  }

  // Feuille du bas (menu « Plus » sur mobile)
  function feuille(html, o) {
    o = o || {};
    var fond = document.createElement('div');
    fond.className = 'feuille-fond';
    fond.innerHTML = '<div class="feuille" role="dialog" aria-modal="true" aria-label="' + F.e(o.titre || 'Menu') + '"><div class="modale-poignee" aria-hidden="true"></div>' + html + '<div style="margin-top:12px">' + btn('Fermer', { v: 'secondaire', cls: 'btn-grand', data: ' data-fermer-couche="1"' }) + '</div></div>';
    var fermer = ouvrirCouche(fond, o);
    fond.addEventListener('mousedown', function (ev) { if (ev.target === fond) fermer(); });
    fond.querySelector('[data-fermer-couche]').addEventListener('click', fermer);
    fond.querySelectorAll('a[href]').forEach(function (a) { a.addEventListener('click', fermer); });
    var premier = fond.querySelector('a, button'); if (premier) premier.focus();
    return { fermer: fermer, el: fond };
  }

  // Panneau latéral sur ordinateur, feuille du bas sur mobile (« Relancer », détail d'un paiement)
  function panneau(titre, html, o) {
    o = o || {};
    var fond = document.createElement('div');
    fond.className = 'panneau-fond';
    fond.innerHTML = '<div class="panneau" role="dialog" aria-modal="true" aria-labelledby="panneau-titre"><div class="modale-poignee" aria-hidden="true"></div>' +
      '<div class="panneau-entete"><h2 id="panneau-titre">' + titre + '</h2><button type="button" class="btn-icone" data-fermer-couche="1" aria-label="Fermer">' + ic('x') + '</button></div>' +
      '<div class="panneau-corps">' + html + '</div></div>';
    var fermer = ouvrirCouche(fond, o);
    fond.addEventListener('mousedown', function (ev) { if (ev.target === fond) fermer(); });
    fond.querySelector('[data-fermer-couche]').addEventListener('click', fermer);
    var premier = fond.querySelector('.panneau-corps input, .panneau-corps select, .panneau-corps button, .panneau-corps a'); if (premier) premier.focus();
    return { fermer: fermer, el: fond, corps: fond.querySelector('.panneau-corps') };
  }

  // Messagerie simulée : texte prêt, aucun message réel, aucun lien réel, le mot « WhatsApp » en texte seulement
  function messagerie(o) {
    var canal = o.canal === 'sms' ? 'SMS' : 'WhatsApp';
    var fond = document.createElement('div');
    fond.className = 'msg-fond';
    fond.innerHTML = '<div class="msg" role="dialog" aria-modal="true" aria-label="' + canal + ' (simulation)">' +
      '<div class="msg-entete"><button type="button" class="btn-icone msg-retour" data-msg="retour" aria-label="Retour à Immo Suite">' + ic('chevron-left') + '</button>' +
      '<span class="avatar" aria-hidden="true">' + F.initiales(o.nom) + '</span><div class="msg-contact"><strong>' + F.e(o.nom) + '</strong><span>' + F.e(o.telephone) + '</span></div>' +
      '</div><div class="msg-canal">' + canal + ' · simulation</div>' +
      '<div class="msg-fil" id="msg-fil" aria-live="polite"><div class="msg-info">Simulation : aucun message n’est réellement envoyé. Le texte est prêt, vous gardez la main sur l’envoi.</div></div>' +
      '<div class="msg-composeur" id="msg-composeur"><div class="msg-brouillon" id="msg-brouillon">' + F.e(o.texte) + '</div>' +
      '<button type="button" class="btn btn-primaire btn-grand" data-msg="envoyer">' + ic('arrow-right') + '<span>Envoyer</span></button></div></div>';
    var envoye = false, fermer = ouvrirCouche(fond, { surFermeture: function () { if (o.surRetour) o.surRetour(envoye); } });
    fond.querySelector('[data-msg="retour"]').addEventListener('click', fermer);
    fond.querySelector('[data-msg="envoyer"]').addEventListener('click', function () {
      envoye = true;
      var heure = window.DEMO[window.Etat.orgCle()].meta.heure;
      fond.querySelector('#msg-fil').insertAdjacentHTML('beforeend', '<div class="msg-bulle"><div class="msg-texte">' + F.e(o.texte) + '</div><div class="msg-heure">' + heure + ' ' + ic('check', 'ic-sm') + ' Envoyé</div></div>');
      fond.querySelector('#msg-composeur').innerHTML = '<p class="msg-fin">Message envoyé (simulation).</p><button type="button" class="btn btn-primaire btn-grand" data-msg="fin">' + ic('arrow-right') + '<span>Retour à Immo Suite</span></button>';
      var fin = fond.querySelector('[data-msg="fin"]'); fin.addEventListener('click', fermer); fin.focus();
      if (o.surEnvoi) o.surEnvoi();
    });
    fond.querySelector('[data-msg="envoyer"]').focus();
    return { fermer: fermer, el: fond };
  }

  function legende() {
    modale({
      titre: 'Légende des repères',
      corps: '<div class="legende-bloc">' +
        '<div>' + repere({ type: 'v1' }) + '<span>Prévu dans la première version. La mention « livré pendant le pilote » signale une fonction livrée après le démarrage du pilote.</span></div>' +
        '<div>' + repere({ type: 'avenir' }) + '<span>Envisagé selon la demande des clients, non planifié, sans engagement de date.</span></div></div>',
      actions: [{ lib: 'Fermer', v: 'primaire', focus: true }]
    });
  }

  window.UI = { fermerTout: fermerTout, ic: ic, statut: statut, neutre: neutre, usage: usage, repere: repere, montant: montant, btn: btn, etatVide: etatVide, toast: toast, modale: modale, popover: popover, feuille: feuille, panneau: panneau, messagerie: messagerie, legende: legende, STATUTS: STATUTS };
})();
