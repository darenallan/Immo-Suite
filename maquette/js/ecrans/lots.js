/*
 * S06 : Biens et lots · S07 : Fiche lot (onglets Aperçu, Baux, Paiements, Travaux, Documents).
 * Parcours 5 : filtre « Ouaga 2000 » → Appartement A3 → chronologie des travaux → pièces jointes → compte rendu d'octobre.
 * Le statut de l'échéance du mois et les soldes viennent du jeu de données (échéances précalculées).
 */
(function () {
  var F = window.Fmt, UI = window.UI, Data = window.Data, App = window.App, Roles = window.Roles;
  var f = { q: '', quartier: '', statut: '', usage: '', mandant: '' };
  var STATUT_LOT = { occupe: ['Occupé', 'users'], vacant: ['Vacant', 'home'], travaux: ['En travaux', 'wrench'], archive: ['Archivé', 'ban'] };
  var STATUT_BAIL = { actif: ['Actif', 'check'], preavis: ['En préavis', 'clock'], brouillon: ['Brouillon', 'pen'], termine: ['Terminé', 'ban'] };
  function D() { return Data.org(); }
  function lecture(ctx) { return Roles.droit(ctx.role.id, 'lots') !== 'E'; }

  function badgeLot(s) { var m = STATUT_LOT[s]; return UI.neutre(m[0], m[1]); }
  function badgeBail(s) { var m = STATUT_BAIL[s]; return UI.neutre(m[0], m[1], s === 'brouillon' ? 'badge-pointille' : ''); }
  // Statut d'affichage de l'échéance du mois : « en retard » prime si une échéance est en retard (DP-13)
  function echeanceMois(bailId) {
    var d = D(), es = d.echeances.filter(function (e) { return e.bailId === bailId; }), retard = es.filter(function (e) { return e.statut === 'en_retard'; });
    var nov = es.filter(function (e) { return e.periode === d.meta.moisReference; })[0];
    return { retard: retard, nov: nov, html: retard.length ? UI.statut('en_retard', F.jours(Math.max.apply(null, retard.map(function (e) { return e.ancienneteJours; })))) : nov ? (nov.statut === 'due' ? UI.statut('due', 'aujourd’hui') : UI.statut(nov.statut)) : '<span class="muted">—</span>' };
  }

  /* ---------- S06 ---------- */
  function lotsFiltres() {
    var n = F.norm(f.q);
    return D().lots.filter(function (l) {
      var bien = Data.bien(l.bienId), b = l.bailId ? Data.bail(l.bailId) : null;
      if (f.quartier && bien.quartier !== f.quartier) return false;
      if (f.statut && l.statut !== f.statut) return false;
      if (f.usage && l.usage !== f.usage) return false;
      if (f.mandant && l.mandantId !== f.mandant) return false;
      if (n && F.norm(l.nom + ' ' + bien.nom + ' ' + bien.quartier + ' ' + (b ? Data.nomLocataires(b.locataireIds) : '')).indexOf(n) < 0) return false;
      return true;
    });
  }
  function htmlLots(liste) {
    if (!liste.length) return UI.etatVide({ ic: 'search', titre: 'Aucun lot ne correspond', texte: 'Aucun résultat avec ces filtres.', action: UI.btn('Effacer les filtres', { v: 'secondaire', action: 'lots-effacer' }) });
    var lignes = liste.map(function (l) {
      var bien = Data.bien(l.bienId), b = l.bailId ? Data.bail(l.bailId) : null, em = b ? echeanceMois(b.id) : null;
      return '<tr class="ligne-cliquable" tabindex="0" data-action="lot-ouvrir" data-id="' + l.id + '"><td><span class="gras">' + F.e(l.nom) + '</span></td><td>' + F.e(bien.nom) + '<br><span class="muted" style="font-size:var(--text-sm)">' + F.e(bien.quartier) + '</span></td><td>' + UI.usage(l.usage) + '</td><td>' + badgeLot(l.statut) + '</td>' +
        '<td>' + (b ? F.e(Data.nomLocataires(b.locataireIds)) : '<span class="muted">—</span>') + '</td><td class="droite">' + F.montant(b ? b.loyer + b.charges : l.loyerReference) + (b ? '' : '<br><span class="muted" style="font-size:var(--text-sm)">de référence</span>') + '</td><td>' + (em ? em.html : '<span class="muted">—</span>') + '</td></tr>';
    }).join('');
    var cartes = liste.map(function (l) {
      var bien = Data.bien(l.bienId), b = l.bailId ? Data.bail(l.bailId) : null, em = b ? echeanceMois(b.id) : null;
      return '<li><button type="button" class="carte-impaye" data-action="lot-ouvrir" data-id="' + l.id + '"><span class="ci-haut"><span class="liste-gauche" style="flex:1"><span class="liste-titre">' + F.e(l.nom) + '</span><span class="liste-sous">' + F.e(bien.nom) + ' · ' + F.e(bien.quartier) + '</span></span><span class="liste-droite"><span class="liste-montant">' + F.montant(b ? b.loyer + b.charges : l.loyerReference) + '</span>' + (em ? em.html : badgeLot(l.statut)) + '</span></span>' +
        '<span class="ci-bas">' + (b ? '<span>' + UI.ic('user', 'ic-sm') + ' ' + F.e(Data.nomLocataires(b.locataireIds)) + '</span>' : '') + badgeLot(l.statut) + UI.usage(l.usage) + '</span></button></li>';
    }).join('');
    return '<div class="tab-bureau tableau-cadre"><table class="tableau"><thead><tr><th scope="col">Lot</th><th scope="col">Bien</th><th scope="col">Usage</th><th scope="col">Statut du lot</th><th scope="col">Locataire</th><th scope="col" class="droite">Loyer</th><th scope="col">Échéance du mois</th></tr></thead><tbody>' + lignes + '</tbody></table></div><ul class="tab-mobile">' + cartes + '</ul>';
  }
  function htmlBiens() {
    var d = D(), liste = d.biens.filter(function (b) { return (!f.quartier || b.quartier === f.quartier) && (!f.mandant || b.mandantId === f.mandant); });
    return '<ul class="biens-grille">' + liste.map(function (b) {
      var lots = d.lots.filter(function (l) { return l.bienId === b.id; }), occ = lots.filter(function (l) { return l.statut === 'occupe'; }).length;
      return '<li class="carte"><div class="gras">' + F.e(b.nom) + '</div><div class="secondaire" style="font-size:var(--text-sm)">' + UI.ic('map-pin', 'ic-sm') + ' ' + F.e(b.quartier) + (d.organisation.plan === 'agence' ? ' · ' + F.e(Data.mandant(b.mandantId).nom) : '') + '</div><div style="margin:8px 0">' + lots.length + (lots.length > 1 ? ' lots' : ' lot') + ' · ' + occ + ' occupé' + (occ > 1 ? 's' : '') + '</div>' + UI.btn('Voir les lots', { v: 'secondaire', petit: true, action: 'bien-lots', data: ' data-id="' + b.id + '"' }) + '</li>';
    }).join('') + '</ul>';
  }
  function filtresHtml(ctx) {
    var d = D(), quartiers = d.biens.map(function (b) { return b.quartier; }).filter(function (q, i, a) { return a.indexOf(q) === i; }).sort();
    var sel = function (id, lib, opts, val) { return '<div class="champ"><label class="champ-etiquette" for="' + id + '">' + lib + '</label><select class="saisie" id="' + id + '">' + opts.map(function (o) { return '<option value="' + o[0] + '"' + (o[0] === val ? ' selected' : '') + '>' + F.e(o[1]) + '</option>'; }).join('') + '</select></div>'; };
    var h = '<div class="filtres filtres-imp"><div class="champ" style="flex:2 1 220px"><label class="champ-etiquette" for="l-q">Rechercher</label><div style="position:relative"><span style="position:absolute;left:12px;top:50%;transform:translateY(-50%);color:var(--color-text-muted)">' + UI.ic('search') + '</span><input class="saisie" id="l-q" type="search" style="padding-left:40px" value="' + F.e(f.q) + '" placeholder="Lot, bien, locataire" autocomplete="off"></div></div>' +
      sel('l-quartier', 'Quartier', [['', 'Tous les quartiers']].concat(quartiers.map(function (q) { return [q, q]; })), f.quartier) + sel('l-statut', 'Statut du lot', [['', 'Tous'], ['occupe', 'Occupé'], ['vacant', 'Vacant'], ['travaux', 'En travaux']], f.statut) + sel('l-usage', 'Usage', [['', 'Tous les usages'], ['habitation', 'Habitation'], ['commercial', 'Commercial'], ['mixte', 'Mixte']], f.usage);
    if (d.organisation.plan === 'agence') h += sel('l-mandant', 'Mandant', [['', 'Tous les mandants']].concat(d.mandants.map(function (m) { return [m.id, m.nom]; })), f.mandant);
    return h + '</div>';
  }
  function majLots() {
    var vue = document.getElementById('lots-vue').getAttribute('data-vue'), l = lotsFiltres();
    document.getElementById('lots-liste').innerHTML = vue === 'biens' ? htmlBiens() : htmlLots(l);
    document.getElementById('lots-compte').textContent = vue === 'biens' ? '' : l.length + (l.length > 1 ? ' lots' : ' lot');
    var p = []; if (f.quartier) p.push(['quartier', f.quartier]); if (f.statut) p.push(['statut', STATUT_LOT[f.statut][0]]); if (f.usage) p.push(['usage', F.majuscule(f.usage)]); if (f.mandant) p.push(['mandant', Data.mandant(f.mandant).nom]);
    document.getElementById('lots-puces').innerHTML = p.length ? '<div class="puces-filtres">' + p.map(function (x) { return '<button type="button" class="puce-filtre" data-action="lots-retirer" data-k="' + x[0] + '">' + F.e(x[1]) + UI.ic('x', 'ic-sm') + '<span class="sr-only"> : retirer ce filtre</span></button>'; }).join('') + '<button type="button" class="lien-bouton" data-action="lots-effacer">Effacer tout</button></div>' : '';
  }
  function syncSel() { [['l-q', 'q'], ['l-quartier', 'quartier'], ['l-statut', 'statut'], ['l-usage', 'usage'], ['l-mandant', 'mandant']].forEach(function (x) { var e = document.getElementById(x[0]); if (e) e.value = f[x[1]]; }); }

  App.action('lots-effacer', function () { f = { q: '', quartier: '', statut: '', usage: '', mandant: '' }; syncSel(); majLots(); });
  App.action('lots-retirer', function (el) { f[el.getAttribute('data-k')] = ''; syncSel(); majLots(); });
  App.action('lot-ouvrir', function (el) { App.aller('/lot/' + el.getAttribute('data-id')); });
  App.action('bien-lots', function (el) { f.quartier = ''; f.q = Data.bien(el.getAttribute('data-id')).nom; App.aller('/lots'); });
  App.action('bien-creer', function () {
    var d = D(), q = d.biens.map(function (b) { return b.quartier; }).filter(function (x, i, a) { return a.indexOf(x) === i; }).sort();
    UI.modale({ titre: 'Créer un bien', corps: '<div class="champ"><label class="champ-etiquette" for="nb-nom">Nom du bien (obligatoire)</label><input class="saisie" id="nb-nom" autocomplete="off" placeholder="Exemple : Cour Kaboré"><div class="champ-erreur cache" id="nb-err" role="alert"></div></div><div class="champ"><label class="champ-etiquette" for="nb-q">Quartier</label><select class="saisie" id="nb-q">' + q.map(function (x) { return '<option>' + F.e(x) + '</option>'; }).join('') + '</select></div><p class="champ-aide" style="margin-top:12px">Une maison seule crée automatiquement 1 bien et 1 lot.</p>',
      actions: [{ lib: 'Annuler', v: 'secondaire' }, { lib: 'Créer le bien', v: 'primaire', focus: true, fn: function (fermer) { if (!document.getElementById('nb-nom').value.trim()) { var e = document.getElementById('nb-err'); e.classList.remove('cache'); e.textContent = 'Indiquez le nom du bien.'; document.getElementById('nb-nom').focus(); return; } fermer(); UI.toast('Action de démonstration. Le bien n’est pas ajouté au portefeuille.', { type: 'info' }); } }] });
  });

  App.enregistrer('lots', {
    meta: function (ctx) {
      var l = lecture(ctx);
      return { titre: 'Biens et lots', sous: l ? 'Lecture seule : vous consultez le patrimoine sans le modifier.' : 'Un lot, un appartement, une boutique : leur situation du mois en un coup d’œil.', repere: { type: 'v1' }, actions: l ? '' : UI.btn('Créer un bien', { v: 'secondaire', ic: 'plus', action: 'bien-creer' }) + UI.btn('Importer mon portefeuille', { v: 'tertiaire', ic: 'upload', href: '#/import' }) };
    },
    rendre: function (ctx) {
      var vue = ctx.query.vue === 'biens' ? 'biens' : 'lots', q = ctx.query.quartier;
      if (q !== undefined) f = { q: '', quartier: q, statut: '', usage: '', mandant: '' };
      var onglets = '<div class="onglets" role="tablist"><a class="onglet" role="tab" href="#/lots" aria-selected="' + (vue === 'lots') + '">Lots</a><a class="onglet" role="tab" href="#/lots?vue=biens" aria-selected="' + (vue === 'biens') + '">Biens</a></div>';
      var liste = lotsFiltres();
      return onglets + '<section class="carte" aria-labelledby="t-lots"><h2 id="t-lots" class="sr-only">' + (vue === 'biens' ? 'Liste des biens' : 'Liste des lots') + '</h2>' + filtresHtml(ctx) + '<div id="lots-puces"></div><p class="secondaire" style="margin:8px 0 12px" id="lots-compte" aria-live="polite"></p><div id="lots-vue" data-vue="' + vue + '"><div id="lots-liste">' + (vue === 'biens' ? htmlBiens() : htmlLots(liste)) + '</div></div></section>';
    },
    apres: function (ctx, rac) {
      [['l-quartier', 'quartier'], ['l-statut', 'statut'], ['l-usage', 'usage'], ['l-mandant', 'mandant']].forEach(function (x) { var e = rac.querySelector('#' + x[0]); if (e) e.addEventListener('change', function () { f[x[1]] = e.value; majLots(); }); });
      var q = rac.querySelector('#l-q'), t = null; q.addEventListener('input', function () { clearTimeout(t); t = setTimeout(function () { f.q = q.value; majLots(); }, 200); });
      rac.addEventListener('keydown', function (ev) { if (ev.key === 'Enter' && ev.target.classList && ev.target.classList.contains('ligne-cliquable')) App.aller('/lot/' + ev.target.getAttribute('data-id')); });
      majLots();
    }
  });

  /* ---------- S07 ---------- */
  function ongletsLot(lot, actif, n) {
    var t = [['apercu', 'Aperçu'], ['baux', 'Baux', n.baux], ['paiements', 'Paiements', n.paiements], ['travaux', 'Travaux', n.travaux], ['documents', 'Documents', n.documents]];
    return '<div class="onglets" role="tablist">' + t.map(function (x) { return '<a class="onglet" role="tab" href="#/lot/' + lot.id + (x[0] === 'apercu' ? '' : '?onglet=' + x[0]) + '" aria-selected="' + (x[0] === actif) + '">' + x[1] + (x[2] !== undefined ? ' <span class="badge badge-neutre" style="min-height:22px;padding:0 8px">' + x[2] + '</span>' : '') + '</a>'; }).join('') + '</div>';
  }
  function carteBail(b) {
    var loc = b.locataireIds.map(function (id) { var t = Data.locataire(id); return '<a href="#/locataire/' + id + '">' + F.e(t.nom) + '</a>'; }).join(' et ');
    var commercial = b.type === 'commercial' ? '<div><dt>Conditions commerciales</dt><dd>' + F.e(b.commercial.duree) + ' · ' + F.e(b.commercial.indexation) + '</dd></div>' + (b.tvaIncluse ? '<div><dt>TVA</dt><dd>TVA incluse</dd></div>' : '') : '';
    var preavis = b.preavis ? '<div class="bandeau bandeau-info" style="margin-top:12px">' + UI.ic('clock') + '<span>Préavis noté le ' + F.date(b.preavis.saisiLe) + ' : départ prévu le ' + F.date(b.preavis.dateDepart) + '. Les échéances continuent jusqu’au départ.</span></div>' : '';
    var depot = b.depotAlerte ? '<div class="bandeau bandeau-avert" style="margin-top:12px">' + UI.ic('alert-triangle') + '<span>Le dépôt de garantie dépasse deux mois de loyer. Vérifiez qu’il respecte la loi sur le bail d’habitation. Vous pouvez continuer.</span></div>' : '';
    return '<dl class="def-liste" style="margin:0"><div><dt>Locataire' + (b.locataireIds.length > 1 ? 's (colocation)' : '') + '</dt><dd>' + loc + '</dd></div><div><dt>Statut du bail</dt><dd>' + badgeBail(b.statut) + '</dd></div><div><dt>Du / au</dt><dd>' + F.date(b.dateDebut) + ' au ' + F.date(b.dateFin) + '</dd></div><div><dt>Loyer</dt><dd class="num">' + F.montant(b.loyer) + '</dd></div><div><dt>Charges</dt><dd class="num">' + F.montant(b.charges) + '</dd></div><div><dt>Jour d’échéance</dt><dd>le ' + b.jourEcheance + ' de chaque mois</dd></div><div><dt>Dépôt de garantie</dt><dd class="num">' + F.montant(b.depotGarantie) + '</dd></div>' + commercial + '</dl>' + depot + preavis;
  }

  function ongletApercu(ctx, lot, bien, b) {
    var d = D(), agence = d.organisation.plan === 'agence', em = b ? echeanceMois(b.id) : null, peut = !lecture(ctx);
    var blocBail = b ? '<section class="carte" aria-labelledby="t-bail"><div class="carte-entete"><h2 id="t-bail" class="carte-titre">' + UI.ic('file-text') + 'Bail en cours</h2></div>' + carteBail(b) + '</section>'
      : '<section class="carte">' + UI.etatVide({ ic: 'home', titre: lot.statut === 'travaux' ? 'Lot en travaux' : 'Aucun bail en cours', texte: lot.statut === 'travaux' ? 'Remettez le lot à « vacant » pour activer un bail.' : 'Ce lot est vacant. Créez un bail pour générer ses échéances.', action: peut ? UI.btn(lot.statut === 'travaux' ? 'Retirer « en travaux »' : 'Créer un bail', { v: 'primaire', action: 'demo' }) : '' }) + '</section>';
    var blocEch = '';
    if (b) {
      var nov = em.nov, lignes = em.retard.map(function (e) { return '<li><div class="liste-gauche"><span class="liste-titre">' + F.mois(e.periode) + '</span><span class="liste-sous">Échéance du ' + F.date(e.dateEcheance) + '</span></div><div class="liste-droite"><span class="liste-montant">' + F.montant(e.reste) + '</span>' + UI.statut('en_retard', F.jours(e.ancienneteJours)) + '</div></li>'; }).join('');
      blocEch = '<section class="carte" aria-labelledby="t-ech"><div class="carte-entete"><h2 id="t-ech" class="carte-titre">' + UI.ic('calendar') + 'Échéance du mois</h2>' + (nov ? (nov.statut === 'due' ? UI.statut('due', 'aujourd’hui') : UI.statut(nov.statut)) : '') + '</div>' +
        (nov ? '<dl class="def-liste" style="margin:0"><div><dt>' + F.majuscule(F.mois(nov.periode)) + ' · échéance du ' + F.date(nov.dateEcheance) + '</dt><dd class="num">' + F.montant(nov.montant) + '</dd></div><div><dt>Payé</dt><dd class="num">' + F.montant(nov.paye) + '</dd></div><div><dt>Reste dû</dt><dd class="num">' + F.montant(nov.reste) + '</dd></div></dl>' : '') +
        (lignes ? '<h3 style="margin:16px 0 8px">Échéances en retard</h3><ul class="liste">' + lignes + '</ul>' : '') + '</section>';
    }
    var blocMandat = agence ? '<section class="carte" aria-labelledby="t-mand"><div class="carte-entete"><h2 id="t-mand" class="carte-titre">' + UI.ic('briefcase') + 'Mandat</h2></div><dl class="def-liste" style="margin:0"><div><dt>Mandant</dt><dd>' + F.e(Data.mandant(lot.mandantId).nom) + '</dd></div><div><dt>Commission</dt><dd>' + F.pourcent(d.mandats.filter(function (m) { return m.id === lot.mandatId; })[0].taux) + '</dd></div><div><dt>Loyer de référence</dt><dd class="num">' + F.montant(lot.loyerReference) + '</dd></div>' + (lot.activiteAutorisee ? '<div><dt>Activité autorisée</dt><dd>' + F.e(lot.activiteAutorisee) + '</dd></div>' : '') + '</dl></section>' : '';
    var actions = '';
    if (peut) {
      actions = '<section class="carte pas-imprimer" aria-labelledby="t-act"><h2 id="t-act" class="sr-only">Actions</h2><div class="btn-groupe">' + UI.btn('Ajouter une intervention', { v: 'primaire', ic: 'plus', action: 'trv-ajouter', data: ' data-lot="' + lot.id + '"' }) +
        (lot.statut === 'vacant' ? UI.btn('Mettre en travaux', { v: 'secondaire', ic: 'wrench', action: 'demo' }) + UI.btn('Créer un bail', { v: 'secondaire', ic: 'file-text', action: 'demo' }) : '') +
        UI.btn('Archiver le lot', { v: 'tertiaire', ic: 'ban', action: 'lot-archiver', data: ' data-id="' + lot.id + '"' }) + '</div><div id="lot-refus" aria-live="polite"></div></section>';
    }
    return '<div class="lot-grille"><div class="lot-col">' + blocBail + blocEch + '</div><div class="lot-col">' + blocMandat + actions + '</div></div>';
  }
  function ongletPaiements(lot, b) {
    if (!b) return '<section class="carte">' + UI.etatVide({ ic: 'receipt', titre: 'Aucun paiement', texte: 'Ce lot n’a pas de bail en cours.' }) + '</section>';
    var d = D(), es = d.echeances.filter(function (e) { return e.bailId === b.id; }).sort(function (a, c) { return c.dateEcheance.localeCompare(a.dateEcheance); });
    var pays = d.paiements.filter(function (p) { return p.lotId === lot.id && p.type === 'paiement'; }).sort(function (a, c) { return (c.dateSaisie + c.heure).localeCompare(a.dateSaisie + a.heure); });
    var ech = '<ul class="liste">' + es.map(function (e) { return '<li><div class="liste-gauche"><span class="liste-titre">' + F.majuscule(F.mois(e.periode)) + '</span><span class="liste-sous">Échéance du ' + F.date(e.dateEcheance) + ' · dû ' + F.montant(e.montant) + '</span></div><div class="liste-droite"><span class="liste-montant">' + F.montant(e.paye) + '</span>' + (e.statut === 'en_retard' ? UI.statut('en_retard', F.jours(e.ancienneteJours)) : UI.statut(e.statut)) + '</div></li>'; }).join('') + '</ul>';
    var pl = pays.length ? '<ol class="chrono">' + pays.map(function (p) {
      var q = p.quittanceId ? Data.quittance(p.quittanceId) : null;
      return '<li class="chrono-item"><div class="chrono-date"><span class="gras">' + F.date(p.date) + '</span></div><a class="chrono-corps" href="#/paiements?ouvrir=' + p.id + '"><span class="chrono-haut"><span class="gras num">' + F.montant(p.montant) + '</span>' + (p.statut === 'corrige' ? UI.statut('corrige') : UI.statut('valide')) + '</span><span class="chrono-bas"><span>' + Data.modeLibelle(p.mode) + '</span><span>' + F.e(Data.nomLocataires(p.locataireIds)) + '</span>' + (q ? '<span>Quittance ' + F.e(q.numero) + '</span>' : '') + '</span></a></li>';
    }).join('') + '</ol>' : '<p class="secondaire">Aucun paiement sur cette période.</p>';
    return '<div class="lot-grille"><section class="carte" aria-labelledby="t-pl"><h2 id="t-pl" class="carte-titre" style="margin-bottom:12px">' + UI.ic('clock') + 'Chronologie des paiements</h2>' + pl + '</section><section class="carte" aria-labelledby="t-ee"><h2 id="t-ee" class="carte-titre" style="margin-bottom:12px">' + UI.ic('calendar') + 'Échéancier</h2>' + ech + '</section></div>';
  }

  App.action('lot-archiver', function (el) {
    var lot = Data.lot(el.getAttribute('data-id')), z = document.getElementById('lot-refus');
    if (lot.bailId) { z.innerHTML = '<div class="bandeau bandeau-erreur" style="margin-top:12px">' + UI.ic('alert-circle') + '<span>Ce lot a un bail actif. Terminez le bail avant d’archiver le lot.</span></div>'; return; }
    UI.modale({ titre: 'Archiver ce lot ?', corps: '<p>Rien ne se supprime : l’historique reste consultable.</p>', actions: [{ lib: 'Annuler', v: 'secondaire', focus: true }, { lib: 'Archiver', v: 'primaire', fn: function (f2) { f2(); UI.toast('Action de démonstration. Le lot n’est pas archivé.', { type: 'info' }); } }] });
  });

  App.enregistrer('lot', {
    meta: function (ctx) {
      var lot = Data.lot(ctx.params[0]);
      if (!lot) return { titre: 'Introuvable', repere: { type: 'v1' } };
      var bien = Data.bien(lot.bienId);
      return { titre: F.e(lot.nom), sous: F.e(bien.nom) + ' · ' + F.e(bien.quartier) + (D().organisation.plan === 'agence' ? ' · ' + F.e(Data.mandant(lot.mandantId).nom) : ''), repere: { type: 'v1' }, actions: '<span style="display:flex;gap:8px;flex-wrap:wrap;align-items:center">' + UI.usage(lot.usage) + badgeLot(lot.statut) + '</span>' + UI.btn('Tous les lots', { v: 'tertiaire', ic: 'chevron-left', href: '#/lots' }) };
    },
    rendre: function (ctx) {
      var lot = Data.lot(ctx.params[0]);
      if (!lot) return '<div class="carte reserve">' + UI.etatVide({ ic: 'search', titre: 'Introuvable', texte: 'Cette page n’existe pas ou vous n’y avez pas accès.', action: UI.btn('Retour aux lots', { v: 'primaire', href: '#/lots' }) }) + '</div>';
      var d = D(), on = ctx.query.onglet || 'apercu', b = lot.bailId ? Data.bail(lot.bailId) : null, bien = Data.bien(lot.bienId);
      var inters = d.interventions.filter(function (i) { return i.lotId === lot.id; }), docs = d.documents.filter(function (x) { return x.lotId === lot.id; }), pays = d.paiements.filter(function (p) { return p.lotId === lot.id && p.type === 'paiement'; });
      var n = { baux: b ? 1 : 0, paiements: pays.length, travaux: inters.length, documents: docs.length }, peut = !lecture(ctx), corps;
      if (on === 'baux') corps = b ? '<section class="carte"><div class="carte-entete"><h2 class="carte-titre">' + UI.ic('file-text') + 'Bail en cours</h2></div>' + carteBail(b) + '</section>' : '<section class="carte">' + UI.etatVide({ ic: 'file-text', titre: 'Aucun bail', texte: 'Ce lot n’a pas de bail en cours.' }) + '</section>';
      else if (on === 'paiements') corps = ongletPaiements(lot, b);
      else if (on === 'travaux') corps = '<section class="carte" aria-labelledby="t-trv"><div class="carte-entete"><h2 id="t-trv" class="carte-titre">' + UI.ic('wrench') + 'Travaux' + '</h2>' + UI.repere({ type: 'v1', pilote: true, mini: true }) + '</div>' + (peut && inters.length ? '<div class="btn-groupe" style="margin-bottom:16px">' + UI.btn('Ajouter une intervention', { v: 'secondaire', petit: true, ic: 'plus', action: 'trv-ajouter', data: ' data-lot="' + lot.id + '"' }) + '</div>' : '') + window.Travaux.chronologie(inters, { lotId: lot.id, peutAjouter: peut }) + '</section>';
      else if (on === 'documents') corps = '<section class="carte" aria-labelledby="t-doc"><div class="carte-entete"><h2 id="t-doc" class="carte-titre">' + UI.ic('paperclip') + 'Documents</h2>' + UI.repere({ type: 'v1', pilote: true, mini: true }) + '</div>' + (docs.length ? '<div class="doc-grille">' + docs.map(window.Travaux.vignette).join('') + '</div>' : UI.etatVide({ ic: 'file-text', titre: 'Aucun document', texte: 'Joignez le bail signé pour retrouver le dossier en cas de litige.', action: peut ? UI.btn('Joindre un document', { v: 'primaire', action: 'demo' }) : '' })) + '</section>';
      else corps = ongletApercu(ctx, lot, bien, b);
      return ongletsLot(lot, on, n) + corps;
    },
    apres: function (ctx) { if (ctx.query.onglet === 'travaux' && ctx.query.intervention && Data.intervention(ctx.query.intervention)) setTimeout(function () { window.Travaux.detail(ctx.query.intervention); }, 0); }
  });
})();
