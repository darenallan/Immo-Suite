import { useState } from 'react';
import { ApiProblem, type Me } from '../api/client';
import { useAuth } from '../auth';
import { messageFor } from '../messages';
import { fr, type RoleKey } from '../texts/fr';

/** Utilisateur de plusieurs organisations : le choix précède tout le reste (F01 §3). */
export function ChooseOrganization({ me }: { me: Me }) {
  const { selectOrganization, logout } = useAuth();
  const [busy, setBusy] = useState<string | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);

  const choose = async (id: string) => {
    setBusy(id);
    setError(undefined);
    try {
      await selectOrganization(id);
    } catch (e) {
      setError(messageFor(e instanceof ApiProblem ? e : new ApiProblem(0, 'server.unexpected', '')));
      setBusy(undefined);
    }
  };

  return (
    <main className="login">
      <header className="login__header">
        <h1>{fr.app.name}</h1>
        <p className="muted">
          {fr.home.welcome} {me.user.display_name}
        </p>
      </header>
      <section className="card" aria-labelledby="choose-title">
        <h2 id="choose-title">{fr.chooser.title}</h2>
        <p>{fr.chooser.intro}</p>
        {error !== undefined && (
          <p role="alert" className="notice notice--error">
            {error}
          </p>
        )}
        <ul className="choices">
          {me.organizations.map((o) => (
            <li key={o.id}>
              <button
                className="choice"
                type="button"
                disabled={busy !== undefined}
                onClick={() => {
                  void choose(o.id);
                }}
              >
                <strong>{o.name}</strong>
                <span className="muted">
                  {fr.chooser.roleLabel} : {fr.roles[o.role as RoleKey]}
                </span>
              </button>
            </li>
          ))}
        </ul>
        <button
          className="button button--secondary"
          type="button"
          onClick={() => {
            void logout();
          }}
        >
          {fr.menu.logout}
        </button>
      </section>
    </main>
  );
}
