/*
 * S08 : Locataires (liste, création) · S09 : Fiche locataire (onglets Aperçu, Baux, Paiements, Relances, Documents).
 * Niveau 2 : listes filtrables sur le jeu de données, formulaires qui s'ouvrent et se ferment, boutons d'action en « Action de démonstration ».
 * Garde-fous montrés : téléphone invalide (R-018), doublon de téléphone (R-017), pièce d'identité masquée (R-019).
 */
(function () {
  var F = window.Fmt, UI = window.UI, Data = window.Data, App = window.App, Roles = window.Roles;
  var f = { q: '', statut: '' };
  function D() { return Data.org(); }
  function lecture(ctx) { return Roles.droit(ctx.role.id, 'locataires') !== 'E'; }
  function statutHtml(s) {
    if (s.statut === 'payee') return UI.statut('payee', 'à jour');
    if (s.statut === 'en_retard') return UI.statut('en_retard', F.jours(s.anciennete));
    return s.statut === 'due' ? UI.statut('due', 'aujourd’hui') : UI.statut(s.statut);
  }
  function lotsDe(t) { return D().baux.filter(function (b) { return b.locataireIds.indexOf(t.id) >= 0; }).map(function (b) { return Data.lot(b.lotId); }); }

  function filtrer() {
    var n = F.norm(f.q), chiffres = f.q.replace(/\D/g, '');
    return D().locataires.filter(function (t) {
      var s = Data.situation(t.id);
      if (f.statut === 'retard' && s.statut !== 'en_retard') return false;
      if (f.statut === 'jour' && s.statut !== 'payee') return false;
      if (f.statut === 'du' && (s.statut === 'payee' || s.statut === 'en_retard')) return false;
      if (!n) return true;
      return F.norm(t.nom + ' ' + (t.activite || '') + ' ' + lotsDe(t).map(function (l) { return l.nom; }).join(' ')).indexOf(n) >= 0 || (chiffres.length >= 2 && t.telephone.replace(/\D/g, '').indexOf(chiffres) >= 0);
    }).sort(function (a, b) { return a.nom.localeCompare(b.nom); });
  }
  function htmlListe(l) {
    if (!l.length) return UI.etatVide({ ic: 'search', titre: 'Aucun résultat', texte: 'Vérifiez l’orthographe ou cherchez par numéro de téléphone.', action: UI.btn('Créer un locataire', { v: 'secondaire', action: 'loc-creer' }) });
    var lignes = l.map(function (t) {
      var s = Data.situation(t.id), lots = lotsDe(t);
      return '<tr class="ligne-cliquable" tabindex="0" data-action="loc-ouvrir" data-id="' + t.id + '"><td><span class="gras">' + F.e(t.nom) + '</span>' + (t.activite ? '<br><span class="muted" style="font-size:var(--text-sm)">' + F.e(t.activite) + '</span>' : '') + '</td><td class="nowrap">' + F.e(t.telephone) + (t.whatsapp ? '' : '<br><span class="muted" style="font-size:var(--text-sm)">pas de WhatsApp</span>') + '</td><td>' + lots.map(function (x) { return F.e(x.nom); }).join('<br>') + '</td><td class="droite">' + (s.solde ? '<span class="gras">' + F.montant(s.solde) + '</span>' : '<span class="muted">0 FCFA</span>') + '</td><td>' + statutHtml(s) + '</td></tr>';
    }).join('');
    var cartes = l.map(function (t) {
      var s = Data.situation(t.id), lots = lotsDe(t);
      return '<li><button type="button" class="carte-impaye" data-action="loc-ouvrir" data-id="' + t.id + '"><span class="ci-haut"><span class="liste-gauche" style="flex:1"><span class="liste-titre">' + F.e(t.nom) + '</span><span class="liste-sous">' + lots.map(function (x) { return F.e(x.nom); }).join(', ') + '</span></span><span class="liste-droite">' + (s.solde ? '<span class="liste-montant">' + F.montant(s.solde) + '</span>' : '') + statutHtml(s) + '</span></span></button></li>';
    }).join('');
    return '<div class="tab-bureau tableau-cadre"><table class="tableau"><thead><tr><th scope="col">Nom</th><th scope="col">Téléphone</th><th scope="col">Lot(s)</th><th scope="col" class="droite">Solde</th><th scope="col">Situation</th></tr></thead><tbody>' + lignes + '</tbody></table></div><ul class="tab-mobile">' + cartes + '</ul>';
  }
  function maj() { var l = filtrer(); document.getElementById('loc-liste').innerHTML = htmlListe(l); document.getElementById('loc-compte').textContent = l.length + (l.length > 1 ? ' locataires' : ' locataire'); }

  /* ---------- Création (formulaire de démonstration) ---------- */
  function creer() {
    UI.modale({ titre: 'Créer un locataire', corps:
      '<fieldset class="champ" style="border:0;padding:0;margin:0"><legend class="champ-etiquette" style="padding:0;margin-bottom:6px">Type</legend><div class="segmente" style="grid-template-columns:repeat(2,minmax(0,1fr))"><input type="radio" name="lt" id="lt-p" checked><label for="lt-p">' + UI.ic('check', 'ic-sm coche') + 'Personne</label><input type="radio" name="lt" id="lt-s"><label for="lt-s">' + UI.ic('check', 'ic-sm coche') + 'Société ou activité</label></div></fieldset>' +
      '<div class="champ"><label class="champ-etiquette" for="lc-nom">Nom (obligatoire)</label><input class="saisie" id="lc-nom" autocomplete="off"><div class="champ-erreur cache" id="lc-nom-err" role="alert"></div></div>' +
      '<div class="champ"><label class="champ-etiquette" for="lc-tel">Téléphone (obligatoire)</label><div class="saisie-montant" style="min-height:48px"><span class="suffixe" style="padding:0 4px 0 14px;color:var(--color-text)">+226</span><input id="lc-tel" inputmode="numeric" style="font-size:var(--text-base);text-align:left;font-weight:400" placeholder="70 12 34 56" autocomplete="off"></div><div class="champ-erreur cache" id="lc-tel-err" role="alert"></div></div>' +
      '<div class="champ"><label style="display:flex;gap:10px;align-items:center;min-height:44px"><input type="checkbox" id="lc-wa" checked><span>Ce numéro a WhatsApp</span></label></div>' +
      '<div class="champ"><label class="champ-etiquette" for="lc-sec">Contact de secours (facultatif)</label><input class="saisie" id="lc-sec" autocomplete="off"></div>',
      actions: [{ lib: 'Annuler', v: 'secondaire' }, { lib: 'Créer le locataire', v: 'primaire', focus: true, fn: function (fermer) {
        var nom = document.getElementById('lc-nom').value.trim(), tel = document.getElementById('lc-tel').value.replace(/\D/g, ''), ok = true;
        var err = function (id, t) { var e = document.getElementById(id); e.classList.toggle('cache', !t); e.innerHTML = t || ''; if (t) ok = false; };
        err('lc-nom-err', nom ? '' : 'Indiquez le nom du locataire.'); err('lc-tel-err', '');
        if (tel.length !== 8) err('lc-tel-err', 'Numéro invalide. Format attendu : +226 suivi du numéro (exemple : +226 70 12 34 56).');
        else { var dup = D().locataires.filter(function (t) { return t.telephone.replace(/\D/g, '').slice(-8) === tel; })[0];
          if (dup) { err('lc-tel-err', 'Un locataire avec ce numéro existe déjà : ' + F.e(dup.nom) + '. <a href="#/locataire/' + dup.id + '">Ouvrir la fiche</a> · <button type="button" class="lien-bouton" data-action="demo">Fusionner</button>'); } }
        if (!ok) return;
        fermer(); UI.toast('Locataire créé (démonstration). Aucune donnée n’est ajoutée.', { type: 'succes' });
      } }] });
    document.getElementById('lc-nom').focus();
  }
  App.action('loc-creer', creer);
  App.action('loc-ouvrir', function (el) { App.aller('/locataire/' + el.getAttribute('data-id')); });
  App.action('loc-effacer', function () { f = { q: '', statut: '' }; document.getElementById('loc-q').value = ''; document.getElementById('loc-statut').value = ''; maj(); });

  App.enregistrer('locataires', {
    meta: function (ctx) { var l = lecture(ctx); return { titre: 'Locataires', sous: l ? 'Lecture seule : vous retrouvez un locataire sans pouvoir le modifier.' : 'Retrouvez un locataire, sa situation et son historique.', repere: { type: 'v1' }, actions: l ? '' : UI.btn('Créer un locataire', { v: 'secondaire', ic: 'plus', action: 'loc-creer' }) + UI.btn('Fusionner les doublons', { v: 'tertiaire', ic: 'users', action: 'demo' }) }; },
    rendre: function (ctx) {
      f = { q: ctx.query.q || '', statut: '' }; var l = filtrer();
      return '<section class="carte" aria-labelledby="t-loc"><h2 id="t-loc" class="sr-only">Liste des locataires</h2><div class="filtres"><div class="champ" style="flex:2 1 240px"><label class="champ-etiquette" for="loc-q">Rechercher</label><div style="position:relative"><span style="position:absolute;left:12px;top:50%;transform:translateY(-50%);color:var(--color-text-muted)">' + UI.ic('search') + '</span><input class="saisie" id="loc-q" type="search" style="padding-left:40px" value="' + F.e(f.q) + '" placeholder="Nom, téléphone, lot" autocomplete="off"></div></div>' +
        '<div class="champ" style="flex:1 1 180px"><label class="champ-etiquette" for="loc-statut">Situation</label><select class="saisie" id="loc-statut"><option value="">Toutes</option><option value="retard">En retard</option><option value="du">À régler</option><option value="jour">À jour</option></select></div></div>' +
        '<p class="secondaire" style="margin:8px 0 12px" id="loc-compte" aria-live="polite">' + l.length + ' locataires</p><div id="loc-liste">' + htmlListe(l) + '</div></section>';
    },
    apres: function (ctx, rac) {
      var q = rac.querySelector('#loc-q'), t = null; q.addEventListener('input', function () { clearTimeout(t); t = setTimeout(function () { f.q = q.value; maj(); }, 200); });
      rac.querySelector('#loc-statut').addEventListener('change', function (e) { f.statut = e.target.value; maj(); });
      rac.addEventListener('keydown', function (ev) { if (ev.key === 'Enter' && ev.target.classList && ev.target.classList.contains('ligne-cliquable')) App.aller('/locataire/' + ev.target.getAttribute('data-id')); });
    }
  });

  /* ---------- S09 ---------- */
  var ICONE_MODE = { especes: 'banknote', mobile_money: 'phone', virement: 'building', cheque: 'file-text' };
  App.action('loc-piece', function (el) {
    UI.modale({ titre: 'Afficher la pièce d’identité ?', corps: '<p>La pièce est masquée par défaut. Chaque consultation est tracée dans le journal d’audit.</p>',
      actions: [{ lib: 'Annuler', v: 'secondaire', focus: true }, { lib: 'Afficher', v: 'primaire', fn: function (fermer) { fermer(); var z = document.getElementById('piece'); if (z) z.innerHTML = '<span class="gras">CNIB n° B•••••••• (affichage fictif)</span> <button type="button" class="lien-bouton" data-action="loc-piece-masquer">Masquer</button>'; } }] });
  });
  App.action('loc-piece-masquer', function () { var z = document.getElementById('piece'); if (z) z.innerHTML = 'Fournie · masquée <button type="button" class="btn btn-tertiaire btn-petit" data-action="loc-piece">' + UI.ic('eye-off', 'ic-sm') + ' Afficher</button>'; });
  App.action('loc-relancer', function (el) { var bail = el.getAttribute('data-bail'); if (bail) window.Relance.ouvrir(bail, 'retard'); });

  App.enregistrer('locataire', {
    meta: function (ctx) {
      var t = Data.locataire(ctx.params[0]); if (!t) return { titre: 'Introuvable', repere: { type: 'v1' } };
      var s = Data.situation(t.id), lots = lotsDe(t), peutEnc = Roles.droit(ctx.role.id, 'encaisser') === 'E', peutRel = Roles.droit(ctx.role.id, 'impayes') === 'E' && s.retard.length;
      var bailRetard = s.retard.length ? s.retard[0].bailId : null;
      return { titre: F.e(t.nom), sous: lots.map(function (x) { return F.e(x.nom); }).join(' · ') + (t.activite ? ' · ' + F.e(t.activite) : ''), repere: { type: 'v1' },
        actions: (peutEnc ? UI.btn('Encaisser', { v: 'primaire', ic: 'banknote', href: '#/encaisser?q=' + encodeURIComponent(t.nomFamille) }) : '') + (peutRel ? UI.btn('Relancer', { v: 'secondaire', ic: 'message', action: 'loc-relancer', data: ' data-bail="' + bailRetard + '"' }) : '') + UI.btn('Appeler', { v: 'secondaire', ic: 'phone', action: 'demo' }) + UI.btn('WhatsApp', { v: 'secondaire', ic: 'message', action: 'demo' }) };
    },
    rendre: function (ctx) {
      var t = Data.locataire(ctx.params[0]);
      if (!t) return '<div class="carte reserve">' + UI.etatVide({ ic: 'search', titre: 'Introuvable', texte: 'Cette page n’existe pas ou vous n’y avez pas accès.', action: UI.btn('Retour aux locataires', { v: 'primaire', href: '#/locataires' }) }) + '</div>';
      var d = D(), on = ctx.query.onglet || 'apercu', s = Data.situation(t.id), baux = d.baux.filter(function (b) { return b.locataireIds.indexOf(t.id) >= 0; });
      var pays = d.paiements.filter(function (p) { return p.type === 'paiement' && p.locataireIds.indexOf(t.id) >= 0; }).sort(function (a, c) { return (c.dateSaisie + c.heure).localeCompare(a.dateSaisie + a.heure); });
      var rels = d.relances.filter(function (r) { return r.locataireId === t.id; }).sort(function (a, c) { return (c.date + c.heure).localeCompare(a.date + a.heure); });
      var docs = d.documents.filter(function (x) { return x.objets.some(function (o) { return o.kind === 'bail' && baux.some(function (b) { return b.id === o.id; }); }); });
      var tabs = [['apercu', 'Aperçu'], ['baux', 'Baux', baux.length], ['paiements', 'Paiements', pays.length], ['relances', 'Relances', rels.length], ['documents', 'Documents', docs.length]];
      var onglets = '<div class="onglets" role="tablist">' + tabs.map(function (x) { return '<a class="onglet" role="tab" href="#/locataire/' + t.id + (x[0] === 'apercu' ? '' : '?onglet=' + x[0]) + '" aria-selected="' + (x[0] === on) + '">' + x[1] + (x[2] !== undefined ? ' <span class="badge badge-neutre" style="min-height:22px;padding:0 8px">' + x[2] + '</span>' : '') + '</a>'; }).join('') + '</div>';
      var corps;
      if (on === 'baux') corps = '<section class="carte"><ul class="liste">' + baux.map(function (b) { var lot = Data.lot(b.lotId); return '<li><div class="liste-gauche"><span class="liste-titre"><a href="#/bail/' + b.id + '">' + F.e(lot.nom) + ', ' + F.e(Data.bien(lot.bienId).quartier) + '</a></span><span class="liste-sous">Du ' + F.date(b.dateDebut) + ' au ' + F.date(b.dateFin) + ' · loyer ' + F.montant(b.loyer) + '</span></div>' + UI.neutre(b.statut === 'preavis' ? 'En préavis' : 'Actif', b.statut === 'preavis' ? 'clock' : 'check') + '</li>'; }).join('') + '</ul></section>';
      else if (on === 'paiements') corps = '<section class="carte">' + (pays.length ? '<ol class="chrono">' + pays.map(function (p) { var q = p.quittanceId ? Data.quittance(p.quittanceId) : null; return '<li class="chrono-item"><div class="chrono-date"><span class="gras">' + F.date(p.date) + '</span></div><a class="chrono-corps" href="#/paiements?ouvrir=' + p.id + '"><span class="chrono-haut"><span class="gras num">' + F.montant(p.montant) + '</span>' + (p.statut === 'corrige' ? UI.statut('corrige') : UI.statut('valide')) + '</span><span class="chrono-bas"><span>' + UI.ic(ICONE_MODE[p.mode], 'ic-sm') + ' ' + Data.modeLibelle(p.mode) + '</span><span>' + F.e(Data.lot(p.lotId).nom) + '</span>' + (q ? '<span>' + F.e(q.numero) + '</span>' : '') + '</span></a></li>'; }).join('') + '</ol>' : UI.etatVide({ ic: 'receipt', titre: 'Aucun paiement', texte: 'Aucun paiement sur cette période.' })) + '</section>';
      else if (on === 'relances') corps = '<section class="carte">' + (rels.length ? '<ol class="chrono">' + rels.map(function (r) { return '<li class="chrono-item"><div class="chrono-date"><span class="gras">' + F.date(r.date) + '</span><span class="muted" style="font-size:var(--text-sm)">' + r.heure + '</span></div><div class="chrono-corps" style="cursor:default"><span class="chrono-haut"><span class="gras">' + { whatsapp: 'WhatsApp', sms: 'SMS', appel: 'Appel' }[r.canal] + ' · ' + r.etape.replace('-', '−') + '</span>' + (r.promesseId ? '<span class="badge badge-due">' + UI.ic('clock') + 'Promesse</span>' : '') + '</span><span class="chrono-bas"><span>' + F.e(Data.utilisateur(r.auteurId).nom) + '</span>' + (r.prochaineAction ? '<span>' + UI.ic('calendar', 'ic-sm') + ' ' + F.e(r.prochaineAction.libelle) + ' le ' + F.date(r.prochaineAction.date) + '</span>' : '') + '</span>' + (r.texte ? '<span class="secondaire" style="font-size:var(--text-sm);white-space:pre-line">' + F.e(r.texte.split('\n').slice(2, 4).join(' ')) + '</span>' : '<span class="secondaire" style="font-size:var(--text-sm)">' + F.e(r.note || '') + '</span>') + '</div></li>'; }).join('') + '</ol>' : UI.etatVide({ ic: 'message', titre: 'Aucune relance', texte: 'Aucune relance enregistrée pour ce locataire.' })) + '</section>';
      else if (on === 'documents') corps = '<section class="carte">' + (docs.length ? '<div class="doc-grille">' + docs.map(window.Travaux.vignette).join('') + '</div>' : UI.etatVide({ ic: 'file-text', titre: 'Aucun document', texte: 'Joignez le bail signé pour retrouver le dossier en cas de litige.', action: UI.btn('Joindre un document', { v: 'primaire', action: 'demo' }) })) + '</section>';
      else corps = '<div class="lot-grille"><div class="lot-col"><section class="carte" aria-labelledby="t-coord"><div class="carte-entete"><h2 id="t-coord" class="carte-titre">' + UI.ic('user') + 'Coordonnées</h2><span class="badge badge-neutre">' + (t.type === 'activite' ? 'Activité commerciale' : 'Personne') + '</span></div><dl class="def-liste" style="margin:0"><div><dt>Téléphone</dt><dd>' + F.e(t.telephone) + '</dd></div><div><dt>WhatsApp</dt><dd>' + (t.whatsapp ? 'Oui' : 'Non : <button type="button" class="lien-bouton" data-action="demo">Ajouter le numéro</button>') + '</dd></div>' + (t.activite ? '<div><dt>Activité</dt><dd>' + F.e(t.activite) + '</dd></div>' : '') + '<div><dt>Contact de secours</dt><dd class="muted">Non renseigné</dd></div><div><dt>Pièce d’identité</dt><dd id="piece">' + (t.pieceIdentite.presente ? 'Fournie · masquée <button type="button" class="btn btn-tertiaire btn-petit" data-action="loc-piece">' + UI.ic('eye-off', 'ic-sm') + ' Afficher</button>' : '<span class="muted">Non fournie</span>') + '</dd></div></dl></section>' +
        '<section class="carte" aria-labelledby="t-esp"><div class="carte-entete"><h2 id="t-esp" class="carte-titre">' + UI.ic('lock') + 'Espace locataire</h2>' + UI.repere({ type: 'avenir', mini: true }) + '</div><p class="secondaire" style="margin-bottom:12px">Le locataire consulterait ses quittances et ses échéances en ligne. Pour l’instant, il reçoit ses quittances par WhatsApp.</p>' + UI.btn('Inviter le locataire', { v: 'secondaire', petit: true, disabled: true }) + '</section></div>' +
        '<div class="lot-col"><section class="carte" aria-labelledby="t-sol"><div class="carte-entete"><h2 id="t-sol" class="carte-titre">' + UI.ic('wallet') + 'Solde</h2>' + statutHtml(s) + '</div><div class="enc-solde-montant" style="margin-bottom:12px" aria-label="' + F.e(F.montantSr(s.solde)) + '">' + F.montant(s.solde) + '</div>' + (s.echeances.length ? '<ul class="liste">' + s.echeances.map(function (e) { return '<li><div class="liste-gauche"><span class="liste-titre">' + F.majuscule(F.mois(e.periode)) + '</span><span class="liste-sous">Échéance du ' + F.date(e.dateEcheance) + '</span></div><div class="liste-droite"><span class="liste-montant">' + F.montant(e.reste) + '</span>' + (e.statut === 'en_retard' ? UI.statut('en_retard', F.jours(e.ancienneteJours)) : e.statut === 'due' ? UI.statut('due', 'aujourd’hui') : UI.statut(e.statut)) + '</div></li>'; }).join('') + '</ul>' : '<p class="secondaire">Aucun montant dû pour ' + F.mois(d.meta.moisReference) + '.</p>') + '</section>' +
        '<section class="carte" aria-labelledby="t-bx"><div class="carte-entete"><h2 id="t-bx" class="carte-titre">' + UI.ic('file-text') + 'Baux</h2></div><ul class="liste">' + baux.map(function (b) { var lot = Data.lot(b.lotId); return '<li><div class="liste-gauche"><span class="liste-titre"><a href="#/bail/' + b.id + '">' + F.e(lot.nom) + '</a></span><span class="liste-sous">Loyer ' + F.montant(b.loyer) + ' · le ' + b.jourEcheance + ' du mois</span></div>' + UI.neutre(b.statut === 'preavis' ? 'En préavis' : 'Actif', b.statut === 'preavis' ? 'clock' : 'check') + '</li>'; }).join('') + '</ul></section></div></div>';
      return onglets + corps;
    }
  });
})();
