/*
 * Démonstration 4 : importer un fichier Excel (60 s). Rôle : gestionnaire. MAQUETTE.md, vidéo 4.
 * L'import de démonstration n'ajoute aucune donnée au portefeuille.
 */
DemoGuidee.enregistrer('p4', {
  titre: 'Importer un fichier Excel', resume: 'Gestionnaire · 60 s', role: 'gestionnaire',
  pas: [
    { ecran: '#/import', d: 9, cible: '#depot', titre: 'Votre portefeuille est déjà dans un fichier',
      texte: 'Un modèle Excel est téléchargeable. Pour la démonstration, on choisit le fichier d’exemple « portefeuille-agence.xlsx ».',
      voix: 'Vous avez déjà votre portefeuille dans un fichier Excel.',
      faire: async function (H) { await H.cliquer('#btn-exemple'); await H.attendre('#t-e2', 6000); } },
    { d: 9, cible: '#imp-etape', titre: 'Les colonnes sont reconnues',
      texte: 'Sept colonnes sur huit sont associées automatiquement, la huitième est à vérifier.',
      voix: 'Immo Suite reconnaît vos colonnes.',
      faire: async function (H) { await H.cliquer('#btn-vers3'); await H.attendre('#t-err'); } },
    { d: 11, cible: '#imp-etape', titre: '197 lignes valides, 3 erreurs',
      texte: 'Les erreurs sont listées avec leur numéro de ligne : 14, 87 et 152.',
      voix: 'Les erreurs sont listées ligne par ligne.' },
    { d: 10, cible: '.doublon', titre: 'Les doublons',
      texte: 'Deux numéros de téléphone existent déjà : la fusion est proposée. Vous choisissez de continuer sans les lignes en erreur.',
      voix: 'Les doublons sont proposés à la fusion.',
      faire: async function (H) {
        await H.cliquer('label[for="er-continuer"]'); await H.cliquer('label[for="dbl1g"]');
        await H.cliquer('#btn-vers4'); await H.attendre('#btn-confirmer');
      } },
    { d: 10, cible: '#btn-confirmer', titre: 'Confirmer',
      texte: 'Rien n’est créé avant votre confirmation.',
      voix: 'Rien n’est créé tant que vous n’avez pas confirmé.',
      faire: async function (H) { await H.cliquer('#btn-confirmer'); await H.attendre('.modale'); await H.cliquer('.modale-actions button:last-child'); await H.attendre('#t-fin', 6000); } },
    { d: 8, cible: '#imp-etape', titre: 'Le rapport',
      texte: 'Import terminé : 197 lignes créées. Le rapport se télécharge et l’import figure dans l’historique, annulable.',
      voix: 'Et un rapport complet reste disponible.' }
  ]
});
