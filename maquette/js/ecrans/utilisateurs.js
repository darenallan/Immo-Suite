/*
 * S18 : Utilisateurs et invitations (console d'invitation).
 * Liste (actif, invité, expiré, désactivé), invitation par téléphone, changement de rôle (effet immédiat), désactivation.
 * Garde-fous montrés : identifiant déjà utilisé, invitation expirée (7 jours), dernier gérant non retirable (R-081).
 */
(function () {
  var F = window.Fmt, UI = window.UI, Data = window.Data, App = window.App, Mut = window.Mut, Etat = window.Etat;
  function D() { return Data.org(); }
  var ROLE = { gerant: 'Gérant', gestionnaire: 'Gestionnaire', caissier: 'Caissier', agent_terrain: 'Agent terrain', comptable: 'Comptable (lecture)', proprietaire: 'Propriétaire', collaborateur: 'Collaborateur' };
  var STAT = { actif: ['Actif', 'check', ''], invitation_en_attente: ['Invité', 'clock', 'badge-pointille'], invitation_expiree: ['Invitation expirée', 'alert-triangle', 'badge-pointille'], desactive: ['Désactivé', 'ban', 'badge-void'] };
  function rolesPossibles() { return D().organisation.plan === 'agence' ? ['gerant', 'gestionnaire', 'caissier', 'agent_terrain', 'comptable'] : ['proprietaire', 'collaborateur']; }
  function badge(s) { var m = STAT[s]; return UI.neutre(m[0], m[1], m[2]); }
  function liste() {
    var us = D().utilisateurs, ctx = App.ctx;
    var lignes = us.map(function (u) {
      var acts = u.statut === 'invitation_expiree' ? UI.btn('Renvoyer l’invitation', { v: 'secondaire', petit: true, action: 'us-renvoyer', data: ' data-id="' + u.id + '"' }) : u.statut === 'actif' ? UI.btn('Changer le rôle', { v: 'tertiaire', petit: true, action: 'us-role', data: ' data-id="' + u.id + '"' }) + UI.btn('Désactiver', { v: 'tertiaire', petit: true, action: 'us-desactiver', data: ' data-id="' + u.id + '"' }) : '';
      var moi = u.id === ctx.utilisateur.id ? ' <span class="badge badge-brand">Vous</span>' : '';
      return '<li class="us-ligne"><span class="avatar" aria-hidden="true">' + F.initiales(u.nom) + '</span><div class="liste-gauche" style="flex:1"><span class="liste-titre">' + F.e(u.nom) + moi + '</span><span class="liste-sous">' + F.e(u.telephone) + ' · ' + ROLE[u.role] + (u.statut === 'invitation_expiree' ? ' · valable 7 jours' : '') + '</span></div><div class="liste-droite" style="flex-direction:row;flex-wrap:wrap;justify-content:flex-end;align-items:center;gap:8px">' + badge(u.statut) + acts + '</div></li>';
    }).join('');
    return '<ul class="liste us-liste">' + lignes + '</ul>';
  }
  function err(id, t) { var e = document.getElementById(id); e.classList.toggle('cache', !t); e.innerHTML = t ? UI.ic('alert-circle', 'ic-sm') + '<span>' + t + '</span>' : ''; return !t; }
  function maj() { document.getElementById('us-zone').innerHTML = liste(); }

  App.action('us-inviter', function () {
    var roles = rolesPossibles().filter(function (r) { return r !== 'gerant' && r !== 'proprietaire'; });
    UI.modale({ titre: 'Inviter un collaborateur', corps: '<div class="champ"><label class="champ-etiquette" for="iu-nom">Nom (obligatoire)</label><input class="saisie" id="iu-nom" autocomplete="off"><div class="champ-erreur cache" id="iu-nom-err" role="alert"></div></div><div class="champ"><label class="champ-etiquette" for="iu-tel">Téléphone (obligatoire)</label><div class="saisie-montant" style="min-height:48px"><span class="suffixe" style="padding:0 4px 0 14px;color:var(--color-text)">+226</span><input id="iu-tel" inputmode="numeric" style="font-size:var(--text-base);text-align:left;font-weight:400" placeholder="70 12 34 56" autocomplete="off"></div><div class="champ-erreur cache" id="iu-tel-err" role="alert"></div></div><div class="champ"><label class="champ-etiquette" for="iu-role">Rôle</label><select class="saisie" id="iu-role">' + roles.map(function (r) { return '<option value="' + r + '">' + ROLE[r] + '</option>'; }).join('') + '</select></div><div class="champ cache" id="iu-lots"><span class="champ-etiquette">Lots assignés</span>' + D().lots.filter(function (l) { return l.statut === 'occupe'; }).slice(0, 8).map(function (l) { return '<label style="display:flex;gap:10px;align-items:center;min-height:36px"><input type="checkbox"><span>' + F.e(l.nom) + '</span></label>'; }).join('') + '</div><p class="champ-aide" style="margin-top:12px">L’invitation est valable 7 jours.</p>',
      actions: [{ lib: 'Annuler', v: 'secondaire' }, { lib: 'Envoyer l’invitation', v: 'primaire', focus: true, fn: function (fermer) {
        var nom = document.getElementById('iu-nom').value.trim(), tel = document.getElementById('iu-tel').value.replace(/\D/g, '');
        var ok = err('iu-nom-err', nom ? '' : 'Indiquez le nom de la personne invitée.'); err('iu-tel-err', '');
        if (tel.length !== 8) ok = err('iu-tel-err', 'Numéro invalide. Format attendu : +226 suivi du numéro (exemple : +226 70 12 34 56).') && ok;
        else if (D().utilisateurs.some(function (u) { return u.telephone.replace(/\D/g, '').slice(-8) === tel; })) ok = err('iu-tel-err', 'Cet identifiant a déjà un compte. Voulez-vous vous connecter ? <a href="#/connexion">Se connecter</a>') && ok;
        if (!ok) return; var role = document.getElementById('iu-role').value, tl = '+226 ' + tel.replace(/(\d{2})(?=\d)/g, '$1 ').trim();
        fermer(); Mut.appliquer('utilisateur.inviter', { org: Etat.orgCle(), nom: nom, role: role, telephone: tl }); maj(); UI.toast('Invitation envoyée à ' + F.e(nom) + ' (valable 7 jours).', { type: 'succes' });
      } }] });
    var sel = document.getElementById('iu-role'); sel.addEventListener('change', function () { document.getElementById('iu-lots').classList.toggle('cache', sel.value !== 'agent_terrain'); });
    document.getElementById('iu-nom').focus();
  });
  App.action('us-renvoyer', function (el) { Mut.appliquer('utilisateur.renvoyer', { org: Etat.orgCle(), id: el.getAttribute('data-id') }); maj(); UI.toast('Invitation renvoyée (valable 7 jours).', { type: 'succes' }); });
  App.action('us-role', function (el) {
    var u = Data.utilisateur(el.getAttribute('data-id'));
    UI.modale({ titre: 'Changer le rôle de ' + F.e(u.nom), corps: '<p class="secondaire">Le changement est immédiat.</p><div class="champ" style="margin-top:12px"><label class="champ-etiquette" for="cr-role">Nouveau rôle</label><select class="saisie" id="cr-role">' + rolesPossibles().map(function (r) { return '<option value="' + r + '"' + (r === u.role ? ' selected' : '') + '>' + ROLE[r] + '</option>'; }).join('') + '</select><div class="champ-erreur cache" id="cr-err" role="alert"></div></div>',
      actions: [{ lib: 'Annuler', v: 'secondaire' }, { lib: 'Changer le rôle', v: 'primaire', focus: true, fn: function (fermer) {
        var r = document.getElementById('cr-role').value, gerants = D().utilisateurs.filter(function (x) { return (x.role === 'gerant' || x.role === 'proprietaire') && x.statut === 'actif'; });
        if ((u.role === 'gerant' || u.role === 'proprietaire') && r !== u.role && gerants.length <= 1) { err('cr-err', 'Vous ne pouvez pas retirer le dernier gérant. Nommez d’abord un autre gérant.'); return; }
        fermer(); Mut.appliquer('utilisateur.role', { org: Etat.orgCle(), id: u.id, role: r }); maj(); UI.toast(F.e(u.nom) + ' est maintenant ' + ROLE[r].toLowerCase() + '.', { type: 'succes' });
      } }] });
  });
  App.action('us-desactiver', function (el) {
    var u = Data.utilisateur(el.getAttribute('data-id')), gerants = D().utilisateurs.filter(function (x) { return (x.role === 'gerant' || x.role === 'proprietaire') && x.statut === 'actif'; });
    if ((u.role === 'gerant' || u.role === 'proprietaire') && gerants.length <= 1) { UI.toast('Vous ne pouvez pas retirer le dernier gérant. Nommez d’abord un autre gérant.', { type: 'erreur' }); return; }
    UI.modale({ titre: 'Désactiver ' + F.e(u.nom) + ' ?', corps: '<p>Cette personne ne pourra plus se connecter.</p>', actions: [{ lib: 'Annuler', v: 'secondaire', focus: true }, { lib: 'Désactiver', v: 'primaire', fn: function (fermer) { fermer(); Mut.appliquer('utilisateur.desactiver', { org: Etat.orgCle(), id: u.id }); maj(); UI.toast(F.e(u.nom) + ' est désactivé.', { type: 'info' }); } }] });
  });

  App.enregistrer('utilisateurs', {
    meta: function () { return { titre: 'Utilisateurs', sous: 'Donnez l’accès à votre équipe, au bon niveau.', repere: { type: 'v1' }, actions: UI.btn('Inviter un collaborateur', { v: 'primaire', ic: 'user-plus', action: 'us-inviter' }) }; },
    rendre: function (ctx) {
      var agence = D().organisation.plan === 'agence';
      var info = agence ? '' : '<div class="bandeau bandeau-info" style="margin-bottom:16px">' + UI.ic('info') + '<span>Plan Propriétaire : deux profils, « propriétaire » (tous les droits) et « collaborateur » (enregistrer les paiements et relancer).</span></div>';
      return info + '<section class="carte" aria-labelledby="t-us"><h2 id="t-us" class="sr-only">Équipe</h2><div id="us-zone">' + liste() + '</div></section><p class="muted" style="margin-top:12px;font-size:var(--text-sm)">Les droits réglables se trouvent dans <a href="#/parametres">Paramètres</a>. Le dernier gérant actif ne peut pas être retiré.</p>';
    }
  });
})();
