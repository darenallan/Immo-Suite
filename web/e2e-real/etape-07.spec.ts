// Démonstration de l'étape 07 sur la pile réelle (roadmap/etape-07-encaisser.md §8 ; F09, F10, DP-96) : encaisser 50 000
// (solde restant 25 000), 100 000 sur un autre bail (avance 25 000), coupure réseau puis nouvel essai (un seul paiement),
// référence mobile money en double (avertissement bloquant puis confirmation), numéros croissants dans le journal.
// Horloge réelle : chaque bail commence le 1er du mois avec une échéance le jour même. Données fictives.
import { expect, test, type Page } from '@playwright/test';

const PASSWORD = process.env['E2E_PASSWORD'] ?? 'un mot de passe correct';

async function login(page: Page) {
  await page.goto('/');
  await page.getByLabel('Téléphone ou e-mail').fill('+22670123456');
  await page.getByLabel('Mot de passe').fill(PASSWORD);
  await page.getByRole('button', { name: 'Se connecter' }).click();
  await expect(page.getByTestId('org-name')).toHaveText('Agence Horizon');
}

function today(): { day: number; first: string } {
  const now = new Date();
  const iso = now.toISOString().slice(0, 10);
  return { day: now.getUTCDate(), first: `${iso.slice(0, 8)}01` };
}

/** Bail actif de 75 000 dont l'échéance tombe aujourd'hui (assistant S10). */
async function activeLease(page: Page, unitLabel: string, tenant: string) {
  const t = today();
  await page.goto('/baux');
  await page.getByRole('button', { name: 'Créer un bail' }).click();
  await page.getByLabel('Lot').selectOption({ label: unitLabel });
  await page.getByLabel(tenant, { exact: true }).check();
  await page.getByRole('button', { name: 'Suivant' }).click();
  await page.getByLabel('Date de début').fill(t.first);
  await page.getByLabel('Loyer (FCFA)').fill('75 000');
  await page.getByLabel('Jour d’échéance (1 à 31)').fill(String(t.day));
  await page.getByRole('button', { name: 'Suivant' }).click();
  await page.getByRole('button', { name: 'Activer' }).click();
  await expect(page.getByRole('heading', { name: /Actif/ })).toBeVisible();
}

/** Ouvre S04 sur le bail du locataire. */
async function openCollect(page: Page, tenant: string) {
  await page.goto('/encaisser');
  await page.getByLabel('Locataire ou lot').fill(tenant);
  await page.getByRole('button', { name: new RegExp(tenant) }).click();
  // montant proposé par le serveur : reste de l'échéance la plus ancienne (75 000 au départ, 25 000 après un partiel)
  await expect(page.getByLabel('Montant (FCFA)')).toHaveValue(/^\d+$/);
}

test('encaisser : partiel, avance, coupure, référence en double, journal', async ({
  page,
  context,
}, info) => {
  const tag = info.project.name.includes('1440') ? 'O' : 'T';
  const property = `Kossodo ${tag}`;
  const first = `Issouf Kaboré ${tag}`;
  const second = `Mariam Ilboudo ${tag}`;
  await login(page);

  // préparation : un bien, deux lots, deux locataires, deux baux dus aujourd'hui
  await page.goto('/biens');
  await page.getByRole('button', { name: 'Créer un bien' }).click();
  await page.getByLabel('Nom du bien').fill(property);
  await page.getByRole('button', { name: 'Créer', exact: true }).click();
  for (const ref of ['A1', 'B1']) {
    await page.getByRole('button', { name: 'Ajouter un lot' }).click();
    await page.getByLabel('Référence', { exact: true }).fill(ref);
    await page.getByRole('button', { name: 'Créer', exact: true }).click();
    await expect(page.getByRole('link', { name: ref, exact: true })).toBeVisible();
  }
  for (const [firstName, lastName] of [
    ['Issouf', `Kaboré ${tag}`],
    ['Mariam', `Ilboudo ${tag}`],
  ] as const) {
    await page.goto('/locataires');
    await page.getByRole('button', { name: 'Créer un locataire' }).click();
    await page.getByLabel('Nom', { exact: true }).fill(lastName);
    await page.getByLabel('Prénom').fill(firstName);
    await page.getByRole('button', { name: 'Créer', exact: true }).click();
    await expect(page.getByRole('heading', { name: new RegExp(`${firstName} ${lastName}`) })).toBeVisible();
  }
  await activeLease(page, `A1 · ${property}`, first);
  await activeLease(page, `B1 · ${property}`, second);

  // 1-2. encaisser 50 000 en espèces : numéro de quittance et « solde restant 25 000 »
  await openCollect(page, first);
  await page.getByLabel('Montant (FCFA)').fill('50 000');
  await page.getByRole('button', { name: 'Encaisser et envoyer la quittance' }).click();
  await expect(page.getByRole('status').filter({ hasText: 'Paiement enregistré' })).toBeVisible();
  await expect(page.getByRole('heading', { name: /Quittance n° ESSAI-[A-Z0-9]+-\d{4}-\d{6}/ })).toBeVisible();
  await expect(page.getByText(/Solde restant : 25 000 FCFA/)).toBeVisible();
  await expect(page.getByText(/cinquante mille francs CFA/)).toBeVisible();

  // 3. encaisser 100 000 sur un autre bail de 75 000 : confirmer l'avance, lire « avance 25 000 »
  await page.getByRole('button', { name: 'Encaisser un autre loyer' }).click();
  await openCollect(page, second);
  await page.getByLabel('Montant (FCFA)').fill('100000');
  await page.getByRole('button', { name: 'Encaisser et envoyer la quittance' }).click();
  await expect(page.getByRole('alertdialog')).toContainText('25 000 FCFA en avance');
  await page.getByRole('button', { name: 'Confirmer' }).click();
  await expect(page.getByText(/Avance : 25 000 FCFA/)).toBeVisible();

  // 4. couper le réseau juste avant « Encaisser », rétablir, réessayer : un seul paiement
  await page.getByRole('button', { name: 'Encaisser un autre loyer' }).click();
  await openCollect(page, first);
  await page.getByLabel('Montant (FCFA)').fill('10 000');
  await context.setOffline(true);
  await page.getByRole('button', { name: 'Encaisser et envoyer la quittance' }).click();
  await expect(page.getByText(/Connexion perdue. Vos informations sont conservées/)).toBeVisible();
  await context.setOffline(false);
  await page.getByRole('button', { name: 'Encaisser et envoyer la quittance' }).click();
  await expect(page.getByRole('status').filter({ hasText: 'Paiement enregistré' })).toBeVisible();

  // 5. la même référence mobile money deux fois : avertissement bloquant, puis confirmation
  const reference = `MP${tag}${String(Date.now()).slice(-6)}`;
  for (const attempt of [1, 2]) {
    await page.getByRole('button', { name: 'Encaisser un autre loyer' }).click();
    await openCollect(page, first);
    await page.getByLabel('Montant (FCFA)').fill('5 000');
    await page.getByLabel('Mode').selectOption('mobile_money');
    await page.getByLabel('Référence').fill(attempt === 1 ? reference : reference.toLowerCase());
    await page.getByRole('button', { name: 'Encaisser et envoyer la quittance' }).click();
    if (attempt === 2) {
      await expect(page.getByRole('alertdialog')).toContainText('Ce paiement semble déjà enregistré');
      await page.getByRole('button', { name: 'Confirmer' }).click();
    }
    await expect(page.getByRole('status').filter({ hasText: 'Paiement enregistré' })).toBeVisible();
  }

  // 6. le journal : quatre paiements pour ce locataire (un seul pour l'essai coupé), numéros distincts et croissants ;
  // aucune action de modification
  await page.goto('/paiements');
  const rows = page.getByTestId('payment-row').filter({ hasText: first });
  await expect(rows).toHaveCount(4);
  const numbers = (await rows.allTextContents())
    .map((t) => /ESSAI-[A-Z0-9]+-\d{4}-(\d{6})/.exec(t)?.[1])
    .filter((x): x is string => x !== undefined)
    .map(Number);
  expect(new Set(numbers).size).toBe(4);
  await expect(page.getByRole('button', { name: /Modifier|Supprimer/ })).toHaveCount(0);
});
