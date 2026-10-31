/*
 * S01 : Connexion et inscription · S02 : Assistant de démarrage (+ visite guidée du produit de 3 écrans).
 * Écrans publics (sans menu). Aucune authentification réelle : le mot de passe « faux » sert à montrer le blocage après 3 tentatives.
 * Garde-fous montrés : identifiant déjà utilisé, trop de tentatives, code à usage unique de 15 minutes, invitation expirée après 7 jours.
 */
(function () {
  var F = window.Fmt, UI = window.UI, Data = window.Data, App = window.App, Roles = window.Roles, Etat = window.Etat;
  var essais = 0;
  function marque() { return '<div class="cx-marque"><img class="marque-logo" src="assets/symbole.svg" alt="" width="44" height="44"><img class="marque-nom-img" src="assets/nom.svg" alt="Immo Suite" height="24"></div>'; }
  function err(id, t) { var e = document.getElementById(id); if (!e) return !t; e.classList.toggle('cache', !t); e.innerHTML = t ? UI.ic('alert-circle', 'ic-sm') + '<span>' + t + '</span>' : ''; return !t; }
  function champ(id, lib, type, ph, aide) { return '<div class="champ"><label class="champ-etiquette" for="' + id + '">' + lib + '</label><input class="saisie" id="' + id + '" type="' + type + '" autocomplete="off"' + (ph ? ' placeholder="' + ph + '"' : '') + '>' + (aide ? '<div class="champ-aide">' + aide + '</div>' : '') + '<div class="champ-erreur cache" id="' + id + '-err" role="alert"></div></div>'; }
  function lienMode(m, lib, actif) { return '<a class="onglet" role="tab" href="#/connexion?mode=' + m + '" aria-selected="' + actif + '">' + lib + '</a>'; }
  function tel8(t) { return t.replace(/\D/g, '').replace(/^226/, '').slice(-8); }

  function formConnexion() {
    return '<h1 class="cx-titre">Connexion</h1><p class="secondaire" style="margin-bottom:16px">Connectez-vous avec votre téléphone.</p>' + champ('cx-id', 'Téléphone ou e-mail (obligatoire)', 'text', '70 12 34 56') + champ('cx-mdp', 'Mot de passe (obligatoire)', 'password', '', 'Démonstration : tout mot de passe est accepté, sauf « faux » (blocage après 3 tentatives).') + '<div id="cx-bloc" aria-live="polite"></div>' +
      '<div style="margin-top:16px">' + UI.btn('Se connecter', { v: 'primaire', grand: true, ic: 'arrow-right', action: 'cx-valider', id: 'btn-cx' }) + '</div><p style="margin-top:12px;text-align:center"><a href="#/connexion?mode=oubli">Mot de passe oublié ?</a></p>';
  }
  function formInscription() {
    return '<h1 class="cx-titre">Créer mon espace</h1><p class="secondaire" style="margin-bottom:16px">Une organisation, un compte gérant.</p>' + champ('in-nom', 'Nom de l’organisation (obligatoire)', 'text') + champ('in-id', 'Téléphone (obligatoire)', 'text', '70 12 34 56') + champ('in-mdp', 'Mot de passe (obligatoire)', 'password', '', '8 caractères au moins.') +
      '<fieldset class="champ" style="border:0;padding:0;margin:16px 0 0"><legend class="champ-etiquette" style="padding:0;margin-bottom:6px">Type de compte</legend><div class="segmente" style="grid-template-columns:repeat(2,minmax(0,1fr))"><input type="radio" name="ty" id="ty-p"><label for="ty-p">' + UI.ic('check', 'ic-sm coche') + 'Propriétaire</label><input type="radio" name="ty" id="ty-a" checked><label for="ty-a">' + UI.ic('check', 'ic-sm coche') + 'Agence</label></div></fieldset>' +
      '<div class="champ"><label class="champ-etiquette" for="in-ville">Ville</label><select class="saisie" id="in-ville"><option>Ouagadougou</option><option>Bobo-Dioulasso</option></select></div><div class="champ"><label class="champ-etiquette" for="in-logo">Logo (facultatif)</label><input class="saisie" type="file" id="in-logo" accept=".png,.jpg,.svg"></div><div id="in-bloc" aria-live="polite"></div>' +
      '<div style="margin-top:16px">' + UI.btn('Créer mon espace', { v: 'primaire', grand: true, ic: 'check', action: 'cx-inscrire' }) + '</div>';
  }
  function formOubli() {
    return '<h1 class="cx-titre">Mot de passe oublié</h1><p class="secondaire" style="margin-bottom:16px">Nous vous envoyons un code à usage unique.</p>' + champ('ou-id', 'Téléphone (obligatoire)', 'text', '70 12 34 56') + '<div id="ou-etape"></div><div style="margin-top:16px" id="ou-act">' + UI.btn('Envoyer le code', { v: 'primaire', grand: true, action: 'cx-code' }) + '</div>';
  }
  function invitation() {
    return '<h1 class="cx-titre">Invitation</h1><div class="bandeau bandeau-erreur" style="margin-top:12px">' + UI.ic('alert-circle') + '<span>Cette invitation a expiré (validité : 7 jours). Demandez au gérant de la renvoyer.</span></div><p style="margin-top:16px"><a href="#/connexion">Retour à la connexion</a></p>';
  }

  App.action('cx-valider', function () {
    var id = document.getElementById('cx-id').value.trim(), mdp = document.getElementById('cx-mdp').value, ok = err('cx-id-err', id ? '' : 'Saisissez votre téléphone.'); ok = err('cx-mdp-err', mdp ? '' : 'Saisissez votre mot de passe.') && ok; if (!ok) return;
    var b = document.getElementById('btn-cx'); b.classList.add('charge'); b.querySelector('span').textContent = 'Connexion…';
    App.attendre(function () {
      b.classList.remove('charge'); b.querySelector('span').textContent = 'Se connecter';
      if (mdp === 'faux') { essais += 1; if (essais >= 3) { document.getElementById('cx-bloc').innerHTML = '<div class="bandeau bandeau-erreur" style="margin-top:12px">' + UI.ic('lock') + '<div>Trop de tentatives. Réessayez dans 15 minutes ou réinitialisez votre mot de passe.<div style="margin-top:8px"><a class="btn btn-secondaire btn-petit" href="#/connexion?mode=oubli">Réinitialiser</a></div></div></div>'; } else document.getElementById('cx-bloc').innerHTML = '<div class="bandeau bandeau-erreur" style="margin-top:12px">' + UI.ic('alert-circle') + '<span>Identifiant ou mot de passe incorrect. Il vous reste ' + (3 - essais) + ' essai' + (3 - essais > 1 ? 's' : '') + '.</span></div>'; return; }
      essais = 0; App.aller(Roles.def(Etat.role()).arrivee);
    });
  });
  App.action('cx-inscrire', function () {
    var nom = document.getElementById('in-nom').value.trim(), id = document.getElementById('in-id').value, mdp = document.getElementById('in-mdp').value, ok = err('in-nom-err', nom ? '' : 'Indiquez le nom de l’organisation.');
    if (tel8(id).length !== 8) ok = err('in-id-err', 'Numéro invalide. Format attendu : +226 suivi du numéro (exemple : +226 70 12 34 56).') && ok; else ok = err('in-id-err', '') && ok;
    ok = err('in-mdp-err', mdp.length >= 8 ? '' : 'Choisissez un mot de passe de 8 caractères au moins.') && ok;
    var dup = ok && ['hor', 'saw'].some(function (k) { return window.DEMO[k].utilisateurs.some(function (u) { return tel8(u.telephone) === tel8(id); }); });
    if (dup) { document.getElementById('in-bloc').innerHTML = '<div class="bandeau bandeau-avert" style="margin-top:12px">' + UI.ic('alert-triangle') + '<div>Cet identifiant a déjà un compte. Voulez-vous vous connecter ?<div style="margin-top:8px"><a class="btn btn-secondaire btn-petit" href="#/connexion">Se connecter</a></div></div></div>'; return; }
    if (!ok) return; document.getElementById('in-bloc').innerHTML = ''; UI.toast('Espace créé (démonstration). Aucune donnée n’est enregistrée.', { type: 'succes' }); App.aller('/assistant');
  });
  App.action('cx-code', function () {
    var id = document.getElementById('ou-id').value; if (!err('ou-id-err', tel8(id).length === 8 ? '' : 'Numéro invalide. Format attendu : +226 suivi du numéro (exemple : +226 70 12 34 56).')) return;
    document.getElementById('ou-etape').innerHTML = '<div class="bandeau bandeau-info" style="margin-top:12px">' + UI.ic('info') + '<span>Un code à usage unique vous a été envoyé. Il est valable 15 minutes.</span></div>' + champ('ou-code', 'Code reçu (6 chiffres)', 'text', '123456') + champ('ou-mdp', 'Nouveau mot de passe', 'password');
    document.getElementById('ou-act').innerHTML = UI.btn('Réinitialiser le mot de passe', { v: 'primaire', grand: true, action: 'cx-reinit-mdp' }); document.getElementById('ou-code').focus();
  });
  App.action('cx-reinit-mdp', function () {
    var ok = err('ou-code-err', /^\d{6}$/.test(document.getElementById('ou-code').value.trim()) ? '' : 'Saisissez le code à 6 chiffres.'); ok = err('ou-mdp-err', document.getElementById('ou-mdp').value.length >= 8 ? '' : 'Choisissez un mot de passe de 8 caractères au moins.') && ok; if (!ok) return;
    UI.toast('Mot de passe réinitialisé (démonstration).', { type: 'succes' }); App.aller('/connexion');
  });

  App.enregistrer('connexion', {
    meta: function () { return { titre: 'Connexion', repere: { type: 'v1' } }; },
    rendre: function (ctx) {
      var m = ctx.query.mode || 'connexion', corps = m === 'inscription' ? formInscription() : m === 'oubli' ? formOubli() : m === 'invitation' ? invitation() : formConnexion();
      return '<div class="cx-page"><div class="carte cx-carte">' + marque() + '<div class="onglets" role="tablist" style="margin-bottom:16px">' + lienMode('connexion', 'Se connecter', m === 'connexion') + lienMode('inscription', 'Créer mon espace', m === 'inscription') + lienMode('invitation', 'Invitation', m === 'invitation') + '</div>' + corps + '</div><p class="muted" style="text-align:center;margin-top:12px;font-size:var(--text-sm)">Maquette de démonstration · <a href="#/accueil">Retour à la maquette</a></p></div>';
    },
    apres: function (ctx, rac) { var i = rac.querySelector('input'); if (i) i.focus(); }
  });

  /* ---------- S02 : assistant de démarrage ---------- */
  var st = { etape: 1 };
  var ETAPES = ['Organisation', 'Premier bien', 'Premier bail', 'Invitations'];
  function etapes() { return '<nav aria-label="Étapes de l’assistant"><ol class="etapes">' + ETAPES.map(function (e, i) { var n = i + 1, c = n < st.etape ? 'fait' : n === st.etape ? 'courant' : ''; return '<li class="etape ' + c + '"' + (n === st.etape ? ' aria-current="step"' : '') + '><span class="etape-n">' + (n < st.etape ? UI.ic('check', 'ic-sm') : n) + '</span><span class="etape-lib">' + e + '</span></li>'; }).join('') + '</ol></nav><p class="sr-only" aria-live="polite">Étape ' + st.etape + ' sur 4</p>'; }
  function nav(avecPasser) { return '<div class="btn-groupe" style="margin-top:20px">' + UI.btn(st.etape === 4 ? 'Terminer' : 'Suivant', { v: 'primaire', ic: 'arrow-right', action: 'as-suite' }) + (avecPasser ? UI.btn('Passer', { v: 'tertiaire', action: 'as-suite' }) : '') + (st.etape > 1 ? UI.btn('Retour', { v: 'tertiaire', action: 'as-retour' }) : '') + '</div>'; }
  function corpsAs() {
    var d = Data.org(), o = d.parametres.organisation;
    if (st.etape === 1) return '<section class="carte"><h2 class="carte-titre" style="margin-bottom:12px">' + UI.ic('building') + 'Votre organisation</h2>' + champ('as-nom', 'Nom (obligatoire)', 'text').replace('autocomplete="off"', 'autocomplete="off" value="' + F.e(o.nom) + '"') + '<div class="champ"><label class="champ-etiquette" for="as-logo">Logo (facultatif)</label><input class="saisie" type="file" id="as-logo" accept=".png,.jpg,.svg"></div>' + champ('as-entete', 'En-tête de la quittance', 'text').replace('autocomplete="off"', 'autocomplete="off" value="' + F.e(o.enteteQuittance) + '"') + nav(false) + '</section>';
    if (st.etape === 2) return '<section class="carte"><h2 class="carte-titre" style="margin-bottom:8px">' + UI.ic('building') + 'Votre premier bien</h2><p class="secondaire" style="margin-bottom:12px">Une maison seule crée automatiquement 1 bien et 1 lot. Vous avez déjà un fichier Excel ? Importez-le.</p><div class="btn-groupe">' + UI.btn('Importer mon portefeuille', { v: 'secondaire', ic: 'upload', href: '#/import' }) + '</div>' + champ('as-bien', 'Ou créez un bien', 'text', 'Exemple : Cour Kaboré, Pissy') + nav(true) + '</section>';
    if (st.etape === 3) return '<section class="carte"><h2 class="carte-titre" style="margin-bottom:8px">' + UI.ic('file-text') + 'Votre premier bail</h2><p class="secondaire" style="margin-bottom:12px">Le bail génère les échéances et permet d’encaisser.</p><div class="btn-groupe">' + UI.btn('Créer un bail', { v: 'secondaire', ic: 'plus', href: '#/baux/nouveau' }) + '</div>' + nav(true) + '</section>';
    return '<section class="carte"><h2 class="carte-titre" style="margin-bottom:8px">' + UI.ic('user-plus') + 'Inviter votre équipe</h2><p class="secondaire" style="margin-bottom:12px">Chaque personne a le niveau d’accès qui lui convient. Vous pourrez inviter plus tard.</p>' + champ('as-tel', 'Téléphone du collaborateur', 'text', '70 12 34 56') + nav(true) + '</section>';
  }
  function termine() {
    return '<section class="carte"><div class="etat-vide">' + UI.ic('check-circle') + '<h2>Votre espace est prêt</h2><p class="secondaire">Aucun impayé pour le moment. Importez vos baux pour commencer.</p><div class="btn-groupe" style="justify-content:center">' + UI.btn('Voir la visite guidée', { v: 'primaire', ic: 'play', action: 'visite' }) + UI.btn('Aller à l’accueil', { v: 'secondaire', href: '#/accueil' }) + '</div></div></section>';
  }
  function afficher() { document.getElementById('as').innerHTML = st.etape > 4 ? termine() : etapes() + corpsAs(); }
  App.action('as-suite', function () {
    if (st.etape === 1) { var n = document.getElementById('as-nom'); if (!err('as-nom-err', n.value.trim() ? '' : 'Indiquez le nom de l’organisation.')) { n.focus(); return; } }
    st.etape += 1; afficher(); window.scrollTo(0, 0);
  });
  App.action('as-retour', function () { st.etape -= 1; afficher(); });
  App.action('visite', function () {
    var ecrans = [['banknote', 'Encaisser en quelques secondes', 'Recherchez le locataire, vérifiez le montant et le mode, validez. La quittance numérotée part sur WhatsApp.'], ['alert-triangle', 'Savoir qui doit quoi', 'Les impayés sont classés par ancienneté. Un message prêt, un clic pour relancer, la promesse est notée.'], ['chart', 'Rendre compte au propriétaire', 'En fin de mois, le compte rendu est calculé : loyers, commission, dépenses, net à reverser.']], i = 0;
    var m = UI.modale({ titre: 'Visite guidée', corps: '<div id="vg"></div>', actions: [{ lib: 'Précédent', v: 'secondaire', fn: function () { if (i > 0) { i -= 1; rendu(); } } }, { lib: 'Suivant', v: 'primaire', focus: true, fn: function (fermer) { if (i < 2) { i += 1; rendu(); } else fermer(); } }] });
    function rendu() { var e = ecrans[i]; document.getElementById('vg').innerHTML = '<div class="etat-vide" style="padding:8px">' + UI.ic(e[0]) + '<h3 style="font-size:var(--text-xl)">' + e[1] + '</h3><p class="secondaire">' + e[2] + '</p><p class="muted" style="font-size:var(--text-sm)">Écran ' + (i + 1) + ' sur 3</p></div>'; var b = m.el.querySelectorAll('.modale-actions button'); b[0].disabled = i === 0; b[1].textContent = i === 2 ? 'Terminer' : 'Suivant'; }
    rendu();
  });
  App.enregistrer('assistant', {
    meta: function () { return { titre: 'Assistant de démarrage', sous: 'Quatre étapes pour obtenir votre première donnée utile.', repere: { type: 'v1' } }; },
    rendre: function () { st = { etape: 1 }; return '<div id="as"></div>'; },
    apres: function () { afficher(); }
  });
})();
