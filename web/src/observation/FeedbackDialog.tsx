import { useEffect, useRef, useState, type SyntheticEvent } from 'react';
import { ApiProblem, api } from '../api/client';
import { messageFor } from '../messages';
import { fr } from '../texts/fr';
import { appVersion } from '../version';
import { diagnostics } from './diagnostics';

type Category = 'bug' | 'request' | 'question';
const CATEGORIES: readonly Category[] = ['bug', 'request', 'question'];
export const MIN_CHARS = 5;
export const MAX_CHARS = 4000;

/** Nombre de caractères (points de code) : mêmes bornes que le serveur, qui compte des caractères et non des octets. */
function charCount(text: string): number {
  return Array.from(text).length;
}

/** Erreur de description, ou `undefined` si elle est correcte (mêmes bornes que le serveur : 5 à 4 000 caractères). */
export function descriptionError(text: string): string | undefined {
  const length = charCount(text.trim());
  if (length < MIN_CHARS) return fr.feedback.errorTooShort;
  if (length > MAX_CHARS) return fr.feedback.errorTooLong;
  return undefined;
}

/**
 * Formulaire « Signaler un problème » (F39-observation §7) : catégorie, description, « vous pouvez me rappeler ». Le contexte
 * technique (écran, adresse sans paramètre, version, taille d'écran, cinq derniers identifiants de requête) est joint sans saisie.
 * Une clé d'idempotence par ouverture : un double clic ne crée qu'un signalement.
 */
export function FeedbackDialog({
  csrf,
  onClose,
  onSent,
}: {
  csrf: () => string;
  onClose: () => void;
  onSent: (reference: string) => void;
}) {
  const [category, setCategory] = useState<Category>('bug');
  const [description, setDescription] = useState('');
  const [callback, setCallback] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>(undefined);
  const [fieldError, setFieldError] = useState<string | undefined>(undefined);
  const key = useRef(crypto.randomUUID());
  const area = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    area.current?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
    };
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('keydown', onKey);
    };
  }, [onClose]);

  const submit = async (event: SyntheticEvent) => {
    event.preventDefault();
    if (busy) return;
    const problem = descriptionError(description);
    setFieldError(problem);
    if (problem !== undefined) {
      area.current?.focus();
      return;
    }
    setBusy(true);
    setError(undefined);
    try {
      const created = await api.sendFeedback(
        {
          category,
          description,
          can_call_back: callback,
          context: {
            screen: diagnostics.screen,
            path: window.location.pathname,
            client_version: appVersion,
            viewport: `${String(window.innerWidth)}x${String(window.innerHeight)}`,
            last_request_ids: diagnostics.requestIds,
            last_error_code: diagnostics.lastErrorCode ?? null,
          },
        },
        csrf(),
        key.current,
      );
      onSent(created.reference);
    } catch (e) {
      setError(messageFor(e instanceof ApiProblem ? e : new ApiProblem(0, 'server.unexpected', '')));
      setBusy(false);
    }
  };

  return (
    <div className="dialog__backdrop">
      <form
        className="card form dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="feedback-title"
        onSubmit={(e) => void submit(e)}
        noValidate
      >
        <h2 id="feedback-title">{fr.feedback.title}</h2>
        <p className="muted">{fr.feedback.intro}</p>
        {fieldError !== undefined && (
          <div role="alert" className="notice notice--error">
            <p>{fr.feedback.summary}</p>
            <ul>
              <li>
                <a href="#feedback-description">{fieldError}</a>
              </li>
            </ul>
          </div>
        )}
        {error !== undefined && (
          <p role="alert" className="notice notice--error">
            {error}
          </p>
        )}
        <fieldset className="field">
          <legend>{fr.feedback.category}</legend>
          {CATEGORIES.map((c) => (
            <label key={c} className="choice">
              <input
                type="radio"
                name="feedback-category"
                value={c}
                checked={category === c}
                onChange={() => {
                  setCategory(c);
                }}
              />
              <span>{fr.feedback.categories[c]}</span>
            </label>
          ))}
        </fieldset>
        <div className="field">
          <label htmlFor="feedback-description">{fr.feedback.description}</label>
          <textarea
            ref={area}
            id="feedback-description"
            name="feedback-description"
            rows={6}
            value={description}
            aria-invalid={fieldError !== undefined}
            aria-describedby="feedback-hint"
            onChange={(e) => {
              setDescription(e.target.value);
              if (fieldError !== undefined) setFieldError(undefined);
            }}
          />
          <span id="feedback-hint" className="muted">
            {fr.feedback.descriptionHint} {fr.feedback.counter(charCount(description))}
          </span>
        </div>
        <label className="choice">
          <input
            type="checkbox"
            name="feedback-callback"
            checked={callback}
            onChange={(e) => {
              setCallback(e.target.checked);
            }}
          />
          <span>{fr.feedback.callback}</span>
        </label>
        <div className="actions">
          <button className="button" type="submit" disabled={busy}>
            {busy ? fr.feedback.sending : fr.feedback.submit}
          </button>
          <button className="button button--secondary" type="button" onClick={onClose}>
            {fr.feedback.cancel}
          </button>
        </div>
      </form>
    </div>
  );
}
