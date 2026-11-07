// Mesure d'usage sans service tiers (F39-observation §7, décision n° 28) : petits lots vers `POST /api/events`, sur le même domaine,
// sans cookie supplémentaire ni identifiant publicitaire. La file reste en mémoire (jamais écrite sur le téléphone) ; l'envoi
// n'attend rien et n'est jamais rejoué : une mesure perdue vaut mieux qu'un écran ralenti.
import { appVersion } from '../version';
import { diagnostics } from './diagnostics';

export type WidthClass = 'mobile' | 'tablet' | 'desktop';

/** Les seuls événements que le front a le droit d'envoyer à l'étape 04 (liste blanche : `tests/contract/events-whitelist.json`). */
export type ClientEvent =
  | { name: 'screen_viewed'; properties: { screen: string; width_class: WidthClass } }
  | { name: 'validation_error'; properties: { screen: string; field: string; rule_code: string } }
  | {
      name: 'search_performed';
      properties: { length: number; group: 'tenants' | 'units' | 'receipts' | 'none'; empty: 'yes' | 'no' };
    }
  // étape 07 (F09 règle 21) : ouverture et abandon du formulaire d'encaissement, sans aucune donnée saisie
  | { name: 'payment_form_opened'; properties: { screen: 'S04' } }
  | { name: 'payment_form_abandoned'; properties: { screen: 'S04' } };

/** Taille maximale de la file en mémoire : au-delà, les plus anciens événements sont jetés. */
export const MAX_QUEUE = 100;
/** Un lot contient 20 événements au plus. */
export const BATCH_SIZE = 20;
/** Délai avant l'envoi d'un lot incomplet. */
export const FLUSH_MS = 10_000;

/** Écran de la liste de `UI_UX_DESIGN.md` pour une adresse ; `S00` pour une adresse hors liste. */
const SCREENS: Record<string, string> = {
  '/': 'S03',
  '/encaisser': 'S04',
  '/impayes': 'S05',
  '/biens': 'S06',
  '/locataires': 'S08',
  '/baux': 'S10',
  '/paiements': 'S11',
  '/comptes-rendus': 'S12',
  '/mandants': 'S13',
  '/travaux': 'S14',
  '/agenda': 'S15',
  '/import': 'S16',
  '/parametres': 'S17',
  '/utilisateurs': 'S18',
  '/cloture-caisse': 'S19',
  '/notifications': 'S20',
  '/journal-audit': 'S21',
};

export function screenOf(path: string): string {
  // fiches (étape 05) : l'identifiant n'entre jamais dans l'événement, seul le code de l'écran
  if (path.startsWith('/biens/')) return 'S06';
  if (path.startsWith('/lots/')) return 'S07';
  if (path.startsWith('/locataires/')) return 'S09';
  if (path.startsWith('/baux/')) return 'S10';
  if (path.startsWith('/paiements/')) return 'S11';
  return SCREENS[path] ?? 'S00';
}

/** Recherche unique (F04 §4.1) : la longueur tapée et le groupe ouvert, jamais le texte. */
export function trackSearch(
  length: number,
  group: 'tenants' | 'units' | 'receipts' | 'none',
  empty: boolean,
): void {
  if (length < 2 || length > 80) return;
  track({ name: 'search_performed', properties: { length, group, empty: empty ? 'yes' : 'no' } });
}

export function widthClass(width: number): WidthClass {
  if (width < 768) return 'mobile';
  if (width < 1024) return 'tablet';
  return 'desktop';
}

let queue: ClientEvent[] = [];
let timer: number | undefined;
let csrf: () => string | undefined = () => undefined;
let blockedUntil = 0;
let lastScreen: { screen: string; at: number } | undefined;

/** Branche la lecture du jeton anti-falsification (en mémoire). Sans jeton, rien n'est envoyé : aucun événement avant la connexion. */
export function configureObservation(getCsrf: () => string | undefined): void {
  csrf = getCsrf;
}

/** Remet à zéro (tests et déconnexion) : la file n'est jamais conservée d'une session à l'autre. */
export function resetObservation(): void {
  queue = [];
  blockedUntil = 0;
  lastScreen = undefined;
  if (timer !== undefined) window.clearTimeout(timer);
  timer = undefined;
}

export function queuedCount(): number {
  return queue.length;
}

/** Envoie un lot de 20 au plus. Ne bloque jamais l'appelant, ne lève jamais d'erreur, ne rejoue rien. */
export function flush(): void {
  if (timer !== undefined) window.clearTimeout(timer);
  timer = undefined;
  const token = csrf();
  if (queue.length === 0) return;
  if (token === undefined || token === '' || Date.now() < blockedUntil) {
    queue = [];
    return;
  }
  const events = queue.slice(0, BATCH_SIZE).map((e) => ({ ...e, client_version: appVersion }));
  queue = queue.slice(BATCH_SIZE);
  void fetch('/api/events', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'X-CSRF-Token': token, 'X-Client-Version': appVersion },
    body: JSON.stringify({ events }),
    credentials: 'same-origin',
    keepalive: true,
  })
    .then((response) => {
      if (response.status === 429) {
        // limite atteinte : on espace les envois, sans bandeau (la mesure n'est pas l'affaire de l'utilisateur)
        const seconds = Number(response.headers.get('Retry-After') ?? '60');
        blockedUntil = Date.now() + (Number.isFinite(seconds) ? seconds : 60) * 1000;
      }
    })
    .catch(() => {
      // hors connexion ou serveur injoignable : l'événement est perdu, rien n'est rejoué
    });
  if (queue.length > 0) schedule();
}

function schedule(): void {
  if (timer === undefined) timer = window.setTimeout(flush, FLUSH_MS);
}

/** Enregistre un événement. Retour immédiat : l'envoi se fait en différé, par lots. */
export function track(event: ClientEvent): void {
  if (csrf() === undefined) return;
  queue.push(event);
  if (queue.length > MAX_QUEUE) queue = queue.slice(queue.length - MAX_QUEUE);
  if (queue.length >= BATCH_SIZE) flush();
  else schedule();
}

/** Écran affiché : mémorisé pour les signalements, et compté (le même écran répété dans la seconde est ignoré). */
export function trackScreen(path: string, width: number): void {
  const screen = screenOf(path);
  diagnostics.screen = screen;
  const now = Date.now();
  if (lastScreen?.screen === screen && now - lastScreen.at < 1000) return;
  lastScreen = { screen, at: now };
  track({ name: 'screen_viewed', properties: { screen, width_class: widthClass(width) } });
}

/** Erreur de saisie affichée : champ et code de règle seulement, jamais la valeur saisie. */
export function trackValidation(field: string, code: string): void {
  const clean = field
    .toLowerCase()
    .replace(/[^a-z0-9_]/g, '_')
    .slice(0, 41);
  if (!/^[a-z][a-z0-9_]{1,40}$/.test(clean) || !/^[a-z][a-z0-9_]*(\.[a-z0-9_]+)+$/.test(code)) return;
  track({
    name: 'validation_error',
    properties: { screen: diagnostics.screen, field: clean, rule_code: code },
  });
}

/** À la mise en arrière-plan de la page, on envoie ce qui reste. */
export function installVisibilityFlush(): () => void {
  const onHide = () => {
    if (document.visibilityState === 'hidden') flush();
  };
  document.addEventListener('visibilitychange', onHide);
  return () => {
    document.removeEventListener('visibilitychange', onHide);
  };
}
