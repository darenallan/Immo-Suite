// Client d'API typé à partir du contrat généré (`schema.d.ts`, DP-84). Aucun jeton n'est écrit dans le stockage du navigateur :
// le cookie de session est `HttpOnly`, et le jeton anti-falsification vit en mémoire (React) seulement.
import { appVersion } from '../version';
import type { components } from './schema';

export type Me = components['schemas']['MeBody'];
export type LoginBody = components['schemas']['LoginBody'];
export type OrganizationChoice = components['schemas']['OrganizationChoiceBody'];
export type OrganizationDetail = components['schemas']['OrganizationDetailBody'];

/** Erreur du contrat (`application/problem+json`) : le front s'appuie sur `code`, jamais sur `title` ni `detail`. */
export class ApiProblem extends Error {
  readonly status: number;
  readonly code: string;
  readonly requestId: string | undefined;
  readonly retryAfterSeconds: number | undefined;

  constructor(status: number, code: string, detail: string, requestId?: string, retryAfterSeconds?: number) {
    super(detail);
    this.name = 'ApiProblem';
    this.status = status;
    this.code = code;
    this.requestId = requestId;
    this.retryAfterSeconds = retryAfterSeconds;
  }
}

interface Problem {
  code?: unknown;
  detail?: unknown;
  request_id?: unknown;
  retry_after_seconds?: unknown;
}

function asProblem(status: number, body: unknown): ApiProblem {
  const p: Problem = typeof body === 'object' && body !== null ? body : {};
  return new ApiProblem(
    status,
    typeof p.code === 'string' ? p.code : 'server.unexpected',
    typeof p.detail === 'string' ? p.detail : '',
    typeof p.request_id === 'string' ? p.request_id : undefined,
    typeof p.retry_after_seconds === 'number' ? p.retry_after_seconds : undefined,
  );
}

interface Options {
  body?: unknown;
  csrf?: string | undefined;
}

async function request<T>(method: 'GET' | 'POST', path: string, options: Options = {}): Promise<T> {
  const headers: Record<string, string> = { 'X-Client-Version': appVersion };
  if (options.body !== undefined) headers['Content-Type'] = 'application/json';
  if (method !== 'GET' && options.csrf !== undefined) headers['X-CSRF-Token'] = options.csrf;
  let response: Response;
  try {
    response = await fetch(path, {
      method,
      headers,
      credentials: 'same-origin',
      ...(options.body !== undefined ? { body: JSON.stringify(options.body) } : {}),
    });
  } catch {
    throw new ApiProblem(0, 'network.unreachable', '');
  }
  if (response.status === 204) return undefined as T;
  const text = await response.text();
  let parsed: unknown = null;
  try {
    parsed = text === '' ? null : (JSON.parse(text) as unknown);
  } catch {
    parsed = null;
  }
  if (!response.ok) throw asProblem(response.status, parsed);
  return parsed as T;
}

export const api = {
  login: (identifier: string, password: string) =>
    request<LoginBody>('POST', '/api/auth/login', { body: { identifier, password } }),
  me: () => request<Me>('GET', '/api/me'),
  selectOrganization: (organizationId: string, csrf: string) =>
    request<Me>('POST', '/api/me/organization', { body: { organization_id: organizationId }, csrf }),
  logout: (csrf: string | undefined) => request<undefined>('POST', '/api/auth/logout', { csrf }),
  logoutAll: (csrf: string) => request<undefined>('POST', '/api/auth/logout-all', { csrf }),
};
