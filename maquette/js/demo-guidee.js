/*
 * Démonstration guidée et mode tournage (MAQUETTE.md, « Mode Démonstration guidée »).
 * Route : #/demo/{parcours}[?pas=N]. Un parcours = une liste d'étapes (js/demos/*.js) :
 *   { ecran, role, titre, texte, voix, d, cible, entree(H), faire(H), muet, chapitre }
 *   - ecran / role : appliqués à l'arrivée sur l'étape ;
 *   - cible        : élément mis en évidence (sélecteur, {sel, texte}, fonction, ou liste de ces formes) ;
 *   - entree(H)    : exécuté à l'arrivée, avant la bulle ;
 *   - faire(H)     : l'action visible, exécutée en quittant l'étape (« Suivant ») ;
 *   - muet         : étape jouée sans bulle (sert aux versions condensées du film) ;
 *   - d            : durée en secondes pour la lecture automatique.
 * « Précédent » rejoue le parcours depuis le début, sans animation, après remise à zéro du journal :
 * aucune modification de la démo ne peut rester d'une étape plus avancée.
 * Les actions simulées passent par les vrais gestionnaires de l'application (clics et saisies réels).
 */
(function () {
  'use strict';
  var UI = window.UI, Etat = window.Etat, App = window.App, F = window.Fmt;
  var DG = window.DemoGuidee = { parcours: {}, ordre: [], actif: null, jeton: 0 };
  var ANNULE = { annule: true };
  var el = {}; // éléments de la surcouche
  var minuteur = null;
  var cur = { x: 0, y: 0, vu: false };

  DG.enregistrer = function (id, def) { def.id = id; DG.parcours[id] = def; DG.ordre.push(id); };

  /* ---------- Aides : recherche d'éléments ---------- */
  function visible(e) {
    var r = e.getBoundingClientRect();
    if (!r.width && !r.height) return false;
    var s = getComputedStyle(e);
    return s.visibility !== 'hidden' && s.display !== 'none';
  }
  function net(t) { return String(t || '').replace(/[  ]/g, ' ').replace(/\s+/g, ' ').toLowerCase(); }
  // Premier élément visible qui correspond au sélecteur (et qui contient le texte, si donné)
  function chercher(sel, texte) {
    var liste = document.querySelectorAll(sel);
    for (var i = 0; i < liste.length; i++) {
      if (!visible(liste[i])) continue;
      if (texte && net(liste[i].textContent).indexOf(net(texte)) < 0) continue;
      return liste[i];
    }
    return null;
  }
  // Forme générale d'une cible -> liste d'éléments trouvés (éventuellement vide)
  function resoudre(c) {
    if (!c) return [];
    if (Array.isArray(c)) return c.reduce(function (a, x) { return a.concat(resoudre(x)); }, []);
    if (typeof c === 'function') { var r = c(); return r ? [r] : []; }
    if (c.nodeType) return [c];
    var e = typeof c === 'string' ? chercher(c) : chercher(c.sel, c.texte);
    return e ? [e] : [];
  }
  function verifier() { if (DG.actif && DG.actif.jeton !== DG.jeton) throw ANNULE; }

  /* ---------- Aides exposées aux scénarios (H) ---------- */
  var H = DG.H = {
    vite: false,
    pause: function (ms) {
      verifier();
      return new Promise(function (ok) { setTimeout(ok, H.vite ? 0 : ms); }).then(verifier);
    },
    // Attend qu'une cible apparaisse (4 s par défaut). Renvoie le premier élément, ou null avec un avertissement
    attendre: async function (c, ms) {
      var fin = Date.now() + (ms || 4000), r;
      for (;;) {
        verifier(); r = resoudre(c);
        if (r.length) return r[0];
        if (Date.now() > fin) { console.warn('Démonstration guidée : cible introuvable', c && (c.sel || c)); return null; }
        await new Promise(function (ok) { setTimeout(ok, 40); });
      }
    },
    // Attend qu'une cible disparaisse (fenêtre fermée, bouton remplacé)
    disparait: async function (c, ms) {
      var fin = Date.now() + (ms || 4000);
      while (resoudre(c).length && Date.now() < fin) { verifier(); await new Promise(function (ok) { setTimeout(ok, 40); }); }
    },
    aller: aller,
    role: function (id) { if (Etat.role() !== id) Etat.definirRole(id); },
    fermerTout: function () { UI.fermerTout(); },
    cliquer: async function (c) {
      var e = await H.attendre(c); if (!e) return null;
      await montrerGeste(e);
      e.click();
      await H.pause(260);
      return e;
    },
    saisir: async function (c, texte) {
      var e = await H.attendre(c); if (!e) return null;
      await montrerGeste(e);
      e.focus(); e.value = '';
      if (!Etat.option('tournage') || H.vite) { e.value = texte; e.dispatchEvent(new Event('input', { bubbles: true })); }
      else { for (var i = 0; i < texte.length; i++) { e.value += texte.charAt(i); e.dispatchEvent(new Event('input', { bubbles: true })); await H.pause(90); } }
      e.dispatchEvent(new Event('change', { bubbles: true }));
      e.blur();
      await H.pause(200);
      return e;
    },
    choisir: async function (c, valeur) {
      var e = await H.attendre(c); if (!e) return null;
      await montrerGeste(e);
      e.value = valeur;
      e.dispatchEvent(new Event('input', { bubbles: true })); e.dispatchEvent(new Event('change', { bubbles: true }));
      await H.pause(260);
      return e;
    }
  };

  // Défile jusqu'à l'élément, déplace le curseur de tournage dessus et montre le cercle de clic
  async function montrerGeste(e) {
    amener(e);
    if (H.vite || !Etat.option('tournage')) return;
    var r = e.getBoundingClientRect();
    deplacerCurseur(Math.min(r.right - 8, r.left + Math.max(12, r.width / 2)), r.top + r.height / 2);
    await H.pause(620);
    cercle(cur.x, cur.y);
    await H.pause(260);
  }
  function amener(e) {
    var r = e.getBoundingClientRect(), vh = window.innerHeight;
    if (r.top < 70 || r.bottom > vh - 70) {
      e.scrollIntoView({ block: r.height > vh * 0.7 ? 'start' : 'center' });
      if (r.height > vh * 0.7) window.scrollBy(0, -80);
    }
  }

  /* ---------- Navigation ---------- */
  function aller(h) {
    return new Promise(function (ok) {
      if (location.hash === h) { UI.fermerTout(); App.rendre(); setTimeout(ok, 40); return; }
      var f = function () { window.removeEventListener('hashchange', f); setTimeout(ok, 50); };
      window.addEventListener('hashchange', f);
      location.hash = h;
    });
  }
  // Arrivée sur une étape : rôle, écran, puis hook d'entrée
  async function entrer(p) {
    if (p.role && Etat.role() !== p.role) Etat.definirRole(p.role);
    if (p.ecran) await aller(p.ecran);
    if (p.entree) await p.entree(H);
  }
  async function jouer(p) { await entrer(p); if (p.faire) await p.faire(H); }

  /* ---------- Surcouche : halo, bulle, carte, curseur ---------- */
  function creerSurcouche() {
    if (el.bulle) return;
    var b = document.body;
    function noeud(tag, id, cls) { var n = document.createElement(tag); n.id = id; if (cls) n.className = cls; b.appendChild(n); return n; }
    el.halo = noeud('div', 'demo-halo', 'cache');
    el.carte = noeud('div', 'demo-carte', 'cache');
    el.bulle = noeud('section', 'demo-bulle', 'cache');
    el.bulle.setAttribute('aria-label', 'Démonstration guidée');
    el.bulle.addEventListener('click', function (ev) {
      var a = ev.target.closest('[data-demo]'); if (!a) return;
      ev.preventDefault(); ev.stopPropagation();
      var k = a.getAttribute('data-demo');
      if (k === 'suiv') DG.suivant(); else if (k === 'prec') DG.precedent(); else if (k === 'lecture') DG.lecture(); else if (k === 'quitter') DG.quitter();
    });
  }
  function enleverSurcouche() { ['halo', 'carte', 'bulle'].forEach(function (k) { if (el[k]) { el[k].remove(); el[k] = null; } }); }

  function rectUnion(liste) {
    var x0 = 1e9, y0 = 1e9, x1 = -1e9, y1 = -1e9;
    liste.forEach(function (e) { var r = e.getBoundingClientRect(); x0 = Math.min(x0, r.left); y0 = Math.min(y0, r.top); x1 = Math.max(x1, r.right); y1 = Math.max(y1, r.bottom); });
    return { left: x0, top: y0, right: x1, bottom: y1, width: x1 - x0, height: y1 - y0 };
  }
  function poserHalo() {
    var a = DG.actif; if (!a || !el.halo) return;
    var liste = resoudre(a.cible);
    if (!liste.length) { el.halo.classList.add('cache'); return; }
    var r = rectUnion(liste), m = 5;
    el.halo.style.cssText = 'left:' + (r.left - m) + 'px;top:' + (r.top - m) + 'px;width:' + (r.width + 2 * m) + 'px;height:' + (r.height + 2 * m) + 'px';
    el.halo.classList.remove('cache');
  }
  // Position de la bulle : docked en haut ou en bas sur mobile, à côté de la cible sur ordinateur
  function poserBulle() {
    var a = DG.actif; if (!a || !el.bulle) return;
    var vw = window.innerWidth, vh = window.innerHeight, g = 14, liste = resoudre(a.cible), r = liste.length ? rectUnion(liste) : null, b = el.bulle;
    b.style.cssText = 'visibility:hidden';
    b.classList.remove('cache', 'haut', 'bas', 'flottante', 'sous-titre');
    // Mode tournage : la bulle devient un sous-titre fixe en bas de l'écran
    if (Etat.option('tournage')) { b.classList.add('sous-titre'); b.style.visibility = ''; return; }
    if (vw < 600) {
      b.classList.add(r && (r.top + r.height / 2) > vh / 2 ? 'haut' : 'bas');
      b.style.visibility = ''; return;
    }
    b.classList.add('flottante');
    var bw = b.offsetWidth, bh = b.offsetHeight, x, y;
    if (!r) { x = (vw - bw) / 2; y = vh - bh - 24; }
    else if (r.right + g + bw <= vw - g) { x = r.right + g; y = r.top; }
    else if (r.left - g - bw >= g) { x = r.left - g - bw; y = r.top; }
    else if (r.bottom + g + bh <= vh - g) { x = r.left; y = r.bottom + g; }
    else if (r.top - g - bh >= g) { x = r.left; y = r.top - g - bh; }
    else { x = vw - bw - g; y = vh - bh - g; }
    x = Math.max(g, Math.min(x, vw - bw - g)); y = Math.max(g, Math.min(y, vh - bh - g));
    b.style.cssText = 'left:' + x + 'px;top:' + y + 'px';
  }

  function remplirBulle(p, i) {
    var a = DG.actif, vis = a.visibles, n = vis.indexOf(i) + 1, total = vis.length, dernier = n === total;
    var fil = (p.chapitre ? p.chapitre + ' · ' : '') + a.def.titre + ' · étape ' + n + ' sur ' + total;
    el.bulle.innerHTML = '<div class="demo-tete"><span class="demo-fil">' + F.e(fil) + '</span><button type="button" class="demo-x" data-demo="quitter" aria-label="Quitter la démonstration guidée">' + UI.ic('x') + '</button></div>' +
      '<div class="demo-barre" role="progressbar" aria-valuemin="0" aria-valuemax="' + total + '" aria-valuenow="' + n + '"><span style="width:' + Math.round(100 * n / total) + '%"></span></div>' +
      '<div aria-live="polite"><h2 class="demo-titre">' + F.e(p.titre) + '</h2>' + (p.texte ? '<p class="demo-texte">' + F.e(p.texte) + '</p>' : '') + '</div>' +
      (p.voix ? '<p class="demo-voix" id="demo-voix"><span class="demo-voix-lib">Voix off</span> « ' + F.e(p.voix) + ' »</p>' : '') +
      '<div class="demo-pied"><button type="button" class="btn btn-secondaire btn-petit" data-demo="prec"' + (n === 1 ? ' disabled' : '') + '>Précédent</button>' +
      '<button type="button" class="btn btn-secondaire btn-petit" data-demo="lecture" aria-pressed="' + !!a.auto + '">' + UI.ic(a.auto ? 'pause' : 'play', 'ic-sm') + (a.auto ? ' Pause' : ' Lecture') + '</button>' +
      '<button type="button" class="btn btn-primaire btn-petit" data-demo="suiv">' + (dernier ? 'Terminer' : 'Suivant') + '</button></div>' +
      '<p class="demo-aide">← → naviguer · Espace lecture automatique · Échap quitter · T mode tournage</p>';
    el.bulle.classList.toggle('derniere', dernier);
  }

  // Affiche l'étape i : bulle, halo, lecture automatique
  async function montrer(i) {
    var a = DG.actif, p = a.def.pas[i];
    a.idx = i; a.cible = p.cible;
    if (p.carte) { afficherCarte(p.carte); a.cible = null; el.halo.classList.add('cache'); }
    else {
      masquerCarte();
      var cible = p.cible ? await H.attendre(p.cible, 3000) : null;
      if (cible) amener(cible);
    }
    // Un champ resté actif capterait les flèches du clavier : on le quitte pour que la navigation fonctionne
    var actif = document.activeElement;
    if (actif && /^(INPUT|TEXTAREA|SELECT)$/.test(actif.tagName)) actif.blur();
    remplirBulle(p, i);
    poserHalo(); poserBulle();
    clearTimeout(minuteur);
    if (a.auto) minuteur = setTimeout(function () { DG.suivant(); }, (p.d || 6) * 1000);
  }
  function afficherCarte(c) {
    el.carte.innerHTML = '<div class="demo-carte-corps"><p class="demo-carte-sur">' + F.e(c.sur || '') + '</p><h2>' + F.e(c.titre) + '</h2>' +
      (c.items ? '<ul class="demo-carte-liste">' + c.items.map(function (t, k) { return '<li style="--r:' + ((k % 2 ? 1 : -1) * (2 + k)) + 'deg">' + F.e(t) + '</li>'; }).join('') + '</ul>' : '') +
      (c.sous ? '<p class="demo-carte-sous">' + F.e(c.sous) + '</p>' : '') + '</div>';
    el.carte.classList.remove('cache');
  }
  function masquerCarte() { if (el.carte) el.carte.classList.add('cache'); }

  /* ---------- Commandes ---------- */
  DG.occupe = false;
  async function executer(fn) {
    if (DG.occupe || !DG.actif) return;
    DG.occupe = true;
    try { await fn(); } catch (e) { if (e !== ANNULE) console.error(e); }
    DG.occupe = false;
  }
  DG.suivant = function () {
    executer(async function () {
      var a = DG.actif, pas = a.def.pas, i = a.idx, suite = a.visibles[a.visibles.indexOf(i) + 1];
      clearTimeout(minuteur);
      if (suite === undefined) {
        DG.quitter();
        if (!Etat.option('tournage')) UI.toast('Démonstration terminée. « Réinitialiser la démo » remet les données à zéro.', { type: 'succes' });
        return;
      }
      if (pas[i].faire) await pas[i].faire(H);
      for (var j = i + 1; j < suite; j++) await jouer(pas[j]);
      await entrer(pas[suite]);
      await montrer(suite);
    });
  };
  DG.precedent = function () {
    var a = DG.actif; if (!a) return;
    var k = a.visibles[a.visibles.indexOf(a.idx) - 1];
    if (k === undefined) return;
    DG.rejouerA(k);
  };
  // Remise à zéro du journal puis rechargement : la démo repart d'un état propre et rejoue les étapes sans animation
  DG.rejouerA = function (k) {
    var a = DG.actif;
    Etat.donnees.journal = []; Etat.donnees.options.erreur = null; Etat.donnees.options.lotSansMandat = false; Etat.enregistrer();
    history.replaceState(null, '', location.href.split('#')[0] + '#/demo/' + a.def.id + '?pas=' + k + (a.auto ? '&auto=1' : ''));
    location.reload();
  };
  DG.lecture = function () {
    var a = DG.actif; if (!a) return;
    a.auto = !a.auto; clearTimeout(minuteur);
    remplirBulle(a.def.pas[a.idx], a.idx);
    poserBulle();
    if (a.auto) minuteur = setTimeout(function () { DG.suivant(); }, (a.def.pas[a.idx].d || 6) * 1000);
  };
  DG.quitter = function () {
    clearTimeout(minuteur);
    DG.jeton++; DG.actif = null; DG.occupe = false; H.vite = false;
    enleverSurcouche();
    document.body.classList.remove('demo-active');
  };

  /* ---------- Entrée par l'adresse #/demo/{parcours} ---------- */
  DG.entrer = function (r) {
    var def = DG.parcours[r.params[0]];
    if (!def) { location.replace('#/accueil'); setTimeout(DG.liste, 200); return; }
    var k = Math.max(0, parseInt(r.query.pas || '0', 10) || 0);
    // État de départ propre : rechargement si des modifications sont déjà dans le journal
    if (Etat.donnees.journal.length) { Etat.donnees.journal = []; Etat.enregistrer(); location.reload(); return; }
    Etat.donnees.options.erreur = null; Etat.donnees.options.lotSansMandat = false; Etat.enregistrer();
    DG.quitter();
    var visibles = []; def.pas.forEach(function (p, i) { if (!p.muet) visibles.push(i); });
    DG.actif = { def: def, idx: 0, visibles: visibles, auto: r.query.auto === '1', jeton: ++DG.jeton, cible: null };
    document.body.classList.add('demo-active');
    creerSurcouche();
    if (def.role) Etat.definirRole(def.role);
    DG.occupe = true;
    (async function () {
      try {
        H.vite = k > 0;
        for (var i = 0; i < k; i++) await jouer(def.pas[i]);
        H.vite = false;
        await entrer(def.pas[k]);
        await montrer(k);
      } catch (e) { if (e !== ANNULE) console.error(e); }
      DG.occupe = false;
    })();
  };

  /* ---------- Liste des démonstrations ---------- */
  DG.liste = function () {
    UI.fermerTout();
    var items = DG.ordre.map(function (id) {
      var d = DG.parcours[id];
      return '<a class="menu-item" href="#/demo/' + id + '">' + UI.ic('play') + '<span class="menu-texte"><span class="menu-nom">' + F.e(d.titre) + '</span><span class="menu-sous">' + F.e(d.resume) + '</span></span></a>';
    }).join('');
    UI.modale({
      titre: 'Démonstration guidée',
      corps: '<p>Une bulle explique chaque étape et met l’élément en évidence. La démonstration part toujours de l’état initial des données. Clavier : flèche droite suivant, flèche gauche précédent, Espace lecture automatique, Échap pour quitter.</p><div class="demo-liste">' + items + '</div>' +
        '<p class="champ-aide">Pour une vidéo, activez le mode tournage avec la touche T (curseur agrandi, saisie simulée, sous-titres).</p>',
      actions: [{ lib: 'Fermer', v: 'secondaire', focus: true }]
    });
  };
  App.action('demo-liste', DG.liste);
  App.action('tournage', function () { UI.fermerTout(); DG.basculerTournage(); });

  /* ---------- Mode tournage ---------- */
  DG.appliquerClasse = function () { document.body.classList.toggle('tournage', !!Etat.option('tournage')); };
  function enCadre() { return window.top !== window.self; }
  function recharger() { if (enCadre()) window.parent.postMessage('immo-cadre-recharger', '*'); else location.reload(); }
  DG.basculerTournage = function () {
    var on = !Etat.option('tournage'), cadre = Etat.option('cadre');
    Etat.definirOption('tournage', on);
    DG.appliquerClasse();
    if (cadre && cadre !== 'libre') { recharger(); return; }
    if (!on) { var c = document.getElementById('demo-curseur'); if (c) c.classList.remove('vu'); }
    else UI.toast('Mode tournage activé. Touche T pour le quitter.', { type: 'info', duree: 2500 });
    if (DG.actif) { poserHalo(); poserBulle(); }
  };
  // Changement du format de fenêtre dans les options : recharge si le tournage est actif
  DG.changerCadre = function (c) {
    var avant = Etat.option('cadre') || 'libre';
    Etat.definirOption('cadre', c);
    if (Etat.option('tournage') && avant !== c) recharger();
  };

  function curseurEl() {
    var c = document.getElementById('demo-curseur');
    if (!c) {
      c = document.createElement('div'); c.id = 'demo-curseur'; c.setAttribute('aria-hidden', 'true');
      c.innerHTML = '<svg viewBox="0 0 24 24" width="36" height="36"><path d="M5 3l14 8-6 2 4 7-3 1.5-4-7.2-5 4z" fill="#111" stroke="#fff" stroke-width="1.6" stroke-linejoin="round"/></svg>';
      document.body.appendChild(c);
    }
    return c;
  }
  function deplacerCurseur(x, y) {
    cur.x = x; cur.y = y;
    var c = curseurEl(); c.classList.add('vu');
    c.style.transform = 'translate(' + (x - 6) + 'px,' + (y - 4) + 'px)';
  }
  function cercle(x, y) {
    var n = document.createElement('div'); n.className = 'demo-clic'; n.setAttribute('aria-hidden', 'true');
    n.style.left = x + 'px'; n.style.top = y + 'px';
    document.body.appendChild(n); setTimeout(function () { n.remove(); }, 700);
  }

  document.addEventListener('mousemove', function (ev) {
    if (!Etat.option('tournage')) return;
    var c = curseurEl(); c.classList.add('suit'); deplacerCurseur(ev.clientX, ev.clientY);
  }, { passive: true });
  document.addEventListener('mousedown', function (ev) { if (Etat.option('tournage')) cercle(ev.clientX, ev.clientY); }, true);

  /* ---------- Clavier ---------- */
  document.addEventListener('keydown', function (ev) {
    if (ev.ctrlKey || ev.metaKey || ev.altKey) return;
    var cible = ev.target, saisie = cible && /^(INPUT|TEXTAREA|SELECT)$/.test(cible.tagName);
    var couche = document.querySelector('.modale-fond, .msg-fond, .panneau-fond, .feuille-fond, .popover-fond');
    if ((ev.key === 't' || ev.key === 'T') && !saisie) { ev.preventDefault(); DG.basculerTournage(); return; }
    if (!DG.actif || saisie) return;
    if (ev.key === 'ArrowRight') { ev.preventDefault(); DG.suivant(); }
    else if (ev.key === 'ArrowLeft') { ev.preventDefault(); DG.precedent(); }
    else if (ev.key === ' ') { ev.preventDefault(); ev.stopPropagation(); DG.lecture(); }
    else if (ev.key === 'Escape' && !couche) { DG.quitter(); }
  }, true);

  window.addEventListener('resize', function () { if (DG.actif) { poserHalo(); poserBulle(); } });
  window.addEventListener('scroll', function () { if (DG.actif) poserHalo(); }, true);
  // Le contenu de l'écran peut changer sous la bulle (panneau qui s'ouvre) : le halo suit
  setInterval(function () { if (DG.actif && !DG.occupe) poserHalo(); }, 400);

  /* ---------- Cadre de taille fixe (1440 × 900 ou 390 × 844) ---------- */
  // Le cadre est un iframe de la même page : les points de rupture s'appliquent à la taille réelle du cadre
  DG.cadrer = function () {
    var c = Etat.option('cadre');
    if (enCadre() || !Etat.option('tournage') || !c || c === 'libre') return false;
    var L = c === 'mobile' ? 390 : 1440, Ha = c === 'mobile' ? 844 : 900;
    var src = location.href.split('#')[0].split('?')[0] + '?interne=1' + location.hash;
    document.body.className = 'cadre-page';
    document.body.innerHTML = '<div class="cadre-zone"><iframe id="cadre-app" title="Maquette Immo Suite" src="' + src + '" width="' + L + '" height="' + Ha + '"></iframe></div>';
    var f = document.getElementById('cadre-app');
    function ajuster() { var s = Math.min(1, window.innerWidth / L, window.innerHeight / Ha); f.style.transform = 'scale(' + s + ')'; f.style.marginLeft = (-(L * (1 - s)) / 2) + 'px'; f.style.marginTop = (-(Ha * (1 - s)) / 2) + 'px'; }
    ajuster(); window.addEventListener('resize', ajuster);
    f.addEventListener('load', function () { try { f.contentWindow.focus(); } catch (e) { /* ignoré */ } });
    window.addEventListener('message', function (ev) { if (ev.data === 'immo-cadre-recharger') location.reload(); });
    return true;
  };
})();
