import { useState, type SyntheticEvent } from 'react';
import { ApiProblem } from '../api/client';
import { useAuth } from '../auth';
import { messageFor } from '../messages';
import { fr } from '../texts/fr';
import { appEnvironment, appVersion } from '../version';

/** S01, mode connexion : identifiant, mot de passe, états chargement et erreur (UI_UX_DESIGN.md). */
export function Login({ notice }: { notice: string | undefined }) {
  const { login } = useAuth();
  const [identifier, setIdentifier] = useState('');
  const [password, setPassword] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>(undefined);

  const noticeText =
    notice === undefined ? undefined : (fr.errors as Record<string, string | undefined>)[notice];

  const submit = async (event: SyntheticEvent) => {
    event.preventDefault();
    if (busy) return;
    setBusy(true);
    setError(undefined);
    try {
      await login(identifier, password);
    } catch (e) {
      setError(messageFor(e instanceof ApiProblem ? e : new ApiProblem(0, 'server.unexpected', '')));
      setPassword('');
    } finally {
      setBusy(false);
    }
  };

  return (
    <main className="login">
      <header className="login__header">
        <h1>{fr.app.name}</h1>
        <p className="muted">{fr.app.tagline}</p>
      </header>
      <form className="card form" onSubmit={(e) => void submit(e)} noValidate>
        <h2>{fr.login.title}</h2>
        {noticeText !== undefined && error === undefined && (
          <p role="status" className="notice notice--info">
            {noticeText}
          </p>
        )}
        {error !== undefined && (
          <p role="alert" className="notice notice--error">
            {error}
          </p>
        )}
        <label className="field">
          <span>{fr.login.identifier}</span>
          <input
            name="identifier"
            type="text"
            inputMode="email"
            autoComplete="username"
            autoCapitalize="none"
            autoCorrect="off"
            spellCheck={false}
            required
            placeholder={fr.login.identifierHint}
            value={identifier}
            onChange={(e) => {
              setIdentifier(e.target.value);
            }}
          />
        </label>
        <label className="field">
          <span>{fr.login.password}</span>
          <input
            name="password"
            type="password"
            autoComplete="current-password"
            required
            value={password}
            onChange={(e) => {
              setPassword(e.target.value);
            }}
          />
        </label>
        <button
          className="button"
          type="submit"
          disabled={busy || identifier.trim() === '' || password === ''}
        >
          {busy ? fr.login.submitting : fr.login.submit}
        </button>
        <details className="forgot">
          <summary>{fr.login.forgot}</summary>
          <p>{fr.login.forgotHelp}</p>
        </details>
      </form>
      <p className="muted version">
        {fr.app.name} {appVersion} · {appEnvironment}
      </p>
    </main>
  );
}
