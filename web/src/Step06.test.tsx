// Étape 06 : baux et échéancier (S10 ; F07, F08, DP-94). API simulée ; l'API réelle est couverte par
// `tests/db/tests/step06_leases.rs`. Données fictives (bloc `+226 00 00 …`).
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
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
  headers: Headers;
}
type Handler = (call: Call) => Reply;

const calls: Call[] = [];
const UID = '22222222-2222-4222-8222-222222222222';
const TID = '33333333-3333-4333-8333-333333333333';
const TID2 = '44444444-4444-4444-8444-444444444444';
const LID = '55555555-5555-4555-8555-555555555555';

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
    today: '2027-03-12',
    server_time: '2027-03-12T09:00:00Z',
    csrf_token: 'csrf-1',
    min_client_version: '0.0.0',
    settings: {},
  };
}

const MANAGER = ['catalog.read', 'catalog.write', 'search.use', 'mandate.write'];
const CASHIER = ['catalog.read', 'search.use'];

const unit = {
  id: UID,
  property_id: 'p1',
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
const person = (id: string, first: string, last: string) => ({
  id,
  kind: 'person',
  last_name: last,
  first_name: first,
  display_name: `${first} ${last}`,
  trade_activity: null,
  phone: '+22600000101',
  phone_is_whatsapp: true,
  email: null,
  unit_labels: [],
  balance: null,
  merged_into_id: null,
  archived_at: null,
  version: 1,
});

function lease(over: Record<string, unknown> = {}) {
  return {
    id: LID,
    unit_id: UID,
    unit_reference: 'A1',
    property_name: 'Zogona',
    tenants: [{ id: TID, display_name: 'Issouf Tapsoba', is_primary: true }],
    kind: 'residential',
    status: 'draft',
    start_date: '2027-03-01',
    end_date: null,
    rent: 75000,
    charges: 0,
    current_rent: 75000,
    current_charges: 0,
    due_day: 5,
    due_day_defaulted: false,
    deposit_amount: 200000,
    vat_included: false,
    amendments: [],
    warnings: ['lease.deposit_above_two_months'],
    version: 1,
    ...over,
  };
}

function installment(due: string, status: string, over: Record<string, unknown> = {}) {
  return {
    id: `i-${due}`,
    period: due.slice(0, 7),
    due_date: due,
    due_amount: 75000,
    allocated_amount: 0,
    remaining_amount: 75000,
    days_late: 0,
    status,
    lines: [
      {
        id: `l-${due}`,
        kind: 'rent',
        amount: 75000,
        due_date: due,
        label: null,
        voided: false,
        void_reason: null,
      },
    ],
    ...over,
  };
}

function install(routes: Record<string, Handler | Reply>) {
  vi.stubGlobal(
    'fetch',
    vi.fn((url: string, init?: RequestInit) => {
      const call: Call = {
        key: `${init?.method ?? 'GET'} ${url}`,
        body: typeof init?.body === 'string' ? (JSON.parse(init.body) as Record<string, unknown>) : undefined,
        headers: new Headers(init?.headers),
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

const LISTS = {
  'GET /api/units?limit=100': { status: 200, body: { items: [unit], next_cursor: null } },
  'GET /api/tenants?limit=100': {
    status: 200,
    body: {
      items: [person(TID, 'Issouf', 'Tapsoba'), person(TID2, 'Mariam', 'Ouédraogo')],
      next_cursor: null,
    },
  },
};

/** Étape 1 : lot A1 et le locataire Issouf Tapsoba. */
async function stepOne() {
  fireEvent.click(await screen.findByRole('button', { name: 'Créer un bail' }));
  await screen.findByRole('option', { name: 'A1 · Zogona' });
  fireEvent.change(screen.getByLabelText('Lot'), { target: { value: UID } });
  fireEvent.click(await screen.findByLabelText('Issouf Tapsoba'));
}

describe('S10 création d’un bail en trois étapes', () => {
  it('lot et colocataires, conditions, vérification puis activation ; montants en entiers', async () => {
    window.history.pushState(null, '', '/baux');
    let activated = false;
    install({
      'GET /api/me': { status: 200, body: me('manager', MANAGER) },
      'GET /api/leases': { status: 200, body: { items: [] } },
      ...LISTS,
      'POST /api/leases': { status: 201, body: lease() },
      [`POST /api/leases/${LID}/activate`]: () => {
        activated = true;
        return { status: 200, body: lease({ status: 'active' }) };
      },
      [`GET /api/leases/${LID}`]: () => ({
        status: 200,
        body: lease({ status: activated ? 'active' : 'draft' }),
      }),
      [`GET /api/leases/${LID}/installments`]: {
        status: 200,
        body: { items: [installment('2027-03-05', 'late', { days_late: 7 })] },
      },
    });
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Créer un bail' }));
    expect(screen.getByText('Étape 1 sur 3')).toBeTruthy();
    expect(screen.getByRole<HTMLButtonElement>('button', { name: 'Suivant' }).disabled).toBe(true);
    await screen.findByRole('option', { name: 'A1 · Zogona' });
    fireEvent.change(screen.getByLabelText('Lot'), { target: { value: UID } });
    fireEvent.click(await screen.findByLabelText('Issouf Tapsoba'));
    fireEvent.click(screen.getByLabelText('Mariam Ouédraogo'));
    // colocation : choix du locataire principal
    fireEvent.change(screen.getByLabelText(/Locataire principal/), { target: { value: TID2 } });
    fireEvent.click(screen.getByRole('button', { name: 'Suivant' }));
    expect(screen.getByText('Étape 2 sur 3')).toBeTruthy();
    // le loyer de référence du lot est proposé
    expect(screen.getByLabelText<HTMLInputElement>('Loyer (FCFA)').value).toBe('75000');
    fireEvent.change(screen.getByLabelText('Date de début'), { target: { value: '2027-03-01' } });
    fireEvent.change(screen.getByLabelText('Dépôt de garantie (FCFA)'), { target: { value: '200 000' } });
    fireEvent.click(screen.getByRole('button', { name: 'Suivant' }));
    expect(screen.getByText('Étape 3 sur 3')).toBeTruthy();
    expect(screen.getByText(/principal : Mariam Ouédraogo/)).toBeTruthy();
    expect(screen.getByText('200 000 FCFA')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Activer' }));
    await screen.findByRole('heading', { name: /Bail · A1 · Zogona/ });
    const body = sent('POST /api/leases')[0]?.body;
    expect(body?.['tenant_ids']).toEqual([TID, TID2]);
    expect(body?.['primary_tenant_id']).toBe(TID2);
    // montants en entiers FCFA (R-001)
    expect(body?.['rent']).toBe(75000);
    expect(body?.['deposit_amount']).toBe(200000);
    expect(body?.['due_day']).toBe(5);
    // activation idempotente (API_CONTRACTS.md §7.5)
    expect(sent(`POST /api/leases/${LID}/activate`)[0]?.headers.get('Idempotency-Key')).toBeTruthy();
    expect(window.location.pathname).toBe(`/baux/${LID}`);
  });

  it('un refus du serveur s’affiche à l’étape 3, sans quitter l’assistant ni activer', async () => {
    window.history.pushState(null, '', '/baux');
    install({
      'GET /api/me': { status: 200, body: me('manager', MANAGER) },
      'GET /api/leases': { status: 200, body: { items: [] } },
      ...LISTS,
      'POST /api/leases': problem(409, 'lease.overlap', {
        params: { start_date: '2027-01-01', end_date: '2027-12-31' },
      }),
    });
    render(<App />);
    await stepOne();
    fireEvent.click(screen.getByRole('button', { name: 'Suivant' }));
    fireEvent.change(screen.getByLabelText('Date de début'), { target: { value: '2027-03-01' } });
    fireEvent.click(screen.getByRole('button', { name: 'Suivant' }));
    fireEvent.click(screen.getByRole('button', { name: 'Enregistrer le brouillon' }));
    expect(
      await screen.findByText(
        'Ce lot a déjà un bail du 01/01/2027 au 31/12/2027. Terminez-le ou choisissez d’autres dates.',
      ),
    ).toBeTruthy();
    expect(screen.getByText('Étape 3 sur 3')).toBeTruthy();
    expect(sent(`POST /api/leases/${LID}/activate`)).toHaveLength(0);
  });

  it('activation refusée (lot en travaux) : le brouillon est gardé, un nouvel essai ne crée pas de second bail', async () => {
    window.history.pushState(null, '', '/baux');
    install({
      'GET /api/me': { status: 200, body: me('manager', MANAGER) },
      'GET /api/leases': { status: 200, body: { items: [] } },
      ...LISTS,
      'POST /api/leases': { status: 201, body: lease() },
      [`POST /api/leases/${LID}/activate`]: problem(409, 'unit.under_works'),
    });
    render(<App />);
    await stepOne();
    fireEvent.click(screen.getByRole('button', { name: 'Suivant' }));
    fireEvent.change(screen.getByLabelText('Date de début'), { target: { value: '2027-03-01' } });
    fireEvent.click(screen.getByRole('button', { name: 'Suivant' }));
    fireEvent.click(screen.getByRole('button', { name: 'Activer' }));
    expect(await screen.findByText(/Ce lot est en travaux.*Le brouillon est enregistré\./)).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Activer' }));
    await waitFor(() => {
      expect(sent(`POST /api/leases/${LID}/activate`)).toHaveLength(2);
    });
    expect(sent('POST /api/leases')).toHaveLength(1);
    expect(screen.getByRole('button', { name: 'Ouvrir le brouillon' })).toBeTruthy();
  });

  it('« Précédent » garde la saisie', async () => {
    window.history.pushState(null, '', '/baux');
    install({
      'GET /api/me': { status: 200, body: me('manager', MANAGER) },
      'GET /api/leases': { status: 200, body: { items: [] } },
      ...LISTS,
    });
    render(<App />);
    await stepOne();
    fireEvent.click(screen.getByRole('button', { name: 'Suivant' }));
    fireEvent.click(screen.getByRole('button', { name: 'Précédent' }));
    expect(screen.getByLabelText<HTMLInputElement>('Issouf Tapsoba').checked).toBe(true);
    expect(screen.getByLabelText<HTMLSelectElement>('Lot').value).toBe(UID);
  });
});

describe('S10 fiche du bail', () => {
  it('avertissement du dépôt, badges calculés par le serveur, avenant envoyé en entier', async () => {
    window.history.pushState(null, '', `/baux/${LID}`);
    install({
      'GET /api/me': { status: 200, body: me('manager', MANAGER) },
      [`GET /api/leases/${LID}`]: { status: 200, body: lease({ status: 'active' }) },
      [`GET /api/leases/${LID}/installments`]: {
        status: 200,
        body: {
          items: [installment('2027-03-05', 'late', { days_late: 7 }), installment('2027-04-05', 'upcoming')],
        },
      },
      [`POST /api/leases/${LID}/amendments`]: { status: 201, body: lease({ status: 'active' }) },
    });
    render(<App />);
    expect(await screen.findByText(/dépasse deux mois de loyer/)).toBeTruthy();
    const rows = await screen.findAllByTestId('installment');
    expect(within(rows[0] as HTMLElement).getByText('En retard · 7 jours')).toBeTruthy();
    expect(within(rows[1] as HTMLElement).getByText('À venir')).toBeTruthy();
    fireEvent.change(screen.getByLabelText('Date d’effet'), { target: { value: '2027-04-01' } });
    fireEvent.change(screen.getByLabelText('Nouveau loyer (FCFA)'), { target: { value: '80 000' } });
    fireEvent.click(screen.getByRole('button', { name: 'Ajouter l’avenant' }));
    await waitFor(() => {
      expect(sent(`POST /api/leases/${LID}/amendments`)).toHaveLength(1);
    });
    expect(sent(`POST /api/leases/${LID}/amendments`)[0]?.body).toEqual({
      effective_from: '2027-04-01',
      rent: 80000,
    });
  });

  it('le caissier n’a pas l’entrée « Baux » : une adresse tapée à la main affiche « Accès refusé »', async () => {
    window.history.pushState(null, '', `/baux/${LID}`);
    install({ 'GET /api/me': { status: 200, body: me('cashier', CASHIER) } });
    render(<App />);
    expect(await screen.findByRole('heading', { name: 'Accès refusé' })).toBeTruthy();
    expect(sent(`GET /api/leases/${LID}`)).toHaveLength(0);
  });
});
