// Étape 04 : mesure d'usage sans tiers, capture des erreurs, formulaire « Signaler un problème » (F39-observation §7 et §9).
// API simulée ; l'API réelle est couverte par `tests/db/tests/step04_*.rs`.
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from '../App';
import type { Me } from '../api/client';
import { descriptionError } from './FeedbackDialog';
import { resetDiagnostics } from './diagnostics';
import { describe as describeError, reportError, resetErrorCapture } from './errors';
import {
  configureObservation,
  flush,
  queuedCount,
  resetObservation,
  screenOf,
  track,
  trackValidation,
  widthClass,
} from './track';

interface Call {
  key: string;
  body: Record<string, unknown> | undefined;
  headers: Record<string, string>;
}
type Reply = { status: number; body?: unknown; headers?: Record<string, string> };

const calls: Call[] = [];

function install(routes: Record<string, Reply | ((call: Call) => Reply)>) {
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
          ? { status: 404, body: { code: 'resource.not_found' } }
          : typeof route === 'function'
            ? route(call)
            : route;
      const text = r.body === undefined ? '' : JSON.stringify(r.body);
      const responseInit: ResponseInit =
        r.headers === undefined ? { status: r.status } : { status: r.status, headers: r.headers };
      return Promise.resolve(new Response(r.status === 204 ? null : text, responseInit));
    }),
  );
}

const sent = (key: string) => calls.filter((c) => c.key === key);

function me(role = 'manager'): Me {
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
    permissions: ['feedback.create', 'events.create'],
    today: '2026-11-13',
    server_time: '2026-11-13T09:15:00Z',
    csrf_token: 'csrf-1',
    min_client_version: '0.0.0',
    settings: {},
  };
}

beforeEach(() => {
  calls.length = 0;
  window.history.pushState(null, '', '/');
  resetObservation();
  resetDiagnostics();
  resetErrorCapture();
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe('mesure d’usage : file en mémoire et envoi par lots', () => {
  it('classe les adresses en écrans de la liste et les largeurs en trois classes', () => {
    expect(screenOf('/')).toBe('S03');
    expect(screenOf('/impayes')).toBe('S05');
    expect(screenOf('/journal-audit')).toBe('S21');
    expect(screenOf('/une/adresse/inconnue')).toBe('S00');
    expect([widthClass(390), widthClass(768), widthClass(1440)]).toEqual(['mobile', 'tablet', 'desktop']);
  });

  it('envoie un lot de 20 événements, sans attendre ni rejouer', () => {
    install({ 'POST /api/events': { status: 204 } });
    configureObservation(() => 'csrf-1');
    for (let i = 0; i < 20; i += 1) {
      track({ name: 'screen_viewed', properties: { screen: 'S05', width_class: 'mobile' } });
    }
    const batches = sent('POST /api/events');
    expect(batches).toHaveLength(1);
    const events = batches[0]?.body?.events as { name: string; client_version: string }[];
    expect(events).toHaveLength(20);
    expect(events[0]?.name).toBe('screen_viewed');
    expect(events[0]?.client_version).toMatch(/^\d+\.\d+\.\d+/);
    expect(batches[0]?.headers['X-CSRF-Token']).toBe('csrf-1');
    expect(queuedCount()).toBe(0);
  });

  it('un lot incomplet part à la mise en arrière-plan ; au-delà de 20, deux lots', () => {
    install({ 'POST /api/events': { status: 204 } });
    configureObservation(() => 'csrf-1');
    for (let i = 0; i < 25; i += 1) {
      track({ name: 'screen_viewed', properties: { screen: 'S01', width_class: 'desktop' } });
    }
    expect(sent('POST /api/events')).toHaveLength(1);
    expect(queuedCount()).toBe(5);
    flush();
    expect(sent('POST /api/events')).toHaveLength(2);
    expect((sent('POST /api/events')[1]?.body?.events as unknown[]).length).toBe(5);
  });

  it('sans session (jeton absent) rien n’est conservé ni envoyé', () => {
    install({ 'POST /api/events': { status: 204 } });
    configureObservation(() => undefined);
    track({ name: 'screen_viewed', properties: { screen: 'S05', width_class: 'mobile' } });
    flush();
    expect(queuedCount()).toBe(0);
    expect(sent('POST /api/events')).toHaveLength(0);
  });

  it('une limite de débit espace les envois sans rien afficher', async () => {
    install({ 'POST /api/events': { status: 429, headers: { 'Retry-After': '60' } } });
    configureObservation(() => 'csrf-1');
    for (let i = 0; i < 20; i += 1) {
      track({ name: 'screen_viewed', properties: { screen: 'S05', width_class: 'mobile' } });
    }
    await waitFor(() => {
      expect(sent('POST /api/events')).toHaveLength(1);
    });
    await Promise.resolve();
    track({ name: 'screen_viewed', properties: { screen: 'S05', width_class: 'mobile' } });
    flush();
    expect(sent('POST /api/events')).toHaveLength(1);
  });

  it('hors connexion : l’erreur réseau est avalée, rien n’est rejoué', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.reject(new TypeError('Failed to fetch'))),
    );
    configureObservation(() => 'csrf-1');
    for (let i = 0; i < 20; i += 1) {
      track({ name: 'screen_viewed', properties: { screen: 'S05', width_class: 'mobile' } });
    }
    await Promise.resolve();
    expect(queuedCount()).toBe(0);
    expect(vi.mocked(fetch)).toHaveBeenCalledTimes(1);
  });

  it('une erreur de saisie n’envoie que le champ et le code, jamais la valeur', () => {
    install({ 'POST /api/events': { status: 204 } });
    configureObservation(() => 'csrf-1');
    trackValidation('login_phone', 'validation.phone_format');
    trackValidation('Nom du Locataire', 'validation.required');
    trackValidation('x', 'validation.required');
    trackValidation('amount', 'pas un code');
    flush();
    const events = sent('POST /api/events')[0]?.body?.events as { name: string; properties: unknown }[];
    expect(events).toEqual([
      expect.objectContaining({
        name: 'validation_error',
        properties: { screen: 'S00', field: 'login_phone', rule_code: 'validation.phone_format' },
      }),
      expect.objectContaining({
        name: 'validation_error',
        properties: { screen: 'S00', field: 'nom_du_locataire', rule_code: 'validation.required' },
      }),
    ]);
  });
});

describe('erreurs JavaScript', () => {
  it('le message porte le type d’erreur, jamais la pile', () => {
    expect(describeError(new TypeError('x is undefined'), false)).toBe('TypeError: x is undefined');
    expect(describeError(new RangeError('trop grand'), true)).toBe(
      'unhandled_rejection: RangeError: trop grand',
    );
    expect(describeError('texte brut', false)).toBe('Error: texte brut');
  });

  it('au plus une erreur identique par minute et cinq par session', () => {
    install({ 'POST /api/client-errors': { status: 204 } });
    const csrf = () => 'csrf-1';
    reportError('TypeError: a', csrf);
    reportError('TypeError: a', csrf);
    expect(sent('POST /api/client-errors')).toHaveLength(1);
    for (let i = 0; i < 10; i += 1) reportError(`TypeError: ${String(i)}`, csrf);
    expect(sent('POST /api/client-errors')).toHaveLength(5);
    const body = sent('POST /api/client-errors')[0]?.body;
    expect(Object.keys(body ?? {}).sort()).toEqual(['client_version', 'message', 'request_ids', 'screen']);
  });

  it('sans session rien n’est envoyé', () => {
    install({ 'POST /api/client-errors': { status: 204 } });
    reportError('TypeError: a', () => undefined);
    expect(sent('POST /api/client-errors')).toHaveLength(0);
  });
});

describe('« Signaler un problème »', () => {
  it('la description vaut 5 à 4 000 caractères (points de code)', () => {
    expect(descriptionError('abc')).toBeDefined();
    expect(descriptionError('     ')).toBeDefined();
    expect(descriptionError('abcde')).toBeUndefined();
    expect(descriptionError('é'.repeat(4000))).toBeUndefined();
    expect(descriptionError('é'.repeat(4001))).toBeDefined();
  });

  const feedbackRoutes = (reply: Reply) => ({
    'GET /api/me': { status: 200, body: me(), headers: { 'X-Request-Id': 'INC-ABC123' } },
    'POST /api/feedback': reply,
    'POST /api/events': { status: 204 },
  });

  async function openForm() {
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: /Menu utilisateur/ }));
    fireEvent.click(screen.getByRole('button', { name: 'Signaler un problème' }));
    return screen.findByRole('dialog', { name: 'Signaler un problème' });
  }

  it('une description trop courte affiche le résumé d’erreur et n’envoie rien', async () => {
    install(feedbackRoutes({ status: 201, body: { id: 'x', reference: 'SIG-000001' } }));
    await openForm();
    fireEvent.change(screen.getByLabelText('Décrivez ce qui s’est passé'), { target: { value: 'abc' } });
    fireEvent.click(screen.getByRole('button', { name: 'Envoyer' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Décrivez le problème en au moins 5 caractères.',
    );
    expect(sent('POST /api/feedback')).toHaveLength(0);
  });

  it('envoie la catégorie, la description, le rappel et le contexte, avec une clé d’idempotence, puis affiche la référence', async () => {
    install(feedbackRoutes({ status: 201, body: { id: 'f1', reference: 'SIG-000042' } }));
    await openForm();
    fireEvent.click(screen.getByLabelText('Une demande'));
    fireEvent.change(screen.getByLabelText('Décrivez ce qui s’est passé'), {
      target: { value: 'Le bouton ne répond plus' },
    });
    fireEvent.click(screen.getByLabelText('Vous pouvez me rappeler'));
    fireEvent.click(screen.getByRole('button', { name: 'Envoyer' }));
    await waitFor(() => {
      expect(screen.getByText('Merci. Votre référence : SIG-000042')).toBeInTheDocument();
    });
    expect(screen.queryByRole('dialog')).toBeNull();
    const call = sent('POST /api/feedback')[0];
    expect(call?.body).toMatchObject({
      category: 'request',
      description: 'Le bouton ne répond plus',
      can_call_back: true,
      context: { screen: 'S03', path: '/', last_request_ids: ['INC-ABC123'] },
    });
    expect(call?.headers['Idempotency-Key']).toMatch(/^[0-9a-f-]{36}$/);
    expect(call?.headers['X-CSRF-Token']).toBe('csrf-1');
    // ni jeton, ni paramètre de requête dans le contexte
    expect(JSON.stringify(call?.body)).not.toMatch(/csrf|token|\?/);
  });

  it('une limite de débit affiche le message du catalogue et garde la description', async () => {
    install(
      feedbackRoutes({
        status: 429,
        body: { code: 'rate_limit.exceeded', retry_after_seconds: 120 },
        headers: { 'Retry-After': '120' },
      }),
    );
    await openForm();
    fireEvent.change(screen.getByLabelText('Décrivez ce qui s’est passé'), {
      target: { value: 'Un message assez long' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Envoyer' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Trop de demandes. Réessayez dans 2 minutes.');
    expect(screen.getByLabelText('Décrivez ce qui s’est passé')).toHaveValue('Un message assez long');
  });

  it('Échap ferme le formulaire sans rien envoyer', async () => {
    install(feedbackRoutes({ status: 201, body: { id: 'x', reference: 'SIG-000001' } }));
    await openForm();
    fireEvent.keyDown(document, { key: 'Escape' });
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(sent('POST /api/feedback')).toHaveLength(0);
  });
});

describe('écrans consultés', () => {
  it('l’accueil est compté une fois la session ouverte, envoyé à la mise en arrière-plan', async () => {
    install({ 'GET /api/me': { status: 200, body: me() }, 'POST /api/events': { status: 204 } });
    render(<App />);
    await screen.findByTestId('org-name');
    Object.defineProperty(document, 'visibilityState', { configurable: true, get: () => 'hidden' });
    document.dispatchEvent(new Event('visibilitychange'));
    Object.defineProperty(document, 'visibilityState', { configurable: true, get: () => 'visible' });
    const events = sent('POST /api/events')[0]?.body?.events as { name: string; properties: unknown }[];
    expect(events[0]).toMatchObject({
      name: 'screen_viewed',
      properties: {
        screen: 'S03',
        width_class: expect.stringMatching(/^(mobile|tablet|desktop)$/) as string,
      },
    });
  });
});
