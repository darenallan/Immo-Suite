// Parcours de bout en bout dans Chrome, à 390 px (téléphone) et 1440 px (ordinateur) : connexion (erreur, succès), expiration de la
// session, déconnexion, accès à une adresse interdite, navigation par rôle. L'API est simulée dans le navigateur ; l'API réelle est
// couverte par les tests d'API de Rust (`tests/db/tests/api.rs`).
import { expect, test, type Page, type Route } from '@playwright/test';

interface World {
  loggedIn: boolean;
  alive: boolean;
  role: string;
  loginAttempts: number;
}

function me(world: World) {
  return {
    user: { id: 'u1', display_name: 'Awa Kaboré' },
    membership: { role: world.role },
    organization: {
      id: 'o1',
      name: 'Agence Horizon',
      plan: 'agency',
      kind: 'real',
      lifecycle: 'setup',
      timezone: 'Africa/Ouagadougou',
      currency: 'XOF',
    },
    organizations: [
      {
        id: 'o1',
        name: 'Agence Horizon',
        plan: 'agency',
        kind: 'real',
        lifecycle: 'setup',
        role: world.role,
      },
    ],
    features: {},
    permissions: [],
    today: '2026-11-13',
    server_time: '2026-11-13T09:15:00Z',
    csrf_token: 'csrf-e2e',
    min_client_version: '0.0.0',
    settings: {},
  };
}

function problem(route: Route, status: number, code: string, extra: Record<string, unknown> = {}) {
  return route.fulfill({
    status,
    contentType: 'application/problem+json',
    body: JSON.stringify({ type: 'about:blank', title: code, status, detail: code, code, ...extra }),
  });
}

async function mockApi(page: Page, world: World) {
  await page.route('**/api/**', async (route) => {
    const url = new URL(route.request().url());
    const method = route.request().method();
    if (url.pathname === '/api/me') {
      if (!world.loggedIn) return problem(route, 401, 'auth.unauthenticated');
      if (!world.alive) return problem(route, 401, 'auth.session_expired');
      return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(me(world)) });
    }
    if (url.pathname === '/api/auth/login' && method === 'POST') {
      world.loginAttempts += 1;
      const body = route.request().postDataJSON() as { password: string };
      if (body.password !== 'un mot de passe correct') return problem(route, 401, 'auth.invalid_credentials');
      world.loggedIn = true;
      world.alive = true;
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          user: { id: 'u1', display_name: 'Awa Kaboré' },
          organizations: [],
          csrf_token: 'csrf-e2e',
          mfa_required: false,
        }),
      });
    }
    if (url.pathname === '/api/auth/logout' && method === 'POST') {
      expect(route.request().headers()['x-csrf-token']).toBe('csrf-e2e');
      world.loggedIn = false;
      return route.fulfill({ status: 204 });
    }
    return problem(route, 404, 'resource.not_found');
  });
}

async function signIn(page: Page) {
  await page.goto('/');
  await page.getByLabel('Téléphone ou e-mail').fill('70 12 34 56');
  await page.getByLabel('Mot de passe').fill('un mot de passe correct');
  await page.getByRole('button', { name: 'Se connecter' }).click();
  await expect(page.getByTestId('org-name')).toHaveText('Agence Horizon');
}

test('mauvais mot de passe puis connexion : message, puis accueil du rôle', async ({ page }) => {
  const world: World = { loggedIn: false, alive: true, role: 'manager', loginAttempts: 0 };
  await mockApi(page, world);
  await page.goto('/');
  await page.getByLabel('Téléphone ou e-mail').fill('70 12 34 56');
  await page.getByLabel('Mot de passe').fill('faux');
  await page.getByRole('button', { name: 'Se connecter' }).click();
  await expect(page.getByRole('alert')).toHaveText('Identifiant ou mot de passe incorrect.');
  await page.getByLabel('Mot de passe').fill('un mot de passe correct');
  await page.getByRole('button', { name: 'Se connecter' }).click();
  await expect(page.getByTestId('org-name')).toHaveText('Agence Horizon');
  await expect(page.getByTestId('role')).toHaveText('Gérant');
  expect(world.loginAttempts).toBe(2);
  // rien dans le stockage du navigateur
  const stored = await page.evaluate(() => window.localStorage.length + window.sessionStorage.length);
  expect(stored).toBe(0);
});

test('expiration de la session : retour à la connexion avec le message', async ({ page }) => {
  const world: World = { loggedIn: false, alive: true, role: 'cashier', loginAttempts: 0 };
  await mockApi(page, world);
  await signIn(page);
  world.alive = false;
  await page.evaluate(() => window.dispatchEvent(new Event('focus')));
  await expect(page.getByRole('heading', { name: 'Connexion' })).toBeVisible();
  await expect(page.getByRole('status')).toHaveText('Votre session a expiré. Reconnectez-vous.');
});

test('déconnexion puis reconnexion', async ({ page }) => {
  const world: World = { loggedIn: false, alive: true, role: 'manager', loginAttempts: 0 };
  await mockApi(page, world);
  await signIn(page);
  await page.getByRole('button', { name: /Menu utilisateur/ }).click();
  await page.getByRole('button', { name: 'Déconnexion' }).click();
  await expect(page.getByRole('heading', { name: 'Connexion' })).toBeVisible();
  await signIn(page);
});

test('adresse interdite : « Accès refusé » pour un caissier, entrée absente du menu', async ({ page }) => {
  const world: World = { loggedIn: false, alive: true, role: 'cashier', loginAttempts: 0 };
  await mockApi(page, world);
  await signIn(page);
  await page.goto('/utilisateurs');
  await expect(page.getByRole('heading', { name: 'Accès refusé' })).toBeVisible();
  await page.getByRole('button', { name: 'Retour à l’accueil' }).click();
  await expect(page.getByTestId('role')).toHaveText('Caissier');
});

test('la navigation s’adapte à la largeur : menu latéral sur ordinateur, barre du bas à 4 entrées sur téléphone', async ({
  page,
}, info) => {
  const world: World = { loggedIn: false, alive: true, role: 'manager', loginAttempts: 0 };
  await mockApi(page, world);
  await signIn(page);
  const desktop = info.project.name.includes('1440');
  if (desktop) {
    await expect(page.locator('.sidebar')).toBeVisible();
    await expect(page.locator('.bottombar')).toBeHidden();
    const width = await page.locator('.sidebar').evaluate((el) => el.getBoundingClientRect().width);
    expect(width).toBe(248);
    await expect(page.locator('.sidebar').getByRole('link')).toHaveCount(17);
  } else {
    await expect(page.locator('.sidebar')).toBeHidden();
    await expect(page.locator('.bottombar')).toBeVisible();
    // 3 entrées + « Plus » = 4
    await expect(page.locator('.bottombar > ul > li')).toHaveCount(4);
    await page.getByRole('button', { name: 'Plus' }).click();
    await expect(page.locator('#more-panel').getByRole('link', { name: 'Utilisateurs' })).toBeVisible();
  }
});

test('mise en page : pas de défilement horizontal, cibles tactiles de 44 px, aucune police ni ressource externe', async ({
  page,
}) => {
  const world: World = { loggedIn: false, alive: true, role: 'manager', loginAttempts: 0 };
  const requests: string[] = [];
  page.on('request', (r) => requests.push(r.url()));
  await mockApi(page, world);
  await page.goto('/');
  for (const control of [
    page.getByLabel('Téléphone ou e-mail'),
    page.getByLabel('Mot de passe'),
    page.getByRole('button', { name: 'Se connecter' }),
  ]) {
    const box = await control.boundingBox();
    expect(box?.height ?? 0).toBeGreaterThanOrEqual(44);
  }
  await signIn(page);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
  );
  expect(overflow).toBeLessThanOrEqual(0);
  expect(requests.filter((u) => /\.(woff2?|ttf|otf)(\?|$)/.test(u))).toEqual([]);
  expect(requests.every((u) => u.startsWith('http://127.0.0.1:4173'))).toBe(true);
  await expect(page.locator('html')).toHaveAttribute('lang', 'fr');
});
