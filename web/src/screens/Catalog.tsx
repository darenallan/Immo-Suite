// Étape 05 : biens et lots (S06, S07), locataires (S08, S09), choix du mandant, recherche unique (F04, F05, F06, DP-92).
// Aucune règle n'est décidée ici : le serveur valide, refuse et masque ; l'écran affiche ses réponses et ses codes.
import { useEffect, useRef, useState, type ReactNode, type SyntheticEvent } from 'react';
import {
  ApiProblem,
  api,
  type Owner,
  type Property,
  type SearchResult,
  type Tenant,
  type Unit,
} from '../api/client';
import { useAuth } from '../auth';
import { messageFor } from '../messages';
import { trackSearch } from '../observation/track';
import { fr } from '../texts/fr';

const t = fr.catalog;
const KINDS = Object.keys(t.kinds);
const USAGES = Object.keys(t.usages);

type Navigate = (path: string) => void;

/** Erreur affichée et erreurs par champ (codes du serveur, texte d'ici). */
interface Failure {
  text: string;
  fields: Record<string, string>;
  problem: ApiProblem | undefined;
}

function failure(e: unknown): Failure {
  const problem = e instanceof ApiProblem ? e : new ApiProblem(0, 'server.unexpected', '');
  const fields: Record<string, string> = {};
  for (const f of problem.fields) fields[f.field] = fr.fieldErrors[f.code] ?? fr.errors['validation.failed'];
  return { text: messageFor(problem), fields, problem };
}

/** Champ texte avec étiquette liée et erreur liée (UI_UX_DESIGN.md, Accessibilité). */
function Field({
  id,
  label,
  value,
  onChange,
  error,
  hint,
  type = 'text',
  required = false,
}: {
  id: string;
  label: string;
  value: string;
  onChange: (v: string) => void;
  error?: string | undefined;
  hint?: string;
  type?: string;
  required?: boolean;
}) {
  const describedBy = [hint !== undefined ? `${id}-hint` : '', error !== undefined ? `${id}-error` : '']
    .filter((x) => x !== '')
    .join(' ');
  return (
    <div className="field">
      <label htmlFor={id}>{label}</label>
      <input
        id={id}
        type={type}
        value={value}
        required={required}
        aria-invalid={error !== undefined}
        aria-describedby={describedBy === '' ? undefined : describedBy}
        onChange={(e) => {
          onChange(e.target.value);
        }}
      />
      {hint !== undefined && (
        <p id={`${id}-hint`} className="muted">
          {hint}
        </p>
      )}
      {error !== undefined && (
        <p id={`${id}-error`} className="field__error">
          {error}
        </p>
      )}
    </div>
  );
}

function Select({
  id,
  label,
  value,
  onChange,
  options,
  error,
}: {
  id: string;
  label: string;
  value: string;
  onChange: (v: string) => void;
  options: readonly (readonly [string, string])[];
  error?: string | undefined;
}) {
  return (
    <div className="field">
      <label htmlFor={id}>{label}</label>
      <select
        id={id}
        value={value}
        aria-invalid={error !== undefined}
        onChange={(e) => {
          onChange(e.target.value);
        }}
      >
        {options.map(([v, l]) => (
          <option key={v} value={v}>
            {l}
          </option>
        ))}
      </select>
      {error !== undefined && <p className="field__error">{error}</p>}
    </div>
  );
}

function Notice({ failure: f, children }: { failure?: Failure | undefined; children?: ReactNode }) {
  if (f === undefined && children === undefined) return null;
  return (
    <div className="notice notice--error" role="alert">
      {f?.text}
      {children}
    </div>
  );
}

/** Confirmation d'une action à enjeu (archiver, fusionner) : focus initial sur [Annuler], Échap ferme. */
function Confirm({
  text,
  verb,
  onConfirm,
  onCancel,
}: {
  text: string;
  verb: string;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  const cancel = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    cancel.current?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onCancel();
    };
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('keydown', onKey);
    };
  }, [onCancel]);
  return (
    <div className="card" role="alertdialog" aria-modal="true" aria-labelledby="confirm-text">
      <p id="confirm-text">{text}</p>
      <div className="actions">
        <button ref={cancel} className="button button--secondary" type="button" onClick={onCancel}>
          {t.cancel}
        </button>
        <button className="button" type="button" onClick={onConfirm}>
          {verb}
        </button>
      </div>
    </div>
  );
}

function statusLabel(s: string): string {
  return t.statuses[s] ?? s;
}

function formatFcfa(n: number): string {
  return `${n.toLocaleString('fr-FR').replace(/\s/g, ' ')} FCFA`;
}

function formatPhone(p: string | null | undefined): string {
  if (p === null || p === undefined) return '';
  const local = p.replace('+226', '');
  return `+226 ${local.replace(/(\d{2})(?=\d)/g, '$1 ')}`.trim();
}

function useLoad<T>(
  load: () => Promise<T>,
  deps: readonly unknown[],
): [T | undefined, Failure | undefined, () => void] {
  const [data, setData] = useState<T | undefined>(undefined);
  const [error, setError] = useState<Failure | undefined>(undefined);
  const [tick, setTick] = useState(0);
  useEffect(() => {
    let live = true;
    load().then(
      (d) => {
        if (live) {
          setData(d);
          setError(undefined);
        }
      },
      (e: unknown) => {
        if (live) setError(failure(e));
      },
    );
    return () => {
      live = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tick, ...deps]);
  return [
    data,
    error,
    () => {
      setTick((x) => x + 1);
    },
  ];
}

// ───────── choix du mandant (F05) ─────────

function OwnerSelect({
  value,
  onChange,
  canCreate,
}: {
  value: string;
  onChange: (id: string) => void;
  canCreate: boolean;
}) {
  const { csrf } = useAuth();
  const [owners, , reload] = useLoad(() => api.owners(), []);
  const [creating, setCreating] = useState(false);
  const [name, setName] = useState('');
  const [error, setError] = useState<Failure | undefined>(undefined);
  const create = async () => {
    setError(undefined);
    try {
      const o = await api.createOwner({ kind: 'person', display_name: name }, csrf());
      setCreating(false);
      setName('');
      reload();
      onChange(o.id);
    } catch (e) {
      setError(failure(e));
    }
  };
  const options: [string, string][] = [
    ['', t.noOwner],
    ...(owners?.items ?? []).map((o: Owner) => [o.id, o.display_name] as [string, string]),
  ];
  return (
    <div>
      <Select id="property-owner" label={t.owner} value={value} onChange={onChange} options={options} />
      {canCreate && !creating && (
        <button
          className="button button--secondary button--small"
          type="button"
          onClick={() => {
            setCreating(true);
          }}
        >
          {t.newOwner}
        </button>
      )}
      {creating && (
        <div className="stack">
          <Field
            id="owner-name"
            label={t.ownerName}
            value={name}
            onChange={setName}
            error={error?.fields['display_name']}
          />
          <Notice failure={error?.fields['display_name'] === undefined ? error : undefined} />
          <div className="actions">
            <button className="button" type="button" onClick={() => void create()}>
              {t.create}
            </button>
            <button
              className="button button--secondary"
              type="button"
              onClick={() => {
                setCreating(false);
              }}
            >
              {t.cancel}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

// ───────── champs d'un lot ─────────

interface UnitDraft {
  reference: string;
  kind: string;
  usage: string;
  surface: string;
  rent: string;
  activity: string;
}

const emptyUnit = (kind = 'apartment'): UnitDraft => ({
  reference: '',
  kind,
  usage: 'residential',
  surface: '',
  rent: '',
  activity: '',
});

/** Corps d'un lot : seuls les champs remplis partent ; le loyer est un entier (sinon le serveur répond « nombre entier »). */
function unitBody(d: UnitDraft): Record<string, unknown> {
  const body: Record<string, unknown> = { kind: d.kind, usage: d.usage };
  if (d.reference.trim() !== '') body['reference'] = d.reference;
  if (d.surface.trim() !== '') body['surface_m2'] = d.surface;
  if (d.rent.trim() !== '') {
    const digits = d.rent.replace(/\s/g, '');
    body['reference_rent'] = /^\d+$/.test(digits) ? Number(digits) : digits;
  }
  if (d.usage !== 'residential' && d.activity.trim() !== '') body['permitted_activity'] = d.activity;
  return body;
}

function UnitFields({
  prefix,
  draft,
  set,
  errors,
}: {
  prefix: string;
  draft: UnitDraft;
  set: (d: UnitDraft) => void;
  errors: Record<string, string>;
}) {
  return (
    <>
      <Field
        id={`${prefix}-reference`}
        label={t.reference}
        value={draft.reference}
        onChange={(v) => {
          set({ ...draft, reference: v });
        }}
        error={errors['reference']}
      />
      <Select
        id={`${prefix}-kind`}
        label={t.kind}
        value={draft.kind}
        onChange={(v) => {
          set({ ...draft, kind: v });
        }}
        options={KINDS.map((k) => [k, t.kinds[k] ?? k] as const)}
        error={errors['kind']}
      />
      <Select
        id={`${prefix}-usage`}
        label={t.usage}
        value={draft.usage}
        onChange={(v) => {
          set({ ...draft, usage: v });
        }}
        options={USAGES.map((u) => [u, t.usages[u] ?? u] as const)}
        error={errors['usage']}
      />
      <Field
        id={`${prefix}-surface`}
        label={t.surface}
        value={draft.surface}
        onChange={(v) => {
          set({ ...draft, surface: v });
        }}
        error={errors['surface_m2']}
      />
      <Field
        id={`${prefix}-rent`}
        label={t.referenceRent}
        value={draft.rent}
        onChange={(v) => {
          set({ ...draft, rent: v });
        }}
        error={errors['reference_rent']}
      />
      {/* R-012, DP-41 : l'activité autorisée n'existe que pour un lot commercial ou mixte */}
      {draft.usage !== 'residential' && (
        <Field
          id={`${prefix}-activity`}
          label={t.permittedActivity}
          value={draft.activity}
          onChange={(v) => {
            set({ ...draft, activity: v });
          }}
          error={errors['permitted_activity']}
        />
      )}
    </>
  );
}

// ───────── S06 : biens et lots ─────────

export function PropertiesScreen({
  canWrite,
  canCreateOwner,
  ownerPlan,
  navigate,
}: {
  canWrite: boolean;
  canCreateOwner: boolean;
  ownerPlan: boolean;
  navigate: Navigate;
}) {
  const { csrf } = useAuth();
  const [archived, setArchived] = useState(false);
  const [page, error] = useLoad(() => api.properties(archived ? { archived: 'true' } : {}), [archived]);
  const [form, setForm] = useState(false);
  const [name, setName] = useState('');
  const [district, setDistrict] = useState('');
  const [city, setCity] = useState('');
  const [owner, setOwner] = useState('');
  const [single, setSingle] = useState(false);
  const [unit, setUnit] = useState<UnitDraft>(emptyUnit('house'));
  const [failed, setFailed] = useState<Failure | undefined>(undefined);

  const submit = async (event: SyntheticEvent) => {
    event.preventDefault();
    setFailed(undefined);
    const body: Record<string, unknown> = { name };
    if (district.trim() !== '') body['district'] = district;
    if (city.trim() !== '') body['city'] = city;
    if (owner !== '') body['default_owner_id'] = owner;
    if (single) body['single_unit'] = unitBody(unit);
    try {
      const p = await api.createProperty(body, csrf());
      navigate(`/biens/${p.id}`);
    } catch (e) {
      setFailed(failure(e));
    }
  };

  const items = page?.items ?? [];
  return (
    <section aria-labelledby="properties-title" className="stack">
      <h1 id="properties-title">{t.properties}</h1>
      <Notice failure={error} />
      <div className="actions">
        {canWrite && !form && (
          <button
            className="button"
            type="button"
            onClick={() => {
              setForm(true);
            }}
          >
            {t.newProperty}
          </button>
        )}
        <label className="field field--inline">
          <input
            type="checkbox"
            checked={archived}
            onChange={(e) => {
              setArchived(e.target.checked);
            }}
          />{' '}
          {t.showArchived}
        </label>
      </div>
      {form && (
        <form
          className="card form"
          onSubmit={(e) => void submit(e)}
          aria-labelledby="new-property-title"
          noValidate
        >
          <h2 id="new-property-title">{t.newProperty}</h2>
          <Notice
            failure={failed !== undefined && Object.keys(failed.fields).length === 0 ? failed : undefined}
          />
          <Field
            id="property-name"
            label={t.name}
            value={name}
            onChange={setName}
            error={failed?.fields['name']}
            required
          />
          <Field
            id="property-district"
            label={t.district}
            value={district}
            onChange={setDistrict}
            error={failed?.fields['district']}
          />
          <Field
            id="property-city"
            label={t.city}
            value={city}
            onChange={setCity}
            error={failed?.fields['city']}
          />
          {!ownerPlan && <OwnerSelect value={owner} onChange={setOwner} canCreate={canCreateOwner} />}
          {failed?.fields['default_owner_id'] !== undefined && (
            <p className="field__error">{failed.fields['default_owner_id']}</p>
          )}
          <label className="field field--inline">
            <input
              type="checkbox"
              checked={single}
              onChange={(e) => {
                setSingle(e.target.checked);
              }}
            />{' '}
            {t.singleUnit}
          </label>
          {single && <UnitFields prefix="single" draft={unit} set={setUnit} errors={failed?.fields ?? {}} />}
          <div className="actions">
            <button className="button" type="submit">
              {t.create}
            </button>
            <button
              className="button button--secondary"
              type="button"
              onClick={() => {
                setForm(false);
              }}
            >
              {t.cancel}
            </button>
          </div>
        </form>
      )}
      {page !== undefined && items.length === 0 && <p className="card">{t.emptyProperties}</p>}
      <ul className="rows" aria-live="polite">
        {items.map((p: Property) => (
          <li key={p.id} className="row">
            <a
              href={`/biens/${p.id}`}
              className="row__title"
              onClick={(e) => {
                e.preventDefault();
                navigate(`/biens/${p.id}`);
              }}
            >
              {p.name}
            </a>
            <span className="muted">
              {[p.district, p.default_owner?.display_name]
                .filter((x) => x !== null && x !== undefined && x !== '')
                .join(' · ')}
            </span>
            <span>{t.unitsCount(p.unit_count, p.vacant_count)}</span>
            {p.archived_at !== null && p.archived_at !== undefined && (
              <span className="tag">{t.archived}</span>
            )}
          </li>
        ))}
      </ul>
    </section>
  );
}

/** Fiche d'un bien : ses lots, ajout d'un lot, archivage. */
export function PropertyScreen({
  id,
  canWrite,
  navigate,
}: {
  id: string;
  canWrite: boolean;
  navigate: Navigate;
}) {
  const { csrf } = useAuth();
  const [p, error, reload] = useLoad(() => api.property(id), [id]);
  const [adding, setAdding] = useState(false);
  const [draft, setDraft] = useState<UnitDraft>(emptyUnit());
  const [failed, setFailed] = useState<Failure | undefined>(undefined);
  const [confirming, setConfirming] = useState(false);

  const addUnit = async (event: SyntheticEvent) => {
    event.preventDefault();
    setFailed(undefined);
    try {
      await api.createUnit({ ...unitBody(draft), property_id: id }, csrf());
      setAdding(false);
      setDraft(emptyUnit());
      reload();
    } catch (e) {
      setFailed(failure(e));
    }
  };
  const archive = async () => {
    setConfirming(false);
    try {
      await api.archiveProperty(id, csrf());
      reload();
    } catch (e) {
      setFailed(failure(e));
    }
  };

  if (error !== undefined) return <Notice failure={error} />;
  if (p === undefined)
    return (
      <p role="status" className="muted">
        {fr.common.loading}
      </p>
    );
  const archived = p.archived_at !== null && p.archived_at !== undefined;
  return (
    <section aria-labelledby="property-title" className="stack">
      <button
        className="button button--secondary button--small"
        type="button"
        onClick={() => {
          navigate('/biens');
        }}
      >
        {t.back}
      </button>
      <h1 id="property-title">
        {p.name} {archived && <span className="tag">{t.archived}</span>}
      </h1>
      <p className="muted">
        {[p.address, p.district, p.city].filter((x) => x !== null && x !== undefined && x !== '').join(' · ')}
        {p.default_owner !== null &&
          p.default_owner !== undefined &&
          ` · ${t.owner} : ${p.default_owner.display_name}`}
      </p>
      <Notice
        failure={failed !== undefined && Object.keys(failed.fields).length === 0 ? failed : undefined}
      />
      <h2>{t.units}</h2>
      {p.units.length === 0 && <p className="muted">{t.noUnits}</p>}
      <ul className="rows">
        {p.units.map((u: Unit) => (
          <li key={u.id} className="row">
            <a
              href={`/lots/${u.id}`}
              className="row__title"
              onClick={(e) => {
                e.preventDefault();
                navigate(`/lots/${u.id}`);
              }}
            >
              {u.reference}
            </a>
            <span className="tag">{t.usages[u.usage] ?? u.usage}</span>
            <span>{statusLabel(u.status)}</span>
            {u.reference_rent !== null && u.reference_rent !== undefined && (
              <span>{formatFcfa(u.reference_rent)}</span>
            )}
          </li>
        ))}
      </ul>
      {canWrite && !archived && !adding && (
        <button
          className="button"
          type="button"
          onClick={() => {
            setAdding(true);
          }}
        >
          {t.addUnit}
        </button>
      )}
      {adding && (
        <form
          className="card form"
          onSubmit={(e) => void addUnit(e)}
          aria-labelledby="add-unit-title"
          noValidate
        >
          <h2 id="add-unit-title">{t.addUnit}</h2>
          <UnitFields prefix="unit" draft={draft} set={setDraft} errors={failed?.fields ?? {}} />
          <div className="actions">
            <button className="button" type="submit">
              {t.create}
            </button>
            <button
              className="button button--secondary"
              type="button"
              onClick={() => {
                setAdding(false);
              }}
            >
              {t.cancel}
            </button>
          </div>
        </form>
      )}
      {canWrite && !archived && !confirming && (
        <button
          className="button button--secondary"
          type="button"
          onClick={() => {
            setConfirming(true);
          }}
        >
          {t.archive}
        </button>
      )}
      {confirming && (
        <Confirm
          text={t.archiveConfirm(p.name)}
          verb={t.archive}
          onConfirm={() => void archive()}
          onCancel={() => {
            setConfirming(false);
          }}
        />
      )}
    </section>
  );
}

// ───────── S07 : fiche lot ─────────

function Tabs({
  tabs,
  active,
  onChange,
}: {
  tabs: readonly (readonly [string, string])[];
  active: string;
  onChange: (id: string) => void;
}) {
  return (
    <div role="tablist" className="actions">
      {tabs.map(([id, label]) => (
        <button
          key={id}
          role="tab"
          type="button"
          aria-selected={active === id}
          className={active === id ? 'button' : 'button button--secondary'}
          onClick={() => {
            onChange(id);
          }}
        >
          {label}
        </button>
      ))}
    </div>
  );
}

/** Onglet Baux d'une fiche (S07, S09) : les baux du lot ou du locataire, liés à leur fiche S10. */
function LeaseTab({
  unitId,
  tenantId,
  navigate,
}: {
  unitId?: string;
  tenantId?: string;
  navigate: Navigate;
}) {
  const [leases] = useLoad(
    () =>
      api.leases(
        unitId !== undefined ? { unit_id: unitId } : tenantId !== undefined ? { tenant_id: tenantId } : {},
      ),
    [unitId, tenantId],
  );
  if (leases === undefined) return <p className="muted">{fr.common.loading}</p>;
  if (leases.items.length === 0) return <p className="muted">{t.noLease}</p>;
  return (
    <ul className="rows">
      {leases.items.map((l) => (
        <li key={l.id} className="row">
          <a
            className="row__title"
            href={`/baux/${l.id}`}
            onClick={(e) => {
              e.preventDefault();
              navigate(`/baux/${l.id}`);
            }}
          >
            {l.unit_reference} · {l.property_name} · {l.tenants[0]?.display_name ?? ''}
          </a>
          <span>{formatFcfa(l.current_rent)}</span>
        </li>
      ))}
    </ul>
  );
}

export function UnitScreen({
  id,
  canWrite,
  navigate,
}: {
  id: string;
  canWrite: boolean;
  navigate: Navigate;
}) {
  const { csrf } = useAuth();
  const [u, error, reload] = useLoad(() => api.unit(id), [id]);
  const [tab, setTab] = useState('overview');
  const [confirming, setConfirming] = useState(false);
  const [failed, setFailed] = useState<Failure | undefined>(undefined);
  if (error !== undefined) return <Notice failure={error} />;
  if (u === undefined)
    return (
      <p role="status" className="muted">
        {fr.common.loading}
      </p>
    );
  const archive = async () => {
    setConfirming(false);
    try {
      await api.archiveUnit(id, csrf());
      reload();
    } catch (e) {
      setFailed(failure(e));
    }
  };
  return (
    <section aria-labelledby="unit-title" className="stack">
      <button
        className="button button--secondary button--small"
        type="button"
        onClick={() => {
          navigate(`/biens/${u.property_id}`);
        }}
      >
        {t.back}
      </button>
      <h1 id="unit-title">
        {u.reference} · {u.property_name}
      </h1>
      <p>
        <span className="tag">{t.usages[u.usage] ?? u.usage}</span> {t.kinds[u.kind] ?? u.kind} ·{' '}
        {statusLabel(u.status)}
      </p>
      <Notice failure={failed} />
      <Tabs
        tabs={[
          ['overview', t.tabs.overview],
          ['leases', t.tabs.leases],
          ['payments', t.tabs.payments],
          ['works', t.tabs.works],
          ['documents', t.tabs.documents],
        ]}
        active={tab}
        onChange={setTab}
      />
      <div role="tabpanel" className="card">
        {tab === 'overview' ? (
          <dl className="facts">
            <dt>{t.referenceRent}</dt>
            <dd>
              {u.reference_rent === null || u.reference_rent === undefined
                ? '—'
                : formatFcfa(u.reference_rent)}
            </dd>
            <dt>{t.surface}</dt>
            <dd>{u.surface_m2 ?? '—'}</dd>
            {u.permitted_activity !== null && u.permitted_activity !== undefined && (
              <>
                <dt>{t.permittedActivity}</dt>
                <dd>{u.permitted_activity}</dd>
              </>
            )}
            <dt>{t.tabs.leases}</dt>
            <dd>{t.noLease}</dd>
          </dl>
        ) : tab === 'leases' ? (
          <LeaseTab unitId={id} navigate={navigate} />
        ) : (
          <p className="muted">{t.emptyTab}</p>
        )}
      </div>
      {canWrite && (u.status === 'vacant' || u.status === 'under_works') && (
        <button
          className="button button--secondary"
          type="button"
          onClick={() => {
            void api.setWorks(id, u.status === 'vacant', csrf()).then(reload, (e: unknown) => {
              setFailed(failure(e));
            });
          }}
        >
          {u.status === 'vacant' ? 'Mettre en travaux' : 'Retirer « en travaux »'}
        </button>
      )}
      {canWrite && u.status !== 'archived' && !confirming && (
        <button
          className="button button--secondary"
          type="button"
          onClick={() => {
            setConfirming(true);
          }}
        >
          {t.archive}
        </button>
      )}
      {confirming && (
        <Confirm
          text={t.archiveConfirm(u.reference)}
          verb={t.archive}
          onConfirm={() => void archive()}
          onCancel={() => {
            setConfirming(false);
          }}
        />
      )}
    </section>
  );
}

// ───────── S08 : locataires ─────────

interface TenantDraft {
  kind: string;
  last: string;
  first: string;
  trade: string;
  phone: string;
  whatsapp: boolean;
  email: string;
  backupName: string;
  backupPhone: string;
  notes: string;
}

const emptyTenant: TenantDraft = {
  kind: 'person',
  last: '',
  first: '',
  trade: '',
  phone: '',
  whatsapp: false,
  email: '',
  backupName: '',
  backupPhone: '',
  notes: '',
};

function tenantBody(d: TenantDraft, withPrivate: boolean): Record<string, unknown> {
  const body: Record<string, unknown> = { kind: d.kind, last_name: d.last };
  if (d.kind === 'person' && d.first.trim() !== '') body['first_name'] = d.first;
  if (d.trade.trim() !== '') body['trade_activity'] = d.trade;
  if (d.phone.trim() !== '') {
    body['phone'] = d.phone;
    body['phone_is_whatsapp'] = d.whatsapp;
  }
  if (d.email.trim() !== '') body['email'] = d.email;
  if (withPrivate) {
    if (d.backupName.trim() !== '' || d.backupPhone.trim() !== '')
      body['backup_contact'] = {
        name: d.backupName.trim() === '' ? null : d.backupName,
        phone: d.backupPhone.trim() === '' ? null : d.backupPhone,
      };
    if (d.notes.trim() !== '') body['notes'] = d.notes;
  }
  return body;
}

export function TenantsScreen({ canWrite, navigate }: { canWrite: boolean; navigate: Navigate }) {
  const { csrf } = useAuth();
  const [archived, setArchived] = useState(false);
  const [page, error] = useLoad(() => api.tenants(archived ? { archived: 'true' } : {}), [archived]);
  const [form, setForm] = useState(false);
  const [d, setD] = useState<TenantDraft>(emptyTenant);
  const [failed, setFailed] = useState<Failure | undefined>(undefined);
  // les rôles qui écrivent voient aussi le contact de secours et les notes (API_CONTRACTS.md §4.5)
  const withPrivate = canWrite;

  const submit = async (event: SyntheticEvent) => {
    event.preventDefault();
    setFailed(undefined);
    try {
      const created = await api.createTenant(tenantBody(d, withPrivate), csrf());
      navigate(`/locataires/${created.id}`);
    } catch (e) {
      setFailed(failure(e));
    }
  };

  const duplicate =
    failed?.problem?.code === 'tenant.phone_duplicate'
      ? (failed.problem.params['conflict'] as { id?: string; display_name?: string } | undefined)
      : undefined;
  const items = page?.items ?? [];
  return (
    <section aria-labelledby="tenants-title" className="stack">
      <h1 id="tenants-title">{t.tenants}</h1>
      <Notice failure={error} />
      <div className="actions">
        {canWrite && !form && (
          <button
            className="button"
            type="button"
            onClick={() => {
              setForm(true);
            }}
          >
            {t.newTenant}
          </button>
        )}
        <label className="field field--inline">
          <input
            type="checkbox"
            checked={archived}
            onChange={(e) => {
              setArchived(e.target.checked);
            }}
          />{' '}
          {t.showArchived}
        </label>
      </div>
      {form && (
        <form
          className="card form"
          onSubmit={(e) => void submit(e)}
          aria-labelledby="new-tenant-title"
          noValidate
        >
          <h2 id="new-tenant-title">{t.newTenant}</h2>
          {duplicate !== undefined ? (
            <div className="notice notice--error" role="alert">
              {t.duplicate(duplicate.display_name ?? '')}{' '}
              <button
                className="button button--small"
                type="button"
                onClick={() => {
                  navigate(`/locataires/${duplicate.id ?? ''}`);
                }}
              >
                {t.openExisting}
              </button>
            </div>
          ) : (
            <Notice
              failure={failed !== undefined && Object.keys(failed.fields).length === 0 ? failed : undefined}
            />
          )}
          <Select
            id="tenant-kind"
            label={t.kind}
            value={d.kind}
            onChange={(v) => {
              setD({ ...d, kind: v });
            }}
            options={[
              ['person', t.person],
              ['company', t.company],
            ]}
          />
          <Field
            id="tenant-last"
            label={d.kind === 'company' ? t.companyName : t.lastName}
            value={d.last}
            onChange={(v) => {
              setD({ ...d, last: v });
            }}
            error={failed?.fields['last_name']}
            required
          />
          {d.kind === 'person' && (
            <Field
              id="tenant-first"
              label={t.firstName}
              value={d.first}
              onChange={(v) => {
                setD({ ...d, first: v });
              }}
              error={failed?.fields['first_name']}
            />
          )}
          {d.kind === 'company' && (
            <Field
              id="tenant-trade"
              label={t.tradeActivity}
              value={d.trade}
              onChange={(v) => {
                setD({ ...d, trade: v });
              }}
              error={failed?.fields['trade_activity']}
            />
          )}
          <Field
            id="tenant-phone"
            label={t.phone}
            hint={t.phoneHint}
            type="tel"
            value={d.phone}
            onChange={(v) => {
              setD({ ...d, phone: v });
            }}
            error={failed?.fields['phone']}
          />
          {d.phone.trim() !== '' && (
            <label className="field field--inline">
              <input
                type="checkbox"
                checked={d.whatsapp}
                onChange={(e) => {
                  setD({ ...d, whatsapp: e.target.checked });
                }}
              />{' '}
              {t.whatsapp}
            </label>
          )}
          <Field
            id="tenant-email"
            label={t.email}
            type="email"
            value={d.email}
            onChange={(v) => {
              setD({ ...d, email: v });
            }}
            error={failed?.fields['email']}
          />
          {withPrivate && (
            <fieldset className="stack">
              <legend>{t.backupContact}</legend>
              <Field
                id="tenant-backup-name"
                label={t.backupName}
                value={d.backupName}
                onChange={(v) => {
                  setD({ ...d, backupName: v });
                }}
                error={failed?.fields['backup_contact.name']}
              />
              <Field
                id="tenant-backup-phone"
                label={t.backupPhone}
                type="tel"
                value={d.backupPhone}
                onChange={(v) => {
                  setD({ ...d, backupPhone: v });
                }}
                error={failed?.fields['backup_contact.phone']}
              />
            </fieldset>
          )}
          {withPrivate && (
            <Field
              id="tenant-notes"
              label={t.notes}
              value={d.notes}
              onChange={(v) => {
                setD({ ...d, notes: v });
              }}
              error={failed?.fields['notes']}
            />
          )}
          <p className="muted">{t.idDocumentDisabled}</p>
          <div className="actions">
            <button className="button" type="submit">
              {t.create}
            </button>
            <button
              className="button button--secondary"
              type="button"
              onClick={() => {
                setForm(false);
              }}
            >
              {t.cancel}
            </button>
          </div>
        </form>
      )}
      {page !== undefined && items.length === 0 && <p className="card">{t.emptyTenants}</p>}
      <ul className="rows" aria-live="polite">
        {items.map((x: Tenant) => (
          <li key={x.id} className="row">
            <a
              href={`/locataires/${x.id}`}
              className="row__title"
              onClick={(e) => {
                e.preventDefault();
                navigate(`/locataires/${x.id}`);
              }}
            >
              {x.display_name}
            </a>
            <span>{formatPhone(x.phone)}</span>
            {x.archived_at !== null && x.archived_at !== undefined && (
              <span className="tag">{t.archived}</span>
            )}
          </li>
        ))}
      </ul>
    </section>
  );
}

// ───────── S09 : fiche locataire ─────────

export function TenantScreen({
  id,
  canWrite,
  navigate,
}: {
  id: string;
  canWrite: boolean;
  navigate: Navigate;
}) {
  const { csrf } = useAuth();
  const [x, error, reload] = useLoad(() => api.tenant(id), [id]);
  const [tab, setTab] = useState('overview');
  const [failed, setFailed] = useState<Failure | undefined>(undefined);
  const [merging, setMerging] = useState(false);
  const [term, setTerm] = useState('');
  const [candidates, setCandidates] = useState<Tenant[]>([]);
  const [target, setTarget] = useState<Tenant | undefined>(undefined);
  const [confirmArchive, setConfirmArchive] = useState(false);

  useEffect(() => {
    // saisie trop courte : rien n'est demandé, la liste affichée est vidée par dérivation (filtre au rendu)
    if (!merging || term.trim().length < 2) return;
    let live = true;
    const timer = window.setTimeout(() => {
      api.tenants({ q: term }).then(
        (p) => {
          if (live) setCandidates(p.items.filter((c) => c.id !== id));
        },
        () => {
          if (live) setCandidates([]);
        },
      );
    }, 200);
    return () => {
      live = false;
      window.clearTimeout(timer);
    };
  }, [merging, term, id]);

  if (error !== undefined) return <Notice failure={error} />;
  if (x === undefined)
    return (
      <p role="status" className="muted">
        {fr.common.loading}
      </p>
    );
  const archived = x.archived_at !== null && x.archived_at !== undefined;

  const merge = async () => {
    if (target === undefined) return;
    try {
      const kept = await api.mergeTenant(id, target.id, csrf());
      setMerging(false);
      setTarget(undefined);
      navigate(`/locataires/${kept.id}`);
    } catch (e) {
      setTarget(undefined);
      setFailed(failure(e));
    }
  };
  const archive = async () => {
    setConfirmArchive(false);
    try {
      await api.archiveTenant(id, csrf());
      reload();
    } catch (e) {
      setFailed(failure(e));
    }
  };

  const phone = x.phone ?? undefined;
  return (
    <section aria-labelledby="tenant-title" className="stack">
      <button
        className="button button--secondary button--small"
        type="button"
        onClick={() => {
          navigate('/locataires');
        }}
      >
        {t.back}
      </button>
      <h1 id="tenant-title">
        {x.display_name} {archived && <span className="tag">{t.archived}</span>}
      </h1>
      {x.merged_into_id !== null && x.merged_into_id !== undefined && (
        <div className="notice" role="status">
          {t.mergedInto}{' '}
          <button
            className="button button--small"
            type="button"
            onClick={() => {
              navigate(`/locataires/${x.merged_into_id ?? ''}`);
            }}
          >
            {t.openMerged}
          </button>
        </div>
      )}
      <Notice failure={failed} />
      <div className="card">
        <p>{formatPhone(x.phone)}</p>
        {x.email !== null && x.email !== undefined && <p>{x.email}</p>}
        <div className="actions">
          <button className="button" type="button" disabled title={t.soon}>
            {t.collect}
          </button>
          <button className="button button--secondary" type="button" disabled title={t.soon}>
            {t.remind}
          </button>
          {/* DP-92 : Appeler et WhatsApp actifs dès qu'un numéro existe */}
          {phone !== undefined && (
            <a className="button button--secondary" href={`tel:${phone}`}>
              {t.call}
            </a>
          )}
          {phone !== undefined && x.phone_is_whatsapp && (
            <a
              className="button button--secondary"
              href={`https://wa.me/${phone.replace('+', '')}`}
              target="_blank"
              rel="noreferrer noopener"
            >
              {t.whatsappAction}
            </a>
          )}
        </div>
      </div>
      <Tabs
        tabs={[
          ['overview', t.tabs.overview],
          ['leases', t.tabs.leases],
          ['payments', t.tabs.payments],
          ['reminders', t.tabs.reminders],
          ['documents', t.tabs.documents],
        ]}
        active={tab}
        onChange={setTab}
      />
      <div role="tabpanel" className="card">
        {tab === 'overview' ? (
          <dl className="facts">
            {x.trade_activity !== null && x.trade_activity !== undefined && (
              <>
                <dt>{t.tradeActivity}</dt>
                <dd>{x.trade_activity}</dd>
              </>
            )}
            {x.backup_contact !== undefined && (
              <>
                <dt>{t.backupContact}</dt>
                <dd>
                  {x.backup_contact === null
                    ? '—'
                    : [x.backup_contact.name, formatPhone(x.backup_contact.phone)]
                        .filter((v) => v !== null && v !== undefined && v !== '')
                        .join(' · ')}
                </dd>
              </>
            )}
            {x.notes !== undefined && (
              <>
                <dt>{t.notes}</dt>
                <dd>{x.notes ?? '—'}</dd>
              </>
            )}
            <dt>{t.tabs.leases}</dt>
            <dd>{t.noLease}</dd>
          </dl>
        ) : tab === 'leases' ? (
          <LeaseTab tenantId={id} navigate={navigate} />
        ) : (
          <p className="muted">{t.emptyTab}</p>
        )}
      </div>
      <p className="muted">{t.tenantSpace}</p>
      {canWrite && !archived && (
        <div className="actions">
          {!merging && (
            <button
              className="button button--secondary"
              type="button"
              onClick={() => {
                setMerging(true);
              }}
            >
              {t.merge}
            </button>
          )}
          {!confirmArchive && (
            <button
              className="button button--secondary"
              type="button"
              onClick={() => {
                setConfirmArchive(true);
              }}
            >
              {t.archive}
            </button>
          )}
        </div>
      )}
      {merging && (
        <div className="card stack">
          <Field id="merge-search" label={t.mergeSearch} value={term} onChange={setTerm} />
          <ul className="rows">
            {(term.trim().length >= 2 ? candidates : []).map((c) => (
              <li key={c.id} className="row">
                <button
                  className="button button--secondary"
                  type="button"
                  onClick={() => {
                    setTarget(c);
                  }}
                >
                  {c.display_name} {formatPhone(c.phone)}
                </button>
              </li>
            ))}
          </ul>
          <button
            className="button button--secondary"
            type="button"
            onClick={() => {
              setMerging(false);
            }}
          >
            {t.cancel}
          </button>
        </div>
      )}
      {target !== undefined && (
        <Confirm
          text={t.mergeConfirm(x.display_name, target.display_name)}
          verb={t.merge}
          onConfirm={() => void merge()}
          onCancel={() => {
            setTarget(undefined);
          }}
        />
      )}
      {confirmArchive && (
        <Confirm
          text={t.archiveConfirm(x.display_name)}
          verb={t.archive}
          onConfirm={() => void archive()}
          onCancel={() => {
            setConfirmArchive(false);
          }}
        />
      )}
    </section>
  );
}

// ───────── S18 : lots assignés à un agent terrain (R-081) ─────────

export function UnitAssignments({
  membershipId,
  name,
  assigned,
  onSaved,
}: {
  membershipId: string;
  name: string;
  assigned: readonly string[];
  onSaved: () => void;
}) {
  const { csrf } = useAuth();
  const [open, setOpen] = useState(false);
  const [units, setUnits] = useState<Unit[] | undefined>(undefined);
  const [chosen, setChosen] = useState<string[]>([...assigned]);
  const [failed, setFailed] = useState<Failure | undefined>(undefined);
  const toggle = () => {
    if (!open) {
      setChosen([...assigned]);
      api.units({ limit: '100' }).then(
        (p) => {
          setUnits(p.items);
        },
        (e: unknown) => {
          setFailed(failure(e));
        },
      );
    }
    setOpen(!open);
  };
  const save = async () => {
    setFailed(undefined);
    try {
      await api.setUnitAssignments(membershipId, chosen, csrf());
      setOpen(false);
      onSaved();
    } catch (e) {
      setFailed(failure(e));
    }
  };
  return (
    <div>
      <button
        className="button button--secondary button--small"
        type="button"
        aria-expanded={open}
        onClick={toggle}
      >
        {t.assigned(assigned.length)}
      </button>
      {open && (
        <fieldset className="card stack">
          <legend>{t.assignedTitle(name)}</legend>
          <Notice failure={failed} />
          {units !== undefined && units.length === 0 && <p className="muted">{t.assignedNone}</p>}
          {(units ?? []).map((u) => (
            <label key={u.id} className="field field--inline">
              <input
                type="checkbox"
                checked={chosen.includes(u.id)}
                onChange={(e) => {
                  setChosen(e.target.checked ? [...chosen, u.id] : chosen.filter((x) => x !== u.id));
                }}
              />{' '}
              {u.reference} · {u.property_name}
            </label>
          ))}
          <div className="actions">
            <button className="button" type="button" onClick={() => void save()}>
              {t.save}
            </button>
            <button
              className="button button--secondary"
              type="button"
              onClick={() => {
                setOpen(false);
              }}
            >
              {t.cancel}
            </button>
          </div>
        </fieldset>
      )}
    </div>
  );
}

// ───────── recherche unique (API_CONTRACTS.md §2.6) ─────────

export function SearchBox({ navigate, canCreateTenant }: { navigate: Navigate; canCreateTenant: boolean }) {
  const [term, setTerm] = useState('');
  const [result, setResult] = useState<SearchResult | undefined>(undefined);
  const input = useRef<HTMLInputElement>(null);

  // raccourci « / » (UI_UX_DESIGN.md, Recherche unique)
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      if (event.key === '/' && target?.tagName !== 'INPUT' && target?.tagName !== 'TEXTAREA') {
        event.preventDefault();
        input.current?.focus();
      }
    };
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('keydown', onKey);
    };
  }, []);

  useEffect(() => {
    const q = term.trim();
    // saisie trop courte : rien n'est demandé, le résultat affiché est masqué par dérivation (voir `shown`)
    if (q.length < 2) return;
    let live = true;
    const timer = window.setTimeout(() => {
      api.search(q).then(
        (r) => {
          if (!live) return;
          setResult(r);
          const empty = r.tenants.length === 0 && r.units.length === 0 && r.receipts.length === 0;
          trackSearch(q.length, empty ? 'none' : r.tenants.length > 0 ? 'tenants' : 'units', empty);
        },
        () => {
          if (live) setResult(undefined);
        },
      );
    }, 200);
    return () => {
      live = false;
      window.clearTimeout(timer);
    };
  }, [term]);

  const open = (path: string) => {
    setTerm('');
    setResult(undefined);
    navigate(path);
  };
  const shown = term.trim().length >= 2 ? result : undefined;
  const empty = shown !== undefined && shown.tenants.length === 0 && shown.units.length === 0;
  return (
    <div className="search" role="search">
      <label htmlFor="global-search" className="visually-hidden">
        {t.search}
      </label>
      <input
        ref={input}
        id="global-search"
        type="search"
        placeholder={t.searchPlaceholder}
        value={term}
        onChange={(e) => {
          setTerm(e.target.value);
        }}
        aria-controls="search-results"
      />
      {shown !== undefined && (
        <div id="search-results" className="panel" aria-live="polite">
          {empty && (
            <p>
              {t.searchEmpty(term.trim())}{' '}
              {canCreateTenant && (
                <button
                  className="button button--small"
                  type="button"
                  onClick={() => {
                    open('/locataires');
                  }}
                >
                  {t.newTenant}
                </button>
              )}
            </p>
          )}
          {shown.tenants.length > 0 && (
            <>
              <h2 className="muted">{t.groups.tenants}</h2>
              <ul className="panel__list">
                {shown.tenants.map((x) => (
                  <li key={x.id}>
                    <button
                      className="button button--secondary"
                      type="button"
                      onClick={() => {
                        open(`/locataires/${x.id}`);
                      }}
                    >
                      {x.display_name} {formatPhone(x.phone)}
                    </button>
                  </li>
                ))}
              </ul>
            </>
          )}
          {shown.units.length > 0 && (
            <>
              <h2 className="muted">{t.groups.units}</h2>
              <ul className="panel__list">
                {shown.units.map((u) => (
                  <li key={u.id}>
                    <button
                      className="button button--secondary"
                      type="button"
                      onClick={() => {
                        open(`/lots/${u.id}`);
                      }}
                    >
                      {u.label} · {u.property_name} · {statusLabel(u.status)}
                    </button>
                  </li>
                ))}
              </ul>
            </>
          )}
        </div>
      )}
    </div>
  );
}
