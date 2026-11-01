// Parcours sur la pile réelle (étape 02, démonstration) : mêmes comptes que `ops/dev/real-stack.ps1`.
//   - gérant  +22670123456 « Awa Kaboré »  (Agence Horizon)
//   - caissier +22670123457 « Issa Ouédraogo » (Agence Horizon)
//   - bloqués +22670123458 et +22670123459 (pour la limitation, un par largeur ; jamais utilisés ailleurs)
import { expect, test } from '@playwright/test';

const PASSWORD = process.env['E2E_PASSWORD'] ?? 'un mot de passe correct';
const SHORT_IDLE = process.env['E2E_SHORT_IDLE'] === '1';

async function fillLogin(page: import('@playwright/test').Page, identifier: string, password: string) {
  await page.goto('/');
  await page.getByLabel('Téléphone ou e-mail').fill(identifier);
  await page.getByLabel('Mot de passe').fill(password);
  await page.getByRole('button', { name: 'Se connecter' }).click();
}

test('le gérant se connecte : accueil de son organisation, cookie conforme, en-têtes de sécurité', async ({
  page,
  context,
}) => {
  await fillLogin(page, '70 12 34 56', 'mauvais mot de passe');
  await expect(page.getByRole('alert')).toHaveText('Identifiant ou mot de passe incorrect.');
  await page.getByLabel('Mot de passe').fill(PASSWORD);
  await page.getByRole('button', { name: 'Se connecter' }).click();
  await expect(page.getByTestId('org-name')).toHaveText('Agence Horizon');
  await expect(page.getByTestId('role')).toHaveText('Gérant');

  const cookies = await context.cookies();
  const session = cookies.find((c) => c.name === '__Host-session');
  expect(session, 'cookie __Host-session posé').toBeDefined();
  expect(session?.httpOnly).toBe(true);
  expect(session?.secure).toBe(true);
  expect(session?.sameSite).toBe('Lax');
  expect(session?.path).toBe('/');
  expect(session?.value).toMatch(/^[0-9a-f]{64}$/);
  // invisible du script de la page
  expect(await page.evaluate(() => document.cookie)).not.toContain('__Host-session');

  // en-têtes de sécurité sur une réponse réelle
  const response = await page.request.get('/api/me');
  expect(response.status()).toBe(200);
  const h = response.headers();
  expect(h['cache-control']).toBe('no-store');
  expect(h['x-content-type-options']).toBe('nosniff');
  expect(h['referrer-policy']).toBe('no-referrer');
  expect(h['content-security-policy']).toContain("default-src 'self'");
  expect(h['x-request-id']).toMatch(/^INC-[0-9A-Z]{6}$/);
  const me = (await response.json()) as {
    membership: { role: string };
    permissions: string[];
    today: string;
  };
  expect(me.membership.role).toBe('manager');
  expect(me.permissions).toHaveLength(25);
  expect(me.today).toMatch(/^\d{4}-\d{2}-\d{2}$/);

  // aucune inscription publique
  const register = await page.request.post('/api/auth/register', { data: {} });
  expect(register.status()).toBe(404);
});

test('une écriture sans jeton anti-falsification est refusée par le vrai serveur', async ({ page }) => {
  await fillLogin(page, '70 12 34 56', PASSWORD);
  await expect(page.getByTestId('org-name')).toBeVisible();
  const refused = await page.request.post('/api/auth/logout-all', {
    headers: { Origin: 'http://127.0.0.1:4173' },
  });
  expect(refused.status()).toBe(403);
  expect(((await refused.json()) as { code: string }).code).toBe('auth.csrf_invalid');
  const foreign = await page.request.post('/api/auth/logout-all', {
    headers: { Origin: 'https://pirate.example', 'X-CSRF-Token': 'x' },
  });
  expect(foreign.status()).toBe(403);
  // la session est restée valide
  expect((await page.request.get('/api/me')).status()).toBe(200);
});

test('le caissier : menu réduit, « Accès refusé » sur une adresse interdite, déconnexion effective', async ({
  page,
}) => {
  await fillLogin(page, '70 12 34 57', PASSWORD);
  await expect(page.getByTestId('role')).toHaveText('Caissier');
  await page.goto('/utilisateurs');
  await expect(page.getByRole('heading', { name: 'Accès refusé' })).toBeVisible();
  // la fiche d'organisation est lisible (settings.read partiel), la matrice côté serveur le confirme
  const org = await page.request.get('/api/organization');
  expect(org.status()).toBe(200);
  await page.getByRole('button', { name: 'Retour à l’accueil' }).click();
  await page.getByRole('button', { name: /Menu utilisateur/ }).click();
  await page.getByRole('button', { name: 'Déconnexion' }).click();
  await expect(page.getByRole('heading', { name: 'Connexion' })).toBeVisible();
  expect((await page.request.get('/api/me')).status()).toBe(401);
  await page.reload();
  await expect(page.getByRole('heading', { name: 'Connexion' })).toBeVisible();
});

test('cinq échecs : la sixième tentative reçoit le message de limitation du serveur', async ({
  page,
}, info) => {
  // un compte bloqué par projet (390 px, 1440 px) : la limitation survit d'un projet à l'autre
  const locked = info.project.name.includes('1440') ? '70 12 34 59' : '70 12 34 58';
  for (let i = 0; i < 5; i += 1) {
    await fillLogin(page, locked, 'faux faux faux');
    await expect(page.getByRole('alert')).toHaveText('Identifiant ou mot de passe incorrect.');
  }
  await page.getByLabel('Mot de passe').fill(PASSWORD);
  await page.getByRole('button', { name: 'Se connecter' }).click();
  // le temps restant diminue avec la latence des cinq essais : « 1 minute » à l'instant exact, « 59 secondes » une seconde plus tard
  await expect(page.getByRole('alert')).toHaveText(
    /^Trop de tentatives\. Réessayez dans (1 minute|\d+ secondes?) ou réinitialisez votre mot de passe\.$/,
  );
});

test('expiration d’inactivité raccourcie (essai) : retour à la connexion avec le message', async ({
  page,
}) => {
  test.skip(!SHORT_IDLE, 'serveur lancé avec SESSION_IDLE_MINUTES=1 (E2E_SHORT_IDLE=1)');
  await fillLogin(page, '70 12 34 56', PASSWORD);
  await expect(page.getByTestId('org-name')).toBeVisible();
  // aucune activité pendant plus d'une minute : le contrôle périodique du front ne prolonge pas la session par lui-même, car il
  // interroge /api/me, qui prolonge. On coupe donc le contrôle en bloquant /api/me côté navigateur pendant l'attente.
  await page.route('**/api/me', (route) => route.abort());
  await page.waitForTimeout(65_000);
  await page.unroute('**/api/me');
  await page.evaluate(() => window.dispatchEvent(new Event('focus')));
  await expect(page.getByRole('heading', { name: 'Connexion' })).toBeVisible();
  await expect(page.getByRole('status')).toHaveText('Votre session a expiré. Reconnectez-vous.');
});
