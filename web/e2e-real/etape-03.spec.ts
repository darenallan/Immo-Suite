// Démonstration de l'étape 03 sur la pile réelle (roadmap/etape-03-… §8) : mêmes comptes que `ops/dev/real-stack.ps1`.
//   - gérant « Awa Kaboré »            +22670123456 (Agence Horizon)
//   - gérants de sécurité (un par projet, pour la double authentification et l'accès support) : +22670123460 et +22670123461
// Chaque projet (390 px, 1440 px) invite sa propre personne : les numéros d'invités sont distincts.
import { createHmac } from 'node:crypto';
import { expect, test, type Browser, type Page } from '@playwright/test';

const PASSWORD = process.env['E2E_PASSWORD'] ?? 'un mot de passe correct';
const BASE = process.env['E2E_BASE_URL'] ?? 'http://127.0.0.1:4173';

async function login(page: Page, identifier: string, password: string) {
  await page.goto('/');
  await page.getByLabel('Téléphone ou e-mail').fill(identifier);
  await page.getByLabel('Mot de passe').fill(password);
  await page.getByRole('button', { name: 'Se connecter' }).click();
}

/** Ouvre une seconde personne dans un contexte à part (un second téléphone). */
async function secondPhone(browser: Browser, viewport: { width: number; height: number }) {
  const context = await browser.newContext({ viewport, baseURL: BASE });
  return { context, page: await context.newPage() };
}

function base32Decode(text: string): Buffer {
  const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
  let bits = 0;
  let value = 0;
  const out: number[] = [];
  for (const c of text.replace(/\s/g, '').toUpperCase()) {
    value = (value << 5) | alphabet.indexOf(c);
    bits += 5;
    if (bits >= 8) {
      bits -= 8;
      out.push((value >> bits) & 0xff);
    }
  }
  return Buffer.from(out);
}

/** Code à 6 chiffres (RFC 6238, SHA-1, 30 s) pour le pas `step`. */
function totp(secret: Buffer, step: number): string {
  const counter = Buffer.alloc(8);
  counter.writeBigUInt64BE(BigInt(step));
  const mac = createHmac('sha1', secret).update(counter).digest();
  const offset = (mac[19] ?? 0) & 0x0f;
  const word = (mac.readUInt32BE(offset) & 0x7fffffff) % 1_000_000;
  return String(word).padStart(6, '0');
}

test('invitation, premier mot de passe, changement de rôle sans reconnexion, liens, double authentification, accès support, journal', async ({
  page,
  browser,
}, info) => {
  const small = info.project.name.includes('390');
  const viewport = small ? { width: 390, height: 844 } : { width: 1440, height: 900 };
  const invitee = small ? '70 20 00 01' : '70 20 00 02';
  const guard = small ? '70 12 34 60' : '70 12 34 61';
  const name = small ? 'Fatou Sawadogo' : 'Salif Zongo';

  // ── 1. le gérant invite un caissier par son numéro et copie le lien ──
  await login(page, '70 12 34 56', PASSWORD);
  await expect(page.getByTestId('role')).toHaveText('Gérant');
  await page.goto('/utilisateurs');
  await page.getByLabel('Téléphone ou e-mail').fill(invitee);
  await page.locator('select[name="invite-role"]').selectOption('cashier');
  await page.getByRole('button', { name: 'Créer le lien' }).click();
  const link = await page.getByTestId('shown-link').innerText();
  expect(link).toMatch(/\/invitation\/[A-Za-z0-9_-]{43}$/);
  const token = link.slice(link.lastIndexOf('/') + 1);
  await expect(page.getByRole('link', { name: 'WhatsApp' })).toHaveAttribute(
    'href',
    /^https:\/\/wa\.me\/\?text=/,
  );

  // ── 2. second téléphone : ouvrir le lien, choisir le mot de passe, n'avoir que les entrées permises ──
  const other = await secondPhone(browser, viewport);
  await other.page.goto(`/invitation/${token}`);
  await expect(
    other.page.getByText(/Vous êtes invité à rejoindre Agence Horizon comme Caissier/),
  ).toBeVisible();
  expect(new URL(other.page.url()).pathname, 'le jeton quitte l’adresse').toBe('/');
  await other.page.getByLabel('Votre nom').fill(name);
  await other.page.getByLabel(/Choisissez un mot de passe/).fill('Un-bon-mot-de-passe-2026');
  await other.page.getByRole('button', { name: 'Rejoindre' }).click();
  await expect(other.page.getByTestId('role')).toHaveText('Caissier');
  await other.page.goto('/utilisateurs');
  await expect(other.page.getByRole('heading', { name: 'Accès refusé' })).toBeVisible();
  await other.page.goto('/baux');
  await expect(other.page.getByRole('heading', { name: 'Accès refusé' })).toBeVisible();
  // le lien ne sert qu'une fois
  const again = await secondPhone(browser, viewport);
  await again.page.goto(`/invitation/${token}`);
  await expect(again.page.getByRole('alert')).toHaveText(
    'Cette invitation n’est plus valable. Demandez au gérant de la renvoyer.',
  );
  await again.context.close();

  // ── 3. le gérant passe le caissier gestionnaire (confirmation du mot de passe) : le menu change sans reconnexion ──
  await page.goto('/utilisateurs');
  const row = page.getByTestId('member').filter({ hasText: name });
  await row.locator('select').selectOption('property_manager');
  const dialog = page.getByRole('dialog', { name: 'Confirmez votre mot de passe' });
  await expect(dialog).toBeVisible();
  await dialog.getByLabel('Mot de passe').fill(PASSWORD);
  await dialog.getByRole('button', { name: 'Confirmer' }).click();
  await expect(dialog).toBeHidden();
  await expect(row.locator('.tag')).toHaveText('Gestionnaire'); // l'étiquette du rôle, pas la liste déroulante
  await other.page.goto('/baux'); // rechargement du second téléphone : même session
  await expect(other.page.getByRole('heading', { name: 'Baux' })).toBeVisible();
  await expect(other.page.getByRole('heading', { name: 'Accès refusé' })).toBeHidden();

  // ── 4. impossible de retirer le seul gérant (règle côté serveur, vérifiée sur la route) ──
  const me = (await (await page.request.get('/api/me')).json()) as { csrf_token: string };
  const members = (await (await page.request.get('/api/users')).json()) as {
    users: { membership_id: string; is_self: boolean }[];
  };
  const self = members.users.find((u) => u.is_self);
  const refusal = await page.request.post(`/api/users/${self?.membership_id ?? ''}/disable`, {
    headers: { Origin: BASE, 'X-CSRF-Token': me.csrf_token },
  });
  expect(refusal.status()).toBe(403);
  expect(((await refusal.json()) as { detail: string }).detail).toBe('Demandez à un autre gérant.');

  // ── 5. lien de réinitialisation pour le gestionnaire ; il choisit un nouveau mot de passe, l'ancien cesse de marcher ──
  await row.getByRole('button', { name: 'Lien de réinitialisation' }).click();
  const resetLink = await page.getByTestId('shown-link').innerText();
  expect(resetLink).toMatch(/\/reinitialisation\/[A-Za-z0-9_-]{43}$/);
  const resetToken = resetLink.slice(resetLink.lastIndexOf('/') + 1);
  const third = await secondPhone(browser, viewport);
  await third.page.goto(`/reinitialisation/${resetToken}`);
  await third.page.getByLabel(/^Nouveau mot de passe/).fill('Nouveau-mot-de-passe-2026');
  await third.page.getByLabel('Confirmez le mot de passe').fill('Nouveau-mot-de-passe-2026');
  await third.page.getByRole('button', { name: 'Enregistrer' }).click();
  await expect(third.page.getByRole('status')).toContainText('Votre mot de passe est changé');
  await third.context.close();
  // sessions de la personne révoquées : le second téléphone est renvoyé à la connexion
  await other.page.goto('/');
  await expect(other.page.getByRole('heading', { name: 'Connexion' })).toBeVisible();
  await login(other.page, invitee, 'Un-bon-mot-de-passe-2026');
  await expect(other.page.getByRole('alert')).toHaveText('Identifiant ou mot de passe incorrect.');
  await login(other.page, invitee, 'Nouveau-mot-de-passe-2026');
  await expect(other.page.getByTestId('role')).toHaveText('Gestionnaire');
  // le lien de réinitialisation ne sert qu'une fois
  const fourth = await secondPhone(browser, viewport);
  await fourth.page.goto(`/reinitialisation/${resetToken}`);
  await fourth.page.getByLabel(/^Nouveau mot de passe/).fill('Encore-un-autre-2026');
  await fourth.page.getByLabel('Confirmez le mot de passe').fill('Encore-un-autre-2026');
  await fourth.page.getByRole('button', { name: 'Enregistrer' }).click();
  await expect(fourth.page.getByRole('alert')).toHaveText(
    'Ce lien n’est plus valable. Demandez-en un nouveau à votre gérant.',
  );
  await fourth.context.close();
  await other.context.close();

  // ── 6. double authentification d'un gérant de sécurité, puis accès support accordé et retiré ──
  const sec = await secondPhone(browser, viewport);
  await login(sec.page, guard, PASSWORD);
  await expect(sec.page.getByTestId('role')).toHaveText('Gérant');
  await sec.page.goto('/parametres');
  await sec.page.getByRole('button', { name: 'Activer' }).click();
  const secret = base32Decode(await sec.page.getByTestId('totp-secret').innerText());
  expect(secret.length).toBe(20);
  const step = Math.floor(Date.now() / 30_000);
  await sec.page.getByLabel('Code affiché par l’application').fill(totp(secret, step));
  await sec.page.getByRole('button', { name: 'Valider le code' }).click();
  const codeCells = sec.page.getByTestId('backup-codes').locator('code');
  await expect(codeCells).toHaveCount(10);
  const codes = await codeCells.allInnerTexts();
  await sec.page.getByLabel('J’ai conservé ces codes').check();
  await sec.page.getByRole('button', { name: 'Fermer' }).click();
  // accès support : 24 heures, motif, confirmation du mot de passe
  await sec.page.getByLabel('Motif').fill('Aide au démarrage de la démonstration');
  await sec.page.getByRole('button', { name: 'Autoriser' }).click();
  const confirm = sec.page.getByRole('dialog', { name: 'Confirmez votre mot de passe' });
  await confirm.getByLabel('Mot de passe').fill(PASSWORD);
  await confirm.getByRole('button', { name: 'Confirmer' }).click();
  const grant = sec.page.getByTestId('grant').first();
  await expect(grant).toContainText('Actif');
  await grant.getByRole('button', { name: 'Retirer l’accès' }).click();
  await expect(sec.page.getByTestId('grant').first()).toContainText('Retiré');
  // reconnexion : le mot de passe ne suffit plus, le code est demandé ; un code de secours ne sert qu'une fois
  await sec.page.getByRole('button', { name: /Menu utilisateur/ }).click();
  await sec.page.getByRole('button', { name: 'Déconnexion' }).click();
  await login(sec.page, guard, PASSWORD);
  await expect(sec.page.getByRole('heading', { name: 'Vérification en deux étapes' })).toBeVisible();
  await sec.page.getByLabel('Code de votre application d’authentification').fill('000000');
  await sec.page.getByRole('button', { name: 'Valider' }).click();
  await expect(sec.page.getByRole('alert')).toContainText('Code incorrect');
  await sec.page.getByRole('button', { name: 'Utiliser un code de secours' }).click();
  await sec.page.getByLabel('Code de secours').fill(codes[0] ?? '');
  await sec.page.getByRole('button', { name: 'Valider' }).click();
  await expect(sec.page.getByTestId('role')).toHaveText('Gérant');
  await sec.context.close();

  // ── 7. le journal retrouve chaque action avec son auteur et son heure ──
  await page.goto('/journal-audit');
  const rows = page.getByTestId('audit-row');
  await expect(rows.first()).toBeVisible();
  const text = (await rows.allInnerTexts()).join('\n');
  for (const sentence of [
    'a invité un utilisateur',
    'a rejoint l’organisation',
    'a changé le rôle d’un membre',
    'a généré un lien de réinitialisation',
    'a activé la double authentification',
    'a autorisé l’équipe HABEK',
    'a retiré l’accès à l’équipe HABEK',
  ]) {
    expect(text, sentence).toContain(sentence);
  }
  // aucun jeton, mot de passe ni code n'apparaît dans le journal affiché
  const everything = JSON.stringify(await (await page.request.get('/api/audit?limit=100')).json());
  for (const secretText of [
    token,
    resetToken,
    PASSWORD,
    'Un-bon-mot-de-passe-2026',
    'Nouveau-mot-de-passe-2026',
  ]) {
    expect(everything).not.toContain(secretText);
  }
});
