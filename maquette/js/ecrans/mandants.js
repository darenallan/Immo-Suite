/*
 * S13 : Mandants et mandats (plan Agence, gérant seulement).
 * Liste des mandants, fiche (contacts, mode de reversement, mandats : taux, base, dates, lots couverts).
 * Garde-fous montrés : taux hors plage (R-015), avenant de taux non rétroactif (R-069), un seul mandat actif par lot (R-014), lot sans mandat (R-075).
 */
(function () {
  var F = window.Fmt, UI = window.UI, Data = window.Data, App = window.App;
  function D() { return Data.org(); }
  function baseTxt(b) { return b === 'loyer_charges' ? 'loyer + charges' : 'loyer'; }
  function lotsDe(m) { return D().lots.filter(function (l) { return l.mandantId === m.id; }); }
  function mandatDe(m) { return D().mandats.filter(function (x) { return x.mandantId === m.id; })[0]; }
  function repart(lots) {
    var h = lots.filter(function (l) { return l.usage === 'habitation'; }).length, c = lots.filter(function (l) { return l.usage === 'commercial'; }).length, x = lots.filter(function (l) { return l.usage === 'mixte'; }).length;
    return h + ' habitation' + (c ? ' · ' + c + ' commercial' + (c > 1 ? 'aux' : '') : '') + (x ? ' · ' + x + ' mixte' : '');
  }
  function err(id, t) { var e = document.getElementById(id); e.classList.toggle('cache', !t); e.innerHTML = t ? UI.ic('alert-circle', 'ic-sm') + '<span>' + t + '</span>' : ''; return !t; }
  function taux(txt) { var n = parseFloat(String(txt).replace(',', '.')); return isNaN(n) ? null : n; }

  function creer() {
    UI.modale({ titre: 'Créer un mandant', corps: '<div class="champ"><label class="champ-etiquette" for="mc-nom">Nom du mandant (obligatoire)</label><input class="saisie" id="mc-nom" autocomplete="off"><div class="champ-erreur cache" id="mc-nom-err" role="alert"></div></div><div class="champ"><label class="champ-etiquette" for="mc-taux">Taux de commission en % (obligatoire)</label><input class="saisie" id="mc-taux" inputmode="decimal" placeholder="7,5" autocomplete="off"><div class="champ-erreur cache" id="mc-taux-err" role="alert"></div></div><div class="champ"><label class="champ-etiquette" for="mc-base">Base de calcul</label><select class="saisie" id="mc-base"><option>Loyer seul (par défaut)</option><option>Loyer + charges</option></select></div>',
      actions: [{ lib: 'Annuler', v: 'secondaire' }, { lib: 'Créer le mandant', v: 'primaire', focus: true, fn: function (fermer) {
        var ok = err('mc-nom-err', document.getElementById('mc-nom').value.trim() ? '' : 'Indiquez le nom du mandant.'), t = taux(document.getElementById('mc-taux').value);
        ok = err('mc-taux-err', t === null ? 'Saisissez le taux de commission.' : t < 0 || t > 100 ? 'Le taux de commission est compris entre 0 % et 100 %.' : '') && ok; if (!ok) return;
        fermer(); UI.toast('Mandant créé (démonstration). Aucune donnée n’est ajoutée.', { type: 'succes' }); } }] });
    document.getElementById('mc-nom').focus();
  }
  App.action('mand-creer', creer);
  App.action('mand-ouvrir', function (el) { App.aller('/mandants/' + el.getAttribute('data-id')); });
  App.action('mand-avenant', function (el) {
    var m = Data.mandant(el.getAttribute('data-id')), mt = mandatDe(m);
    UI.modale({ titre: 'Avenant de taux pour ' + F.e(m.nom), corps: '<p class="secondaire">Un changement de taux n’est pas rétroactif : un avenant daté du 1er du mois suivant ne change pas le compte rendu du mois courant.</p><div class="champ" style="margin-top:12px"><label class="champ-etiquette" for="ma-date">Date d’effet (JJ/MM/AAAA)</label><input class="saisie" id="ma-date" value="01/12/2026" inputmode="numeric"></div><div class="champ"><label class="champ-etiquette" for="ma-taux">Nouveau taux en %</label><input class="saisie" id="ma-taux" value="' + String(mt.taux).replace('.', ',') + '" inputmode="decimal"><div class="champ-erreur cache" id="ma-err" role="alert"></div></div>',
      actions: [{ lib: 'Annuler', v: 'secondaire' }, { lib: 'Enregistrer l’avenant', v: 'primaire', focus: true, fn: function (fermer) {
        var t = taux(document.getElementById('ma-taux').value); if (!err('ma-err', t === null ? 'Saisissez le taux de commission.' : t < 0 || t > 100 ? 'Le taux de commission est compris entre 0 % et 100 %.' : '')) return;
        fermer(); UI.toast('Avenant de taux enregistré (démonstration). Le compte rendu du mois courant reste au taux actuel.', { type: 'succes' }); } }] });
  });
  App.action('mand-cloturer', function (el) {
    var m = Data.mandant(el.getAttribute('data-id'));
    UI.modale({ titre: 'Clôturer le mandat de ' + F.e(m.nom) + ' ?', corps: '<div class="bandeau bandeau-avert">' + UI.ic('alert-triangle') + '<span>Les ' + lotsDe(m).length + ' lots n’auront plus de mandat : ils bloqueront la production du compte rendu tant qu’un nouveau mandat n’est pas attribué.</span></div>', actions: [{ lib: 'Annuler', v: 'secondaire', focus: true }, { lib: 'Clôturer le mandat', v: 'primaire', fn: function (f) { f(); UI.toast('Action de démonstration. Le mandat n’est pas clôturé.', { type: 'info' }); } }] });
  });
  App.action('mand-transferer', function (el) {
    var m = Data.mandant(el.getAttribute('data-id')), autres = D().mandants.map(function (x) { return '<option value="' + x.id + '">' + F.e(x.nom) + '</option>'; }).join('');
    UI.modale({ titre: 'Transférer des lots', corps: '<div class="champ"><label class="champ-etiquette" for="mt-vers">Vers le mandant</label><select class="saisie" id="mt-vers">' + autres + '</select><div class="champ-erreur cache" id="mt-err" role="alert"></div></div><fieldset class="champ" style="border:0;padding:0"><legend class="champ-etiquette" style="padding:0;margin-bottom:6px">Lots à transférer</legend>' + lotsDe(m).map(function (l) { return '<label style="display:flex;gap:10px;align-items:center;min-height:36px"><input type="checkbox" name="mt-lot" value="' + l.id + '"><span>' + F.e(l.nom) + '</span></label>'; }).join('') + '</fieldset>',
      actions: [{ lib: 'Annuler', v: 'secondaire' }, { lib: 'Transférer', v: 'primaire', focus: true, fn: function (fermer) {
        if (document.getElementById('mt-vers').value === m.id) { err('mt-err', 'Un lot est couvert par un seul mandat actif. Choisissez un autre mandant.'); return; }
        if (!document.querySelectorAll('input[name="mt-lot"]:checked').length) { err('mt-err', 'Choisissez au moins un lot.'); return; }
        fermer(); UI.toast('Action de démonstration. Aucun lot n’est transféré.', { type: 'info' }); } }] });
  });

  App.enregistrer('mandants', {
    meta: function (ctx) {
      var id = ctx.params[0], m = id ? Data.mandant(id) : null;
      if (!id) return { titre: 'Mandants et mandats', sous: 'Pour qui vous gérez, et à quelles conditions.', repere: { type: 'v1' }, actions: UI.btn('Créer un mandant', { v: 'primaire', ic: 'plus', action: 'mand-creer' }) };
      if (!m) return { titre: 'Introuvable', repere: { type: 'v1' } };
      return { titre: F.e(m.nom), sous: 'Mandat à ' + F.pourcent(mandatDe(m).taux) + ' · base : ' + baseTxt(mandatDe(m).base), repere: { type: 'v1' }, actions: UI.btn('Tous les mandants', { v: 'tertiaire', ic: 'chevron-left', href: '#/mandants' }) };
    },
    rendre: function (ctx) {
      var d = D(), id = ctx.params[0];
      if (!id) {
        var lignes = d.mandants.map(function (m) { var mt = mandatDe(m), lots = lotsDe(m); return '<tr class="ligne-cliquable" tabindex="0" data-action="mand-ouvrir" data-id="' + m.id + '"><td class="gras">' + F.e(m.nom) + '</td><td>' + F.pourcent(mt.taux) + '<br><span class="muted" style="font-size:var(--text-sm)">base : ' + baseTxt(mt.base) + '</span></td><td class="droite">' + lots.length + '</td><td>' + repart(lots) + '</td><td class="nowrap">' + F.e(m.telephone) + '</td><td>' + F.e(m.modeReversement) + '</td></tr>'; }).join('');
        var cartes = d.mandants.map(function (m) { var mt = mandatDe(m); return '<li><button type="button" class="carte-impaye" data-action="mand-ouvrir" data-id="' + m.id + '"><span class="ci-haut"><span class="liste-gauche" style="flex:1"><span class="liste-titre">' + F.e(m.nom) + '</span><span class="liste-sous">' + lotsDe(m).length + ' lots · ' + F.e(m.modeReversement) + '</span></span><span class="liste-droite"><span class="liste-montant">' + F.pourcent(mt.taux) + '</span><span class="muted" style="font-size:var(--text-sm)">' + baseTxt(mt.base) + '</span></span></span></button></li>'; }).join('');
        return '<section class="carte" aria-labelledby="t-m"><h2 id="t-m" class="sr-only">Liste des mandants</h2><div class="tab-bureau tableau-cadre"><table class="tableau"><thead><tr><th scope="col">Mandant</th><th scope="col">Commission</th><th scope="col" class="droite">Lots</th><th scope="col">Répartition</th><th scope="col">Téléphone</th><th scope="col">Reversement</th></tr></thead><tbody>' + lignes + '</tbody></table></div><ul class="tab-mobile">' + cartes + '</ul></section>';
      }
      var m = Data.mandant(id);
      if (!m) return '<div class="carte reserve">' + UI.etatVide({ ic: 'search', titre: 'Introuvable', texte: 'Cette page n’existe pas ou vous n’y avez pas accès.', action: UI.btn('Retour aux mandants', { v: 'primaire', href: '#/mandants' }) }) + '</div>';
      var mt = mandatDe(m), lots = lotsDe(m), crs = d.releves.filter(function (r) { return r.mandantId === m.id; }).sort(function (a, b) { return b.periode.localeCompare(a.periode); });
      var contact = '<section class="carte" aria-labelledby="t-ct"><div class="carte-entete"><h2 id="t-ct" class="carte-titre">' + UI.ic('user') + 'Contacts</h2></div><dl class="def-liste" style="margin:0"><div><dt>Téléphone</dt><dd>' + F.e(m.telephone) + '</dd></div><div><dt>Mode de reversement</dt><dd>' + F.e(m.modeReversement) + '</dd></div><div><dt>Portail mandant</dt><dd>' + UI.repere({ type: 'avenir', mini: true }) + '</dd></div></dl></section>';
      var mandat = '<section class="carte" aria-labelledby="t-mt"><div class="carte-entete"><h2 id="t-mt" class="carte-titre">' + UI.ic('briefcase') + 'Mandat ' + mt.id + '</h2>' + UI.neutre('Actif', 'check') + '</div><dl class="def-liste" style="margin:0"><div><dt>Taux de commission</dt><dd>' + F.pourcent(mt.taux) + '</dd></div><div><dt>Base de calcul</dt><dd>' + baseTxt(mt.base) + '</dd></div><div><dt>Depuis le</dt><dd>' + F.date(mt.dateDebut) + '</dd></div><div><dt>Lots couverts</dt><dd>' + lots.length + ' (' + repart(lots) + ')</dd></div></dl><div class="btn-groupe" style="margin-top:12px">' + UI.btn('Avenant de taux', { v: 'secondaire', petit: true, ic: 'pen', action: 'mand-avenant', data: ' data-id="' + m.id + '"' }) + UI.btn('Transférer des lots', { v: 'secondaire', petit: true, ic: 'refresh', action: 'mand-transferer', data: ' data-id="' + m.id + '"' }) + UI.btn('Clôturer le mandat', { v: 'tertiaire', petit: true, action: 'mand-cloturer', data: ' data-id="' + m.id + '"' }) + '</div></section>';
      var lotsH = '<section class="carte" aria-labelledby="t-lc"><div class="carte-entete"><h2 id="t-lc" class="carte-titre">' + UI.ic('building') + 'Lots couverts</h2></div><ul class="liste">' + lots.map(function (l) { var b = l.bailId ? Data.bail(l.bailId) : null; return '<li><div class="liste-gauche"><span class="liste-titre"><a href="#/lot/' + l.id + '">' + F.e(l.nom) + '</a></span><span class="liste-sous">' + F.e(Data.bien(l.bienId).quartier) + (b ? ' · ' + F.e(Data.nomLocataires(b.locataireIds)) : ' · ' + { vacant: 'vacant', travaux: 'en travaux' }[l.statut]) + '</span></div>' + UI.usage(l.usage) + '</li>'; }).join('') + '</ul></section>';
      var cr = '<section class="carte" aria-labelledby="t-cr2"><div class="carte-entete"><h2 id="t-cr2" class="carte-titre">' + UI.ic('chart') + 'Comptes rendus</h2></div><ul class="liste">' + crs.map(function (r) { return '<li><div class="liste-gauche"><span class="liste-titre"><a href="#/comptes-rendus/' + r.id + '">' + F.majuscule(F.mois(r.periode)) + '</a></span><span class="liste-sous">' + (r.statut === 'a_produire' ? 'À produire' : 'Net ' + F.montant(r.netAffiche !== undefined ? r.netAffiche : r.net)) + '</span></div>' + UI.neutre({ a_produire: 'À produire', brouillon: 'Brouillon', valide: 'Validé', paye: 'Payé' }[r.statut], r.statut === 'paye' ? 'check-circle' : 'clock') + '</li>'; }).join('') + '</ul></section>';
      return '<div class="lot-grille"><div class="lot-col">' + contact + mandat + cr + '</div><div class="lot-col">' + lotsH + '</div></div>';
    },
    apres: function (ctx, rac) { rac.addEventListener('keydown', function (ev) { if (ev.key === 'Enter' && ev.target.classList && ev.target.classList.contains('ligne-cliquable')) App.aller('/mandants/' + ev.target.getAttribute('data-id')); }); }
  });
})();
