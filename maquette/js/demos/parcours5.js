/*
 * Démonstration 5 : consulter l'historique d'un lot (45 s). Rôle : gestionnaire. MAQUETTE.md, vidéo 5.
 */
DemoGuidee.enregistrer('p5', {
  titre: 'Consulter l’historique d’un lot', resume: 'Gestionnaire · 45 s', role: 'gestionnaire',
  pas: [
    { ecran: '#/lots', d: 8, cible: '#l-quartier', titre: 'Un lot, un appartement, une boutique',
      texte: 'Les lots se filtrent par quartier, par bien, par statut.',
      voix: 'Un lot, un appartement, une boutique.',
      faire: async function (H) { await H.choisir('#l-quartier', 'Ouaga 2000'); await H.pause(300); } },
    { d: 5, cible: '[data-action="lot-ouvrir"][data-id="L001"]', titre: 'L’Appartement A3',
      texte: 'Ouaga 2000, mandant M1, loyer 90 000 FCFA.',
      faire: async function (H) { await H.cliquer('[data-action="lot-ouvrir"][data-id="L001"]'); await H.attendre('.onglets'); } },
    { d: 8, cible: '.onglets', titre: 'Le bail, le locataire, l’échéance du mois',
      texte: 'Bail actif, locataire Habiba Kiemdé, échéance du mois payée. Les onglets donnent accès aux baux, paiements, travaux et documents.',
      voix: 'Le bail, le locataire, l’échéance du mois.',
      faire: async function (H) { await H.cliquer({ sel: '.onglet', texte: 'Travaux' }); await H.attendre('.chrono-item'); } },
    { d: 12, cible: { sel: '.chrono-item', texte: 'fuite' }, titre: 'Tous les travaux',
      texte: 'Six interventions sur plusieurs années. Celle du 22/10/2026 : 30 000 FCFA, payée par l’agence, imputée au compte rendu d’octobre.',
      voix: 'Tous les travaux, avec le coût et qui a payé.',
      faire: async function (H) { await H.cliquer({ sel: '.chrono-item .chrono-corps', texte: 'fuite' }); await H.attendre('.panneau'); } },
    { d: 8, cible: '.panneau', titre: 'Factures et photos',
      texte: 'Prestataire, coût, qui paie, pièces jointes : tout reste attaché au lot.',
      voix: 'Factures et photos restent avec le lot.',
      faire: async function (H) { await H.cliquer({ sel: '.panneau a, .panneau button', texte: 'Voir dans le compte rendu' }); await H.attendre('.cr-feuille'); } },
    { d: 7, cible: '.cr-feuille', titre: 'La dépense dans le compte rendu',
      texte: 'La réparation apparaît dans le compte rendu d’octobre du propriétaire.',
      voix: 'Et la dépense apparaît dans le compte rendu du propriétaire.' }
  ]
});
