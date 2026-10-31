/*
 * Démonstration 1 : encaisser et envoyer une quittance (45 s). Rôle : caissier. MAQUETTE.md, vidéo 1.
 * Chaque « faire » est l'action visible exécutée en quittant l'étape.
 */
DemoGuidee.enregistrer('p1', {
  titre: 'Encaisser et envoyer une quittance', resume: 'Caissière · 45 s', role: 'caissier',
  pas: [
    { ecran: '#/encaisser', d: 8, cible: '#rech', titre: 'Un locataire arrive pour payer',
      texte: 'La caissière cherche le locataire par son nom, son numéro de téléphone ou son lot.',
      voix: 'Un locataire arrive pour payer son loyer.',
      faire: async function (H) { await H.saisir('#rech', 'Tapsoba'); await H.attendre('#enc-carte .enc-locataire'); } },
    { d: 7, cible: '#enc-carte', titre: 'Ce qu’il doit, tout de suite',
      texte: 'La carte du locataire affiche « Due aujourd’hui » et le solde : 75 000 FCFA.',
      voix: 'Vous voyez tout de suite ce qu’il doit.' },
    { d: 8, cible: ['#enc-form', '#enc-droite'], titre: 'Montant et mode de paiement',
      texte: 'Le montant est proposé, le mode Espèces est présélectionné. L’aperçu montre que l’échéance du 13/11 sera soldée.',
      voix: 'Montant, mode de paiement. L’affectation se fait toute seule.' },
    { d: 5, cible: '.bouton-valider', titre: 'Un clic',
      texte: 'Le paiement est enregistré et la quittance est créée en même temps.',
      voix: 'Un clic.',
      faire: async function (H) { await H.cliquer('.bouton-valider'); await H.attendre('.q-feuille', 8000); } },
    { d: 8, cible: '.q-feuille', titre: 'Une quittance numérotée',
      texte: 'Numéro suivant le dernier émis, montant en chiffres et en lettres, solde de l’échéance.',
      voix: 'La quittance est numérotée, avec le montant en lettres.',
      faire: async function (H) { await H.cliquer({ sel: 'button, a', texte: 'Envoyer sur WhatsApp' }); await H.attendre('.msg'); } },
    { d: 7, cible: '#msg-brouillon', titre: 'Le message est prêt',
      texte: 'Le texte et le lien de la quittance sont prêts. Cet écran simule WhatsApp : rien n’est réellement envoyé.',
      voix: 'Elle part sur le WhatsApp du locataire.',
      faire: async function (H) { await H.cliquer('[data-msg="envoyer"]'); await H.cliquer('[data-msg="fin"]'); await H.disparait('.msg'); } },
    { ecran: '#/paiements', d: 6, cible: '.tab-bureau tbody tr, .tab-mobile li', titre: 'Le paiement est tracé',
      texte: 'Il apparaît en tête du journal des paiements, au statut « Validé ». Un paiement ne se modifie pas : il se corrige par contre-passation.',
      voix: 'Le paiement est tracé.' }
  ]
});
