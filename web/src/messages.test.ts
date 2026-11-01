import { describe, expect, it } from 'vitest';
import { ApiProblem } from './api/client';
import { formatDuration, messageFor } from './messages';

describe('messages d’erreur', () => {
  it('durées arrondies par excès, comme le serveur', () => {
    expect(formatDuration(1)).toBe('1 seconde');
    expect(formatDuration(30)).toBe('30 secondes');
    expect(formatDuration(60)).toBe('1 minute');
    expect(formatDuration(61)).toBe('2 minutes');
    expect(formatDuration(900)).toBe('15 minutes');
  });

  it('limitation : le texte du contrat avec la durée', () => {
    const p = new ApiProblem(429, 'auth.rate_limited', '', undefined, 300);
    expect(messageFor(p)).toBe(
      'Trop de tentatives. Réessayez dans 5 minutes ou réinitialisez votre mot de passe.',
    );
  });

  it('codes connus : texte du catalogue', () => {
    expect(messageFor(new ApiProblem(401, 'auth.invalid_credentials', ''))).toBe(
      'Identifiant ou mot de passe incorrect.',
    );
    expect(messageFor(new ApiProblem(403, 'org.suspended', ''))).toBe(
      'Cette organisation est suspendue. Contactez le support.',
    );
  });

  it('code inconnu : message générique avec le code d’incident', () => {
    expect(messageFor(new ApiProblem(500, 'quelque.chose', '', 'INC-7K3F9A'))).toBe(
      'Un problème est survenu. Réessayez. Si le problème continue, communiquez le code INC-7K3F9A au support.',
    );
    expect(messageFor(new ApiProblem(500, 'quelque.chose', ''))).toBe('Un problème est survenu. Réessayez.');
  });
});
