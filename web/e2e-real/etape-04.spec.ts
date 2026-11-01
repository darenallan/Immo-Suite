// Démonstration de l'étape 04 sur la pile réelle (roadmap/etape-04-… §8) : « Signaler un problème », alerte, erreur serveur
// agrégée, erreur JavaScript, écrans consultés. La base est lue avec le psql du poste (E2E_PSQL, E2E_DB_URL) ; aucun secret ici.
import { execFileSync } from 'node:child_process';
import { expect, test, type Page } from '@playwright/test';

const PASSWORD = process.env['E2E_PASSWORD'] ?? 'un mot de passe correct';
const PSQL = process.env['E2E_PSQL'] ?? 'psql';
const DB_URL = process.env['E2E_DB_URL'] ?? '';

/** Une valeur lue en base (une seule colonne, une seule ligne). */
function sql(query: string): string {
  return execFileSync(PSQL, [DB_URL, '-At', '-X', '-q', '-c', query], { encoding: 'utf8' }).trim();
}

async function login(page: Page) {
  await page.goto('/');
  await page.getByLabel('Téléphone ou e-mail').fill('+22670123456');
  await page.getByLabel('Mot de passe').fill(PASSWORD);
  await page.getByRole('button', { name: 'Se connecter' }).click();
  await expect(page.getByTestId('org-name')).toHaveText('Agence Horizon');
}

test('signalement, alerte, erreur serveur agrégée, erreur JavaScript, écrans consultés', async ({ page }) => {
  await login(page);

  // 1. « Signaler un problème » depuis le menu, défaut fictif, référence affichée
  await page.getByRole('button', { name: /Menu utilisateur/ }).click();
  await page.getByRole('button', { name: 'Signaler un problème' }).click();
  const dialog = page.getByRole('dialog', { name: 'Signaler un problème' });
  await expect(dialog).toBeVisible();
  // résumé d'erreur si la description est trop courte
  await dialog.getByLabel('Décrivez ce qui s’est passé').fill('abc');
  await dialog.getByRole('button', { name: 'Envoyer' }).click();
  await expect(dialog.getByRole('alert')).toContainText('au moins 5 caractères');
  await dialog.getByLabel('Une question').check();
  await dialog
    .getByLabel('Décrivez ce qui s’est passé')
    .fill('Défaut fictif de démonstration : le bouton ne répond pas.');
  await dialog.getByLabel('Vous pouvez me rappeler').check();
  await dialog.getByRole('button', { name: 'Envoyer' }).click();
  const toast = page.getByRole('status').filter({ hasText: 'Merci. Votre référence' });
  await expect(toast).toBeVisible();
  const reference = /SIG-\d{6}/.exec((await toast.textContent()) ?? '')?.[0] ?? '';
  expect(reference).toMatch(/^SIG-\d{6}$/);

  // 2. la ligne est en base (catégorie, rappel, contexte sans paramètre) et l'alerte part (travailleur)
  expect(
    sql(`SELECT category || '/' || callback_ok FROM feedback_reports WHERE reference = '${reference}'`),
  ).toBe('question/true');
  expect(sql(`SELECT context->>'path' FROM feedback_reports WHERE reference = '${reference}'`)).toBe('/');
  await expect
    .poll(
      () =>
        sql(
          `SELECT count(*) FROM platform_events WHERE action = 'feedback.received' AND detail->>'reference' = '${reference}'`,
        ),
      {
        timeout: 30_000,
      },
    )
    .toBe('1');
  // l'alerte ne contient ni la description ni un nom
  expect(
    sql(
      `SELECT detail::text FROM platform_events WHERE action = 'feedback.received' AND detail->>'reference' = '${reference}'`,
    ),
  ).not.toMatch(/Défaut fictif|Awa|Horizon/);

  // 3. erreur serveur provoquée deux fois : une seule ligne, le nombre d'occurrences monte de 2
  const before = Number(
    sql(`SELECT coalesce(sum(occurrences), 0) FROM error_reports WHERE route = '/api/_test/server-error'`),
  );
  for (let i = 0; i < 2; i += 1) {
    const status = await page.evaluate(async () => (await fetch('/api/_test/server-error')).status);
    expect(status).toBe(500);
  }
  await expect
    .poll(() =>
      Number(
        sql(
          `SELECT coalesce(sum(occurrences), 0) FROM error_reports WHERE route = '/api/_test/server-error'`,
        ),
      ),
    )
    .toBe(before + 2);
  expect(sql(`SELECT count(*) FROM error_reports WHERE route = '/api/_test/server-error'`)).toBe('1');

  // 4. erreur JavaScript provoquée : agrégée, sans pile
  const clientBefore = Number(sql(`SELECT count(*) FROM product_events WHERE name = 'client_error'`));
  await page.evaluate(() => {
    setTimeout(() => {
      throw new TypeError('erreur provoquée par la démonstration');
    }, 0);
  });
  await expect
    .poll(() => Number(sql(`SELECT count(*) FROM product_events WHERE name = 'client_error'`)))
    .toBe(clientBefore + 1);
  expect(sql(`SELECT count(*) FROM error_reports WHERE source = 'client' AND sample ? 'stack'`)).toBe('0');

  // 5. les écrans consultés partent à la mise en arrière-plan de la page
  await page.evaluate(() => {
    Object.defineProperty(document, 'visibilityState', { configurable: true, get: () => 'hidden' });
    document.dispatchEvent(new Event('visibilitychange'));
  });
  await expect
    .poll(() =>
      Number(
        sql(
          `SELECT count(*) FROM product_events WHERE name = 'screen_viewed' AND properties->>'screen' = 'S03'`,
        ),
      ),
    )
    .toBeGreaterThan(0);
  // aucun nom, téléphone ni jeton dans les événements
  expect(sql(`SELECT count(*) FROM product_events WHERE properties::text ~ '(\\+226|@|[0-9]{6})'`)).toBe('0');
});
