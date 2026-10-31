/*
 * S16 : Import guidé Excel / CSV (parcours 4).
 * Pas-à-pas : téléverser → associer les colonnes → aperçu et contrôle → confirmer → rapport. Historique des imports.
 * Règle de cohérence : l'import de la démonstration n'ajoute AUCUNE donnée au portefeuille (R-080). Il inscrit une ligne d'historique.
 * Le contenu du fichier choisi est remplacé par le fichier d'exemple portefeuille-agence.xlsx (200 lignes, 3 erreurs, 2 doublons).
 */
(function () {
  var F = window.Fmt, UI = window.UI, Data = window.Data, App = window.App, Mut = window.Mut, Etat = window.Etat;
  var st = null;
  function EX() { return window.DEMO.hor.importExemple; }
  function neuf() { return { etape: 1, analyse: false, erreurFichier: false, choixErreurs: '', fusions: { 0: 'fusionner', 1: 'fusionner' }, mapping: null, termine: null }; }
  var CIBLES = ['Locataire : nom', 'Locataire : téléphone', 'Bien : quartier', 'Lot : référence', 'Bail : loyer', "Bail : jour d'échéance", 'Bail : date de début', 'Mandant', '— Ne pas importer —'];
  var COLONNES_ATTENDUES = ['Nom du locataire', 'Téléphone', 'Quartier', 'Lot', 'Loyer', 'Jour d’échéance', 'Date d’entrée', 'Mandant'];
  var ETAPES = ['Téléverser', 'Colonnes', 'Contrôle', 'Confirmation'];

  function exemples(source) {
    var l = EX().lignes.filter(function (x) { return !x.erreur; }).slice(0, 3);
    var champ = { 'Nom du locataire': 'nom', 'Tél.': 'telephone', 'Quartier': 'quartier', 'Maison/Ref': 'reference', 'Loyer mensuel': 'loyer', 'Jour de paiement': 'jour', "Date d'entrée": 'dateEntree', 'Propriétaire': 'proprietaire' }[source];
    return l.map(function (x) { return champ === 'loyer' ? F.nombre(x.loyer) : F.e(x[champ]); }).join(' · ');
  }

  function htmlEtapes() {
    return '<nav aria-label="Étapes de l’import"><ol class="etapes">' + ETAPES.map(function (e, i) {
      var n = i + 1, cls = n < st.etape ? 'fait' : n === st.etape ? 'courant' : '';
      return '<li class="etape ' + cls + '"' + (n === st.etape ? ' aria-current="step"' : '') + '><span class="etape-n">' + (n < st.etape ? UI.ic('check', 'ic-sm') : n) + '</span><span class="etape-lib">' + e + '</span></li>';
    }).join('') + '</ol></nav><p class="sr-only" aria-live="polite">Étape ' + st.etape + ' sur 4</p>';
  }

  /* ---------- Étape 1 ---------- */
  function htmlEtape1() {
    var erreur = st.erreurFichier ? '<div class="bandeau bandeau-erreur" style="margin-bottom:16px" role="alert">' + UI.ic('alert-circle') + '<div><strong>Ce fichier n’est pas lisible.</strong><br>Utilisez un fichier Excel (.xlsx) ou CSV avec ces colonnes : ' + COLONNES_ATTENDUES.join(', ') + '.<div style="margin-top:8px">' + UI.btn('Télécharger le modèle Excel', { v: 'secondaire', petit: true, ic: 'download', action: 'imp-modele' }) + '</div></div></div>' : '';
    return erreur + '<section class="carte" aria-labelledby="t-e1"><h2 id="t-e1" class="carte-titre" style="margin-bottom:8px">' + UI.ic('upload') + 'Téléverser votre fichier</h2>' +
      '<p class="secondaire" style="margin-bottom:16px">Vous avez déjà votre portefeuille dans un fichier Excel ? Importez-le sans tout ressaisir. Rien n’est créé avant votre confirmation.</p>' +
      '<div class="btn-groupe" style="margin-bottom:16px">' + UI.btn('Télécharger le modèle Excel', { v: 'secondaire', ic: 'download', action: 'imp-modele' }) + '</div>' +
      '<div class="depot" id="depot" tabindex="-1">' + UI.ic('upload') + '<strong>Déposez votre fichier ici</strong><span class="secondaire">ou choisissez-le sur votre appareil (Excel ou CSV)</span><label class="btn btn-secondaire" for="fichier" style="margin-top:8px">' + UI.ic('file-text') + '<span>Choisir un fichier</span></label><input type="file" id="fichier" accept=".xlsx,.xls,.csv" class="sr-only"></div>' +
      '<div class="imp-exemple"><div><strong>' + F.e(EX().fichier) + '</strong><div class="secondaire" style="font-size:var(--text-sm)">Fichier d’exemple · ' + EX().total + ' lignes · ' + EX().tailleKo + ' Ko</div></div>' + UI.btn('Utiliser ce fichier', { v: 'primaire', ic: 'check', action: 'imp-choisir', id: 'btn-exemple' }) + '</div>' +
      '<p class="muted" style="font-size:var(--text-sm);margin-top:12px">Démonstration : le contenu du fichier choisi est remplacé par ce fichier d’exemple. <button type="button" class="lien-bouton" data-action="imp-invalide">Tester un fichier invalide</button></p></section>';
  }
  function htmlSquelette() {
    return '<section class="carte" aria-busy="true" aria-labelledby="t-an"><h2 id="t-an" class="carte-titre" style="margin-bottom:12px">' + UI.ic('clock') + 'Analyse en cours…</h2><p class="secondaire" style="margin-bottom:12px">Lecture de ' + F.e(EX().fichier) + ', ' + EX().total + ' lignes.</p>' +
      [1, 2, 3, 4, 5].map(function () { return '<div class="squelette" style="height:44px;margin-bottom:8px"></div>'; }).join('') + '</section>';
  }

  /* ---------- Étape 2 ---------- */
  function htmlEtape2() {
    var lignes = EX().colonnes.map(function (c, i) {
      var opts = CIBLES.map(function (o) { return '<option' + (o === c.cible ? ' selected' : '') + '>' + F.e(o) + '</option>'; }).join('');
      var sure = c.confiance === 'sure';
      return '<li class="map-ligne"><div class="map-source"><span class="gras">' + F.e(c.source) + '</span><span class="muted" style="font-size:var(--text-sm)">' + exemples(c.source) + '</span></div><span class="map-fleche" aria-hidden="true">' + UI.ic('arrow-right') + '</span>' +
        '<div class="map-cible"><label class="sr-only" for="map-' + i + '">Champ Immo Suite pour la colonne ' + F.e(c.source) + '</label><select class="saisie" id="map-' + i + '">' + opts + '</select>' +
        (sure ? '<span class="badge badge-paid">' + UI.ic('check-circle') + 'Reconnue</span>' : '<span class="badge badge-due">' + UI.ic('alert-circle') + 'À vérifier</span>') + '</div></li>';
    }).join('');
    return '<section class="carte" aria-labelledby="t-e2"><h2 id="t-e2" class="carte-titre" style="margin-bottom:8px">' + UI.ic('sliders') + 'Associer les colonnes</h2><p class="secondaire" style="margin-bottom:12px">Immo Suite reconnaît vos colonnes. Vérifiez les propositions, corrigez si besoin.</p><ul class="map-liste">' + lignes + '</ul>' +
      '<div class="btn-groupe" style="margin-top:16px">' + UI.btn('Continuer', { v: 'primaire', ic: 'arrow-right', action: 'imp-vers3', id: 'btn-vers3' }) + UI.btn('Retour', { v: 'tertiaire', action: 'imp-retour' }) + '</div></section>';
  }

  /* ---------- Étape 3 ---------- */
  function htmlEtape3() {
    var e = EX(), erreurs = e.lignes.filter(function (l) { return l.erreur; }), liste = erreurs.map(function (l) { return l.ligne; }).join(', ');
    var cartes = '<div class="imp-chiffres"><div class="carte"><div class="chiffre"><span class="chiffre-lib">' + UI.ic('check-circle') + 'Lignes valides</span><span class="chiffre-val" style="color:var(--status-paid-fg)">' + e.valides + '</span></div></div>' +
      '<div class="carte"><div class="chiffre chiffre-retard"><span class="chiffre-lib">' + UI.ic('alert-triangle') + 'Erreurs</span><span class="chiffre-val">' + e.erreurs + '</span></div></div>' +
      '<div class="carte"><div class="chiffre"><span class="chiffre-lib">' + UI.ic('users') + 'Doublons de téléphone</span><span class="chiffre-val">' + e.doublons.length + '</span></div></div></div>';
    var msg = '<div class="bandeau bandeau-avert" style="margin:16px 0">' + UI.ic('alert-triangle') + '<span>' + e.valides + ' lignes valides, ' + e.erreurs + ' erreurs. Corrigez les erreurs dans votre fichier (lignes ' + liste + ') ou continuez sans ces lignes. <button type="button" class="lien-bouton" data-action="imp-rapport">Télécharger le rapport</button></span></div>';
    var tab = '<div class="tableau-cadre"><table class="tableau"><thead><tr><th scope="col">Ligne</th><th scope="col">Locataire</th><th scope="col">Colonne</th><th scope="col">Problème</th></tr></thead><tbody>' + erreurs.map(function (l) {
      var col = { telephone: 'Tél.', loyer: 'Loyer mensuel', dateEntree: 'Date d’entrée' }[l.erreur.champ];
      return '<tr><td class="gras num">' + l.ligne + '</td><td>' + F.e(l.nom) + '</td><td>' + col + '</td><td>' + F.e(l.erreur.message) + '</td></tr>';
    }).join('') + '</tbody></table></div>';
    var choix = '<fieldset class="champ" style="border:0;padding:0;margin:16px 0 0"><legend class="champ-etiquette" style="padding:0;margin-bottom:6px">Que faire des ' + e.erreurs + ' lignes en erreur ? (obligatoire)</legend><div class="segmente segmente-3 choix-erreurs" style="grid-template-columns:repeat(auto-fit,minmax(220px,1fr))">' +
      '<input type="radio" name="erreurs" id="er-continuer" value="continuer"' + (st.choixErreurs === 'continuer' ? ' checked' : '') + '><label for="er-continuer">' + UI.ic('check', 'ic-sm coche') + 'Continuer sans ces lignes</label>' +
      '<input type="radio" name="erreurs" id="er-corriger" value="corriger"' + (st.choixErreurs === 'corriger' ? ' checked' : '') + '><label for="er-corriger">' + UI.ic('check', 'ic-sm coche') + 'Corriger le fichier et recommencer</label></div></fieldset>';
    var dbl = e.doublons.map(function (d, i) {
      var v = st.fusions[i];
      return '<li class="doublon"><div><div class="gras">Même téléphone : ' + F.e(d.telephone) + '</div><div class="secondaire" style="font-size:var(--text-sm)">Lignes ' + d.lignes.join(' et ') + ' · ' + F.e(d.noms[0]) + ' / ' + F.e(d.noms[1]) + '</div></div>' +
        '<div class="segmente" style="grid-template-columns:repeat(2,minmax(0,1fr));margin-top:8px"><input type="radio" name="dbl' + i + '" id="dbl' + i + 'f" value="fusionner"' + (v === 'fusionner' ? ' checked' : '') + '><label for="dbl' + i + 'f">' + UI.ic('check', 'ic-sm coche') + 'Fusionner (proposé)</label><input type="radio" name="dbl' + i + '" id="dbl' + i + 'g" value="garder"' + (v === 'garder' ? ' checked' : '') + '><label for="dbl' + i + 'g">' + UI.ic('check', 'ic-sm coche') + 'Garder les deux</label></div></li>';
    }).join('');
    var ap = '<div class="tableau-cadre"><table class="tableau"><thead><tr><th scope="col">Ligne</th><th scope="col">Locataire</th><th scope="col">Téléphone</th><th scope="col">Quartier</th><th scope="col" class="droite">Loyer</th></tr></thead><tbody>' + e.lignes.filter(function (l) { return !l.erreur; }).slice(0, 5).map(function (l) { return '<tr><td class="num">' + l.ligne + '</td><td>' + F.e(l.nom) + '</td><td>' + F.e(l.telephone) + '</td><td>' + F.e(l.quartier) + '</td><td class="droite">' + F.montant(l.loyer) + '</td></tr>'; }).join('') + '</tbody></table></div>';
    return cartes + msg + '<section class="carte" style="margin-bottom:16px" aria-labelledby="t-err"><h2 id="t-err" class="carte-titre" style="margin-bottom:12px">' + UI.ic('alert-triangle') + 'Erreurs listées ligne par ligne</h2>' + tab + choix + '</section>' +
      '<section class="carte" style="margin-bottom:16px" aria-labelledby="t-dbl"><h2 id="t-dbl" class="carte-titre" style="margin-bottom:12px">' + UI.ic('users') + 'Doublons proposés à la fusion</h2><ul class="doublons">' + dbl + '</ul></section>' +
      '<section class="carte" aria-labelledby="t-ap"><h2 id="t-ap" class="carte-titre" style="margin-bottom:12px">' + UI.ic('file-text') + 'Aperçu des lignes valides (5 sur ' + e.valides + ')</h2>' + ap + '</section>' +
      '<div class="btn-groupe" style="margin-top:16px">' + UI.btn('Continuer', { v: 'primaire', ic: 'arrow-right', action: 'imp-vers4', id: 'btn-vers4' }) + UI.btn('Retour', { v: 'tertiaire', action: 'imp-retour' }) + '</div><p class="champ-aide cache" id="vers4-raison" role="alert" style="margin-top:6px"></p>';
  }

  /* ---------- Étape 4 ---------- */
  function htmlEtape4() {
    var e = EX(), fus = Object.keys(st.fusions).filter(function (k) { return st.fusions[k] === 'fusionner'; }).length;
    if (st.termine) {
      return '<section class="carte" aria-labelledby="t-fin"><div class="bandeau bandeau-succes" style="margin-bottom:16px">' + UI.ic('check-circle') + '<span><strong>Import terminé : ' + e.valides + ' lignes créées.</strong> Dans cette démonstration, aucune donnée n’a été ajoutée à votre portefeuille.</span></div><h2 id="t-fin" class="carte-titre" style="margin-bottom:8px">' + UI.ic('file-text') + 'Rapport d’import</h2>' +
        '<ul class="liste"><li><span>Lignes lues</span><span class="gras">' + e.total + '</span></li><li><span>Lignes créées</span><span class="gras">' + e.valides + '</span></li><li><span>Lignes ignorées (erreurs : ' + e.lignes.filter(function (l) { return l.erreur; }).map(function (l) { return l.ligne; }).join(', ') + ')</span><span class="gras">' + e.erreurs + '</span></li><li><span>Doublons fusionnés</span><span class="gras">' + fus + '</span></li></ul>' +
        '<div class="btn-groupe" style="margin-top:16px">' + UI.btn('Télécharger le rapport', { v: 'primaire', ic: 'download', action: 'imp-rapport' }) + UI.btn('Importer un autre fichier', { v: 'secondaire', action: 'imp-recommencer' }) + '</div></section>';
    }
    return '<section class="carte" aria-labelledby="t-e4"><h2 id="t-e4" class="carte-titre" style="margin-bottom:8px">' + UI.ic('check-circle') + 'Confirmer l’import</h2><p class="secondaire" style="margin-bottom:12px">Rien n’est créé tant que vous n’avez pas confirmé.</p>' +
      '<ul class="liste"><li><span>Fichier</span><span class="gras">' + F.e(e.fichier) + '</span></li><li><span>Lignes qui seront importées</span><span class="gras">' + e.valides + '</span></li><li><span>Lignes ignorées (erreurs)</span><span class="gras">' + e.erreurs + '</span></li><li><span>Doublons fusionnés</span><span class="gras">' + fus + '</span></li></ul>' +
      '<div class="btn-groupe" style="margin-top:16px">' + UI.btn('Importer ' + e.valides + ' lignes', { v: 'primaire', ic: 'check', action: 'imp-confirmer', id: 'btn-confirmer' }) + UI.btn('Retour', { v: 'tertiaire', action: 'imp-retour' }) + '</div></section>';
  }

  /* ---------- Historique ---------- */
  function htmlHistorique() {
    var d = Data.org(), h = d.imports;
    var lignes = h.map(function (i) {
      var statut = i.statut === 'annule' ? '<span class="badge badge-void">' + UI.ic('ban') + '<span class="badge-texte">Annulé</span></span>' : '<span class="badge badge-paid">' + UI.ic('check-circle') + 'Terminé</span>';
      var ann = i.annulable ? '<span class="badge badge-brand">' + UI.ic('refresh') + 'Annulable</span> ' + UI.btn('Annuler l’import', { v: 'tertiaire', petit: true, action: 'imp-annuler', data: ' data-id="' + i.id + '"' }) : '<span class="muted" style="font-size:var(--text-sm)">' + F.e(i.note || '') + '</span>';
      return '<li class="hist-ligne"><div class="liste-gauche"><span class="liste-titre">' + F.e(i.fichier) + '</span><span class="liste-sous">' + F.date(i.date) + ' · ' + F.e(Data.utilisateur(i.parId).nom) + ' · ' + i.creees + ' lignes créées' + (i.erreurs ? ', ' + i.erreurs + ' erreurs' : '') + '</span></div><div class="liste-droite" style="flex-direction:row;flex-wrap:wrap;justify-content:flex-end;align-items:center;gap:8px">' + statut + ann + '</div></li>';
    }).join('');
    return '<section class="carte" style="margin-top:24px" aria-labelledby="t-hist"><h2 id="t-hist" class="carte-titre" style="margin-bottom:8px">' + UI.ic('clock') + 'Historique des imports</h2>' + (h.length ? '<ul class="liste">' + lignes + '</ul>' : '<p class="secondaire">Aucun import pour le moment.</p>') + '</section>';
  }

  function afficher() {
    var z = document.getElementById('imp-corps');
    z.innerHTML = htmlEtapes() + '<div id="imp-etape">' + (st.analyse ? htmlSquelette() : st.etape === 1 ? htmlEtape1() : st.etape === 2 ? htmlEtape2() : st.etape === 3 ? htmlEtape3() : htmlEtape4()) + '</div>' + htmlHistorique();
    var f = z.querySelector('#fichier'); if (f) f.addEventListener('change', function () {
      var nom = (f.files[0] && f.files[0].name || '').toLowerCase();
      if (!/\.(xlsx|xls|csv)$/.test(nom)) { st.erreurFichier = true; afficher(); return; }
      lancerAnalyse();
    });
    z.querySelectorAll('input[name="erreurs"]').forEach(function (r) { r.addEventListener('change', function () { st.choixErreurs = r.value; var m = document.getElementById('vers4-raison'); if (m) m.classList.add('cache'); }); });
    EX().doublons.forEach(function (d, i) { z.querySelectorAll('input[name="dbl' + i + '"]').forEach(function (r) { r.addEventListener('change', function () { st.fusions[i] = r.value; }); }); });
    var cible = z.querySelector('#t-e1, #t-e2, #t-err, #t-e4, #t-fin, #t-an'); var h1 = document.getElementById('titre-page'); if (h1 && !st.premier) { st.premier = true; }
  }
  function lancerAnalyse() {
    st.erreurFichier = false; st.analyse = true; afficher();
    App.attendre(function () { App.attendre(function () { st.analyse = false; st.etape = 2; afficher(); var t = document.getElementById('t-e2'); if (t) { t.setAttribute('tabindex', '-1'); t.focus(); } window.scrollTo(0, 0); }); });
  }
  function rapportCsv() {
    var e = EX(), l = ['ligne;statut;detail'];
    e.lignes.filter(function (x) { return x.erreur; }).forEach(function (x) { l.push(x.ligne + ';erreur;' + x.erreur.message.replace(/;/g, ',')); });
    e.doublons.forEach(function (d, i) { l.push(d.lignes.join('+') + ';doublon ' + (st && st.fusions[i] === 'garder' ? 'conservé' : 'fusionné') + ';telephone ' + d.telephone); });
    l.push('-;resume;' + e.total + ' lignes lues, ' + e.valides + ' valides, ' + e.erreurs + ' erreurs');
    return '﻿' + l.join('\r\n');
  }

  App.action('imp-modele', function () { UI.toast('Action de démonstration : le modèle Excel Immo Suite serait téléchargé.', { type: 'info' }); });
  App.action('imp-choisir', function () { lancerAnalyse(); });
  App.action('imp-invalide', function () { st.erreurFichier = true; afficher(); var b = document.querySelector('#imp-etape .bandeau-erreur'); if (b) b.scrollIntoView({ block: 'nearest' }); });
  App.action('imp-retour', function () { st.etape = Math.max(1, st.etape - 1); st.termine = null; afficher(); });
  App.action('imp-vers3', function () { st.etape = 3; afficher(); window.scrollTo(0, 0); });
  App.action('imp-vers4', function () {
    if (!st.choixErreurs) { var m = document.getElementById('vers4-raison'); m.classList.remove('cache'); m.textContent = 'Choisissez ce que vous faites des lignes en erreur pour continuer.'; document.querySelector('input[name="erreurs"]').focus(); return; }
    if (st.choixErreurs === 'corriger') { st = neuf(); afficher(); UI.toast('Corrigez votre fichier puis téléversez-le de nouveau.', { type: 'info' }); return; }
    st.etape = 4; afficher(); window.scrollTo(0, 0);
  });
  App.action('imp-confirmer', function () {
    var e = EX();
    UI.modale({ titre: 'Importer ' + e.valides + ' lignes ?', corps: '<p>Rien n’est créé avant votre confirmation. ' + e.erreurs + ' lignes en erreur seront ignorées. Dans cette démonstration, aucune donnée n’est ajoutée à votre portefeuille.</p>',
      actions: [{ lib: 'Annuler', v: 'secondaire', focus: true }, { lib: 'Importer', v: 'primaire', fn: function (fermer) {
        fermer(); st.analyse = true; afficher();
        App.attendre(function () {
          var fus = Object.keys(st.fusions).filter(function (k) { return st.fusions[k] === 'fusionner'; }).length;
          Mut.appliquer('import.confirmer', { org: Etat.orgCle(), fichier: e.fichier, creees: e.valides, fusions: fus, parId: App.ctx.utilisateur.id });
          st.analyse = false; st.termine = true; afficher(); UI.toast('Import terminé : ' + e.valides + ' lignes créées.', { type: 'succes' });
          var t = document.getElementById('t-fin'); if (t) { t.setAttribute('tabindex', '-1'); t.focus(); }
        });
      } }] });
  });
  App.action('imp-rapport', function () {
    try {
      var blob = new Blob([rapportCsv()], { type: 'text/csv;charset=utf-8' }), a = document.createElement('a');
      a.href = URL.createObjectURL(blob); a.download = 'rapport-import.csv'; document.body.appendChild(a); a.click(); document.body.removeChild(a); setTimeout(function () { URL.revokeObjectURL(a.href); }, 1000);
      UI.toast('Rapport téléchargé : rapport-import.csv.', { type: 'succes' });
    } catch (e) { UI.toast('Le téléchargement n’est pas disponible ici.', { type: 'avert' }); }
  });
  App.action('imp-recommencer', function () { st = neuf(); afficher(); window.scrollTo(0, 0); });
  App.action('imp-annuler', function (el) {
    var id = el.getAttribute('data-id');
    UI.modale({ titre: 'Annuler cet import ?', corps: '<p>Les données importées disparaîtront du portefeuille tant qu’elles n’ont pas été utilisées. Dans cette démonstration, aucune donnée n’avait été ajoutée.</p>',
      actions: [{ lib: 'Garder l’import', v: 'secondaire', focus: true }, { lib: 'Annuler l’import', v: 'primaire', fn: function (fermer) { fermer(); Mut.appliquer('import.annuler', { org: Etat.orgCle(), id: id }); afficher(); UI.toast('Import annulé.', { type: 'info' }); } }] });
  });

  App.enregistrer('import', {
    meta: function () { return { titre: 'Import', sous: 'Votre carnet devient un outil de gestion, sans tout ressaisir.', repere: { type: 'v1' } }; },
    rendre: function (ctx) {
      st = neuf();
      var droit = ctx.role.id !== 'gestionnaire' || ctx.D.parametres.droits.gestionnaireImport;
      if (!droit) return '<div class="carte reserve">' + UI.etatVide({ ic: 'lock', titre: 'Import non autorisé', texte: 'Vous n’avez pas le droit d’importer. Demandez au gérant d’activer ce droit.' }) + '</div>';
      return '<div id="imp-corps"></div>';
    },
    apres: function (ctx, rac) { if (rac.querySelector('#imp-corps')) afficher(); }
  });
})();
