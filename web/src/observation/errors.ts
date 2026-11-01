// Erreurs JavaScript non traitées vers `POST /api/client-errors` (F39-observation §7). Le serveur nettoie le message (numéros, adresses,
// jetons) ; le front n'envoie ni pile, ni adresse, ni texte de l'écran. Au plus une erreur identique par minute, cinq par session.
import { appVersion } from '../version';
import { ApiProblem } from '../api/client';
import { diagnostics } from './diagnostics';

const PER_SESSION = 5;
const SAME_ERROR_MS = 60_000;

let sent = 0;
const lastSeen = new Map<string, number>();

export function resetErrorCapture(): void {
  sent = 0;
  lastSeen.clear();
}

/** Message de l'erreur : le nom du type en tête (le serveur en tire le code), jamais la pile. */
export function describe(reason: unknown, rejection: boolean): string {
  const base =
    reason instanceof Error ? `${reason.name}: ${reason.message}` : `Error: ${String(reason).slice(0, 200)}`;
  return (rejection ? 'unhandled_rejection: ' : '') + base;
}

export function reportError(message: string, csrf: () => string | undefined): void {
  const token = csrf();
  if (token === undefined || token === '' || sent >= PER_SESSION) return;
  const key = message.slice(0, 200);
  const now = Date.now();
  const previous = lastSeen.get(key);
  if (previous !== undefined && now - previous < SAME_ERROR_MS) return;
  lastSeen.set(key, now);
  sent += 1;
  void fetch('/api/client-errors', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'X-CSRF-Token': token, 'X-Client-Version': appVersion },
    body: JSON.stringify({
      message: key,
      screen: diagnostics.screen,
      client_version: appVersion,
      request_ids: diagnostics.requestIds,
    }),
    credentials: 'same-origin',
    keepalive: true,
  }).catch(() => {
    // l'envoi d'une erreur ne doit jamais en provoquer une autre
  });
}

/** Installe `window.onerror` et le rejet non traité. Retourne la fonction qui les retire. */
export function installErrorCapture(csrf: () => string | undefined): () => void {
  const onError = (event: ErrorEvent) => {
    reportError(describe(event.error ?? event.message, false), csrf);
  };
  const onRejection = (event: PromiseRejectionEvent) => {
    // une réponse d'erreur du contrat est déjà traitée par l'écran qui l'a demandée
    if (event.reason instanceof ApiProblem) return;
    reportError(describe(event.reason, true), csrf);
  };
  window.addEventListener('error', onError);
  window.addEventListener('unhandledrejection', onRejection);
  return () => {
    window.removeEventListener('error', onError);
    window.removeEventListener('unhandledrejection', onRejection);
  };
}
