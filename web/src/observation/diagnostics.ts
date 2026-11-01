// Contexte technique minimal d'un signalement (F39-observation règle 16) : écran courant, cinq derniers identifiants de requête,
// dernier code d'erreur. Gardé en mémoire seulement ; aucune donnée saisie, aucun jeton.

const MAX_IDS = 5;

export const diagnostics: {
  screen: string;
  requestIds: string[];
  lastErrorCode: string | undefined;
} = { screen: 'S00', requestIds: [], lastErrorCode: undefined };

/** Retient l'identifiant de requête (`X-Request-Id`, `INC-…`) d'une réponse. */
export function noteRequest(id: string | null): void {
  if (id === null || !/^INC-[0-9A-Z]{6}$/.test(id)) return;
  diagnostics.requestIds = [...diagnostics.requestIds, id].slice(-MAX_IDS);
}

/** Retient le code de la dernière erreur reçue (`auth.forbidden`…). */
export function noteError(code: string): void {
  if (/^[a-z][a-z0-9_]*(\.[a-z0-9_]+)+$/.test(code)) diagnostics.lastErrorCode = code;
}

export function resetDiagnostics(): void {
  diagnostics.screen = 'S00';
  diagnostics.requestIds = [];
  diagnostics.lastErrorCode = undefined;
}
