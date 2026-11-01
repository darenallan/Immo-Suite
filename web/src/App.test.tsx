// Parcours de l'interface avec une API simulée : connexion (erreur, limitation, succès), expiration, déconnexion, menus par rôle,
// accès refusé. L'API réelle est couverte par les tests d'API (`tests/db/tests/api.rs`).
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import type { Me } from './api/client';

interface Reply {
  status: number;
  body?: unknown;
}
type Handler = (url: string) => Reply;

function problem(status: number, code: string, extra: Record<string, unknown> = {}): Reply {
  return { status, body: { type: 'about:blank', title: code, status, detail: code, code, ...extra } };
}

function me(
  role: string,
  org = 'Agence A',
  permissions: string[] = [],
  organizations?: Me['organizations'],
): Me {
  return {
    user: { id: 'u1', display_name: 'Awa Kaboré' },
    membership: { role },
    organization: {
      id: 'o1',
      name: org,
      plan: 'agency',
      kind: 'real',
      lifecycle: 'setup',
      timezone: 'Africa/Ouagadougou',
      currency: 'XOF',
    },
    organizations: organizations ?? [
      { id: 'o1', name: org, plan: 'agency', kind: 'real', lifecycle: 'setup', role },
    ],
    features: {},
    permissions,
    today: '2026-11-13',
    server_time: '2026-11-13T09:15:00Z',
    csrf_token: 'csrf-1',
    min_client_version: '0.0.0',
    settings: {},
  };
}

interface Call {
  url: string;
  method: string;
  headers: Record<string, string>;
  body: unknown;
}
const calls: Call[] = [];

function install(handler: Handler) {
  vi.stubGlobal(
    'fetch',
    vi.fn((url: string, init?: RequestInit) => {
      calls.push({
        url,
        method: init?.method ?? 'GET',
        headers: (init?.headers ?? {}) as Record<string, string>,
        body: typeof init?.body === 'string' ? (JSON.parse(init.body) as unknown) : undefined,
      });
      const r = handler(url);
      const text = r.body === undefined ? '' : JSON.stringify(r.body);
      return Promise.resolve(new Response(r.status === 204 ? null : text, { status: r.status }));
    }),
  );
}

beforeEach(() => {
  calls.length = 0;
  window.history.pushState(null, '', '/');
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

async function typeCredentials(identifier: string, password: string) {
  fireEvent.change(await screen.findByLabelText('Téléphone ou e-mail'), { target: { value: identifier } });
  fireEvent.change(screen.getByLabelText('Mot de passe'), { target: { value: password } });
}

describe('connexion', () => {
  it('affiche la connexion quand aucune session n’existe', async () => {
    install(() => problem(401, 'auth.unauthenticated'));
    render(<App />);
    expect(await screen.findByRole('heading', { name: 'Connexion' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Se connecter' })).toBeDisabled();
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('mauvais mot de passe : message du catalogue, champ vidé', async () => {
    install((url) =>
      url === '/api/auth/login'
        ? problem(401, 'auth.invalid_credentials')
        : problem(401, 'auth.unauthenticated'),
    );
    render(<App />);
    await typeCredentials('70 12 34 56', 'faux');
    fireEvent.click(screen.getByRole('button', { name: 'Se connecter' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Identifiant ou mot de passe incorrect.');
    expect(screen.getByLabelText('Mot de passe')).toHaveValue('');
    expect(screen.getByLabelText('Téléphone ou e-mail')).toHaveValue('70 12 34 56');
  });

  it('trop de tentatives : durée en minutes dans le message', async () => {
    install((url) =>
      url === '/api/auth/login'
        ? problem(429, 'auth.rate_limited', { retry_after_seconds: 60 })
        : problem(401, 'auth.unauthenticated'),
    );
    render(<App />);
    await typeCredentials('70 12 34 56', 'x');
    fireEvent.click(screen.getByRole('button', { name: 'Se connecter' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Trop de tentatives. Réessayez dans 1 minute ou réinitialisez votre mot de passe.',
    );
  });

  it('pendant l’envoi : bouton « Connexion… » désactivé, un seul appel de connexion', async () => {
    let release: (() => void) | undefined;
    let logged = false;
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string) => {
        calls.push({ url, method: 'x', headers: {}, body: undefined });
        if (url === '/api/auth/login') {
          return new Promise<Response>((resolve) => {
            release = () => {
              logged = true;
              resolve(
                new Response(
                  JSON.stringify({ user: {}, organizations: [], csrf_token: 'x', mfa_required: false }),
                  { status: 200 },
                ),
              );
            };
          });
        }
        return Promise.resolve(
          logged
            ? new Response(JSON.stringify(me('manager')), { status: 200 })
            : new Response(JSON.stringify({ code: 'auth.unauthenticated' }), { status: 401 }),
        );
      }),
    );
    render(<App />);
    await typeCredentials('70 12 34 56', 'secret');
    fireEvent.click(screen.getByRole('button', { name: 'Se connecter' }));
    const busy = await screen.findByRole('button', { name: 'Connexion…' });
    expect(busy).toBeDisabled();
    fireEvent.click(busy);
    await act(async () => {
      await Promise.resolve();
      release?.();
    });
    expect(await screen.findByTestId('org-name')).toBeInTheDocument();
    expect(calls.filter((c) => c.url === '/api/auth/login')).toHaveLength(1);
  });

  it('succès : arrive sur l’accueil avec le nom de l’organisation et le rôle', async () => {
    let logged = false;
    install((url) => {
      if (url === '/api/auth/login') {
        logged = true;
        return {
          status: 200,
          body: {
            user: { id: 'u1', display_name: 'Awa Kaboré' },
            organizations: [],
            csrf_token: 'csrf-1',
            mfa_required: false,
          },
        };
      }
      return logged
        ? { status: 200, body: me('manager', 'Agence Horizon', ['catalog.read']) }
        : problem(401, 'auth.unauthenticated');
    });
    render(<App />);
    await typeCredentials('70 12 34 56', 'un mot de passe correct');
    fireEvent.click(screen.getByRole('button', { name: 'Se connecter' }));
    expect(await screen.findByTestId('org-name')).toHaveTextContent('Agence Horizon');
    expect(screen.getByTestId('role')).toHaveTextContent('Gérant');
    // rien n'est stocké dans le navigateur
    expect(window.localStorage.length + window.sessionStorage.length).toBe(0);
    const login = calls.find((c) => c.url === '/api/auth/login');
    expect(login?.body).toEqual({ identifier: '70 12 34 56', password: 'un mot de passe correct' });
    expect(login?.headers['X-CSRF-Token']).toBeUndefined();
  });
});

describe('session', () => {
  it('une session expirée ramène à la connexion avec le message d’expiration', async () => {
    let alive = true;
    install(() => (alive ? { status: 200, body: me('cashier') } : problem(401, 'auth.session_expired')));
    render(<App />);
    expect(await screen.findByTestId('org-name')).toBeInTheDocument();
    alive = false;
    await act(async () => {
      await Promise.resolve();
      window.dispatchEvent(new Event('focus'));
    });
    expect(await screen.findByRole('heading', { name: 'Connexion' })).toBeInTheDocument();
    expect(screen.getByRole('status')).toHaveTextContent('Votre session a expiré. Reconnectez-vous.');
  });

  it('déconnexion : envoie le jeton anti-falsification et revient à la connexion', async () => {
    let out = false;
    install((url) => {
      if (url === '/api/auth/logout') {
        out = true;
        return { status: 204 };
      }
      return out ? problem(401, 'auth.unauthenticated') : { status: 200, body: me('manager') };
    });
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: /Menu utilisateur/ }));
    fireEvent.click(screen.getByRole('button', { name: 'Déconnexion' }));
    expect(await screen.findByRole('heading', { name: 'Connexion' })).toBeInTheDocument();
    const logout = calls.find((c) => c.url === '/api/auth/logout');
    expect(logout?.method).toBe('POST');
    expect(logout?.headers['X-CSRF-Token']).toBe('csrf-1');
  });

  it('plusieurs organisations : le choix précède tout, puis l’accueil', async () => {
    const two = [
      { id: 'o1', name: 'Agence A', plan: 'agency', kind: 'real', lifecycle: 'setup', role: 'cashier' },
      { id: 'o2', name: 'Agence B', plan: 'agency', kind: 'real', lifecycle: 'setup', role: 'cashier' },
    ];
    const noOrg: Me = {
      ...me('cashier'),
      membership: null,
      organization: null,
      organizations: two,
      permissions: [],
    };
    let chosen = false;
    install((url) => {
      if (url === '/api/me/organization') {
        chosen = true;
        return { status: 200, body: me('cashier', 'Agence B', [], two) };
      }
      return { status: 200, body: chosen ? me('cashier', 'Agence B', [], two) : noOrg };
    });
    render(<App />);
    expect(await screen.findByRole('heading', { name: 'Choisissez une organisation' })).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /Agence B/ }));
    expect(await screen.findByTestId('org-name')).toHaveTextContent('Agence B');
    const select = calls.find((c) => c.url === '/api/me/organization');
    expect(select?.body).toEqual({ organization_id: 'o2' });
    expect(select?.headers['X-CSRF-Token']).toBe('csrf-1');
  });
});

describe('menus par rôle et accès refusé', () => {
  function sidebarLabels(container: HTMLElement): string[] {
    const sidebar = container.querySelector('.sidebar') as HTMLElement;
    return within(sidebar)
      .getAllByRole('link')
      .map((a) => a.textContent);
  }

  it('gérant : les 17 entrées ; caissier : pas d’utilisateurs ni de paramètres', async () => {
    install(() => ({ status: 200, body: me('manager') }));
    const first = render(<App />);
    await screen.findByTestId('org-name');
    const manager = sidebarLabels(first.container);
    expect(manager).toContain('Utilisateurs');
    expect(manager).toContain('Journal d’audit');
    expect(manager).toHaveLength(17);
    first.unmount();

    install(() => ({ status: 200, body: me('cashier') }));
    const second = render(<App />);
    await screen.findByTestId('org-name');
    expect(sidebarLabels(second.container)).toEqual([
      'Accueil',
      'Encaisser',
      'Impayés',
      'Paiements et quittances',
      'Biens et lots',
      'Locataires',
      'Clôture de caisse',
    ]);
  });

  it('une adresse tapée à la main pour une entrée interdite affiche « Accès refusé »', async () => {
    window.history.pushState(null, '', '/utilisateurs');
    install(() => ({ status: 200, body: me('cashier') }));
    render(<App />);
    expect(await screen.findByRole('heading', { name: 'Accès refusé' })).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Retour à l’accueil' }));
    expect(await screen.findByTestId('role')).toHaveTextContent('Caissier');
  });

  it('une adresse inconnue affiche « Introuvable »', async () => {
    window.history.pushState(null, '', '/nimporte-quoi');
    install(() => ({ status: 200, body: me('manager') }));
    render(<App />);
    expect(await screen.findByRole('heading', { name: 'Introuvable' })).toBeInTheDocument();
  });

  it('une entrée réglable apparaît quand le serveur accorde le droit', async () => {
    install(() => ({ status: 200, body: me('property_manager', 'Agence A', []) }));
    const without = render(<App />);
    await screen.findByTestId('org-name');
    expect(sidebarLabels(without.container)).not.toContain('Encaisser');
    without.unmount();
    install(() => ({ status: 200, body: me('property_manager', 'Agence A', ['payment.record']) }));
    const withRight = render(<App />);
    await screen.findByTestId('org-name');
    expect(sidebarLabels(withRight.container)).toContain('Encaisser');
  });

  it('426 : le bandeau de mise à jour apparaît', async () => {
    let first = true;
    install(() => {
      if (first) {
        first = false;
        return { status: 200, body: me('manager') };
      }
      return problem(426, 'client.upgrade_required');
    });
    render(<App />);
    await screen.findByTestId('org-name');
    await act(async () => {
      await Promise.resolve();
      window.dispatchEvent(new Event('focus'));
    });
    await waitFor(() => {
      expect(screen.getByText('Une mise à jour est disponible.')).toBeInTheDocument();
    });
  });
});
