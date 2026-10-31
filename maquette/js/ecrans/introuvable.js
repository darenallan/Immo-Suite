/*
 * Page « Introuvable » : adresse inconnue ou ressource d'une autre organisation (R-004).
 * Aucune entrée de navigation ne mène ici : elle ne sert que pour une adresse saisie à la main.
 */
(function () {
  var UI = window.UI;
  window.App.enregistrer('introuvable', {
    meta: function () { return { titre: 'Introuvable', repere: { type: 'v1' } }; },
    rendre: function (ctx) {
      return '<div class="carte reserve">' + UI.etatVide({ ic: 'search', titre: 'Introuvable', texte: 'Cette page n’existe pas ou vous n’y avez pas accès.', action: UI.btn('Retour à l’accueil', { v: 'primaire', href: '#' + window.Roles.def(ctx.role.id).arrivee }) }) + '</div>';
    }
  });
})();
