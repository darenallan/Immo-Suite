// Démonstration de l'étape 06 sur la pile réelle (roadmap/etape-06-… §8 ; F07, F08, DP-94) : bail de 75 000 au 5 avec un dépôt
// de 200 000 (avertissement), activation et pastilles de l'échéancier, avenant à 80 000, chevauchement refusé avec les dates,
// lot en travaux refusé à l'activation. Horloge réelle : les dates se calculent à partir d'aujourd'hui (UTC = Ouagadougou).
// Chaque largeur (390 et 1440 px) a son propre bien, la base étant commune. Données fictives.
import { expect, test, type Page } from '@playwright/test';

const PASSWORD = process.env['E2E_PASSWORD'] ?? 'un mot de passe correct';

async function login(page: Page) {
  await page.goto('/');
  await page.getByLabel('Téléphone ou e-mail').fill('+22670123456');
  await page.getByLabel('Mot de passe').fill(PASSWORD);
  await page.getByRole('button', { name: 'Se connecter' }).click();
  await expect(page.getByTestId('org-name')).toHaveText('Agence Horizon');
}

/** Premier jour du mois décalé de `shift` mois, au format AAAA-MM-JJ. */
function firstOfMonth(shift: number): string {
  const now = new Date();
  const d = new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth() + shift, 1));
  return d.toISOString().slice(0, 10);
}

function fr(iso: string): string {
  const [y, m, d] = iso.split('-');
  return `${d ?? ''}/${m ?? ''}/${y ?? ''}`;
}

/** Assistant S10 : lot, locataire, conditions, puis le bouton final (`Activer` ou `Enregistrer le brouillon`). */
async function wizard(
  page: Page,
  unitLabel: string,
  tenant: string,
  start: string,
  deposit: string,
  final: 'Activer' | 'Enregistrer le brouillon',
) {
  await page.goto('/baux');
  await page.getByRole('button', { name: 'Créer un bail' }).click();
  await expect(page.getByText('Étape 1 sur 3')).toBeVisible();
  await page.getByLabel('Lot').selectOption({ label: unitLabel });
  await page.getByLabel(tenant, { exact: true }).check();
  await page.getByRole('button', { name: 'Suivant' }).click();
  await expect(page.getByText('Étape 2 sur 3')).toBeVisible();
  await page.getByLabel('Date de début').fill(start);
  await page.getByLabel('Loyer (FCFA)').fill('75 000');
  await page.getByLabel('Jour d’échéance (1 à 31)').fill('5');
  if (deposit !== '') await page.getByLabel('Dépôt de garantie (FCFA)').fill(deposit);
  await page.getByRole('button', { name: 'Suivant' }).click();
  await expect(page.getByText('Étape 3 sur 3')).toBeVisible();
  await page.getByRole('button', { name: final }).click();
}

test('bail au 5, dépôt élevé, échéancier, avenant, chevauchement, lot en travaux', async ({ page }, info) => {
  const tag = info.project.name.includes('1440') ? 'O' : 'T';
  const property = `Gounghin ${tag}`;
  const tenant = `Awa Sawadogo ${tag}`;
  const start = firstOfMonth(0);
  const previous = firstOfMonth(-1);
  await login(page);

  // préparation : un bien avec deux lots d'habitation (A1 et B1) et un locataire
  await page.goto('/biens');
  await page.getByRole('button', { name: 'Créer un bien' }).click();
  await page.getByLabel('Nom du bien').fill(property);
  await page.getByRole('button', { name: 'Créer', exact: true }).click();
  await expect(page.getByRole('heading', { name: new RegExp(property) })).toBeVisible();
  for (const ref of ['A1', 'B1']) {
    await page.getByRole('button', { name: 'Ajouter un lot' }).click();
    await page.getByLabel('Référence', { exact: true }).fill(ref);
    await page.getByLabel('Loyer de référence (FCFA)').fill('75 000');
    await page.getByRole('button', { name: 'Créer', exact: true }).click();
    await expect(page.getByRole('link', { name: ref, exact: true })).toBeVisible();
  }
  const propertyUrl = page.url();
  await page.goto('/locataires');
  await page.getByRole('button', { name: 'Créer un locataire' }).click();
  await page.getByLabel('Nom', { exact: true }).fill(`Sawadogo ${tag}`);
  await page.getByLabel('Prénom').fill('Awa');
  await page.getByRole('button', { name: 'Créer', exact: true }).click();
  await expect(page.getByRole('heading', { name: new RegExp(tenant) })).toBeVisible();

  // 1. bail de 75 000 au 5, dépôt de 200 000 : avertissement non bloquant, activation, pastilles calculées par le serveur
  await wizard(page, `A1 · ${property}`, tenant, start, '200 000', 'Activer');
  await expect(
    page.getByRole('heading', { name: new RegExp(`Bail · A1 · ${property}.*Actif`) }),
  ).toBeVisible();
  await expect(
    page.getByText('Le dépôt de garantie dépasse deux mois de loyer (avertissement, non bloquant).'),
  ).toBeVisible();
  // première échéance : le 5 du mois de début, pastille selon la date du jour (calculée par le serveur)
  const current = page.getByTestId('installment').filter({ hasText: `05/${fr(start).slice(3)}` });
  await expect(current).toContainText('75 000 FCFA');
  await expect(current).toContainText(/À venir|Due aujourd’hui|En retard/);
  // le mois précédent se saisit en arriéré daté (F08 règle 6, R-100, R-036) : il est en retard
  const arrearsDue = `${previous.slice(0, 8)}05`;
  await page.getByLabel('Nature').selectOption('arrears');
  await page.getByLabel('Libellé').fill('Loyer du mois précédent');
  await page.getByLabel('Montant (FCFA)').fill('75 000');
  await page.getByLabel('Date', { exact: true }).fill(arrearsDue);
  await page.getByRole('button', { name: 'Ajouter la ligne' }).click();
  const late = page.getByTestId('installment').filter({ hasText: fr(arrearsDue) });
  await expect(late).toContainText('75 000 FCFA');
  await expect(late).toContainText(/En retard · \d+ jours/);
  const leaseUrl = page.url();

  // 2. avenant à 80 000 à partir du mois prochain : le loyer en vigueur reste 75 000 aujourd'hui (R-104)
  const next = firstOfMonth(1);
  await page.getByLabel('Date d’effet').fill(next);
  await page.getByLabel('Nouveau loyer (FCFA)').fill('80 000');
  await page.getByRole('button', { name: 'Ajouter l’avenant' }).click();
  const amendments = page.getByRole('list', { name: 'Avenants' });
  await expect(amendments).toContainText(`à partir du ${fr(next)}`);
  await expect(amendments).toContainText('loyer 80 000 FCFA');
  await expect(page.getByText(/loyer en vigueur 75 000 FCFA/)).toBeVisible();

  // 3. un second bail sur A1 aux mêmes dates : le brouillon s’enregistre, l’activation est refusée avec les dates du bail existant
  // (l’exclusion porte sur les baux actifs, en préavis ou terminés, F07)
  await wizard(page, `A1 · ${property}`, tenant, start, '', 'Activer');
  await expect(
    page.getByText(
      `Ce lot a déjà un bail du ${fr(start)} au sans date de fin. Terminez-le ou choisissez d’autres dates. Le brouillon est enregistré.`,
    ),
  ).toBeVisible();

  // 4. B1 mis en travaux depuis sa fiche : l'activation d'un bail sur B1 est refusée, le brouillon est gardé
  await page.goto(propertyUrl);
  await page.getByRole('link', { name: 'B1', exact: true }).click();
  await page.getByRole('button', { name: 'Mettre en travaux' }).click();
  await expect(page.getByRole('button', { name: 'Retirer « en travaux »' })).toBeVisible();
  await wizard(page, `B1 · ${property}`, tenant, start, '', 'Activer');
  await expect(page.getByText(/Ce lot est en travaux.*Le brouillon est enregistré\./)).toBeVisible();
  await page.getByRole('button', { name: 'Ouvrir le brouillon' }).click();
  await expect(
    page.getByRole('heading', { name: new RegExp(`Bail · B1 · ${property}.*Brouillon`) }),
  ).toBeVisible();

  // le bail de A1 est toujours actif
  await page.goto(leaseUrl);
  await expect(
    page.getByRole('heading', { name: new RegExp(`Bail · A1 · ${property}.*Actif`) }),
  ).toBeVisible();
});
