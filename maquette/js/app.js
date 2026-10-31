/*
 * Coquille de l'application : routeur par ancre (#/ecran/param?cle=valeur), barre du haut, navigation, rôles,
 * légende, réinitialisation. Chargé avant les écrans, qui s'enregistrent par App.enregistrer(nom, définition).
 * Définition d'un écran : { meta(ctx) -> {titre, sous, repere, actions}, rendre(ctx) -> html, apres(ctx, racine) }.
 */
(function () {
  'use strict';
  var F = window.Fmt, UI = window.UI, Roles = window.Roles, Etat = window.Etat;
  var App = window.App = { ecrans: {}, actions: {}, ctx: null, premierRendu: true };

  App.enregistrer = function (nom, def) { App.ecrans[nom] = def; };
  App.action = function (nom, fn) { App.actions[nom] = fn; };

  // Repère de chaque écran (MAQUETTE.md, tableau des repères). pilote = « livré pendant le pilote »
  var REPERES = { travaux: { type: 'v1', pilote: true }, agenda: { type: 'v1', pilote: true } };

  /* ---------- Routeur ---------- */
  function analyser() {
    var h = (location.hash || '').replace(/^#/, ''), q = {}, i = h.indexOf('?');
    if (i >= 0) {
      h.slice(i + 1).split('&').forEach(function (kv) { var p = kv.split('='); if (p[0]) q[decodeURIComponent(p[0])] = decodeURIComponent(p[1] || ''); });
      h = h.slice(0, i);
    }
    var seg = h.split('/').filter(Boolean);
    return { nom: seg[0] || '', params: seg.slice(1), query: q };
  }
  App.aller = function (route) { location.hash = '#' + route; };
  // Latence simulée de 300 à 600 ms sur les actions (désactivable dans les options de démonstration et en mode tournage)
  App.attendre = function (fn) {
    if (Etat.option('latence') && !Etat.option('tournage')) setTimeout(fn, 300 + Math.floor(Math.random() * 300)); else fn();
  };

  function nomEntree(nom) {
    var e = Roles.nav.filter(function (n) { return n.id === nom; })[0];
    return e ? e.lib : nom;
  }

  /* ---------- Coquille (barre du haut, menu latéral, barre du bas) ---------- */
  function actif(nom) {
    var rattache = { lot: 'lots', locataire: 'locataires', quittance: 'paiements', bail: 'baux' };
    return rattache[nom] || nom;
  }

  function rendreCoquille(nom) {
    var role = Roles.def(Etat.role()), D = window.DEMO[role.org], u = Roles.utilisateur(role.id), cur = actif(nom);
    document.getElementById('marque-org').textContent = D.organisation.nom;
    document.getElementById('role-avatar').textContent = F.initiales(u.nom);
    document.getElementById('role-nom').textContent = u.nom;
    document.getElementById('role-fonction').textContent = role.fonction;
    var nbNotif = D.notifications.filter(function (n) { return !n.lu; }).length;
    var pastille = document.getElementById('notif-pastille');
    pastille.textContent = nbNotif; pastille.classList.toggle('cache', !nbNotif);
    document.getElementById('btn-notif').setAttribute('aria-label', 'Notifications' + (nbNotif ? ' (' + nbNotif + ' non lues)' : ''));

    // Menu latéral (ordinateur)
    var html = '';
    Roles.sections.forEach(function (sec, i) {
      var entrees = Roles.entrees(role.id).filter(function (n) { return n.sec === i; });
      if (!entrees.length) return;
      html += '<div class="nav-section"><div class="nav-section-titre">' + sec + '</div>' + entrees.map(function (n) {
        var droit = Roles.droit(role.id, n.id);
        var cpt = n.id === 'impayes' && D.tableauDeBord ? '<span class="nav-compteur" aria-label="' + D.tableauDeBord.enRetard.echeances + ' échéances en retard">' + D.tableauDeBord.enRetard.echeances + '</span>' : '';
        return '<a class="nav-item" href="#/' + n.id + '"' + (cur === n.id ? ' aria-current="page"' : '') + '>' + UI.ic(n.ic) + '<span>' + n.lib + '</span>' + (droit === 'L' && !cpt ? '<span class="lecture">lecture</span>' : cpt) + '</a>';
      }).join('') + '</div>';
    });
    document.getElementById('sidebar-liens').innerHTML = html;

    // Barre du bas : 4 entrées dont « Plus »
    var bb = Roles.barreBas(role.id).map(function (id) {
      var n = Roles.nav.filter(function (x) { return x.id === id; })[0];
      return '<a class="bb-item" href="#/' + id + '"' + (cur === id ? ' aria-current="page"' : '') + '>' + UI.ic(n.ic) + '<span>' + n.lib + '</span></a>';
    }).join('');
    bb += '<button type="button" class="bb-item" data-action="plus" aria-haspopup="dialog">' + UI.ic('more') + '<span>Plus</span></button>';
    document.getElementById('bottombar').innerHTML = bb;
  }

  /* ---------- Rendu d'un écran ---------- */
  function entetePage(meta) {
    var rep = meta.repere ? UI.repere(meta.repere) : '';
    return '<div class="page-entete"><div class="page-titre-ligne"><h1 id="titre-page" tabindex="-1">' + meta.titre + '</h1>' + rep + '</div>' +
      (meta.sous ? '<p class="page-sous-titre">' + meta.sous + '</p>' : '') + (meta.actions ? '<div class="page-actions">' + meta.actions + '</div>' : '') + '</div>';
  }

  function refus(roleDef, nom) {
    return { meta: { titre: 'Accès refusé', repere: { type: 'v1' } }, html: '<div class="carte reserve">' + UI.etatVide({ ic: 'lock', titre: 'Vous n’avez pas le droit d’ouvrir cet écran',
      texte: 'Le rôle « ' + roleDef.libelle + ' » n’y a pas accès. Changez de rôle avec le sélecteur en haut de l’écran, ou revenez à l’accueil.', action: UI.btn('Retour à l’accueil', { v: 'primaire', href: '#' + roleDef.arrivee }) }) + '</div>' };
  }

  App.rendre = function () {
    var r = analyser(), role = Roles.def(Etat.role());
    if (!r.nom) { location.replace('#' + role.arrivee); return; }
    if (r.nom === 'demo' && window.DemoGuidee) { window.DemoGuidee.entrer(r); return; }
    var D = window.DEMO[role.org];
    var ctx = App.ctx = { route: r, params: r.params, query: r.query, role: role, D: D, utilisateur: Roles.utilisateur(role.id) };
    var def = App.ecrans[r.nom], meta, html, nomTitre = r.nom;
    if (!Roles.peutOuvrir(role.id, r.nom)) {
      var x = refus(role, r.nom); meta = x.meta; html = x.html; def = null;
    } else {
      if (!def) def = App.ecrans.introuvable;
      meta = def.meta(ctx);
      if (!meta.repere) meta.repere = REPERES[r.nom] || { type: 'v1' };
      html = def.rendre(ctx);
    }
    document.body.classList.toggle('public', r.nom === 'partage' || r.nom === 'connexion');
    rendreCoquille(r.nom);
    var racine = document.getElementById('contenu');
    racine.innerHTML = entetePage(meta) + '<div id="ecran">' + html + '</div>' +
      '<footer class="pied"><button type="button" class="lien-bouton" data-action="legende">' + UI.ic('info', 'ic-sm') + ' Légende V1 / À venir</button><button type="button" class="lien-bouton" data-action="reinit">' + UI.ic('refresh', 'ic-sm') + ' Réinitialiser la démo</button><span style="align-self:center">Maquette de démonstration · données fictives · nom provisoire</span></footer>';
    document.title = meta.titre.replace(/<[^>]+>/g, '') + ' · Immo Suite (maquette)';
    // Après une navigation, le titre reçoit le focus (lecteurs d'écran) ; l'écran peut ensuite placer le focus sur son champ principal
    if (!App.premierRendu) { var h1 = document.getElementById('titre-page'); if (h1) h1.focus({ preventScroll: true }); window.scrollTo(0, 0); }
    if (def && def.apres) def.apres(ctx, racine);
    App.premierRendu = false;
  };

  /* ---------- Sélecteur de rôle ---------- */
  function menuRoles() {
    var courant = Etat.role();
    var items = Roles.tous().map(function (r) {
      var u = Roles.utilisateur(r.id), org = window.DEMO[r.org].organisation;
      return '<button type="button" class="menu-item" role="menuitemradio" aria-checked="' + (r.id === courant) + '" data-action="role" data-role="' + r.id + '">' +
        '<span class="avatar" aria-hidden="true">' + F.initiales(u.nom) + '</span><span class="menu-texte"><span class="menu-nom">' + r.libelle + ' · ' + F.e(u.nom) + '</span>' +
        '<span class="menu-sous">' + F.e(org.nom) + (r.org === 'saw' ? ' (plan Propriétaire)' : '') + '</span></span>' + (r.id === courant ? '<span class="menu-coche">' + UI.ic('check') + '</span>' : '') + '</button>';
    }).join('');
    return '<div class="popover-titre">Changer de rôle (démonstration)</div><div role="menu">' + items + '</div><div class="menu-sep"></div>' +
      '<button type="button" class="menu-item" data-action="legende">' + UI.ic('info') + '<span class="menu-texte"><span class="menu-nom">Légende V1 / À venir</span></span></button>' +
      '<button type="button" class="menu-item" data-action="demo-liste">' + UI.ic('play') + '<span class="menu-texte"><span class="menu-nom">Démonstration guidée</span><span class="menu-sous">5 parcours et le film de 3 minutes</span></span></button>' +
      '<button type="button" class="menu-item" data-action="tournage">' + UI.ic('eye-off') + '<span class="menu-texte"><span class="menu-nom">Mode tournage</span><span class="menu-sous">Pour l’enregistrement vidéo (touche T)</span></span></button>' +
      '<button type="button" class="menu-item" data-action="visite">' + UI.ic('play') + '<span class="menu-texte"><span class="menu-nom">Visite guidée du produit</span><span class="menu-sous">3 écrans, rejouable</span></span></button>' +
      '<a class="menu-item" href="#/assistant">' + UI.ic('sparkle') + '<span class="menu-texte"><span class="menu-nom">Assistant de démarrage</span><span class="menu-sous">Écran S02</span></span></a>' +
      '<a class="menu-item" href="#/connexion">' + UI.ic('lock') + '<span class="menu-texte"><span class="menu-nom">Écran de connexion</span><span class="menu-sous">Écran S01</span></span></a>' +
      '<button type="button" class="menu-item" data-action="options">' + UI.ic('sliders') + '<span class="menu-texte"><span class="menu-nom">Options de démonstration</span><span class="menu-sous">Latence, cas d’erreur (touche D)</span></span></button>' +
      '<button type="button" class="menu-item" data-action="reinit">' + UI.ic('refresh') + '<span class="menu-texte"><span class="menu-nom">Réinitialiser la démo</span><span class="menu-sous">Retour au rôle Gérant et à l’accueil</span></span></button>';
  }
  var couchePop = null;
  function fermerPop() { if (couchePop) { couchePop.fermer(); couchePop = null; } }

  App.action('role-menu', function () { fermerPop(); couchePop = UI.popover(menuRoles(), { titre: 'Changer de rôle', surFermeture: function () { couchePop = null; } }); });
  App.action('role', function (el) {
    var id = el.getAttribute('data-role'); Etat.definirRole(id); fermerPop();
    var r = Roles.def(id); location.hash = '#' + r.arrivee; App.rendre();
  });
  App.action('legende', function () { fermerPop(); UI.legende(); });
  App.action('reinit', function () {
    fermerPop();
    UI.modale({
      titre: 'Réinitialiser la démo ?',
      corps: '<p>Toutes les actions faites pendant cette démonstration seront effacées. La maquette revient à son état initial, au rôle Gérant, sur l’écran d’accueil.</p>',
      actions: [{ lib: 'Annuler', v: 'secondaire', focus: true }, { lib: 'Réinitialiser', v: 'primaire', fn: function (fermer) {
        fermer(); Etat.reinitialiser(); location.hash = '#/accueil'; location.reload();
      } }]
    });
  });
  // Options de démonstration : latence simulée et cas d'erreur déclenchables à la demande (MAQUETTE.md)
  App.action('options', function () {
    fermerPop();
    var o = Etat.donnees.options, ch = function (b) { return b ? ' checked' : ''; };
    var radio = function (v, lib, aide) { return '<div class="champ" style="margin-top:8px"><label style="display:flex;gap:10px;align-items:flex-start"><input type="radio" name="opt-erreur" value="' + v + '"' + ch((o.erreur || '') === v) + '><span><strong>' + lib + '</strong><br><span class="champ-aide">' + aide + '</span></span></label></div>'; };
    var m = UI.modale({
      titre: 'Options de démonstration',
      corps: '<div class="champ"><label style="display:flex;gap:10px;align-items:flex-start"><input type="checkbox" id="opt-tournage"' + ch(o.tournage) + '><span><strong>Mode tournage</strong> (touche T)<br><span class="champ-aide">Curseur agrandi, cercle au clic, saisie simulée, latence coupée, barres de défilement masquées.</span></span></label>' +
        '<label class="champ-etiquette" for="opt-cadre" style="margin-top:10px;display:block">Taille de la fenêtre en mode tournage</label><select class="saisie" id="opt-cadre"><option value="libre"' + (o.cadre === 'libre' || !o.cadre ? ' selected' : '') + '>Libre (taille du navigateur)</option><option value="ordi"' + (o.cadre === 'ordi' ? ' selected' : '') + '>Ordinateur 1440 × 900</option><option value="mobile"' + (o.cadre === 'mobile' ? ' selected' : '') + '>Mobile 390 × 844</option></select></div>' +
        '<div class="champ" style="margin-top:16px"><label style="display:flex;gap:10px;align-items:center"><input type="checkbox" id="opt-latence"' + ch(o.latence) + '><span><strong>Latence simulée</strong> (300 à 600 ms sur les actions)</span></label></div>' +
        '<fieldset style="border:0;padding:0;margin:16px 0 0"><legend class="champ-etiquette" style="padding:0">Prochain encaissement</legend>' +
        radio('', 'Normal', 'Le paiement est enregistré.') + radio('doublon', 'Référence en doublon', 'Mobile money : « Ce paiement semble déjà enregistré » (R-041).') + radio('connexion', 'Connexion perdue', 'La saisie est conservée, [Réessayer], aucun doublon (R-042).') + '</fieldset>' +
        '<div class="champ"><label style="display:flex;gap:10px;align-items:flex-start"><input type="checkbox" id="opt-lot"' + ch(o.lotSansMandat) + '><span><strong>Compte rendu : un lot sans mandat</strong><br><span class="champ-aide">La production est bloquée (R-075).</span></span></label></div>',
      actions: [{ lib: 'Fermer', v: 'secondaire' }, { lib: 'Enregistrer', v: 'primaire', focus: true, fn: function (fermer) {
        Etat.definirOption('latence', document.getElementById('opt-latence').checked);
        var r = document.querySelector('input[name="opt-erreur"]:checked'); Etat.definirOption('erreur', r && r.value ? r.value : null);
        Etat.definirOption('lotSansMandat', document.getElementById('opt-lot').checked);
        var tourne = document.getElementById('opt-tournage').checked, cadre = document.getElementById('opt-cadre').value;
        fermer();
        if (window.DemoGuidee) {
          window.DemoGuidee.changerCadre(cadre);
          if (tourne !== !!Etat.option('tournage')) { window.DemoGuidee.basculerTournage(); return; }
        } UI.toast('Options de démonstration enregistrées.', { type: 'succes' });
      } }]
    });
  });
  App.action('demo', function () { UI.toast('Action de démonstration', { type: 'info' }); });
  App.action('notif', function () {
    var D = window.DEMO[Roles.def(Etat.role()).org];
    var items = D.notifications.length ? D.notifications.map(function (n) {
      return '<div class="menu-item" style="align-items:flex-start">' + UI.ic(n.lu ? 'bell' : 'alert-circle') + '<span class="menu-texte"><span class="menu-nom"' + (n.lu ? '' : ' style="font-weight:700"') + '>' + F.e(n.texte) + '</span><span class="menu-sous">' + F.date(n.date) + ' · ' + n.heure + (n.lu ? '' : ' · non lue') + '</span></span></div>';
    }).join('') : '<div class="menu-item"><span class="menu-texte"><span class="menu-nom">Aucune notification.</span></span></div>';
    fermerPop(); couchePop = UI.popover('<div class="popover-titre">Notifications</div>' + items, { titre: 'Notifications', surFermeture: function () { couchePop = null; } });
  });
  App.action('plus', function () {
    var role = Roles.def(Etat.role()), bas = Roles.barreBas(role.id);
    var reste = Roles.entrees(role.id).filter(function (n) { return bas.indexOf(n.id) < 0; });
    var html = '<h2 style="margin-bottom:4px">Plus</h2><div class="feuille-grille">' + reste.map(function (n) {
      return '<a class="menu-item" href="#/' + n.id + '">' + UI.ic(n.ic) + '<span class="menu-texte"><span class="menu-nom">' + n.lib + '</span>' + (Roles.droit(role.id, n.id) === 'L' ? '<span class="menu-sous">lecture</span>' : '') + '</span></a>';
    }).join('') + '<button type="button" class="menu-item" data-action="demo-liste">' + UI.ic('play') + '<span class="menu-texte"><span class="menu-nom">Démonstration guidée</span></span></button><button type="button" class="menu-item" data-action="visite">' + UI.ic('play') + '<span class="menu-texte"><span class="menu-nom">Visite guidée du produit</span></span></button><a class="menu-item" href="#/assistant">' + UI.ic('sparkle') + '<span class="menu-texte"><span class="menu-nom">Assistant de démarrage</span></span></a><a class="menu-item" href="#/connexion">' + UI.ic('lock') + '<span class="menu-texte"><span class="menu-nom">Écran de connexion</span></span></a>' +
      '<button type="button" class="menu-item" data-action="legende">' + UI.ic('info') + '<span class="menu-texte"><span class="menu-nom">Légende V1 / À venir</span></span></button>' +
      '<button type="button" class="menu-item" data-action="reinit">' + UI.ic('refresh') + '<span class="menu-texte"><span class="menu-nom">Réinitialiser la démo</span></span></button></div>';
    UI.feuille(html, { titre: 'Plus' });
  });
  App.action('recherche-mobile', function () {
    var m = UI.modale({
      titre: 'Rechercher',
      corps: '<form id="form-rech-mobile"><label class="champ-etiquette" for="rech-mobile">Locataire, lot, téléphone ou n° de quittance</label><input class="saisie" id="rech-mobile" type="search" autocomplete="off" style="margin-top:6px"></form>',
      actions: [{ lib: 'Fermer', v: 'secondaire' }, { lib: 'Rechercher', v: 'primaire', fn: function (fermer) { var q = document.getElementById('rech-mobile').value.trim(); fermer(); location.hash = '#/encaisser' + (q ? '?q=' + encodeURIComponent(q) : ''); } }]
    });
    var champ = document.getElementById('rech-mobile'); if (champ) champ.focus();
  });
  App.action('recherche', function () {
    var q = document.getElementById('champ-recherche').value.trim();
    location.hash = '#/encaisser' + (q ? '?q=' + encodeURIComponent(q) : '');
  });

  /* ---------- Démarrage ---------- */
  document.addEventListener('click', function (ev) {
    var el = ev.target.closest ? ev.target.closest('[data-action]') : null;
    if (!el) return;
    var fn = App.actions[el.getAttribute('data-action')];
    if (fn) { if (el.tagName !== 'INPUT') ev.preventDefault(); fn(el, ev); }
  });
  document.addEventListener('keydown', function (ev) {
    var cible = ev.target, saisie = cible && /^(INPUT|TEXTAREA|SELECT)$/.test(cible.tagName);
    if ((ev.key === 'd' || ev.key === 'D') && !saisie && !ev.ctrlKey && !ev.metaKey && !ev.altKey && !document.querySelector('.modale-fond, .msg-fond, .panneau-fond')) { App.actions.options(); return; }
    if (ev.key === '/' && !saisie && window.innerWidth >= 1024) { var c = document.getElementById('champ-recherche'); if (c) { ev.preventDefault(); c.focus(); } }
  });

  function demarrer() {
    window.injecterIcones();
    Etat.charger();
    if (window.DemoGuidee) { window.DemoGuidee.appliquerClasse(); if (window.DemoGuidee.cadrer()) return; }
    if (window.Mut) window.Mut.rejouer();
    document.getElementById('form-recherche').addEventListener('submit', function (ev) { ev.preventDefault(); App.actions.recherche(); });
    window.addEventListener('hashchange', function () { UI.fermerTout(); App.rendre(); });
    App.rendre();
    if (Etat.messageApresChargement() === 'reinit') UI.toast('Démo réinitialisée.', { type: 'succes' });
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', demarrer); else demarrer();
})();
