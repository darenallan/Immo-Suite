/*
 * S05 : Impayés + panneau « Relancer » (parcours 2).
 * Le tableau regroupe les échéances en retard par bail ; l'ancienneté, les tranches et les totaux viennent du jeu de données.
 * La relance est assistée (R-061) : le texte est prêt, WhatsApp ou SMS s'ouvre (simulé), l'utilisateur confirme l'envoi.
 * Garde-fous montrés : relance déjà faite aujourd'hui (R-063), variable inconnue (R-064), locataire sans numéro WhatsApp.
 * window.Relance.ouvrir(bailId, mode, echeanceId) est aussi utilisé par l'accueil du gestionnaire (rappels à J−3).
 */
(function () {
  var F = window.Fmt, UI = window.UI, Data = window.Data, App = window.App, Mut = window.Mut, Etat = window.Etat;
  var TRANCHES = [{ cle: '1-7', lib: '1 à 7 jours' }, { cle: '8-30', lib: '8 à 30 jours' }, { cle: '31-59', lib: '31 à 59 jours' }, { cle: '60+', lib: '60 jours et plus' }];
  var f = { mandant: '', bien: '', gestionnaire: '', usage: '', tranche: '' };
  var VARIABLES = ['nom', 'montant', 'mois', 'lot', 'date', 'lien_quittance'];
  function D() { return Data.org(); }
  function orgCle() { return Etat.orgCle(); }
  var ART = { maison: 'la ', villa: 'la ', appartement: 'l’', studio: 'le ', chambre: 'la ', boutique: 'la ', atelier: 'l’', entrepot: 'l’', local: 'le ' };

  /* ---------- Regroupement par bail ---------- */
  function groupes() {
    var d = D(), m = {};
    d.impayes.forEach(function (i) { (m[i.bailId] = m[i.bailId] || { bailId: i.bailId, lignes: [] }).lignes.push(i); });
    return Object.keys(m).map(function (k) {
      var g = m[k], b = Data.bail(k), lot = Data.lot(b.lotId);
      g.bail = b; g.lot = lot; g.bien = Data.bien(lot.bienId); g.locataire = Data.locataire(b.locataireIds[0]);
      g.reste = g.lignes.reduce(function (s, x) { return s + x.reste; }, 0);
      g.anciennete = Math.max.apply(null, g.lignes.map(function (x) { return x.ancienneteJours; }));
      g.tranches = g.lignes.map(function (x) { return x.tranche; });
      g.derniereRelance = d.relances.filter(function (r) { return r.bailId === k; }).sort(function (a, c) { return (a.date + a.heure).localeCompare(c.date + c.heure); }).pop() || null;
      g.promesse = d.promesses.filter(function (p) { return p.bailId === k && p.statut !== 'tenue'; }).pop() || null;
      g.prochaine = (g.lignes.filter(function (x) { return x.prochaineAction; })[0] || {}).prochaineAction || null;
      return g;
    }).sort(function (a, b) { return b.anciennete - a.anciennete; });
  }
  function filtrer(gs) {
    return gs.filter(function (g) {
      if (f.mandant && g.lot.mandantId !== f.mandant) return false;
      if (f.bien && g.lot.bienId !== f.bien) return false;
      if (f.gestionnaire && g.lot.gestionnaireId !== f.gestionnaire) return false;
      if (f.usage && g.lot.usage !== f.usage) return false;
      if (f.tranche && g.tranches.indexOf(f.tranche) < 0) return false;
      return true;
    });
  }

  var CANAL = { whatsapp: 'WhatsApp', sms: 'SMS', appel: 'Appel' };
  var TON = { rappel: 'rappel J−3', courtois: 'courtois', ferme: 'ferme', dernier_avis: 'dernier avis' };
  function htmlRelance(r) {
    if (!r) return '<span class="muted">Aucune relance</span>';
    return '<span>' + F.date(r.date) + '</span><br><span class="muted" style="font-size:var(--text-sm)">' + CANAL[r.canal] + ' · ' + TON[r.ton] + '</span>';
  }
  function htmlPromesse(p) {
    if (!p) return '<span class="muted">—</span>';
    if (p.statut === 'non_tenue') return '<span class="badge badge-late">' + UI.ic('alert-triangle') + 'Non tenue · ' + F.date(p.date).slice(0, 5) + '</span><br><span class="muted" style="font-size:var(--text-sm)">' + F.montant(p.montant) + '</span>';
    return '<span class="badge badge-due">' + UI.ic('clock') + 'Promesse · ' + F.date(p.date).slice(0, 5) + '</span><br><span class="muted" style="font-size:var(--text-sm)">' + F.montant(p.montant) + '</span>';
  }
  function htmlProchaine(a) { return a ? F.e(a.libelle) + '<br><span class="muted" style="font-size:var(--text-sm)">' + F.date(a.date) + '</span>' : '<span class="muted">—</span>'; }
  function ancBadge(g) { return UI.statut('en_retard', F.jours(g.anciennete)) + (g.lignes.length > 1 ? '<br><span class="muted" style="font-size:var(--text-sm)">' + g.lignes.length + ' échéances</span>' : ''); }

  function peutRelancer() { return window.Roles.droit(App.ctx.role.id, 'impayes') === 'E'; }

  function htmlTableau(gs) {
    if (!gs.length) {
      var filtres = f.mandant || f.bien || f.gestionnaire || f.usage || f.tranche;
      return UI.etatVide({ ic: 'check-circle', titre: 'Aucun impayé. Bravo, tout est à jour.', texte: filtres ? 'Aucun impayé ne correspond à ces filtres.' : 'Toutes les échéances échues sont réglées.', action: '<div class="btn-groupe" style="justify-content:center">' + UI.btn('Voir l’historique des paiements', { v: 'primaire', href: '#/paiements' }) + (filtres ? UI.btn('Effacer les filtres', { v: 'secondaire', action: 'imp-effacer' }) : '') + '</div>' });
    }
    var act = peutRelancer();
    var lignes = gs.map(function (g) {
      var attr = act ? ' data-action="imp-relancer" data-bail="' + g.bailId + '" tabindex="0"' : '';
      return '<tr class="ligne-cliquable"' + attr + ' data-ligne="' + g.bailId + '"><td><span class="gras">' + F.e(g.locataire.nom) + '</span>' + (g.locataire.activite ? '<br><span class="muted" style="font-size:var(--text-sm)">' + F.e(g.locataire.activite) + '</span>' : '') + '</td><td>' + F.e(g.lot.nom) + '<br><span class="muted" style="font-size:var(--text-sm)">' + F.e(g.bien.quartier) + '</span></td><td>' + ancBadge(g) + '</td>' +
        '<td class="droite"><span class="gras" aria-label="' + F.e(F.montantSr(g.reste)) + '">' + F.montant(g.reste) + '</span></td><td>' + htmlRelance(g.derniereRelance) + '</td><td>' + htmlPromesse(g.promesse) + '</td><td>' + htmlProchaine(g.prochaine) + '</td>' +
        '<td>' + (act ? UI.btn('Relancer', { v: 'secondaire', petit: true, ic: 'message', action: 'imp-relancer', data: ' data-bail="' + g.bailId + '"' }) : '') + '</td></tr>';
    }).join('');
    var cartes = gs.map(function (g) {
      var attr = act ? ' data-action="imp-relancer" data-bail="' + g.bailId + '"' : ' disabled';
      return '<li><button type="button" class="carte-impaye" data-ligne="' + g.bailId + '"' + attr + '><span class="ci-haut"><span class="liste-gauche" style="flex:1"><span class="liste-titre">' + F.e(g.locataire.nom) + '</span><span class="liste-sous">' + F.e(g.lot.nom) + ', ' + F.e(g.bien.quartier) + '</span></span><span class="liste-droite"><span class="liste-montant">' + F.montant(g.reste) + '</span>' + UI.statut('en_retard', F.jours(g.anciennete)) + '</span></span>' +
        '<span class="ci-bas">' + (g.derniereRelance ? '<span>' + UI.ic('message', 'ic-sm') + ' ' + F.date(g.derniereRelance.date).slice(0, 5) + ' · ' + CANAL[g.derniereRelance.canal] + '</span>' : '<span class="muted">Aucune relance</span>') + (g.promesse ? htmlPromesse(g.promesse).split('<br>')[0] : '') + (g.prochaine ? '<span>' + UI.ic('calendar', 'ic-sm') + ' ' + F.date(g.prochaine.date).slice(0, 5) + '</span>' : '') + '</span></button></li>';
    }).join('');
    var total = gs.reduce(function (s, g) { return s + g.reste; }, 0);
    return '<div class="tab-bureau tableau-cadre"><table class="tableau"><thead><tr><th scope="col">Locataire</th><th scope="col">Lot</th><th scope="col">Ancienneté</th><th scope="col" class="droite">Reste dû</th><th scope="col">Dernière relance</th><th scope="col">Promesse</th><th scope="col">Prochaine action</th><th scope="col"><span class="sr-only">Action</span></th></tr></thead><tbody>' + lignes + '</tbody>' +
      '<tfoot><tr><td colspan="3">Total affiché</td><td class="droite">' + F.montant(total) + '</td><td colspan="4"></td></tr></tfoot></table></div><ul class="tab-mobile">' + cartes + '</ul>';
  }

  function htmlResume() {
    var tb = D().tableauDeBord, max = 0;
    TRANCHES.forEach(function (t) { max = Math.max(max, tb.parTranche[t.cle].montant); });
    var chips = TRANCHES.map(function (t) {
      var x = tb.parTranche[t.cle];
      return '<button type="button" class="tranche" aria-pressed="' + (f.tranche === t.cle) + '" data-action="imp-tranche" data-t="' + t.cle + '"><span class="tranche-lib">' + t.lib + '</span><strong class="num">' + F.montant(x.montant) + '</strong><span class="tranche-n">' + x.nombre + (x.nombre > 1 ? ' échéances' : ' échéance') + '</span><span class="barre-piste"><span class="barre-rempli' + (t.cle === '1-7' ? ' t1' : '') + '" style="width:' + (max ? Math.max(4, Math.round(100 * x.montant / max)) : 0) + '%;display:block"></span></span></button>';
    }).join('');
    return '<section class="carte" style="margin-bottom:16px" aria-labelledby="t-res"><div class="imp-resume"><div class="chiffre chiffre-retard"><span class="chiffre-lib">' + UI.ic('alert-triangle') + 'Total en retard</span><span class="chiffre-val" id="t-res" aria-label="' + F.e(F.montantSr(tb.enRetard.montant)) + '">' + F.nombre(tb.enRetard.montant) + '<small>FCFA</small></span><span class="chiffre-sous">' + tb.enRetard.locataires + ' locataires · ' + tb.enRetard.echeances + ' échéances</span></div>' +
      '<div class="tranches" role="group" aria-label="Répartition par ancienneté (filtre)">' + chips + '</div></div></section>';
  }

  function htmlFiltres() {
    var d = D(), agence = d.organisation.plan === 'agence';
    var sel = function (id, lib, opts, val) { return '<div class="champ"><label class="champ-etiquette" for="' + id + '">' + lib + '</label><select class="saisie" id="' + id + '">' + opts.map(function (o) { return '<option value="' + o[0] + '"' + (o[0] === val ? ' selected' : '') + '>' + F.e(o[1]) + '</option>'; }).join('') + '</select></div>'; };
    var biens = d.biens.slice().sort(function (a, b) { return a.nom.localeCompare(b.nom); });
    var h = '<div class="filtres filtres-imp">';
    if (agence) h += sel('f-mandant', 'Mandant', [['', 'Tous les mandants']].concat(d.mandants.map(function (m) { return [m.id, m.nom]; })), f.mandant);
    h += sel('f-bien', 'Bien', [['', 'Tous les biens']].concat(biens.map(function (b) { return [b.id, b.nom + ' (' + b.quartier + ')']; })), f.bien);
    if (agence) h += sel('f-gest', 'Gestionnaire', [['', 'Tous']].concat(d.utilisateurs.filter(function (u) { return u.role === 'gestionnaire' && u.statut === 'actif'; }).map(function (u) { return [u.id, u.nom]; })), f.gestionnaire);
    h += sel('f-usage', 'Usage', [['', 'Tous les usages'], ['habitation', 'Habitation'], ['commercial', 'Commercial'], ['mixte', 'Mixte']], f.usage) + '</div>';
    return h;
  }
  function puces() {
    var d = D(), p = [];
    if (f.mandant) p.push(['mandant', Data.mandant(f.mandant).nom]); if (f.bien) p.push(['bien', Data.bien(f.bien).nom]); if (f.gestionnaire) p.push(['gestionnaire', Data.utilisateur(f.gestionnaire).nom]);
    if (f.usage) p.push(['usage', F.majuscule(f.usage)]); if (f.tranche) p.push(['tranche', (TRANCHES.filter(function (t) { return t.cle === f.tranche; })[0] || {}).lib]);
    return p.length ? '<div class="puces-filtres">' + p.map(function (x) { return '<button type="button" class="puce-filtre" data-action="imp-retirer" data-k="' + x[0] + '">' + F.e(x[1]) + UI.ic('x', 'ic-sm') + '<span class="sr-only"> : retirer ce filtre</span></button>'; }).join('') + '<button type="button" class="lien-bouton" data-action="imp-effacer">Effacer tout</button></div>' : '';
  }
  function majPage() {
    var gs = filtrer(groupes());
    document.getElementById('imp-resume').innerHTML = htmlResume();
    document.getElementById('imp-puces').innerHTML = puces();
    document.getElementById('imp-liste').innerHTML = htmlTableau(gs);
    document.getElementById('imp-compte').textContent = gs.length + (gs.length > 1 ? ' locataires en retard' : ' locataire en retard');
  }

  /* ---------- Panneau « Relancer » ---------- */
  var ps = null, panneau = null;

  function moisLibelle(periodes) {
    var u = periodes.filter(function (p, i) { return periodes.indexOf(p) === i; }).sort();
    if (u.length === 1) return F.mois(u[0]);
    var noms = u.map(function (p) { return F.mois(p).split(' ')[0]; }), annees = u.map(function (p) { return p.slice(0, 4); });
    if (annees.every(function (a) { return a === annees[0]; })) return noms.slice(0, -1).join(', ') + ' et ' + noms[noms.length - 1] + ' ' + annees[0];
    return u.map(F.mois).join(', ');
  }
  function elision(s) { return s.replace(/\bde (octobre|août|avril)/g, 'd’$1'); }
  function contexte(bailId, mode, echeanceId) {
    var d = D(), b = Data.bail(bailId), lot = Data.lot(b.lotId), bien = Data.bien(lot.bienId), t = Data.locataire(b.locataireIds[0]);
    var eches = mode === 'rappel' ? [Data.echeance(echeanceId)] : d.impayes.filter(function (i) { return i.bailId === bailId; }).map(function (i) { return Data.echeance(i.echeanceId); }).sort(function (a, c) { return a.dateEcheance.localeCompare(c.dateEcheance); });
    var total = eches.reduce(function (s, e) { return s + e.reste; }, 0), anc = mode === 'rappel' ? 0 : Math.max.apply(null, eches.map(function (e) { return e.ancienneteJours; }));
    var jourToday = d.relances.filter(function (r) { return r.bailId === bailId && r.date === d.meta.aujourdhui; }).sort(function (a, c) { return a.heure.localeCompare(c.heure); }).pop() || null;
    return { bailId: bailId, mode: mode, bail: b, lot: lot, bien: bien, t: t, eches: eches, total: total, anciennete: anc, echeanceId: echeanceId, dejaAujourdhui: jourToday,
      variables: { nom: t.prenom, montant: F.montant(total), mois: moisLibelle(eches.map(function (e) { return e.periode; })), lot: ART[lot.type] + lot.nom + ', ' + bien.quartier, date: F.date(eches[0].dateEcheance), lien_quittance: '' } };
  }
  function etapeConseillee(c) { if (c.mode === 'rappel') return 'J-3'; return c.anciennete < 5 ? 'J+1' : c.anciennete < 10 ? 'J+5' : 'J+10'; }
  function modele(etape) { return D().modelesRelance.filter(function (m) { return m.etape === etape; })[0]; }
  function rendreTexte(c, etape) { return elision(modele(etape).texte.replace(/\{(\w+)\}/g, function (m, k) { return c.variables[k] !== undefined ? c.variables[k] : m; })); }

  function dateIso(txt) {
    var m = /^(\d{2})\/(\d{2})\/(\d{4})$/.exec(txt.trim()); if (!m) return null;
    var iso = m[3] + '-' + m[2] + '-' + m[1], d = new Date(iso + 'T00:00:00Z'); if (isNaN(d) || d.toISOString().slice(0, 10) !== iso) return null; return iso;
  }
  function plusJours(iso, n) { var d = new Date(iso + 'T00:00:00Z'); d.setUTCDate(d.getUTCDate() + n); return d.toISOString().slice(0, 10); }

  function htmlCompose() {
    var c = ps.c, t = c.t, etapes = c.mode === 'rappel' ? [['J-3', 'Rappel (J−3)', 'Courtois']] : [['J+1', 'Courtois', 'J+1'], ['J+5', 'Ferme', 'J+5'], ['J+10', 'Dernier avis', 'J+10']];
    var seg = etapes.map(function (e) { return '<input type="radio" name="etape" id="et-' + e[0] + '" value="' + e[0] + '"' + (ps.etape === e[0] ? ' checked' : '') + '><label for="et-' + e[0] + '">' + UI.ic('check', 'ic-sm coche') + e[1] + '</label>'; }).join('');
    var eches = c.eches.map(function (e) { return '<li><div class="liste-gauche"><span class="liste-titre">' + F.mois(e.periode) + '</span><span class="liste-sous">Échéance du ' + F.date(e.dateEcheance) + '</span></div><div class="liste-droite"><span class="liste-montant">' + F.montant(e.reste) + '</span>' + (e.statut === 'en_retard' ? UI.statut('en_retard', F.jours(e.ancienneteJours)) : UI.statut(e.statut)) + '</div></li>'; }).join('');
    var h = '<div class="enc-locataire"><span class="avatar avatar-lg" aria-hidden="true">' + F.initiales(t.nom) + '</span><div class="enc-loc-corps"><span class="enc-loc-nom">' + F.e(t.nom) + '</span><div class="enc-loc-meta"><span>' + UI.ic('phone', 'ic-sm') + ' ' + F.e(t.telephone) + '</span><span>' + F.e(c.lot.nom) + ', ' + F.e(c.bien.quartier) + '</span></div></div></div>' +
      '<ul class="liste" style="margin-top:12px">' + eches + '</ul>';
    if (c.dejaAujourdhui) h += '<div class="bandeau bandeau-avert" style="margin-top:12px">' + UI.ic('alert-triangle') + '<span>Une relance a déjà été envoyée à ' + F.e(t.prenom) + ' aujourd’hui à ' + c.dejaAujourdhui.heure + '. Voulez-vous en envoyer une autre ?</span></div>';
    if (!t.whatsapp) h += '<div class="bandeau bandeau-info" style="margin-top:12px">' + UI.ic('info') + '<div><strong>Ce locataire n’a pas de numéro WhatsApp.</strong><br>Ajoutez-le à sa fiche ou relancez par SMS.<div style="margin-top:8px">' + UI.btn('Ajouter le numéro WhatsApp', { v: 'secondaire', petit: true, action: 'demo' }) + '</div></div></div>';
    h += '<fieldset class="champ" style="border:0;padding:0;margin:16px 0 0"><legend class="champ-etiquette" style="padding:0;margin-bottom:6px">Ton du message</legend><div class="segmente segmente-3" id="seg-etape">' + seg + '</div></fieldset>' +
      '<fieldset class="champ" style="border:0;padding:0;margin:16px 0 0"><legend class="champ-etiquette" style="padding:0;margin-bottom:6px">Canal</legend><div class="segmente" id="seg-canal"><input type="radio" name="canal" id="ca-whatsapp" value="whatsapp"' + (ps.canal === 'whatsapp' ? ' checked' : '') + (t.whatsapp ? '' : ' disabled') + '><label for="ca-whatsapp">' + UI.ic('check', 'ic-sm coche') + 'WhatsApp</label><input type="radio" name="canal" id="ca-sms" value="sms"' + (ps.canal === 'sms' ? ' checked' : '') + '><label for="ca-sms">' + UI.ic('check', 'ic-sm coche') + 'SMS</label></div></fieldset>' +
      '<div class="champ"><label class="champ-etiquette" for="rel-texte">Texte du message</label><textarea class="saisie" id="rel-texte" rows="9" aria-describedby="rel-err rel-aide">' + F.e(ps.texte) + '</textarea><div class="champ-aide" id="rel-aide">Le texte est prêt : vous pouvez le modifier. Vous gardez la main sur l’envoi. <button type="button" class="lien-bouton" data-action="rel-variable">Tester une variable inconnue</button></div><div class="champ-erreur cache" id="rel-err" role="alert"></div></div>' +
      '<div style="margin-top:16px">' + UI.btn(ps.canal === 'sms' ? 'Ouvrir les SMS' : 'Ouvrir WhatsApp', { v: 'primaire', grand: true, ic: 'message', action: 'rel-ouvrir', id: 'btn-rel-ouvrir' }) + '</div>';
    return h;
  }
  function htmlConfirmer() {
    return '<div class="etat-vide" style="padding:24px 8px">' + UI.ic('message') + '<h3 style="font-size:var(--text-xl)">Avez-vous envoyé le message ?</h3><p class="secondaire">Vous avez ouvert ' + (ps.canal === 'sms' ? 'les SMS' : 'WhatsApp') + ' avec le texte prêt. Immo Suite n’envoie rien tout seul.</p>' +
      '<div class="btn-groupe" style="justify-content:center;margin-top:8px">' + UI.btn('Oui, message envoyé', { v: 'primaire', ic: 'check', action: 'rel-oui', id: 'btn-rel-oui' }) + UI.btn('Non, pas envoyé', { v: 'secondaire', action: 'rel-non' }) + '</div></div>';
  }
  function htmlResultat() {
    var c = ps.c, loyer = c.bail.loyer + c.bail.charges, datePromesse = F.date(plusJours(D().meta.aujourdhui, 7));
    return '<div class="bandeau bandeau-succes">' + UI.ic('check-circle') + '<span><strong>Message envoyé.</strong> Notez le résultat pour que rien ne soit perdu.</span></div>' +
      '<div class="champ"><label style="display:flex;gap:10px;align-items:center;min-height:44px"><input type="checkbox" id="rel-promesse"><span class="champ-etiquette">Le locataire a promis de payer</span></label></div>' +
      '<div id="bloc-promesse" class="cache"><div class="champ"><label class="champ-etiquette" for="rel-date">Date promise (JJ/MM/AAAA)</label><input class="saisie" id="rel-date" value="' + datePromesse + '" inputmode="numeric" autocomplete="off" aria-describedby="rel-date-err"><div class="champ-erreur cache" id="rel-date-err" role="alert"></div></div>' +
      '<div class="champ"><label class="champ-etiquette" for="rel-montant">Montant promis</label><div class="saisie-montant" style="min-height:48px"><input id="rel-montant" inputmode="numeric" style="font-size:var(--text-lg)" value="' + F.nombre(loyer) + '"><span class="suffixe">FCFA</span></div></div></div>' +
      '<div class="champ"><label class="champ-etiquette" for="rel-action">Prochaine action</label><select class="saisie" id="rel-action"><option>Vérifier le paiement promis</option><option>Relancer le locataire</option><option>Appeler le locataire</option><option>Visiter le locataire</option></select></div>' +
      '<div class="champ"><label class="champ-etiquette" for="rel-action-date">Date de la prochaine action (JJ/MM/AAAA)</label><input class="saisie" id="rel-action-date" value="' + F.date(plusJours(D().meta.aujourdhui, 3)) + '" inputmode="numeric" autocomplete="off" aria-describedby="rel-ad-err"><div class="champ-erreur cache" id="rel-ad-err" role="alert"></div></div>' +
      '<div style="margin-top:16px">' + UI.btn('Enregistrer la relance', { v: 'primaire', grand: true, ic: 'check', action: 'rel-enregistrer', id: 'btn-rel-enreg' }) + '</div>';
  }
  function afficher() {
    var corps = panneau.corps;
    corps.innerHTML = ps.etapeEcran === 'compose' ? htmlCompose() : ps.etapeEcran === 'confirmer' ? htmlConfirmer() : htmlResultat();
    var titre = panneau.el.querySelector('#panneau-titre'); titre.textContent = ps.c.mode === 'rappel' ? 'Rappeler' : 'Relancer';
    if (ps.etapeEcran === 'compose') {
      corps.querySelectorAll('input[name="etape"]').forEach(function (i) { i.addEventListener('change', function () { ps.etape = i.value; ps.texte = rendreTexte(ps.c, ps.etape); corps.querySelector('#rel-texte').value = ps.texte; }); });
      corps.querySelectorAll('input[name="canal"]').forEach(function (i) { i.addEventListener('change', function () { ps.canal = i.value; corps.querySelector('#btn-rel-ouvrir span').textContent = ps.canal === 'sms' ? 'Ouvrir les SMS' : 'Ouvrir WhatsApp'; }); });
      corps.querySelector('#rel-texte').addEventListener('input', function (e) { ps.texte = e.target.value; });
      corps.querySelector('#btn-rel-ouvrir').focus();
    } else if (ps.etapeEcran === 'confirmer') corps.querySelector('#btn-rel-oui').focus();
    else {
      var cb = corps.querySelector('#rel-promesse'), bloc = corps.querySelector('#bloc-promesse'), act = corps.querySelector('#rel-action');
      cb.addEventListener('change', function () { bloc.classList.toggle('cache', !cb.checked); act.selectedIndex = cb.checked ? 0 : 1; corps.querySelector('#rel-action-date').value = cb.checked ? corps.querySelector('#rel-date').value : F.date(plusJours(D().meta.aujourdhui, 3)); });
      corps.querySelector('#rel-date').addEventListener('input', function (e) { if (cb.checked) corps.querySelector('#rel-action-date').value = e.target.value; });
      act.selectedIndex = 1; cb.focus();
    }
  }

  function ouvrir(bailId, mode, echeanceId) {
    var c = contexte(bailId, mode || 'retard', echeanceId), etape = etapeConseillee(c), t = c.t;
    ps = { c: c, etape: etape, canal: t.whatsapp ? 'whatsapp' : 'sms', texte: '', etapeEcran: 'compose', envoye: false };
    ps.texte = rendreTexte(c, etape);
    panneau = UI.panneau(c.mode === 'rappel' ? 'Rappeler' : 'Relancer', '', { surFermeture: function () { panneau = null; } });
    afficher();
  }
  window.Relance = { ouvrir: ouvrir };

  App.action('imp-relancer', function (el, ev) { ouvrir(el.getAttribute('data-bail'), 'retard'); });
  App.action('rel-variable', function () { var a = document.getElementById('rel-texte'); a.value = a.value.replace(/\s+$/, '') + ' {loyer}'; ps.texte = a.value; a.focus(); });
  App.action('rel-ouvrir', function () {
    var a = document.getElementById('rel-texte'), err = document.getElementById('rel-err'); ps.texte = a.value;
    var inconnues = []; (ps.texte.match(/\{(\w+)\}/g) || []).forEach(function (m) { var k = m.slice(1, -1); if (VARIABLES.indexOf(k) < 0 && inconnues.indexOf(m) < 0) inconnues.push(m); });
    if (inconnues.length) { err.classList.remove('cache'); err.innerHTML = UI.ic('alert-circle', 'ic-sm') + '<span>La variable ' + F.e(inconnues[0]) + ' n’existe pas. Variables disponibles : ' + VARIABLES.map(function (v) { return '{' + v + '}'; }).join(', ') + '.</span>'; a.setAttribute('aria-invalid', 'true'); a.focus(); return; }
    err.classList.add('cache'); a.setAttribute('aria-invalid', 'false');
    var t = ps.c.t;
    UI.messagerie({ canal: ps.canal, nom: t.nom, telephone: t.telephone, texte: ps.texte, surEnvoi: function () { ps.envoye = true; }, surRetour: function () { if (panneau) { ps.etapeEcran = 'confirmer'; afficher(); } } });
  });
  App.action('rel-non', function () { ps.etapeEcran = 'compose'; afficher(); });
  App.action('rel-oui', function () { ps.etapeEcran = 'resultat'; afficher(); });
  App.action('rel-enregistrer', function () {
    var c = ps.c, corps = panneau.corps, promesse = corps.querySelector('#rel-promesse').checked, ok = true;
    var err = function (id, txt) { var e = corps.querySelector(id); e.classList.toggle('cache', !txt); e.innerHTML = txt ? UI.ic('alert-circle', 'ic-sm') + '<span>' + txt + '</span>' : ''; if (txt) ok = false; };
    var jour = D().meta.aujourdhui, pr = null, pa;
    err('#rel-date-err', ''); err('#rel-ad-err', '');
    if (promesse) {
      var di = dateIso(corps.querySelector('#rel-date').value), mt = parseInt(corps.querySelector('#rel-montant').value.replace(/\D/g, ''), 10);
      if (!di) err('#rel-date-err', 'Saisissez une date au format JJ/MM/AAAA.'); else if (di <= jour) err('#rel-date-err', 'La date promise doit être postérieure à aujourd’hui.');
      if (!mt || mt <= 0) { UI.toast('Saisissez un montant promis supérieur à 0.', { type: 'avert' }); ok = false; }
      if (ok) pr = { date: di, montant: mt };
    }
    var da = dateIso(corps.querySelector('#rel-action-date').value); if (!da) err('#rel-ad-err', 'Saisissez une date au format JJ/MM/AAAA.'); else if (da < jour) err('#rel-ad-err', 'La prochaine action ne peut pas être dans le passé.');
    if (!ok) return;
    pa = { date: da, libelle: corps.querySelector('#rel-action').value };
    var ton = modele(ps.etape).ton;
    Mut.appliquer('relance', { org: orgCle(), bailId: c.bailId, etape: ps.etape, ton: ton, canal: ps.canal, texte: ps.texte, promesse: pr, prochaineAction: pa, echeanceId: c.echeanceId, auteurId: App.ctx.utilisateur.id });
    var p = panneau; panneau = null; p.fermer();
    UI.toast('Relance enregistrée. Prochaine action : ' + F.date(pa.date) + '.', { type: 'succes' });
    if (pr) UI.toast('Promesse de paiement notée pour le ' + F.date(pr.date) + '.', { type: 'succes' });
    if (App.ctx.route.nom === 'impayes') { majPage(); var l = document.querySelectorAll('[data-ligne="' + c.bailId + '"]'); l.forEach(function (x) { x.classList.add('ligne-maj'); }); } else App.rendre();
  });

  /* ---------- Écran S05 ---------- */
  App.action('imp-tranche', function (el) { var t = el.getAttribute('data-t'); f.tranche = f.tranche === t ? '' : t; majPage(); });
  App.action('imp-retirer', function (el) { f[el.getAttribute('data-k')] = ''; syncSelects(); majPage(); });
  App.action('imp-effacer', function () { f = { mandant: '', bien: '', gestionnaire: '', usage: '', tranche: '' }; syncSelects(); majPage(); });
  function syncSelects() { [['f-mandant', 'mandant'], ['f-bien', 'bien'], ['f-gest', 'gestionnaire'], ['f-usage', 'usage']].forEach(function (x) { var e = document.getElementById(x[0]); if (e) e.value = f[x[1]]; }); }

  App.enregistrer('impayes', {
    meta: function (ctx) {
      var lecture = window.Roles.droit(ctx.role.id, 'impayes') === 'L';
      return { titre: 'Impayés', sous: lecture ? 'Lecture seule : vous voyez les impayés sans pouvoir relancer.' : 'Ce qui est dû, depuis quand, et ce qui a déjà été dit.', repere: { type: 'v1' }, actions: UI.btn('Exporter', { v: 'tertiaire', ic: 'download', action: 'demo' }) };
    },
    rendre: function (ctx) {
      f = { mandant: '', bien: '', gestionnaire: '', usage: '', tranche: '' };
      var gs = filtrer(groupes());
      return '<div id="imp-resume">' + htmlResume() + '</div><section class="carte" aria-labelledby="t-imp-l"><h2 id="t-imp-l" class="sr-only">Liste des impayés</h2>' + htmlFiltres() + '<div id="imp-puces">' + puces() + '</div><p class="secondaire" style="margin:8px 0 12px" id="imp-compte" aria-live="polite">' + gs.length + ' locataires en retard</p><div id="imp-liste">' + htmlTableau(gs) + '</div></section>';
    },
    apres: function (ctx, rac) {
      [['f-mandant', 'mandant'], ['f-bien', 'bien'], ['f-gest', 'gestionnaire'], ['f-usage', 'usage']].forEach(function (x) { var e = rac.querySelector('#' + x[0]); if (e) e.addEventListener('change', function () { f[x[1]] = e.value; majPage(); }); });
      rac.addEventListener('keydown', function (ev) { if (ev.key === 'Enter' && ev.target.classList && ev.target.classList.contains('ligne-cliquable') && ev.target.getAttribute('data-bail')) ouvrir(ev.target.getAttribute('data-bail'), 'retard'); });
      if (ctx.query.relancer) setTimeout(function () { ouvrir(ctx.query.relancer, ctx.query.mode === 'rappel' ? 'rappel' : 'retard', ctx.query.echeance); }, 0);
    }
  });
})();
