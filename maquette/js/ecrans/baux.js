/*
 * S10 : Baux (liste, création en 3 étapes, fiche bail : échéancier, avenants, préavis, dépôt de garantie).
 * Niveau 2 : formulaires et garde-fous montrés, aucune donnée ajoutée. Garde-fous : chevauchement (R-020), lot en travaux (R-093),
 * loyer nul (R-022), fin avant début (R-021), dépôt supérieur à deux mois en habitation (R-024, non bloquant).
 * Éléments « livré pendant le pilote » : fin de bail et dépôt de garantie complet. « À venir » : indexation automatique (F30).
 */
(function () {
  var F = window.Fmt, UI = window.UI, Data = window.Data, App = window.App;
  var f = { q: '', statut: '', usage: '' }, w = null;
  function D() { return Data.org(); }
  var STAT = { actif: ['Actif', 'check'], preavis: ['En préavis', 'clock'], brouillon: ['Brouillon', 'pen'], termine: ['Terminé', 'ban'] };
  function badge(s) { return UI.neutre(STAT[s][0], STAT[s][1], s === 'brouillon' ? 'badge-pointille' : ''); }
  function alertes(b) {
    var a = [];
    if (b.statut === 'preavis') a.push(UI.neutre('Fin de bail : ' + F.date(b.preavis.dateDepart), 'clock'));
    if (b.depotAlerte) a.push('<span class="badge badge-due">' + UI.ic('alert-triangle') + 'Dépôt &gt; 2 mois</span>');
    if (b.avenants.length) a.push(UI.neutre('Avenant du ' + F.date(b.avenants[0].date).slice(0, 5), 'pen'));
    return a.join(' ') || '<span class="muted">—</span>';
  }
  function filtrer() {
    var n = F.norm(f.q);
    return D().baux.filter(function (b) {
      var lot = Data.lot(b.lotId);
      if (f.statut && b.statut !== f.statut) return false; if (f.usage && lot.usage !== f.usage) return false;
      return !n || F.norm(Data.nomLocataires(b.locataireIds) + ' ' + lot.nom).indexOf(n) >= 0;
    });
  }
  function htmlListe(l) {
    if (!l.length) return UI.etatVide({ ic: 'file-text', titre: 'Aucun bail', texte: 'Aucun bail ne correspond à ces filtres.', action: UI.btn('Créer un bail', { v: 'primaire', href: '#/baux/nouveau' }) });
    var lignes = l.map(function (b) { var lot = Data.lot(b.lotId); return '<tr class="ligne-cliquable" tabindex="0" data-action="bail-ouvrir" data-id="' + b.id + '"><td><span class="gras">' + F.e(Data.nomLocataires(b.locataireIds)) + '</span></td><td>' + F.e(lot.nom) + '<br><span class="muted" style="font-size:var(--text-sm)">' + F.e(Data.bien(lot.bienId).quartier) + '</span></td><td>' + badge(b.statut) + '</td><td class="nowrap">' + F.date(b.dateDebut) + '<br><span class="muted" style="font-size:var(--text-sm)">au ' + F.date(b.dateFin) + '</span></td><td class="droite">' + F.montant(b.loyer + b.charges) + '</td><td>le ' + b.jourEcheance + '</td><td>' + alertes(b) + '</td></tr>'; }).join('');
    var cartes = l.map(function (b) { var lot = Data.lot(b.lotId); return '<li><button type="button" class="carte-impaye" data-action="bail-ouvrir" data-id="' + b.id + '"><span class="ci-haut"><span class="liste-gauche" style="flex:1"><span class="liste-titre">' + F.e(Data.nomLocataires(b.locataireIds)) + '</span><span class="liste-sous">' + F.e(lot.nom) + ' · ' + F.montant(b.loyer + b.charges) + '</span></span><span class="liste-droite">' + badge(b.statut) + '</span></span></button></li>'; }).join('');
    return '<div class="tab-bureau tableau-cadre"><table class="tableau"><thead><tr><th scope="col">Locataire</th><th scope="col">Lot</th><th scope="col">Statut</th><th scope="col">Dates</th><th scope="col" class="droite">Loyer + charges</th><th scope="col">Échéance</th><th scope="col">Alertes</th></tr></thead><tbody>' + lignes + '</tbody></table></div><ul class="tab-mobile">' + cartes + '</ul>';
  }
  function maj() { var l = filtrer(); document.getElementById('bx-liste').innerHTML = htmlListe(l); document.getElementById('bx-compte').textContent = l.length + (l.length > 1 ? ' baux en cours' : ' bail en cours'); }
  App.action('bail-ouvrir', function (el) { App.aller('/bail/' + el.getAttribute('data-id')); });

  /* ---------- Création en 3 étapes ---------- */
  function neuf() { return { etape: 1, lot: '', loc: '', type: '', debut: '', fin: '', loyer: '', charges: '0', jour: '', depot: '', mois: 0 }; }
  function parseDate(t) { var m = /^(\d{2})\/(\d{2})\/(\d{4})$/.exec((t || '').trim()); if (!m) return null; var iso = m[3] + '-' + m[2] + '-' + m[1], d = new Date(iso + 'T00:00:00Z'); return isNaN(d) || d.toISOString().slice(0, 10) !== iso ? null : iso; }
  function entier(t) { return parseInt(String(t || '').replace(/\D/g, ''), 10) || 0; }
  function etapes() { return '<nav aria-label="Étapes de la création"><ol class="etapes">' + ['Lot et locataire', 'Conditions', 'Vérification'].map(function (e, i) { var n = i + 1, c = n < w.etape ? 'fait' : n === w.etape ? 'courant' : ''; return '<li class="etape ' + c + '"' + (n === w.etape ? ' aria-current="step"' : '') + '><span class="etape-n">' + (n < w.etape ? UI.ic('check', 'ic-sm') : n) + '</span><span class="etape-lib">' + e + '</span></li>'; }).join('') + '</ol></nav><p class="sr-only" aria-live="polite">Étape ' + w.etape + ' sur 3</p>'; }
  function etape1() {
    var d = D(), lots = d.lots.map(function (l) { return '<option value="' + l.id + '"' + (w.lot === l.id ? ' selected' : '') + '>' + F.e(l.nom + ' · ' + Data.bien(l.bienId).quartier) + { occupe: ' (occupé)', vacant: ' (vacant)', travaux: ' (en travaux)' }[l.statut] + '</option>'; }).join('');
    var locs = d.locataires.map(function (t) { return '<option value="' + t.id + '"' + (w.loc === t.id ? ' selected' : '') + '>' + F.e(t.nom) + '</option>'; }).join('');
    return '<section class="carte" aria-labelledby="t-b1"><h2 id="t-b1" class="carte-titre" style="margin-bottom:12px">' + UI.ic('building') + 'Lot et locataire</h2><div class="champ"><label class="champ-etiquette" for="bw-lot">Lot (obligatoire)</label><select class="saisie" id="bw-lot"><option value="">Choisir un lot</option>' + lots + '</select><div id="bw-lot-err" role="alert"></div></div>' +
      '<div class="champ"><label class="champ-etiquette" for="bw-loc">Locataire (obligatoire)</label><select class="saisie" id="bw-loc"><option value="">Choisir un locataire</option>' + locs + '</select><div class="champ-aide">Locataire absent ? <button type="button" class="lien-bouton" data-action="loc-creer">Créer un locataire</button></div><div class="champ-erreur cache" id="bw-loc-err" role="alert"></div></div>' +
      '<div class="btn-groupe" style="margin-top:20px">' + UI.btn('Continuer', { v: 'primaire', ic: 'arrow-right', action: 'bw-suite1' }) + UI.btn('Annuler', { v: 'tertiaire', href: '#/baux' }) + '</div></section>';
  }
  function etape2() {
    var lot = Data.lot(w.lot), types = lot.usage === 'mixte' ? ['habitation', 'commercial'] : [lot.usage];
    if (!w.type) w.type = types[0];
    var seg = types.map(function (t) { return '<input type="radio" name="bw-type" id="bt-' + t + '" value="' + t + '"' + (w.type === t ? ' checked' : '') + '><label for="bt-' + t + '">' + UI.ic('check', 'ic-sm coche') + F.majuscule(t) + '</label>'; }).join('');
    var champ = function (id, lib, val, ph, aide) { return '<div class="champ"><label class="champ-etiquette" for="' + id + '">' + lib + '</label><input class="saisie" id="' + id + '" value="' + F.e(val) + '" inputmode="numeric" autocomplete="off"' + (ph ? ' placeholder="' + ph + '"' : '') + '>' + (aide ? '<div class="champ-aide">' + aide + '</div>' : '') + '<div class="champ-erreur cache" id="' + id + '-err" role="alert"></div></div>'; };
    return '<section class="carte" aria-labelledby="t-b2"><h2 id="t-b2" class="carte-titre" style="margin-bottom:12px">' + UI.ic('file-text') + 'Conditions du bail</h2>' + (lot.usage === 'mixte' ? '<p class="champ-aide" style="margin-bottom:8px">Lot mixte : le type du bail détermine les règles applicables.</p>' : '') +
      '<fieldset class="champ" style="border:0;padding:0;margin:0"><legend class="champ-etiquette" style="padding:0;margin-bottom:6px">Type de bail</legend><div class="segmente" style="grid-template-columns:repeat(' + types.length + ',minmax(0,1fr))">' + seg + '</div></fieldset>' +
      champ('bw-debut', 'Date de début (JJ/MM/AAAA)', w.debut, '01/12/2026') + champ('bw-fin', 'Date de fin (JJ/MM/AAAA)', w.fin, '30/11/2029') + champ('bw-loyer', 'Loyer mensuel (FCFA)', w.loyer, '75 000') + champ('bw-charges', 'Charges mensuelles (FCFA)', w.charges, '0') +
      champ('bw-jour', 'Jour d’échéance', w.jour, '5', 'Sans jour d’échéance, le 1er du mois est retenu et signalé.') + champ('bw-depot', 'Dépôt de garantie (FCFA)', w.depot, '150 000') + '<div id="bw-depot-av" aria-live="polite"></div>' +
      '<div class="btn-groupe" style="margin-top:20px">' + UI.btn('Continuer', { v: 'primaire', ic: 'arrow-right', action: 'bw-suite2' }) + UI.btn('Retour', { v: 'tertiaire', action: 'bw-retour' }) + '</div></section>';
  }
  function etape3() {
    var lot = Data.lot(w.lot), t = Data.locataire(w.loc);
    return '<section class="carte" aria-labelledby="t-b3"><h2 id="t-b3" class="carte-titre" style="margin-bottom:12px">' + UI.ic('check-circle') + 'Vérification</h2><dl class="def-liste" style="margin:0"><div><dt>Lot</dt><dd>' + F.e(lot.nom) + '</dd></div><div><dt>Locataire</dt><dd>' + F.e(t.nom) + '</dd></div><div><dt>Type</dt><dd>' + F.majuscule(w.type) + '</dd></div><div><dt>Du / au</dt><dd>' + F.date(parseDate(w.debut)) + ' au ' + F.date(parseDate(w.fin)) + '</dd></div><div><dt>Loyer + charges</dt><dd class="num">' + F.montant(entier(w.loyer) + entier(w.charges)) + '</dd></div><div><dt>Échéance</dt><dd>le ' + (w.jour ? entier(w.jour) : '1 (par défaut)') + ' du mois</dd></div><div><dt>Dépôt de garantie</dt><dd class="num">' + F.montant(entier(w.depot)) + '</dd></div></dl>' +
      '<div class="champ"><label class="champ-etiquette" for="bw-piece">Pièce jointe : bail signé (facultatif)</label><input class="saisie" type="file" id="bw-piece" accept=".pdf,.jpg,.png"></div><div class="bandeau bandeau-info" style="margin-top:12px">' + UI.ic('info') + '<span>L’activation génère les échéances et passe le lot à « occupé ». Dans la démonstration, rien n’est créé.</span></div>' +
      '<div class="btn-groupe" style="margin-top:20px">' + UI.btn('Activer le bail', { v: 'primaire', ic: 'check', action: 'bw-activer' }) + UI.btn('Retour', { v: 'tertiaire', action: 'bw-retour' }) + '</div></section>';
  }
  function afficher() { document.getElementById('bw').innerHTML = etapes() + (w.etape === 1 ? etape1() : w.etape === 2 ? etape2() : etape3()); if (w.etape === 2) { var m = function () { var l = entier(document.getElementById('bw-loyer').value), dp = entier(document.getElementById('bw-depot').value), t = (document.querySelector('input[name="bw-type"]:checked') || {}).value; document.getElementById('bw-depot-av').innerHTML = t === 'habitation' && l > 0 && dp > 2 * l ? '<div class="bandeau bandeau-avert" style="margin-top:8px">' + UI.ic('alert-triangle') + '<span>Le dépôt de garantie dépasse deux mois de loyer. Vérifiez qu’il respecte la loi sur le bail d’habitation. Vous pouvez continuer.</span></div>' : ''; }; ['bw-loyer', 'bw-depot'].forEach(function (i) { document.getElementById(i).addEventListener('input', m); }); document.querySelectorAll('input[name="bw-type"]').forEach(function (r) { r.addEventListener('change', m); }); m(); } }
  function err(id, t) { var e = document.getElementById(id); if (!e) return true; e.classList.toggle('cache', !t); e.innerHTML = t ? UI.ic('alert-circle', 'ic-sm') + '<span>' + t + '</span>' : ''; return !t; }
  App.action('bw-retour', function () { w.etape -= 1; afficher(); });
  App.action('bw-suite1', function () {
    w.lot = document.getElementById('bw-lot').value; w.loc = document.getElementById('bw-loc').value; var ok = true, z = document.getElementById('bw-lot-err'); z.innerHTML = '';
    if (!w.lot) { z.innerHTML = '<div class="champ-erreur" style="margin-top:6px">' + UI.ic('alert-circle', 'ic-sm') + '<span>Choisissez un lot.</span></div>'; ok = false; }
    else { var lot = Data.lot(w.lot);
      if (lot.statut === 'occupe') { var b = Data.bail(lot.bailId); z.innerHTML = '<div class="bandeau bandeau-erreur" style="margin-top:8px">' + UI.ic('alert-circle') + '<span>Ce lot a déjà un bail actif du ' + F.date(b.dateDebut) + ' au ' + F.date(b.dateFin) + '. Terminez-le ou choisissez d’autres dates.</span></div>'; ok = false; }
      if (lot.statut === 'travaux') { z.innerHTML = '<div class="bandeau bandeau-erreur" style="margin-top:8px">' + UI.ic('alert-circle') + '<div>Ce lot est en travaux. Remettez-le à « vacant » pour activer un bail.<div style="margin-top:8px"><button type="button" class="btn btn-secondaire btn-petit" data-action="demo">Retirer « en travaux »</button></div></div></div>'; ok = false; } }
    ok = err('bw-loc-err', w.loc ? '' : 'Choisissez un locataire.') && ok; if (!ok) return; w.type = ''; w.etape = 2; afficher();
  });
  App.action('bw-suite2', function () {
    var g = function (i) { return document.getElementById(i).value; }; w.debut = g('bw-debut'); w.fin = g('bw-fin'); w.loyer = g('bw-loyer'); w.charges = g('bw-charges'); w.jour = g('bw-jour'); w.depot = g('bw-depot'); w.type = document.querySelector('input[name="bw-type"]:checked').value;
    var di = parseDate(w.debut), fi = parseDate(w.fin), ok = true;
    ok = err('bw-debut-err', di ? '' : 'Saisissez une date au format JJ/MM/AAAA.') && ok; ok = err('bw-fin-err', !fi ? 'Saisissez une date au format JJ/MM/AAAA.' : di && fi < di ? 'La date de fin ne peut pas précéder la date de début.' : '') && ok;
    ok = err('bw-loyer-err', entier(w.loyer) > 0 ? '' : 'Le loyer doit être supérieur à 0 FCFA.') && ok; var j = entier(w.jour); ok = err('bw-jour-err', !w.jour || (j >= 1 && j <= 31) ? '' : 'Le jour d’échéance est compris entre 1 et 31.') && ok;
    if (!ok) { var p = document.querySelector('#bw .champ-erreur:not(.cache)'); if (p) p.parentNode.querySelector('input').focus(); return; } w.etape = 3; afficher(); window.scrollTo(0, 0);
  });
  App.action('bw-activer', function () { UI.toast('Bail créé (démonstration). Les échéances seraient générées et le lot passerait à « occupé ».', { type: 'succes' }); App.aller('/baux'); });

  /* ---------- Fiche bail ---------- */
  function modalAvenant(b) {
    UI.modale({ titre: 'Ajouter un avenant', corps: '<p class="secondaire">Un changement de loyer ou de charges est un avenant daté. Il n’est jamais rétroactif : les échéances passées ne changent pas.</p><div class="champ" style="margin-top:12px"><label class="champ-etiquette" for="av-date">Date d’effet (JJ/MM/AAAA)</label><input class="saisie" id="av-date" value="01/12/2026" inputmode="numeric"></div><div class="champ"><label class="champ-etiquette" for="av-loyer">Nouveau loyer (FCFA)</label><input class="saisie" id="av-loyer" value="' + F.nombre(b.loyer) + '" inputmode="numeric"><div class="champ-erreur cache" id="av-err" role="alert"></div></div>',
      actions: [{ lib: 'Annuler', v: 'secondaire' }, { lib: 'Enregistrer l’avenant', v: 'primaire', focus: true, fn: function (fermer) { if (!entier(document.getElementById('av-loyer').value)) { err('av-err', 'Le loyer doit être supérieur à 0 FCFA.'); return; } fermer(); UI.toast('Avenant enregistré (démonstration). Les échéances passées gardent l’ancien montant.', { type: 'succes' }); } }] });
  }
  App.action('bail-avenant', function (el) { modalAvenant(Data.bail(el.getAttribute('data-id'))); });
  App.action('bail-action', function (el) {
    var t = el.getAttribute('data-t'), cfg = { preavis: ['Noter un préavis', 'Saisissez la date de départ prévue. Le bail reste en cours : les échéances continuent jusqu’au départ. La durée du préavis est saisie au bail, sans valeur par défaut.', 'Enregistrer le préavis'], renouveler: ['Renouveler le bail', 'Le renouvellement est un avenant daté qui prolonge la date de fin. Même bail, même dépôt de garantie, même historique.', 'Renouveler'], resilier: ['Résilier le bail ?', 'Le bail sera terminé, le lot repassera à « vacant » et aucune nouvelle échéance ne sera générée.', 'Résilier le bail'], retenue: ['Enregistrer une retenue sur le dépôt', 'Toute retenue est motivée et tracée.', 'Enregistrer la retenue'] }[t];
    var champ = t === 'retenue' ? '<div class="champ" style="margin-top:12px"><label class="champ-etiquette" for="ac-motif">Motif (obligatoire)</label><textarea class="saisie" id="ac-motif"></textarea><div class="champ-erreur cache" id="ac-err" role="alert"></div></div>' : t === 'resilier' ? '' : '<div class="champ" style="margin-top:12px"><label class="champ-etiquette" for="ac-date">Date (JJ/MM/AAAA)</label><input class="saisie" id="ac-date" inputmode="numeric" placeholder="31/01/2027"><div class="champ-erreur cache" id="ac-err" role="alert"></div></div>';
    UI.modale({ titre: cfg[0], corps: '<p class="secondaire">' + cfg[1] + '</p>' + champ, actions: [{ lib: 'Annuler', v: 'secondaire', focus: t === 'resilier' }, { lib: cfg[2], v: 'primaire', fn: function (fermer) {
      if (t === 'retenue' && !document.getElementById('ac-motif').value.trim()) { err('ac-err', 'Indiquez le motif de la retenue.'); return; }
      if ((t === 'preavis' || t === 'renouveler') && !parseDate(document.getElementById('ac-date').value)) { err('ac-err', 'Saisissez une date au format JJ/MM/AAAA.'); return; }
      fermer(); UI.toast('Action de démonstration. Le bail n’est pas modifié.', { type: 'info' }); } }] });
  });

  App.enregistrer('baux', {
    meta: function (ctx) {
      var p = ctx.params[0];
      if (p === 'nouveau') return { titre: 'Créer un bail', sous: 'Trois étapes : lot et locataire, conditions, vérification.', repere: { type: 'v1' } };
      return { titre: 'Baux', sous: 'Créer et suivre les baux, leur échéancier, leur fin.', repere: { type: 'v1' }, actions: UI.btn('Créer un bail', { v: 'primaire', ic: 'plus', href: '#/baux/nouveau' }) + '<span class="muted" style="display:inline-flex;align-items:center;gap:6px">' + UI.btn('Indexation automatique du loyer', { v: 'secondaire', petit: true, disabled: true }) + UI.repere({ type: 'avenir', mini: true }) + '</span>' };
    },
    rendre: function (ctx) {
      if (ctx.params[0] === 'nouveau') { w = neuf(); return '<div id="bw"></div>'; }
      f = { q: '', statut: '', usage: '' }; var l = filtrer();
      return '<section class="carte" aria-labelledby="t-bx"><h2 id="t-bx" class="sr-only">Liste des baux</h2><div class="filtres"><div class="champ" style="flex:2 1 240px"><label class="champ-etiquette" for="bx-q">Rechercher</label><div style="position:relative"><span style="position:absolute;left:12px;top:50%;transform:translateY(-50%);color:var(--color-text-muted)">' + UI.ic('search') + '</span><input class="saisie" id="bx-q" type="search" style="padding-left:40px" placeholder="Locataire ou lot" autocomplete="off"></div></div><div class="champ" style="flex:1 1 170px"><label class="champ-etiquette" for="bx-s">Statut</label><select class="saisie" id="bx-s"><option value="">Tous</option><option value="actif">Actif</option><option value="preavis">En préavis</option></select></div><div class="champ" style="flex:1 1 170px"><label class="champ-etiquette" for="bx-u">Usage</label><select class="saisie" id="bx-u"><option value="">Tous</option><option value="habitation">Habitation</option><option value="commercial">Commercial</option><option value="mixte">Mixte</option></select></div></div>' +
        '<p class="secondaire" style="margin:8px 0 12px" id="bx-compte" aria-live="polite">' + l.length + ' baux en cours</p><div id="bx-liste">' + htmlListe(l) + '</div></section>';
    },
    apres: function (ctx, rac) {
      if (ctx.params[0] === 'nouveau') { afficher(); return; }
      var q = rac.querySelector('#bx-q'), t = null; q.addEventListener('input', function () { clearTimeout(t); t = setTimeout(function () { f.q = q.value; maj(); }, 200); });
      rac.querySelector('#bx-s').addEventListener('change', function (e) { f.statut = e.target.value; maj(); }); rac.querySelector('#bx-u').addEventListener('change', function (e) { f.usage = e.target.value; maj(); });
      rac.addEventListener('keydown', function (ev) { if (ev.key === 'Enter' && ev.target.classList && ev.target.classList.contains('ligne-cliquable')) App.aller('/bail/' + ev.target.getAttribute('data-id')); });
    }
  });

  App.enregistrer('bail', {
    meta: function (ctx) {
      var b = Data.bail(ctx.params[0]); if (!b) return { titre: 'Introuvable', repere: { type: 'v1' } };
      return { titre: 'Bail · ' + F.e(Data.lot(b.lotId).nom), sous: F.e(Data.nomLocataires(b.locataireIds)) + ' · ' + F.majuscule(b.type), repere: { type: 'v1' }, actions: '<span style="display:flex;gap:8px;flex-wrap:wrap;align-items:center">' + badge(b.statut) + '</span>' + UI.btn('Tous les baux', { v: 'tertiaire', ic: 'chevron-left', href: '#/baux' }) };
    },
    rendre: function (ctx) {
      var b = Data.bail(ctx.params[0]);
      if (!b) return '<div class="carte reserve">' + UI.etatVide({ ic: 'search', titre: 'Introuvable', texte: 'Cette page n’existe pas ou vous n’y avez pas accès.', action: UI.btn('Retour aux baux', { v: 'primaire', href: '#/baux' }) }) + '</div>';
      var d = D(), lot = Data.lot(b.lotId), es = d.echeances.filter(function (e) { return e.bailId === b.id; }).sort(function (a, c) { return a.dateEcheance.localeCompare(c.dateEcheance); });
      var cond = '<section class="carte" aria-labelledby="t-cond"><div class="carte-entete"><h2 id="t-cond" class="carte-titre">' + UI.ic('file-text') + 'Conditions</h2></div><dl class="def-liste" style="margin:0"><div><dt>Locataire(s)</dt><dd>' + b.locataireIds.map(function (id) { return '<a href="#/locataire/' + id + '">' + F.e(Data.locataire(id).nom) + '</a>'; }).join(' et ') + '</dd></div><div><dt>Lot</dt><dd><a href="#/lot/' + lot.id + '">' + F.e(lot.nom) + '</a></dd></div><div><dt>Du / au</dt><dd>' + F.date(b.dateDebut) + ' au ' + F.date(b.dateFin) + '</dd></div><div><dt>Loyer</dt><dd class="num">' + F.montant(b.loyer) + '</dd></div><div><dt>Charges</dt><dd class="num">' + F.montant(b.charges) + '</dd></div><div><dt>Jour d’échéance</dt><dd>le ' + b.jourEcheance + ' du mois</dd></div>' + (b.tvaIncluse ? '<div><dt>TVA</dt><dd>TVA incluse</dd></div>' : '') + (b.commercial ? '<div><dt>Conditions commerciales</dt><dd>' + F.e(b.commercial.duree) + ' · ' + F.e(b.commercial.indexation) + '</dd></div>' : '') + '</dl>' + (b.commercial ? '<p class="muted" style="margin-top:12px;font-size:var(--text-sm)">' + UI.repere({ type: 'avenir', mini: true }) + ' Indexation automatique du loyer : pour l’instant, une alerte « révision à étudier » vous prévient.</p>' : '') + '</section>';
      var ech = '<section class="carte" aria-labelledby="t-ech"><div class="carte-entete"><h2 id="t-ech" class="carte-titre">' + UI.ic('calendar') + 'Échéancier</h2></div><ul class="liste">' + es.map(function (e) { return '<li><div class="liste-gauche"><span class="liste-titre">' + F.majuscule(F.mois(e.periode)) + '</span><span class="liste-sous">Échéance du ' + F.date(e.dateEcheance) + ' · dû ' + F.montant(e.montant) + '</span></div><div class="liste-droite"><span class="liste-montant">' + F.montant(e.paye) + '</span>' + (e.statut === 'en_retard' ? UI.statut('en_retard', F.jours(e.ancienneteJours)) : e.statut === 'due' ? UI.statut('due', 'aujourd’hui') : UI.statut(e.statut)) + '</div></li>'; }).join('') + '</ul></section>';
      var av = '<section class="carte" aria-labelledby="t-av"><div class="carte-entete"><h2 id="t-av" class="carte-titre">' + UI.ic('pen') + 'Avenants</h2>' + UI.btn('Ajouter un avenant', { v: 'secondaire', petit: true, ic: 'plus', action: 'bail-avenant', data: ' data-id="' + b.id + '"' }) + '</div>' + (b.avenants.length ? '<ul class="liste">' + b.avenants.map(function (a) { return '<li><div class="liste-gauche"><span class="liste-titre">Au ' + F.date(a.date) + '</span><span class="liste-sous">Loyer ' + F.montant(a.loyerAvant) + ' → ' + F.montant(a.loyerApres) + (a.chargesAvant !== a.chargesApres ? ' · charges ' + F.montant(a.chargesAvant) + ' → ' + F.montant(a.chargesApres) : '') + '</span></div>' + UI.neutre('Non rétroactif', 'info') + '</li>'; }).join('') + '</ul><p class="muted" style="font-size:var(--text-sm);margin-top:8px">Les échéances de septembre et d’octobre gardent l’ancien montant.</p>' : '<p class="secondaire">Aucun avenant.</p>') + '</section>';
      var fin = '<section class="carte" aria-labelledby="t-fin"><div class="carte-entete"><h2 id="t-fin" class="carte-titre">' + UI.ic('clock') + 'Préavis et fin de bail</h2>' + UI.repere({ type: 'v1', pilote: true, mini: true }) + '</div>' + (b.preavis ? '<div class="bandeau bandeau-info" style="margin-bottom:12px">' + UI.ic('clock') + '<span>Préavis noté le ' + F.date(b.preavis.saisiLe) + '. Départ prévu le ' + F.date(b.preavis.dateDepart) + ' : les échéances continuent jusqu’au départ.</span></div>' : '<p class="secondaire" style="margin-bottom:12px">Aucun préavis noté.</p>') + '<div class="btn-groupe">' + (b.preavis ? '' : UI.btn('Noter un préavis', { v: 'secondaire', petit: true, action: 'bail-action', data: ' data-t="preavis"' })) + UI.btn('Renouveler', { v: 'secondaire', petit: true, action: 'bail-action', data: ' data-t="renouveler"' }) + UI.btn('Résilier', { v: 'tertiaire', petit: true, action: 'bail-action', data: ' data-t="resilier"' }) + UI.btn('Imprimer la fiche de bail', { v: 'tertiaire', petit: true, ic: 'printer', action: 'demo' }) + '</div></section>';
      var dep = '<section class="carte" aria-labelledby="t-dep"><div class="carte-entete"><h2 id="t-dep" class="carte-titre">' + UI.ic('wallet') + 'Dépôt de garantie</h2>' + UI.repere({ type: 'v1', pilote: true, mini: true }) + '</div><dl class="def-liste" style="margin:0"><div><dt>Montant reçu</dt><dd class="num">' + F.montant(b.depotGarantie) + '</dd></div><div><dt>Reçu le</dt><dd>' + F.date(b.dateDebut) + '</dd></div><div><dt>Retenues</dt><dd>Aucune</dd></div><div><dt>Restitution</dt><dd class="muted">À la fin du bail</dd></div></dl>' + (b.depotAlerte ? '<div class="bandeau bandeau-avert" style="margin-top:12px">' + UI.ic('alert-triangle') + '<span>Le dépôt de garantie dépasse deux mois de loyer. Vérifiez qu’il respecte la loi sur le bail d’habitation. Vous pouvez continuer.</span></div>' : '') + '<div class="btn-groupe" style="margin-top:12px">' + UI.btn('Enregistrer une retenue', { v: 'secondaire', petit: true, action: 'bail-action', data: ' data-t="retenue"' }) + '</div></section>';
      return '<div class="lot-grille"><div class="lot-col">' + cond + av + fin + '</div><div class="lot-col">' + ech + dep + '</div></div>';
    }
  });
})();
