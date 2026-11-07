// Étape 06 : baux et échéancier (S10 ; F07, F08, DP-94). Le statut d'une échéance, ses montants et ses jours de retard viennent du
// serveur ; l'écran ne calcule rien.
import { useEffect, useState, type SyntheticEvent } from 'react';
import { ApiProblem, api, type Unit, type Tenant } from '../api/client';
import type { components } from '../api/schema';
import { useAuth } from '../auth';
import { messageFor } from '../messages';

type Lease = components['schemas']['LeaseBody'];
type Installment = components['schemas']['InstallmentBody'];
type Navigate = (path: string) => void;

const STATUS: Record<string, string> = {
  upcoming: 'À venir',
  due: 'Due aujourd’hui',
  partial: 'Partiel',
  paid: 'Payé',
  late: 'En retard',
};
const LEASE_STATUS: Record<string, string> = {
  draft: 'Brouillon',
  active: 'Actif',
  notice: 'En préavis',
  ended: 'Terminé',
};
const WARNINGS: Record<string, string> = {
  'lease.deposit_above_two_months':
    'Le dépôt de garantie dépasse deux mois de loyer (avertissement, non bloquant).',
  'lease.due_day_defaulted': 'Jour d’échéance non saisi : le 1er du mois est utilisé.',
};

function fcfa(n: number): string {
  return `${n.toLocaleString('fr-FR').replace(/\s/g, ' ')} FCFA`;
}

function frDate(iso: string | null | undefined): string {
  if (iso === null || iso === undefined) return '—';
  const [y, m, d] = iso.split('-');
  return `${d ?? ''}/${m ?? ''}/${y ?? ''}`;
}

function text(e: unknown): string {
  // chevauchement : le message donne les dates du bail existant (F07 règle 6)
  if (e instanceof ApiProblem && e.code === 'lease.overlap' && typeof e.params['start_date'] === 'string') {
    const to = e.params['end_date'];
    return `Ce lot a déjà un bail du ${frDate(e.params['start_date'])} au ${
      typeof to === 'string' ? frDate(to) : 'sans date de fin'
    }. Terminez-le ou choisissez d’autres dates.`;
  }
  return messageFor(e instanceof ApiProblem ? e : new ApiProblem(0, 'server.unexpected', ''));
}

const KIND: Record<string, string> = { residential: 'Habitation', commercial: 'Commercial' };

/** Création d'un bail en trois étapes (F07, S10) : lot et locataires, conditions, vérification. Le serveur valide tout. */
function LeaseWizard({ onCancel, navigate }: { onCancel: () => void; navigate: Navigate }) {
  const { csrf } = useAuth();
  const [step, setStep] = useState(1);
  const [units, setUnits] = useState<Unit[]>([]);
  const [tenants, setTenants] = useState<Tenant[]>([]);
  const [error, setError] = useState<string | undefined>(undefined);
  const [busy, setBusy] = useState(false);
  // brouillon déjà créé quand l’activation a été refusée : un nouvel essai active ce brouillon, sans en créer un second
  const [created, setCreated] = useState<string | undefined>(undefined);
  const [f, setF] = useState({
    unit: '',
    tenants: [] as string[],
    primary: '',
    kind: 'residential',
    start: '',
    end: '',
    rent: '',
    charges: '',
    dueDay: '5',
    deposit: '',
    vat: false,
  });
  useEffect(() => {
    void api.units({ limit: '100' }).then((p) => {
      setUnits(p.items.filter((u) => u.archived_at === null));
    });
    void api.tenants({ limit: '100' }).then((p) => {
      setTenants(p.items);
    });
  }, []);
  const int = (v: string) => (v.trim() === '' ? undefined : Number(v.replace(/\s/g, '')));
  const unit = units.find((u) => u.id === f.unit);
  const primary = f.primary !== '' && f.tenants.includes(f.primary) ? f.primary : f.tenants[0];
  const names = (ids: string[]) =>
    ids.map((i) => tenants.find((t) => t.id === i)?.display_name ?? '').join(', ');
  const toggle = (id: string) => {
    setF({ ...f, tenants: f.tenants.includes(id) ? f.tenants.filter((t) => t !== id) : [...f.tenants, id] });
  };
  const save = async (activate: boolean) => {
    setError(undefined);
    setBusy(true);
    let leaseId = created;
    try {
      leaseId ??= (
        await api.createLease(
          {
            unit_id: f.unit,
            tenant_ids: f.tenants,
            ...(primary !== undefined ? { primary_tenant_id: primary } : {}),
            kind: f.kind,
            start_date: f.start,
            ...(f.end !== '' ? { end_date: f.end } : {}),
            rent: int(f.rent),
            ...(f.charges !== '' ? { charges: int(f.charges) } : {}),
            ...(f.dueDay !== '' ? { due_day: int(f.dueDay) } : {}),
            ...(f.deposit !== '' ? { deposit_amount: int(f.deposit) } : {}),
            ...(f.kind === 'commercial' ? { vat_included: f.vat } : {}),
          },
          csrf(),
        )
      ).id;
      setCreated(leaseId);
      if (activate) await api.activateLease(leaseId, csrf());
      navigate(`/baux/${leaseId}`);
    } catch (e) {
      setError(leaseId === undefined ? text(e) : `${text(e)} Le brouillon est enregistré.`);
      setBusy(false);
    }
  };
  const field = (
    id: 'start' | 'end' | 'rent' | 'charges' | 'dueDay' | 'deposit',
    label: string,
    type = 'text',
  ) => (
    <div className="field">
      <label htmlFor={`lease-${id}`}>{label}</label>
      <input
        id={`lease-${id}`}
        type={type}
        inputMode={type === 'text' ? 'numeric' : undefined}
        value={f[id]}
        onChange={(e) => {
          setF({ ...f, [id]: e.target.value });
        }}
      />
    </div>
  );
  const ready1 = f.unit !== '' && f.tenants.length > 0;
  const ready2 = f.start !== '' && f.rent.trim() !== '';
  return (
    <form
      className="card form"
      aria-label="Créer un bail"
      noValidate
      onSubmit={(e: SyntheticEvent) => {
        e.preventDefault();
      }}
    >
      <p className="muted" aria-live="polite">
        Étape {step} sur 3
      </p>
      {error !== undefined && (
        <div className="notice notice--error" role="alert">
          {error}
        </div>
      )}
      {step === 1 && (
        <>
          <h2>Lot et locataires</h2>
          <div className="field">
            <label htmlFor="lease-unit">Lot</label>
            <select
              id="lease-unit"
              value={f.unit}
              onChange={(e) => {
                const u = units.find((x) => x.id === e.target.value);
                setF({
                  ...f,
                  unit: e.target.value,
                  kind: u?.usage === 'commercial' ? 'commercial' : f.kind,
                  rent:
                    f.rent === '' && u !== undefined && u.reference_rent !== null
                      ? String(u.reference_rent)
                      : f.rent,
                });
              }}
            >
              <option value="">Choisir un lot</option>
              {units.map((u) => (
                <option key={u.id} value={u.id}>
                  {u.reference} · {u.property_name}
                </option>
              ))}
            </select>
          </div>
          <fieldset className="field">
            <legend>Locataires (plusieurs en colocation)</legend>
            {tenants.map((t) => (
              <div key={t.id} className="choice">
                <input
                  id={`lease-tenant-${t.id}`}
                  type="checkbox"
                  checked={f.tenants.includes(t.id)}
                  onChange={() => {
                    toggle(t.id);
                  }}
                />
                <label htmlFor={`lease-tenant-${t.id}`}>{t.display_name}</label>
              </div>
            ))}
          </fieldset>
          {f.tenants.length > 1 && (
            <div className="field">
              <label htmlFor="lease-primary">Locataire principal (quittances et relances)</label>
              <select
                id="lease-primary"
                value={primary}
                onChange={(e) => {
                  setF({ ...f, primary: e.target.value });
                }}
              >
                {f.tenants.map((i) => (
                  <option key={i} value={i}>
                    {names([i])}
                  </option>
                ))}
              </select>
            </div>
          )}
        </>
      )}
      {step === 2 && (
        <>
          <h2>Conditions</h2>
          <div className="field">
            <label htmlFor="lease-kind">Type de bail</label>
            <select
              id="lease-kind"
              value={f.kind}
              onChange={(e) => {
                setF({ ...f, kind: e.target.value });
              }}
            >
              <option value="residential">Habitation</option>
              <option value="commercial">Commercial</option>
            </select>
          </div>
          {field('start', 'Date de début', 'date')}
          {field('end', 'Date de fin (facultative)', 'date')}
          {field('rent', 'Loyer (FCFA)')}
          {field('charges', 'Charges (FCFA)')}
          {field('dueDay', 'Jour d’échéance (1 à 31)')}
          {field('deposit', 'Dépôt de garantie (FCFA)')}
          {f.kind === 'commercial' && (
            <div className="choice">
              <input
                id="lease-vat"
                type="checkbox"
                checked={f.vat}
                onChange={(e) => {
                  setF({ ...f, vat: e.target.checked });
                }}
              />
              <label htmlFor="lease-vat">Loyer TVA comprise</label>
            </div>
          )}
        </>
      )}
      {step === 3 && (
        <>
          <h2>Vérification</h2>
          <dl className="facts">
            <dt>Lot</dt>
            <dd>{unit !== undefined ? `${unit.reference} · ${unit.property_name}` : '—'}</dd>
            <dt>Locataires</dt>
            <dd>
              {names(f.tenants)}
              {f.tenants.length > 1 && primary !== undefined ? ` (principal : ${names([primary])})` : ''}
            </dd>
            <dt>Type</dt>
            <dd>{KIND[f.kind] ?? f.kind}</dd>
            <dt>Période</dt>
            <dd>
              du {frDate(f.start)} au {f.end === '' ? 'sans date de fin' : frDate(f.end)}
            </dd>
            <dt>Loyer</dt>
            <dd>
              {fcfa(int(f.rent) ?? 0)}
              {f.charges !== '' ? ` + charges ${fcfa(int(f.charges) ?? 0)}` : ''} · échéance le{' '}
              {f.dueDay === '' ? '1er (par défaut)' : f.dueDay}
            </dd>
            <dt>Dépôt de garantie</dt>
            <dd>{fcfa(int(f.deposit) ?? 0)}</dd>
          </dl>
          <p className="muted">
            Les avertissements (dépôt élevé, jour d’échéance) s’affichent sur la fiche du bail.
          </p>
        </>
      )}
      <div className="actions">
        {step > 1 && (
          <button
            className="button button--secondary"
            type="button"
            onClick={() => {
              setStep(step - 1);
            }}
          >
            Précédent
          </button>
        )}
        {step < 3 && (
          <button
            className="button"
            type="button"
            disabled={step === 1 ? !ready1 : !ready2}
            onClick={() => {
              setStep(step + 1);
            }}
          >
            Suivant
          </button>
        )}
        {step === 3 && (
          <>
            <button
              className="button button--secondary"
              type="button"
              disabled={busy}
              onClick={() => void save(false)}
            >
              {created === undefined ? 'Enregistrer le brouillon' : 'Ouvrir le brouillon'}
            </button>
            <button className="button" type="button" disabled={busy} onClick={() => void save(true)}>
              Activer
            </button>
          </>
        )}
        <button className="button button--secondary" type="button" onClick={onCancel}>
          Annuler
        </button>
      </div>
    </form>
  );
}

export function LeasesScreen({ canWrite, navigate }: { canWrite: boolean; navigate: Navigate }) {
  const [items, setItems] = useState<Lease[] | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [form, setForm] = useState(false);

  useEffect(() => {
    api.leases().then(
      (r) => {
        setItems(r.items);
      },
      (e: unknown) => {
        setError(text(e));
      },
    );
  }, []);
  return (
    <section aria-labelledby="leases-title" className="stack">
      <h1 id="leases-title">Baux</h1>
      {error !== undefined && (
        <div className="notice notice--error" role="alert">
          {error}
        </div>
      )}
      {canWrite && !form && (
        <button
          className="button"
          type="button"
          onClick={() => {
            setForm(true);
          }}
        >
          Créer un bail
        </button>
      )}
      {form && (
        <LeaseWizard
          navigate={navigate}
          onCancel={() => {
            setForm(false);
          }}
        />
      )}
      {items !== undefined && items.length === 0 && <p className="card">Aucun bail pour le moment.</p>}
      <ul className="rows">
        {(items ?? []).map((l) => (
          <li key={l.id} className="row">
            <a
              className="row__title"
              href={`/baux/${l.id}`}
              onClick={(e) => {
                e.preventDefault();
                navigate(`/baux/${l.id}`);
              }}
            >
              {l.tenants[0]?.display_name ?? '—'} · {l.unit_reference} · {l.property_name}
            </a>
            <span className="tag">{LEASE_STATUS[l.status] ?? l.status}</span>
            <span>{fcfa(l.current_rent)}</span>
          </li>
        ))}
      </ul>
    </section>
  );
}

export function LeaseScreen({
  id,
  canWrite,
  navigate,
}: {
  id: string;
  canWrite: boolean;
  navigate: Navigate;
}) {
  const { csrf } = useAuth();
  const [lease, setLease] = useState<Lease | undefined>(undefined);
  const [schedule, setSchedule] = useState<Installment[]>([]);
  const [error, setError] = useState<string | undefined>(undefined);
  const [tick, setTick] = useState(0);
  const [amend, setAmend] = useState({ from: '', rent: '' });
  const [end, setEnd] = useState({ on: '', reason: '' });
  const [line, setLine] = useState({ kind: 'arrears', label: '', amount: '', due: '' });

  useEffect(() => {
    let live = true;
    Promise.all([api.lease(id), api.installments(id)]).then(
      ([l, s]) => {
        if (!live) return;
        setLease(l);
        setSchedule(s.items);
      },
      (e: unknown) => {
        if (live) setError(text(e));
      },
    );
    return () => {
      live = false;
    };
  }, [id, tick]);
  const run = async (action: () => Promise<unknown>) => {
    setError(undefined);
    try {
      await action();
      setTick((t) => t + 1);
    } catch (e) {
      setError(text(e));
    }
  };
  if (lease === undefined)
    return (
      <p role="status" className="muted">
        {error ?? 'Chargement…'}
      </p>
    );
  const inProgress = lease.status === 'active' || lease.status === 'notice';
  return (
    <section aria-labelledby="lease-title" className="stack">
      <button
        className="button button--secondary button--small"
        type="button"
        onClick={() => {
          navigate('/baux');
        }}
      >
        Retour
      </button>
      <h1 id="lease-title">
        Bail · {lease.unit_reference} · {lease.property_name}{' '}
        <span className="tag">{LEASE_STATUS[lease.status] ?? lease.status}</span>
      </h1>
      <p>
        {lease.tenants.map((t) => t.display_name).join(', ')} · du {frDate(lease.start_date)} au{' '}
        {frDate(lease.end_date)} · loyer en vigueur {fcfa(lease.current_rent)} · échéance le {lease.due_day}
      </p>
      {lease.warnings.map((w) => (
        <p key={w} className="notice" role="status">
          {WARNINGS[w] ?? w}
        </p>
      ))}
      {error !== undefined && (
        <div className="notice notice--error" role="alert">
          {error}
        </div>
      )}
      {inProgress && (
        <button
          className="button"
          type="button"
          onClick={() => {
            window.history.pushState(null, '', `/encaisser?bail=${lease.id}`);
            navigate('/encaisser');
          }}
        >
          Encaisser
        </button>
      )}
      {canWrite && lease.status === 'draft' && (
        <button
          className="button"
          type="button"
          onClick={() => void run(() => api.activateLease(id, csrf()))}
        >
          Activer le bail
        </button>
      )}
      {lease.amendments.length > 0 && (
        <>
          <h2>Avenants</h2>
          <ul className="rows" aria-label="Avenants">
            {lease.amendments.map((m) => (
              <li key={m.id} className="row">
                <span className="row__title">à partir du {frDate(m.effective_from)}</span>
                <span>
                  {[
                    m.new_rent !== null && m.new_rent !== undefined ? `loyer ${fcfa(m.new_rent)}` : '',
                    m.new_charges !== null && m.new_charges !== undefined
                      ? `charges ${fcfa(m.new_charges)}`
                      : '',
                    m.new_end_date !== null && m.new_end_date !== undefined
                      ? `fin au ${frDate(m.new_end_date)}`
                      : '',
                  ]
                    .filter((x) => x !== '')
                    .join(' · ')}
                </span>
                <span className="muted">saisi le {frDate(m.entered_on)}</span>
              </li>
            ))}
          </ul>
        </>
      )}
      <h2>Échéancier</h2>
      {schedule.length === 0 && <p className="muted">Aucune échéance pour le moment.</p>}
      <ul className="rows" aria-label="Échéancier">
        {schedule.map((i) => (
          <li key={i.id} className="row" data-testid="installment">
            <span className="row__title">{frDate(i.due_date)}</span>
            <span>{fcfa(i.due_amount)}</span>
            <span className={`tag tag--${i.status}`}>
              {i.due_amount === 0 && i.lines.every((l) => l.voided)
                ? 'Annulée'
                : (STATUS[i.status] ?? i.status)}
              {i.status === 'late' ? ` · ${String(i.days_late)} jour${i.days_late > 1 ? 's' : ''}` : ''}
            </span>
            {canWrite &&
              i.lines
                .filter((l) => !l.voided && (l.kind === 'arrears' || l.kind === 'exceptional'))
                .map((l) => (
                  <button
                    key={l.id}
                    className="button button--secondary button--small"
                    type="button"
                    onClick={() =>
                      void run(() => api.voidLine(l.id, 'Annulée depuis la fiche du bail', csrf()))
                    }
                  >
                    Annuler « {l.label ?? l.kind} »
                  </button>
                ))}
          </li>
        ))}
      </ul>
      {canWrite && inProgress && (
        <form
          className="card form"
          aria-label="Ajouter une ligne"
          onSubmit={(e) => {
            e.preventDefault();
            void run(() =>
              api.addLine(
                id,
                {
                  kind: line.kind,
                  label: line.label,
                  amount: Number(line.amount.replace(/\s/g, '')),
                  due_date: line.due,
                },
                csrf(),
              ),
            );
          }}
        >
          <h2>Ligne d’arriéré ou exceptionnelle</h2>
          <div className="field">
            <label htmlFor="line-kind">Nature</label>
            <select
              id="line-kind"
              value={line.kind}
              onChange={(e) => {
                setLine({ ...line, kind: e.target.value });
              }}
            >
              <option value="arrears">Arriéré</option>
              <option value="exceptional">Exceptionnelle</option>
            </select>
          </div>
          <div className="field">
            <label htmlFor="line-label">Libellé</label>
            <input
              id="line-label"
              value={line.label}
              onChange={(e) => {
                setLine({ ...line, label: e.target.value });
              }}
            />
          </div>
          <div className="field">
            <label htmlFor="line-amount">Montant (FCFA)</label>
            <input
              id="line-amount"
              value={line.amount}
              onChange={(e) => {
                setLine({ ...line, amount: e.target.value });
              }}
            />
          </div>
          <div className="field">
            <label htmlFor="line-due">Date</label>
            <input
              id="line-due"
              type="date"
              value={line.due}
              onChange={(e) => {
                setLine({ ...line, due: e.target.value });
              }}
            />
          </div>
          <button className="button" type="submit">
            Ajouter la ligne
          </button>
        </form>
      )}
      {canWrite && inProgress && (
        <form
          className="card form"
          aria-label="Ajouter un avenant"
          onSubmit={(e) => {
            e.preventDefault();
            void run(() =>
              api.amendLease(
                id,
                { effective_from: amend.from, rent: Number(amend.rent.replace(/\s/g, '')) },
                csrf(),
              ),
            );
          }}
        >
          <h2>Avenant</h2>
          <div className="field">
            <label htmlFor="amend-from">Date d’effet</label>
            <input
              id="amend-from"
              type="date"
              value={amend.from}
              onChange={(e) => {
                setAmend({ ...amend, from: e.target.value });
              }}
            />
          </div>
          <div className="field">
            <label htmlFor="amend-rent">Nouveau loyer (FCFA)</label>
            <input
              id="amend-rent"
              value={amend.rent}
              onChange={(e) => {
                setAmend({ ...amend, rent: e.target.value });
              }}
            />
          </div>
          <button className="button" type="submit">
            Ajouter l’avenant
          </button>
        </form>
      )}
      {canWrite && inProgress && (
        <form
          className="card form"
          aria-label="Résilier le bail"
          onSubmit={(e) => {
            e.preventDefault();
            void run(() => api.terminateLease(id, { terminated_on: end.on, reason: end.reason }, csrf()));
          }}
        >
          <h2>Résiliation</h2>
          <div className="field">
            <label htmlFor="end-on">Date de fin</label>
            <input
              id="end-on"
              type="date"
              value={end.on}
              onChange={(e) => {
                setEnd({ ...end, on: e.target.value });
              }}
            />
          </div>
          <div className="field">
            <label htmlFor="end-reason">Motif</label>
            <input
              id="end-reason"
              value={end.reason}
              onChange={(e) => {
                setEnd({ ...end, reason: e.target.value });
              }}
            />
          </div>
          <button className="button button--secondary" type="submit">
            Résilier
          </button>
        </form>
      )}
    </section>
  );
}
