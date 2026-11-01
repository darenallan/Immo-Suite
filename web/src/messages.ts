import { ApiProblem } from './api/client';
import { fr } from './texts/fr';

/** « 30 secondes », « 1 minute » : durée arrondie par excès, comme le serveur. */
export function formatDuration(seconds: number): string {
  if (seconds < 60) return fr.duration.seconds(Math.max(1, Math.round(seconds)));
  return fr.duration.minutes(Math.ceil(seconds / 60));
}

/** Texte affiché pour une erreur du contrat. Le serveur n'envoie que des codes : le texte est ici (décision n° 66). */
export function messageFor(problem: ApiProblem): string {
  if (problem.code === 'auth.rate_limited') {
    return fr.rateLimited(formatDuration(problem.retryAfterSeconds ?? 60));
  }
  if (problem.code === 'rate_limit.exceeded') {
    return fr.requestRateLimited(formatDuration(problem.retryAfterSeconds ?? 60));
  }
  if (problem.code === 'auth.password_weak' && problem.reason !== undefined) {
    const reason = fr.passwordWeak[problem.reason];
    if (reason !== undefined) return reason;
  }
  if (problem.code === 'validation.failed' && problem.fields.length > 0) {
    return problem.fields.map((f) => fr.fieldErrors[f.code] ?? fr.errors['validation.failed']).join(' ');
  }
  const known = (fr.errors as Record<string, string | undefined>)[problem.code];
  if (known !== undefined) return known;
  return problem.requestId !== undefined
    ? fr.errorWithIncident(problem.requestId)
    : fr.errors['server.unexpected'];
}
