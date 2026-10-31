/*
 * S03 : Accueil et tableau de bord.
 * Variante rendue ici : gérant (encaissé, attendu, en retard, impayés par ancienneté, alertes du jour).
 * Les autres variantes (gestionnaire, caissier, propriétaire direct) arrivent avec leurs parcours.
 * Tous les chiffres viennent du jeu de données (tableauDeBord, impayes, paiements) : aucune règle métier ici.
 */
(function () {
  var F = window.Fmt, UI = window.UI, Data = window.Data;

  var TRANCHES = [
    { cle: '1-7', lib: '1 à 7 jours' }, { cle: '8-30', lib: '8 à 30 jours' },
    { cle: '31-59', lib: '31 à 59 jours' }, { cle: '60+', lib: '60 jours et plus' }
  ];

  function chiffre(lib, icone, valeur, sous, lien, extra) {
    return '<div class="carte' + (extra ? ' ' + extra : '') + '"><div class="chiffre">' +
      '<span class="chiffre-lib">' + UI.ic(icone) + lib + '</span>' +
      '<span class="chiffre-val" aria-label="' + F.e(F.montantSr(valeur)) + '">' + F.nombre(valeur) + '<small>FCFA</small></span>' +
      '<span class="chiffre-sous">' + sous + '</span></div>' +
      '<a class="carte-lien" href="' + lien.href + '">' + lien.lib + UI.ic('chevron-right', 'ic-sm') + '</a></div>';
  }

  function alerte(niveau, icone, titre, detail, lien, extra) {
    return '<li class="alerte"><span class="alerte-ic ' + niveau + '">' + UI.ic(icone) + '</span><div class="alerte-corps">' +
      '<div class="alerte-titre">' + titre + (extra ? ' ' + extra : '') + '</div><div class="alerte-detail">' + detail + '</div>' +
      '<a class="alerte-action" href="' + lien.href + '">' + lien.lib + UI.ic('chevron-right', 'ic-sm') + '</a></div></li>';
  }

  function blocAlertes(D) {
    var items = '';
    D.promesses.filter(function (p) { return p.statut === 'non_tenue'; }).forEach(function (p) {
      var t = Data.locataire(p.locataireId);
      items += alerte('erreur', 'alert-triangle', 'Promesse non tenue : ' + F.e(t.nom),
        F.montant(p.montant) + ' promis le ' + F.date(p.date) + ', non reçus.', { href: '#/impayes', lib: 'Voir les impayés' });
    });
    var rel = D.releves.filter(function (r) { return r.periode === D.meta.moisClos; });
    var aProduire = rel.filter(function (r) { return r.statut === 'a_produire'; }).length, brouillons = rel.filter(function (r) { return r.statut === 'brouillon'; }).length;
    items += alerte('avert', 'briefcase', 'Comptes rendus d’' + F.mois(D.meta.moisClos).split(' ')[0] + ' à produire',
      aProduire + ' mandants sans compte rendu' + (brouillons ? ', ' + brouillons + ' brouillon à valider' : '') + '.', { href: '#/comptes-rendus', lib: 'Produire un compte rendu' });
    D.demandesCorrection.filter(function (d) { return d.statut === 'en_attente'; }).forEach(function (d) {
      var p = Data.paiement(d.paiementId), auteur = Data.utilisateur(d.auteurId);
      items += alerte('info', 'pen', 'Demande de correction en attente',
        F.e(auteur.nom) + ' · paiement de ' + F.montant(p.montant) + ' (' + F.e(Data.nomLocataires(p.locataireIds)) + ').', { href: '#/paiements', lib: 'Traiter la demande' });
    });
    D.agenda.filter(function (a) { return a.type === 'fin_bail' && !a.fait; }).forEach(function (a) {
      items += alerte('info', 'calendar', F.e(a.titre), F.e(a.detail) + '.', { href: '#/agenda', lib: 'Ouvrir l’agenda' }, UI.repere({ type: 'v1', pilote: true, mini: true }));
    });
    return '<section class="carte tdb-d" aria-labelledby="t-alertes"><div class="carte-entete"><h2 id="t-alertes" class="carte-titre">' + UI.ic('bell') + 'Alertes du jour</h2></div><ul>' + items + '</ul></section>';
  }

  function blocImpayes(D) {
    var t = D.tableauDeBord, max = 0;
    TRANCHES.forEach(function (r) { max = Math.max(max, t.parTranche[r.cle].montant); });
    var lignes = TRANCHES.map(function (r, i) {
      var x = t.parTranche[r.cle], pct = max ? Math.max(3, Math.round(100 * x.montant / max)) : 0;
      return '<div class="barre-ligne"><div class="barre-libelle"><span>' + r.lib + ' · ' + x.nombre + (x.nombre > 1 ? ' échéances' : ' échéance') + '</span><strong>' + F.montant(x.montant) + '</strong></div>' +
        '<div class="barre-piste" role="img" aria-label="' + r.lib + ' : ' + F.e(F.montantSr(x.montant)) + '"><div class="barre-rempli' + (i === 0 ? ' t1' : '') + '" style="width:' + pct + '%"></div></div></div>';
    }).join('');
    return '<section class="carte tdb-g" aria-labelledby="t-imp"><div class="carte-entete"><h2 id="t-imp" class="carte-titre">' + UI.ic('alert-triangle') + 'Impayés par ancienneté</h2>' +
      '<a class="carte-lien" href="#/impayes">Voir les impayés' + UI.ic('chevron-right', 'ic-sm') + '</a></div>' + lignes + '</section>';
  }

  var ICONE_MODE = { especes: 'banknote', mobile_money: 'phone', virement: 'building', cheque: 'file-text' };
  function blocPaiements() {
    var lignes = Data.paiementsRecents(5).map(function (p) {
      var quand = p.dateSaisie === Data.meta().aujourdhui ? 'Aujourd’hui ' + p.heure : F.date(p.date);
      return '<li class="pay-ligne"><span class="pay-mode" aria-hidden="true">' + UI.ic(ICONE_MODE[p.mode]) + '</span>' +
        '<div class="liste-gauche" style="flex:1"><span class="liste-titre">' + F.e(Data.nomLocataires(p.locataireIds)) + '</span><span class="liste-sous">' + F.e(Data.lot(p.lotId).nom) + ' · ' + Data.modeLibelle(p.mode) + ' · ' + quand + '</span></div>' +
        '<div class="liste-droite"><span class="liste-montant">' + F.montant(p.montant) + '</span>' + UI.statut('valide') + '</div></li>';
    }).join('');
    return '<section class="carte tdb-moitie" aria-labelledby="t-pay"><div class="carte-entete"><h2 id="t-pay" class="carte-titre">' + UI.ic('receipt') + 'Derniers encaissements</h2>' +
      '<a class="carte-lien" href="#/paiements">Tout voir' + UI.ic('chevron-right', 'ic-sm') + '</a></div><ul>' + lignes + '</ul></section>';
  }

  function blocBaux(D) {
    var t = D.tableauDeBord, s = t.statutsBaux, n = t.paiementsSurTotal.total;
    var seg = [['rep-paid', s.paye, 'Payés'], ['rep-credit', s.avance, 'Avance'], ['rep-partial', s.partiel, 'Partiels'], ['rep-due', s.due, 'Dus'], ['rep-late', s.retard, 'En retard']];
    var barre = seg.filter(function (x) { return x[1] > 0; }).map(function (x) { return '<span class="' + x[0] + '" style="flex:' + x[1] + '"></span>'; }).join('');
    var icones = { 'Payés': 'check-circle', 'Avance': 'arrow-up-circle', 'Partiels': 'half-circle', 'Dus': 'bell', 'En retard': 'alert-triangle' };
    var leg = seg.map(function (x) { return '<li><span class="pt ' + x[0] + '"></span>' + UI.ic(icones[x[2]], 'ic-sm') + x[2] + '<strong>' + x[1] + '</strong></li>'; }).join('');
    return '<section class="carte tdb-moitie" aria-labelledby="t-baux"><div class="carte-entete"><h2 id="t-baux" class="carte-titre">' + UI.ic('file-text') + 'Loyers de ' + F.mois(t.mois) + '</h2>' +
      '<a class="carte-lien" href="#/baux">Voir les baux' + UI.ic('chevron-right', 'ic-sm') + '</a></div>' +
      '<div class="repartition" role="img" aria-label="' + n + ' baux : ' + s.paye + ' payés, ' + s.avance + ' avec avance, ' + s.partiel + ' partiels, ' + s.due + ' dû aujourd’hui, ' + s.retard + ' en retard">' + barre + '</div>' +
      '<ul class="legende-liste">' + leg + '</ul>' +
      '<div class="carte-pied secondaire" style="font-size:var(--text-sm)">' + t.lots.total + ' lots : ' + t.lots.occupes + ' occupés, ' + t.lots.vacants + ' vacants, ' + t.lots.travaux + ' en travaux.</div></section>';
  }

  function rendreGerant(ctx) {
    var D = ctx.D, t = D.tableauDeBord, nb = D.paiements.filter(function (p) { return p.type === 'paiement' && p.statut === 'valide' && p.date.slice(0, 7) === t.mois; }).length;
    var moisLib = F.mois(t.mois);
    var chiffres = '<div class="tdb-chiffres">' +
      chiffre('Encaissé en ' + moisLib, 'check-circle', t.encaisse, nb + ' paiements au ' + F.date(D.meta.aujourdhui) +
        '<div class="tdb-progres"><div class="tdb-progres-piste" role="img" aria-label="' + t.tauxRecouvrement + ' % de l’attendu"><div class="tdb-progres-rempli" style="width:' + t.tauxRecouvrement + '%"></div></div>' +
        '<div class="tdb-progres-texte"><span>' + t.tauxRecouvrement + ' % de l’attendu</span><span class="badge badge-pointille">Formule à confirmer</span></div></div>',
        { href: '#/paiements', lib: 'Voir les paiements' }) +
      chiffre('Attendu en ' + moisLib, 'calendar', t.attendu, t.paiementsSurTotal.total + ' baux · ' + t.lots.occupes + ' lots occupés', { href: '#/baux', lib: 'Voir les baux' }) +
      chiffre('En retard', 'alert-triangle', t.enRetard.montant, t.enRetard.locataires + ' locataires · ' + t.enRetard.echeances + ' échéances', { href: '#/impayes', lib: 'Voir les impayés' }, 'carte-alerte chiffre-retard') +
      '</div>';
    return '<div class="tdb">' + chiffres + blocImpayes(D) + blocAlertes(D) + blocPaiements() + blocBaux(D) + '</div>';
  }

  /* ---------- Gestionnaire : « À faire aujourd'hui » (déduit de F12, F17, F22) ---------- */
  function blocAFaire(id, icone, titre, n, contenu, vide) {
    return '<section class="carte" aria-labelledby="' + id + '"><div class="carte-entete"><h2 id="' + id + '" class="carte-titre">' + UI.ic(icone) + titre + '</h2><span class="badge badge-neutre">' + n + '</span></div>' +
      (n ? '<ul class="liste">' + contenu + '</ul>' : '<p class="secondaire" style="display:flex;gap:8px;align-items:center">' + UI.ic('check-circle') + vide + '</p>') + '</section>';
  }
  function ligneAFaire(titre, sous, droite) {
    return '<li><div class="liste-gauche"><span class="liste-titre">' + titre + '</span><span class="liste-sous">' + sous + '</span></div><div class="liste-droite">' + droite + '</div></li>';
  }
  function rendreGestionnaire(ctx) {
    var D = ctx.D, jour = D.meta.aujourdhui;
    var nonTenues = D.promesses.filter(function (p) { return p.statut === 'non_tenue'; });
    var promesses = nonTenues.map(function (p) {
      return ligneAFaire(F.e(Data.locataire(p.locataireId).nom), F.montant(p.montant) + ' promis le ' + F.date(p.date) + ', non reçus', '<span class="badge badge-late">' + UI.ic('alert-triangle') + 'Non tenue</span>' + UI.btn('Relancer', { v: 'secondaire', petit: true, ic: 'message', action: 'imp-relancer', data: ' data-bail="' + p.bailId + '"' }));
    }).join('');
    var rappels = D.rappels.map(function (r) {
      return ligneAFaire(F.e(Data.locataire(r.locataireId).nom), 'Échéance du ' + F.date(r.dateEcheance) + ' · reste ' + F.montant(r.reste), UI.statut('a_venir', 'J−3') + UI.btn('Rappeler', { v: 'secondaire', petit: true, ic: 'message', action: 'home-rappel', data: ' data-bail="' + r.bailId + '" data-ech="' + r.echeanceId + '"' }));
    }).join('');
    var dues = D.echeances.filter(function (e) { return e.dateEcheance === jour && e.reste > 0; }).map(function (e) {
      var b = Data.bail(e.bailId), t = Data.locataire(b.locataireIds[0]);
      return ligneAFaire(F.e(t.nom), F.e(Data.lot(b.lotId).nom) + ' · ' + F.mois(e.periode), '<span class="liste-montant">' + F.montant(e.reste) + '</span>' + UI.statut('due') + UI.btn('Encaisser', { v: 'secondaire', petit: true, ic: 'banknote', href: '#/encaisser?q=' + encodeURIComponent(t.nomFamille) }));
    }).join('');
    var planifiees = D.relances.filter(function (r) { return r.prochaineAction && r.prochaineAction.date === jour; });
    var plan = planifiees.map(function (r) {
      return ligneAFaire(F.e(Data.locataire(r.locataireId).nom), F.e(r.prochaineAction.libelle), UI.neutre('Aujourd’hui', 'calendar') + UI.btn('Relancer', { v: 'secondaire', petit: true, ic: 'message', action: 'imp-relancer', data: ' data-bail="' + r.bailId + '"' }));
    }).join('');
    var fins = D.agenda.filter(function (a) { return a.type === 'fin_bail' && !a.fait; }).map(function (a) {
      return ligneAFaire(F.e(a.titre), F.e(a.detail), UI.repere({ type: 'v1', pilote: true, mini: true }) + UI.btn('Ouvrir l’agenda', { v: 'tertiaire', petit: true, href: '#/agenda' }));
    }).join('');
    var n = nonTenues.length + D.rappels.length;
    return '<div class="grille-faire">' + blocAFaire('t-nt', 'alert-triangle', 'Promesses non tenues', nonTenues.length, promesses, 'Aucune promesse non tenue.') +
      blocAFaire('t-rap', 'calendar', 'Rappels à envoyer (J−3)', D.rappels.length, rappels, 'Aucun rappel à envoyer.') +
      blocAFaire('t-due', 'bell', 'Échéances dues aujourd’hui', D.echeances.filter(function (e) { return e.dateEcheance === jour && e.reste > 0; }).length, dues, 'Aucune échéance due aujourd’hui.') +
      blocAFaire('t-plan', 'clock', 'Relances planifiées aujourd’hui', planifiees.length, plan, 'Aucune relance planifiée.') +
      blocAFaire('t-fin', 'file-text', 'Fins de bail à 60 jours', D.agenda.filter(function (a) { return a.type === 'fin_bail' && !a.fait; }).length, fins, 'Aucune fin de bail proche.') + '</div>';
  }
  /* ---------- Caissier : « Aujourd'hui » (encaissements du jour, par mode, dernier paiement) ---------- */
  var MODE_LIB = { especes: 'Espèces', mobile_money: 'Mobile money', virement: 'Virement', cheque: 'Chèque' };
  function rendreCaissier(ctx) {
    var D = ctx.D, tb = D.tableauDeBord, ajd = tb.aujourdhui, jour = D.meta.aujourdhui;
    var paiements = D.paiements.filter(function (p) { return p.type === 'paiement' && p.statut === 'valide' && p.dateSaisie === jour; }).sort(function (a, b) { return b.heure.localeCompare(a.heure); });
    var modes = Object.keys(ajd.parMode).map(function (m) { return '<li><span>' + MODE_LIB[m] + '</span><span class="gras num">' + F.montant(ajd.parMode[m]) + '</span></li>'; }).join('');
    var chiffres = '<div class="tdb-chiffres"><div class="carte"><div class="chiffre"><span class="chiffre-lib">' + UI.ic('banknote') + 'Encaissé aujourd’hui</span><span class="chiffre-val" aria-label="' + F.e(F.montantSr(ajd.montant)) + '">' + F.nombre(ajd.montant) + '<small>FCFA</small></span><span class="chiffre-sous">' + ajd.nombre + (ajd.nombre > 1 ? ' paiements' : ' paiement') + '</span></div></div>' +
      '<div class="carte"><div class="carte-titre" style="margin-bottom:8px">' + UI.ic('wallet') + 'Par mode</div>' + (modes ? '<ul class="liste">' + modes + '</ul>' : '<p class="secondaire">Aucun encaissement.</p>') + '</div></div>';
    var der = paiements.length ? '<section class="carte" aria-labelledby="t-der"><div class="carte-entete"><h2 id="t-der" class="carte-titre">' + UI.ic('receipt') + 'Dernier paiement saisi</h2></div><ul class="liste">' + paiements.slice(0, 1).map(function (p) { var q = Data.quittance(p.quittanceId); return '<li><div class="liste-gauche"><span class="liste-titre">' + F.e(Data.nomLocataires(p.locataireIds)) + '</span><span class="liste-sous">' + F.e(Data.lot(p.lotId).nom) + ' · ' + p.heure + ' · <a href="#/quittance/' + q.id + '">' + F.e(q.numero) + '</a></span></div><div class="liste-droite"><span class="liste-montant">' + F.montant(p.montant) + '</span>' + UI.statut('valide') + '</div></li>'; }).join('') + '</ul></section>' : '';
    var liste = paiements.length ? '<section class="carte" aria-labelledby="t-j2"><div class="carte-entete"><h2 id="t-j2" class="carte-titre">' + UI.ic('clock') + 'Journal du jour</h2><a class="carte-lien" href="#/paiements">Tout voir' + UI.ic('chevron-right', 'ic-sm') + '</a></div><ul class="liste">' + paiements.map(function (p) { return '<li><div class="liste-gauche"><span class="liste-titre">' + F.e(Data.nomLocataires(p.locataireIds)) + '</span><span class="liste-sous">' + F.e(Data.lot(p.lotId).nom) + ' · ' + MODE_LIB[p.mode] + ' · ' + p.heure + '</span></div><span class="liste-montant">' + F.montant(p.montant) + '</span></li>'; }).join('') + '</ul></section>'
      : '<section class="carte">' + UI.etatVide({ ic: 'banknote', titre: 'Aucun encaissement aujourd’hui', texte: 'Recherchez un locataire pour encaisser.', action: UI.btn('Encaisser', { v: 'primaire', href: '#/encaisser' }) }) + '</section>';
    return '<div class="grille-faire">' + chiffres.replace('class="tdb-chiffres"', 'class="tdb-chiffres" style="grid-column:1/-1"') + der + liste + '</div><p class="muted" style="margin-top:12px;font-size:var(--text-sm)">La clôture de caisse viendra après les premiers entretiens (écran reporté).</p>';
  }

  /* ---------- Propriétaire direct : « X payés sur Y, Z en retard », rappel en un clic ---------- */
  function rendreProprietaire(ctx) {
    var D = ctx.D, tb = D.tableauDeBord, s = tb.statutsBaux, payes = s.paye + s.avance;
    var haut = '<section class="carte" aria-labelledby="t-mois"><h2 id="t-mois" class="sr-only">Ce mois</h2><div class="chiffre"><span class="chiffre-lib">' + UI.ic('calendar') + 'Ce mois · ' + F.mois(tb.mois) + '</span><span class="chiffre-val prop-phrase"><strong>' + payes + '</strong> payés sur ' + tb.paiementsSurTotal.total + ', <span class="texte-neg">' + s.retard + ' en retard</span></span><span class="chiffre-sous">' + F.montant(tb.encaisse) + ' encaissés sur ' + F.montant(tb.attendu) + ' attendus' + (s.partiel ? ' · ' + s.partiel + ' partiel' : '') + '</span></div></section>';
    var retard = D.impayes.reduce(function (m, i) { (m[i.bailId] = m[i.bailId] || []).push(i); return m; }, {});
    var cles = Object.keys(retard), rel = cles.map(function (k) {
      var l = retard[k], t = Data.locataire(l[0].locataireId), reste = l.reduce(function (a, x) { return a + x.reste; }, 0), anc = Math.max.apply(null, l.map(function (x) { return x.ancienneteJours; }));
      return '<li><div class="liste-gauche"><span class="liste-titre">' + F.e(t.nom) + '</span><span class="liste-sous">' + F.e(Data.lot(l[0].lotId).nom) + ' · ' + F.montant(reste) + '</span></div><div class="liste-droite" style="flex-direction:row;align-items:center;gap:8px">' + UI.statut('en_retard', F.jours(anc)) + UI.btn('Rappeler', { v: 'secondaire', petit: true, ic: 'message', action: 'imp-relancer', data: ' data-bail="' + k + '"' }) + '</div></li>';
    }).join('');
    var aRelancer = '<section class="carte" aria-labelledby="t-rel"><div class="carte-entete"><h2 id="t-rel" class="carte-titre">' + UI.ic('alert-triangle') + 'À rappeler</h2><a class="carte-lien" href="#/impayes">Voir les impayés' + UI.ic('chevron-right', 'ic-sm') + '</a></div>' + (cles.length ? '<ul class="liste">' + rel + '</ul>' : '<p class="secondaire">Aucun impayé. Bravo, tout est à jour.</p>') + '</section>';
    return '<div class="grille-faire"><div style="grid-column:1/-1">' + haut + '</div>' + aRelancer + blocPaiements() + '</div>';
  }

  window.App.action('home-rappel', function (el) { window.Relance.ouvrir(el.getAttribute('data-bail'), 'rappel', el.getAttribute('data-ech')); });

  window.App.enregistrer('accueil', {
    meta: function (ctx) {
      var r = ctx.role.id;
      if (r === 'gerant') return { titre: 'Tableau de bord', sous: F.majuscule(F.dateLongue(ctx.D.meta.aujourdhui)) + ' · ' + F.e(ctx.D.organisation.nom), repere: { type: 'v1' },
        actions: UI.btn('Encaisser un loyer', { v: 'primaire', href: '#/encaisser', ic: 'banknote' }) };
      if (r === 'gestionnaire') return { titre: 'À faire aujourd’hui', sous: F.majuscule(F.dateLongue(ctx.D.meta.aujourdhui)) + ' · chaque matin, vous savez par où commencer.', repere: { type: 'v1' },
        actions: UI.btn('Voir les impayés', { v: 'primaire', href: '#/impayes', ic: 'alert-triangle' }) };
      if (r === 'caissier') return { titre: 'Aujourd’hui', sous: F.majuscule(F.dateLongue(ctx.D.meta.aujourdhui)) + ' · vos encaissements du jour', repere: { type: 'v1' }, actions: UI.btn('Encaisser', { v: 'primaire', href: '#/encaisser', ic: 'banknote' }) };
      var tb = ctx.D.tableauDeBord, s = tb.statutsBaux;
      return { titre: 'Accueil', sous: F.majuscule(F.dateLongue(ctx.D.meta.aujourdhui)) + ' · ce mois : ' + (s.paye + s.avance) + ' payés sur ' + tb.paiementsSurTotal.total + ', ' + s.retard + ' en retard', repere: { type: 'v1' }, actions: UI.btn('Encaisser un loyer', { v: 'primaire', href: '#/encaisser', ic: 'banknote' }) };
    },
    rendre: function (ctx) {
      if (ctx.role.id === 'gerant') return rendreGerant(ctx);
      if (ctx.role.id === 'gestionnaire') return rendreGestionnaire(ctx);
      if (ctx.role.id === 'caissier') return rendreCaissier(ctx);
      return rendreProprietaire(ctx);
    }
  });
})();
