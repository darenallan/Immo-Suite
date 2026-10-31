/*
 * Accès au jeu de données fictives (window.DEMO, rempli par data/*.js).
 * Ce module ne fait que retrouver des objets liés (jointures d'affichage). Aucun calcul métier :
 * montants, statuts, totaux et numéros viennent tels quels du jeu de données (vérifié par scripts/verifier-donnees.js).
 */
(function () {
  function org() { return window.DEMO[window.Etat.orgCle()]; }
  function par(liste, id) { for (var i = 0; i < liste.length; i++) if (liste[i].id === id) return liste[i]; return null; }
  var Data = {
    org: org,
    meta: function () { return org().meta; },
    locataire: function (id) { return par(org().locataires, id); },
    lot: function (id) { return par(org().lots, id); },
    bien: function (id) { return par(org().biens, id); },
    bail: function (id) { return par(org().baux, id); },
    mandant: function (id) { return par(org().mandants, id); },
    utilisateur: function (id) { return par(org().utilisateurs, id); },
    echeance: function (id) { return par(org().echeances, id); },
    paiement: function (id) { return par(org().paiements, id); },
    quittance: function (id) { return par(org().quittances, id); },
    intervention: function (id) { return par(org().interventions, id); },
    releve: function (id) { return par(org().releves, id); },
    // Paiements validés les plus récents (écriture dans le journal : date de saisie puis heure)
    paiementsRecents: function (n) {
      return org().paiements.filter(function (p) { return p.type === 'paiement'; })
        .sort(function (a, b) { return (b.dateSaisie + b.heure).localeCompare(a.dateSaisie + a.heure); }).slice(0, n);
    },
    // Situation d'un locataire à la date de démonstration : échéances non soldées jusqu'au mois de référence (lecture des statuts stockés)
    situation: function (tid) {
      var d = org(), es = [];
      d.baux.filter(function (b) { return b.locataireIds.indexOf(tid) >= 0; }).forEach(function (b) {
        d.echeances.forEach(function (e) { if (e.bailId === b.id && e.periode <= d.meta.moisReference && e.reste > 0) es.push(e); });
      });
      es.sort(function (a, c) { return a.dateEcheance.localeCompare(c.dateEcheance); });
      var retard = es.filter(function (e) { return e.statut === 'en_retard'; });
      return { echeances: es, retard: retard, solde: es.reduce(function (s, e) { return s + e.reste; }, 0), statut: retard.length ? 'en_retard' : es.length ? es[0].statut : 'payee', anciennete: retard.length ? Math.max.apply(null, retard.map(function (e) { return e.ancienneteJours; })) : 0 };
    },
    nomLocataires: function (ids) { return ids.map(function (i) { return par(org().locataires, i).nom; }).join(' et '); },
    modeLibelle: function (m) { return { especes: 'Espèces', mobile_money: 'Mobile money', virement: 'Virement', cheque: 'Chèque' }[m] || m; }
  };
  window.Data = Data;
})();
