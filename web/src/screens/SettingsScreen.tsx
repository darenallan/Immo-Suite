import { useEffect, useState, type SyntheticEvent } from 'react';
import {
  ApiProblem,
  api,
  type OrganizationDetail,
  type RoleSetting,
  type Settings,
  type SupportOverview,
  type TotpEnrollment,
} from '../api/client';
import { ConfirmCancelled, useAuth } from '../auth';
import { messageFor } from '../messages';
import { fr } from '../texts/fr';

function problemText(e: unknown): string | undefined {
  if (e instanceof ConfirmCancelled) return undefined;
  return messageFor(e instanceof ApiProblem ? e : new ApiProblem(0, 'server.unexpected', ''));
}

function Message({ error, ok }: { error: string | undefined; ok?: string | undefined }) {
  if (error !== undefined) {
    return (
      <p role="alert" className="notice notice--error">
        {error}
      </p>
    );
  }
  return ok === undefined ? null : (
    <p role="status" className="notice notice--info">
      {ok}
    </p>
  );
}

/** Section Organisation : nom, ville et paramètres (préfixe, papier, durée des liens, seuil de confirmation). */
function OrganizationSection() {
  const { csrf, reload } = useAuth();
  const [org, setOrg] = useState<OrganizationDetail | undefined>(undefined);
  const [settings, setSettings] = useState<Settings | undefined>(undefined);
  const [form, setForm] = useState({
    name: '',
    city: '',
    prefix: '',
    header: '',
    address: '',
    paper: 'a5',
    days: '30',
    threshold: '',
  });
  const [error, setError] = useState<string | undefined>(undefined);
  const [saved, setSaved] = useState(false);

  const [tick, setTick] = useState(0);
  useEffect(() => {
    let live = true;
    Promise.all([api.organization(), api.settings()]).then(
      ([o, s]) => {
        if (!live) return;
        setOrg(o);
        setSettings(s);
        setForm({
          name: o.name,
          city: o.city ?? '',
          prefix: s.receipt_prefix,
          header: s.receipt_header ?? '',
          address: s.address ?? '',
          paper: s.default_paper,
          days: String(s.share_link_ttl_days),
          threshold: String(s.payment_confirm_threshold),
        });
      },
      (e: unknown) => {
        if (live) setError(problemText(e));
      },
    );
    return () => {
      live = false;
    };
  }, [tick]);

  const save = async (event: SyntheticEvent) => {
    event.preventDefault();
    if (org === undefined || settings === undefined) return;
    setError(undefined);
    setSaved(false);
    try {
      await api.patchOrganization({ name: form.name, city: form.city }, org.version, csrf());
      // le seuil est un montant entier de francs : on l'envoie tel que saisi, le serveur refuse un décimal ou un négatif
      const threshold = /^-?\d+$/.test(form.threshold.trim())
        ? Number(form.threshold.trim())
        : form.threshold;
      await api.patchSettings(
        {
          receipt_prefix: form.prefix,
          receipt_header: form.header,
          address: form.address,
          default_paper: form.paper,
          share_link_ttl_days: Number(form.days),
          payment_confirm_threshold: threshold,
        },
        settings.version,
        csrf(),
      );
      setTick((t) => t + 1);
      await reload();
      setSaved(true);
    } catch (e) {
      setError(problemText(e));
    }
  };

  const set = (key: keyof typeof form) => (e: { target: { value: string } }) => {
    setForm({ ...form, [key]: e.target.value });
  };

  return (
    <details className="card section" open>
      <summary>{fr.settings.organization}</summary>
      <form className="form" onSubmit={(e) => void save(e)} noValidate>
        <Message error={error} ok={saved ? fr.settings.saved : undefined} />
        <label className="field">
          <span>{fr.settings.name}</span>
          <input name="name" type="text" value={form.name} onChange={set('name')} />
        </label>
        <label className="field">
          <span>{fr.settings.city}</span>
          <input name="city" type="text" value={form.city} onChange={set('city')} />
        </label>
        <label className="field">
          <span>{fr.settings.receiptPrefix}</span>
          <input
            name="prefix"
            type="text"
            autoCapitalize="characters"
            value={form.prefix}
            onChange={set('prefix')}
          />
        </label>
        <label className="field">
          <span>{fr.settings.receiptHeader}</span>
          <input name="header" type="text" value={form.header} onChange={set('header')} />
        </label>
        <label className="field">
          <span>{fr.settings.address}</span>
          <input name="address" type="text" value={form.address} onChange={set('address')} />
        </label>
        <label className="field">
          <span>{fr.settings.paper}</span>
          <select name="paper" value={form.paper} onChange={set('paper')}>
            <option value="a5">{fr.settings.paperA5}</option>
            <option value="a4">{fr.settings.paperA4}</option>
          </select>
        </label>
        <label className="field">
          <span>{fr.settings.shareDays}</span>
          <input name="days" type="text" inputMode="numeric" value={form.days} onChange={set('days')} />
        </label>
        <label className="field">
          <span>{fr.settings.threshold}</span>
          <input
            name="threshold"
            type="text"
            inputMode="numeric"
            value={form.threshold}
            onChange={set('threshold')}
          />
        </label>
        <button className="button" type="submit">
          {fr.settings.save}
        </button>
      </form>
    </details>
  );
}

/** Section Sécurité : changement de mot de passe et double authentification (clé affichée une fois, codes de secours une fois). */
function SecuritySection() {
  const { changePassword, csrf, reload } = useAuth();
  const [current, setCurrent] = useState('');
  const [next, setNext] = useState('');
  const [pwError, setPwError] = useState<string | undefined>(undefined);
  const [pwOk, setPwOk] = useState(false);
  const [enroll, setEnroll] = useState<TotpEnrollment | undefined>(undefined);
  const [code, setCode] = useState('');
  const [codes, setCodes] = useState<string[] | undefined>(undefined);
  const [kept, setKept] = useState(false);
  const [totpError, setTotpError] = useState<string | undefined>(undefined);
  const [on, setOn] = useState(false);

  const submitPassword = async (event: SyntheticEvent) => {
    event.preventDefault();
    setPwError(undefined);
    setPwOk(false);
    try {
      await changePassword(current, next);
      setCurrent('');
      setNext('');
      setPwOk(true);
    } catch (e) {
      setPwError(problemText(e));
    }
  };

  const begin = async () => {
    setTotpError(undefined);
    try {
      setEnroll(await api.totpBegin(csrf()));
    } catch (e) {
      setTotpError(problemText(e));
    }
  };

  const confirm = async (event: SyntheticEvent) => {
    event.preventDefault();
    setTotpError(undefined);
    try {
      const done = await api.totpConfirm(code, csrf());
      setCodes(done.backup_codes);
      setEnroll(undefined);
      setCode('');
      setOn(true);
    } catch (e) {
      setTotpError(problemText(e));
    }
  };

  return (
    <details className="card section" open>
      <summary>{fr.settings.security}</summary>
      <div className="stack">
        <form className="form" onSubmit={(e) => void submitPassword(e)} noValidate>
          <h3>{fr.settings.changePassword}</h3>
          <Message error={pwError} ok={pwOk ? fr.settings.passwordChanged : undefined} />
          <label className="field">
            <span>{fr.settings.currentPassword}</span>
            <input
              name="current-password"
              type="password"
              autoComplete="current-password"
              value={current}
              onChange={(e) => {
                setCurrent(e.target.value);
              }}
            />
          </label>
          <label className="field">
            <span>{fr.settings.newPassword}</span>
            <input
              name="new-password"
              type="password"
              autoComplete="new-password"
              value={next}
              onChange={(e) => {
                setNext(e.target.value);
              }}
            />
          </label>
          <button className="button" type="submit" disabled={current === '' || next === ''}>
            {fr.settings.save}
          </button>
        </form>

        <div className="form">
          <h3>{fr.settings.totpTitle}</h3>
          <Message error={totpError} />
          {on && codes === undefined && <p>{fr.settings.totpOn}</p>}
          {!on && enroll === undefined && codes === undefined && (
            <>
              <p className="muted">{fr.settings.totpOff}</p>
              <button
                className="button button--secondary"
                type="button"
                onClick={() => {
                  void begin();
                }}
              >
                {fr.settings.totpStart}
              </button>
            </>
          )}
          {enroll !== undefined && (
            <form className="form" onSubmit={(e) => void confirm(e)} noValidate>
              <p>{fr.settings.totpKey}</p>
              <code className="secret" data-testid="totp-secret">
                {enroll.secret_base32.match(/.{1,4}/g)?.join(' ')}
              </code>
              <a className="button button--secondary" href={enroll.otpauth_url}>
                {fr.settings.totpOpen}
              </a>
              <label className="field">
                <span>{fr.settings.totpCode}</span>
                <input
                  name="totp-code"
                  type="text"
                  inputMode="numeric"
                  autoComplete="one-time-code"
                  value={code}
                  onChange={(e) => {
                    setCode(e.target.value);
                  }}
                />
              </label>
              <button className="button" type="submit" disabled={code.trim() === ''}>
                {fr.settings.totpConfirm}
              </button>
            </form>
          )}
          {codes !== undefined && (
            <div className="form">
              <h4>{fr.settings.backupTitle}</h4>
              <p className="muted">{fr.settings.backupNote}</p>
              <ul className="rows" data-testid="backup-codes">
                {codes.map((c) => (
                  <li key={c}>
                    <code className="secret">{c}</code>
                  </li>
                ))}
              </ul>
              <label className="field field--inline">
                <input
                  type="checkbox"
                  checked={kept}
                  onChange={(e) => {
                    setKept(e.target.checked);
                  }}
                />
                <span>{fr.settings.backupKept}</span>
              </label>
              <button
                className="button"
                type="button"
                disabled={!kept}
                onClick={() => {
                  setCodes(undefined);
                  void reload();
                }}
              >
                {fr.common.close}
              </button>
            </div>
          )}
        </div>
      </div>
    </details>
  );
}

/** Droits réglables : refusés tant qu'ils ne sont pas accordés ; l'effet est immédiat, sans reconnexion de la personne. */
function RightsSection() {
  const { csrf } = useAuth();
  const [items, setItems] = useState<RoleSetting[]>([]);
  const [error, setError] = useState<string | undefined>(undefined);

  const [tick, setTick] = useState(0);
  useEffect(() => {
    let live = true;
    api.roleSettings().then(
      (r) => {
        if (live) setItems(r.settings);
      },
      (e: unknown) => {
        if (live) setError(problemText(e));
      },
    );
    return () => {
      live = false;
    };
  }, [tick]);

  const toggle = async (s: RoleSetting, allowed: boolean) => {
    setError(undefined);
    try {
      await api.putRoleSetting({ role: s.role, action: s.action, allowed }, s.version, csrf());
    } catch (e) {
      setError(problemText(e));
    }
    setTick((t) => t + 1);
  };

  return (
    <details className="card section" open>
      <summary>{fr.settings.rights}</summary>
      <div className="form">
        <p className="muted">{fr.settings.rightsNote}</p>
        <Message error={error} />
        {items.map((s) => {
          const key = `${s.role}.${s.action}`;
          return (
            <label key={key} className="field field--inline">
              <input
                type="checkbox"
                checked={s.allowed}
                onChange={(e) => {
                  void toggle(s, e.target.checked);
                }}
              />
              <span>{fr.settings.rightLabels[key] ?? key}</span>
            </label>
          );
        })}
      </div>
    </details>
  );
}

const DURATIONS = ['24', '72', '168'] as const;

function formatDateTime(iso: string): string {
  return new Intl.DateTimeFormat('fr-FR', {
    dateStyle: 'short',
    timeStyle: 'short',
    timeZone: 'Africa/Ouagadougou',
  }).format(new Date(iso));
}

/** Accès support : accorder à l'équipe HABEK pour une durée limitée, retirer, historique. Confirmation du mot de passe exigée. */
function SupportSection() {
  const { confirmed } = useAuth();
  const [overview, setOverview] = useState<SupportOverview | undefined>(undefined);
  const [scope, setScope] = useState('read');
  const [reason, setReason] = useState('');
  const [hours, setHours] = useState<string>('24');
  const [error, setError] = useState<string | undefined>(undefined);

  const [tick, setTick] = useState(0);
  useEffect(() => {
    let live = true;
    api.supportGrants().then(
      (o) => {
        if (live) setOverview(o);
      },
      (e: unknown) => {
        if (live) setError(problemText(e));
      },
    );
    return () => {
      live = false;
    };
  }, [tick]);

  const grant = async (event: SyntheticEvent) => {
    event.preventDefault();
    setError(undefined);
    try {
      await confirmed((csrf) => api.grantSupport(scope, reason, Number(hours), csrf));
      setReason('');
      setTick((t) => t + 1);
    } catch (e) {
      setError(problemText(e));
    }
  };

  const revoke = async (id: string) => {
    setError(undefined);
    try {
      await confirmed((csrf) => api.revokeSupport(id, csrf));
      setTick((t) => t + 1);
    } catch (e) {
      setError(problemText(e));
    }
  };

  const grants = overview?.grants ?? [];
  const ordered = [...grants].sort((a, b) => Number(b.status === 'active') - Number(a.status === 'active'));
  return (
    <details className="card section" open>
      <summary>{fr.settings.support}</summary>
      <div className="stack">
        <p className="muted">{fr.settings.supportIntro}</p>
        <Message error={error} />
        <form className="form" onSubmit={(e) => void grant(e)} noValidate>
          <label className="field">
            <span>{fr.settings.supportScope}</span>
            <select
              name="support-scope"
              value={scope}
              onChange={(e) => {
                setScope(e.target.value);
              }}
            >
              <option value="read">{fr.settings.supportScopes.read}</option>
              <option value="import_assist">{fr.settings.supportScopes.import_assist}</option>
            </select>
          </label>
          <label className="field">
            <span>{fr.settings.supportReason}</span>
            <input
              name="support-reason"
              type="text"
              value={reason}
              onChange={(e) => {
                setReason(e.target.value);
              }}
            />
          </label>
          <label className="field">
            <span>{fr.settings.supportDuration}</span>
            <select
              name="support-duration"
              value={hours}
              onChange={(e) => {
                setHours(e.target.value);
              }}
            >
              {DURATIONS.map((d) => (
                <option key={d} value={d}>
                  {fr.settings.supportDurations[d]}
                </option>
              ))}
            </select>
          </label>
          <button className="button" type="submit" disabled={reason.trim().length < 5}>
            {fr.settings.supportGrant}
          </button>
        </form>
        {ordered.length === 0 ? (
          <p className="muted">{fr.settings.supportEmpty}</p>
        ) : (
          <ul className="rows">
            {ordered.map((g) => (
              <li key={g.id} className="row" data-testid="grant">
                <div>
                  <span className="tag">
                    {fr.settings.supportStatus[g.status as keyof typeof fr.settings.supportStatus]}
                  </span>{' '}
                  <span className="row__title">
                    {fr.settings.supportScopes[g.scope as keyof typeof fr.settings.supportScopes]}
                  </span>{' '}
                  <span className="muted">
                    {g.reason} · {g.granted_by} · {fr.settings.supportUntil} {formatDateTime(g.expires_at)}
                  </span>
                </div>
                {g.status === 'active' && (
                  <button
                    className="button button--secondary button--small"
                    type="button"
                    onClick={() => {
                      void revoke(g.id);
                    }}
                  >
                    {fr.settings.supportRevoke}
                  </button>
                )}
              </li>
            ))}
          </ul>
        )}
        {(overview?.accesses.length ?? 0) > 0 && (
          <div>
            <h3>{fr.settings.supportViews}</h3>
            <ul className="rows">
              {overview?.accesses.map((a, i) => (
                <li key={`${a.occurred_at}-${String(i)}`} className="row">
                  {formatDateTime(a.occurred_at)} · {a.actor_label} · {a.action} · {a.object_type}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </details>
  );
}

/** S17 : Organisation, Sécurité, Droits réglables et Accès support (selon les droits du rôle). */
export function SettingsScreen({ permissions }: { permissions: readonly string[] }) {
  return (
    <div className="stack">
      <h1>{fr.settings.title}</h1>
      {permissions.includes('settings.write') && <OrganizationSection />}
      <SecuritySection />
      {permissions.includes('user.manage') && <RightsSection />}
      {permissions.includes('support.grant') && <SupportSection />}
    </div>
  );
}
