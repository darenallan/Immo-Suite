import { expect, test } from '@playwright/test';

test('la page technique s’affiche, sans défilement horizontal ni police téléchargée', async ({ page }) => {
  const requests: string[] = [];
  page.on('request', (r) => requests.push(r.url()));
  await page.goto('/');
  await expect(page.getByRole('heading', { level: 1, name: 'Immo Suite' })).toBeVisible();
  await expect(page.getByTestId('version')).not.toBeEmpty();
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
  );
  expect(overflow).toBeLessThanOrEqual(0);
  expect(requests.filter((u) => /\.(woff2?|ttf|otf)(\?|$)/.test(u))).toEqual([]);
  expect(requests.every((u) => u.startsWith('http://127.0.0.1:4173'))).toBe(true);
});
