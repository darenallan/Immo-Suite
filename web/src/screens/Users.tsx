import { useEffect, useState, type SyntheticEvent } from 'react';
import { ApiProblem, api, type Invitation, type Member } from '../api/client';
import { ConfirmCancelled, useAuth } from '../auth';
import { messageFor } from '../messages';
import { fr, type RoleKey } from '../texts/fr';

/** Un lien à remettre : affiché une seule fois (invitation ou réinitialisation). */
interface Shown {
  title: string;
  note: string;
  url: string;
  whatsapp: string | undefined;
}

const AGENCY_ROLES: readonly RoleKey[] = [
  'manager',
  'property_manager',
  'cashier',
  'field_agent',
  'accountant',
];

function problemText(e: unknown): string | undefined {
  if (e instanceof ConfirmCancelled) return undefined;
  return messageFor(e instanceof ApiProblem ? e : new ApiProblem(0, 'server.unexpected', ''));
}

/** S18 : équipe, invitations, rôles, désactivation, liens de réinitialisation. Les actions à enjeu demandent le mot de passe. */
export function Users({ organizationName, plan }: { organizationName: string; plan: string }) {
  const { confirmed } = useAuth();
  const [members, setMembers] = useState<Member[]>([]);
  const [invitations, setInvitations] = useState<Invitation[]>([]);
  const [shown, setShown] = useState<Shown | undefined>(undefined);
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState<string | undefined>(undefined);
  const [identifier, setIdentifier] = useState('');
  const invitableRoles: readonly RoleKey[] = plan === 'owner' ? ['collaborator'] : AGENCY_ROLES;
  const [role, setRole] = useState<string>(invitableRoles[invitableRoles.length - 1] ?? 'cashier');

  // chargement : la réponse arrive après le rendu ; `tick` demande un nouveau chargement après une action
  const [tick, setTick] = useState(0);
  useEffect(() => {
    let live = true;
    Promise.all([api.users(), api.invitations()]).then(
      ([u, i]) => {
        if (!live) return;
        setMembers(u.users);
        setInvitations(i.invitations);
      },
      (e: unknown) => {
        if (live) setError(problemText(e));
      },
    );
    return () => {
      live = false;
    };
  }, [tick]);

  /** Exécute une action, puis recharge ; l'erreur s'affiche en tête de l'écran. */
  const run = async (action: () => Promise<void>) => {
    setError(undefined);
    try {
      await action();
    } catch (e) {
      setError(problemText(e));
    }
    setTick((t) => t + 1);
  };

  const showInvitation = (url: string) => {
    setCopied(false);
    setShown({
      title: fr.users.linkTitle,
      note: fr.users.linkNote,
      url,
      whatsapp: `https://wa.me/?text=${encodeURIComponent(fr.users.whatsappText(organizationName, url))}`,
    });
  };

  const invite = async (event: SyntheticEvent) => {
    event.preventDefault();
    await run(async () => {
      const created = await confirmed((csrf) => api.invite(identifier, role, csrf));
      setIdentifier('');
      showInvitation(created.url);
    });
  };

  const copy = async (url: string) => {
    try {
      await navigator.clipboard.writeText(url);
      setCopied(true);
    } catch {
      setCopied(false);
    }
  };

  return (
    <div className="stack">
      <h1>{fr.users.title}</h1>
      {error !== undefined && (
        <p role="alert" className="notice notice--error">
          {error}
        </p>
      )}
      {shown !== undefined && (
        <section className="card form" aria-labelledby="link-title">
          <h2 id="link-title">{shown.title}</h2>
          <p className="muted">{shown.note}</p>
          <code className="secret" data-testid="shown-link">
            {shown.url}
          </code>
          <div className="actions">
            <button
              className="button"
              type="button"
              onClick={() => {
                void copy(shown.url);
              }}
            >
              {copied ? fr.users.copied : fr.users.copy}
            </button>
            {shown.whatsapp !== undefined && (
              <a
                className="button button--secondary"
                href={shown.whatsapp}
                target="_blank"
                rel="noopener noreferrer"
              >
                {fr.users.whatsapp}
              </a>
            )}
          </div>
        </section>
      )}

      <section className="card" aria-labelledby="members-title">
        <h2 id="members-title">{fr.users.members}</h2>
        {members.length === 0 ? (
          <p className="muted">{fr.users.empty}</p>
        ) : (
          <ul className="rows">
            {members.map((m) => (
              <li key={m.membership_id} className="row" data-testid="member">
                <div>
                  <span className="row__title">
                    {m.display_name}
                    {m.is_self ? fr.users.youSuffix : ''}
                  </span>{' '}
                  <span className="tag">{fr.roles[m.role as RoleKey]}</span>{' '}
                  <span className="muted">
                    {m.identifier} · {fr.users.status[m.status === 'disabled' ? 'disabled' : 'active']}
                  </span>
                </div>
                {!m.is_self && (
                  <div className="actions">
                    {plan !== 'owner' && m.status === 'active' && (
                      <label className="field field--inline">
                        <span>{fr.users.changeRole}</span>
                        <select
                          aria-label={`${fr.users.changeRole} : ${m.display_name}`}
                          value={m.role}
                          onChange={(e) => {
                            // la valeur est lue ICI : la liste revient à l'ancien rôle tant que l'action n'a pas abouti, et la
                            // confirmation du mot de passe rejoue l'action plus tard
                            const wanted = e.target.value;
                            void run(async () => {
                              await confirmed((csrf) => api.changeRole(m.membership_id, wanted, csrf));
                            });
                          }}
                        >
                          {AGENCY_ROLES.map((r) => (
                            <option key={r} value={r}>
                              {fr.roles[r]}
                            </option>
                          ))}
                        </select>
                      </label>
                    )}
                    {m.status === 'active' ? (
                      <>
                        <button
                          className="button button--secondary button--small"
                          type="button"
                          onClick={() => {
                            void run(async () => {
                              await confirmed((csrf) => api.disableUser(m.membership_id, csrf));
                            });
                          }}
                        >
                          {fr.users.disable}
                        </button>
                        {m.role !== 'manager' && m.role !== 'owner' && (
                          <button
                            className="button button--secondary button--small"
                            type="button"
                            onClick={() => {
                              void run(async () => {
                                const link = await confirmed((csrf) => api.issueReset(m.membership_id, csrf));
                                setCopied(false);
                                setShown({
                                  title: fr.users.resetTitle,
                                  note: fr.users.resetNote,
                                  url: link.url,
                                  whatsapp: undefined,
                                });
                              });
                            }}
                          >
                            {fr.users.reset}
                          </button>
                        )}
                      </>
                    ) : (
                      <button
                        className="button button--secondary button--small"
                        type="button"
                        onClick={() => {
                          void run(async () => {
                            await confirmed((csrf) => api.enableUser(m.membership_id, csrf));
                          });
                        }}
                      >
                        {fr.users.enable}
                      </button>
                    )}
                  </div>
                )}
              </li>
            ))}
          </ul>
        )}
      </section>

      <section className="card" aria-labelledby="invitations-title">
        <h2 id="invitations-title">{fr.users.invitationsTitle}</h2>
        <form className="form" onSubmit={(e) => void invite(e)} noValidate aria-label={fr.users.inviteTitle}>
          <label className="field">
            <span>{fr.users.inviteIdentifier}</span>
            <input
              name="invite-identifier"
              type="text"
              inputMode="email"
              autoCapitalize="none"
              spellCheck={false}
              placeholder="70 12 34 56"
              value={identifier}
              onChange={(e) => {
                setIdentifier(e.target.value);
              }}
            />
          </label>
          <label className="field">
            <span>{fr.users.inviteRole}</span>
            <select
              name="invite-role"
              value={role}
              onChange={(e) => {
                setRole(e.target.value);
              }}
            >
              {invitableRoles.map((r) => (
                <option key={r} value={r}>
                  {fr.roles[r]}
                </option>
              ))}
            </select>
          </label>
          <button className="button" type="submit" disabled={identifier.trim() === ''}>
            {fr.users.inviteSubmit}
          </button>
        </form>
        <ul className="rows">
          {invitations.map((i) => (
            <li key={i.id} className="row" data-testid="invitation">
              <div>
                <span className="row__title">{i.identifier}</span>{' '}
                <span className="tag">{fr.roles[i.role as RoleKey]}</span>{' '}
                <span className="muted">
                  {fr.users.invitationStatus[i.status as keyof typeof fr.users.invitationStatus]}
                </span>
              </div>
              {(i.status === 'pending' || i.status === 'expired') && (
                <div className="actions">
                  <button
                    className="button button--secondary button--small"
                    type="button"
                    onClick={() => {
                      void run(async () => {
                        const created = await confirmed((csrf) => api.renewInvitation(i.id, csrf));
                        showInvitation(created.url);
                      });
                    }}
                  >
                    {fr.users.renew}
                  </button>
                  <button
                    className="button button--secondary button--small"
                    type="button"
                    onClick={() => {
                      void run(async () => {
                        await confirmed((csrf) => api.revokeInvitation(i.id, csrf));
                      });
                    }}
                  >
                    {fr.users.revoke}
                  </button>
                </div>
              )}
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}
