/*
 * S25 : Quittance (aperçu, détail, impression) et S26 : page de partage publique.
 * La quittance est affichée en HTML selon la mise en page imprimable de UI_UX_DESIGN.md (A4 ou A5, une seule page).
 * Module partagé : window.Doc.quittance(id, format) retourne la feuille ; elle sert aussi à la page publique.
 */
(function () {
  var F = window.Fmt, UI = window.UI, Data = window.Data, App = window.App, Mut = window.Mut;
  var FONCTION = { U1: 'gérante', U2: 'gestionnaire', U3: 'caissière', S1: 'propriétaire', S2: 'collaborateur' };
  function D() { return Data.org(); }
  function lienDe(num) { return 'https://immo-suite.demo/q/' + num; }

  function ligne(lib, val, cls) { return '<div class="q-ligne' + (cls ? ' ' + cls : '') + '"><span>' + lib + '</span><span class="q-val">' + val + '</span></div>'; }

  // Feuille de quittance (HTML). format : 'a5' ou 'a4'
  function feuille(q, format) {
    var org = Data.org().organisation, p = Data.paiement(q.paiementId), bail = Data.bail(q.bailId), lot = Data.lot(bail.lotId), bien = Data.bien(lot.bienId);
    var u = Data.utilisateur(p.recuPar), noms = Data.nomLocataires(bail.locataireIds);
    var payePar = p.payePar && p.payePar.type === 'tiers' ? '<div class="q-ligne"><span>Payé par</span><span class="q-val">' + F.e(p.payePar.nom) + '</span></div>' : '';
    var situation;
    if (q.avance) situation = 'Avance de ' + F.montant(q.avance.montant) + ' déduite de l’échéance du ' + F.date(q.avance.echeanceDate) + '.';
    else if (q.creditBail) situation = 'Crédit du bail : ' + F.montant(q.creditBail) + ', affecté à la prochaine échéance.';
    else situation = 'Solde restant dû après ce paiement : <strong>' + F.montant(q.soldeRestant) + '</strong>' + (q.soldeRestant === 0 ? ' — échéance soldée' : '');
    var ban = '';
    if (q.rectificative) { var orig = Data.quittance(q.remplaceQuittanceId); ban = '<div class="q-bandeau">Quittance rectificative — annule et remplace la quittance n° ' + F.e(orig.numero) + '</div>'; }
    var filigrane = '';
    if (q.statut === 'annulee') { var rem = q.remplaceeParQuittanceId ? Data.quittance(q.remplaceeParQuittanceId) : null; filigrane = '<div class="q-filigrane" aria-hidden="true">ANNULÉE' + (rem ? ' — remplacée par la quittance n° ' + F.e(rem.numero) : '') + '</div>'; }
    var charges = q.ventilation[1] ? q.ventilation[1].montant : 0;
    return '<article class="q-feuille q-' + format + '" aria-label="Quittance de loyer n° ' + F.e(q.numero) + '">' + filigrane +
      '<header class="q-entete"><div class="q-org">' + F.e(org.nom.toUpperCase()) + '</div><div class="q-sous">' + F.e(org.adresse) + ' — Tél. ' + F.e(org.telephone) + '</div></header>' +
      '<div class="q-titre-bloc"><h2 class="q-titre">QUITTANCE DE LOYER</h2><div class="q-num"><strong>N° ' + F.e(q.numero) + '</strong><span>Émise le ' + F.date(q.date) + '</span></div></div>' + ban +
      '<div class="q-bloc">' + ligne('Reçu de', F.e(noms) + (Data.locataire(bail.locataireIds[0]).telephone ? ' — ' + F.e(Data.locataire(bail.locataireIds[0]).telephone) : '')) + payePar +
      ligne('Pour le lot', F.e(lot.nom) + ' — ' + F.e(bien.quartier) + ' (' + { habitation: 'habitation', commercial: 'commercial', mixte: 'mixte' }[lot.usage] + ')') +
      ligne('Période', 'loyer de ' + F.e(q.periode) + ' (échéance du ' + F.date(q.echeanceDate) + ')') +
      ligne('Paiement', F.date(p.date) + ' — ' + Data.modeLibelle(p.mode) + (p.reference ? ' — réf. ' + F.e(p.reference) : '')) + '</div>' +
      '<div class="q-bloc q-montants">' + ligne('Loyer', F.montant(q.ventilation[0].montant)) + ligne('Charges', F.montant(charges)) + ligne('TOTAL REÇU', F.montant(q.montant), 'q-total') + '</div>' +
      '<p class="q-lettres">En lettres : <strong>' + F.e(F.majuscule(q.montantLettres)) + '</strong></p><p class="q-situation">' + situation + '</p>' +
      '<div class="q-signature"><span>Reçu par : ' + F.e(u.nom) + ', ' + FONCTION[u.id] + '</span><span class="q-cachet">cachet</span></div>' +
      '<footer class="q-pied">' + (bail.tvaIncluse ? '<div>TVA incluse</div>' : '') + '<div>Mentions légales complémentaires : à valider</div><div>Document généré par Immo Suite</div></footer></article>';
  }
  window.Doc = { quittance: function (id, format) { return feuille(Data.quittance(id), format || 'a5'); }, ligne: ligne, texte: function (id) { return texteQuittance(Data.quittance(id)); } };

  function texteQuittance(q) {
    var bail = Data.bail(q.bailId), t = Data.locataire(bail.locataireIds[0]), org = D().organisation.nom;
    return 'Bonjour ' + t.prenom + ', voici votre quittance n° ' + q.numero + ' pour ' + F.montant(q.montant) + ' (loyer de ' + q.periode + ') : ' + lienDe(q.numero) + '\n' + org;
  }

  var format = 'a5';

  function panneauDroite(q) {
    var p = Data.paiement(q.paiementId), bail = Data.bail(q.bailId), t = Data.locataire(bail.locataireIds[0]);
    var envois = q.envois.length ? '<ul class="liste">' + q.envois.map(function (e) { var u = Data.utilisateur(e.par); return '<li><div class="liste-gauche"><span class="liste-titre">' + (e.canal === 'whatsapp' ? 'WhatsApp' : 'SMS') + '</span><span class="liste-sous">' + F.date(e.date) + ' · ' + F.e(u.nom) + '</span></div>' + UI.ic('check-circle', 'ic-lg') + '</li>'; }).join('') + '</ul>' : '<p class="secondaire" style="font-size:var(--text-sm)">Cette quittance n’a pas encore été envoyée.</p>';
    var liens = '<li><a href="#/paiements?ouvrir=' + p.id + '">' + UI.ic('receipt', 'ic-sm') + ' Voir le paiement dans le journal</a></li>';
    if (q.remplaceQuittanceId) liens += '<li><a href="#/quittance/' + q.remplaceQuittanceId + '">' + UI.ic('link', 'ic-sm') + ' Quittance remplacée : ' + F.e(Data.quittance(q.remplaceQuittanceId).numero) + '</a></li>';
    if (q.remplaceeParQuittanceId) liens += '<li><a href="#/quittance/' + q.remplaceeParQuittanceId + '">' + UI.ic('link', 'ic-sm') + ' Remplacée par : ' + F.e(Data.quittance(q.remplaceeParQuittanceId).numero) + '</a></li>';
    liens += '<li><a href="#/partage/' + q.id + '">' + UI.ic('eye-off', 'ic-sm') + ' Voir la page de partage (ce que voit le locataire)</a></li>';
    return '<section class="carte pas-imprimer" aria-labelledby="t-env"><h2 id="t-env" class="carte-titre" style="margin-bottom:12px">' + UI.ic('message') + 'Envois</h2>' + envois + '</section>' +
      '<section class="carte pas-imprimer" aria-labelledby="t-liens"><h2 id="t-liens" class="carte-titre" style="margin-bottom:12px">' + UI.ic('link') + 'Liens</h2><ul class="liste-liens">' + liens + '</ul></section>';
  }

  App.enregistrer('quittance', {
    meta: function (ctx) {
      var q = Data.quittance(ctx.params[0]);
      if (!q) return { titre: 'Introuvable', repere: { type: 'v1' } };
      return { titre: 'Quittance n° ' + F.e(q.numero), sous: F.e(Data.nomLocataires(Data.bail(q.bailId).locataireIds)) + ' · ' + F.e(Data.lot(Data.bail(q.bailId).lotId).nom), repere: { type: 'v1' } };
    },
    rendre: function (ctx) {
      var q = Data.quittance(ctx.params[0]);
      if (!q) return '<div class="carte reserve">' + UI.etatVide({ ic: 'search', titre: 'Introuvable', texte: 'Cette page n’existe pas ou vous n’y avez pas accès.', action: UI.btn('Retour à l’accueil', { v: 'primaire', href: '#/accueil' }) }) + '</div>';
      var apres = ctx.query.apres === '1', statutQ = q.statut === 'annulee' ? UI.neutre('Annulée', 'ban', 'badge-void') : UI.neutre('Émise', 'check');
      var bandeau = apres ? '<div class="bandeau bandeau-succes pas-imprimer" style="margin-bottom:16px">' + UI.ic('check-circle') + '<span>Paiement de ' + F.montant(q.montant) + ' enregistré. Quittance n° ' + F.e(q.numero) + ' prête.</span></div>' : '';
      var actions = '<div class="q-actions pas-imprimer">' + UI.btn('Envoyer sur WhatsApp', { v: 'primaire', ic: 'message', action: 'q-envoyer', data: ' data-id="' + q.id + '"' }) +
        '<div class="q-format" role="group" aria-label="Format d’impression"><button type="button" class="puce q-fmt" data-action="q-format" data-f="a5" aria-pressed="true"><strong>A5</strong><span>148 × 210 mm</span></button><button type="button" class="puce q-fmt" data-action="q-format" data-f="a4" aria-pressed="false"><strong>A4</strong><span>210 × 297 mm</span></button></div>' +
        UI.btn('Imprimer', { v: 'secondaire', ic: 'printer', action: 'q-imprimer' }) + UI.btn('Copier le lien', { v: 'secondaire', ic: 'copy', action: 'q-copier', data: ' data-id="' + q.id + '"' }) + '</div>';
      var suite = '<div class="btn-groupe pas-imprimer" style="margin-top:16px">' + UI.btn('Encaisser un autre loyer', { v: 'secondaire', href: '#/encaisser', ic: 'banknote' }) + UI.btn('Émettre une quittance rectificative', { v: 'tertiaire', action: 'demo' }) + '</div>';
      format = 'a5';
      setTimeout(function () { poserPage('a5'); window.addEventListener('hashchange', function retirer() { var s = document.getElementById('style-page'); if (s) s.parentNode.removeChild(s); window.removeEventListener('hashchange', retirer); }); }, 0);
      return bandeau + '<div class="q-page"><div class="q-apercu" id="q-apercu">' + feuille(q, 'a5') + '</div><div class="q-cote">' + '<div class="pas-imprimer" style="display:flex;gap:8px;align-items:center;flex-wrap:wrap">' + statutQ + '<span class="muted" style="font-size:var(--text-sm)">Montant en chiffres et en lettres, numéro sans trou.</span></div>' + actions + '<div id="q-droite">' + panneauDroite(q) + '</div>' + suite + '</div></div>';
    }
  });

  // Format de page d'impression : A4 (marges 15 mm) ou A5 (marges 10 mm). La règle @page est retirée en quittant l'écran.
  function poserPage(fmt) {
    var s = document.getElementById('style-page'); if (!s) { s = document.createElement('style'); s.id = 'style-page'; document.head.appendChild(s); }
    s.textContent = fmt === 'a4' ? '@page { size: A4 portrait; margin: 15mm; }' : '@page { size: A5 portrait; margin: 10mm; }';
  }
  App.action('q-format', function (el) {
    format = el.getAttribute('data-f'); poserPage(format);
    document.querySelectorAll('.q-fmt').forEach(function (b) { b.setAttribute('aria-pressed', String(b === el)); });
    var f = document.querySelector('.q-feuille'); f.classList.remove('q-a4', 'q-a5'); f.classList.add('q-' + format);
  });
  App.action('q-imprimer', function () { try { window.print(); } catch (e) { UI.toast('L’impression n’est pas disponible ici.', { type: 'avert' }); } });
  App.action('q-copier', function (el) {
    var q = Data.quittance(el.getAttribute('data-id')), lien = lienDe(q.numero);
    try { if (navigator.clipboard && navigator.clipboard.writeText) navigator.clipboard.writeText(lien); } catch (e) { /* copie facultative */ }
    UI.toast('Lien copié : ' + F.e(lien), { type: 'succes' });
  });
  App.action('q-envoyer', function (el) {
    var q = Data.quittance(el.getAttribute('data-id')), bail = Data.bail(q.bailId), t = Data.locataire(bail.locataireIds[0]);
    UI.messagerie({ canal: 'whatsapp', nom: t.nom, telephone: t.telephone, texte: texteQuittance(q),
      surEnvoi: function () { Mut.appliquer('quittance.envoyer', { id: q.id, canal: 'whatsapp', par: window.App.ctx.utilisateur.id }); document.getElementById('q-droite').innerHTML = panneauDroite(q); } });
  });

  /* ---------- S26 : page de partage publique ---------- */
  App.enregistrer('partage', {
    meta: function () { return { titre: 'Document partagé', repere: { type: 'v1' } }; },
    rendre: function (ctx) {
      var id = ctx.params[0], q = Data.quittance(id), r = Data.releve(id), org = Data.org().organisation;
      var entete = function (t) { return '<p class="muted" style="margin-bottom:12px">Document partagé par ' + F.e(org.nom) + '. ' + t + '</p>'; };
      if (ctx.query.etat === 'expire') return '<div class="carte reserve">' + UI.etatVide({ ic: 'clock', titre: 'Ce lien a expiré', texte: 'Demandez un nouveau lien à ' + F.e(org.nom) + '.' }) + '</div>';
      if (!q && !r) return '<div class="carte reserve">' + UI.etatVide({ ic: 'search', titre: 'Introuvable', texte: 'Cette page n’existe pas ou le lien est invalide.' }) + '</div>';
      var actions = '<div class="btn-groupe" style="margin-bottom:16px">' + UI.btn('Télécharger', { v: 'primaire', ic: 'download', action: 'demo' }) + UI.btn('Imprimer', { v: 'secondaire', ic: 'printer', action: 'q-imprimer' }) + '</div>';
      var simul = '<div class="bandeau bandeau-info pas-imprimer" style="margin-bottom:16px">' + UI.ic('info') + '<span>Aperçu : ce que verrait le destinataire du lien, sans compte, sans menu. ' + UI.btn('Retour à Immo Suite', { v: 'tertiaire', petit: true, action: 'partage-retour' }) + (ctx.query.etat ? '' : ' <button type="button" class="lien-bouton" data-action="partage-expire" data-id="' + F.e(id) + '">Voir un lien expiré</button>') + '</span></div>';
      if (q) return simul + actions + '<div class="q-apercu">' + feuille(q, 'a5') + '</div>';
      return simul + actions + '<div class="q-apercu">' + window.Doc.releve(r) + '</div>';
    }
  });
  App.action('partage-retour', function () { if (history.length > 1) history.back(); else App.aller('/accueil'); });
  App.action('partage-expire', function (el) { App.aller('/partage/' + el.getAttribute('data-id') + '?etat=expire'); });
})();
