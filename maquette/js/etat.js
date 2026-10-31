/*
 * État de démonstration : rôle courant, options de démonstration et journal des modifications faites pendant la démo.
 * Stockage dans sessionStorage, toujours entouré de try/catch (la maquette fonctionne sans, y compris en file://).
 * Le journal est rejoué au chargement (voir mutations.js). « Réinitialiser la démo » l'efface et recharge la page :
 * le jeu de données revient exactement à l'état initial.
 */
(function () {
  var CLE = 'immo-suite-maquette-v1', CLE_MSG = 'immo-suite-maquette-message';
  function defaut() { return { role: 'gerant', journal: [], options: { latence: true, erreur: null, lotSansMandat: false, tournage: false, cadre: 'libre' } }; }

  function lire() {
    try { var s = window.sessionStorage.getItem(CLE); return s ? JSON.parse(s) : null; } catch (e) { return null; }
  }
  var Etat = {
    donnees: defaut(),
    charger: function () {
      var s = lire();
      if (s && s.role) { var d = defaut(); this.donnees = { role: s.role, journal: s.journal || [], options: Object.assign(d.options, s.options || {}) }; }
    },
    enregistrer: function () {
      try { window.sessionStorage.setItem(CLE, JSON.stringify(this.donnees)); } catch (e) { /* stockage indisponible : on continue sans */ }
    },
    reinitialiser: function () {
      this.donnees = defaut();
      try { window.sessionStorage.removeItem(CLE); window.sessionStorage.setItem(CLE_MSG, 'reinit'); } catch (e) { /* ignoré */ }
    },
    // Message à afficher après le rechargement (réinitialisation)
    messageApresChargement: function () {
      try { var m = window.sessionStorage.getItem(CLE_MSG); window.sessionStorage.removeItem(CLE_MSG); return m; } catch (e) { return null; }
    },
    role: function () { return this.donnees.role; },
    definirRole: function (r) { this.donnees.role = r; this.enregistrer(); },
    orgCle: function () { return window.Roles.def(this.donnees.role).org; },
    option: function (k) { return this.donnees.options[k]; },
    definirOption: function (k, v) { this.donnees.options[k] = v; this.enregistrer(); },
    consigner: function (nom, params) { this.donnees.journal.push({ nom: nom, params: params }); this.enregistrer(); }
  };
  window.Etat = Etat;
})();
