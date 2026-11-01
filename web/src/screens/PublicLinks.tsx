import { useEffect, useState, type SyntheticEvent } from 'react';
import { ApiProblem, api, type InvitationPreview } from '../api/client';
import { useAuth } from '../auth';
import { messageFor } from '../messages';
import { fr, type RoleKey } from '../texts/fr';
import { Login } from './Login';

/** Lien reçu dans l'adresse : invitation ou réinitialisation. Le jeton ne reste ni dans l'adresse ni dans l'historique. */
export type PublicLink = { kind: 'invite'; token: string } | { kind: 'reset'; token: string };

export function readPublicLink(path: string): PublicLink | undefined {
  const invite = /^\/invitation\/([A-Za-z0-9_-]{43})\/?$/.exec(path);
  if (invite?.[1] !== undefined) return { kind: 'invite', token: invite[1] };
  const reset = /^\/reinitialisation\/([A-Za-z0-9_-]{43})\/?$/.exec(path);
  if (reset?.[1] !== undefined) return { kind: 'reset', token: reset[1] };
  return undefined;
}

function problemText(e: unknown): string {
  return messageFor(e instanceof ApiProblem ? e : new ApiProblem(0, 'server.unexpected', ''));
}

function Page({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <main className="login">
      <header className="login__header">
        <h1>{fr.app.name}</h1>
      </header>
      <section className="card form">
        <h2>{title}</h2>
        {children}
      </section>
    </main>
  );
}

/** S01, mode 2 : accepter une invitation (nouveau compte), ou rejoindre avec un compte existant déjà connecté. */
export function InvitePage({ token, onDone }: { token: string; onDone: () => void }) {
  const { state, csrf, reload } = useAuth();
  const [preview, setPreview] = useState<InvitationPreview | 'gone' | 'loading'>('loading');
  const [name, setName] = useState('');
  const [password, setPassword] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>(undefined);
  const [joined, setJoined] = useState(false);

  useEffect(() => {
    let live = true;
    api.invitation(token).then(
      (p) => {
        if (live) setPreview(p);
      },
      () => {
        if (live) setPreview('gone');
      },
    );
    return () => {
      live = false;
    };
  }, [token]);

  if (preview === 'loading') {
    return (
      <Page title={fr.invitePage.title}>
        <p role="status" className="muted">
          {fr.common.loading}
        </p>
      </Page>
    );
  }
  if (preview === 'gone') {
    return (
      <Page title={fr.invitePage.title}>
        <p role="alert" className="notice notice--error">
          {fr.invitePage.expired}
        </p>
        <button className="button" type="button" onClick={onDone}>
          {fr.invitePage.toLogin}
        </button>
      </Page>
    );
  }
  const roleText = fr.roles[preview.role as RoleKey];
  const signedIn = state.status === 'ready';

  // compte existant : connexion préalable (le lien reste en mémoire pendant la connexion), puis adhésion d'un clic
  if (preview.identifier_exists && !signedIn) {
    return <Login notice={fr.invitePage.existing(preview.organization_name)} />;
  }

  const submit = async (event: SyntheticEvent) => {
    event.preventDefault();
    if (busy) return;
    setBusy(true);
    setError(undefined);
    try {
      if (preview.identifier_exists) {
        await api.acceptInvitation(token, undefined, undefined, csrf());
        setJoined(true);
      } else {
        await api.acceptInvitation(token, name, password);
        await reload();
        onDone();
      }
      await reload();
    } catch (e) {
      setError(problemText(e));
      setPassword('');
    } finally {
      setBusy(false);
    }
  };

  if (joined) {
    return (
      <Page title={fr.invitePage.title}>
        <p role="status" className="notice notice--info">
          {fr.invitePage.joined}
        </p>
        <button className="button" type="button" onClick={onDone}>
          {fr.denied.back}
        </button>
      </Page>
    );
  }
  return (
    <main className="login">
      <header className="login__header">
        <h1>{fr.app.name}</h1>
      </header>
      <form className="card form" onSubmit={(e) => void submit(e)} noValidate>
        <h2>{fr.invitePage.title}</h2>
        <p>{fr.invitePage.intro(preview.organization_name, roleText)}</p>
        {error !== undefined && (
          <p role="alert" className="notice notice--error">
            {error}
          </p>
        )}
        {!preview.identifier_exists && (
          <>
            <label className="field">
              <span>{fr.invitePage.name}</span>
              <input
                name="display_name"
                type="text"
                autoComplete="name"
                required
                value={name}
                onChange={(e) => {
                  setName(e.target.value);
                }}
              />
            </label>
            <label className="field">
              <span>{fr.invitePage.password}</span>
              <input
                name="new-password"
                type="password"
                autoComplete="new-password"
                required
                value={password}
                onChange={(e) => {
                  setPassword(e.target.value);
                }}
              />
              <small className="muted">{fr.invitePage.passwordHint}</small>
            </label>
          </>
        )}
        <button
          className="button"
          type="submit"
          disabled={busy || (!preview.identifier_exists && (name.trim() === '' || password === ''))}
        >
          {fr.invitePage.submit}
        </button>
      </form>
    </main>
  );
}

/** S01, mode 4 : choisir un nouveau mot de passe avec un lien de réinitialisation. Aucune session n'est ouverte par le lien. */
export function ResetPage({ token, onDone }: { token: string; onDone: () => void }) {
  const [password, setPassword] = useState('');
  const [again, setAgain] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>(undefined);
  const [done, setDone] = useState(false);

  const submit = async (event: SyntheticEvent) => {
    event.preventDefault();
    if (busy) return;
    if (password !== again) {
      setError(fr.resetPage.mismatch);
      return;
    }
    setBusy(true);
    setError(undefined);
    try {
      await api.completeReset(token, password);
      setDone(true);
    } catch (e) {
      setError(problemText(e));
    } finally {
      setBusy(false);
    }
  };

  if (done) {
    return (
      <Page title={fr.resetPage.title}>
        <p role="status" className="notice notice--info">
          {fr.resetPage.done}
        </p>
        <button className="button" type="button" onClick={onDone}>
          {fr.invitePage.toLogin}
        </button>
      </Page>
    );
  }
  return (
    <main className="login">
      <header className="login__header">
        <h1>{fr.app.name}</h1>
      </header>
      <form className="card form" onSubmit={(e) => void submit(e)} noValidate>
        <h2>{fr.resetPage.title}</h2>
        {error !== undefined && (
          <p role="alert" className="notice notice--error">
            {error}
          </p>
        )}
        <label className="field">
          <span>{fr.resetPage.password}</span>
          <input
            name="new-password"
            type="password"
            autoComplete="new-password"
            required
            value={password}
            onChange={(e) => {
              setPassword(e.target.value);
            }}
          />
          <small className="muted">{fr.invitePage.passwordHint}</small>
        </label>
        <label className="field">
          <span>{fr.resetPage.confirm}</span>
          <input
            name="confirm-password"
            type="password"
            autoComplete="new-password"
            required
            value={again}
            onChange={(e) => {
              setAgain(e.target.value);
            }}
          />
        </label>
        <button className="button" type="submit" disabled={busy || password === '' || again === ''}>
          {fr.resetPage.submit}
        </button>
      </form>
    </main>
  );
}
