/*
 * S15 : Agenda (tâches et alertes non financières triées par date : fin de bail à 60 jours, promesses, rappels, visites, révisions).
 * Repère « V1 · livré pendant le pilote ». Les prochaines actions saisies dans « Relancer » (parcours 2) apparaissent ici.
 */
(function () {
  var F = window.Fmt, UI = window.UI, Data = window.Data, App = window.App, Mut = window.Mut, Etat = window.Etat;
  var f = { qui: '' };
  function D() { return Data.org(); }
  var ICONE = { promesse_non_tenue: 'alert-triangle', promesse: 'clock', rappel: 'bell', rappel_visite: 'map-pin', intervention: 'wrench', revision: 'chart', fin_bail: 'calendar', tache: 'check-circle' };
  function lien(a) {
    var o = a.objet || {};
    if (o.kind === 'promesse' || a.type === 'promesse_non_tenue') return { href: '#/impayes', lib: 'Voir les impayés' };
    if (o.kind === 'echeance') return { href: '#/accueil', lib: 'Ouvrir l’accueil' };
    if (o.kind === 'bail' && a.type === 'fin_bail') return { href: '#/bail/' + o.id, lib: 'Ouvrir le bail' };
    if (o.kind === 'bail' && a.type === 'revision') return { href: '#/bail/' + o.id, lib: 'Ouvrir le bail' };
    if (o.kind === 'bail') return { href: '#/locataire/' + Data.bail(o.id).locataireIds[0], lib: 'Ouvrir la fiche' };
    if (o.kind === 'intervention') return { href: '#/lot/' + Data.intervention(o.id).lotId + '?onglet=travaux&intervention=' + o.id, lib: 'Voir l’intervention' };
    if (o.kind === 'releve') return { href: '#/comptes-rendus', lib: 'Comptes rendus' };
    return null;
  }
  function plus7(j) { var d = new Date(j + 'T00:00:00Z'); d.setUTCDate(d.getUTCDate() + 7); return d.toISOString().slice(0, 10); }
  function ligne(a) {
    var l = lien(a), u = Data.utilisateur(a.assigneId), fait = a.fait;
    return '<li class="ag-ligne' + (fait ? ' fait' : '') + '"><label class="ag-case"><input type="checkbox" data-action="ag-fait" data-id="' + a.id + '"' + (fait ? ' checked' : '') + '><span class="sr-only">Marquer « ' + F.e(a.titre) + ' » comme ' + (fait ? 'non faite' : 'faite') + '</span></label>' +
      '<span class="pay-mode" aria-hidden="true">' + UI.ic(ICONE[a.type] || 'calendar') + '</span><div class="liste-gauche" style="flex:1"><span class="liste-titre"' + (fait ? ' style="text-decoration:line-through"' : '') + '>' + F.e(a.titre) + '</span><span class="liste-sous">' + F.e(a.detail) + ' · ' + F.e(u ? u.nom : '—') + '</span>' + (l ? '<a class="alerte-action" href="' + l.href + '">' + l.lib + UI.ic('chevron-right', 'ic-sm') + '</a>' : '') + '</div>' +
      '<div class="liste-droite"><span class="gras nowrap">' + F.date(a.date) + '</span>' + (a.type === 'fin_bail' ? UI.repere({ type: 'v1', pilote: true, mini: true }) : a.type === 'revision' ? UI.neutre('À étudier', 'info') : '') + '</div></li>';
  }
  function groupes() {
    var j = D().meta.aujourdhui, l = D().agenda.filter(function (a) { return !f.qui || a.assigneId === f.qui; }).sort(function (a, b) { return a.date.localeCompare(b.date); });
    var g = [['En retard', l.filter(function (a) { return !a.fait && a.date < j; })], ['Aujourd’hui', l.filter(function (a) { return !a.fait && a.date === j; })], ['Cette semaine', l.filter(function (a) { return !a.fait && a.date > j && a.date <= plus7(j); })], ['Plus tard', l.filter(function (a) { return !a.fait && a.date > plus7(j); })]];
    var h = g.filter(function (x) { return x[1].length; }).map(function (x) { return '<section class="carte" style="margin-bottom:16px" aria-label="' + x[0] + '"><div class="carte-entete"><h2 class="carte-titre">' + x[0] + '</h2><span class="badge badge-neutre">' + x[1].length + '</span></div><ul class="liste ag-liste">' + x[1].map(ligne).join('') + '</ul></section>'; }).join('');
    var faites = l.filter(function (a) { return a.fait; });
    if (!h && !faites.length) return '<section class="carte">' + UI.etatVide({ ic: 'calendar', titre: 'Rien de prévu', texte: 'Les alertes de fin de bail apparaîtront ici.', action: UI.btn('Ajouter une tâche', { v: 'primaire', action: 'ag-ajouter' }) }) + '</section>';
    return h + (faites.length ? '<details class="repliable"><summary>' + UI.ic('check-circle') + 'Faites (' + faites.length + ')' + UI.ic('chevron-down') + '</summary><div class="repliable-corps"><ul class="liste ag-liste">' + faites.map(ligne).join('') + '</ul></div></details>' : '');
  }
  function maj() { document.getElementById('ag-liste').innerHTML = groupes(); }
  App.action('ag-fait', function (el) { Mut.appliquer('agenda.fait', { org: Etat.orgCle(), id: el.getAttribute('data-id'), fait: el.checked }); maj(); UI.toast(el.checked ? 'Tâche marquée comme faite.' : 'Tâche rouverte.', { type: 'succes' }); });
  App.action('ag-ajouter', function () {
    var users = D().utilisateurs.filter(function (u) { return u.statut === 'actif'; });
    UI.modale({ titre: 'Ajouter une tâche', corps: '<div class="champ"><label class="champ-etiquette" for="ag-titre">Tâche (obligatoire)</label><input class="saisie" id="ag-titre" autocomplete="off" placeholder="Exemple : visite du lot Q1"><div class="champ-erreur cache" id="ag-titre-err" role="alert"></div></div><div class="champ"><label class="champ-etiquette" for="ag-date">Date (JJ/MM/AAAA)</label><input class="saisie" id="ag-date" value="' + F.date(plus7(D().meta.aujourdhui)) + '" inputmode="numeric"><div class="champ-erreur cache" id="ag-date-err" role="alert"></div></div><div class="champ"><label class="champ-etiquette" for="ag-qui">Assignée à</label><select class="saisie" id="ag-qui">' + users.map(function (u) { return '<option value="' + u.id + '">' + F.e(u.nom) + '</option>'; }).join('') + '</select></div>',
      actions: [{ lib: 'Annuler', v: 'secondaire' }, { lib: 'Ajouter la tâche', v: 'primaire', focus: true, fn: function (fermer) {
        var t = document.getElementById('ag-titre').value.trim(), m = /^(\d{2})\/(\d{2})\/(\d{4})$/.exec(document.getElementById('ag-date').value.trim()), ok = true, iso = m ? m[3] + '-' + m[2] + '-' + m[1] : null;
        var e1 = document.getElementById('ag-titre-err'), e2 = document.getElementById('ag-date-err'); e1.classList.toggle('cache', !!t); e1.textContent = t ? '' : 'Indiquez la tâche.'; var dv = iso && !isNaN(new Date(iso + 'T00:00:00Z')); e2.classList.toggle('cache', !!dv); e2.textContent = dv ? '' : 'Saisissez une date au format JJ/MM/AAAA.';
        if (!t || !dv) return; var qui = document.getElementById('ag-qui').value; fermer(); Mut.appliquer('agenda.ajouter', { org: Etat.orgCle(), titre: t, date: iso, assigneId: qui }); maj(); UI.toast('Tâche ajoutée à l’agenda.', { type: 'succes' });
      } }] });
    document.getElementById('ag-titre').focus();
  });
  App.enregistrer('agenda', {
    meta: function () { return { titre: 'Agenda', sous: 'Ne rater aucune échéance non financière.', repere: { type: 'v1', pilote: true }, actions: UI.btn('Ajouter une tâche', { v: 'primaire', ic: 'plus', action: 'ag-ajouter' }) }; },
    rendre: function () {
      f = { qui: '' };
      var users = D().utilisateurs.filter(function (u) { return u.statut === 'actif'; });
      return '<div class="filtres" style="margin-bottom:16px"><div class="champ" style="min-width:220px"><label class="champ-etiquette" for="ag-f">Assignée à</label><select class="saisie" id="ag-f"><option value="">Toute l’équipe</option>' + users.map(function (u) { return '<option value="' + u.id + '">' + F.e(u.nom) + '</option>'; }).join('') + '</select></div></div><div id="ag-liste">' + groupes() + '</div>';
    },
    apres: function (ctx, rac) { rac.querySelector('#ag-f').addEventListener('change', function (e) { f.qui = e.target.value; maj(); }); }
  });
})();
