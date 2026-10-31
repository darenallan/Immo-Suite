/*
 * S17 : Paramètres (gérant et propriétaire direct).
 * Organisation et en-tête de quittance, modèles de messages (J−3, J+1, J+5, J+10), droits réglables, plan, sécurité, exports, accès support.
 * Éléments : exports « V1 · livré pendant le pilote » ; « Abonnement en ligne » (F31) « À venir » et désactivé.
 * Les droits réglables et l'accès support modifient vraiment la démonstration (journal rejoué) ; le reste affiche « enregistré (démonstration) ».
 */
(function () {
  var F = window.Fmt, UI = window.UI, Data = window.Data, App = window.App, Mut = window.Mut, Etat = window.Etat;
  function D() { return Data.org(); }
  function plus(j, n) { var d = new Date(j + 'T00:00:00Z'); d.setUTCDate(d.getUTCDate() + n); return d.toISOString().slice(0, 10); }
  function interrupteur(cle, lib, aide, val, action) {
    return '<div class="reglage"><div class="liste-gauche"><span class="gras" id="lb-' + cle + '">' + lib + '</span><span class="liste-sous">' + aide + '</span></div><button type="button" role="switch" class="interrupteur" aria-checked="' + !!val + '" aria-labelledby="lb-' + cle + '" data-action="' + action + '" data-cle="' + cle + '"><span class="interrupteur-etat">' + (val ? 'Oui' : 'Non') + '</span><span class="interrupteur-poignee" aria-hidden="true"></span></button></div>';
  }
  function carte(id, ic, titre, corps, repere) { return '<section class="carte" id="' + id + '" aria-labelledby="t-' + id + '"><div class="carte-entete"><h2 id="t-' + id + '" class="carte-titre">' + UI.ic(ic) + titre + '</h2>' + (repere || '') + '</div>' + corps + '</section>'; }

  function sectionOrg(d, p) {
    var o = p.organisation;
    var ch = function (id, lib, val, aide) { return '<div class="champ"><label class="champ-etiquette" for="' + id + '">' + lib + '</label><input class="saisie" id="' + id + '" value="' + F.e(val) + '" autocomplete="off">' + (aide ? '<div class="champ-aide">' + aide + '</div>' : '') + '<div class="champ-erreur cache" id="' + id + '-err" role="alert"></div></div>'; };
    return carte('s-org', 'building', 'Organisation et quittance', '<div class="param-grille">' + ch('po-nom', 'Nom de l’organisation', o.nom) + ch('po-ville', 'Ville', o.ville) + ch('po-adresse', 'Adresse', o.adresse) + ch('po-tel', 'Téléphone', o.telephone) + ch('po-entete', 'En-tête de la quittance', o.enteteQuittance) +
      ch('po-prefixe', 'Préfixe de numérotation', o.prefixeQuittance, 'Les quittances s’écrivent ' + F.e(o.prefixeQuittance) + '-2026-' + String(d.meta.prochaineSequence).padStart(6, '0') + '. La séquence repart à 1 chaque année.') + ch('po-liens', 'Expiration des liens de partage (jours)', String(p.expirationLiensJours), 'Valeur indicative, à fixer.') + ch('po-id', 'Identifiants légaux (facultatif)', '') +
      '<div class="champ"><label class="champ-etiquette" for="po-logo">Logo (facultatif)</label><input class="saisie" type="file" id="po-logo" accept=".png,.jpg,.svg"><div class="champ-aide">40 × 20 mm au plus sur la quittance.</div></div><div class="champ"><label class="champ-etiquette" for="po-sign">Signature ou cachet (image)</label><input class="saisie" type="file" id="po-sign" accept=".png,.jpg"><div class="champ-aide">' + F.e(o.signature) + '</div></div></div><div style="margin-top:16px">' + UI.btn('Enregistrer', { v: 'primaire', ic: 'check', action: 'par-enreg' }) + '</div>');
  }
  function sectionModeles(d) {
    var ms = d.modelesRelance;
    return carte('s-mod', 'message', 'Modèles de messages', '<p class="secondaire" style="margin-bottom:12px">Un modèle par étape de relance. Variables : <code>{nom}</code>, <code>{montant}</code>, <code>{mois}</code>, <code>{lot}</code>, <code>{date}</code>, <code>{lien_quittance}</code>. Une variable inconnue est refusée.</p>' +
      ms.map(function (m, i) { return '<div class="champ"><label class="champ-etiquette" for="pm-' + i + '">' + F.e(m.libelle) + '</label><textarea class="saisie" id="pm-' + i + '" rows="5">' + F.e(m.texte) + '</textarea><div class="btn-groupe" style="margin-top:6px">' + UI.btn('Tester ce modèle', { v: 'tertiaire', petit: true, ic: 'play', action: 'par-tester', data: ' data-i="' + i + '"' }) + '</div></div>'; }).join('') + '<div style="margin-top:12px">' + UI.btn('Enregistrer les modèles', { v: 'primaire', ic: 'check', action: 'par-enreg' }) + '</div>');
  }
  function sectionDroits(d, p) {
    var agence = d.organisation.plan === 'agence', dr = p.droits;
    var h = agence ? interrupteur('gestionnaireEncaisse', 'Le gestionnaire peut encaisser', 'Sinon « Encaisser » est remplacé par « Locataires » dans sa barre du bas.', dr.gestionnaireEncaisse, 'par-droit') + interrupteur('gestionnaireImport', 'Le gestionnaire peut importer', 'Le droit d’importer est donné au cas par cas.', dr.gestionnaireImport, 'par-droit') + interrupteur('gestionnaireExport', 'Le gestionnaire peut exporter', 'Exports CSV, Excel et PDF.', dr.gestionnaireExport, 'par-droit')
      : interrupteur('collaborateurEncaisse', 'Le collaborateur peut encaisser', 'Profil « collaborateur » : enregistrer les paiements et relancer.', dr.collaborateurEncaisse, 'par-droit');
    return carte('s-droits', 'users', 'Droits réglables', h + '<p class="muted" style="font-size:var(--text-sm);margin-top:8px">Les droits sont vérifiés côté serveur. Gérez l’équipe dans <a href="#/utilisateurs">Utilisateurs</a>.</p>');
  }
  function sectionSupport(d, p) {
    var a = p.accesSupport || { actif: false, jusqu: null, historique: [] };
    var hist = a.historique.length ? '<h3 style="margin:16px 0 8px">Historique des accès</h3><ul class="liste">' + a.historique.map(function (x) { return '<li><span>' + F.e(x.action) + '</span><span class="muted nowrap">' + F.date(x.date) + '</span></li>'; }).join('') + '</ul>' : '<p class="muted" style="font-size:var(--text-sm);margin-top:12px">Aucun accès pour le moment.</p>';
    return carte('s-sup', 'shield', 'Accès support', '<p class="secondaire" style="margin-bottom:12px">Vous pouvez autoriser l’équipe HABEK à consulter votre dossier pour vous dépanner, pour une durée choisie. Chaque consultation est tracée dans le journal d’audit. L’accès expire tout seul et vous pouvez le révoquer à tout moment.</p>' +
      (a.actif ? '<div class="bandeau bandeau-succes">' + UI.ic('check-circle') + '<span>Accès autorisé jusqu’au ' + F.date(a.jusqu) + '.</span></div><div style="margin-top:12px">' + UI.btn('Révoquer l’accès', { v: 'secondaire', ic: 'lock', action: 'par-revoquer' }) + '</div>' : '<div class="champ"><label class="champ-etiquette" for="ps-duree">Durée de l’autorisation</label><select class="saisie" id="ps-duree" style="max-width:280px"><option value="1">24 heures</option><option value="7" selected>7 jours</option><option value="30">30 jours</option></select></div><div style="margin-top:12px">' + UI.btn('Autoriser l’accès support', { v: 'primaire', ic: 'shield', action: 'par-autoriser' }) + '</div>') + hist);
  }

  App.action('par-droit', function (el) {
    var cle = el.getAttribute('data-cle'), v = el.getAttribute('aria-checked') !== 'true';
    Mut.appliquer('parametres.droit', { org: Etat.orgCle(), cle: cle, valeur: v }); el.setAttribute('aria-checked', String(v)); el.querySelector('.interrupteur-etat').textContent = v ? 'Oui' : 'Non';
    UI.toast('Droit mis à jour : effet immédiat.', { type: 'succes' });
  });
  App.action('par-securite', function (el) { var v = el.getAttribute('aria-checked') !== 'true'; el.setAttribute('aria-checked', String(v)); el.querySelector('.interrupteur-etat').textContent = v ? 'Oui' : 'Non'; UI.toast(v ? 'Double authentification activée (démonstration).' : 'Double authentification désactivée (démonstration).', { type: 'info' }); });
  App.action('par-autoriser', function () {
    var n = parseInt(document.getElementById('ps-duree').value, 10), jusqu = plus(D().meta.aujourdhui, n);
    UI.modale({ titre: 'Autoriser l’accès support ?', corps: '<p>L’équipe HABEK pourra consulter votre dossier jusqu’au ' + F.date(jusqu) + '. Vous pouvez révoquer l’accès à tout moment.</p>', actions: [{ lib: 'Annuler', v: 'secondaire', focus: true }, { lib: 'Autoriser', v: 'primaire', fn: function (fermer) { fermer(); Mut.appliquer('support.autoriser', { org: Etat.orgCle(), jusqu: jusqu, par: App.ctx.utilisateur.id }); App.rendre(); UI.toast('Accès support autorisé jusqu’au ' + F.date(jusqu) + '.', { type: 'succes' }); } }] });
  });
  App.action('par-revoquer', function () { Mut.appliquer('support.revoquer', { org: Etat.orgCle(), par: App.ctx.utilisateur.id }); App.rendre(); UI.toast('Accès support révoqué.', { type: 'info' }); });
  App.action('par-tester', function (el) {
    var i = +el.getAttribute('data-i'), txt = document.getElementById('pm-' + i).value, ok = { nom: 'Awa', montant: F.montant(75000), mois: 'novembre 2026', lot: 'la Maison H1, Gounghin', date: '10/11/2026', lien_quittance: 'https://immo-suite.demo/q/…' };
    var inconnues = []; (txt.match(/\{(\w+)\}/g) || []).forEach(function (m) { if (!(m.slice(1, -1) in ok) && inconnues.indexOf(m) < 0) inconnues.push(m); });
    if (inconnues.length) { UI.modale({ titre: 'Variable inconnue', corps: '<div class="bandeau bandeau-erreur">' + UI.ic('alert-circle') + '<span>La variable ' + F.e(inconnues[0]) + ' n’existe pas. Variables disponibles : {nom}, {montant}, {mois}, {lot}, {date}, {lien_quittance}.</span></div>', actions: [{ lib: 'Fermer', v: 'primaire', focus: true }] }); return; }
    UI.modale({ titre: 'Aperçu du modèle', corps: '<div class="msg-brouillon">' + F.e(txt.replace(/\{(\w+)\}/g, function (m, k) { return ok[k]; })) + '</div><p class="muted" style="font-size:var(--text-sm);margin-top:8px">Exemple avec des valeurs fictives.</p>', actions: [{ lib: 'Fermer', v: 'primaire', focus: true }] });
  });
  App.action('par-enreg', function () {
    var pre = document.getElementById('po-prefixe'), ok = true;
    if (pre) { var v = pre.value.trim(), e = document.getElementById('po-prefixe-err'); var bon = /^[A-Z]{2,5}$/.test(v); e.classList.toggle('cache', bon); e.textContent = bon ? '' : 'Le préfixe comprend de 2 à 5 lettres majuscules.'; ok = bon; if (!bon) { pre.focus(); return; } }
    UI.toast('Paramètres enregistrés (démonstration).', { type: 'succes' });
  });
  App.action('par-export', function () { UI.toast('Action de démonstration : l’export serait téléchargé.', { type: 'info' }); });

  App.enregistrer('parametres', {
    meta: function () { return { titre: 'Paramètres', sous: 'Configurez l’organisation, les messages et les accès.', repere: { type: 'v1' }, actions: UI.btn('Relancer l’assistant de démarrage', { v: 'tertiaire', ic: 'sparkle', href: '#/assistant' }) }; },
    rendre: function (ctx) {
      var d = ctx.D, p = d.parametres, agence = d.organisation.plan === 'agence';
      var secu = carte('s-secu', 'lock', 'Sécurité', interrupteur('deuxFacteurs', 'Double authentification du gérant', 'Un code à usage unique à chaque connexion. Priorité P1, tranche à définir.', false, 'par-securite'));
      var plan = carte('s-plan', 'briefcase', 'Plan et abonnement', '<dl class="def-liste" style="margin:0"><div><dt>Plan</dt><dd>' + (agence ? 'Agence' : 'Propriétaire') + '</dd></div><div><dt>Changement de plan</dt><dd>Par HABEK, sur demande</dd></div></dl><div style="margin-top:12px;display:flex;gap:8px;align-items:center;flex-wrap:wrap">' + UI.btn('Gérer l’abonnement en ligne', { v: 'secondaire', petit: true, disabled: true }) + UI.repere({ type: 'avenir', mini: true }) + '</div>');
      var exp = carte('s-exp', 'download', 'Exports', '<p class="secondaire" style="margin-bottom:12px">Exports pour le comptable : paiements, impayés, comptes rendus.</p><div class="btn-groupe">' + UI.btn('Exporter en CSV', { v: 'secondaire', petit: true, action: 'par-export' }) + UI.btn('Exporter en Excel', { v: 'secondaire', petit: true, action: 'par-export' }) + UI.btn('Exporter en PDF', { v: 'secondaire', petit: true, action: 'par-export' }) + '</div>', UI.repere({ type: 'v1', pilote: true, mini: true }));
      return '<div class="param-page"><nav class="param-nav" aria-label="Sections des paramètres"><a href="#/parametres" data-ancre="s-org">Organisation</a><a href="#/parametres" data-ancre="s-mod">Modèles de messages</a><a href="#/parametres" data-ancre="s-droits">Droits</a><a href="#/parametres" data-ancre="s-secu">Sécurité</a><a href="#/parametres" data-ancre="s-exp">Exports</a><a href="#/parametres" data-ancre="s-sup">Accès support</a></nav><div class="param-corps">' + sectionOrg(d, p) + sectionModeles(d) + sectionDroits(d, p) + plan + secu + exp + sectionSupport(d, p) + '</div></div>';
    },
    apres: function (ctx, rac) { rac.querySelectorAll('[data-ancre]').forEach(function (a) { a.addEventListener('click', function (ev) { ev.preventDefault(); var c = document.getElementById(a.getAttribute('data-ancre')); if (c) c.scrollIntoView({ behavior: 'auto', block: 'start' }); }); }); }
  });
})();
