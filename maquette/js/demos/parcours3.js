/*
 * Démonstration 3 : produire un compte rendu propriétaire (75 s). Rôle : gérant. MAQUETTE.md, vidéo 3.
 */
DemoGuidee.enregistrer('p3', {
  titre: 'Produire un compte rendu propriétaire', resume: 'Gérante · 75 s', role: 'gerant',
  pas: [
    { ecran: '#/accueil', d: 7, cible: { sel: '#ecran li, #ecran a, #ecran p', texte: 'Comptes rendus d’octobre à produire' }, titre: 'Fin de mois',
      texte: 'Le tableau de bord signale les comptes rendus d’octobre à produire.',
      voix: 'Fin de mois. Vos mandants attendent leur compte rendu.',
      faire: async function (H) { await H.cliquer({ sel: '#ecran a, #ecran button', texte: 'Produire un compte rendu' }); await H.attendre('.bandeau-avert'); } },
    { d: 7, cible: '.page-actions', titre: 'La liste des comptes rendus',
      texte: 'Trois comptes rendus d’octobre sont à produire. Un clic sur « Produire un compte rendu ».',
      faire: async function (H) { await H.cliquer({ sel: '.page-actions a, .page-actions button', texte: 'Produire un compte rendu' }); await H.attendre('#btn-previsualiser'); } },
    { d: 7, cible: '#nv-mandant', titre: 'Le mandant et le mois',
      texte: 'Mme Georgette Ouédraogo (M1), octobre 2026.',
      voix: 'Choisissez le mandant et le mois.',
      faire: async function (H) { await H.cliquer('#btn-previsualiser'); await H.attendre('.cr-feuille', 6000); } },
    { d: 12, cible: '.cr-feuille', titre: 'Le net à reverser est calculé',
      texte: 'Six loyers encaissés : 400 000 FCFA. Commission de 8 % : 32 000. Travaux imputés : 30 000. Net à reverser : 338 000 FCFA. Un impayé de 45 000 FCFA est signalé.',
      voix: 'Loyers encaissés, commission, travaux : le net à reverser est calculé.',
      faire: async function (H) { await H.cliquer('[data-action="cr-detail"]'); await H.attendre('.cr-depense-detail'); } },
    { d: 8, cible: '.cr-depense-detail', titre: 'Chaque ligne se justifie',
      texte: 'La réparation de fuite : prestataire, coût, pièces, et un lien vers l’historique du lot.',
      voix: 'Chaque ligne se justifie.',
      faire: async function (H) { await H.cliquer({ sel: 'button, a', texte: 'Valider le compte rendu' }); await H.attendre('.modale'); } },
    { d: 8, cible: '.modale', titre: 'Valider',
      texte: 'Après validation, le compte rendu est figé. Il reste réouvrable tant que le reversement n’est pas enregistré.',
      voix: 'Vous validez : le compte rendu est figé.',
      faire: async function (H) { await H.cliquer('.modale-actions button:last-child'); await H.attendre({ sel: 'button, a', texte: 'Exporter en PDF' }); } },
    { d: 6, cible: { sel: 'button, a', texte: 'Exporter en PDF' }, titre: 'PDF et partage',
      texte: 'Le statut passe à « Validé ». Le PDF et le lien à partager sont disponibles.',
      faire: async function (H) { await H.cliquer({ sel: 'button, a', texte: 'Exporter en PDF' }); await H.attendre('.modale-large'); } },
    { d: 7, cible: '.modale-large', titre: 'Un PDF à partager',
      texte: 'L’aperçu du compte rendu, prêt à imprimer ou à enregistrer en PDF. Le lien ouvre la page que verrait le mandant.',
      voix: 'Un PDF à partager en un clic.',
      faire: async function (H) {
        H.fermerTout(); await H.pause(200);
        await H.cliquer({ sel: 'button, a', texte: 'Partager le lien' }); await H.attendre('.modale');
        await H.cliquer('.modale-actions button:last-child'); await H.attendre('.cr-feuille');
      } },
    { d: 6, cible: '.cr-feuille', titre: 'Ce que voit le mandant',
      texte: 'Le document seul, sans menu ni connexion. Il reste lisible tant que le lien est valide.',
      faire: async function (H) { await H.aller('#/comptes-rendus/R-M1-2026-10'); await H.cliquer({ sel: 'button, a', texte: 'Enregistrer le reversement' }); await H.attendre('#rv-ref'); } },
    { d: 9, cible: '.modale', titre: 'Le reversement',
      texte: 'Date, mode, référence du virement. L’action est définitive : le compte rendu passe à « Payé ».',
      voix: 'Et le reversement est enregistré avec sa référence.',
      faire: async function (H) {
        await H.saisir('#rv-date', '13/11/2026'); await H.saisir('#rv-ref', 'VIR-261113-0001');
        await H.cliquer('.modale-actions button:last-child'); await H.disparait('.modale');
      } },
    { d: 6, cible: '.cr-feuille', titre: 'Compte rendu payé',
      texte: 'Le document mentionne le reversement et sa référence. Le bouton « Rouvrir » a disparu.',
      voix: 'Le compte rendu du propriétaire, en deux clics.' }
  ]
});
