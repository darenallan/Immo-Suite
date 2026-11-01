import { useEffect, useRef, useState, type SyntheticEvent } from 'react';
import { ApiProblem } from '../api/client';
import { messageFor } from '../messages';
import { fr } from '../texts/fr';

/**
 * Modale « Confirmez votre mot de passe » (F01 §9.6). Le mot de passe n'est conservé ni dans le stockage du navigateur ni après
 * l'envoi ; la confirmation vaut 5 minutes côté serveur.
 */
export function ConfirmDialog({
  onConfirm,
  onCancel,
}: {
  onConfirm: (password: string) => Promise<void>;
  onCancel: () => void;
}) {
  const [password, setPassword] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>(undefined);
  const input = useRef<HTMLInputElement>(null);

  useEffect(() => {
    input.current?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onCancel();
    };
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('keydown', onKey);
    };
  }, [onCancel]);

  const submit = async (event: SyntheticEvent) => {
    event.preventDefault();
    if (busy || password === '') return;
    setBusy(true);
    setError(undefined);
    try {
      await onConfirm(password);
    } catch (e) {
      setError(messageFor(e instanceof ApiProblem ? e : new ApiProblem(0, 'server.unexpected', '')));
      setPassword('');
      setBusy(false);
    }
  };

  return (
    <div className="dialog__backdrop">
      <form
        className="card form dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="confirm-title"
        onSubmit={(e) => void submit(e)}
        noValidate
      >
        <h2 id="confirm-title">{fr.confirm.title}</h2>
        <p className="muted">{fr.confirm.body}</p>
        {error !== undefined && (
          <p role="alert" className="notice notice--error">
            {error}
          </p>
        )}
        <label className="field">
          <span>{fr.confirm.password}</span>
          <input
            ref={input}
            name="confirm-password"
            type="password"
            autoComplete="current-password"
            value={password}
            onChange={(e) => {
              setPassword(e.target.value);
            }}
          />
        </label>
        <div className="actions">
          <button className="button" type="submit" disabled={busy || password === ''}>
            {fr.confirm.submit}
          </button>
          <button className="button button--secondary" type="button" onClick={onCancel}>
            {fr.confirm.cancel}
          </button>
        </div>
      </form>
    </div>
  );
}
