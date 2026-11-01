// Étape 03 : double authentification à la connexion, confirmation par mot de passe, utilisateurs et invitations, liens publics,
// paramètres (TOTP), journal d'audit. API simulée ; l'API réelle est couverte par `tests/db/tests/step03_*.rs`.
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import type { Me } from './api/client';
import { ApiProblem } from './api/client';
import { messageFor } from './messages';

interface Reply {
  status: number;
  body?: unknown;
}
interface Call {
  key: string;
  body: Record<string, unknown> | undefined;
  headers: Record<string, string>;
}
type Handler = (call: Call) => Reply;

const calls: Call[] = [];
const TOKEN = 'A'.repeat(43);

function problem(status: number, code: string, extra: Record<string, unknown> = {}): Reply {
  return { status, body: { type: 'about:blank', title: code, status, detail: code, code, ...extra } };
}

function me(role: string, permissions: string[], plan = 'agency'): Me {
  return {
    user: { id: 'u1', display_name: 'Awa Kaboré' },
    membership: { role },
    organization: {
      id: 'o1',
      name: 'Agence A',
      plan,
      kind: 'real',
      lifecycle: 'setup',
      timezone: 'Africa/Ouagadougou',
      currency: 'XOF',
    },
    organizations: [{ id: 'o1', name: 'Agence A', plan, kind: 'real', lifecycle: 'setup', role }],
    features: {},
    permissions,
    today: '2026-11-13',
    server_time: '2026-11-13T09:15:00Z',
    csrf_token: 'csrf-1',
    min_client_version: '0.0.0',
    settings: {},
  };
}

/** Serveur simulé : `routes` associe « MÉTHODE /chemin » à une réponse ; le reste répond 404. */
function install(routes: Record<string, Handler | Reply>) {
  vi.stubGlobal(
    'fetch',
    vi.fn((url: string, init?: RequestInit) => {
      const call: Call = {
        key: `${init?.method ?? 'GET'} ${url}`,
        body: typeof init?.body === 'string' ? (JSON.parse(init.body) as Record<string, unknown>) : undefined,
        headers: (init?.headers ?? {}) as Record<string, string>,
      };
      calls.push(call);
      const route = routes[call.key];
      const r =
        route === undefined
          ? problem(404, 'resource.not_found')
          : typeof route === 'function'
            ? route(call)
            : route;
      const text = r.body === undefined ? '' : JSON.stringify(r.body);
      return Promise.resolve(new Response(r.status === 204 ? null : text, { status: r.status }));
    }),
  );
}

const sent = (key: string) => calls.filter((c) => c.key === key);

beforeEach(() => {
  calls.length = 0;
  window.history.pushState(null, '', '/');
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe('double authentification à la connexion', () => {
  it('demande le code, le valide avec le défi, puis ouvre la session', async () => {
    let signedIn = false;
    install({
      'GET /api/me': () =>
        signedIn ? { status: 200, body: me('manager', []) } : problem(401, 'auth.unauthenticated'),
      'POST /api/auth/login': {
        status: 200,
        body: {
          organizations: [],
          mfa_required: true,
          challenge: 'defi-signe',
          user: null,
          csrf_token: null,
        },
      },
      'POST /api/auth/totp/verify': () => {
        signedIn = true;
        return { status: 200, body: { mfa_required: false, organizations: [], user: null, csrf_token: 'c' } };
      },
    });
    render(<App />);
    fireEvent.change(await screen.findByLabelText('Téléphone ou e-mail'), {
      target: { value: '70 12 34 56' },
    });
    fireEvent.change(screen.getByLabelText('Mot de passe'), { target: { value: 'un mot de passe correct' } });
    fireEvent.click(screen.getByRole('button', { name: 'Se connecter' }));
    expect(await screen.findByRole('heading', { name: 'Vérification en deux étapes' })).toBeInTheDocument();
    expect(screen.queryByLabelText('Mot de passe')).toBeNull();
    fireEvent.change(screen.getByLabelText('Code de votre application d’authentification'), {
      target: { value: '123456' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Valider' }));
    await waitFor(() => {
      expect(screen.getByTestId('org-name')).toHaveTextContent('Agence A');
    });
    expect(sent('POST /api/auth/totp/verify')[0]?.body).toEqual({ challenge: 'defi-signe', code: '123456' });
  });

  it('un code de secours se saisit après bascule, un code refusé affiche le message du catalogue', async () => {
    install({
      'GET /api/me': problem(401, 'auth.unauthenticated'),
      'POST /api/auth/login': {
        status: 200,
        body: { organizations: [], mfa_required: true, challenge: 'd' },
      },
      'POST /api/auth/totp/verify': problem(401, 'auth.totp_invalid'),
    });
    render(<App />);
    fireEvent.change(await screen.findByLabelText('Téléphone ou e-mail'), {
      target: { value: '70 12 34 56' },
    });
    fireEvent.change(screen.getByLabelText('Mot de passe'), { target: { value: 'x' } });
    fireEvent.click(screen.getByRole('button', { name: 'Se connecter' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Utiliser un code de secours' }));
    fireEvent.change(screen.getByLabelText('Code de secours'), { target: { value: 'ABCD-EFGH-JKMN' } });
    fireEvent.click(screen.getByRole('button', { name: 'Valider' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Code incorrect. Vérifiez l’heure de votre téléphone.',
    );
    expect(screen.getByLabelText('Code de secours')).toHaveValue('');
  });
});

describe('utilisateurs et confirmation du mot de passe', () => {
  const members = {
    users: [
      {
        membership_id: 'm1',
        user_id: 'u1',
        display_name: 'Awa Kaboré',
        identifier: '+22670000011',
        role: 'manager',
        status: 'active',
        is_self: true,
      },
      {
        membership_id: 'm2',
        user_id: 'u2',
        display_name: 'Issa Ouédraogo',
        identifier: '+22670000012',
        role: 'cashier',
        status: 'active',
        is_self: false,
      },
    ],
  };

  it('changer un rôle : modale de confirmation, puis action rejouée une fois', async () => {
    window.history.pushState(null, '', '/utilisateurs');
    let confirmedOnce = false;
    install({
      'GET /api/me': { status: 200, body: me('manager', ['user.manage']) },
      'GET /api/users': { status: 200, body: members },
      'GET /api/invitations': { status: 200, body: { invitations: [] } },
      'PATCH /api/users/m2': () =>
        confirmedOnce
          ? { status: 200, body: { ...members.users[1], role: 'accountant' } }
          : problem(403, 'auth.reauth_required'),
      'POST /api/auth/reauth': () => {
        confirmedOnce = true;
        return { status: 204 };
      },
    });
    render(<App />);
    const select = await screen.findByLabelText('Changer le rôle : Issa Ouédraogo');
    fireEvent.change(select, { target: { value: 'accountant' } });
    const dialog = await screen.findByRole('dialog', { name: 'Confirmez votre mot de passe' });
    fireEvent.change(within(dialog).getByLabelText('Mot de passe'), {
      target: { value: 'un mot de passe correct' },
    });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Confirmer' }));
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(sent('PATCH /api/users/m2')).toHaveLength(2);
    // le rôle choisi est celui de l'action rejouée, pas l'ancien rôle que la liste affiche entre-temps
    expect(sent('PATCH /api/users/m2').map((c) => c.body)).toEqual([
      { role: 'accountant' },
      { role: 'accountant' },
    ]);
    expect(sent('POST /api/auth/reauth')[0]?.body).toEqual({ password: 'un mot de passe correct' });
    // le jeton anti-falsification accompagne chaque écriture
    expect(sent('PATCH /api/users/m2')[0]?.headers['X-CSRF-Token']).toBe('csrf-1');
  });

  it('annuler la confirmation n’exécute rien et ne montre aucune erreur', async () => {
    window.history.pushState(null, '', '/utilisateurs');
    install({
      'GET /api/me': { status: 200, body: me('manager', ['user.manage']) },
      'GET /api/users': { status: 200, body: members },
      'GET /api/invitations': { status: 200, body: { invitations: [] } },
      'POST /api/users/m2/disable': problem(403, 'auth.reauth_required'),
    });
    render(<App />);
    await screen.findByText('Issa Ouédraogo');
    const row = screen.getAllByTestId('member')[1];
    fireEvent.click(within(row as HTMLElement).getByRole('button', { name: 'Désactiver' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Annuler' }));
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(screen.queryByRole('alert')).toBeNull();
    expect(sent('POST /api/users/m2/disable')).toHaveLength(1);
  });

  it('inviter : le lien s’affiche une fois, avec « Copier » et le message WhatsApp', async () => {
    window.history.pushState(null, '', '/utilisateurs');
    install({
      'GET /api/me': { status: 200, body: me('manager', ['user.manage']) },
      'GET /api/users': { status: 200, body: members },
      'GET /api/invitations': { status: 200, body: { invitations: [] } },
      'POST /api/invitations': {
        status: 200,
        body: { url: `https://app.test/invitation/${TOKEN}`, expires_at: 'x', invitation: {} },
      },
    });
    render(<App />);
    fireEvent.change(await screen.findByLabelText('Téléphone ou e-mail'), {
      target: { value: '70 12 34 56' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Créer le lien' }));
    expect(await screen.findByTestId('shown-link')).toHaveTextContent(`https://app.test/invitation/${TOKEN}`);
    expect(sent('POST /api/invitations')[0]?.body).toEqual({ phone: '70 12 34 56', role: 'accountant' });
    const whatsapp = screen.getByRole('link', { name: 'WhatsApp' });
    const href = whatsapp.getAttribute('href') ?? '';
    expect(href.startsWith('https://wa.me/?text=')).toBe(true);
    expect(decodeURIComponent(href)).toContain(
      'Bonjour, voici votre invitation à rejoindre Agence A sur Immo Suite',
    );
    expect(decodeURIComponent(href)).toContain('Elle est valable 7 jours.');
  });

  it('un caissier qui tape l’adresse à la main voit « Accès refusé »', async () => {
    window.history.pushState(null, '', '/utilisateurs');
    install({ 'GET /api/me': { status: 200, body: me('cashier', ['payment.record']) } });
    render(<App />);
    expect(await screen.findByRole('heading', { name: 'Accès refusé' })).toBeInTheDocument();
    expect(sent('GET /api/users')).toHaveLength(0);
  });
});

describe('liens publics', () => {
  it('la réinitialisation retire le jeton de l’adresse, refuse deux mots de passe différents, puis confirme', async () => {
    window.history.pushState(null, '', `/reinitialisation/${TOKEN}`);
    install({
      'GET /api/me': problem(401, 'auth.unauthenticated'),
      'POST /api/auth/password-reset/complete': { status: 204 },
    });
    render(<App />);
    expect(await screen.findByRole('heading', { name: 'Nouveau mot de passe' })).toBeInTheDocument();
    expect(window.location.pathname).toBe('/');
    fireEvent.change(screen.getByLabelText(/^Nouveau mot de passe/), {
      target: { value: 'un nouveau mot de passe' },
    });
    fireEvent.change(screen.getByLabelText('Confirmez le mot de passe'), {
      target: { value: 'autre chose' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Enregistrer' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Les deux mots de passe ne sont pas identiques.',
    );
    expect(sent('POST /api/auth/password-reset/complete')).toHaveLength(0);
    fireEvent.change(screen.getByLabelText('Confirmez le mot de passe'), {
      target: { value: 'un nouveau mot de passe' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Enregistrer' }));
    expect(await screen.findByRole('status')).toHaveTextContent(
      'Votre mot de passe est changé. Connectez-vous avec le nouveau.',
    );
    expect(sent('POST /api/auth/password-reset/complete')[0]?.body).toEqual({
      token: TOKEN,
      new_password: 'un nouveau mot de passe',
    });
  });

  it('une invitation expirée affiche le texte du contrat', async () => {
    window.history.pushState(null, '', `/invitation/${TOKEN}`);
    install({
      'GET /api/me': problem(401, 'auth.unauthenticated'),
      [`GET /api/invitations/${TOKEN}`]: problem(410, 'invitation.expired'),
    });
    render(<App />);
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Cette invitation n’est plus valable. Demandez au gérant de la renvoyer.',
    );
    expect(window.location.pathname).toBe('/');
  });

  it('un nouveau compte choisit son nom et son mot de passe puis arrive connecté', async () => {
    window.history.pushState(null, '', `/invitation/${TOKEN}`);
    let signedIn = false;
    install({
      'GET /api/me': () =>
        signedIn
          ? { status: 200, body: me('cashier', ['payment.record']) }
          : problem(401, 'auth.unauthenticated'),
      [`GET /api/invitations/${TOKEN}`]: {
        status: 200,
        body: { organization_name: 'Agence A', role: 'cashier', expires_at: 'x', identifier_exists: false },
      },
      'POST /api/invitations/accept': () => {
        signedIn = true;
        return { status: 200, body: { mfa_required: false, organizations: [] } };
      },
    });
    render(<App />);
    expect(
      await screen.findByText('Vous êtes invité à rejoindre Agence A comme Caissier.'),
    ).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText('Votre nom'), { target: { value: 'Awa' } });
    fireEvent.change(screen.getByLabelText(/Choisissez un mot de passe/), {
      target: { value: 'un bon mot de passe' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Rejoindre' }));
    await waitFor(() => {
      expect(screen.getByTestId('role')).toHaveTextContent('Caissier');
    });
    expect(sent('POST /api/invitations/accept')[0]?.body).toEqual({
      token: TOKEN,
      display_name: 'Awa',
      password: 'un bon mot de passe',
    });
  });
});

describe('paramètres', () => {
  it('activer la double authentification : clé affichée, codes de secours conservés avant de fermer', async () => {
    window.history.pushState(null, '', '/parametres');
    install({
      'GET /api/me': { status: 200, body: me('manager', ['user.manage', 'support.grant', 'settings.write']) },
      'GET /api/organization': {
        status: 200,
        body: { name: 'Agence A', city: null, plan: 'agency', kind: 'real', lifecycle: 'setup', version: 1 },
      },
      'GET /api/settings': {
        status: 200,
        body: {
          receipt_prefix: 'REC',
          receipt_header: null,
          address: null,
          legal_ids: {},
          default_paper: 'a5',
          share_link_ttl_days: 30,
          payment_confirm_threshold: 1000000,
          version: 1,
        },
      },
      'GET /api/role-settings': {
        status: 200,
        body: {
          settings: [{ role: 'property_manager', action: 'record_payment', allowed: false, version: 0 }],
        },
      },
      'GET /api/support-grants': { status: 200, body: { grants: [], accesses: [] } },
      'POST /api/me/totp': {
        status: 200,
        body: { secret_base32: 'JBSWY3DPEHPK3PXP', otpauth_url: 'otpauth://totp/x' },
      },
      'POST /api/me/totp/confirm': {
        status: 200,
        body: { backup_codes: ['ABCD-EFGH-JKMN', 'PQRS-TVWX-YZ23'] },
      },
    });
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Activer' }));
    expect(await screen.findByTestId('totp-secret')).toHaveTextContent('JBSW Y3DP EHPK 3PXP');
    fireEvent.change(screen.getByLabelText('Code affiché par l’application'), {
      target: { value: '123456' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Valider le code' }));
    const codes = await screen.findByTestId('backup-codes');
    expect(codes).toHaveTextContent('ABCD-EFGH-JKMN');
    const close = screen.getByRole('button', { name: 'Fermer' });
    expect(close).toBeDisabled();
    fireEvent.click(screen.getByLabelText('J’ai conservé ces codes'));
    expect(close).toBeEnabled();
    // droits réglables et accès support visibles pour un gérant, dans leur état par défaut
    expect(screen.getByLabelText('Le gestionnaire peut encaisser')).not.toBeChecked();
    expect(
      screen.getByText('Aucun accès accordé. L’équipe HABEK ne voit pas vos données.'),
    ).toBeInTheDocument();
  });
});

describe('journal d’audit', () => {
  const page = (rows: unknown[], next: string | null = null) => ({
    status: 200,
    body: { items: rows, next_cursor: next, meta: { actors: [{ id: 'u1', name: 'Awa Kaboré' }] } },
  });
  const row = (id: number, action: string, extra: Record<string, unknown> = {}) => ({
    id,
    occurred_at: '2026-11-13T09:15:00Z',
    actor: { kind: 'user', user_id: 'u1', name: 'Awa Kaboré' },
    action,
    object: { type: 'invitation', id: 'i1' },
    before: null,
    after: { role: 'cashier' },
    request_id: 'INC-1',
    ...extra,
  });

  it('lit le journal en phrases, déplie l’avant et l’après, affiche l’IP seulement si le serveur l’envoie', async () => {
    window.history.pushState(null, '', '/journal-audit');
    install({
      'GET /api/me': { status: 200, body: me('manager', ['audit.read']) },
      'GET /api/audit': page(
        [row(2, 'user.invited', { ip: '10.9.9.9' }), row(1, 'security.rate_limited')],
        'curseur',
      ),
      'GET /api/audit?cursor=curseur': page([row(0, 'auth.login')]),
    });
    render(<App />);
    const rows = await screen.findAllByTestId('audit-row');
    expect(rows).toHaveLength(2);
    expect(rows[0]).toHaveTextContent('a invité un utilisateur');
    expect(rows[1]).toHaveTextContent('a été ralenti après des échecs de connexion');
    const first = rows[0] as HTMLElement;
    expect(first).toHaveTextContent('Adresse IP 10.9.9.9');
    expect(within(first).getByText('Après')).toBeInTheDocument();
    expect(screen.getAllByTestId('audit-row')[1]).not.toHaveTextContent('Adresse IP');
    // « Exporter » est un repère « À venir »
    expect(screen.getByRole('button', { name: 'Exporter' })).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: 'Voir plus' }));
    await waitFor(() => {
      expect(screen.getAllByTestId('audit-row')).toHaveLength(3);
    });
    expect(screen.getAllByTestId('audit-row')[2]).toHaveTextContent('s’est connecté');
    expect(screen.queryByRole('button', { name: 'Voir plus' })).toBeNull();
  });

  it('un filtre relance la lecture ; aucun événement : état vide', async () => {
    window.history.pushState(null, '', '/journal-audit');
    install({
      'GET /api/me': { status: 200, body: me('accountant', ['audit.read']) },
      'GET /api/audit': page([row(1, 'user.invited')]),
      'GET /api/audit?actor_kind=support': page([]),
    });
    render(<App />);
    await screen.findAllByTestId('audit-row');
    fireEvent.change(screen.getByLabelText('Auteur'), { target: { value: 'support' } });
    expect(await screen.findByText('Aucun événement pour ces filtres.')).toBeInTheDocument();
  });
});

describe('messages', () => {
  it('mot de passe refusé : la raison du serveur choisit le texte', () => {
    for (const [reason, text] of [
      ['too_short', 'Au moins 10 caractères.'],
      ['common', 'Ce mot de passe est trop courant. Choisissez-en un autre.'],
      ['contains_identifier', 'Le mot de passe ne doit pas contenir votre identifiant.'],
    ] as const) {
      expect(messageFor(new ApiProblem(422, 'auth.password_weak', '', undefined, undefined, reason))).toBe(
        text,
      );
    }
  });

  it('erreur de champ : le texte vient du code, jamais du serveur', () => {
    const e = new ApiProblem(422, 'validation.failed', 'texte du serveur', undefined, undefined, undefined, [
      { field: 'login_phone', code: 'validation.phone_format' },
    ]);
    expect(messageFor(e)).toBe('Numéro de téléphone invalide. Exemple : 70 12 34 56.');
  });
});
