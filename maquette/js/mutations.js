/*
 * Modifications de démonstration. Chaque action de la démo (encaisser, relancer, valider un compte rendu, importer…)
 * est une fonction nommée qui met à jour le jeu de données en mémoire. Elle est consignée dans le journal de l'état
 * puis rejouée au rechargement. Aucune règle métier générale : les valeurs viennent des scénarios précalculés
 * (data/agence.js, scenarios) ou des paramètres passés par l'écran.
 */
(function () {
  var fns = {}, Mut = { dernier: null };
  var D = function () { return window.DEMO.hor; };
  function par(liste, id) { for (var i = 0; i < liste.length; i++) if (liste[i].id === id) return liste[i]; return null; }
  function pad(n, l) { return String(n).padStart(l || 2, '0'); }

  function def(nom, fn) { fns[nom] = fn; }

  // Parcours 1 : encaisser le loyer d'Issouf Tapsoba (75 000, 50 000 ou 100 000 FCFA)
  def('encaisser', function (p) {
    var d = D(), sc = d.scenarios.encaisser, v = sc.variantes[p.montant], seq = d.meta.prochaineSequence;
    var pid = 'P' + pad(seq, 3), qid = 'Q' + pad(seq, 3), eNov = par(d.echeances, 'E-B08-2026-11'), eDec = par(d.echeances, 'E-B08-2026-12');
    var affs = [{ echeanceId: eNov.id, ligne: 'loyer', montant: v.affectations[0].affecte }];
    if (v.avance) affs.push({ echeanceId: eDec.id, ligne: 'loyer', montant: v.avance.montant });
    var paiement = { id: pid, type: 'paiement', bailId: 'B08', lotId: 'L008', locataireIds: ['T08'], date: d.meta.aujourdhui, dateSaisie: d.meta.aujourdhui, heure: d.meta.heure, montant: p.montant, mode: p.mode, reference: p.reference || null,
      recuPar: p.recuPar || 'U3', payePar: p.tiers ? { type: 'tiers', nom: p.tiers, telephone: null } : { type: 'locataire', locataireId: 'T08' }, statut: 'valide', note: p.note || null, journeeCaisse: d.meta.aujourdhui, affectations: affs, creditBail: 0, quittanceId: qid };
    d.paiements.push(paiement);
    d.quittances.push({ id: qid, numero: sc.prochainNumero, paiementId: pid, bailId: 'B08', date: d.meta.aujourdhui, statut: 'emise', rectificative: false, remplaceQuittanceId: null, remplaceeParQuittanceId: null,
      montant: p.montant, montantLettres: v.montantLettres, periode: 'novembre 2026', echeanceDate: '2026-11-13', ventilation: [{ libelle: 'Loyer', montant: affs[0].montant }, { libelle: 'Charges', montant: 0 }],
      soldeRestant: v.soldeRestant, avance: v.avance, creditBail: 0, envois: [] });
    eNov.paye = affs[0].montant; eNov.reste = eNov.montant - eNov.paye; eNov.statut = v.affectations[0].statut;
    if (v.avance) { eDec.paye = v.avance.montant; eDec.reste = eDec.montant - eDec.paye; eDec.statut = 'partielle'; }
    // Totaux du tableau de bord et de la journée de caisse (valeurs mises à jour pour rester égales aux listes)
    var tb = d.tableauDeBord, cle = { 75000: 'paye', 50000: 'partiel', 100000: 'avance' }[p.montant];
    tb.encaisse += p.montant; tb.tauxRecouvrement = Math.round(tb.encaisse * 100 / tb.attendu);
    tb.aujourdhui.nombre += 1; tb.aujourdhui.montant += p.montant; tb.aujourdhui.parMode[p.mode] = (tb.aujourdhui.parMode[p.mode] || 0) + p.montant;
    tb.statutsBaux.due -= 1; tb.statutsBaux[cle] += 1; tb.paiementsSurTotal.payes = tb.statutsBaux.paye + tb.statutsBaux.avance; tb.paiementsSurTotal.partiels = tb.statutsBaux.partiel;
    var j = par(d.caisse.map(function (c) { return { id: c.date, c: c }; }), d.meta.aujourdhui).c;
    j.paiementIds.push(pid); if (p.mode === 'especes') j.attendu += p.montant;
    d.meta.prochaineSequence = seq + 1; d.meta.prochainNumero = 'HOR-2026-' + pad(seq + 1, 6);
    sc.fait = { paiementId: pid, quittanceId: qid, montant: p.montant };
    Mut.dernier = { paiementId: pid, quittanceId: qid };
  });

  // Marque l'envoi d'une quittance (historique des envois, R-055)
  def('quittance.envoyer', function (p) {
    var q = par(D().quittances, p.id); q.envois.push({ date: D().meta.aujourdhui, canal: p.canal || 'whatsapp', par: p.par || 'U3' });
  });

  // Parcours 2 : relance (ou rappel J-3), promesse de paiement, prochaine action
  def('relance', function (p) {
    var d = window.DEMO[p.org || 'hor'], n = d.relances.length + 1, b = par(d.baux, p.bailId), id = 'RL' + pad(n);
    var cible = d.echeances.filter(function (e) { return e.bailId === p.bailId && e.statut === 'en_retard'; });
    var ech = p.etape === 'J-3' ? par(d.echeances, p.echeanceId) : cible[cible.length - 1];
    var promesseId = null;
    if (p.promesse) {
      promesseId = 'PRM' + (d.promesses.length + 1);
      d.promesses.push({ id: promesseId, bailId: p.bailId, locataireId: b.locataireIds[0], echeanceId: ech.id, date: p.promesse.date, montant: p.promesse.montant, creeLe: d.meta.aujourdhui, auteurId: p.auteurId || 'U2', statut: 'en_cours' });
    }
    var r = { id: id, bailId: p.bailId, locataireId: b.locataireIds[0], echeanceId: ech.id, date: d.meta.aujourdhui, heure: d.meta.heure, canal: p.canal, etape: p.etape, ton: p.ton, texte: p.texte, auteurId: p.auteurId || 'U2', resultat: p.promesse ? 'promesse' : 'message_envoye', promesseId: promesseId, prochaineAction: p.prochaineAction || null };
    d.relances.push(r);
    d.impayes.filter(function (i) { return i.bailId === p.bailId; }).forEach(function (i) { i.derniereRelanceId = id; if (promesseId) i.promesseId = promesseId; if (p.prochaineAction) i.prochaineAction = p.prochaineAction; });
    if (p.etape === 'J-3') d.rappels = d.rappels.filter(function (x) { return x.echeanceId !== p.echeanceId; });
    // La prochaine action planifiée apparaît dans l'agenda (S15)
    if (p.prochaineAction) d.agenda.push({ id: 'AG' + (d.agenda.length + 1), type: p.promesse ? 'promesse' : 'rappel', date: p.prochaineAction.date, titre: p.prochaineAction.libelle + ' : ' + par(d.locataires, b.locataireIds[0]).nom, detail: p.promesse ? p.promesse.montant + ' FCFA promis' : 'Après la relance du ' + d.meta.aujourdhui, assigneId: p.auteurId || 'U2', fait: false, objet: { kind: 'bail', id: p.bailId } });
  });

  // Parcours 3 : cycle de vie d'un compte rendu
  def('releve.produire', function (p) { var r = par(D().releves, p.id); r.statut = 'brouillon'; r.creeLe = D().meta.aujourdhui; });
  def('releve.valider', function (p) { var r = par(D().releves, p.id); r.statut = 'valide'; r.valideLe = D().meta.aujourdhui; r.validePar = 'U1'; });
  def('releve.rouvrir', function (p) { var r = par(D().releves, p.id); r.statut = 'brouillon'; r.version += 1; r.valideLe = null; r.validePar = null; r.motifReouverture = p.motif; });
  def('releve.reverser', function (p) { var r = par(D().releves, p.id); r.statut = 'paye'; r.reversement = { date: p.date, mode: p.mode, reference: p.reference, montant: r.net }; });
  def('releve.ajuster', function (p) { var r = par(D().releves, p.id), a = r.ajustementDemo; r.ajuste = true; r.depenses.push({ interventionId: null, date: D().meta.aujourdhui, libelle: a.libelle, montant: a.montant, piece: false, ajustement: true }); r.depensesTotal = a.depensesTotal; r.netAffiche = a.net; });
  def('releve.annulerAjustement', function (p) { var r = par(D().releves, p.id); r.ajuste = false; r.depenses = r.depenses.filter(function (x) { return !x.ajustement; }); r.depensesTotal = r.depenses.reduce(function (s, x) { return s + x.montant; }, 0); delete r.netAffiche; });

  // Parcours 4 : import (la démo n'ajoute aucune donnée, elle inscrit seulement une ligne d'historique, R-080)
  def('import.confirmer', function (p) {
    var d = window.DEMO[p.org || 'hor']; d.imports.unshift({ id: 'IMP' + (d.imports.length + 1), date: d.meta.aujourdhui, fichier: p.fichier, lignes: 200, creees: p.creees, erreurs: 3, fusions: p.fusions, statut: 'termine', annulable: true, parId: p.parId || 'U2', note: 'Démonstration : aucune donnée ajoutée au portefeuille' });
  });
  def('import.annuler', function (p) { var i = par(window.DEMO[p.org || 'hor'].imports, p.id); i.statut = 'annule'; i.annulable = false; });

  // Parcours 5 : ajout d'une intervention (modification de démonstration)
  def('intervention.ajouter', function (p) {
    var d = window.DEMO[p.org || 'hor'], n = d.interventions.length + 1, lot = par(d.lots, p.lotId);
    d.interventions.push({ id: (p.org === 'saw' ? 'SI' : 'I') + pad(n), lotId: p.lotId, bienId: lot.bienId, date: p.date, nature: p.nature, description: p.nature, cout: p.cout, prestataireId: p.prestataireId, payeur: p.payeur, statut: p.statut || 'planifiee', imputable: p.payeur === 'agence', imputeAuReleveId: null, horsHistorique: false, documentIds: [] });
  });

  // S11 : demande de correction d'un paiement (jamais de modification directe, R-047)
  def('correction.demander', function (p) {
    var d = D(); d.demandesCorrection.push({ id: 'DC' + (d.demandesCorrection.length + 1), paiementId: p.paiementId, auteurId: p.auteurId || 'U3', date: d.meta.aujourdhui, motif: p.motif, statut: 'en_attente', traiteeLe: null, traiteePar: null });
  });

  // S15 : agenda (tâche faite, nouvelle tâche)
  def('agenda.fait', function (p) { var a = par(window.DEMO[p.org || 'hor'].agenda, p.id); a.fait = !!p.fait; });
  def('agenda.ajouter', function (p) {
    var d = window.DEMO[p.org || 'hor']; d.agenda.push({ id: 'AG' + (d.agenda.length + 1), type: 'tache', date: p.date, titre: p.titre, detail: p.detail || 'Tâche ajoutée à la main', assigneId: p.assigneId, fait: false, objet: { kind: 'tache', id: null } });
  });
  // S18 : utilisateurs et invitations
  def('utilisateur.inviter', function (p) {
    var d = window.DEMO[p.org || 'hor'], n = d.utilisateurs.length + 1;
    d.utilisateurs.push({ id: (p.org === 'saw' ? 'S' : 'U') + n, nom: p.nom, role: p.role, statut: 'invitation_en_attente', telephone: p.telephone });
  });
  def('utilisateur.renvoyer', function (p) { par(window.DEMO[p.org || 'hor'].utilisateurs, p.id).statut = 'invitation_en_attente'; });
  def('utilisateur.desactiver', function (p) { par(window.DEMO[p.org || 'hor'].utilisateurs, p.id).statut = 'desactive'; });
  def('utilisateur.role', function (p) { par(window.DEMO[p.org || 'hor'].utilisateurs, p.id).role = p.role; });
  // S17 : paramètres (droits réglables, accès support)
  function acces(p) { var d = window.DEMO[p.org || 'hor']; if (!d.parametres.accesSupport) d.parametres.accesSupport = { actif: false, jusqu: null, historique: [] }; return d.parametres.accesSupport; }
  def('parametres.droit', function (p) { window.DEMO[p.org || 'hor'].parametres.droits[p.cle] = !!p.valeur; });
  def('support.autoriser', function (p) { var a = acces(p); a.actif = true; a.jusqu = p.jusqu; a.historique.unshift({ date: window.DEMO[p.org || 'hor'].meta.aujourdhui, action: 'Accès autorisé jusqu’au ' + p.jusqu, par: p.par }); });
  def('support.revoquer', function (p) { var a = acces(p); a.actif = false; a.jusqu = null; a.historique.unshift({ date: window.DEMO[p.org || 'hor'].meta.aujourdhui, action: 'Accès révoqué', par: p.par }); });

  Mut.appliquer = function (nom, params) {
    fns[nom](params || {});
    window.Etat.consigner(nom, params || {});
    return Mut.dernier;
  };
  // Rejoue le journal au chargement de la page (sans le consigner de nouveau)
  Mut.rejouer = function () {
    (window.Etat.donnees.journal || []).forEach(function (m) { if (fns[m.nom]) fns[m.nom](m.params || {}); });
  };
  window.Mut = Mut;
})();
