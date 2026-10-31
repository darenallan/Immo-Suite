/*
 * S12 : Comptes rendus propriétaires (parcours 3) et document partagé (S26 utilise Doc.releve).
 * Cycle de vie : à produire → brouillon → validé → payé (reversement). Réouverture avec motif tant qu'aucun reversement (R-083).
 * Les montants (encaissé, commission, dépenses, net, impayés) sont précalculés dans data/releves.js, vérifiés par le script de cohérence.
 */
(function () {
  var F = window.Fmt, UI = window.UI, Data = window.Data, App = window.App, Mut = window.Mut, Etat = window.Etat;
  function D() { return window.DEMO.hor; }
  var MODES = { virement: 'Virement', cheque: 'Chèque', especes: 'Espèces', mobile_money: 'Mobile money' };
  var NOMS_MOIS = ['janvier', 'février', 'mars', 'avril', 'mai', 'juin', 'juillet', 'août', 'septembre', 'octobre', 'novembre', 'décembre'];

  function badge(r) {
    if (r.statut === 'a_produire') return '<span class="badge badge-pointille">' + UI.ic('clock') + 'À produire</span>';
    if (r.statut === 'brouillon') return '<span class="badge badge-pointille">' + UI.ic('pen') + 'Brouillon' + (r.version > 1 ? ' · version ' + r.version : '') + '</span>';
    if (r.statut === 'valide') return '<span class="badge badge-brand">' + UI.ic('check') + 'Validé</span>';
    return UI.statut('payee');
  }
  function netAffiche(r) { return r.netAffiche !== undefined ? r.netAffiche : r.net; }
  function baseTexte(b) { return b === 'loyer_charges' ? 'loyer + charges' : 'loyer'; }
  function periodeLib(p) { return F.majuscule(F.mois(p)); }
  function finPeriode(p) { var a = +p.slice(0, 4), m = +p.slice(5, 7), d = new Date(Date.UTC(a, m, 0)).getUTCDate(); return a + '-' + p.slice(5, 7) + '-' + String(d).padStart(2, '0'); }

  /* ---------- Document (aperçu, PDF, page publique) ---------- */
  function docReleve(r, opt) {
    var org = D().organisation, mand = Data.mandant(r.mandantId), L = window.Doc.ligne, mois = periodeLib(r.periode);
    var tauxTxt = F.pourcent(r.tauxCommission);
    var loyers = r.lignesLoyers.map(function (l) {
      var lot = Data.lot(l.lotId), bien = Data.bien(lot.bienId);
      return L(F.date(l.date).slice(0, 5) + ' &nbsp;' + F.e(lot.nom) + ', ' + F.e(bien.quartier) + ' — ' + F.e(Data.locataire(l.locataireId).nom), F.montant(l.montant));
    }).join('');
    var deps = r.depenses.length ? r.depenses.map(function (d) { return L(F.date(d.date).slice(0, 5) + ' &nbsp;' + F.e(d.libelle), '− ' + F.montant(d.montant)); }).join('') : L('Aucune dépense imputée', F.montant(0));
    var imp = r.impayes.length ? r.impayes.map(function (i) { var lot = Data.lot(i.lotId); return L(F.e(lot.nom) + ', ' + F.e(Data.bien(lot.bienId).quartier) + ' — ' + F.e(Data.locataire(i.locataireId).nom) + ' — échéance du ' + F.date(i.dateEcheance).slice(0, 5), F.montant(i.reste)); }).join('') : L('Aucun impayé', F.montant(0));
    var statut = r.statut === 'brouillon' || r.statut === 'a_produire' ? 'brouillon' : r.statut === 'valide' ? 'validé le ' + F.date(r.valideLe) : 'payé le ' + F.date(r.reversement.date) + ' — ' + MODES[r.reversement.mode].toLowerCase() + ', réf. ' + F.e(r.reversement.reference);
    return '<article class="q-feuille q-a4 cr-feuille" aria-label="Compte rendu de gestion, ' + F.e(mois) + '">' +
      '<header class="q-entete"><div class="q-org">COMPTE RENDU DE GESTION — ' + F.e(mois) + '</div><div class="q-sous">Mandant : ' + F.e(mand.nom) + ' — Mandat à ' + tauxTxt + ' (base : ' + baseTexte(r.baseCommission) + ')</div><div class="q-sous">' + F.e(org.nom) + ' — établi le ' + F.date(r.creeLe || D().meta.aujourdhui) + '</div></header>' +
      '<div class="cr-section">LOYERS ENCAISSÉS</div>' + loyers + L('Total encaissé', F.montant(r.encaisse), 'q-total') +
      '<div class="q-bloc">' + L('Commission de l’agence (' + tauxTxt + ')', '− ' + F.montant(r.commission)) + '</div>' +
      '<div class="cr-section">DÉPENSES IMPUTÉES</div>' + deps +
      '<div class="cr-net q-ligne"><span>NET À REVERSER</span><span class="q-val">' + F.montant(netAffiche(r)) + '</span></div>' +
      '<div class="cr-section">IMPAYÉS AU ' + F.date(finPeriode(r.periode)) + '</div>' + imp +
      '<footer class="q-pied"><div>Statut : ' + statut + '</div><div>Document généré par Immo Suite</div></footer></article>';
  }
  window.Doc.releve = docReleve;

  /* ---------- Liste ---------- */
  function pageListe(ctx) {
    var gerant = ctx.role.id === 'gerant';
    var rs = D().releves.slice().sort(function (a, b) { return b.periode.localeCompare(a.periode) || a.mandantId.localeCompare(b.mandantId); });
    var lignes = rs.map(function (r) {
      var m = Data.mandant(r.mandantId);
      return '<tr class="ligne-cliquable" tabindex="0" data-action="cr-ouvrir" data-id="' + r.id + '"><td class="gras">' + F.e(m.nom) + '</td><td>' + periodeLib(r.periode) + '</td><td>' + badge(r) + '</td><td class="droite">' + (r.statut === 'a_produire' ? '<span class="muted">—</span>' : '<span class="gras">' + F.montant(netAffiche(r)) + '</span>') + '</td><td>' + (r.reversement ? F.date(r.reversement.date) + ' · ' + MODES[r.reversement.mode] : '<span class="muted">—</span>') + '</td><td>' + UI.btn(r.statut === 'a_produire' ? (gerant ? 'Produire' : 'Voir') : 'Ouvrir', { v: 'secondaire', petit: true, action: 'cr-ouvrir', data: ' data-id="' + r.id + '"' }) + '</td></tr>';
    }).join('');
    var cartes = rs.map(function (r) {
      var m = Data.mandant(r.mandantId);
      return '<li><button type="button" class="carte-paiement" data-action="cr-ouvrir" data-id="' + r.id + '"><span class="liste-gauche" style="flex:1"><span class="liste-titre">' + F.e(m.nom) + '</span><span class="liste-sous">' + periodeLib(r.periode) + (r.statut === 'a_produire' ? '' : ' · net ' + F.montant(netAffiche(r))) + '</span></span><span class="liste-droite">' + badge(r) + '</span></button></li>';
    }).join('');
    var aProd = rs.filter(function (r) { return r.statut === 'a_produire'; }).length;
    return (aProd ? '<div class="bandeau bandeau-avert" style="margin-bottom:16px">' + UI.ic('briefcase') + '<span>Fin de mois : ' + aProd + ' compte' + (aProd > 1 ? 's' : '') + ' rendu' + (aProd > 1 ? 's' : '') + ' d’octobre ' + (aProd > 1 ? 'sont à produire' : 'est à produire') + '. Vos mandants les attendent.</span></div>' : '') +
      '<section class="carte" aria-labelledby="t-cr"><h2 id="t-cr" class="sr-only">Liste des comptes rendus</h2><div class="tab-bureau tableau-cadre"><table class="tableau"><thead><tr><th scope="col">Mandant</th><th scope="col">Période</th><th scope="col">Statut</th><th scope="col" class="droite">Net à reverser</th><th scope="col">Reversement</th><th scope="col"><span class="sr-only">Action</span></th></tr></thead><tbody>' + lignes + '</tbody></table></div><ul class="tab-mobile">' + cartes + '</ul></section>' +
      '<p class="muted" style="margin-top:12px;font-size:var(--text-sm)">' + UI.repere({ type: 'avenir', mini: true }) + ' Portail mandant : le mandant lit son compte rendu en ligne (F28). Pour l’instant, le compte rendu est partagé en PDF ou par lien.</p>';
  }

  /* ---------- Production : mandant + période ---------- */
  function pageNouveau(ctx) {
    var d = D(), mand = d.mandants.map(function (m) { return '<option value="' + m.id + '">' + F.e(m.nom) + '</option>'; }).join('');
    var periodes = ['2026-10', '2026-09'].map(function (p) { return '<option value="' + p + '">' + periodeLib(p) + (p === d.meta.moisClos ? ' (mois clos)' : '') + '</option>'; }).join('');
    return '<section class="carte" style="max-width:640px" aria-labelledby="t-nv"><h2 id="t-nv" class="carte-titre" style="margin-bottom:16px">' + UI.ic('chart') + 'Choisir le mandant et le mois</h2>' +
      '<div class="champ"><label class="champ-etiquette" for="nv-mandant">Mandant (obligatoire)</label><select class="saisie" id="nv-mandant">' + mand + '</select></div>' +
      '<div class="champ"><label class="champ-etiquette" for="nv-periode">Période (obligatoire)</label><select class="saisie" id="nv-periode">' + periodes + '</select><div class="champ-aide">Un loyer entre dans le compte rendu du mois de sa date d’encaissement. <span class="badge badge-pointille">À confirmer sur le terrain</span></div></div>' +
      '<div id="nv-erreur" aria-live="polite"></div>' +
      '<div class="btn-groupe" style="margin-top:20px">' + UI.btn('Prévisualiser', { v: 'primaire', ic: 'chart', action: 'cr-previsualiser', id: 'btn-previsualiser' }) + UI.btn('Annuler', { v: 'tertiaire', href: '#/comptes-rendus' }) + '</div></section>';
  }

  /* ---------- Détail ---------- */
  function detailDepenses(r) {
    if (!r.depenses.length) return '<p class="secondaire">Aucune dépense imputée.</p>';
    return '<ul class="liste">' + r.depenses.map(function (d) {
      var it = d.interventionId ? Data.intervention(d.interventionId) : null, pr = it ? D().prestataires.filter(function (p) { return p.id === it.prestataireId; })[0] : null;
      var detail = it ? '<div class="cr-depense-detail" id="dep-' + d.interventionId + '" hidden><dl class="def-liste" style="margin:8px 0 0"><div><dt>Prestataire</dt><dd>' + F.e(pr.nom) + '</dd></div><div><dt>Payée par</dt><dd>' + { agence: 'l’agence', proprietaire: 'le propriétaire', locataire: 'le locataire' }[it.payeur] + '</dd></div><div><dt>Imputation</dt><dd>Imputée au compte rendu ' + (/^[aeiouyàâéèêîôû]/i.test(F.mois(r.periode)) ? 'd’' : 'de ') + F.mois(r.periode).split(' ')[0] + '</dd></div><div><dt>Pièces jointes</dt><dd>' + it.documentIds.length + ' (facture, photos)</dd></div></dl><div style="margin-top:8px"><a class="carte-lien" href="#/lot/' + it.lotId + '?onglet=travaux&intervention=' + it.id + '">Voir dans l’historique du lot' + UI.ic('chevron-right', 'ic-sm') + '</a></div></div>'
        : '<div class="bandeau bandeau-avert" style="margin-top:8px">' + UI.ic('alert-triangle') + '<span>Aucune pièce jointe. Vous pouvez enregistrer, mais le propriétaire pourra la demander.</span></div>';
      return '<li style="display:block"><div style="display:flex;justify-content:space-between;gap:12px;align-items:center"><div class="liste-gauche"><span class="liste-titre">' + F.e(d.libelle) + '</span><span class="liste-sous">' + F.date(d.date) + (d.ajustement ? ' · ajustement de démonstration' : '') + '</span></div><div class="liste-droite" style="flex-direction:row;align-items:center;gap:8px"><span class="liste-montant">− ' + F.montant(d.montant) + '</span>' + (it ? '<button type="button" class="btn btn-tertiaire btn-petit" data-action="cr-detail" data-id="' + d.interventionId + '" aria-expanded="false">Détail</button>' : '<button type="button" class="btn btn-tertiaire btn-petit" data-action="cr-annuler-ajust" data-id="' + r.id + '">Annuler l’ajustement</button>') + '</div></div>' + detail + '</li>';
    }).join('') + '</ul>';
  }

  function pageDetail(ctx) {
    var r = Data.releve(ctx.params[0]);
    if (!r) return '<div class="carte reserve">' + UI.etatVide({ ic: 'search', titre: 'Introuvable', texte: 'Cette page n’existe pas ou vous n’y avez pas accès.', action: UI.btn('Retour aux comptes rendus', { v: 'primaire', href: '#/comptes-rendus' }) }) + '</div>';
    var m = Data.mandant(r.mandantId), gerant = ctx.role.id === 'gerant';
    var resume = '<section class="carte" aria-labelledby="t-res-cr"><div class="carte-entete"><h2 id="t-res-cr" class="carte-titre">' + UI.ic('chart') + 'Résumé</h2>' + badge(r) + '</div>' +
      '<dl class="def-liste" style="margin:0"><div><dt>Loyers encaissés (' + r.lignesLoyers.length + ' loyers)</dt><dd class="num">' + F.montant(r.encaisse) + '</dd></div><div><dt>Commission ' + F.pourcent(r.tauxCommission) + ' (base : ' + baseTexte(r.baseCommission) + ')</dt><dd class="num">− ' + F.montant(r.commission) + '</dd></div><div><dt>Dépenses imputées</dt><dd class="num">− ' + F.montant(r.depensesTotal) + '</dd></div><div class="cr-net-ligne"><dt>Net à reverser</dt><dd class="num" aria-label="' + F.e(F.montantSr(netAffiche(r))) + '">' + F.montant(netAffiche(r)) + '</dd></div></dl>' +
      (r.impayes.length ? '<div class="bandeau bandeau-info" style="margin-top:12px">' + UI.ic('alert-triangle') + '<span>Impayé au ' + F.date(finPeriode(r.periode)) + ' : ' + F.e(Data.locataire(r.impayes[0].locataireId).nom) + ', ' + F.montant(r.impayes[0].reste) + (r.impayes.length > 1 ? ' (+ ' + (r.impayes.length - 1) + ')' : '') + '.</span></div>' : '') + '</section>';
    var deps = '<section class="carte" aria-labelledby="t-dep"><div class="carte-entete"><h2 id="t-dep" class="carte-titre">' + UI.ic('wrench') + 'Dépenses imputées</h2>' + (r.statut === 'brouillon' && gerant && r.ajustementDemo && !r.ajuste ? UI.btn('Ajouter une dépense', { v: 'secondaire', petit: true, ic: 'plus', action: 'cr-ajuster', data: ' data-id="' + r.id + '"' }) : '') + '</div>' + detailDepenses(r) + '</section>';
    var actions = '';
    if (gerant) {
      if (r.statut === 'brouillon') actions = UI.btn('Valider le compte rendu', { v: 'primaire', grand: true, ic: 'check', action: 'cr-valider', data: ' data-id="' + r.id + '"' });
      if (r.statut === 'valide') actions = UI.btn('Enregistrer le reversement', { v: 'primaire', grand: true, ic: 'wallet', action: 'cr-reverser', data: ' data-id="' + r.id + '"' }) + '<div class="btn-groupe" style="margin-top:8px">' + UI.btn('Exporter en PDF', { v: 'secondaire', ic: 'download', action: 'cr-pdf', data: ' data-id="' + r.id + '"' }) + UI.btn('Partager le lien', { v: 'secondaire', ic: 'link', action: 'cr-partager', data: ' data-id="' + r.id + '"' }) + UI.btn('Rouvrir', { v: 'tertiaire', ic: 'refresh', action: 'cr-rouvrir', data: ' data-id="' + r.id + '"' }) + '</div>';
      if (r.statut === 'paye') actions = '<div class="btn-groupe">' + UI.btn('Exporter en PDF', { v: 'primaire', ic: 'download', action: 'cr-pdf', data: ' data-id="' + r.id + '"' }) + UI.btn('Partager le lien', { v: 'secondaire', ic: 'link', action: 'cr-partager', data: ' data-id="' + r.id + '"' }) + '</div><p class="muted" style="font-size:var(--text-sm);margin-top:8px">Reversement enregistré : le compte rendu ne peut plus être rouvert. Toute correction va en période suivante.</p>';
    } else if (r.statut !== 'a_produire' && r.statut !== 'brouillon') actions = '<div class="btn-groupe">' + UI.btn('Exporter en PDF', { v: 'secondaire', ic: 'download', action: 'cr-pdf', data: ' data-id="' + r.id + '"' }) + '</div>';
    var rev = r.reversement ? '<div class="bandeau bandeau-succes" style="margin-top:12px">' + UI.ic('check-circle') + '<span>Reversement de ' + F.montant(r.reversement.montant) + ' le ' + F.date(r.reversement.date) + ' par ' + MODES[r.reversement.mode].toLowerCase() + ', réf. ' + F.e(r.reversement.reference) + '.</span></div>' : '';
    var lecture = !gerant ? '<div class="bandeau bandeau-info">' + UI.ic('info') + '<span>Lecture seule : seul le gérant produit, valide et rouvre un compte rendu.</span></div>' : '';
    var vide = r.statut === 'a_produire' ? '<div class="bandeau bandeau-avert">' + UI.ic('clock') + '<span>Ce compte rendu n’est pas encore produit.</span></div>' + (gerant ? UI.btn('Produire le compte rendu', { v: 'primaire', grand: true, ic: 'chart', action: 'cr-produire', data: ' data-id="' + r.id + '"' }) : '') : '';
    return '<div class="cr-page"><div class="cr-colg">' + resume + deps + '<div class="carte pas-imprimer" style="display:flex;flex-direction:column;gap:12px">' + lecture + vide + actions + rev + '</div></div><div class="cr-cold"><div class="q-apercu">' + docReleve(r) + '</div></div></div>';
  }

  App.enregistrer('comptes-rendus', {
    meta: function (ctx) {
      var p = ctx.params[0];
      if (!p) return { titre: 'Comptes rendus', sous: 'Produire, valider et partager le compte rendu mensuel de chaque mandant.', repere: { type: 'v1' }, actions: ctx.role.id === 'gerant' ? UI.btn('Produire un compte rendu', { v: 'primaire', ic: 'plus', href: '#/comptes-rendus/nouveau' }) : '' };
      if (p === 'nouveau') return { titre: 'Produire un compte rendu', sous: 'Choisissez le mandant et le mois.', repere: { type: 'v1' } };
      var r = Data.releve(p);
      if (!r) return { titre: 'Introuvable', repere: { type: 'v1' } };
      return { titre: 'Compte rendu · ' + F.e(Data.mandant(r.mandantId).nom), sous: periodeLib(r.periode), repere: { type: 'v1' }, actions: UI.btn('Tous les comptes rendus', { v: 'tertiaire', ic: 'chevron-left', href: '#/comptes-rendus' }) };
    },
    rendre: function (ctx) {
      var p = ctx.params[0];
      if (!p) return pageListe(ctx);
      if (p === 'nouveau') return pageNouveau(ctx);
      return pageDetail(ctx);
    }
  });

  /* ---------- Actions ---------- */
  App.action('cr-ouvrir', function (el) { App.aller('/comptes-rendus/' + el.getAttribute('data-id')); });
  App.action('cr-detail', function (el) {
    var z = document.getElementById('dep-' + el.getAttribute('data-id')), ouvert = z.hasAttribute('hidden');
    if (ouvert) z.removeAttribute('hidden'); else z.setAttribute('hidden', '');
    el.setAttribute('aria-expanded', String(ouvert)); el.textContent = ouvert ? 'Masquer' : 'Détail';
  });
  App.action('cr-previsualiser', function () {
    var mid = document.getElementById('nv-mandant').value, per = document.getElementById('nv-periode').value, id = 'R-' + mid + '-' + per, r = Data.releve(id);
    var zone = document.getElementById('nv-erreur'); zone.innerHTML = '';
    if (Etat.option('lotSansMandat')) {
      var lt = D().scenarios.lotSansMandat;
      zone.innerHTML = '<div class="bandeau bandeau-erreur" style="margin-top:12px">' + UI.ic('alert-circle') + '<div><strong>Impossible de produire le compte rendu : 1 lot n’a pas de mandat (' + F.e(lt.nom) + ').</strong><div style="margin-top:8px">' + UI.btn('Attribuer un mandat', { v: 'secondaire', petit: true, action: 'demo' }) + '</div></div></div>';
      return;
    }
    if (!r) { zone.innerHTML = '<div class="bandeau bandeau-info" style="margin-top:12px">' + UI.ic('info') + '<span>Dans la démo, utilisez octobre ou septembre 2026.</span></div>'; return; }
    var btn = document.getElementById('btn-previsualiser'); btn.classList.add('charge'); btn.querySelector('span').textContent = 'Calcul en cours…';
    App.attendre(function () {
      if (r.statut === 'a_produire') Mut.appliquer('releve.produire', { id: id });
      App.aller('/comptes-rendus/' + id);
    });
  });
  App.action('cr-produire', function (el) {
    var id = el.getAttribute('data-id');
    if (Etat.option('lotSansMandat')) { var lt = D().scenarios.lotSansMandat; UI.toast('Impossible de produire le compte rendu : 1 lot n’a pas de mandat (' + F.e(lt.nom) + ').', { type: 'erreur' }); return; }
    Mut.appliquer('releve.produire', { id: id }); App.rendre();
  });
  App.action('cr-ajuster', function (el) {
    var id = el.getAttribute('data-id'), r = Data.releve(id), a = r.ajustementDemo;
    UI.modale({ titre: 'Ajouter une dépense au compte rendu', corps: '<p class="secondaire">Démonstration d’un ajustement : une dépense est ajoutée avant la validation. Vous pouvez l’annuler ensuite.</p>' +
      '<div class="champ" style="margin-top:12px"><label class="champ-etiquette" for="aj-lib">Libellé</label><input class="saisie" id="aj-lib" value="' + F.e(a.libelle) + '" readonly></div><div class="champ"><label class="champ-etiquette" for="aj-mt">Montant (obligatoire)</label><div class="saisie-montant" style="min-height:48px"><input id="aj-mt" inputmode="numeric" style="font-size:var(--text-lg)" value="' + F.nombre(a.montant) + '"><span class="suffixe">FCFA</span></div></div>' +
      '<div class="bandeau bandeau-avert" style="margin-top:12px">' + UI.ic('alert-triangle') + '<span>Aucune pièce jointe. Vous pouvez enregistrer, mais le propriétaire pourra la demander.</span></div>',
      actions: [{ lib: 'Annuler', v: 'secondaire' }, { lib: 'Ajouter la dépense', v: 'primaire', focus: true, fn: function (fermer) {
        var n = parseInt(document.getElementById('aj-mt').value.replace(/\D/g, ''), 10);
        if (n !== a.montant) { UI.toast('Dans la démo, utilisez le montant proposé.', { type: 'info' }); return; }
        fermer(); Mut.appliquer('releve.ajuster', { id: id }); App.rendre(); UI.toast('Dépense ajoutée. Net à reverser : ' + F.montant(a.net) + '.', { type: 'avert' });
      } }] });
  });
  App.action('cr-annuler-ajust', function (el) { Mut.appliquer('releve.annulerAjustement', { id: el.getAttribute('data-id') }); App.rendre(); UI.toast('Ajustement annulé.', { type: 'info' }); });
  App.action('cr-valider', function (el) {
    var id = el.getAttribute('data-id'), r = Data.releve(id), m = Data.mandant(r.mandantId);
    UI.modale({ titre: 'Valider le compte rendu de ' + F.e(m.nom) + ' pour ' + F.mois(r.periode) + ' ?', corps: '<p>Il sera figé. Net à reverser : <strong>' + F.montant(netAffiche(r)) + '</strong>. Vous pourrez le rouvrir tant qu’aucun reversement n’est enregistré.</p>',
      actions: [{ lib: 'Annuler', v: 'secondaire' }, { lib: 'Valider', v: 'primaire', focus: true, fn: function (fermer) { fermer(); Mut.appliquer('releve.valider', { id: id }); App.rendre(); UI.toast('Compte rendu de ' + F.e(m.nom) + ' validé.', { type: 'succes' }); } }] });
  });
  App.action('cr-rouvrir', function (el) {
    var id = el.getAttribute('data-id');
    UI.modale({ titre: 'Rouvrir le compte rendu ?', corps: '<p>Une nouvelle version sera créée. Indiquez le motif.</p><div class="champ" style="margin-top:12px"><label class="champ-etiquette" for="ro-motif">Motif (obligatoire)</label><textarea class="saisie" id="ro-motif"></textarea><div class="champ-erreur cache" id="ro-err" role="alert"></div></div>',
      actions: [{ lib: 'Annuler', v: 'secondaire' }, { lib: 'Rouvrir', v: 'primaire', fn: function (fermer) {
        var v = document.getElementById('ro-motif').value.trim(); if (!v) { var e = document.getElementById('ro-err'); e.classList.remove('cache'); e.textContent = 'Indiquez le motif de la réouverture.'; document.getElementById('ro-motif').focus(); return; }
        fermer(); Mut.appliquer('releve.rouvrir', { id: id, motif: v }); App.rendre(); UI.toast('Compte rendu rouvert (nouvelle version).', { type: 'info' });
      } }] });
    document.getElementById('ro-motif').focus();
  });
  App.action('cr-reverser', function (el) {
    var id = el.getAttribute('data-id'), r = Data.releve(id), m = Data.mandant(r.mandantId);
    UI.modale({ titre: 'Enregistrer le reversement de ' + F.montant(netAffiche(r)) + ' à ' + F.e(m.nom) + ' ?', corps: '<div class="bandeau bandeau-avert">' + UI.ic('alert-triangle') + '<span><strong>Cette action est définitive :</strong> le compte rendu ne pourra plus être rouvert.</span></div>' +
      '<div class="champ" style="margin-top:16px"><label class="champ-etiquette" for="rv-date">Date du reversement (JJ/MM/AAAA)</label><input class="saisie" id="rv-date" value="' + F.date(D().meta.aujourdhui) + '" inputmode="numeric" autocomplete="off"><div class="champ-erreur cache" id="rv-date-err" role="alert"></div></div>' +
      '<div class="champ"><label class="champ-etiquette" for="rv-mode">Mode</label><select class="saisie" id="rv-mode"><option value="virement">Virement</option><option value="cheque">Chèque</option><option value="especes">Espèces</option><option value="mobile_money">Mobile money</option></select></div>' +
      '<div class="champ"><label class="champ-etiquette" for="rv-ref">Référence (obligatoire)</label><input class="saisie" id="rv-ref" placeholder="Exemple : VIR-261113-0001" autocomplete="off"><div class="champ-erreur cache" id="rv-ref-err" role="alert"></div></div>',
      actions: [{ lib: 'Annuler', v: 'secondaire', focus: true }, { lib: 'Enregistrer le reversement', v: 'primaire', fn: function (fermer) {
        var dv = document.getElementById('rv-date').value.trim(), ref = document.getElementById('rv-ref').value.trim(), ok = true;
        var mm = /^(\d{2})\/(\d{2})\/(\d{4})$/.exec(dv), iso = mm ? mm[3] + '-' + mm[2] + '-' + mm[1] : null;
        var eD = document.getElementById('rv-date-err'), eR = document.getElementById('rv-ref-err');
        eD.classList.add('cache'); eR.classList.add('cache');
        if (!iso || isNaN(new Date(iso + 'T00:00:00Z'))) { eD.classList.remove('cache'); eD.textContent = 'Saisissez une date au format JJ/MM/AAAA.'; ok = false; } else if (iso > D().meta.aujourdhui) { eD.classList.remove('cache'); eD.textContent = 'La date du reversement ne peut pas être dans le futur.'; ok = false; }
        if (!ref) { eR.classList.remove('cache'); eR.textContent = 'Saisissez la référence du reversement.'; ok = false; }
        if (!ok) return;
        var mode = document.getElementById('rv-mode').value;
        fermer(); Mut.appliquer('releve.reverser', { id: id, date: iso, mode: mode, reference: ref }); App.rendre(); UI.toast('Reversement de ' + F.montant(netAffiche(r)) + ' enregistré.', { type: 'succes' });
      } }] });
  });
  App.action('cr-pdf', function (el) {
    var r = Data.releve(el.getAttribute('data-id'));
    document.body.classList.add('apercu-doc');
    var m = UI.modale({ titre: 'Aperçu du compte rendu (PDF)', corps: '<div class="q-apercu">' + docReleve(r) + '</div><p class="muted" style="font-size:var(--text-sm);margin-top:8px">Le PDF est identique à la dernière version validée.</p>',
      actions: [{ lib: 'Fermer', v: 'secondaire', fn: function (f) { f(); } }, { lib: 'Imprimer ou enregistrer en PDF', v: 'primaire', fn: function () { try { window.print(); } catch (e) { UI.toast('L’impression n’est pas disponible ici.', { type: 'avert' }); } } }] });
    m.el.querySelector('.modale').classList.add('modale-large');
    var obs = new MutationObserver(function () { if (!document.body.contains(m.el)) { document.body.classList.remove('apercu-doc'); obs.disconnect(); } }); obs.observe(document.getElementById('calques'), { childList: true });
  });
  App.action('cr-partager', function (el) {
    var r = Data.releve(el.getAttribute('data-id')), lien = 'https://immo-suite.demo/c/' + r.id;
    UI.modale({ titre: 'Partager le compte rendu', corps: '<p>Le lien ne donne accès qu’à ce document, sans compte. Il expire après une durée configurable (30 jours <span class="badge badge-pointille">valeur à fixer</span>).</p><div class="champ" style="margin-top:12px"><label class="champ-etiquette" for="lien">Lien de partage (simulation)</label><input class="saisie" id="lien" value="' + F.e(lien) + '" readonly></div>',
      actions: [{ lib: 'Copier le lien', v: 'secondaire', fn: function () { try { if (navigator.clipboard && navigator.clipboard.writeText) navigator.clipboard.writeText(lien); } catch (e) { /* facultatif */ } UI.toast('Lien copié.', { type: 'succes' }); } },
        { lib: 'Voir ce que verrait le mandant', v: 'primaire', focus: true, fn: function (fermer) { fermer(); App.aller('/partage/' + r.id); } }] });
  });
})();
