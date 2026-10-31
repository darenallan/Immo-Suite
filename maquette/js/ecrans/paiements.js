/*
 * S11 : Paiements et quittances (journal des paiements, détail, demandes de correction).
 * Un paiement validé est immuable (R-047) : on ne peut que demander une correction ; seul le gérant contre-passe (R-048).
 * « Payé » est réservé à l'échéance : un paiement enregistré est « Validé », un paiement contre-passé « Corrigé » (DP-43).
 */
(function () {
  var F = window.Fmt, UI = window.UI, Data = window.Data, App = window.App, Mut = window.Mut;
  var f = { q: '', mode: '', page: 1 }, PAGE = 25;
  function D() { return Data.org(); }
  var ICONE_MODE = { especes: 'banknote', mobile_money: 'phone', virement: 'building', cheque: 'file-text' };

  function liste() {
    var n = F.norm(f.q);
    return D().paiements.slice().sort(function (a, b) { return (b.dateSaisie + b.heure + b.id).localeCompare(a.dateSaisie + a.heure + a.id); }).filter(function (p) {
      if (f.mode && p.mode !== f.mode) return false;
      if (!n) return true;
      var q = p.quittanceId ? Data.quittance(p.quittanceId) : null;
      return F.norm(Data.nomLocataires(p.locataireIds) + ' ' + Data.lot(p.lotId).nom + ' ' + (q ? q.numero : '') + ' ' + (p.reference || '')).indexOf(n) >= 0;
    });
  }
  function badgeStatut(p) {
    if (p.type === 'contre-passation') return UI.neutre('Écriture inverse', 'refresh');
    if (p.statut === 'corrige') return '<span title="Paiement corrigé : une écriture inverse a été créée, le paiement d’origine reste visible.">' + UI.statut('corrige') + '</span>';
    return UI.statut('valide');
  }
  function numero(p) { var q = p.quittanceId ? Data.quittance(p.quittanceId) : null; return q ? '<a href="#/quittance/' + q.id + '" class="nowrap">' + F.e(q.numero) + '</a>' : '<span class="muted">—</span>'; }
  function quand(p) { return F.date(p.date) + (p.dateSaisie !== p.date ? '<br><span class="muted" style="font-size:var(--text-sm)">saisi le ' + F.date(p.dateSaisie) + '</span>' : ''); }
  function montantCell(p) { return '<span class="num' + (p.montant < 0 ? ' texte-neg' : '') + '" aria-label="' + F.e(F.montantSr(p.montant)) + '">' + F.montant(p.montant) + '</span>'; }

  function htmlTableau(l) {
    var visibles = l.slice(0, f.page * PAGE);
    if (!visibles.length) return UI.etatVide({ ic: 'search', titre: 'Aucun paiement sur cette période', texte: 'Aucun résultat avec ces filtres.', action: UI.btn('Effacer les filtres', { v: 'secondaire', action: 'pay-effacer' }) });
    var lignes = visibles.map(function (p) {
      return '<tr class="ligne-cliquable" data-action="pay-ouvrir" data-id="' + p.id + '" tabindex="0"><td>' + quand(p) + '</td><td><span class="gras">' + F.e(Data.nomLocataires(p.locataireIds)) + '</span></td><td>' + F.e(Data.lot(p.lotId).nom) + '</td>' +
        '<td class="droite">' + montantCell(p) + '</td><td>' + UI.ic(ICONE_MODE[p.mode], 'ic-sm') + ' ' + Data.modeLibelle(p.mode) + '</td><td class="muted nowrap">' + F.e(p.reference || '—') + '</td><td>' + F.e(Data.utilisateur(p.recuPar).nom) + '</td><td>' + (p.type === 'paiement' ? numero(p) : '<span class="muted">—</span>') + '</td><td>' + badgeStatut(p) + '</td></tr>';
    }).join('');
    var cartes = visibles.map(function (p) {
      return '<li><button type="button" class="carte-paiement" data-action="pay-ouvrir" data-id="' + p.id + '"><span class="pay-mode" aria-hidden="true">' + UI.ic(ICONE_MODE[p.mode]) + '</span><span class="liste-gauche" style="flex:1"><span class="liste-titre">' + F.e(Data.nomLocataires(p.locataireIds)) + '</span><span class="liste-sous">' + F.e(Data.lot(p.lotId).nom) + ' · ' + F.date(p.date) + '</span></span><span class="liste-droite">' + montantCell(p) + badgeStatut(p) + '</span></button></li>';
    }).join('');
    var total = visibles.filter(function (p) { return p.type === 'paiement' && p.statut === 'valide'; }).reduce(function (s, p) { return s + p.montant; }, 0);
    var plus = l.length > visibles.length ? '<div style="text-align:center;margin-top:12px">' + UI.btn('Afficher plus (' + (l.length - visibles.length) + ')', { v: 'secondaire', action: 'pay-plus' }) + '</div>' : '';
    return '<div class="tab-bureau tableau-cadre"><table class="tableau"><thead><tr><th scope="col">Date</th><th scope="col">Locataire</th><th scope="col">Lot</th><th scope="col" class="droite">Montant</th><th scope="col">Mode</th><th scope="col">Référence</th><th scope="col">Reçu par</th><th scope="col">N° de quittance</th><th scope="col">Statut</th></tr></thead><tbody>' + lignes + '</tbody>' +
      '<tfoot><tr><td colspan="3">Paiements validés affichés (' + visibles.length + ' lignes)</td><td class="droite">' + F.montant(total) + '</td><td colspan="5"></td></tr></tfoot></table></div>' +
      '<ul class="tab-mobile">' + cartes + '</ul>' + plus;
  }

  function htmlDemandes() {
    var role = window.App.ctx.role.id, att = D().demandesCorrection.filter(function (d) { return d.statut === 'en_attente'; });
    if (!att.length || (role !== 'gerant' && role !== 'gestionnaire')) return '';
    return '<section class="carte" style="margin-bottom:16px" aria-labelledby="t-dem"><h2 id="t-dem" class="carte-titre" style="margin-bottom:8px">' + UI.ic('pen') + 'Demandes de correction (' + att.length + ')</h2><ul class="liste">' + att.map(function (d) {
      var p = Data.paiement(d.paiementId);
      return '<li><div class="liste-gauche"><span class="liste-titre">' + F.e(Data.nomLocataires(p.locataireIds)) + ' · ' + F.montant(p.montant) + '</span><span class="liste-sous">' + F.e(Data.utilisateur(d.auteurId).nom) + ' · ' + F.date(d.date) + ' : ' + F.e(d.motif) + '</span></div>' + UI.btn(role === 'gerant' ? 'Traiter' : 'Voir', { v: 'secondaire', petit: true, action: 'pay-ouvrir', data: ' data-id="' + p.id + '"' }) + '</li>';
    }).join('') + '</ul></section>';
  }

  function htmlFiltres() {
    var modes = [['', 'Tous les modes'], ['especes', 'Espèces'], ['mobile_money', 'Mobile money'], ['virement', 'Virement'], ['cheque', 'Chèque']];
    return '<div class="filtres"><div class="champ" style="flex:1;min-width:220px"><label class="champ-etiquette" for="pay-q">Rechercher</label><div style="position:relative"><span style="position:absolute;left:12px;top:50%;transform:translateY(-50%);color:var(--color-text-muted)">' + UI.ic('search') + '</span><input class="saisie" id="pay-q" type="search" style="padding-left:40px" value="' + F.e(f.q) + '" placeholder="Nom, lot, référence, n° de quittance" autocomplete="off"></div></div>' +
      '<div class="champ" style="min-width:180px"><label class="champ-etiquette" for="pay-mode">Mode de paiement</label><select class="saisie" id="pay-mode">' + modes.map(function (m) { return '<option value="' + m[0] + '"' + (f.mode === m[0] ? ' selected' : '') + '>' + m[1] + '</option>'; }).join('') + '</select></div></div>';
  }

  function majListe() {
    var l = liste(), z = document.getElementById('pay-liste');
    z.innerHTML = htmlTableau(l);
    document.getElementById('pay-compte').textContent = l.length + (l.length > 1 ? ' paiements' : ' paiement');
  }

  /* ---------- Détail d'un paiement ---------- */
  function ouvrirDetail(id) {
    var p = Data.paiement(id), q = p.quittanceId ? Data.quittance(p.quittanceId) : null, role = App.ctx.role.id, bail = Data.bail(p.bailId);
    var aff = p.affectations && p.affectations.length ? '<ul class="liste">' + p.affectations.map(function (a) { var e = Data.echeance(a.echeanceId); return '<li><div class="liste-gauche"><span class="liste-titre">' + (a.ligne === 'loyer' ? 'Loyer' : 'Charges') + ' de ' + F.mois(e.periode) + '</span><span class="liste-sous">Échéance du ' + F.date(e.dateEcheance) + '</span></div><span class="liste-montant">' + F.montant(a.montant) + '</span></li>'; }).join('') + (p.creditBail ? '<li><div class="liste-gauche"><span class="liste-titre">Crédit du bail</span><span class="liste-sous">Affecté à la prochaine échéance</span></div><span class="liste-montant">' + F.montant(p.creditBail) + '</span></li>' : '') + '</ul>' : '<p class="secondaire">Aucune affectation.</p>';
    var tiers = p.payePar && p.payePar.type === 'tiers' ? 'Payé par ' + F.e(p.payePar.nom) : 'Payé par ' + F.e(Data.locataire(p.payePar.locataireId).nom);
    var dem = D().demandesCorrection.filter(function (d) { return d.paiementId === p.id; });
    var demHtml = dem.map(function (d) { return '<div class="bandeau ' + (d.statut === 'en_attente' ? 'bandeau-avert' : 'bandeau-info') + '" style="margin-top:12px">' + UI.ic('pen') + '<span><strong>Demande de correction ' + (d.statut === 'en_attente' ? 'en attente' : 'traitée') + '</strong> · ' + F.e(Data.utilisateur(d.auteurId).nom) + ', ' + F.date(d.date) + '.<br>' + F.e(d.motif) + (d.traiteeLe ? '<br>Traitée le ' + F.date(d.traiteeLe) + ' par ' + F.e(Data.utilisateur(d.traiteePar).nom) + '.' : '') + '</span></div>'; }).join('');
    var h = '<div style="display:flex;justify-content:space-between;align-items:flex-start;gap:12px"><div><div class="enc-solde-montant">' + F.montant(p.montant) + '</div><div class="secondaire">' + F.e(Data.nomLocataires(p.locataireIds)) + ' · ' + F.e(Data.lot(p.lotId).nom) + '</div></div>' + badgeStatut(p) + '</div>' +
      '<dl class="def-liste"><div><dt>Date du paiement</dt><dd>' + F.date(p.date) + '</dd></div><div><dt>Saisi le</dt><dd>' + F.date(p.dateSaisie) + ' à ' + p.heure + '</dd></div><div><dt>Mode</dt><dd>' + Data.modeLibelle(p.mode) + (p.reference ? ' · ' + F.e(p.reference) : '') + '</dd></div><div><dt>Reçu par</dt><dd>' + F.e(Data.utilisateur(p.recuPar).nom) + '</dd></div><div><dt>Payeur</dt><dd>' + tiers + '</dd></div>' + (q ? '<div><dt>Quittance</dt><dd><a href="#/quittance/' + q.id + '">' + F.e(q.numero) + '</a> · ' + (q.statut === 'annulee' ? 'annulée' : q.rectificative ? 'rectificative' : 'émise') + '</dd></div>' : '') + '</dl>';
    if (p.type === 'contre-passation') h += '<div class="bandeau bandeau-info" style="margin-top:12px">' + UI.ic('info') + '<span>Écriture inverse liée au paiement ' + F.e(p.origineId) + '. Motif : ' + F.e(p.motif) + '</span></div>';
    else h += '<h3 style="margin:16px 0 8px">Affectations</h3>' + aff;
    h += demHtml;
    var acts = '';
    if (q) acts += UI.btn('Voir la quittance', { v: 'primaire', href: '#/quittance/' + q.id, ic: 'receipt' }) + UI.btn('Envoyer sur WhatsApp', { v: 'secondaire', ic: 'message', action: 'pay-envoyer', data: ' data-id="' + p.id + '"' });
    if (p.type === 'paiement' && p.statut === 'valide') {
      acts += UI.btn('Demander une correction', { v: 'secondaire', ic: 'pen', action: 'pay-demande', data: ' data-id="' + p.id + '"' });
      if (role === 'gerant') acts += UI.btn('Corriger le paiement', { v: 'secondaire', ic: 'refresh', action: 'pay-corriger', data: ' data-id="' + p.id + '"' });
    }
    h += '<div class="btn-groupe" style="margin-top:20px;flex-direction:column;align-items:stretch">' + acts + '</div><p class="muted" style="font-size:var(--text-sm);margin-top:12px">Un paiement validé ne peut pas être modifié. Vous pouvez demander une correction au gérant.</p>';
    UI.panneau('Paiement ' + F.e(p.id), h);
  }

  App.enregistrer('paiements', {
    meta: function (ctx) { return { titre: 'Paiements et quittances', sous: 'Journal des paiements, quittances, demandes de correction.', repere: { type: 'v1' }, actions: UI.btn('Exporter', { v: 'secondaire', ic: 'download', action: 'demo' }) + UI.repere({ type: 'v1', pilote: true, mini: true }) }; },
    rendre: function (ctx) {
      f = { q: ctx.query.q || '', mode: '', page: 1 };
      var l = liste();
      return htmlDemandes() + '<section class="carte" aria-labelledby="t-j"><h2 id="t-j" class="sr-only">Journal des paiements</h2>' + htmlFiltres() + '<p class="secondaire" style="margin:8px 0 12px" id="pay-compte" aria-live="polite">' + l.length + ' paiements</p><div id="pay-liste">' + htmlTableau(l) + '</div></section>';
    },
    apres: function (ctx, rac) {
      var q = rac.querySelector('#pay-q'), t = null;
      q.addEventListener('input', function () { clearTimeout(t); t = setTimeout(function () { f.q = q.value; f.page = 1; majListe(); }, 200); });
      rac.querySelector('#pay-mode').addEventListener('change', function (e) { f.mode = e.target.value; f.page = 1; majListe(); });
      rac.addEventListener('keydown', function (ev) { if (ev.key === 'Enter' && ev.target.classList && ev.target.classList.contains('ligne-cliquable')) ouvrirDetail(ev.target.getAttribute('data-id')); });
      if (ctx.query.ouvrir && Data.paiement(ctx.query.ouvrir)) ouvrirDetail(ctx.query.ouvrir);
    }
  });

  App.action('pay-ouvrir', function (el) { ouvrirDetail(el.getAttribute('data-id')); });
  App.action('pay-plus', function () { f.page += 1; majListe(); });
  App.action('pay-effacer', function () { f = { q: '', mode: '', page: 1 }; document.getElementById('pay-q').value = ''; document.getElementById('pay-mode').value = ''; majListe(); });
  App.action('pay-envoyer', function (el) {
    var p = Data.paiement(el.getAttribute('data-id')), q = Data.quittance(p.quittanceId), t = Data.locataire(p.locataireIds[0]);
    UI.messagerie({ canal: 'whatsapp', nom: t.nom, telephone: t.telephone, texte: window.Doc.texte(q.id), surEnvoi: function () { Mut.appliquer('quittance.envoyer', { id: q.id, canal: 'whatsapp', par: App.ctx.utilisateur.id }); } });
  });
  App.action('pay-demande', function (el) {
    var id = el.getAttribute('data-id'), p = Data.paiement(id);
    var m = UI.modale({ titre: 'Demander la correction du paiement ?', corps: '<p>Le paiement reste inchangé. Le gérant recevra votre demande et pourra le corriger par une écriture inverse.</p><div class="champ" style="margin-top:12px"><label class="champ-etiquette" for="motif">Motif (obligatoire)</label><textarea class="saisie" id="motif" placeholder="Exemple : montant saisi à tort"></textarea><div class="champ-erreur cache" id="motif-err" role="alert"></div></div>',
      actions: [{ lib: 'Annuler', v: 'secondaire' }, { lib: 'Envoyer la demande', v: 'primaire', fn: function (fermer) {
        var v = document.getElementById('motif').value.trim();
        if (!v) { var e = document.getElementById('motif-err'); e.classList.remove('cache'); e.textContent = 'Indiquez le motif de la correction.'; document.getElementById('motif').focus(); return; }
        Mut.appliquer('correction.demander', { paiementId: id, motif: v, auteurId: App.ctx.utilisateur.id }); fermer();
        document.querySelectorAll('.panneau-fond').forEach(function (x) { x.parentNode.removeChild(x); });
        UI.toast('Demande de correction envoyée au gérant.', { type: 'succes' }); App.rendre();
      } }] });
    document.getElementById('motif').focus();
  });
  App.action('pay-corriger', function (el) {
    var p = Data.paiement(el.getAttribute('data-id')), q = Data.quittance(p.quittanceId);
    UI.modale({ titre: 'Corriger le paiement n° ' + F.e(q.numero) + ' ?', corps: '<p>Une écriture inverse de ' + F.montant(p.montant) + ' sera créée. Le paiement d’origine reste visible. Motif obligatoire.</p><div class="champ" style="margin-top:12px"><label class="champ-etiquette" for="motif2">Motif (obligatoire)</label><textarea class="saisie" id="motif2"></textarea></div>',
      actions: [{ lib: 'Annuler', v: 'secondaire', focus: true }, { lib: 'Corriger le paiement', v: 'primaire', fn: function (fermer) { fermer(); UI.toast('Action de démonstration. Le paiement du scénario de correction (57 000 FCFA) est déjà corrigé dans le journal.', { type: 'info' }); } }] });
  });
})();
