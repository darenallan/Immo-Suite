/*
 * S04 : Encaisser (parcours 1).
 * Recherche unique → carte du locataire → montant, mode, date, payé par → aperçu d'affectation → « Encaisser et envoyer la quittance »
 * → S25 (quittance). Scénario complet préparé pour Issouf Tapsoba (75 000, 50 000 ou 100 000 FCFA).
 * Garde-fous montrés : référence en doublon (R-041), connexion perdue (R-042), confirmation au-delà du seuil (R-038), montant vide (R-037).
 * Aucun calcul métier : les affectations viennent des variantes précalculées (data/agence.js, scenarios.encaisser).
 */
(function () {
  var F = window.Fmt, UI = window.UI, Data = window.Data, App = window.App, Etat = window.Etat;
  var racine = null, st = null;

  function neuf(q) {
    return { q: q || '', locataireId: null, montant: null, mode: 'especes', reference: '', payePar: 'locataire', tiers: '', note: '', enCours: false, erreur: null, cleIdempotence: 'idem-' + Math.random().toString(36).slice(2, 8) };
  }
  function D() { return Data.org(); }
  function SC() { return D().scenarios.encaisser || {}; }

  /* ---------- Situation d'un locataire (lecture des échéances du jeu de données) ---------- */
  function situation(tid) {
    var d = D(), baux = d.baux.filter(function (b) { return b.locataireIds.indexOf(tid) >= 0; });
    var moisRef = d.meta.moisReference, choisi = null;
    baux.forEach(function (b) {
      var es = d.echeances.filter(function (e) { return e.bailId === b.id && e.periode <= moisRef && e.reste > 0; });
      if (es.length && !choisi) choisi = { bail: b, echeances: es };
    });
    if (!choisi) return { bail: baux[0], echeances: [], solde: 0 };
    var es = choisi.echeances.sort(function (a, b) { return a.dateEcheance.localeCompare(b.dateEcheance); });
    var retard = es.filter(function (e) { return e.statut === 'en_retard'; });
    return { bail: choisi.bail, echeances: es, solde: es.reduce(function (s, e) { return s + e.reste; }, 0), cible: es[0], retard: retard };
  }
  function modeComplet(tid) { var sc = SC(); return tid === sc.locataireId && !sc.fait; }

  /* ---------- Recherche unique ---------- */
  function chercher(q) {
    var d = D(), n = F.norm(q), chiffres = q.replace(/\D/g, ''), res = { locataires: [], lots: [], quittances: [] };
    if (n.length < 2) return res;
    res.locataires = d.locataires.filter(function (t) { return F.norm(t.nom).indexOf(n) >= 0 || (chiffres.length >= 2 && t.telephone.replace(/\D/g, '').slice(3).indexOf(chiffres.replace(/^226/, '')) >= 0); }).slice(0, 8);
    res.lots = d.lots.filter(function (l) { return F.norm(l.nom).indexOf(n) >= 0 && l.bailId; }).slice(0, 8);
    res.quittances = d.quittances.filter(function (x) { return F.norm(x.numero).indexOf(n) >= 0; }).slice(0, 8);
    return res;
  }

  function htmlResultats() {
    var q = st.q.trim();
    if (!q) return recents();
    if (st.locataireId) return '';
    var r = chercher(q), total = r.locataires.length + r.lots.length + r.quittances.length;
    if (q.length < 2) return '<p class="champ-aide" style="margin-top:8px">Saisissez au moins 2 caractères.</p>';
    if (!total) return '<div class="etat-vide" style="padding:16px 8px">' + UI.ic('search') + '<p class="gras">Aucun résultat pour « ' + F.e(q) + ' »</p><p class="secondaire">Vérifiez l’orthographe ou cherchez par numéro de téléphone.</p>' + UI.btn('Créer un locataire', { v: 'secondaire', action: 'demo' }) + '</div>';
    var h = '<div class="resultats" role="listbox" aria-label="Résultats de la recherche">';
    if (r.locataires.length) h += '<div class="resultats-titre">Locataires</div>' + r.locataires.map(function (t) { return ligneLoc(t, 'loc'); }).join('');
    if (r.lots.length) h += '<div class="resultats-titre">Lots</div>' + r.lots.map(function (l) { var b = Data.bail(l.bailId); return ligneLoc(Data.locataire(b.locataireIds[0]), 'loc', l.nom); }).join('');
    if (r.quittances.length) h += '<div class="resultats-titre">Quittances</div>' + r.quittances.map(function (x) { return '<a class="resultat" href="#/quittance/' + x.id + '"><span class="gras">' + F.e(x.numero) + '</span><span class="secondaire">' + F.mois(x.echeanceDate.slice(0, 7)) + ' · ' + F.montant(x.montant) + '</span></a>'; }).join('');
    return h + '</div>';
  }
  function ligneLoc(t, type, lotNom) {
    var s = situation(t.id), lot = s.bail ? Data.lot(s.bail.lotId) : null;
    var badge = s.solde > 0 ? (s.retard && s.retard.length ? UI.statut('en_retard', F.jours(s.retard[0].ancienneteJours)) : UI.statut(s.cible.statut)) : UI.statut('payee', 'à jour');
    return '<button type="button" class="resultat" role="option" data-action="enc-choisir" data-id="' + t.id + '"><span class="resultat-corps"><span class="gras">' + F.e(t.nom) + '</span><span class="secondaire">' + F.e(lotNom || (lot ? lot.nom : '')) + ' · ' + F.e(t.telephone) + '</span></span><span class="resultat-droite">' + (s.solde > 0 ? '<span class="gras num">' + F.montant(s.solde) + '</span>' : '') + badge + '</span></button>';
  }
  function recents() {
    var vus = {}, liste = [];
    Data.paiementsRecents(12).forEach(function (p) { var t = p.locataireIds[0]; if (!vus[t] && liste.length < 4) { vus[t] = 1; liste.push(t); } });
    var sc = SC();
    var h = !sc.locataireId ? '' : '<div class="champ-aide" style="margin-top:12px">Pour la démonstration : ' + UI.statut('due') + ' <button type="button" class="lien-bouton" data-action="enc-choisir" data-id="' + sc.locataireId + '">Issouf Tapsoba, Villa B2</button></div>';
    h += '<div class="champ-aide" style="margin-top:12px;margin-bottom:6px">Locataires récents</div><div class="enc-recents">' + liste.map(function (id) {
      var t = Data.locataire(id), s = situation(id);
      return '<button type="button" class="puce" data-action="enc-choisir" data-id="' + id + '"><strong>' + F.e(t.nom) + '</strong><span>' + F.e(s.bail ? Data.lot(s.bail.lotId).nom : '') + '</span></button>';
    }).join('') + '</div>';
    return h;
  }

  /* ---------- Carte du locataire ---------- */
  function htmlCarte() {
    if (!st.locataireId) return '';
    var t = Data.locataire(st.locataireId), s = situation(t.id), lot = Data.lot(s.bail.lotId), bien = Data.bien(lot.bienId);
    var statutHtml, ligneEch;
    if (s.solde > 0) {
      var plusAncien = s.cible;
      if (s.retard.length) { statutHtml = UI.statut('en_retard', F.jours(s.retard[0].ancienneteJours)); }
      else if (plusAncien.statut === 'due') statutHtml = UI.statut('due', 'aujourd’hui');
      else statutHtml = UI.statut(plusAncien.statut);
      ligneEch = 'Reste dû · ' + (s.echeances.length > 1 ? s.echeances.length + ' échéances' : F.mois(plusAncien.periode) + ' · échéance du ' + F.date(plusAncien.dateEcheance));
    } else { statutHtml = UI.statut('payee', 'à jour'); ligneEch = 'Aucun montant dû pour ' + F.mois(D().meta.moisReference); }
    var fait = SC().fait;
    return '<section class="carte" aria-labelledby="t-loc"><h2 class="sr-only" id="t-loc">Locataire choisi</h2><div class="enc-locataire"><span class="avatar avatar-lg" aria-hidden="true">' + F.initiales(t.nom) + '</span>' +
      '<div class="enc-loc-corps"><span class="enc-loc-nom">' + F.e(t.nom) + (t.activite ? ' <span class="muted" style="font-weight:400;font-size:var(--text-base)">· ' + F.e(t.activite) + '</span>' : '') + '</span>' +
      '<div class="enc-loc-meta"><span>' + UI.ic('phone', 'ic-sm') + ' ' + F.e(t.telephone) + '</span><span>' + UI.ic('building', 'ic-sm') + ' ' + F.e(lot.nom) + ', ' + F.e(bien.quartier) + '</span>' + UI.usage(lot.usage) + '</div></div>' +
      '<button type="button" class="btn btn-tertiaire btn-petit" data-action="enc-changer">Changer</button></div>' +
      '<div class="enc-solde"><div><div class="chiffre-sous">' + ligneEch + '</div><div class="enc-solde-montant" aria-label="' + F.e(F.montantSr(s.solde)) + '">' + F.montant(s.solde) + '</div></div>' + statutHtml + '</div>' +
      (fait && t.id === fait.locataireId ? '' : '') +
      '<div class="btn-groupe" style="margin-top:12px">' + UI.btn('Appeler', { v: 'secondaire', petit: true, ic: 'phone', action: 'demo' }) + UI.btn('WhatsApp', { v: 'secondaire', petit: true, ic: 'message', action: 'demo' }) +
      (fait && t.id === SC().locataireId ? UI.btn('Voir la quittance', { v: 'tertiaire', petit: true, href: '#/quittance/' + fait.quittanceId }) : '') + '</div></section>';
  }

  /* ---------- Formulaire ---------- */
  function htmlForm() {
    if (!st.locataireId) return '';
    var t = Data.locataire(st.locataireId), s = situation(t.id), sc = SC(), complet = modeComplet(t.id);
    if (s.solde === 0) return '<div class="bandeau bandeau-succes">' + UI.ic('check-circle') + '<span>Ce locataire est à jour : il n’y a rien à encaisser pour ' + F.mois(D().meta.moisReference) + '.</span></div>';
    var prepares = complet ? sc.montantsPrepares : [{ montant: s.solde, libelle: 'Solde dû' }];
    if (st.montant === null) st.montant = prepares[0].montant;
    var puces = prepares.map(function (m) { return '<button type="button" class="puce" aria-pressed="' + (st.montant === m.montant) + '" data-action="enc-montant" data-m="' + m.montant + '"><strong>' + F.nombre(m.montant) + ' FCFA</strong><span>' + m.libelle + '</span></button>'; }).join('');
    var modes = [['especes', 'Espèces'], ['mobile_money', 'Mobile money'], ['virement', 'Virement'], ['cheque', 'Chèque']];
    var seg = modes.map(function (m) { return '<input type="radio" name="mode" id="mode-' + m[0] + '" value="' + m[0] + '"' + (st.mode === m[0] ? ' checked' : '') + '><label for="mode-' + m[0] + '">' + UI.ic('check', 'ic-sm coche') + m[1] + '</label>'; }).join('');
    var bandeauDemo = complet ? '' : '<div class="bandeau bandeau-info" style="margin-bottom:16px">' + UI.ic('info') + '<span>Démonstration : l’encaissement complet est préparé pour Issouf Tapsoba. Pour les autres locataires, la fiche et le solde sont ceux du jeu de données.</span></div>';
    return bandeauDemo + '<div id="enc-erreur" aria-live="polite">' + htmlErreur() + '</div>' +
      '<section class="carte" aria-labelledby="t-form"><h2 id="t-form" class="carte-titre" style="margin-bottom:16px">' + UI.ic('banknote') + 'Paiement</h2>' +
      '<div class="champ"><label class="champ-etiquette" for="montant">Montant reçu (obligatoire)</label><div class="saisie-montant"><input id="montant" inputmode="numeric" autocomplete="off" value="' + (st.montant ? F.nombre(st.montant) : '') + '" aria-describedby="montant-aide montant-err"><span class="suffixe">FCFA</span></div>' +
      '<div class="champ-erreur cache" id="montant-err" role="alert"></div><div class="champ-aide" id="montant-aide">Montants proposés pour la démonstration :</div><div class="puces" id="puces">' + puces + '</div></div>' +
      '<fieldset class="champ" style="border:0;padding:0;margin:16px 0 0"><legend class="champ-etiquette" style="padding:0;margin-bottom:6px">Mode de paiement (obligatoire)</legend><div class="segmente" id="modes">' + seg + '</div></fieldset>' +
      '<div class="champ' + (st.mode === 'especes' ? ' cache' : '') + '" id="champ-ref"><label class="champ-etiquette" for="reference">Référence du paiement (obligatoire)</label><input class="saisie" id="reference" value="' + F.e(st.reference) + '" placeholder="Exemple : MM-261113-4821" autocomplete="off" aria-describedby="ref-err"><div class="champ-erreur cache" id="ref-err" role="alert"></div></div>' +
      '<div class="champ"><label class="champ-etiquette" for="date">Date du paiement</label><input class="saisie" id="date" value="' + F.date(D().meta.aujourdhui) + '" readonly aria-describedby="date-aide"><div class="champ-aide" id="date-aide">Aujourd’hui. Une date future est refusée.</div></div>' +
      '<div class="champ"><label class="champ-etiquette" for="paye-par">Payé par</label><select class="saisie" id="paye-par"><option value="locataire"' + (st.payePar === 'locataire' ? ' selected' : '') + '>' + F.e(t.nom) + ' (locataire du bail)</option><option value="tiers"' + (st.payePar === 'tiers' ? ' selected' : '') + '>Une autre personne (tiers)</option></select></div>' +
      '<div class="champ' + (st.payePar === 'tiers' ? '' : ' cache') + '" id="champ-tiers"><label class="champ-etiquette" for="tiers">Nom du tiers (obligatoire)</label><input class="saisie" id="tiers" value="' + F.e(st.tiers) + '" autocomplete="off"></div>' +
      '<div class="champ"><label class="champ-etiquette" for="note">Note (facultative)</label><input class="saisie" id="note" value="' + F.e(st.note) + '" placeholder="Exemple : paiement apporté par son frère" autocomplete="off"></div>' +
      '<details class="repliable" style="margin-top:16px"><summary>' + UI.ic('message') + 'Coller le SMS mobile money ' + UI.repere({ type: 'v1', pilote: true, mini: true }) + UI.ic('chevron-down') + '</summary><div class="repliable-corps"><p class="secondaire" style="font-size:var(--text-sm);margin-bottom:8px">Collez le SMS reçu : le montant et la référence sont lus pour vous.</p><label class="sr-only" for="sms">SMS à coller</label><textarea class="saisie" id="sms" placeholder="Collez ici le texte du SMS"></textarea>' + UI.btn('Lire le SMS', { v: 'secondaire', petit: true, action: 'demo', cls: '' }) + '</div></details></section>';
  }

  function htmlErreur() {
    if (!st.erreur) return '';
    if (st.erreur.type === 'doublon') return '<div class="bandeau bandeau-erreur" style="margin-bottom:16px">' + UI.ic('alert-circle') + '<div><strong>Ce paiement semble déjà enregistré (quittance n° ' + F.e(st.erreur.numero) + ').</strong><br>Vérifiez la référence avant de continuer.<div style="margin-top:8px">' + UI.btn('Voir le paiement', { v: 'secondaire', petit: true, href: '#/quittance/' + st.erreur.quittanceId }) + '</div></div></div>';
    return '<div class="bandeau bandeau-erreur" style="margin-bottom:16px">' + UI.ic('alert-circle') + '<div><strong>Connexion perdue.</strong><br>Vos informations sont conservées. Réessayez dès que le réseau revient. Si le paiement a déjà été enregistré, il ne sera pas créé deux fois.' +
      '<div class="champ-aide" style="margin-top:6px;color:inherit">Code d’incident : ' + F.e(st.cleIdempotence) + '</div><div style="margin-top:8px">' + UI.btn('Réessayer', { v: 'primaire', petit: true, action: 'enc-valider', id: 'btn-reessayer' }) + '</div></div></div>';
  }

  /* ---------- Colonne de droite : aperçu et validation ---------- */
  function htmlDroite() {
    if (!st.locataireId) return '<section class="carte"><div class="etat-vide" style="padding:24px 8px">' + UI.ic('banknote') + '<h2>Choisissez un locataire</h2><p class="secondaire">Recherchez par nom, téléphone, lot ou numéro de quittance. L’aperçu de l’affectation et la quittance apparaissent ici.</p></div></section>';
    var t = Data.locataire(st.locataireId), s = situation(t.id), sc = SC(), complet = modeComplet(t.id);
    if (s.solde === 0) return '<section class="carte"><p class="secondaire">Rien à encaisser pour ce locataire.</p></section>';
    var v = complet ? sc.variantes[st.montant] : null;
    var apercu;
    if (v) {
      apercu = '<ul class="liste">' + v.affectations.map(function (a) {
        return '<li><div class="liste-gauche"><span class="liste-titre">' + a.libelle + '</span><span class="liste-sous">Échéance du ' + F.date(a.echeanceDate) + ' · dû ' + F.montant(a.du) + '</span></div><div class="liste-droite"><span class="liste-montant" aria-label="' + F.e(F.montantSr(a.affecte)) + ' affectés">' + F.montant(a.affecte) + '</span>' + UI.statut(a.statut) + (a.statut === 'partielle' && a.reste ? '<span class="liste-sous">reste ' + F.montant(a.reste) + '</span>' : '') + '</div></li>';
      }).join('') + '</ul>';
    } else apercu = '<p class="secondaire" style="font-size:var(--text-sm)">' + (complet ? 'Choisissez un montant proposé pour voir l’affectation.' : 'L’aperçu d’affectation est préparé pour le scénario d’Issouf Tapsoba.') + '</p>';
    var total = st.montant || 0;
    return '<section class="carte" aria-labelledby="t-aff"><h2 id="t-aff" class="carte-titre" style="margin-bottom:4px">' + UI.ic('chart') + 'Aperçu de l’affectation</h2><p class="secondaire" style="font-size:var(--text-sm);margin-bottom:12px">Les échéances les plus anciennes sont réglées en premier.</p>' + apercu + '</section>' +
      '<section class="carte enc-recap" aria-labelledby="t-recap"><h2 id="t-recap" class="sr-only">Récapitulatif</h2><div class="enc-recap-ligne"><span class="secondaire">Total à encaisser</span><span class="enc-recap-total" aria-label="' + F.e(F.montantSr(total)) + '">' + F.montant(total) + '</span></div>' +
      '<div class="enc-resultat">' + UI.ic('receipt', 'ic-lg') + '<div><div class="gras">Quittance n° ' + F.e(D().meta.prochainNumero) + '</div><div class="secondaire" style="font-size:var(--text-sm)">Aperçu A5' + (v ? ' · ' + F.e(v.libelleResultat) : '') + '</div></div></div>' +
      '<div class="enc-action enc-action-bureau">' + boutonValider() + '<span class="enc-action-aide">Un clic : paiement enregistré, quittance numérotée.</span></div></section>';
  }
  function boutonValider() {
    return UI.btn(st.enCours ? 'Enregistrement…' : 'Encaisser et envoyer la quittance', { v: 'primaire', grand: true, ic: st.enCours ? 'clock' : 'check', action: 'enc-valider', cls: 'bouton-valider' + (st.enCours ? ' charge' : ''), disabled: st.enCours });
  }

  /* ---------- Rendu et mises à jour ---------- */
  function htmlRecherche() {
    return '<section class="carte" aria-labelledby="t-rech"><h2 id="t-rech" class="sr-only">Recherche</h2><div class="champ"><label class="champ-etiquette" for="rech">Locataire, lot, téléphone ou n° de quittance</label>' +
      '<div style="position:relative"><span style="position:absolute;left:12px;top:50%;transform:translateY(-50%);color:var(--color-text-muted)">' + UI.ic('search') + '</span><input class="saisie" id="rech" type="search" value="' + F.e(st.q) + '" style="padding-left:40px" autocomplete="off" aria-controls="enc-resultats"></div></div>' +
      '<div id="enc-resultats" aria-live="polite">' + htmlResultats() + '</div></section>';
  }
  function maj(quoi) {
    var q = function (id) { return racine.querySelector(id); };
    if (!quoi || quoi === 'tout' || quoi === 'resultats') q('#enc-resultats').innerHTML = htmlResultats();
    if (quoi === 'tout') { q('#enc-carte').innerHTML = htmlCarte(); q('#enc-form').innerHTML = htmlForm(); bindForm(); }
    if (quoi === 'tout' || quoi === 'droite') { q('#enc-droite').innerHTML = htmlDroite(); var m = q('#enc-action-mobile'); if (m) m.innerHTML = st.locataireId && situation(st.locataireId).solde > 0 ? boutonValider() : ''; }
  }

  function lireChamps() {
    var g = function (id) { var e = racine.querySelector(id); return e ? e.value : ''; };
    var m = racine.querySelector('input[name="mode"]:checked');
    if (m) st.mode = m.value; st.reference = g('#reference'); st.payePar = g('#paye-par') || st.payePar; st.tiers = g('#tiers'); st.note = g('#note');
  }
  function erreurChamp(idChamp, idMsg, texte) {
    var c = racine.querySelector(idChamp), m = racine.querySelector(idMsg);
    if (!m) return; m.classList.toggle('cache', !texte); m.innerHTML = texte ? UI.ic('alert-circle', 'ic-sm') + '<span>' + texte + '</span>' : '';
    if (c) c.setAttribute('aria-invalid', texte ? 'true' : 'false');
  }

  function valider() {
    if (st.enCours) return;
    lireChamps();
    var t = Data.locataire(st.locataireId), sc = SC(), s = situation(t.id), seuil = D().parametres.seuilConfirmationPaiement;
    erreurChamp('#montant', '#montant-err', ''); erreurChamp('#reference', '#ref-err', '');
    if (!st.montant || st.montant <= 0) { erreurChamp('#montant', '#montant-err', 'Saisissez un montant supérieur à 0.'); racine.querySelector('#montant').focus(); return; }
    if (st.mode !== 'especes' && !st.reference.trim()) { erreurChamp('#reference', '#ref-err', 'Saisissez la référence du paiement.'); racine.querySelector('#reference').focus(); return; }
    if (st.payePar === 'tiers' && !st.tiers.trim()) { UI.toast('Saisissez le nom du tiers qui a payé.', { type: 'avert' }); racine.querySelector('#tiers').focus(); return; }
    if (st.montant > seuil) {
      UI.modale({ titre: 'Confirmer un paiement de ' + F.montant(st.montant) + ' ?', corps: '<p>Ce montant est élevé. Vérifiez le montant et le mode avant de valider.</p>',
        actions: [{ lib: 'Corriger', v: 'secondaire', fn: function (f) { f(); racine.querySelector('#montant').focus(); } },
          { lib: 'Confirmer le paiement', v: 'primaire', fn: function (f) { f(); UI.toast('Dans la démo, utilisez les montants proposés.', { type: 'info' }); racine.querySelector('#montant').focus(); } }] });
      return;
    }
    if (!modeComplet(t.id)) { UI.toast('Action de démonstration. Le parcours complet est préparé pour Issouf Tapsoba.', { type: 'info' }); return; }
    if (!sc.variantes[st.montant]) { UI.toast('Dans la démo, utilisez les montants proposés.', { type: 'info' }); return; }
    // Référence déjà utilisée pour ce mode (R-041) : rencontre d'un paiement existant ou cas déclenché dans les options
    var doublon = null;
    if (st.mode !== 'especes') D().paiements.forEach(function (p) { if (p.type === 'paiement' && p.mode === st.mode && p.reference && p.reference.toLowerCase() === st.reference.trim().toLowerCase()) doublon = p; });
    if (!doublon && Etat.option('erreur') === 'doublon') {
      if (st.mode === 'especes') { UI.toast('Cas « référence en doublon » : choisissez Mobile money et saisissez une référence.', { type: 'info' }); return; }
      doublon = D().paiements.filter(function (p) { return p.mode === st.mode && p.reference; })[0] || D().paiements.filter(function (p) { return p.reference; })[0];
      Etat.definirOption('erreur', null);
    }
    st.enCours = true; st.erreur = null; maj('droite'); racine.querySelector('#enc-erreur').innerHTML = '';
    App.attendre(function () {
      if (doublon) {
        var q = Data.quittance(doublon.quittanceId);
        st.enCours = false; st.erreur = { type: 'doublon', numero: q.numero, quittanceId: q.id }; maj('droite'); racine.querySelector('#enc-erreur').innerHTML = htmlErreur(); return;
      }
      if (Etat.option('erreur') === 'connexion') {
        Etat.definirOption('erreur', null);
        st.enCours = false; st.erreur = { type: 'connexion' }; maj('droite'); racine.querySelector('#enc-erreur').innerHTML = htmlErreur();
        var r = racine.querySelector('#btn-reessayer'); if (r) r.focus(); return;
      }
      var res = window.Mut.appliquer('encaisser', { montant: st.montant, mode: st.mode, reference: st.mode === 'especes' ? null : st.reference.trim(), tiers: st.payePar === 'tiers' ? st.tiers.trim() : null, note: st.note.trim() || null });
      UI.toast('Paiement de ' + F.montant(st.montant) + ' enregistré. Quittance n° ' + F.e(sc.prochainNumero) + ' prête.', { type: 'succes' });
      App.aller('/quittance/' + res.quittanceId + '?apres=1');
    });
  }

  function bindForm() {
    var montant = racine.querySelector('#montant');
    if (montant) {
      montant.addEventListener('input', function () {
        var n = parseInt(montant.value.replace(/\D/g, ''), 10) || 0;
        st.montant = n || null; montant.value = n ? F.nombre(n) : '';
        racine.querySelectorAll('#puces .puce').forEach(function (b) { b.setAttribute('aria-pressed', String(+b.getAttribute('data-m') === n)); });
        erreurChamp('#montant', '#montant-err', ''); maj('droite');
      });
    }
    racine.querySelectorAll('#modes input').forEach(function (i) { i.addEventListener('change', function () { st.mode = i.value; racine.querySelector('#champ-ref').classList.toggle('cache', i.value === 'especes'); if (i.value !== 'especes') racine.querySelector('#reference').focus(); }); });
    var pp = racine.querySelector('#paye-par'); if (pp) pp.addEventListener('change', function () { st.payePar = pp.value; racine.querySelector('#champ-tiers').classList.toggle('cache', pp.value !== 'tiers'); if (pp.value === 'tiers') racine.querySelector('#tiers').focus(); });
  }

  App.action('enc-choisir', function (el) { st.locataireId = el.getAttribute('data-id'); st.montant = null; st.erreur = null; st.q = Data.locataire(st.locataireId).nom; racine.querySelector('#rech').value = st.q; maj('tout'); var m = racine.querySelector('#montant'); if (m) m.focus(); });
  App.action('enc-changer', function () { st = neuf(''); racine.innerHTML = htmlTout(); bind(); racine.querySelector('#rech').focus(); });
  App.action('enc-montant', function (el) { st.montant = +el.getAttribute('data-m'); var m = racine.querySelector('#montant'); m.value = F.nombre(st.montant); racine.querySelectorAll('#puces .puce').forEach(function (b) { b.setAttribute('aria-pressed', String(b === el)); }); erreurChamp('#montant', '#montant-err', ''); maj('droite'); });
  App.action('enc-valider', function () { valider(); });

  function htmlTout() {
    return '<div class="enc"><div class="enc-col">' + htmlRecherche() + '<div id="enc-carte">' + htmlCarte() + '</div><div id="enc-form">' + htmlForm() + '</div></div>' +
      '<div class="enc-col enc-col-droite" id="enc-droite">' + htmlDroite() + '</div></div><div class="enc-action enc-action-mobile" id="enc-action-mobile">' + (st.locataireId && situation(st.locataireId).solde > 0 ? boutonValider() : '') + '</div>';
  }
  function bind() {
    var champ = racine.querySelector('#rech'), minuteur = null;
    champ.addEventListener('input', function () {
      st.q = champ.value; st.locataireId = null; st.montant = null; st.erreur = null;
      clearTimeout(minuteur);
      minuteur = setTimeout(function () {
        var r = chercher(st.q.trim());
        // un seul locataire trouvé : la carte apparaît directement
        if (st.q.trim().length >= 2 && r.locataires.length === 1 && !r.quittances.length) { st.locataireId = r.locataires[0].id; }
        maj('tout');
      }, 200);
    });
    bindForm();
  }

  App.enregistrer('encaisser', {
    meta: function () { return { titre: 'Encaisser', sous: 'Recherchez un locataire, saisissez le montant, validez.', repere: { type: 'v1' } }; },
    rendre: function (ctx) {
      st = neuf(ctx.query.q || '');
      if (st.q) { var r = chercher(st.q); if (r.locataires.length === 1 && !r.quittances.length) st.locataireId = r.locataires[0].id; }
      return '<div id="enc-racine">' + htmlTout() + '</div>';
    },
    apres: function (ctx, rac) {
      racine = rac.querySelector('#enc-racine'); bind();
      if (!st.locataireId) racine.querySelector('#rech').focus();
    }
  });
})();
