#!/usr/bin/env node
/*
 * Vérification de cohérence du jeu de données fictives de la maquette Immo Suite.
 *
 * Usage : node scripts/verifier-donnees.js   (depuis le dossier maquette/)
 *
 * Ce script ne partage aucun code avec l'interface. Il recalcule indépendamment les
 * échéances, les affectations, les commissions et les tranches à partir des règles de
 * DOMAINE.md, puis les compare aux valeurs stockées dans data/*.js.
 * Il se termine avec le code 1 si un contrôle échoue.
 */
'use strict';
const fs = require('fs');
const path = require('path');
const vm = require('vm');

// ---------- chargement des données (scripts classiques qui remplissent window.DEMO) ----------
const DOSSIER = path.join(__dirname, '..', 'data');
const bac = { window: {} };
vm.createContext(bac);
for (const f of fs.readdirSync(DOSSIER).filter(n => n.endsWith('.js')).sort()) {
  vm.runInContext(fs.readFileSync(path.join(DOSSIER, f), 'utf8'), bac, { filename: f });
}
const D = bac.window.DEMO;

// ---------- outils de date (jours entiers, UTC : pas de fuseau ni d'heure d'été) ----------
const jour = s => Date.UTC(+s.slice(0, 4), +s.slice(5, 7) - 1, +s.slice(8, 10)) / 864e5;
const ecart = (a, b) => jour(a) - jour(b);
const plus = (s, n) => new Date((jour(s) + n) * 864e5).toISOString().slice(0, 10);
const moisDe = s => s.slice(0, 7);

// ---------- règles recalculées ----------
// R-095, R-033, DP-13 : statut d'une échéance à la date de démonstration
function statutAttendu(montant, paye, dateEcheance, auj) {
  if (montant - paye === 0) return 'payee';
  if (auj < dateEcheance) return paye > 0 ? 'partielle' : 'a_venir';
  if (auj === dateEcheance) return paye > 0 ? 'partielle' : 'due';
  return 'en_retard';
}
// R-058, DP-12 : tranches d'ancienneté
function trancheDe(j) { return j <= 7 ? '1-7' : j <= 30 ? '8-30' : j <= 59 ? '31-59' : '60+'; }
// R-068, DP-14 : taux (en dixièmes de pour cent entiers) x base, arrondi au plus proche (0,5 vers le haut)
function commissionDe(taux, base) { return Math.floor((Math.round(taux * 10) * base + 500) / 1000); }

// ---------- cadre de contrôle ----------
const resultats = [];
function controle(n, titre, regle, fn) {
  let erreurs = [];
  try { erreurs = fn() || []; } catch (e) { erreurs = ['Exception : ' + e.message]; }
  resultats.push({ n, titre, regle, erreurs });
}
const ORGS = ['hor', 'saw'];
const parOrg = (fn) => () => ORGS.flatMap(k => (fn(D[k], k) || []).map(m => `[${k}] ${m}`));
const trouve = (liste, id) => liste.find(x => x.id === id);
const AUJ = D.hor.meta.aujourdhui;

// 1 : montants entiers
controle(1, 'Tous les montants sont des entiers', 'R-001', parOrg((o) => {
  const e = [];
  const cles = /^(montant|loyer|charges|reste|paye|affecte|du|net|commission|encaisse|depotGarantie|cout|creditBail|attendu|compte|ecart|soldeRestant|loyerReference|loyerAvant|loyerApres|chargesAvant|chargesApres|depensesTotal|seuilConfirmationPaiement|tauxRecouvrement)$/;
  (function marcher(v, chemin) {
    if (Array.isArray(v)) v.forEach((x, i) => marcher(x, chemin + '[' + i + ']'));
    else if (v && typeof v === 'object') for (const k of Object.keys(v)) {
      if (typeof v[k] === 'number' && cles.test(k) && !Number.isInteger(v[k])) e.push(`${chemin}.${k} = ${v[k]} n'est pas entier`);
      marcher(v[k], chemin + '.' + k);
    }
  })(o, 'org');
  return e;
}));

// 2 : paiements (dates, montants, références uniques)
controle(2, 'Paiements : aucune date future, montants positifs, références uniques par mode', 'R-039, R-037, R-041', parOrg((o) => {
  const e = [], vues = new Set();
  for (const p of o.paiements) {
    if (p.date > AUJ || p.dateSaisie > AUJ) e.push(`${p.id} daté dans le futur`);
    if (p.date < o.meta.debutUsage) e.push(`${p.id} antérieur au début d'usage`);
    if (p.type === 'paiement' && !(p.montant > 0)) e.push(`${p.id} montant non positif`);
    if (p.type === 'paiement' && p.reference) { const c = p.mode + '|' + p.reference; if (vues.has(c)) e.push(`${p.id} référence en doublon`); vues.add(c); }
    if (p.type === 'paiement' && p.mode !== 'especes' && !p.reference) e.push(`${p.id} sans référence externe`);
  }
  return e;
}));

// 3 : volumes
controle(3, 'Volumes : 40 lots (10 commerciaux ou mixtes), 4 mandants, 31 locataires, 33 baux en cours (dont 1 en préavis)', 'DP-39', () => {
  const o = D.hor, e = [];
  const enCours = o.baux.filter(b => b.statut === 'actif' || b.statut === 'preavis');
  const att = (nom, v, cible) => { if (v !== cible) e.push(`${nom} : ${v} au lieu de ${cible}`); };
  att('lots', o.lots.length, 40); att('lots commerciaux ou mixtes', o.lots.filter(l => l.usage !== 'habitation').length, 10);
  att('lots commerciaux', o.lots.filter(l => l.usage === 'commercial').length, 9); att('lots mixtes', o.lots.filter(l => l.usage === 'mixte').length, 1);
  att('lots habitation', o.lots.filter(l => l.usage === 'habitation').length, 30);
  att('mandants', o.mandants.length, 4); att('locataires', o.locataires.length, 31);
  att('baux en cours', enCours.length, 33); att('baux en préavis', o.baux.filter(b => b.statut === 'preavis').length, 1);
  att('lots occupés', o.lots.filter(l => l.statut === 'occupe').length, 33); att('lots vacants', o.lots.filter(l => l.statut === 'vacant').length, 5);
  att('lots en travaux', o.lots.filter(l => l.statut === 'travaux').length, 2); att('biens', o.biens.length, 22);
  const parM = { M1: 7, M2: 12, M3: 11, M4: 10 };
  for (const m of Object.keys(parM)) att('lots de ' + m, o.lots.filter(l => l.mandantId === m).length, parM[m]);
  const tauxAttendus = { M1: [8, 'loyer'], M2: [7.5, 'loyer'], M3: [10, 'loyer_charges'], M4: [6, 'loyer'] };
  for (const mt of o.mandats) { const t = tauxAttendus[mt.mandantId]; if (mt.taux !== t[0] || mt.base !== t[1]) e.push('mandat ' + mt.id + ' : taux ou base inattendu'); }
  // locataires distincts + colocation + double bail
  const compte = {}; o.baux.forEach(b => b.locataireIds.forEach(t => { compte[t] = (compte[t] || 0) + 1; }));
  att('locataires avec deux baux', Object.values(compte).filter(n => n === 2).length, 3);
  att('baux en colocation', o.baux.filter(b => b.locataireIds.length === 2).length, 1);
  att('locataires distincts dans les baux', Object.keys(compte).length, 31);
  // Patrimoine Sawadogo
  const s = D.saw;
  att('[saw] lots', s.lots.length, 12); att('[saw] locataires', s.locataires.length, 10); att('[saw] baux', s.baux.length, 10);
  att('[saw] lots vacants', s.lots.filter(l => l.statut === 'vacant').length, 1); att('[saw] lots en travaux', s.lots.filter(l => l.statut === 'travaux').length, 1);
  att('[saw] boutiques occupées', s.baux.filter(b => b.type === 'commercial').length, 2);
  if (s.mandats.length) e.push('[saw] ne doit avoir aucun mandat');
  if (s.releves.length) e.push('[saw] ne doit avoir aucun compte rendu');
  return e;
});

// 4 : un seul bail en cours par lot
controle(4, 'Un seul bail en cours par lot, sans chevauchement', 'R-020', parOrg((o) => {
  const e = [];
  for (const l of o.lots) {
    const bs = o.baux.filter(b => b.lotId === l.id && (b.statut === 'actif' || b.statut === 'preavis'));
    if (bs.length > 1) e.push(`lot ${l.id} : ${bs.length} baux en cours`);
    for (let i = 0; i < bs.length; i++) for (let j = i + 1; j < bs.length; j++) if (bs[i].dateDebut <= bs[j].dateFin && bs[j].dateDebut <= bs[i].dateFin) e.push(`lot ${l.id} : chevauchement`);
  }
  return e;
}));

// 5 : statut du lot cohérent avec ses baux
controle(5, 'Statut du lot cohérent avec ses baux (occupé = bail en cours ; vacant ou en travaux = aucun)', 'R-013, R-093', parOrg((o) => {
  const e = [];
  for (const l of o.lots) {
    const n = o.baux.filter(b => b.lotId === l.id && (b.statut === 'actif' || b.statut === 'preavis')).length;
    if (l.statut === 'occupe' && n !== 1) e.push(`lot ${l.id} occupé sans bail en cours`);
    if ((l.statut === 'vacant' || l.statut === 'travaux') && n !== 0) e.push(`lot ${l.id} ${l.statut} avec un bail en cours`);
  }
  return e;
}));

// 6 : un seul mandat actif par lot (plan Agence)
controle(6, 'Chaque lot couvert par un seul mandat actif', 'R-014', () => {
  const o = D.hor, e = [];
  for (const l of o.lots) {
    const ms = o.mandats.filter(m => m.statut === 'actif' && m.lotIds.includes(l.id));
    if (ms.length !== 1) e.push(`lot ${l.id} couvert par ${ms.length} mandats`);
    else if (l.mandatId !== ms[0].id) e.push(`lot ${l.id} : mandat du lot différent de celui qui le couvre`);
  }
  return e;
});

// 7 : une échéance par bail et par période
controle(7, 'Une échéance par bail et par période', 'R-027', parOrg((o) => {
  const e = [], vus = new Set();
  for (const x of o.echeances) {
    const c = x.bailId + '|' + x.periode; if (vus.has(c)) e.push('doublon ' + c); vus.add(c);
    const b = trouve(o.baux, x.bailId);
    if (b && x.dateEcheance !== x.periode + '-' + String(b.jourEcheance).padStart(2, '0')) e.push(`${x.id} : date incohérente avec le jour d'échéance`);
    if (x.montant !== x.lignes.reduce((s, l) => s + l.montant, 0)) e.push(`${x.id} : total différent de la somme des lignes`);
    if (x.dateEcheance > plus(AUJ, 30)) e.push(`${x.id} : échéance au-delà des 30 jours générés`);
  }
  return e;
}));

// 8 : affectations
controle(8, "Somme des affectations ≤ montant du paiement, vers des lignes existantes", 'R-044', parOrg((o) => {
  const e = [];
  for (const p of o.paiements.filter(x => x.type === 'paiement')) {
    const s = p.affectations.reduce((t, a) => t + a.montant, 0);
    if (s > p.montant) e.push(`${p.id} : affectations ${s} > paiement ${p.montant}`);
    if (s + p.creditBail !== p.montant) e.push(`${p.id} : affectations + crédit du bail ≠ montant`);
    for (const a of p.affectations) { const ech = trouve(o.echeances, a.echeanceId); if (!ech || !ech.lignes.some(l => l.type === a.ligne)) e.push(`${p.id} : affectation vers une ligne inexistante`); }
  }
  return e;
}));

// Reconstitution indépendante des affectations (utilisée par les contrôles 9, 10 et 11)
function rejouer(o) {
  const reste = {}; // clé échéance|ligne => reste dû
  o.echeances.forEach(x => x.lignes.forEach(l => { reste[x.id + '|' + l.type] = l.montant; }));
  const attendues = {}, credits = {};
  const ordre = o.paiements.filter(p => p.type === 'paiement').slice().sort((a, b) => (a.dateSaisie + a.heure + a.id).localeCompare(b.dateSaisie + b.heure + b.id));
  for (const p of ordre) {
    const cp = o.paiements.find(c => c.type === 'contre-passation' && c.remplaceParId === p.id);
    if (cp) for (const k of Object.keys(attendues[cp.origineId])) reste[k] += attendues[cp.origineId][k];
    let rest = p.montant; attendues[p.id] = {};
    const cand = o.echeances.filter(x => x.bailId === p.bailId && x.dateEcheance <= plus(p.date, 30)).sort((a, b) => a.dateEcheance.localeCompare(b.dateEcheance));
    for (const x of cand) for (const l of x.lignes) {
      const k = x.id + '|' + l.type; if (rest <= 0 || reste[k] <= 0) continue;
      const m = Math.min(rest, reste[k]); reste[k] -= m; rest -= m; attendues[p.id][k] = m;
    }
    credits[p.id] = rest;
  }
  return { reste, attendues, credits };
}

// 9 : statut d'échéance recalculé
controle(9, 'Statut de chaque échéance recalculé à la date de démonstration = statut stocké', 'R-032, R-033, R-095', parOrg((o) => {
  const e = [];
  for (const x of o.echeances) {
    let paye = 0;
    for (const p of o.paiements) if (p.type === 'paiement' && p.statut === 'valide') paye += p.affectations.filter(a => a.echeanceId === x.id).reduce((s, a) => s + a.montant, 0);
    if (x.paye !== paye) e.push(`${x.id} : payé stocké ${x.paye}, recalculé ${paye}`);
    if (x.reste !== x.montant - paye) e.push(`${x.id} : reste stocké ${x.reste}, recalculé ${x.montant - paye}`);
    const st = statutAttendu(x.montant, paye, x.dateEcheance, AUJ);
    if (x.statut !== st) e.push(`${x.id} : statut stocké ${x.statut}, recalculé ${st}`);
  }
  return e;
}));

// 10 : en retard dès J+1, même partielle
controle(10, 'Une échéance au solde non nul est « en retard » dès J+1, même partielle', 'R-033, DP-13', parOrg((o) => {
  const e = [];
  for (const x of o.echeances) {
    if (x.reste > 0 && x.dateEcheance < AUJ && x.statut !== 'en_retard') e.push(`${x.id} : ${x.statut} alors que le retard est de ${ecart(AUJ, x.dateEcheance)} jour(s)`);
    if (x.statut === 'partielle' && x.dateEcheance < AUJ) e.push(`${x.id} : « partielle » après J+1`);
    if (x.statut === 'en_retard' && x.dateEcheance >= AUJ) e.push(`${x.id} : en retard avant J+1`);
  }
  return e;
}));

// 11 : ordre d'affectation
controle(11, "Ordre d'affectation : date d'échéance, puis loyer, charges, lignes exceptionnelles", 'R-087, DP-21', parOrg((o) => {
  const e = [], { attendues, credits } = rejouer(o);
  for (const p of o.paiements.filter(x => x.type === 'paiement')) {
    const stock = {}; p.affectations.forEach(a => { stock[a.echeanceId + '|' + a.ligne] = (stock[a.echeanceId + '|' + a.ligne] || 0) + a.montant; });
    const cles = new Set([...Object.keys(stock), ...Object.keys(attendues[p.id])]);
    for (const k of cles) if ((stock[k] || 0) !== (attendues[p.id][k] || 0)) e.push(`${p.id} : affectation ${k} stockée ${stock[k] || 0}, attendue ${attendues[p.id][k] || 0}`);
    if (p.creditBail !== credits[p.id]) e.push(`${p.id} : crédit du bail stocké ${p.creditBail}, attendu ${credits[p.id]}`);
  }
  return e;
}));

// 12 : quittances
controle(12, 'Une quittance par paiement. Numéros consécutifs, sans trou, par année', 'R-049, R-050', parOrg((o) => {
  const e = [], pays = o.paiements.filter(p => p.type === 'paiement');
  for (const p of pays) { const qs = o.quittances.filter(q => q.paiementId === p.id); if (qs.length !== 1) e.push(`${p.id} : ${qs.length} quittance(s)`); else if (p.quittanceId !== qs[0].id) e.push(`${p.id} : quittanceId incohérent`); }
  for (const q of o.quittances) if (!pays.some(p => p.id === q.paiementId)) e.push(`${q.id} : paiement introuvable`);
  const ordre = pays.slice().sort((a, b) => (a.dateSaisie + a.heure + a.id).localeCompare(b.dateSaisie + b.heure + b.id));
  ordre.forEach((p, i) => {
    const q = o.quittances.find(x => x.paiementId === p.id); if (!q) return;
    const attendu = `${o.meta.prefixe}-${p.dateSaisie.slice(0, 4)}-${String(i + 1).padStart(6, '0')}`;
    if (q.numero !== attendu) e.push(`${p.id} : numéro ${q.numero}, attendu ${attendu}`);
  });
  if (o.meta.prochainNumero !== `${o.meta.prefixe}-2026-${String(pays.length + 1).padStart(6, '0')}`) e.push('prochain numéro incohérent avec la suite');
  for (const c of o.paiements.filter(p => p.type === 'contre-passation')) if (c.quittanceId) e.push(`${c.id} : une contre-passation n'a pas de quittance`);
  return e;
}));

// 13 : contre-passation et quittances
controle(13, 'Quittance d’un paiement contre-passé « Annulée » ; la rectificative du remplacement la référence', 'R-051, R-096, DP-38', () => {
  const o = D.hor, e = [], corriges = o.paiements.filter(p => p.statut === 'corrige');
  if (corriges.length !== 1) e.push(`${corriges.length} paiement(s) corrigé(s), 1 attendu`);
  for (const p of corriges) {
    const qa = o.quittances.find(q => q.paiementId === p.id), cp = o.paiements.find(c => c.type === 'contre-passation' && c.origineId === p.id);
    if (!qa || qa.statut !== 'annulee') e.push(`${p.id} : sa quittance doit être « annulée »`);
    const rep = cp && o.paiements.find(x => x.id === cp.remplaceParId), qr = rep && o.quittances.find(q => q.paiementId === rep.id);
    if (!qr || !qr.rectificative) e.push(`${p.id} : le remplacement doit porter une quittance rectificative`);
    else {
      if (qr.remplaceQuittanceId !== qa.id) e.push('la rectificative ne référence pas la quittance annulée');
      if (qa.remplaceeParQuittanceId !== qr.id) e.push("la quittance annulée ne référence pas sa remplaçante");
      if (qr.numero <= qa.numero) e.push('la rectificative doit avoir un numéro postérieur');
      if (rep.montant !== 75000 || p.montant !== 57000) e.push('cas E-38 : 57 000 puis 75 000 attendus');
    }
  }
  for (const q of o.quittances) if (q.statut === 'annulee' && o.paiements.find(p => p.id === q.paiementId).statut !== 'corrige') e.push(`${q.id} annulée sans contre-passation`);
  return e;
});

// 14 : contre-passation liée à l'origine
controle(14, "Chaque contre-passation est liée à un paiement d'origine, par une écriture inverse", 'R-047, R-048', () => {
  const o = D.hor, e = [];
  for (const c of o.paiements.filter(p => p.type === 'contre-passation')) {
    const orig = trouve(o.paiements, c.origineId);
    if (!orig) { e.push(`${c.id} : origine introuvable`); continue; }
    if (c.montant !== -orig.montant) e.push(`${c.id} : montant non inverse`);
    if (orig.statut !== 'corrige' || orig.contrePassationId !== c.id) e.push(`${c.id} : l'origine n'est pas marquée corrigée`);
    if (c.bailId !== orig.bailId) e.push(`${c.id} : bail différent de l'origine`);
    if (c.recuPar !== 'U1') e.push(`${c.id} : seul le gérant contre-passe`);
  }
  if (o.paiements.filter(p => p.statut === 'corrige').some(p => !o.paiements.some(c => c.origineId === p.id))) e.push('paiement corrigé sans contre-passation');
  for (const d of o.demandesCorrection) if (!trouve(o.paiements, d.paiementId)) e.push(`${d.id} : paiement introuvable`);
  if (o.demandesCorrection.filter(d => d.statut === 'en_attente').length !== 1) e.push('une demande de correction en attente est attendue');
  return e;
});

// 15 : ancienneté et tranches
controle(15, 'Ancienneté et tranches des impayés (1–7, 8–30, 31–59, 60 et +)', 'R-058, DP-12', parOrg((o) => {
  const e = [];
  for (const x of o.echeances) {
    if (x.statut !== 'en_retard') { if (x.ancienneteJours !== undefined) e.push(`${x.id} : ancienneté sur une échéance non en retard`); continue; }
    const j = ecart(AUJ, x.dateEcheance);
    if (x.ancienneteJours !== j) e.push(`${x.id} : ancienneté ${x.ancienneteJours}, attendue ${j}`);
    if (x.tranche !== trancheDe(j)) e.push(`${x.id} : tranche ${x.tranche}, attendue ${trancheDe(j)}`);
    const lignes = o.impayes.filter(i => i.echeanceId === x.id);
    if (lignes.length !== 1) e.push(`${x.id} : ${lignes.length} ligne(s) dans la liste des impayés`);
    else if (lignes[0].reste !== x.reste || lignes[0].tranche !== x.tranche) e.push(`${x.id} : ligne d'impayé incohérente`);
  }
  if (o.impayes.length !== o.echeances.filter(x => x.statut === 'en_retard').length) e.push('la liste des impayés ne correspond pas aux échéances en retard');
  return e;
}));

// Recalcul d'un compte rendu à partir des paiements
function recalculerReleve(o, r) {
  const mandat = o.mandats.find(m => m.mandantId === r.mandantId), lots = o.lots.filter(l => l.mandantId === r.mandantId).map(l => l.id);
  let total = 0; const dates = [];
  for (const p of o.paiements) {
    if (p.type !== 'paiement' || p.statut !== 'valide' || moisDe(p.date) !== r.periode || !lots.includes(p.lotId)) continue;
    const m = p.affectations.filter(a => a.ligne === 'loyer' || (mandat.base === 'loyer_charges' && a.ligne === 'charges')).reduce((s, a) => s + a.montant, 0);
    total += m; if (m) dates.push(p.id);
  }
  const dep = o.interventions.filter(i => i.payeur === 'agence' && i.imputable && i.imputeAuReleveId === r.id).reduce((s, i) => s + i.cout, 0);
  const commission = commissionDe(mandat.taux, total);
  return { total, commission, dep, net: total - commission - dep, lignes: dates.length };
}
// 16 : commissions et nets
controle(16, 'Commission = taux × encaissé (arrondi au plus proche). Net = encaissé − commission − dépenses imputées', 'R-068, R-070, DP-14', () => {
  const o = D.hor, e = [];
  for (const r of o.releves) {
    const c = recalculerReleve(o, r);
    if (r.encaisse !== c.total) e.push(`${r.id} : encaissé ${r.encaisse}, recalculé ${c.total}`);
    if (r.commission !== c.commission) e.push(`${r.id} : commission ${r.commission}, recalculée ${c.commission}`);
    if (r.depensesTotal !== c.dep) e.push(`${r.id} : dépenses ${r.depensesTotal}, recalculées ${c.dep}`);
    if (r.net !== c.net) e.push(`${r.id} : net ${r.net}, recalculé ${c.net}`);
    if (r.lignesLoyers.length !== c.lignes) e.push(`${r.id} : nombre de lignes de loyers incohérent`);
    if (r.lignesLoyers.reduce((s, l) => s + l.montant, 0) !== r.encaisse) e.push(`${r.id} : lignes ≠ total encaissé`);
    if (r.statut === 'paye' && (!r.reversement || r.reversement.montant !== r.net)) e.push(`${r.id} : reversement absent ou différent du net`);
    if (r.statut !== 'paye' && r.reversement) e.push(`${r.id} : reversement sur un compte rendu non payé`);
  }
  // ajustement de démonstration (parcours 3) : dépense supplémentaire, net recalculé
  for (const r of o.releves.filter(x => x.ajustementDemo)) { const a = r.ajustementDemo; if (a.depensesTotal !== r.depensesTotal + a.montant || a.net !== r.encaisse - r.commission - a.depensesTotal) e.push(r.id + ' : ajustement de démonstration incohérent'); }
  // un loyer encaissé ne figure que dans un seul compte rendu (R-072)
  const vus = new Set();
  for (const r of o.releves) for (const l of r.lignesLoyers) { if (vus.has(l.paiementId)) e.push(`paiement ${l.paiementId} dans deux comptes rendus`); vus.add(l.paiementId); }
  return e;
});

// 17 : compte rendu de référence M1 d'octobre
controle(17, 'Compte rendu M1 d’octobre : 400 000 → 32 000 → 338 000', 'E-18', () => {
  const o = D.hor, r = o.releves.find(x => x.id === 'R-M1-2026-10'), e = [];
  if (!r) return ['R-M1-2026-10 absent'];
  const montants = r.lignesLoyers.map(l => l.montant).sort((a, b) => b - a).join(',');
  if (montants !== '90000,85000,75000,60000,50000,40000') e.push('les six loyers de M1 ne sont pas ceux attendus : ' + montants);
  if (r.encaisse !== 400000 || r.commission !== 32000 || r.depensesTotal !== 30000 || r.net !== 338000) e.push(`chiffres : ${r.encaisse} / ${r.commission} / ${r.depensesTotal} / ${r.net}`);
  if (r.statut !== 'brouillon') e.push('statut « brouillon » attendu');
  if (r.impayes.length !== 1 || r.impayes[0].reste !== 45000 || r.impayes[0].locataireId !== 'T07') e.push('impayé attendu : Rasmané Bonkoungou, 45 000');
  const dep = r.depenses[0]; if (r.depenses.length !== 1 || dep.montant !== 30000 || dep.date !== '2026-10-22') e.push('dépense imputée attendue : 30 000 FCFA, 22/10/2026');
  const fuite = trouve(o.interventions, 'I05'); if (!fuite || fuite.lotId !== 'L001' || fuite.payeur !== 'agence') e.push("intervention I05 (fuite A3, payée par l'agence) absente");
  const sept = ['M1', 'M3', 'M4'].map(m => o.releves.find(x => x.id === `R-${m}-2026-09`)), m2 = o.releves.find(x => x.id === 'R-M2-2026-09');
  if (sept.some(x => !x || x.statut !== 'paye')) e.push('septembre : M1, M3, M4 doivent être « payé »');
  if (!m2 || m2.statut !== 'valide') e.push('septembre : M2 doit être « validé »');
  if (o.releves.some(x => x.periode === '2026-10' && x.mandantId !== 'M1' && x.statut !== 'a_produire')) e.push("octobre : seuls les comptes rendus de M1 existent (M2 à M4 « à produire »)");
  if (o.releves.filter(x => x.periode === '2026-10').length !== 4) e.push('octobre : un compte rendu ou un aperçu par mandant attendu');
  return e;
});

// 18 : dépenses imputables
controle(18, 'Dépenses imputables seulement si payées par l’agence (par défaut)', 'R-084, DP-18', parOrg((o) => {
  const e = [];
  for (const i of o.interventions) {
    if (i.imputable && i.payeur !== 'agence') e.push(`${i.id} : imputable alors que payée par ${i.payeur}`);
    if (i.payeur === 'agence' && !i.imputable && !i.horsHistorique) e.push(`${i.id} : payée par l'agence mais non imputable`);
    if (i.imputeAuReleveId && !i.imputable) e.push(`${i.id} : imputée sans être imputable`);
  }
  return e;
}));

// 19 : répartition des statuts de novembre
controle(19, 'Répartition des statuts de novembre dans les tolérances', '—', () => {
  const o = D.hor, e = [], cat = { paye: 0, partiel: 0, avance: 0, retard: 0, due: 0 };
  for (const b of o.baux) {
    const es = o.echeances.filter(x => x.bailId === b.id), en = es.find(x => x.periode === '2026-11');
    if (es.some(x => x.statut === 'en_retard')) cat.retard++;
    else if (es.some(x => x.periode > '2026-11' && x.paye > 0) || b.creditBail > 0) cat.avance++;
    else if (en.statut === 'payee') cat.paye++; else if (en.statut === 'partielle') cat.partiel++; else cat.due++;
  }
  const n = o.baux.length, pc = k => 100 * k / n;
  if (pc(cat.paye) < 66 || pc(cat.paye) > 74) e.push(`payé ${pc(cat.paye).toFixed(1)} % hors 66–74 %`);
  if (pc(cat.partiel + cat.avance) < 10 || pc(cat.partiel + cat.avance) > 18) e.push(`partiel ou avance ${pc(cat.partiel + cat.avance).toFixed(1)} % hors 10–18 %`);
  if (pc(cat.retard) < 12 || pc(cat.retard) > 18) e.push(`en retard ${pc(cat.retard).toFixed(1)} % hors 12–18 %`);
  if (cat.paye !== 23 || cat.partiel !== 2 || cat.avance !== 2 || cat.retard !== 5 || cat.due !== 1) e.push('cible 23 / 2 / 2 / 5 / 1 non atteinte : ' + JSON.stringify(cat));
  const st = new Set(o.echeances.filter(x => x.periode === '2026-11').map(x => x.statut));
  for (const s of ['a_venir', 'due', 'partielle', 'en_retard', 'payee']) if (!st.has(s)) e.push(`aucune échéance « ${s} » en novembre`);
  const jours = new Set(o.baux.map(b => b.jourEcheance));
  for (const j of [1, 4, 5, 10, 13, 14, 15, 20, 24, 25, 30]) if (!jours.has(j)) e.push(`jour d'échéance ${j} absent`);
  const avances = o.baux.filter(b => o.echeances.some(x => x.bailId === b.id && x.periode > '2026-11' && x.paye > 0)), credits = o.baux.filter(b => b.creditBail > 0);
  if (avances.length !== 1 || credits.length !== 1) e.push('une avance affectée et un crédit du bail attendus');
  // modes de paiement
  const pays = o.paiements.filter(p => p.type === 'paiement'), part = m => 100 * pays.filter(p => p.mode === m).length / pays.length;
  if (part('especes') < 52 || part('especes') > 68) e.push(`espèces ${part('especes').toFixed(0)} %`);
  if (part('mobile_money') < 24 || part('mobile_money') > 36) e.push(`mobile money ${part('mobile_money').toFixed(0)} %`);
  // Patrimoine Sawadogo : 7 payés, 1 partiel, 2 en retard
  const s = D.saw, cs = { paye: 0, partiel: 0, retard: 0 };
  for (const b of s.baux) { const es = s.echeances.filter(x => x.bailId === b.id), en = es.find(x => x.periode === '2026-11'); if (es.some(x => x.statut === 'en_retard')) cs.retard++; else if (en.statut === 'payee') cs.paye++; else cs.partiel++; }
  if (cs.paye !== 7 || cs.partiel !== 1 || cs.retard !== 2) e.push('[saw] 7 / 1 / 2 attendu : ' + JSON.stringify(cs));
  return e;
});

// 20 : totaux du tableau de bord
controle(20, 'Totaux du tableau de bord = sommes des listes', '—', parOrg((o) => {
  const e = [], t = o.tableauDeBord;
  const valides = o.paiements.filter(p => p.type === 'paiement' && p.statut === 'valide');
  const enc = valides.filter(p => moisDe(p.date) === t.mois).reduce((s, p) => s + p.montant, 0);
  if (t.encaisse !== enc) e.push(`encaissé ${t.encaisse} ≠ ${enc}`);
  const att = o.echeances.filter(x => x.periode === t.mois).reduce((s, x) => s + x.montant, 0);
  if (t.attendu !== att) e.push(`attendu ${t.attendu} ≠ ${att}`);
  const imp = o.impayes.reduce((s, i) => s + i.reste, 0);
  if (t.enRetard.montant !== imp) e.push(`en retard ${t.enRetard.montant} ≠ ${imp}`);
  if (t.enRetard.echeances !== o.impayes.length) e.push('nombre d’échéances en retard incohérent');
  if (t.enRetard.locataires !== new Set(o.impayes.map(i => i.bailId)).size) e.push('nombre de locataires en retard incohérent');
  for (const k of ['1-7', '8-30', '31-59', '60+']) {
    const l = o.impayes.filter(i => i.tranche === k);
    if (t.parTranche[k].nombre !== l.length || t.parTranche[k].montant !== l.reduce((s, i) => s + i.reste, 0)) e.push(`tranche ${k} incohérente`);
  }
  if (t.lots.total !== o.lots.length || t.lots.occupes !== o.lots.filter(l => l.statut === 'occupe').length) e.push('compteurs de lots incohérents');
  const ajd = valides.filter(p => p.dateSaisie === AUJ);
  if (t.aujourdhui.nombre !== ajd.length || t.aujourdhui.montant !== ajd.reduce((s, p) => s + p.montant, 0)) e.push('encaissements du jour incohérents');
  for (const j of o.caisse) {
    const attendu = valides.filter(p => p.mode === 'especes' && p.recuPar === j.caissierId && p.dateSaisie === j.date).reduce((s, p) => s + p.montant, 0);
    if (j.attendu !== attendu) e.push(`caisse du ${j.date} : attendu ${j.attendu} ≠ ${attendu}`);
    if (j.statut === 'cloturee' && j.ecart !== j.compte - j.attendu) e.push(`caisse du ${j.date} : écart incohérent`);
    if (j.ecart && !j.motif) e.push(`caisse du ${j.date} : écart sans motif`);
  }
  return e;
}));

// 21 : cas de référence
controle(21, 'Cas de référence reproduits : E-05, E-06, E-13, E-15, E-16, E-18, E-20, E-25, E-37', 'DOMAINE.md', () => {
  const o = D.hor, e = [], sc = o.scenarios.encaisser;
  // E-37 : le 4 à venir, le 5 due, le 6 en retard (règle) + Issouf Tapsoba
  if (statutAttendu(75000, 0, '2026-11-05', '2026-11-04') !== 'a_venir' || statutAttendu(75000, 0, '2026-11-05', '2026-11-05') !== 'due' || statutAttendu(75000, 0, '2026-11-05', '2026-11-06') !== 'en_retard') e.push('E-37 : règle J0 / J+1');
  const iss = o.echeances.find(x => x.id === sc.echeanceId); const tIss = trouve(o.locataires, 'T08'), lot = trouve(o.lots, 'L008');
  if (!iss || iss.statut !== 'due' || iss.montant !== 75000 || iss.reste !== 75000 || iss.dateEcheance !== '2026-11-13') e.push('E-37 : échéance d’Issouf Tapsoba (due, 75 000, 13/11/2026)');
  if (tIss.telephone !== '+226 00 00 01 12' || lot.nom !== 'Villa B2' || lot.mandantId !== 'M2') e.push('E-05/E-06 : fiche d’Issouf Tapsoba (Villa B2, M2, +226 00 00 01 12)');
  // E-05 : paiement partiel de 50 000
  const v50 = sc.variantes[50000]; if (v50.soldeRestant !== 25000 || v50.statutApres !== 'partielle' || v50.affectations[0].affecte !== 50000) e.push('E-05 : 50 000 → reste 25 000, partielle');
  // E-06 : avance de 25 000 sur l'échéance suivante
  const v100 = sc.variantes[100000]; const dec = o.echeances.find(x => x.bailId === 'B08' && x.periode === '2026-12');
  if (!v100.avance || v100.avance.montant !== 25000 || v100.affectations[0].affecte !== 75000 || !dec || dec.dateEcheance !== v100.avance.echeanceDate) e.push('E-06 : 100 000 → 75 000 + avance de 25 000 sur le 13/12');
  // E-13 : 3, 20 et 70 jours → trois tranches ; E-25 : Mariam Sanou
  const par = id => o.echeances.find(x => x.id === id);
  const e13 = [['E-B09-2026-11', 3, '1-7'], ['E-B25-2026-10', 20, '8-30'], ['E-B21-2026-09', 70, '60+'], ['E-B21-2026-10', 40, '31-59'], ['E-B21-2026-11', 9, '8-30'], ['E-B17-2026-09', 60, '60+'], ['E-B17-2026-10', 30, '8-30'], ['E-B07-2026-10', 31, '31-59']];
  for (const [id, j, tr] of e13) { const x = par(id); if (!x || x.ancienneteJours !== j || x.tranche !== tr) e.push(`E-13 : ${id} attendu ${j} j (${tr})`); }
  const m = par('E-B09-2026-11'); const bm = trouve(o.baux, 'B09');
  if (m.montant !== 75000 || m.paye !== 50000 || m.reste !== 25000 || m.statut !== 'en_retard' || m.dateEcheance !== '2026-11-10' || trouve(o.locataires, bm.locataireIds[0]).nom !== 'Mariam Sanou') e.push('E-25 : Mariam Sanou 75 000, 50 000 payés, reste 25 000, en retard');
  const pm = o.promesses.find(p => p.id === 'PRM2'); if (!pm || pm.date !== '2026-11-16' || pm.statut !== 'en_cours') e.push('E-25 : promesse au 16/11 en cours');
  const adama = trouve(o.locataires, 'T24'), a = par('E-B25-2026-10');
  if (adama.nom !== 'Adama Ouédraogo' || a.montant !== 60000 || a.dateEcheance !== '2026-10-24' || trouve(o.lots, trouve(o.baux, 'B25').lotId).mandantId !== 'M4') e.push('E-13/E-15 : Adama Ouédraogo 60 000, M4, 24/10');
  // E-15 : promesse au 12 non tenue → alerte dans l'accueil du gestionnaire le 13
  const p1 = o.promesses.find(p => p.id === 'PRM1');
  if (!p1 || p1.date !== '2026-11-12' || p1.statut !== 'non_tenue') e.push('E-15 : promesse du 12/11 non tenue');
  if (!o.agenda.some(x => x.type === 'promesse_non_tenue' && x.date === AUJ && x.objet.id === 'PRM1')) e.push("E-15 : alerte du 13/11 absente de l'agenda");
  // autres ancrages
  const ras = par('E-B07-2026-10'); if (!ras || ras.montant !== 45000 || trouve(o.lots, 'L007').mandantId !== 'M1' || par('E-B07-2026-11').statut !== 'due') e.push('E-13b : Rasmané Bonkoungou');
  const yac = par('E-B17-2026-11'); if (!yac || yac.statut !== 'a_venir' || yac.montant !== 120000 || trouve(o.lots, 'L020').mandantId !== 'M3') e.push('Yacouba Compaoré : 120 000, M3, échéance du 14/11 à venir');
  if (par('E-B21-2026-11').montant !== 35000 || trouve(o.lots, 'L025').mandantId !== 'M3' || trouve(o.biens, trouve(o.lots, 'L025').bienId).quartier !== 'Cissin') e.push('Pauline Yaméogo : 35 000, M3, Cissin');
  // E-16 et E-26 : règle de commission
  if (commissionDe(8, 100000) !== 8000) e.push('E-16 : 8 % de 100 000 ≠ 8 000');
  if (commissionDe(7.5, 333333) !== 25000) e.push('E-26 : 7,5 % de 333 333 ≠ 25 000');
  // E-18 : voir contrôle 17
  const r = o.releves.find(x => x.id === 'R-M1-2026-10'); if (!r || r.net !== 338000) e.push('E-18 : net de M1 ≠ 338 000');
  // E-20 : fichier d'import de 200 lignes, 3 erreurs aux lignes 14, 87 et 152, 2 paires de doublons
  const imp = o.importExemple, errs = imp.lignes.filter(l => l.erreur).map(l => l.ligne).join(',');
  if (imp.total !== 200 || imp.lignes.length !== 200 || imp.valides !== 197 || imp.erreurs !== 3 || errs !== '14,87,152') e.push('E-20 : 200 lignes, 197 valides, erreurs aux lignes 14, 87, 152');
  const tels = {}; imp.lignes.filter(l => !l.erreur).forEach(l => { tels[l.telephone.replace('+226 ', '')] = (tels[l.telephone.replace('+226 ', '')] || 0) + 1; });
  if (Object.values(tels).filter(n => n === 2).length !== 2 || imp.doublons.length !== 2) e.push('E-20 : 2 paires de doublons de téléphone');
  // numérotation (E-10, E-27) : voir contrôle 12
  return e;
});

// 22 : cas limites de l'ancienneté
controle(22, 'Cas limites du calcul d’ancienneté : 30, 31, 59 et 60 jours', 'R-058', () => {
  const e = [];
  const attendu = { 1: '1-7', 7: '1-7', 8: '8-30', 30: '8-30', 31: '31-59', 59: '31-59', 60: '60+', 61: '60+' };
  for (const j of Object.keys(attendu)) if (trancheDe(+j) !== attendu[j]) e.push(`${j} jours : ${trancheDe(+j)}, attendu ${attendu[j]}`);
  const jours = D.hor.impayes.map(i => i.ancienneteJours);
  for (const j of [30, 31, 60]) if (!jours.includes(j)) e.push(`aucun impayé de ${j} jours dans les données`);
  return e;
});

// 23 : téléphones fictifs, aucun e-mail
controle(23, 'Numéros de téléphone au format fictif. Aucune donnée réelle, aucun e-mail', '—', () => {
  const e = [], motif = /^\+226 00 00 0[1-2] \d\d$/;
  (function marcher(v, chemin) {
    if (Array.isArray(v)) v.forEach((x, i) => marcher(x, chemin + '[' + i + ']'));
    else if (v && typeof v === 'object') for (const k of Object.keys(v)) {
      const x = v[k];
      if (k === 'telephone' && typeof x === 'string' && !motif.test(x) && chemin.indexOf('importExemple') < 0) e.push(`${chemin}.${k} = « ${x} » hors du bloc fictif`);
      if (typeof x === 'string' && /@/.test(x)) e.push(`${chemin}.${k} contient « @ »`);
      if (k === 'telephone' && typeof x === 'string' && /\+226 [5-7]\d/.test(x)) e.push(`${chemin}.${k} ressemble à un numéro réel`);
      marcher(x, chemin + '.' + k);
    }
  })(D, 'DEMO');
  const imp = D.hor.importExemple;
  for (const l of imp.lignes) if (!l.erreur && !/^(\+226 )?00 00 0[34] \d\d$/.test(l.telephone)) e.push(`import ligne ${l.ligne} : numéro hors du bloc fictif`);
  return e;
});

// 24 : clés étrangères
controle(24, 'Aucune référence à un identifiant orphelin', '—', parOrg((o) => {
  const e = [], u = o.utilisateurs;
  const ex = (liste, id, quoi) => { if (id !== null && id !== undefined && !liste.some(x => x.id === id)) e.push(`${quoi} → ${id} introuvable`); };
  o.biens.forEach(b => ex(o.mandants, b.mandantId, 'bien ' + b.id + ' mandant'));
  o.lots.forEach(l => { ex(o.biens, l.bienId, 'lot ' + l.id + ' bien'); ex(o.mandants, l.mandantId, 'lot ' + l.id + ' mandant'); if (l.mandatId) ex(o.mandats, l.mandatId, 'lot ' + l.id + ' mandat'); if (l.bailId) ex(o.baux, l.bailId, 'lot ' + l.id + ' bail'); if (l.gestionnaireId) ex(u, l.gestionnaireId, 'lot gestionnaire'); if (l.agentId) ex(u, l.agentId, 'lot agent'); });
  o.baux.forEach(b => { ex(o.lots, b.lotId, 'bail ' + b.id + ' lot'); b.locataireIds.forEach(t => ex(o.locataires, t, 'bail ' + b.id + ' locataire')); });
  o.echeances.forEach(x => ex(o.baux, x.bailId, 'échéance ' + x.id + ' bail'));
  o.paiements.forEach(p => { ex(o.baux, p.bailId, p.id + ' bail'); ex(o.lots, p.lotId, p.id + ' lot'); ex(u, p.recuPar, p.id + ' reçu par'); p.locataireIds.forEach(t => ex(o.locataires, t, p.id + ' locataire')); if (p.payePar && p.payePar.locataireId) ex(o.locataires, p.payePar.locataireId, p.id + ' payé par'); if (p.quittanceId) ex(o.quittances, p.quittanceId, p.id + ' quittance'); });
  o.quittances.forEach(q => { ex(o.paiements, q.paiementId, q.id + ' paiement'); ex(o.baux, q.bailId, q.id + ' bail'); if (q.remplaceQuittanceId) ex(o.quittances, q.remplaceQuittanceId, q.id + ' remplace'); if (q.remplaceeParQuittanceId) ex(o.quittances, q.remplaceeParQuittanceId, q.id + ' remplacée par'); });
  o.relances.forEach(r => { ex(o.baux, r.bailId, r.id + ' bail'); ex(o.locataires, r.locataireId, r.id + ' locataire'); ex(o.echeances, r.echeanceId, r.id + ' échéance'); ex(u, r.auteurId, r.id + ' auteur'); if (r.promesseId) ex(o.promesses, r.promesseId, r.id + ' promesse'); });
  o.promesses.forEach(p => { ex(o.baux, p.bailId, p.id + ' bail'); ex(o.echeances, p.echeanceId, p.id + ' échéance'); ex(u, p.auteurId, p.id + ' auteur'); });
  o.interventions.forEach(i => { ex(o.lots, i.lotId, i.id + ' lot'); ex(o.biens, i.bienId, i.id + ' bien'); ex(o.prestataires, i.prestataireId, i.id + ' prestataire'); i.documentIds.forEach(d => ex(o.documents, d, i.id + ' document')); if (i.imputeAuReleveId) ex(o.releves, i.imputeAuReleveId, i.id + ' compte rendu'); });
  o.documents.forEach(d => { if (d.lotId) ex(o.lots, d.lotId, d.id + ' lot'); d.objets.forEach(ob => { const col = { bail: o.baux, intervention: o.interventions, mandant: o.mandants }[ob.kind]; ex(col, ob.id, d.id + ' objet'); }); });
  o.releves.forEach(r => { ex(o.mandants, r.mandantId, r.id + ' mandant'); r.lignesLoyers.forEach(l => { ex(o.paiements, l.paiementId, r.id + ' paiement'); ex(o.lots, l.lotId, r.id + ' lot'); }); r.depenses.forEach(d => ex(o.interventions, d.interventionId, r.id + ' dépense')); });
  o.impayes.forEach(i => { ex(o.echeances, i.echeanceId, 'impayé échéance'); ex(o.lots, i.lotId, 'impayé lot'); ex(o.locataires, i.locataireId, 'impayé locataire'); if (i.derniereRelanceId) ex(o.relances, i.derniereRelanceId, 'impayé relance'); if (i.promesseId) ex(o.promesses, i.promesseId, 'impayé promesse'); });
  o.rappels.forEach(r => ex(o.echeances, r.echeanceId, 'rappel échéance'));
  o.caisse.forEach(j => j.paiementIds.forEach(id => ex(o.paiements, id, 'caisse paiement')));
  o.audit.forEach(a => ex(u, a.userId, a.id + ' utilisateur'));
  o.agenda.forEach(a => ex(u, a.assigneId, a.id + ' assigné'));
  o.mandats.forEach(m => { ex(o.mandants, m.mandantId, m.id + ' mandant'); m.lotIds.forEach(l => ex(o.lots, l, m.id + ' lot')); });
  return e;
}));

// 25 : date de fin >= date de début
controle(25, 'La date de fin d’un bail ne précède pas sa date de début', 'R-021', parOrg((o) => o.baux.filter(b => b.dateFin < b.dateDebut).map(b => `${b.id} : fin ${b.dateFin} avant début ${b.dateDebut}`)));

// 26 : loyer strictement positif
controle(26, 'Le loyer de chaque bail est strictement positif', 'R-022', parOrg((o) => {
  const e = [];
  for (const b of o.baux) { if (!(b.loyer > 0)) e.push(`${b.id} : loyer ${b.loyer}`); b.avenants.forEach(a => { if (!(a.loyerAvant > 0) || !(a.loyerApres > 0)) e.push(`${b.id} : avenant à loyer non positif`); }); }
  return e;
}));

// 27 : avenants non rétroactifs
controle(27, 'Un avenant n’est jamais rétroactif : les échéances antérieures gardent l’ancien montant', 'R-023', parOrg((o) => {
  const e = [];
  for (const b of o.baux) for (const a of b.avenants) for (const x of o.echeances.filter(y => y.bailId === b.id)) {
    const loyer = x.lignes.find(l => l.type === 'loyer').montant, ch = (x.lignes.find(l => l.type === 'charges') || { montant: 0 }).montant;
    const [l, c] = x.dateEcheance < a.date ? [a.loyerAvant, a.chargesAvant] : [a.loyerApres, a.chargesApres];
    if (loyer !== l || ch !== c) e.push(`${x.id} : ${loyer}/${ch} au lieu de ${l}/${c}`);
  }
  if (o === D.hor && o.baux.filter(b => b.avenants.some(a => a.date === '2026-11-01')).length !== 2) e.push('deux baux avec un avenant au 01/11/2026 attendus');
  return e;
}));

// 28 : dépôt de garantie
controle(28, 'Bail d’habitation dont le dépôt dépasse deux mois de loyer : avertissement non bloquant', 'R-024, E-22', parOrg((o) => {
  const e = [];
  for (const b of o.baux) { const doit = b.type === 'habitation' && b.depotGarantie > 2 * b.loyer; if (doit !== b.depotAlerte) e.push(`${b.id} : avertissement ${b.depotAlerte} au lieu de ${doit}`); }
  if (o === D.hor && o.baux.filter(b => b.depotAlerte).length !== 1) e.push('un bail avec avertissement attendu');
  return e;
}));

// ---------- rapport ----------
let ko = 0;
for (const r of resultats) {
  const ok = r.erreurs.length === 0; if (!ok) ko++;
  console.log(`${ok ? '✔' : '✘'} ${String(r.n).padStart(2)}. ${r.titre}  (${r.regle})`);
  r.erreurs.slice(0, 8).forEach(m => console.log('      - ' + m));
  if (r.erreurs.length > 8) console.log(`      … ${r.erreurs.length - 8} autre(s)`);
}
console.log(`\n${resultats.length - ko} contrôle(s) réussi(s) sur ${resultats.length}. ${ko ? ko + ' en échec.' : 'Tout est cohérent.'}`);
process.exit(ko ? 1 : 0);
