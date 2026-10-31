/*
 * Film complet de 3 minutes (MAQUETTE.md, « Film complet »). Chaque séquence reprend un parcours condensé :
 * les étapes non retenues sont jouées sans bulle (muet) pour que l'enchaînement des écrans reste cohérent.
 * Durées de la lecture automatique : accroche 15 s, encaisser 40 s, relancer 40 s, rendre compte 40 s,
 * migrer 20 s, mémoire 15 s, clôture 10 s.
 */
(function () {
  var DG = window.DemoGuidee;
  // garde : { indice d'étape : durée en secondes } ; surcharge : propriétés de la première étape retenue
  function sequence(id, chapitre, garde, premiere) {
    return DG.parcours[id].pas.map(function (p, i) {
      var c = Object.assign({}, p);
      if (!(i in garde)) { c.muet = true; return c; }
      c.d = garde[i]; c.chapitre = chapitre;
      return c;
    }).map(function (p, i, tout) {
      var premier = Object.keys(garde).map(Number).sort(function (a, b) { return a - b; })[0];
      if (i === premier) { p.role = DG.parcours[id].role; Object.assign(p, premiere || {}); }
      return p;
    });
  }

  var pas = [].concat(
    [
      { carte: { sur: 'Aujourd’hui', titre: 'Les loyers, les preuves, les propriétaires : tout est dispersé.', items: ['Un carnet', 'Un tableur', 'Des captures d’écran', 'Des messages WhatsApp'] },
        d: 8, chapitre: 'Accroche', titre: 'Tout est dispersé', texte: 'Carnet, tableur, captures d’écran : chaque preuve est ailleurs.',
        voix: 'Les loyers, les preuves, les propriétaires : tout est dispersé.' },
      { ecran: '#/accueil', role: 'gerant', d: 7, chapitre: 'Accroche', titre: 'Immo Suite', cible: null,
        texte: 'Un seul endroit pour encaisser, quittancer, relancer et rendre compte aux propriétaires.' }
    ],
    sequence('p1', 'Encaisser', { 0: 7, 1: 5, 3: 5, 4: 9, 5: 7, 6: 7 }, { voix: 'Un paiement, une quittance.' }),
    sequence('p2', 'Relancer', { 0: 7, 1: 6, 3: 5, 5: 7, 7: 8, 8: 7 }, { voix: 'Qui doit quoi, et ce qui a été dit.' }),
    sequence('p3', 'Rendre compte', { 0: 6, 3: 9, 5: 6, 7: 6, 9: 7, 10: 6 }, { voix: 'Le compte rendu, en deux clics.' }),
    sequence('p4', 'Migrer', { 0: 5, 2: 6, 4: 5, 5: 4 }, { voix: 'Sans tout ressaisir.' }),
    sequence('p5', 'Mémoire', { 0: 3, 2: 4, 3: 5, 5: 3 }, { voix: 'La mémoire du bien.' }),
    [
      { ecran: '#/accueil', role: 'gerant', d: 10, chapitre: 'Clôture', titre: 'Voici la première version', cible: '.modale',
        texte: 'Ce qui est livré en V1, et ce qui viendra plus tard (« À venir »). Aucun prix, aucune date.',
        voix: 'Voici la première version. Parlons de vos besoins.',
        entree: async function (H) { await H.pause(200); window.UI.legende(); await H.attendre('.modale'); } }
    ]
  );

  DG.enregistrer('film', { titre: 'Film complet', resume: 'Les cinq parcours en 3 minutes', role: 'gerant', pas: pas });
})();
