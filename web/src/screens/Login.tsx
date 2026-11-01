import { useState, type SyntheticEvent } from 'react';
import { ApiProblem } from '../api/client';
import { useAuth } from '../auth';
import { messageFor } from '../messages';
import { fr } from '../texts/fr';
import { appEnvironment, appVersion } from '../version';

/** S01, mode connexion : identifiant, mot de passe, puis code de la double authentification si le compte en a une. */
export function Login({ notice }: { notice: string | undefined }) {
  const { login, verifyTotp } = useAuth();
  const [identifier, setIdentifier] = useState('');
  const [password, setPassword] = useState('');
  const [challenge, setChallenge] = useState<string | undefined>(undefined);
  const [code, setCode] = useState('');
  const [backup, setBackup] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>(undefined);

  const noticeText =
    notice === undefined ? undefined : ((fr.errors as Record<string, string | undefined>)[notice] ?? notice);

  const fail = (e: unknown) => {
    setError(messageFor(e instanceof ApiProblem ? e : new ApiProblem(0, 'server.unexpected', '')));
  };

  const submit = async (event: SyntheticEvent) => {
    event.preventDefault();
    if (busy) return;
    setBusy(true);
    setError(undefined);
    try {
      const result = await login(identifier, password);
      if (result.status === 'mfa') setChallenge(result.challenge);
    } catch (e) {
      fail(e);
      setPassword('');
    } finally {
      setBusy(false);
    }
  };

  const submitCode = async (event: SyntheticEvent) => {
    event.preventDefault();
    if (busy || challenge === undefined) return;
    setBusy(true);
    setError(undefined);
    try {
      await verifyTotp(challenge, code);
    } catch (e) {
      fail(e);
      setCode('');
      // le défi vaut 5 minutes : s'il a expiré, on revient à la saisie du mot de passe
      if (e instanceof ApiProblem && e.code === 'auth.session_expired') {
        setChallenge(undefined);
        setPassword('');
      }
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
      {challenge === undefined ? (
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
      ) : (
        <form className="card form" onSubmit={(e) => void submitCode(e)} noValidate>
          <h2>{fr.login.mfaTitle}</h2>
          {error !== undefined && (
            <p role="alert" className="notice notice--error">
              {error}
            </p>
          )}
          <label className="field">
            <span>{backup ? fr.login.mfaBackup : fr.login.mfaCode}</span>
            <input
              name="code"
              type="text"
              inputMode={backup ? 'text' : 'numeric'}
              autoComplete="one-time-code"
              autoCapitalize="none"
              autoCorrect="off"
              spellCheck={false}
              required
              value={code}
              onChange={(e) => {
                setCode(e.target.value);
              }}
            />
          </label>
          <p className="muted">{fr.login.mfaClock}</p>
          <button className="button" type="submit" disabled={busy || code.trim() === ''}>
            {fr.login.mfaSubmit}
          </button>
          <button
            className="button button--secondary"
            type="button"
            onClick={() => {
              setBackup(!backup);
              setCode('');
            }}
          >
            {backup ? fr.login.mfaUseApp : fr.login.mfaUseBackup}
          </button>
        </form>
      )}
      <p className="muted version">
        {fr.app.name} {appVersion} · {appEnvironment}
      </p>
    </main>
  );
}
