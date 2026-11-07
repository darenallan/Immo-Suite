// Étape 05 : biens et lots (S06, S07), locataires (S08, S09), recherche unique (F04, F05, F06, DP-92). API simulée ; l'API réelle
// est couverte par `tests/db/tests/step05_catalog.rs`. Données fictives (bloc `+226 00 00 …`).
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import type { Me } from './api/client';

interface Reply {
  status: number;
  body?: unknown;
}
interface Call {
  key: string;
  body: Record<string, unknown> | undefined;
}
type Handler = (call: Call) => Reply;

const calls: Call[] = [];
const PID = '11111111-1111-4111-8111-111111111111';
const UID = '22222222-2222-4222-8222-222222222222';
const TID = '33333333-3333-4333-8333-333333333333';

function problem(status: number, code: string, extra: Record<string, unknown> = {}): Reply {
  return { status, body: { type: 'about:blank', title: code, status, detail: code, code, ...extra } };
}

function me(role: string, permissions: string[]): Me {
  return {
    user: { id: 'u1', display_name: 'Awa Kaboré' },
    membership: { role },
    organization: {
      id: 'o1',
      name: 'Agence A',
      plan: 'agency',
      kind: 'real',
      lifecycle: 'setup',
      timezone: 'Africa/Ouagadougou',
      currency: 'XOF',
    },
    organizations: [{ id: 'o1', name: 'Agence A', plan: 'agency', kind: 'real', lifecycle: 'setup', role }],
    features: {},
    permissions,
    today: '2026-11-13',
    server_time: '2026-11-13T09:15:00Z',
    csrf_token: 'csrf-1',
    min_client_version: '0.0.0',
    settings: {},
  };
}

const MANAGER = ['catalog.read', 'catalog.write', 'search.use', 'mandate.write'];
const CASHIER = ['catalog.read', 'search.use'];

const unit = {
  id: UID,
  property_id: PID,
  property_name: 'Zogona',
  reference: 'A1',
  kind: 'apartment',
  usage: 'residential',
  surface_m2: null,
  reference_rent: 75000,
  permitted_activity: null,
  status: 'vacant',
  archived_at: null,
  version: 1,
};
const property = {
  id: PID,
  name: 'Zogona',
  address: null,
  district: 'Zogona',
  city: 'Ouagadougou',
  default_owner: null,
  notes: null,
  unit_count: 1,
  vacant_count: 1,
  archived_at: null,
  version: 1,
  units: [unit],
};
const tenant = {
  id: TID,
  kind: 'person',
  last_name: 'Tapsoba',
  first_name: 'Issouf',
  display_name: 'Issouf Tapsoba',
  trade_activity: null,
  phone: '+22600000101',
  phone_is_whatsapp: true,
  email: null,
  unit_labels: [],
  balance: null,
  merged_into_id: null,
  archived_at: null,
  version: 1,
};

function install(routes: Record<string, Handler | Reply>) {
  vi.stubGlobal(
    'fetch',
    vi.fn((url: string, init?: RequestInit) => {
      const call: Call = {
        key: `${init?.method ?? 'GET'} ${url}`,
        body: typeof init?.body === 'string' ? (JSON.parse(init.body) as Record<string, unknown>) : undefined,
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
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe('S06 biens et lots', () => {
  it('maison seule : un seul formulaire crée le bien et son lot, puis ouvre la fiche', async () => {
    window.history.pushState(null, '', '/biens');
    install({
      'GET /api/me': { status: 200, body: me('manager', MANAGER) },
      'GET /api/properties': { status: 200, body: { items: [], next_cursor: null } },
      'GET /api/owners': { status: 200, body: { items: [], next_cursor: null } },
      'POST /api/properties': { status: 201, body: property },
      [`GET /api/properties/${PID}`]: { status: 200, body: property },
    });
    render(<App />);
    expect(await screen.findByText(/Aucun bien pour le moment/)).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Créer un bien' }));
    fireEvent.change(screen.getByLabelText('Nom du bien'), { target: { value: 'Villa Kaboré' } });
    fireEvent.click(screen.getByLabelText('Maison seule (un bien et un lot)'));
    fireEvent.change(screen.getByLabelText('Loyer de référence (FCFA)'), { target: { value: '75 000' } });
    fireEvent.click(screen.getByRole('button', { name: 'Créer' }));
    await screen.findByRole('heading', { name: /Zogona/ });
    const body = sent('POST /api/properties')[0]?.body;
    expect(body?.['name']).toBe('Villa Kaboré');
    // le loyer part en entier (R-001), jamais en texte ni en décimal
    expect((body?.['single_unit'] as Record<string, unknown>)['reference_rent']).toBe(75000);
    expect(window.location.pathname).toBe(`/biens/${PID}`);
  });

  it('« Activité autorisée » n’apparaît que pour un lot commercial ou mixte (R-012)', async () => {
    window.history.pushState(null, '', `/biens/${PID}`);
    install({
      'GET /api/me': { status: 200, body: me('manager', MANAGER) },
      [`GET /api/properties/${PID}`]: { status: 200, body: property },
    });
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Ajouter un lot' }));
    expect(screen.queryByLabelText('Activité autorisée')).toBeNull();
    fireEvent.change(screen.getByLabelText('Usage'), { target: { value: 'commercial' } });
    expect(screen.getByLabelText('Activité autorisée')).toBeTruthy();
  });

  it('une référence déjà utilisée dans le bien s’affiche sous le message du serveur', async () => {
    window.history.pushState(null, '', `/biens/${PID}`);
    install({
      'GET /api/me': { status: 200, body: me('manager', MANAGER) },
      [`GET /api/properties/${PID}`]: { status: 200, body: property },
      'POST /api/units': problem(409, 'unit.reference_duplicate'),
    });
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Ajouter un lot' }));
    fireEvent.change(screen.getByLabelText('Référence'), { target: { value: 'A1' } });
    fireEvent.click(screen.getByRole('button', { name: 'Créer' }));
    expect(await screen.findByText('Cette référence est déjà utilisée dans ce bien.')).toBeTruthy();
  });

  it('le caissier lit les biens sans bouton de création', async () => {
    window.history.pushState(null, '', '/biens');
    install({
      'GET /api/me': { status: 200, body: me('cashier', CASHIER) },
      'GET /api/properties': { status: 200, body: { items: [property], next_cursor: null } },
    });
    render(<App />);
    expect(await screen.findByRole('link', { name: 'Zogona' })).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Créer un bien' })).toBeNull();
  });
});

describe('S08 et S09 locataires', () => {
  it('un numéro déjà porté propose d’ouvrir la fiche existante, sans « Fusionner » (DP-92)', async () => {
    window.history.pushState(null, '', '/locataires');
    install({
      'GET /api/me': { status: 200, body: me('manager', MANAGER) },
      'GET /api/tenants': { status: 200, body: { items: [], next_cursor: null } },
      'POST /api/tenants': problem(409, 'tenant.phone_duplicate', {
        params: { conflict: { id: TID, display_name: 'Issouf Tapsoba' } },
      }),
      [`GET /api/tenants/${TID}`]: { status: 200, body: tenant },
    });
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Créer un locataire' }));
    fireEvent.change(screen.getByLabelText('Nom'), { target: { value: 'Tapsoba' } });
    fireEvent.change(screen.getByLabelText('Téléphone'), { target: { value: '00 00 01 01' } });
    fireEvent.click(screen.getByRole('button', { name: 'Créer' }));
    expect(await screen.findByText('Un locataire avec ce numéro existe déjà : Issouf Tapsoba.')).toBeTruthy();
    expect(screen.queryByRole('button', { name: /Fusionner/ })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Ouvrir la fiche' }));
    await screen.findByRole('heading', { name: /Issouf Tapsoba/ });
    expect(window.location.pathname).toBe(`/locataires/${TID}`);
  });

  it('la fiche propose Appeler et WhatsApp ; Encaisser et Relancer restent désactivés', async () => {
    window.history.pushState(null, '', `/locataires/${TID}`);
    install({
      'GET /api/me': { status: 200, body: me('manager', MANAGER) },
      [`GET /api/tenants/${TID}`]: { status: 200, body: tenant },
    });
    render(<App />);
    const call = await screen.findByRole('link', { name: 'Appeler' });
    expect(call.getAttribute('href')).toBe('tel:+22600000101');
    expect(screen.getByRole('link', { name: 'WhatsApp' }).getAttribute('href')).toBe(
      'https://wa.me/22600000101',
    );
    expect(screen.getByRole<HTMLButtonElement>('button', { name: 'Encaisser' }).disabled).toBe(true);
    expect(screen.getByText('+226 00 00 01 01')).toBeTruthy();
  });

  it('le contact de secours absent de la réponse (caissier) n’est pas affiché', async () => {
    window.history.pushState(null, '', `/locataires/${TID}`);
    install({
      'GET /api/me': { status: 200, body: me('cashier', CASHIER) },
      [`GET /api/tenants/${TID}`]: { status: 200, body: tenant },
    });
    render(<App />);
    await screen.findByRole('heading', { name: /Issouf Tapsoba/ });
    expect(screen.queryByText('Contact de secours')).toBeNull();
    expect(screen.queryByText('Notes')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Archiver' })).toBeNull();
  });
});

describe('recherche unique', () => {
  it('montre les groupes, ouvre la fiche, et ne mesure jamais le texte tapé', async () => {
    window.history.pushState(null, '', '/');
    install({
      'GET /api/me': { status: 200, body: me('manager', MANAGER) },
      'GET /api/search?q=tapsoba': {
        status: 200,
        body: {
          tenants: [
            {
              id: TID,
              display_name: 'Issouf Tapsoba',
              phone: '+22600000101',
              unit_labels: [],
              balance: null,
            },
          ],
          units: [],
          receipts: [],
        },
      },
      'GET /api/search?q=inconnu': { status: 200, body: { tenants: [], units: [], receipts: [] } },
      [`GET /api/tenants/${TID}`]: { status: 200, body: tenant },
      'POST /api/events': { status: 204 },
    });
    render(<App />);
    const box = await screen.findByLabelText('Rechercher');
    fireEvent.change(box, { target: { value: 'inconnu' } });
    expect(await screen.findByText(/Aucun résultat pour « inconnu »/)).toBeTruthy();
    fireEvent.change(box, { target: { value: 'tapsoba' } });
    fireEvent.click(await screen.findByRole('button', { name: /Issouf Tapsoba/ }));
    await screen.findByRole('heading', { name: /Issouf Tapsoba/ });
    await waitFor(() => {
      expect(window.location.pathname).toBe(`/locataires/${TID}`);
    });
    for (const c of sent('POST /api/events')) {
      expect(JSON.stringify(c.body)).not.toContain('tapsoba');
      expect(JSON.stringify(c.body)).not.toContain('inconnu');
    }
  });
});

describe('S18 lots assignés (R-081)', () => {
  it('le gérant coche les lots d’un agent terrain et les enregistre', async () => {
    window.history.pushState(null, '', '/utilisateurs');
    const agent = {
      membership_id: 'm-agent',
      user_id: 'u-agent',
      display_name: 'Salif Agent',
      identifier: '+22600000909',
      role: 'field_agent',
      status: 'active',
      is_self: false,
      unit_ids: [],
    };
    install({
      'GET /api/me': { status: 200, body: me('manager', [...MANAGER, 'user.manage']) },
      'GET /api/users': { status: 200, body: { users: [agent] } },
      'GET /api/invitations': { status: 200, body: { invitations: [] } },
      'GET /api/units?limit=100': { status: 200, body: { items: [unit], next_cursor: null } },
      'PUT /api/users/m-agent/unit-assignments': { status: 200, body: { unit_ids: [UID] } },
    });
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Lots assignés (0)' }));
    fireEvent.click(await screen.findByLabelText(/A1 · Zogona/));
    fireEvent.click(screen.getByRole('button', { name: 'Enregistrer' }));
    await waitFor(() => {
      expect(sent('PUT /api/users/m-agent/unit-assignments')[0]?.body).toEqual({ unit_ids: [UID] });
    });
  });
});
