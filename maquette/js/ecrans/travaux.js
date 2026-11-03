/*
 * S14 : Travaux et documents (journal d'interventions, documents, prestataires) et briques partagées avec la fiche lot (S07).
 * Une intervention porte un payeur et un statut d'imputation au compte rendu (R-078, R-084) : payée par l'agence = imputable.
 * « État des lieux complet » (F24) : repère « À venir », élément désactivé.
 * window.Travaux expose : detail, ajouter, vignette, chronologie, imputation.
 */
(function () {
  var F = window.Fmt, UI = window.UI, Data = window.Data, App = window.App, Mut = window.Mut, Etat = window.Etat;
  var PAYEUR = { agence: 'l’agence', proprietaire: 'le propriétaire', locataire: 'le locataire' };
  var STATUT = { planifiee: ['Planifiée', 'calendar'], en_cours: ['En cours', 'clock'], terminee: ['Terminée', 'check'] };
  var ft = { type: '' };

  function statutBadge(s) { return UI.neutre(STATUT[s][0], STATUT[s][1]); }
  function prestataire(i) { return Data.org().prestataires.filter(function (p) { return p.id === i.prestataireId; })[0]; }
  function lienReleve(id) { var r = Data.releve(id); return r ? r : null; }

  // Imputation au compte rendu (DP-18) : badge + libellé
  function imputation(i) {
    if (i.imputeAuReleveId) { var r = lienReleve(i.imputeAuReleveId), mois = F.mois(r.periode).split(' ')[0]; return '<span class="badge badge-brand">' + UI.ic('check') + 'Imputée au compte rendu ' + (/^[aeiouyàâéèêîôû]/i.test(mois) ? 'd’' : 'de ') + mois + '</span>'; }
    if (i.horsHistorique) return UI.neutre('Antérieure à Immo Suite', 'clock', 'badge-pointille');
    if (i.imputable) return UI.neutre('Imputable au prochain compte rendu', 'calendar', 'badge-pointille');
    return UI.neutre('Non imputable (payée par ' + PAYEUR[i.payeur] + ')', 'info');
  }

  /* ---------- Vignettes de documents (SVG fictifs) ---------- */
  function svgVignette(type) {
    var base = '<path d="M8 4h52l22 22v80H8z" fill="#fff" stroke="currentColor" stroke-width="2.5" stroke-linejoin="round"/><path d="M60 4v22h22" fill="#DCE6F3" stroke="currentColor" stroke-width="2.5" stroke-linejoin="round"/>';
    var corps;
    if (type === 'photo') corps = '<rect x="18" y="38" width="54" height="42" rx="3" fill="#DCE6F3" stroke="currentColor" stroke-width="2"/><circle cx="32" cy="52" r="5" fill="currentColor"/><path d="M20 78l16-16 12 10 10-8 14 14" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linejoin="round"/>';
    else if (type === 'facture' || type === 'devis') corps = '<path d="M18 40h30M18 50h54M18 60h54M18 70h40" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/><rect x="48" y="84" width="24" height="12" rx="2" fill="#DCE6F3" stroke="currentColor" stroke-width="2"/>';
    else corps = '<path d="M18 42h54M18 52h54M18 62h54M18 72h36M18 86h20" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/>';
    return '<svg viewBox="0 0 90 112" aria-hidden="true" focusable="false">' + base + corps + '</svg>';
  }
  var LIB_TYPE = { bail: 'Bail', facture: 'Facture', photo: 'Photo', devis: 'Devis', etat_des_lieux: 'État des lieux', mandat: 'Mandat' };
  function vignette(d) {
    return '<button type="button" class="doc-v" data-action="doc-voir" data-id="' + d.id + '"><span class="doc-v-img">' + svgVignette(d.type) + '</span><span class="doc-v-nom">' + F.e(d.nom) + '</span><span class="doc-v-meta">' + LIB_TYPE[d.type] + ' · ' + d.format + ' · ' + F.nombre(d.tailleKo) + ' Ko · ' + F.date(d.date) + '</span></button>';
  }

  /* ---------- Chronologie des interventions ---------- */
  function chronologie(liste, opt) {
    opt = opt || {};
    if (!liste.length) return UI.etatVide({ ic: 'wrench', titre: 'Aucune intervention enregistrée sur ce lot', texte: 'Ajoutez la première pour garder l’historique du bien.', action: opt.peutAjouter ? UI.btn('Ajouter une intervention', { v: 'primaire', ic: 'plus', action: 'trv-ajouter', data: ' data-lot="' + (opt.lotId || '') + '"' }) : '' });
    var l = liste.slice().sort(function (a, b) { return b.date.localeCompare(a.date); });
    return '<ol class="chrono">' + l.map(function (i) {
      return '<li class="chrono-item"><div class="chrono-date"><span class="gras">' + F.date(i.date) + '</span>' + (i.statut === 'planifiee' ? '<span class="muted" style="font-size:var(--text-sm)">à venir</span>' : '') + '</div>' +
        '<button type="button" class="chrono-corps" data-action="trv-detail" data-id="' + i.id + '"><span class="chrono-haut"><span class="gras">' + F.e(i.nature) + '</span><span class="gras num nowrap">' + F.montant(i.cout) + '</span></span>' +
        '<span class="chrono-bas">' + (opt.montrerLot ? '<span>' + UI.ic('building', 'ic-sm') + ' ' + F.e(Data.lot(i.lotId).nom) + '</span>' : '') + '<span>Payée par ' + PAYEUR[i.payeur] + '</span>' + statutBadge(i.statut) + imputation(i) + '</span></button></li>';
    }).join('') + '</ol>';
  }

  /* ---------- Détail d'une intervention ---------- */
  function detail(id) {
    var i = Data.intervention(id), pr = prestataire(i), lot = Data.lot(i.lotId), role = App.ctx.role.id;
    var docs = i.documentIds.map(function (x) { return Data.org().documents.filter(function (d) { return d.id === x; })[0]; });
    var pieces = docs.length ? '<div class="doc-grille" style="grid-template-columns:repeat(auto-fill,minmax(130px,1fr))">' + docs.map(vignette).join('') + '</div>' : '<div class="bandeau bandeau-avert">' + UI.ic('alert-triangle') + '<span>Aucune pièce jointe. Vous pouvez enregistrer, mais le propriétaire pourra la demander.</span></div>';
    var h = '<div style="display:flex;justify-content:space-between;gap:12px;align-items:flex-start"><div><div class="enc-solde-montant">' + F.montant(i.cout) + '</div><div class="secondaire">' + F.e(lot.nom) + ' · ' + F.date(i.date) + '</div></div>' + statutBadge(i.statut) + '</div>' +
      '<dl class="def-liste"><div><dt>Nature</dt><dd>' + F.e(i.nature) + '</dd></div><div><dt>Prestataire</dt><dd>' + F.e(pr ? pr.nom : '—') + '</dd></div><div><dt>Coût</dt><dd>' + F.montant(i.cout) + '</dd></div><div><dt>Qui paie</dt><dd>' + F.majuscule(PAYEUR[i.payeur]) + '</dd></div><div><dt>Imputation</dt><dd>' + imputation(i) + '</dd></div></dl>' +
      '<h3 style="margin:16px 0 8px">Pièces jointes</h3>' + pieces;
    var act = '';
    if (i.imputeAuReleveId) { var r = lienReleve(i.imputeAuReleveId); act += UI.btn('Voir dans le compte rendu d’' + F.mois(r.periode).split(' ')[0], { v: 'primaire', ic: 'chart', href: '#/comptes-rendus/' + r.id }); }
    if (App.ctx.route.nom !== 'lot') act += UI.btn('Voir le lot', { v: 'secondaire', ic: 'building', href: '#/lot/' + lot.id + '?onglet=travaux' });
    UI.panneau(F.e(i.nature), h + (act ? '<div class="btn-groupe" style="margin-top:20px;flex-direction:column;align-items:stretch">' + act + '</div>' : ''));
  }

  /* ---------- Ajout d'une intervention (formulaire, sans modification durable) ---------- */
  function ajouter(lotId) {
    var d = Data.org(), lots = d.lots, sel = lotId || lots[0].id;
    var optsLot = lots.map(function (l) { return '<option value="' + l.id + '"' + (l.id === sel ? ' selected' : '') + '>' + F.e(l.nom + ' · ' + Data.bien(l.bienId).quartier) + '</option>'; }).join('');
    var optsPr = d.prestataires.map(function (p) { return '<option value="' + p.id + '">' + F.e(p.nom) + '</option>'; }).join('');
    var seg = [['agence', 'Agence'], ['proprietaire', 'Propriétaire'], ['locataire', 'Locataire']].map(function (x, k) { return '<input type="radio" name="payeur" id="py-' + x[0] + '" value="' + x[0] + '"' + (k === 0 ? ' checked' : '') + '><label for="py-' + x[0] + '">' + UI.ic('check', 'ic-sm coche') + x[1] + '</label>'; }).join('');
    UI.modale({ titre: 'Ajouter une intervention',
      corps: '<div class="champ"><label class="champ-etiquette" for="iv-lot">Lot</label><select class="saisie" id="iv-lot"' + (lotId ? ' disabled' : '') + '>' + optsLot + '</select></div>' +
        '<div class="champ"><label class="champ-etiquette" for="iv-nature">Nature des travaux (obligatoire)</label><input class="saisie" id="iv-nature" placeholder="Exemple : remplacement du robinet" autocomplete="off"><div class="champ-erreur cache" id="iv-nature-err" role="alert"></div></div>' +
        '<div class="champ"><label class="champ-etiquette" for="iv-date">Date (JJ/MM/AAAA)</label><input class="saisie" id="iv-date" value="' + F.date(d.meta.aujourdhui) + '" inputmode="numeric" autocomplete="off"><div class="champ-erreur cache" id="iv-date-err" role="alert"></div></div>' +
        '<div class="champ"><label class="champ-etiquette" for="iv-cout">Coût (obligatoire)</label><div class="saisie-montant" style="min-height:48px"><input id="iv-cout" inputmode="numeric" style="font-size:var(--text-lg)" placeholder="0"><span class="suffixe">FCFA</span></div><div class="champ-erreur cache" id="iv-cout-err" role="alert"></div></div>' +
        '<div class="champ"><label class="champ-etiquette" for="iv-pr">Prestataire</label><select class="saisie" id="iv-pr">' + optsPr + '</select></div>' +
        '<fieldset class="champ" style="border:0;padding:0;margin:16px 0 0"><legend class="champ-etiquette" style="padding:0;margin-bottom:6px">Qui paie ?</legend><div class="segmente" style="grid-template-columns:repeat(3,minmax(0,1fr))">' + seg + '</div><div class="champ-aide" id="iv-imput" style="margin-top:6px">Payée par l’agence : imputable au prochain compte rendu.</div></fieldset>' +
        '<div class="bandeau bandeau-avert" style="margin-top:12px">' + UI.ic('alert-triangle') + '<span>Aucune pièce jointe. Vous pouvez enregistrer, mais le propriétaire pourra la demander.</span></div>',
      actions: [{ lib: 'Annuler', v: 'secondaire' }, { lib: 'Ajouter l’intervention', v: 'primaire', focus: true, fn: function (fermer) {
        var nature = document.getElementById('iv-nature').value.trim(), cout = parseInt(document.getElementById('iv-cout').value.replace(/\D/g, ''), 10), ok = true;
        var mm = /^(\d{2})\/(\d{2})\/(\d{4})$/.exec(document.getElementById('iv-date').value.trim()), iso = mm ? mm[3] + '-' + mm[2] + '-' + mm[1] : null;
        var err = function (id, t) { var e = document.getElementById(id); e.classList.toggle('cache', !t); e.textContent = t || ''; if (t) ok = false; };
        err('iv-nature-err', nature ? '' : 'Indiquez la nature des travaux.'); err('iv-cout-err', cout > 0 ? '' : 'Saisissez un coût supérieur à 0.'); err('iv-date-err', iso && !isNaN(new Date(iso + 'T00:00:00Z')) ? '' : 'Saisissez une date au format JJ/MM/AAAA.');
        if (!ok) return;
        var lot = document.getElementById('iv-lot').value, pr = document.getElementById('iv-pr').value, py = document.querySelector('input[name="payeur"]:checked').value;
        fermer(); Mut.appliquer('intervention.ajouter', { org: Etat.orgCle(), lotId: lot, nature: nature, date: iso, cout: cout, prestataireId: pr, payeur: py, statut: iso > d.meta.aujourdhui ? 'planifiee' : 'terminee' });
        UI.toast('Intervention ajoutée (démonstration).', { type: 'succes' }); App.rendre();
      } }] });
    document.querySelectorAll('input[name="payeur"]').forEach(function (r) { r.addEventListener('change', function () { document.getElementById('iv-imput').textContent = r.value === 'agence' ? 'Payée par l’agence : imputable au prochain compte rendu.' : 'Payée par ' + PAYEUR[r.value] + ' : non imputable, affichée à titre d’information.'; }); });
    document.getElementById('iv-nature').focus();
  }

  window.Travaux = { detail: detail, ajouter: ajouter, vignette: vignette, chronologie: chronologie, imputation: imputation, statutBadge: statutBadge };

  App.action('trv-detail', function (el) { detail(el.getAttribute('data-id')); });
  App.action('trv-ajouter', function (el) { ajouter(el.getAttribute('data-lot') || null); });
  App.action('doc-voir', function (el) {
    var d = Data.org().documents.filter(function (x) { return x.id === el.getAttribute('data-id'); })[0];
    UI.modale({ titre: F.e(d.nom), corps: '<div class="doc-apercu-grand">' + svgVignette(d.type) + '</div><p class="muted" style="text-align:center;margin-top:8px">Document fictif · ' + LIB_TYPE[d.type] + ' · ' + d.format + ' · ' + F.date(d.date) + '</p>', actions: [{ lib: 'Fermer', v: 'primaire', focus: true }] });
  });
  App.action('trv-filtre', function (el) { ft.type = el.getAttribute('data-t'); majDocs(); });

  /* ---------- Écran S14 ---------- */
  function ongletsS14(actif) {
    var t = [['interventions', 'Interventions'], ['documents', 'Documents'], ['prestataires', 'Prestataires']];
    return '<div class="onglets" role="tablist">' + t.map(function (x) { return '<a class="onglet" role="tab" href="#/travaux?onglet=' + x[0] + '" aria-selected="' + (x[0] === actif) + '">' + x[1] + '</a>'; }).join('') + '</div>';
  }
  function docsFiltres() {
    var d = Data.org().documents, types = ['', 'bail', 'facture', 'photo', 'devis', 'etat_des_lieux', 'mandat'].filter(function (t, i) { return i === 0 || d.some(function (x) { return x.type === t; }); });
    return '<div class="puces-filtres" style="margin:0 0 16px" role="group" aria-label="Filtrer par type de document">' + types.map(function (t) { return '<button type="button" class="tranche" style="min-height:40px;padding:6px 14px;flex-direction:row" aria-pressed="' + (ft.type === t) + '" data-action="trv-filtre" data-t="' + t + '">' + (t ? LIB_TYPE[t] : 'Tous') + '</button>'; }).join('') + '</div>';
  }
  function majDocs() { var z = document.getElementById('trv-docs'); if (z) z.innerHTML = docsFiltres() + gridDocs(); }
  function gridDocs() {
    var l = Data.org().documents.filter(function (d) { return !ft.type || d.type === ft.type; });
    return l.length ? '<div class="doc-grille">' + l.map(vignette).join('') + '</div>' : UI.etatVide({ ic: 'file-text', titre: 'Aucun document', texte: 'Joignez le bail signé pour retrouver le dossier en cas de litige.', action: UI.btn('Joindre un document', { v: 'primaire', action: 'demo' }) });
  }

  App.enregistrer('travaux', {
    meta: function (ctx) {
      var peut = window.Roles.droit(ctx.role.id, 'travaux') === 'E';
      return { titre: 'Travaux et documents', sous: 'La mémoire du bien : travaux, factures, photos et pièces du dossier.', repere: { type: 'v1', pilote: true }, actions: peut ? UI.btn('Ajouter une intervention', { v: 'primaire', ic: 'plus', action: 'trv-ajouter' }) + UI.btn('Joindre un document', { v: 'secondaire', ic: 'paperclip', action: 'demo' }) : '' };
    },
    rendre: function (ctx) {
      var on = ctx.query.onglet || 'interventions', d = Data.org(), peut = window.Roles.droit(ctx.role.id, 'travaux') === 'E';
      ft = { type: '' };
      var corps;
      if (on === 'documents') corps = '<section class="carte" aria-labelledby="t-docs"><h2 id="t-docs" class="sr-only">Documents</h2><div id="trv-docs">' + docsFiltres() + gridDocs() + '</div></section>';
      else if (on === 'prestataires') corps = '<section class="carte" aria-labelledby="t-pr"><h2 id="t-pr" class="sr-only">Prestataires</h2><ul class="liste">' + d.prestataires.map(function (p) { var n = d.interventions.filter(function (i) { return i.prestataireId === p.id; }).length; return '<li><div class="liste-gauche"><span class="liste-titre">' + F.e(p.nom) + '</span><span class="liste-sous">' + p.metier + ' · ' + F.e(p.telephone) + '</span></div><span class="badge badge-neutre">' + n + (n > 1 ? ' interventions' : ' intervention') + '</span></li>'; }).join('') + '</ul></section>';
      else corps = '<section class="carte" aria-labelledby="t-iv"><h2 id="t-iv" class="sr-only">Journal des interventions</h2>' + chronologie(d.interventions, { montrerLot: true, peutAjouter: peut }) + '</section>';
      return ongletsS14(on) + corps + '<p class="muted" style="margin-top:12px;font-size:var(--text-sm)">' + UI.btn('État des lieux complet', { v: 'secondaire', petit: true, disabled: true }) + ' ' + UI.repere({ type: 'avenir', mini: true }) + ' <span>En attendant, joignez l’état des lieux comme pièce du bail.</span></p>';
    }
  });
})();
