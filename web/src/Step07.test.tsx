// Étape 07 : encaisser (S04), journal (S11), quittance (S25) ; F09, F10, DP-96. API simulée ; l'API réelle est couverte par
// `tests/db/tests/step07_payments.rs`. Données fictives (bloc `+226 00 00 …`).
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import type { Me } from './api/client';

interface Reply {
  status: number;
  body?: unknown;
  network?: boolean;
}
interface Call {
  key: string;
  body: Record<string, unknown> | undefined;
  headers: Headers;
}
type Handler = (call: Call) => Reply;

const calls: Call[] = [];
const LID = '55555555-5555-4555-8555-555555555555';
const TID = '33333333-3333-4333-8333-333333333333';

function problem(status: number, code: string, params: Record<string, unknown> = {}): Reply {
  return { status, body: { type: 'about:blank', title: code, status, detail: code, code, params } };
}

function me(role: string, permissions: string[]): Me {
  return {
    user: { id: 'u1', display_name: 'Awa Zongo' },
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

const CASHIER = ['payment.record', 'payment.read', 'receipt.issue', 'catalog.read', 'search.use'];

const lease = {
  id: LID,
  unit_id: 'u1',
  unit_reference: 'A1',
  property_name: 'Zogona',
  tenants: [{ id: TID, display_name: 'Issouf Tapsoba', is_primary: true }],
  kind: 'residential',
  status: 'active',
  start_date: '2026-11-01',
  end_date: null,
  rent: 75000,
  charges: 0,
  current_rent: 75000,
  current_charges: 0,
  due_day: 13,
  due_day_defaulted: false,
  deposit_amount: 0,
  vat_included: false,
  amendments: [],
  warnings: [],
  version: 1,
};

const context = {
  lease: {
    id: LID,
    status: 'active',
    kind: 'residential',
    due_day: 13,
    unit: { id: 'u1', label: 'A1', property_name: 'Zogona' },
  },
  tenants: [{ id: TID, display_name: 'Issouf Tapsoba', is_primary: true, phone: '+22600000101' }],
  balance: { remaining_amount: 75000, overdue_amount: 0, credit_amount: 0 },
  open_lines: [
    {
      installment_id: 'i1',
      installment_line_id: 'l1',
      period: '2026-11-01',
      due_date: '2026-11-13',
      kind: 'rent',
      label: null,
      remaining_amount: 75000,
      status: 'due',
    },
  ],
  suggested_amount: 75000,
  default_method: 'cash',
  today: '2026-11-13',
  confirm_threshold: 1000000,
};

function receipt(number: string, thirdParty = false) {
  return {
    id: 'r1',
    number,
    series: 'trial',
    status: 'issued',
    issued_on: '2026-11-13',
    amount: 50000,
    amount_in_words: 'cinquante mille francs CFA',
    remaining_after: 25000,
    advance_amount: 0,
    payment_id: 'p1',
    snapshot: {
      issuer: { name: 'Agence A', extra_mentions: null },
      receipt: {
        number,
        series: 'trial',
        watermark: 'ESSAI',
        issued_on: '2026-11-13',
        issued_by: 'Awa Zongo',
      },
      tenants: ['Issouf Tapsoba'],
      payer: thirdParty
        ? {
            kind: 'third_party',
            name: 'Moussa Tapsoba',
            mention: 'Payé par Moussa Tapsoba pour le compte de Issouf Tapsoba',
          }
        : { kind: 'tenant', name: 'Issouf Tapsoba' },
      unit: {
        reference: 'A1',
        property_name: 'Zogona',
        address: null,
        district: 'Zogona',
        city: 'Ouagadougou',
        vat_included: false,
      },
      payment: {
        amount: 50000,
        amount_text: '50 000 FCFA',
        amount_in_words: 'cinquante mille francs CFA',
        paid_on: '2026-11-13',
        method_label: 'Espèces',
        external_ref: null,
      },
      breakdown: [
        {
          period: '2026-11-01',
          period_label: 'novembre 2026',
          lines: [{ kind: 'rent', label: 'Loyer', amount: 50000 }],
        },
      ],
      advance_amount: 0,
      remaining_after: 25000,
    },
  };
}

const recorded = (number: string, thirdParty = false): Reply => ({
  status: 201,
  body: {
    payment: { id: 'p1', amount: 50000, allocations: [], unallocated_amount: 0 },
    receipt: receipt(number, thirdParty),
    installments: [],
    warnings: [],
  },
});

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
      if (r.network === true) return Promise.reject(new TypeError('Failed to fetch'));
      const text = r.body === undefined ? '' : JSON.stringify(r.body);
      return Promise.resolve(new Response(r.status === 204 ? null : text, { status: r.status }));
    }),
  );
}

const sent = (key: string) => calls.filter((c) => c.key === key);

const BASE: Record<string, Reply> = {
  'GET /api/me': { status: 200, body: me('cashier', CASHIER) },
  'GET /api/leases': { status: 200, body: { items: [lease] } },
  [`GET /api/leases/${LID}/payment-context`]: { status: 200, body: context },
  'POST /api/payments/preview': {
    status: 200,
    body: {
      allocations: [
        {
          installment_line_id: 'l1',
          installment_id: 'i1',
          period: '2026-11-01',
          due_date: '2026-11-13',
          kind: 'rent',
          label: null,
          amount: 50000,
        },
      ],
      surplus_amount: 0,
      credit_amount: 0,
      remaining_after: 25000,
      requires_surplus_confirmation: false,
      requires_high_amount_confirmation: false,
    },
  },
};

beforeEach(() => {
  calls.length = 0;
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

async function openForm() {
  window.history.pushState(null, '', '/encaisser');
  render(<App />);
  fireEvent.click(await screen.findByRole('button', { name: /Issouf Tapsoba · A1 · Zogona/ }));
  await waitFor(() => {
    expect(screen.getByLabelText<HTMLInputElement>('Montant (FCFA)').value).toBe('75000');
  });
}

describe('S04 encaisser', () => {
  it('coupure : la saisie et la clé sont conservées, le nouvel essai réutilise la clé (R-042)', async () => {
    let attempt = 0;
    install({
      ...BASE,
      'POST /api/payments': () => {
        attempt += 1;
        return attempt === 1 ? { status: 0, network: true } : recorded('ESSAI-REC-2026-000001');
      },
    });
    await openForm();
    fireEvent.change(screen.getByLabelText('Montant (FCFA)'), { target: { value: '50 000' } });
    fireEvent.click(screen.getByRole('button', { name: 'Encaisser et envoyer la quittance' }));
    expect(await screen.findByText(/Connexion perdue. Vos informations sont conservées/)).toBeTruthy();
    expect(screen.getByLabelText<HTMLInputElement>('Montant (FCFA)').value).toBe('50 000');
    fireEvent.click(screen.getByRole('button', { name: 'Encaisser et envoyer la quittance' }));
    expect(await screen.findByRole('heading', { name: 'Quittance n° ESSAI-REC-2026-000001' })).toBeTruthy();
    const posts = sent('POST /api/payments');
    expect(posts).toHaveLength(2);
    const k1 = posts[0]?.headers.get('Idempotency-Key');
    expect(k1).toBeTruthy();
    expect(posts[1]?.headers.get('Idempotency-Key')).toBe(k1);
    // montant en entier (R-001), payé par le locataire principal par défaut (R-088)
    expect(posts[0]?.body?.['amount']).toBe(50000);
    expect(posts[0]?.body?.['payer']).toEqual({ kind: 'tenant', tenant_id: TID });
    expect(posts[0]?.body?.['paid_on']).toBe('2026-11-13');
    // S25 : montant en lettres, solde restant et filigrane d'essai, depuis l'instantané
    expect(screen.getByText(/cinquante mille francs CFA/)).toBeTruthy();
    expect(screen.getByText(/Solde restant/).textContent).toContain('25 000 FCFA');
    expect(screen.getByLabelText('Quittance d’essai')).toBeTruthy();
    // « Encaisser un autre loyer » : nouveau formulaire, nouvelle clé
    fireEvent.click(screen.getByRole('button', { name: 'Encaisser un autre loyer' }));
    fireEvent.click(await screen.findByRole('button', { name: /Issouf Tapsoba · A1 · Zogona/ }));
    await waitFor(() => {
      expect(screen.getByLabelText<HTMLInputElement>('Montant (FCFA)').value).toBe('75000');
    });
    fireEvent.click(screen.getByRole('button', { name: 'Encaisser et envoyer la quittance' }));
    await waitFor(() => {
      expect(sent('POST /api/payments')).toHaveLength(3);
    });
    expect(sent('POST /api/payments')[2]?.headers.get('Idempotency-Key')).not.toBe(k1);
  });

  it('seuil (R-038) puis avance (R-046) : chaque confirmation renvoie la même clé avec son indicateur', async () => {
    install({
      ...BASE,
      'POST /api/payments': (c) => {
        if (c.body?.['confirm_high_amount'] !== true)
          return problem(422, 'payment.confirmation_required', { threshold: 1000000 });
        if (c.body['surplus'] !== 'advance')
          return problem(422, 'payment.surplus_not_confirmed', { surplus_amount: 1175000 });
        return recorded('ESSAI-REC-2026-000002');
      },
    });
    await openForm();
    fireEvent.change(screen.getByLabelText('Montant (FCFA)'), { target: { value: '1250000' } });
    fireEvent.click(screen.getByRole('button', { name: 'Encaisser et envoyer la quittance' }));
    expect((await screen.findByRole('alertdialog')).textContent).toContain('1 000 000 FCFA');
    fireEvent.click(screen.getByRole('button', { name: 'Confirmer' }));
    expect((await screen.findByRole('alertdialog')).textContent).toContain('1 175 000 FCFA en avance');
    fireEvent.click(screen.getByRole('button', { name: 'Confirmer' }));
    expect(await screen.findByRole('heading', { name: 'Quittance n° ESSAI-REC-2026-000002' })).toBeTruthy();
    const posts = sent('POST /api/payments');
    expect(posts).toHaveLength(3);
    expect(new Set(posts.map((p) => p.headers.get('Idempotency-Key'))).size).toBe(1);
    expect(posts[2]?.body).toMatchObject({ confirm_high_amount: true, surplus: 'advance', amount: 1250000 });
  });

  it('référence déjà utilisée (R-102) : avertissement bloquant avec le numéro, puis confirmation explicite', async () => {
    install({
      ...BASE,
      'POST /api/payments': (c) =>
        c.body?.['acknowledge_external_ref_duplicate'] === true
          ? recorded('ESSAI-REC-2026-000003', true)
          : problem(409, 'payment.external_ref_duplicate', { receipt_number: 'ESSAI-REC-2026-000001' }),
    });
    await openForm();
    fireEvent.change(screen.getByLabelText('Mode'), { target: { value: 'mobile_money' } });
    fireEvent.change(screen.getByLabelText('Référence'), { target: { value: 'MP2611.AB' } });
    fireEvent.change(screen.getByLabelText('Payé par'), { target: { value: 'third_party' } });
    fireEvent.change(screen.getByLabelText('Nom du tiers'), { target: { value: 'Moussa Tapsoba' } });
    fireEvent.click(screen.getByRole('button', { name: 'Encaisser et envoyer la quittance' }));
    expect((await screen.findByRole('alertdialog')).textContent).toContain('ESSAI-REC-2026-000001');
    fireEvent.click(screen.getByRole('button', { name: 'Confirmer' }));
    expect(await screen.findByText('Payé par Moussa Tapsoba pour le compte de Issouf Tapsoba')).toBeTruthy();
    expect(sent('POST /api/payments')[0]?.body).toMatchObject({
      method: 'mobile_money',
      external_ref: 'MP2611.AB',
      payer: { kind: 'third_party', third_party_name: 'Moussa Tapsoba' },
    });
  });
});

describe('S11 journal', () => {
  it('liste les paiements validés avec leur numéro, sans action de modification', async () => {
    window.history.pushState(null, '', '/paiements');
    install({
      'GET /api/me': { status: 200, body: me('cashier', CASHIER) },
      'GET /api/payments': {
        status: 200,
        body: {
          items: [
            {
              id: 'p1',
              status: 'validated',
              lease_id: LID,
              amount: 50000,
              method: 'cash',
              external_ref: null,
              paid_on: '2026-11-13',
              recorded_at: '2026-11-13T09:15:42Z',
              received_by: { id: 'u1', display_name: 'Awa Zongo' },
              receipt_id: 'r1',
              receipt_number: 'ESSAI-REC-2026-000001',
              unit_reference: 'A1',
              property_name: 'Zogona',
              tenants: 'Issouf Tapsoba',
            },
          ],
        },
      },
    });
    render(<App />);
    const row = await screen.findByTestId('payment-row');
    expect(row.textContent).toContain('ESSAI-REC-2026-000001');
    expect(row.textContent).toContain('50 000 FCFA');
    expect(row.textContent).toContain('Validé');
    expect(screen.queryByRole('button', { name: /Modifier|Supprimer/ })).toBeNull();
  });
});
