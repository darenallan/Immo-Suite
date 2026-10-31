/*
 * Démonstration 2 : voir et relancer les impayés (60 s). Rôle : gestionnaire. MAQUETTE.md, vidéo 2.
 */
DemoGuidee.enregistrer('p2', {
  titre: 'Voir et relancer les impayés', resume: 'Gestionnaire · 60 s', role: 'gestionnaire',
  pas: [
    { ecran: '#/accueil', d: 9, cible: '.grille-faire', titre: 'Par où commencer ce matin',
      texte: 'Promesses non tenues, rappels à envoyer avant l’échéance, échéances du jour : tout est regroupé dans « À faire aujourd’hui ».',
      voix: 'Chaque matin, vous savez par où commencer.',
      faire: async function (H) { await H.cliquer({ sel: 'a, button', texte: 'Voir les impayés' }); await H.attendre('.tranches'); } },
    { d: 8, cible: ['#t-res', '.tranches'], titre: 'Qui doit combien, depuis quand',
      texte: 'Le total en retard est réparti par ancienneté : 1 à 7 jours, 8 à 30, 31 à 59, 60 et plus.',
      voix: 'Qui doit combien, depuis combien de jours.' },
    { d: 7, cible: '#f-mandant', titre: 'Filtrer',
      texte: 'Les filtres : mandant, bien, gestionnaire, usage. Ici, le mandant M3.',
      voix: 'Filtrez par mandant, par bien, par gestionnaire.',
      faire: async function (H) { await H.choisir('#f-mandant', 'M3'); await H.pause(300); } },
    { d: 8, cible: '[data-action="imp-relancer"][data-bail="B21"]', titre: 'Relancer',
      texte: 'Pauline Yaméogo doit 105 000 FCFA, dont une échéance de 70 jours.',
      voix: 'Choisissez le ton : courtois, ferme, dernier avis.',
      faire: async function (H) { await H.cliquer('[data-action="imp-relancer"][data-bail="B21"]'); await H.attendre('.panneau'); } },
    { d: 7, cible: '.panneau', titre: 'Un modèle selon l’ancienneté',
      texte: 'Trois tons sont proposés. Le dernier avis est présélectionné, car la dette a plus de 10 jours. Le texte est prêt avec le nom, le montant et le mois.',
      faire: async function (H) { await H.cliquer('#btn-rel-ouvrir'); await H.attendre('.msg'); } },
    { d: 7, cible: '#msg-brouillon', titre: 'Le message est prêt',
      texte: 'Cet écran simule WhatsApp. Vous restez maître de l’envoi : Immo Suite ne l’envoie pas à votre place.',
      voix: 'Le message est prêt. Vous restez maître de l’envoi.',
      faire: async function (H) { await H.cliquer('[data-msg="envoyer"]'); await H.cliquer('[data-msg="fin"]'); await H.attendre('#btn-rel-oui'); } },
    { d: 5, cible: '#btn-rel-oui', titre: 'Avez-vous envoyé le message ?',
      texte: 'Au retour dans Immo Suite, la confirmation évite de noter une relance qui n’est pas partie.',
      faire: async function (H) { await H.cliquer('#btn-rel-oui'); await H.attendre('#rel-promesse'); } },
    { d: 8, cible: '.panneau-corps', titre: 'La promesse de paiement',
      texte: 'Le locataire promet 35 000 FCFA pour le 20/11/2026. La prochaine action est planifiée.',
      voix: 'Vous notez la promesse. Immo Suite vous le rappellera.',
      faire: async function (H) { await H.cliquer('#rel-promesse'); await H.pause(400); await H.cliquer('#btn-rel-enreg'); await H.disparait('.panneau'); } },
    { d: 6, cible: '[data-ligne="B21"]', titre: 'La ligne est à jour',
      texte: 'La relance du jour, la promesse et la prochaine action s’affichent. La ligne reste dans la liste tant que la dette existe.',
      voix: 'Plus aucune relance oubliée ni perdue.' }
  ]
});
