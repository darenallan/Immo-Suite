import { useEffect, useState } from 'react';
import { ApiProblem, api, type AuditFilters, type AuditRow } from '../api/client';
import { messageFor } from '../messages';
import { fr } from '../texts/fr';

function formatDateTime(iso: string): string {
  return new Intl.DateTimeFormat('fr-FR', {
    dateStyle: 'short',
    timeStyle: 'short',
    timeZone: 'Africa/Ouagadougou',
  }).format(new Date(iso));
}

function actorText(row: AuditRow): string {
  if (row.actor.kind === 'support') return fr.audit.support;
  if (row.actor.kind === 'system') return fr.audit.system;
  return row.actor.name ?? '';
}

function Values({ title, value }: { title: string; value: unknown }) {
  if (value === null || value === undefined) return null;
  return (
    <div>
      <strong>{title}</strong>
      <pre className="json">{JSON.stringify(value, null, 2)}</pre>
    </div>
  );
}

/** S21 : journal d'audit en lecture seule, filtres, ligne dépliable pour l'avant et l'après. */
export function AuditScreen() {
  const [filters, setFilters] = useState<AuditFilters>({});
  const [rows, setRows] = useState<AuditRow[]>([]);
  const [actors, setActors] = useState<{ id: string; name: string }[]>([]);
  const [next, setNext] = useState<string | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loaded, setLoaded] = useState(false);

  const fail = (e: unknown) => {
    setError(messageFor(e instanceof ApiProblem ? e : new ApiProblem(0, 'server.unexpected', '')));
    setLoaded(true);
  };

  // première page, rechargée à chaque changement de filtre ; la réponse arrive après le rendu
  useEffect(() => {
    let live = true;
    api.audit(filters).then(
      (page) => {
        if (!live) return;
        setRows(page.items);
        setActors(page.meta.actors);
        setNext(page.next_cursor ?? undefined);
        setError(undefined);
        setLoaded(true);
      },
      (e: unknown) => {
        if (live) fail(e);
      },
    );
    return () => {
      live = false;
    };
  }, [filters]);

  const more = (cursor: string) => {
    api.audit({ ...filters, cursor }).then((page) => {
      setRows((current) => [...current, ...page.items]);
      setNext(page.next_cursor ?? undefined);
    }, fail);
  };

  const set = (key: keyof AuditFilters) => (e: { target: { value: string } }) => {
    const value = e.target.value;
    setFilters((f) => {
      const copy = Object.fromEntries(Object.entries(f).filter(([k]) => k !== key)) as AuditFilters;
      return value === '' ? copy : { ...copy, [key]: value };
    });
  };

  return (
    <div className="stack">
      <h1>{fr.audit.title}</h1>
      <section className="card" aria-label={fr.audit.title}>
        <div className="filters">
          <label className="field">
            <span>{fr.audit.user}</span>
            <select name="actor" value={filters.actor_id ?? ''} onChange={set('actor_id')}>
              <option value="">{fr.audit.all}</option>
              {actors.map((a) => (
                <option key={a.id} value={a.id}>
                  {a.name}
                </option>
              ))}
            </select>
          </label>
          <label className="field">
            <span>{fr.audit.kind}</span>
            <select name="kind" value={filters.actor_kind ?? ''} onChange={set('actor_kind')}>
              <option value="">{fr.audit.all}</option>
              {(Object.keys(fr.audit.kinds) as (keyof typeof fr.audit.kinds)[]).map((k) => (
                <option key={k} value={k}>
                  {fr.audit.kinds[k]}
                </option>
              ))}
            </select>
          </label>
          <label className="field">
            <span>{fr.audit.action}</span>
            <select name="action" value={filters.action ?? ''} onChange={set('action')}>
              <option value="">{fr.audit.all}</option>
              {Object.entries(fr.audit.sentences).map(([code, text]) => (
                <option key={code} value={code}>
                  {text}
                </option>
              ))}
            </select>
          </label>
          <label className="field">
            <span>{fr.audit.from}</span>
            <input name="from" type="date" value={filters.from ?? ''} onChange={set('from')} />
          </label>
          <label className="field">
            <span>{fr.audit.to}</span>
            <input name="to" type="date" value={filters.to ?? ''} onChange={set('to')} />
          </label>
        </div>
        <p className="muted">
          <button className="button button--secondary button--small" type="button" disabled>
            {fr.audit.export}
          </button>{' '}
          {fr.audit.exportSoon}
        </p>
      </section>
      {error !== undefined && (
        <p role="alert" className="notice notice--error">
          {error}
        </p>
      )}
      <section className="card" aria-live="polite">
        {loaded && rows.length === 0 && error === undefined ? (
          <p className="muted">{fr.audit.empty}</p>
        ) : (
          <ul className="rows">
            {rows.map((r) => (
              <li key={r.id} className="row" data-testid="audit-row">
                <details>
                  <summary>
                    <span className="muted">{formatDateTime(r.occurred_at)}</span>{' '}
                    <strong>{actorText(r)}</strong> {fr.audit.sentences[r.action] ?? r.action}
                  </summary>
                  <p className="muted">
                    {r.object.type} · {r.object.id}
                    {r.ip !== undefined && r.ip !== null ? ` · ${fr.audit.ip} ${r.ip}` : ''}
                  </p>
                  <Values title={fr.audit.before} value={r.before} />
                  <Values title={fr.audit.after} value={r.after} />
                </details>
              </li>
            ))}
          </ul>
        )}
        {next !== undefined && (
          <button
            className="button button--secondary"
            type="button"
            onClick={() => {
              more(next);
            }}
          >
            {fr.audit.more}
          </button>
        )}
      </section>
    </div>
  );
}
