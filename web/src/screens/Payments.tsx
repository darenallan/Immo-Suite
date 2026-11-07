// Étape 07 : encaisser (S04), journal des paiements (S11), aperçu de la quittance (S25) ; F09, F10, DP-96.
// Le front ne calcule aucun montant (DP-45) : l'affectation, l'avance, le solde et le montant en lettres viennent du serveur.
import { useEffect, useRef, useState } from 'react';
import {
  ApiProblem,
  api,
  type PaymentContext,
  type PaymentPreview,
  type PaymentRow,
  type ReceiptBody,
} from '../api/client';
import type { components } from '../api/schema';
import { useAuth } from '../auth';
import { messageFor } from '../messages';
import { track } from '../observation/track';

type Lease = components['schemas']['LeaseBody'];
type Navigate = (path: string) => void;

const METHODS: [string, string][] = [
  ['cash', 'Espèces'],
  ['mobile_money', 'Mobile money'],
  ['bank_transfer', 'Virement'],
  ['cheque', 'Chèque'],
];
const KIND: Record<string, string> = {
  rent: 'Loyer',
  charges: 'Charges',
  exceptional: 'Exceptionnelle',
  arrears: 'Arriéré',
};

function fcfa(n: number): string {
  return `${n.toLocaleString('fr-FR').replace(/\s/g, ' ')} FCFA`;
}

function frDate(iso: string | null | undefined): string {
  if (iso === null || iso === undefined) return '—';
  const [y, m, d] = iso.slice(0, 10).split('-');
  return `${d ?? ''}/${m ?? ''}/${y ?? ''}`;
}

function problemOf(e: unknown): ApiProblem {
  return e instanceof ApiProblem ? e : new ApiProblem(0, 'server.unexpected', '');
}

function newKey(): string {
  return crypto.randomUUID();
}

// ───────── S25 : quittance en HTML, depuis l'instantané seulement ─────────

export function ReceiptView({ receipt }: { receipt: ReceiptBody }) {
  const s = receipt.snapshot;
  const place = [s.unit.address, s.unit.district, s.unit.city]
    .filter((x) => x !== null && x !== '')
    .join(', ');
  return (
    <article className="card receipt" aria-label={`Quittance ${receipt.number}`}>
      {s.receipt.watermark !== null && (
        <p className="receipt__watermark" aria-label="Quittance d’essai">
          {s.receipt.watermark}
        </p>
      )}
      <header>
        <p className="muted">{s.issuer.name}</p>
        <h2>Quittance n° {receipt.number}</h2>
        <p>
          Émise le {frDate(s.receipt.issued_on)} par {s.receipt.issued_by}
          {receipt.status === 'cancelled' ? ' · Annulée' : ''}
        </p>
      </header>
      <dl className="facts">
        <dt>Locataire</dt>
        <dd>{s.tenants.join(', ')}</dd>
        {s.payer.mention !== undefined && (
          <>
            <dt>Payé par</dt>
            <dd>{s.payer.mention}</dd>
          </>
        )}
        <dt>Lot</dt>
        <dd>
          {s.unit.reference} · {s.unit.property_name}
          {place !== '' ? ` · ${place}` : ''}
          {s.unit.vat_included ? ' · TVA incluse' : ''}
        </dd>
        <dt>Montant</dt>
        <dd>
          <strong>{s.payment.amount_text}</strong> ({s.payment.amount_in_words})
        </dd>
        <dt>Paiement</dt>
        <dd>
          le {frDate(s.payment.paid_on)} · {s.payment.method_label}
          {s.payment.external_ref !== null ? ` · réf. ${s.payment.external_ref}` : ''}
        </dd>
      </dl>
      {s.breakdown.map((b) => (
        <section key={b.period} aria-label={`Période ${b.period_label}`}>
          <h3>{b.period_label}</h3>
          <ul className="rows">
            {b.lines.map((l, i) => (
              <li key={`${l.kind}-${String(i)}`} className="row">
                <span className="row__title">{l.label}</span>
                <span>{fcfa(l.amount)}</span>
              </li>
            ))}
          </ul>
        </section>
      ))}
      <p>
        Solde restant : <strong>{fcfa(s.remaining_after)}</strong>
        {s.advance_amount > 0 ? ` · Avance : ${fcfa(s.advance_amount)}` : ''}
      </p>
      {s.issuer.extra_mentions !== null && <p className="muted">{s.issuer.extra_mentions}</p>}
    </article>
  );
}

// ───────── S04 : encaisser ─────────

type Pending = 'threshold' | 'surplus' | 'duplicate';

function PaymentForm({ leaseId, onDone }: { leaseId: string; onDone: (r: ReceiptBody) => void }) {
  const { csrf } = useAuth();
  // clé créée à l'ouverture du formulaire et réutilisée à chaque nouvel essai (DP-49, API §5.1)
  const [key] = useState(newKey);
  const [ctx, setCtx] = useState<PaymentContext | undefined>(undefined);
  const [amount, setAmount] = useState('');
  const [paidOn, setPaidOn] = useState('');
  const [method, setMethod] = useState('cash');
  const [ref, setRef] = useState('');
  const [payer, setPayer] = useState('');
  const [thirdName, setThirdName] = useState('');
  const [thirdPhone, setThirdPhone] = useState('');
  const [note, setNote] = useState('');
  const [preview, setPreview] = useState<PaymentPreview | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [lost, setLost] = useState(false);
  const [busy, setBusy] = useState(false);
  const [pending, setPending] = useState<{ kind: Pending; text: string } | undefined>(undefined);
  const [confirmed, setConfirmed] = useState({ threshold: false, surplus: false, duplicate: false });
  const done = useRef(false);

  useEffect(() => {
    track({ name: 'payment_form_opened', properties: { screen: 'S04' } });
    return () => {
      if (!done.current) track({ name: 'payment_form_abandoned', properties: { screen: 'S04' } });
    };
  }, []);
  useEffect(() => {
    api.paymentContext(leaseId).then(
      (c) => {
        setCtx(c);
        setAmount(c.suggested_amount > 0 ? String(c.suggested_amount) : '');
        setPaidOn(c.today);
        setPayer(c.tenants.find((t) => t.is_primary)?.id ?? c.tenants[0]?.id ?? '');
      },
      (e: unknown) => {
        setError(messageFor(problemOf(e)));
      },
    );
  }, [leaseId]);
  const value = Number(amount.replace(/\s/g, ''));
  const valid = amount.trim() !== '' && Number.isInteger(value) && value > 0;
  // aperçu d'affectation du serveur, 300 ms après la dernière frappe
  useEffect(() => {
    if (ctx === undefined || !valid) return;
    const t = window.setTimeout(() => {
      api.previewPayment({ lease_id: leaseId, amount: value }, csrf()).then(setPreview, () => {
        setPreview(undefined);
      });
    }, 300);
    return () => {
      window.clearTimeout(t);
    };
  }, [ctx, leaseId, value, valid, csrf]);

  const submit = async (flags = confirmed) => {
    setError(undefined);
    setLost(false);
    setBusy(true);
    const body: Record<string, unknown> = {
      lease_id: leaseId,
      amount: value,
      paid_on: paidOn,
      method,
      ...(ref.trim() !== '' && method !== 'cash' ? { external_ref: ref.trim() } : {}),
      payer:
        payer === 'third_party'
          ? {
              kind: 'third_party',
              third_party_name: thirdName.trim(),
              ...(thirdPhone.trim() !== '' ? { third_party_phone: thirdPhone.trim() } : {}),
            }
          : { kind: 'tenant', tenant_id: payer },
      ...(note.trim() !== '' ? { note: note.trim() } : {}),
      ...(flags.surplus ? { surplus: 'advance' } : {}),
      ...(flags.threshold ? { confirm_high_amount: true } : {}),
      ...(flags.duplicate ? { acknowledge_external_ref_duplicate: true } : {}),
    };
    try {
      const r = await api.recordPayment(body, key, csrf());
      done.current = true;
      onDone(r.receipt);
    } catch (e) {
      const p = problemOf(e);
      if (p.code === 'network.unreachable') {
        setLost(true);
      } else if (p.code === 'payment.confirmation_required') {
        setPending({
          kind: 'threshold',
          text: `Ce montant dépasse ${fcfa(Number(p.params['threshold'] ?? 0))}. Confirmez-vous ${fcfa(value)} ?`,
        });
      } else if (p.code === 'payment.surplus_not_confirmed') {
        setPending({
          kind: 'surplus',
          text: `Le montant dépasse ce qui est dû aujourd’hui. Enregistrer ${fcfa(Number(p.params['surplus_amount'] ?? 0))} en avance ?`,
        });
      } else if (p.code === 'payment.external_ref_duplicate') {
        const n = typeof p.params['receipt_number'] === 'string' ? p.params['receipt_number'] : '';
        setPending({
          kind: 'duplicate',
          text: `Ce paiement semble déjà enregistré (quittance ${n}). Est-ce bien un autre paiement ?`,
        });
      } else {
        setError(messageFor(p));
      }
    } finally {
      setBusy(false);
    }
  };

  if (ctx === undefined)
    return (
      <p role="status" className="muted">
        {error ?? 'Chargement…'}
      </p>
    );
  return (
    <form
      className="card form"
      aria-label="Encaisser"
      noValidate
      onSubmit={(e) => {
        e.preventDefault();
        void submit();
      }}
    >
      <p>
        <strong>{ctx.tenants.map((t) => t.display_name).join(', ')}</strong> · {ctx.lease.unit.label} ·{' '}
        {ctx.lease.unit.property_name}
      </p>
      <p className="muted" data-testid="balance">
        Reste dû aujourd’hui : {fcfa(ctx.balance.remaining_amount)}
        {ctx.balance.credit_amount > 0 ? ` · Crédit du bail : ${fcfa(ctx.balance.credit_amount)}` : ''}
      </p>
      {error !== undefined && (
        <div className="notice notice--error" role="alert">
          {error}
        </div>
      )}
      {lost && (
        <div className="notice notice--error" role="alert">
          Connexion perdue. Vos informations sont conservées. Réessayez dès que le réseau revient. Si le
          paiement a déjà été enregistré, il ne sera pas créé deux fois.
        </div>
      )}
      <div className="field">
        <label htmlFor="pay-amount">Montant (FCFA)</label>
        <input
          id="pay-amount"
          inputMode="numeric"
          value={amount}
          onChange={(e) => {
            setAmount(e.target.value);
          }}
        />
      </div>
      <div className="field">
        <label htmlFor="pay-date">Date du paiement</label>
        <input
          id="pay-date"
          type="date"
          max={ctx.today}
          value={paidOn}
          onChange={(e) => {
            setPaidOn(e.target.value);
          }}
        />
      </div>
      <div className="field">
        <label htmlFor="pay-method">Mode</label>
        <select
          id="pay-method"
          value={method}
          onChange={(e) => {
            setMethod(e.target.value);
          }}
        >
          {METHODS.map(([v, l]) => (
            <option key={v} value={v}>
              {l}
            </option>
          ))}
        </select>
      </div>
      {method !== 'cash' && (
        <div className="field">
          <label htmlFor="pay-ref">Référence</label>
          <input
            id="pay-ref"
            value={ref}
            onChange={(e) => {
              setRef(e.target.value);
            }}
          />
        </div>
      )}
      <div className="field">
        <label htmlFor="pay-payer">Payé par</label>
        <select
          id="pay-payer"
          value={payer}
          onChange={(e) => {
            setPayer(e.target.value);
          }}
        >
          {ctx.tenants.map((t) => (
            <option key={t.id} value={t.id}>
              {t.display_name}
            </option>
          ))}
          <option value="third_party">Un tiers</option>
        </select>
      </div>
      {payer === 'third_party' && (
        <>
          <div className="field">
            <label htmlFor="pay-third-name">Nom du tiers</label>
            <input
              id="pay-third-name"
              value={thirdName}
              onChange={(e) => {
                setThirdName(e.target.value);
              }}
            />
          </div>
          <div className="field">
            <label htmlFor="pay-third-phone">Téléphone du tiers (facultatif)</label>
            <input
              id="pay-third-phone"
              inputMode="tel"
              value={thirdPhone}
              onChange={(e) => {
                setThirdPhone(e.target.value);
              }}
            />
          </div>
        </>
      )}
      <div className="field">
        <label htmlFor="pay-note">Note</label>
        <input
          id="pay-note"
          value={note}
          onChange={(e) => {
            setNote(e.target.value);
          }}
        />
      </div>
      {preview !== undefined && valid && (
        <section aria-label="Affectation">
          <h3>Affectation</h3>
          <ul className="rows">
            {preview.allocations.map((a) => (
              <li key={a.installment_line_id} className="row">
                <span className="row__title">
                  {KIND[a.kind ?? ''] ?? a.kind} · échéance du {frDate(a.due_date)}
                </span>
                <span>{fcfa(a.amount)}</span>
              </li>
            ))}
          </ul>
          <p className="muted">
            Reste dû après ce paiement : {fcfa(preview.remaining_after)}
            {preview.surplus_amount > 0 ? ` · Avance : ${fcfa(preview.surplus_amount)}` : ''}
          </p>
        </section>
      )}
      {pending !== undefined && (
        <div className="notice" role="alertdialog" aria-label="Confirmation">
          <p>{pending.text}</p>
          <div className="actions">
            <button
              className="button"
              type="button"
              onClick={() => {
                const flags = { ...confirmed, [pending.kind]: true };
                setConfirmed(flags);
                setPending(undefined);
                void submit(flags);
              }}
            >
              Confirmer
            </button>
            <button
              className="button button--secondary"
              type="button"
              onClick={() => {
                setPending(undefined);
              }}
            >
              Revenir
            </button>
          </div>
        </div>
      )}
      <div className="sticky-action">
        <button
          className="button button--large"
          type="submit"
          disabled={busy || !valid || pending !== undefined}
        >
          Encaisser et envoyer la quittance
        </button>
      </div>
    </form>
  );
}

export function CollectScreen({ navigate }: { navigate: Navigate }) {
  const initial = new URLSearchParams(window.location.search).get('bail');
  const [leaseId, setLeaseId] = useState<string | undefined>(initial ?? undefined);
  const [leases, setLeases] = useState<Lease[] | undefined>(undefined);
  const [filter, setFilter] = useState('');
  const [receipt, setReceipt] = useState<ReceiptBody | undefined>(undefined);
  // chaque formulaire a sa propre clé : « Encaisser un autre loyer » en ouvre un nouveau
  const [round, setRound] = useState(0);

  useEffect(() => {
    api.leases().then(
      (r) => {
        setLeases(r.items.filter((l) => l.status === 'active' || l.status === 'notice'));
      },
      () => {
        setLeases([]);
      },
    );
  }, []);
  const words = filter.trim().toLowerCase();
  const shown = (leases ?? []).filter(
    (l) =>
      words === '' ||
      `${l.tenants.map((t) => t.display_name).join(' ')} ${l.unit_reference} ${l.property_name}`
        .toLowerCase()
        .includes(words),
  );
  return (
    <section aria-labelledby="collect-title" className="stack">
      <h1 id="collect-title">Encaisser</h1>
      {receipt !== undefined ? (
        <>
          <div className="notice" role="status">
            Paiement enregistré · quittance {receipt.number}
          </div>
          <ReceiptView receipt={receipt} />
          <div className="actions">
            <button
              className="button"
              type="button"
              onClick={() => {
                setReceipt(undefined);
                setLeaseId(undefined);
                setRound((r) => r + 1);
              }}
            >
              Encaisser un autre loyer
            </button>
            <button
              className="button button--secondary"
              type="button"
              onClick={() => {
                navigate('/paiements');
              }}
            >
              Voir le journal
            </button>
          </div>
        </>
      ) : leaseId === undefined ? (
        <>
          <div className="field">
            <label htmlFor="collect-search">Locataire ou lot</label>
            <input
              id="collect-search"
              type="search"
              value={filter}
              onChange={(e) => {
                setFilter(e.target.value);
              }}
            />
          </div>
          {leases !== undefined && shown.length === 0 && <p className="muted">Aucun bail en cours.</p>}
          <ul className="rows">
            {shown.map((l) => (
              <li key={l.id} className="row">
                <button
                  className="row__title button--link"
                  type="button"
                  onClick={() => {
                    setLeaseId(l.id);
                  }}
                >
                  {l.tenants[0]?.display_name ?? '—'} · {l.unit_reference} · {l.property_name}
                </button>
                <span>{fcfa(l.current_rent)}</span>
              </li>
            ))}
          </ul>
        </>
      ) : (
        <>
          <button
            className="button button--secondary button--small"
            type="button"
            onClick={() => {
              setLeaseId(undefined);
            }}
          >
            Changer de bail
          </button>
          <PaymentForm key={`${leaseId}-${String(round)}`} leaseId={leaseId} onDone={setReceipt} />
        </>
      )}
    </section>
  );
}

// ───────── S11 : journal et détail ─────────

const METHOD_LABEL: Record<string, string> = Object.fromEntries(METHODS);

export function PaymentsScreen({ navigate }: { navigate: Navigate }) {
  const [items, setItems] = useState<PaymentRow[] | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  useEffect(() => {
    api.payments().then(
      (r) => {
        setItems(r.items);
      },
      (e: unknown) => {
        setError(messageFor(problemOf(e)));
      },
    );
  }, []);
  return (
    <section aria-labelledby="payments-title" className="stack">
      <h1 id="payments-title">Paiements et quittances</h1>
      {error !== undefined && (
        <div className="notice notice--error" role="alert">
          {error}
        </div>
      )}
      {items !== undefined && items.length === 0 && <p className="card">Aucun paiement sur cette période.</p>}
      <ul className="rows" aria-label="Journal des paiements">
        {(items ?? []).map((p) => (
          <li key={p.id} className="row" data-testid="payment-row">
            <a
              className="row__title"
              href={`/paiements/${p.id}`}
              onClick={(e) => {
                e.preventDefault();
                navigate(`/paiements/${p.id}`);
              }}
            >
              {frDate(p.paid_on)} · {p.tenants ?? '—'} · {p.unit_reference}
            </a>
            <span>{fcfa(p.amount)}</span>
            <span>{METHOD_LABEL[p.method] ?? p.method}</span>
            <span>{p.receipt_number ?? ''}</span>
            <span className="tag">Validé</span>
          </li>
        ))}
      </ul>
    </section>
  );
}

export function PaymentScreen({ id, navigate }: { id: string; navigate: Navigate }) {
  const [p, setP] = useState<Awaited<ReturnType<typeof api.payment>> | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  useEffect(() => {
    api.payment(id).then(setP, (e: unknown) => {
      setError(messageFor(problemOf(e)));
    });
  }, [id]);
  if (p === undefined)
    return (
      <p role="status" className="muted">
        {error ?? 'Chargement…'}
      </p>
    );
  return (
    <section aria-labelledby="payment-title" className="stack">
      <button
        className="button button--secondary button--small"
        type="button"
        onClick={() => {
          navigate('/paiements');
        }}
      >
        Retour
      </button>
      <h1 id="payment-title">
        Paiement de {fcfa(p.amount)} · {p.receipt_number ?? ''} <span className="tag">Validé</span>
      </h1>
      <p>
        Reçu par {p.received_by.display_name} · {METHOD_LABEL[p.method] ?? p.method}
        {p.external_ref !== null ? ` · réf. ${p.external_ref}` : ''}
      </p>
      <ul className="rows" aria-label="Affectations">
        {p.allocations.map((a) => (
          <li key={`${a.installment_line_id}-${String(a.amount)}`} className="row">
            <span className="row__title">
              {KIND[a.kind ?? ''] ?? a.kind} · échéance du {frDate(a.due_date)}
            </span>
            <span>{fcfa(a.amount)}</span>
          </li>
        ))}
      </ul>
      {p.receipt !== null && <ReceiptView receipt={p.receipt} />}
    </section>
  );
}
