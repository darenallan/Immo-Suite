// Démonstration de l'étape 05 sur la pile réelle (roadmap/etape-05-… §8, adaptée par DP-92) : bien et lots, maison seule,
// locataire et doublon de numéro, fusion de deux fiches, recherche par nom et par numéro. Données fictives (`+226 00 00 …`) ;
// chaque largeur (390 et 1440 px) a ses propres noms et numéros, la base étant commune.
import { expect, test, type Page } from '@playwright/test';

const PASSWORD = process.env['E2E_PASSWORD'] ?? 'un mot de passe correct';

async function login(page: Page, identifier = '+22670123456') {
  await page.goto('/');
  await page.getByLabel('Téléphone ou e-mail').fill(identifier);
  await page.getByLabel('Mot de passe').fill(PASSWORD);
  await page.getByRole('button', { name: 'Se connecter' }).click();
  await expect(page.getByTestId('org-name')).toHaveText('Agence Horizon');
}

test('bien et lots, maison seule, doublon de numéro, fusion, recherche', async ({ page }, info) => {
  const wide = info.project.name.includes('1440');
  const tag = wide ? 'O' : 'T';
  const digit = wide ? '2' : '1';
  await login(page);

  // 1. le bien « Zogona » avec un lot d'habitation « A1 » et une boutique « B1 » (activité autorisée sur la boutique seulement)
  await page.goto('/biens');
  await page.getByRole('button', { name: 'Créer un bien' }).click();
  await page.getByLabel('Nom du bien').fill(`Zogona ${tag}`);
  await page.getByLabel('Quartier', { exact: true }).fill('Zogona');
  await page.getByRole('button', { name: 'Créer', exact: true }).click();
  await expect(page.getByRole('heading', { name: new RegExp(`Zogona ${tag}`) })).toBeVisible();
  await page.getByRole('button', { name: 'Ajouter un lot' }).click();
  await page.getByLabel('Référence', { exact: true }).fill('A1');
  await expect(page.getByLabel('Activité autorisée')).toHaveCount(0);
  await page.getByLabel('Loyer de référence (FCFA)').fill('75 000');
  await page.getByRole('button', { name: 'Créer', exact: true }).click();
  await expect(page.getByRole('link', { name: 'A1' })).toBeVisible();
  await page.getByRole('button', { name: 'Ajouter un lot' }).click();
  await page.getByLabel('Référence', { exact: true }).fill('B1');
  await page.getByLabel('Type', { exact: true }).selectOption('shop');
  await page.getByLabel('Usage', { exact: true }).selectOption('commercial');
  await page.getByLabel('Activité autorisée').fill('Quincaillerie');
  await page.getByRole('button', { name: 'Créer', exact: true }).click();
  await expect(page.getByRole('link', { name: 'B1' })).toBeVisible();
  // une seconde « a1 » dans le même bien : refusée (R-011, casse ignorée)
  await page.getByRole('button', { name: 'Ajouter un lot' }).click();
  await page.getByLabel('Référence', { exact: true }).fill('a1');
  await page.getByRole('button', { name: 'Créer', exact: true }).click();
  await expect(page.getByText('Cette référence est déjà utilisée dans ce bien.').first()).toBeVisible();

  // 2. maison seule « Villa Kaboré » : 1 bien et 1 lot
  await page.goto('/biens');
  await page.getByRole('button', { name: 'Créer un bien' }).click();
  await page.getByLabel('Nom du bien').fill(`Villa Kaboré ${tag}`);
  await page.getByLabel('Maison seule (un bien et un lot)').check();
  await page.getByRole('button', { name: 'Créer', exact: true }).click();
  await expect(page.getByRole('heading', { name: new RegExp(`Villa Kaboré ${tag}`) })).toBeVisible();
  await expect(page.getByRole('link', { name: '1', exact: true })).toBeVisible();

  // 3. le locataire « Issouf Tapsoba », puis le même numéro : proposition d'ouvrir la fiche existante (DP-92)
  const phone = `00 00 0${digit} 01`;
  await page.goto('/locataires');
  await page.getByRole('button', { name: 'Créer un locataire' }).click();
  await page.getByLabel('Nom', { exact: true }).fill(`Tapsoba ${tag}`);
  await page.getByLabel('Prénom').fill('Issouf');
  await page.getByLabel('Téléphone', { exact: true }).fill(phone);
  await page.getByRole('button', { name: 'Créer', exact: true }).click();
  await expect(page.getByRole('heading', { name: new RegExp(`Issouf Tapsoba ${tag}`) })).toBeVisible();
  const kept = page.url();
  await page.goto('/locataires');
  await page.getByRole('button', { name: 'Créer un locataire' }).click();
  await page.getByLabel('Nom', { exact: true }).fill(`I. Tapsoba ${tag}`);
  await page.getByLabel('Téléphone', { exact: true }).fill(`+226 ${phone}`);
  await page.getByRole('button', { name: 'Créer', exact: true }).click();
  await expect(
    page.getByText(`Un locataire avec ce numéro existe déjà : Issouf Tapsoba ${tag}.`),
  ).toBeVisible();
  // la seconde fiche se crée sans numéro, puis se fusionne dans la première
  await page.getByLabel('Téléphone', { exact: true }).fill('');
  await page.getByRole('button', { name: 'Créer', exact: true }).click();
  await expect(page.getByRole('heading', { name: new RegExp(`I. Tapsoba ${tag}`) })).toBeVisible();
  await page.getByRole('button', { name: 'Fusionner avec une autre fiche' }).click();
  await page.getByLabel('Fiche à conserver').fill(`Issouf Tapsoba ${tag}`);
  await page.getByRole('button', { name: new RegExp(`Issouf Tapsoba ${tag}`) }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Fusionner avec une autre fiche' }).click();
  await expect(page).toHaveURL(kept);

  // 4. recherche par nom puis par numéro
  const search = page.getByLabel('Rechercher');
  for (const term of [`tapsoba ${tag.toLowerCase()}`, `00000${digit}01`]) {
    await search.fill(term);
    await expect(page.getByRole('button', { name: new RegExp(`Issouf Tapsoba ${tag}`) })).toBeVisible();
  }
  await search.fill(`zogona ${tag.toLowerCase()}`);
  await expect(page.getByRole('button', { name: new RegExp(`A1 · Zogona ${tag}`) })).toBeVisible();

  // 5. agent terrain : le gérant lui assigne un seul lot dans S18, l'agent ne voit que ce lot (R-081)
  await page.goto('/utilisateurs');
  const row = page.getByTestId('member').filter({ hasText: 'Agent terrain' });
  await row.getByRole('button', { name: /Lots assignés/ }).click();
  await expect(row.getByLabel(`B1 · Zogona ${tag}`)).toBeVisible();
  for (const box of await row.getByRole('checkbox').all()) await box.uncheck();
  await row.getByLabel(`B1 · Zogona ${tag}`).check();
  await row.getByRole('button', { name: 'Enregistrer' }).click();
  await expect(row.getByRole('button', { name: 'Lots assignés (1)' })).toBeVisible();
  await page.context().clearCookies();
  await login(page, '+22670123462');
  await page.goto('/biens');
  await expect(page.getByRole('link', { name: `Zogona ${tag}` })).toBeVisible();
  await expect(page.getByRole('link', { name: `Villa Kaboré ${tag}` })).toHaveCount(0);
  await page.getByRole('link', { name: `Zogona ${tag}` }).click();
  await expect(page.getByRole('link', { name: 'B1' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'A1' })).toHaveCount(0);
});
