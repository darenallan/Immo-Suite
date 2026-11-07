// Client d'API typé à partir du contrat généré (`schema.d.ts`, DP-84). Aucun jeton n'est écrit dans le stockage du navigateur :
// le cookie de session est `HttpOnly`, et le jeton anti-falsification vit en mémoire (React) seulement.
import { appVersion } from '../version';
import { noteError, noteRequest } from '../observation/diagnostics';
import { trackValidation } from '../observation/track';
import type { components } from './schema';

type Schemas = components['schemas'];

export type Me = Schemas['MeBody'];
export type LoginBody = Schemas['LoginBody'];
export type OrganizationChoice = Schemas['OrganizationChoiceBody'];
export type OrganizationDetail = Schemas['OrganizationDetailBody'];
export type Member = Schemas['MemberBody'];
export type Invitation = Schemas['InvitationBody'];
export type CreatedInvitation = Schemas['CreatedInvitationBody'];
export type InvitationPreview = Schemas['InvitationPreviewBody'];
export type RoleSetting = Schemas['RoleSettingBody'];
export type Settings = Schemas['SettingsBody'];
export type SupportOverview = Schemas['SupportOverviewBody'];
export type SupportGrant = Schemas['SupportGrantBody'];
export type AuditPage = Schemas['AuditPageBody'];
export type AuditRow = Schemas['AuditRowBody'];
export type TotpEnrollment = Schemas['TotpEnrollmentBody'];
export type ResetLink = Schemas['ResetLinkBody'];
export type FeedbackPayload = Schemas['FeedbackPayload'];
export type FeedbackCreated = Schemas['FeedbackCreatedBody'];
export type Property = Schemas['PropertyBody'];
export type Unit = Schemas['UnitBody'];
export type Tenant = Schemas['TenantBody'];
export type Owner = Schemas['OwnerBody'];
export type SearchResult = Schemas['SearchBody'];
export interface PageOf<T> {
  items: T[];
  next_cursor?: string | null;
}

/** Une erreur de champ (API_CONTRACTS.md §3.3). */
export interface FieldError {
  field: string;
  code: string;
}

/** Erreur du contrat (`application/problem+json`) : le front s'appuie sur `code`, jamais sur `title` ni `detail`. */
export class ApiProblem extends Error {
  readonly status: number;
  readonly code: string;
  readonly requestId: string | undefined;
  readonly retryAfterSeconds: number | undefined;
  readonly reason: string | undefined;
  readonly fields: readonly FieldError[];
  /** `params` du problème (par exemple `conflict` d'un doublon de téléphone, `count` d'un mandant propriétaire de biens). */
  params: Record<string, unknown> = {};

  constructor(
    status: number,
    code: string,
    detail: string,
    requestId?: string,
    retryAfterSeconds?: number,
    reason?: string,
    fields: readonly FieldError[] = [],
  ) {
    super(detail);
    this.name = 'ApiProblem';
    this.status = status;
    this.code = code;
    this.requestId = requestId;
    this.retryAfterSeconds = retryAfterSeconds;
    this.reason = reason;
    this.fields = fields;
  }
}

// ───────── étape 07 : paiements et quittances (corps décrits dans API_CONTRACTS.md §6.1) ─────────

export interface OpenLineRow {
  installment_id: string;
  installment_line_id: string;
  period: string;
  due_date: string;
  kind: string;
  label: string | null;
  remaining_amount: number;
  status: string;
}
export interface PaymentContext {
  lease: {
    id: string;
    status: string;
    kind: string;
    due_day: number;
    unit: { id: string; label: string; property_name: string };
  };
  tenants: { id: string; display_name: string; is_primary: boolean; phone: string | null }[];
  balance: { remaining_amount: number; overdue_amount: number; credit_amount: number };
  open_lines: OpenLineRow[];
  suggested_amount: number;
  default_method: string;
  today: string;
  confirm_threshold: number;
}
export interface AllocationRow {
  installment_line_id: string;
  installment_id: string | null;
  period: string | null;
  due_date: string | null;
  kind: string | null;
  label: string | null;
  amount: number;
}
export interface PaymentPreview {
  allocations: AllocationRow[];
  surplus_amount: number;
  credit_amount: number;
  remaining_after: number;
  requires_surplus_confirmation: boolean;
  requires_high_amount_confirmation: boolean;
}
export interface ReceiptSnapshot {
  issuer: { name: string; extra_mentions: string | null };
  receipt: { number: string; series: string; watermark: string | null; issued_on: string; issued_by: string };
  tenants: string[];
  payer: { kind: string; name: string | null; mention?: string };
  unit: {
    reference: string;
    property_name: string;
    address: string | null;
    district: string | null;
    city: string | null;
    vat_included: boolean;
  };
  payment: {
    amount: number;
    amount_text: string;
    amount_in_words: string;
    paid_on: string;
    method_label: string;
    external_ref: string | null;
  };
  breakdown: {
    period: string;
    period_label: string;
    lines: { kind: string; label: string; amount: number }[];
  }[];
  advance_amount: number;
  remaining_after: number;
}
export interface ReceiptBody {
  id: string;
  number: string;
  series: string;
  status: string;
  issued_on: string;
  amount: number;
  amount_in_words: string;
  remaining_after: number;
  advance_amount: number;
  payment_id: string;
  snapshot: ReceiptSnapshot;
}
export interface PaymentRow {
  id: string;
  status: string;
  lease_id: string;
  amount: number;
  method: string;
  external_ref: string | null;
  paid_on: string;
  recorded_at: string;
  received_by: { id: string; display_name: string };
  receipt_id: string | null;
  receipt_number: string | null;
  unit_reference: string;
  property_name: string;
  tenants: string | null;
}
export interface RecordedPayment {
  payment: { id: string; amount: number; allocations: AllocationRow[]; unallocated_amount: number };
  receipt: ReceiptBody;
  installments: { id: string; period: string; status: string; remaining_amount: number }[];
  warnings: string[];
}

interface Problem {
  code?: unknown;
  detail?: unknown;
  request_id?: unknown;
  retry_after_seconds?: unknown;
  params?: { reason?: unknown };
  errors?: unknown;
}

function asFields(value: unknown): FieldError[] {
  if (!Array.isArray(value)) return [];
  const out: FieldError[] = [];
  for (const item of value as unknown[]) {
    const e = item as { field?: unknown; code?: unknown } | null;
    if (typeof e?.field === 'string' && typeof e.code === 'string')
      out.push({ field: e.field, code: e.code });
  }
  return out;
}

function asProblem(status: number, body: unknown): ApiProblem {
  const p: Problem = typeof body === 'object' && body !== null ? body : {};
  const problem = new ApiProblem(
    status,
    typeof p.code === 'string' ? p.code : 'server.unexpected',
    typeof p.detail === 'string' ? p.detail : '',
    typeof p.request_id === 'string' ? p.request_id : undefined,
    typeof p.retry_after_seconds === 'number' ? p.retry_after_seconds : undefined,
    typeof p.params?.reason === 'string' ? p.params.reason : undefined,
    asFields(p.errors),
  );
  if (typeof p.params === 'object') problem.params = p.params;
  return problem;
}

type Method = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';

interface Options {
  body?: unknown;
  csrf?: string | undefined;
  /** Version lue, pour `If-Match` (verrouillage optimiste, décision n° 64). */
  version?: number;
  /** Clé d'idempotence (UUID) : un rejeu avec la même clé ne crée rien de plus. */
  idempotencyKey?: string;
}

async function request<T>(method: Method, path: string, options: Options = {}): Promise<T> {
  const headers: Record<string, string> = { 'X-Client-Version': appVersion };
  if (options.body !== undefined) headers['Content-Type'] = 'application/json';
  if (method !== 'GET' && options.csrf !== undefined) headers['X-CSRF-Token'] = options.csrf;
  if (options.version !== undefined) headers['If-Match'] = `"${String(options.version)}"`;
  if (options.idempotencyKey !== undefined) headers['Idempotency-Key'] = options.idempotencyKey;
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
  noteRequest(response.headers.get('X-Request-Id'));
  if (response.status === 204) return undefined as T;
  const text = await response.text();
  let parsed: unknown = null;
  try {
    parsed = text === '' ? null : (JSON.parse(text) as unknown);
  } catch {
    parsed = null;
  }
  if (!response.ok) {
    const problem = asProblem(response.status, parsed);
    noteError(problem.code);
    // mesure d'usage : champ et code de règle seulement, jamais la valeur saisie
    for (const f of problem.fields) trackValidation(f.field, f.code);
    throw problem;
  }
  return parsed as T;
}

const enc = encodeURIComponent;

/** Filtres de lecture du journal (F03 §8.1). */
export interface AuditFilters {
  actor_id?: string;
  action?: string;
  object_type?: string;
  from?: string;
  to?: string;
  actor_kind?: string;
  cursor?: string;
}

function query(
  filters:
    AuditFilters | ListFilters | { unit_id?: string; tenant_id?: string } | { lease_id?: string; q?: string },
): string {
  const parts = Object.entries(filters as Record<string, string | undefined>)
    .filter(([, v]) => v !== undefined && v !== '')
    .map(([k, v]) => `${k}=${enc(String(v))}`);
  return parts.length === 0 ? '' : `?${parts.join('&')}`;
}

export const api = {
  login: (identifier: string, password: string) =>
    request<LoginBody>('POST', '/api/auth/login', { body: { identifier, password } }),
  totpVerify: (challenge: string, code: string) =>
    request<LoginBody>('POST', '/api/auth/totp/verify', { body: { challenge, code } }),
  me: () => request<Me>('GET', '/api/me'),
  selectOrganization: (organizationId: string, csrf: string) =>
    request<Me>('POST', '/api/me/organization', { body: { organization_id: organizationId }, csrf }),
  logout: (csrf: string | undefined) => request<undefined>('POST', '/api/auth/logout', { csrf }),
  logoutAll: (csrf: string) => request<undefined>('POST', '/api/auth/logout-all', { csrf }),
  reauth: (password: string, csrf: string) =>
    request<undefined>('POST', '/api/auth/reauth', { body: { password }, csrf }),

  // compte
  changePassword: (current: string, next: string, csrf: string) =>
    request<undefined>('PUT', '/api/me/password', {
      body: { current_password: current, new_password: next },
      csrf,
    }),
  totpBegin: (csrf: string) => request<TotpEnrollment>('POST', '/api/me/totp', { csrf }),
  totpConfirm: (code: string, csrf: string) =>
    request<{ backup_codes: string[] }>('POST', '/api/me/totp/confirm', { body: { code }, csrf }),

  // page publique d'invitation et de réinitialisation (par jeton)
  invitation: (token: string) => request<InvitationPreview>('GET', `/api/invitations/${enc(token)}`),
  acceptInvitation: (
    token: string,
    displayName: string | undefined,
    password: string | undefined,
    csrf?: string,
  ) =>
    request<LoginBody>('POST', '/api/invitations/accept', {
      body: { token, display_name: displayName, password },
      csrf,
    }),
  completeReset: (token: string, newPassword: string) =>
    request<undefined>('POST', '/api/auth/password-reset/complete', {
      body: { token, new_password: newPassword },
    }),

  // membres et invitations (S18)
  users: () => request<{ users: Member[] }>('GET', '/api/users'),
  changeRole: (id: string, role: string, csrf: string) =>
    request<Member>('PATCH', `/api/users/${enc(id)}`, { body: { role }, csrf }),
  disableUser: (id: string, csrf: string) =>
    request<Member>('POST', `/api/users/${enc(id)}/disable`, { csrf }),
  enableUser: (id: string, csrf: string) => request<Member>('POST', `/api/users/${enc(id)}/enable`, { csrf }),
  issueReset: (id: string, csrf: string) =>
    request<ResetLink>('POST', `/api/users/${enc(id)}/password-resets`, { csrf }),
  invitations: () => request<{ invitations: Invitation[] }>('GET', '/api/invitations'),
  invite: (phoneOrEmail: string, role: string, csrf: string) =>
    request<CreatedInvitation>('POST', '/api/invitations', {
      body: phoneOrEmail.includes('@') ? { email: phoneOrEmail, role } : { phone: phoneOrEmail, role },
      csrf,
    }),
  renewInvitation: (id: string, csrf: string) =>
    request<CreatedInvitation>('POST', `/api/invitations/${enc(id)}/renew`, { csrf }),
  revokeInvitation: (id: string, csrf: string) =>
    request<Invitation>('POST', `/api/invitations/${enc(id)}/revoke`, { csrf }),

  // paramètres (S17)
  organization: () => request<OrganizationDetail>('GET', '/api/organization'),
  patchOrganization: (body: { name?: string; city?: string }, version: number, csrf: string) =>
    request<OrganizationDetail>('PATCH', '/api/organization', { body, version, csrf }),
  settings: () => request<Settings>('GET', '/api/settings'),
  patchSettings: (body: Record<string, unknown>, version: number, csrf: string) =>
    request<Settings>('PATCH', '/api/settings', { body, version, csrf }),
  roleSettings: () => request<{ settings: RoleSetting[] }>('GET', '/api/role-settings'),
  putRoleSetting: (s: { role: string; action: string; allowed: boolean }, version: number, csrf: string) =>
    request<RoleSetting>('PUT', '/api/role-settings', { body: s, version, csrf }),
  supportGrants: () => request<SupportOverview>('GET', '/api/support-grants'),
  grantSupport: (scope: string, reason: string, hours: number, csrf: string) =>
    request<SupportGrant>('POST', '/api/support-grants', {
      body: { scope, reason, duration_hours: hours },
      csrf,
    }),
  revokeSupport: (id: string, csrf: string) =>
    request<SupportGrant>('POST', `/api/support-grants/${enc(id)}/revoke`, { csrf }),

  // signalement (F39-observation)
  sendFeedback: (body: FeedbackPayload, csrf: string, idempotencyKey: string) =>
    request<FeedbackCreated>('POST', '/api/feedback', { body, csrf, idempotencyKey }),

  // journal (S21)
  audit: (filters: AuditFilters) => request<AuditPage>('GET', `/api/audit${query(filters)}`),

  // patrimoine (étape 05 : F04, F05, F06)
  properties: (filters: ListFilters = {}) =>
    request<PageOf<Property>>('GET', `/api/properties${query(filters)}`),
  property: (id: string) => request<Property>('GET', `/api/properties/${enc(id)}`),
  createProperty: (body: Record<string, unknown>, csrf: string) =>
    request<Property>('POST', '/api/properties', { body, csrf }),
  patchProperty: (id: string, body: Record<string, unknown>, version: number, csrf: string) =>
    request<Property>('PATCH', `/api/properties/${enc(id)}`, { body, version, csrf }),
  archiveProperty: (id: string, csrf: string) =>
    request<Property>('POST', `/api/properties/${enc(id)}/archive`, { csrf }),
  units: (filters: ListFilters = {}) => request<PageOf<Unit>>('GET', `/api/units${query(filters)}`),
  unit: (id: string) => request<Unit>('GET', `/api/units/${enc(id)}`),
  createUnit: (body: Record<string, unknown>, csrf: string) =>
    request<Unit>('POST', '/api/units', { body, csrf }),
  patchUnit: (id: string, body: Record<string, unknown>, version: number, csrf: string) =>
    request<Unit>('PATCH', `/api/units/${enc(id)}`, { body, version, csrf }),
  archiveUnit: (id: string, csrf: string) => request<Unit>('POST', `/api/units/${enc(id)}/archive`, { csrf }),
  tenants: (filters: ListFilters = {}) => request<PageOf<Tenant>>('GET', `/api/tenants${query(filters)}`),
  tenant: (id: string) => request<Tenant>('GET', `/api/tenants/${enc(id)}`),
  createTenant: (body: Record<string, unknown>, csrf: string) =>
    request<Tenant>('POST', '/api/tenants', { body, csrf }),
  patchTenant: (id: string, body: Record<string, unknown>, version: number, csrf: string) =>
    request<Tenant>('PATCH', `/api/tenants/${enc(id)}`, { body, version, csrf }),
  mergeTenant: (id: string, intoId: string, csrf: string) =>
    request<Tenant>('POST', `/api/tenants/${enc(id)}/merge`, { body: { into_id: intoId }, csrf }),
  archiveTenant: (id: string, csrf: string) =>
    request<Tenant>('POST', `/api/tenants/${enc(id)}/archive`, { csrf }),
  owners: (filters: ListFilters = {}) => request<PageOf<Owner>>('GET', `/api/owners${query(filters)}`),
  createOwner: (body: Record<string, unknown>, csrf: string) =>
    request<Owner>('POST', '/api/owners', { body, csrf }),
  search: (q: string) => request<SearchResult>('GET', `/api/search?q=${enc(q)}`),
  // baux et échéancier (étape 06 : F07, F08)
  // étape 07 (F09, F10, DP-96) : l'affectation, le statut et le montant en lettres viennent du serveur
  paymentContext: (leaseId: string) =>
    request<PaymentContext>('GET', `/api/leases/${enc(leaseId)}/payment-context`),
  previewPayment: (body: Record<string, unknown>, csrf: string) =>
    request<PaymentPreview>('POST', '/api/payments/preview', { body, csrf }),
  recordPayment: (body: Record<string, unknown>, idempotencyKey: string, csrf: string) =>
    request<RecordedPayment>('POST', '/api/payments', { body, csrf, idempotencyKey }),
  payments: (filters: { lease_id?: string; q?: string } = {}) =>
    request<{ items: PaymentRow[] }>('GET', `/api/payments${query(filters)}`),
  payment: (id: string) =>
    request<PaymentRow & { allocations: AllocationRow[]; receipt: ReceiptBody | null }>(
      'GET',
      `/api/payments/${enc(id)}`,
    ),
  receipt: (id: string) => request<ReceiptBody>('GET', `/api/receipts/${enc(id)}`),
  leases: (filters: { unit_id?: string; tenant_id?: string } = {}) =>
    request<{ items: Schemas['LeaseBody'][] }>('GET', `/api/leases${query(filters)}`),
  addLine: (id: string, body: Record<string, unknown>, csrf: string) =>
    request<Schemas['LineBody']>('POST', `/api/leases/${enc(id)}/installment-lines`, { body, csrf }),
  voidLine: (id: string, reason: string, csrf: string) =>
    request<Schemas['LineBody']>('POST', `/api/installment-lines/${enc(id)}/void`, {
      body: { reason },
      csrf,
    }),
  setWorks: (id: string, underWorks: boolean, csrf: string) =>
    request<Unit>('POST', `/api/units/${enc(id)}/works`, { body: { under_works: underWorks }, csrf }),
  lease: (id: string) => request<Schemas['LeaseBody']>('GET', `/api/leases/${enc(id)}`),
  createLease: (body: Record<string, unknown>, csrf: string) =>
    request<Schemas['LeaseBody']>('POST', '/api/leases', { body, csrf }),
  activateLease: (id: string, csrf: string) =>
    request<Schemas['LeaseBody']>('POST', `/api/leases/${enc(id)}/activate`, {
      csrf,
      idempotencyKey: crypto.randomUUID(),
    }),
  amendLease: (id: string, body: Record<string, unknown>, csrf: string) =>
    request<Schemas['LeaseBody']>('POST', `/api/leases/${enc(id)}/amendments`, { body, csrf }),
  terminateLease: (id: string, body: Record<string, unknown>, csrf: string) =>
    request<Schemas['LeaseBody']>('POST', `/api/leases/${enc(id)}/terminate`, { body, csrf }),
  installments: (id: string) =>
    request<{ items: Schemas['InstallmentBody'][] }>('GET', `/api/leases/${enc(id)}/installments`),
  setUnitAssignments: (membershipId: string, unitIds: string[], csrf: string) =>
    request<{ unit_ids: string[] }>('PUT', `/api/users/${enc(membershipId)}/unit-assignments`, {
      body: { unit_ids: unitIds },
      csrf,
    }),
};

/** Filtres d'une liste du patrimoine (API_CONTRACTS.md §2.5). */
export interface ListFilters {
  q?: string;
  archived?: string;
  district?: string;
  owner_id?: string;
  property_id?: string;
  usage?: string;
  cursor?: string;
  limit?: string;
}
