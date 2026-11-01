// État d'authentification : la session est un cookie `HttpOnly` (invisible du script) ; le jeton anti-falsification vit en mémoire.
// Aucun jeton, aucune donnée personnelle dans le stockage du navigateur.
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from 'react';
import { ApiProblem, api, type Me } from './api/client';
import { installErrorCapture } from './observation/errors';
import { configureObservation, installVisibilityFlush, resetObservation } from './observation/track';
import { ConfirmDialog } from './screens/ConfirmDialog';

export type AuthState =
  { status: 'loading' } | { status: 'anonymous'; notice: string | undefined } | { status: 'ready'; me: Me };

/** Résultat d'une connexion : ouverte, ou second facteur à saisir (le défi vaut 5 minutes). */
export type LoginResult = { status: 'ok' } | { status: 'mfa'; challenge: string };

/** La confirmation par mot de passe a été abandonnée par la personne (modale fermée). */
export class ConfirmCancelled extends Error {
  constructor() {
    super('confirmation annulée');
    this.name = 'ConfirmCancelled';
  }
}

interface AuthApi {
  state: AuthState;
  updateAvailable: boolean;
  login: (identifier: string, password: string) => Promise<LoginResult>;
  verifyTotp: (challenge: string, code: string) => Promise<void>;
  selectOrganization: (id: string) => Promise<void>;
  logout: () => Promise<void>;
  logoutAll: () => Promise<void>;
  /** Recharge le profil (droits, réglages) après une modification. */
  reload: () => Promise<void>;
  /** Jeton anti-falsification courant, pour les appels d'écriture. */
  csrf: () => string;
  /**
   * Exécute une action à enjeu. Si le serveur répond uth.reauth_required, ouvre la modale « Confirmez votre mot de passe »,
   * confirme, puis rejoue l'action une fois (F01 §9.6 point 2).
   */
  confirmed: <T>(action: (csrf: string) => Promise<T>) => Promise<T>;
  /** Change son mot de passe ; la session est renouvelée par le serveur. */
  changePassword: (current: string, next: string) => Promise<void>;
}

const Context = createContext<AuthApi | undefined>(undefined);

/** Intervalle de contrôle de la session pendant que l'application est ouverte (expiration par inactivité côté serveur). */
const REFRESH_MS = 60_000;

export function AuthProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<AuthState>({ status: 'loading' });
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const csrf = useRef<string | undefined>(undefined);
  const [asking, setAsking] = useState<{ resolve: () => void; reject: () => void } | undefined>(undefined);

  const apply = useCallback((me: Me) => {
    csrf.current = me.csrf_token;
    setState({ status: 'ready', me });
  }, []);

  const expire = useCallback((notice: string | undefined) => {
    csrf.current = undefined;
    setState({ status: 'anonymous', notice });
  }, []);

  /** Échec d'un contrôle du profil. 401 : retour à la connexion, avec le message d'expiration si une session existait. */
  const failed = useCallback(
    (e: unknown, hadSession: boolean) => {
      if (e instanceof ApiProblem && e.code === 'client.upgrade_required') setUpdateAvailable(true);
      if (e instanceof ApiProblem && e.status === 401) expire(hadSession ? e.code : undefined);
      else if (!hadSession) expire(undefined);
    },
    [expire],
  );

  /** Contrôle périodique du profil pendant une session. */
  const refresh = useCallback(() => {
    api.me().then(apply, (e: unknown) => {
      failed(e, true);
    });
  }, [apply, failed]);

  // Chargement initial : la réponse arrive après le rendu (le setState est dans un rappel, pas dans le corps de l'effet).
  useEffect(() => {
    let live = true;
    api.me().then(
      (me) => {
        if (live) apply(me);
      },
      (e: unknown) => {
        if (live) failed(e, false);
      },
    );
    return () => {
      live = false;
    };
  }, [apply, failed]);

  // mesure d'usage et capture des erreurs (F39-observation) : tant qu'il n'y a pas de session, rien n'est envoyé
  useEffect(() => {
    const token = () => csrf.current;
    configureObservation(token);
    const stopErrors = installErrorCapture(token);
    const stopFlush = installVisibilityFlush();
    return () => {
      stopErrors();
      stopFlush();
      resetObservation();
    };
  }, []);

  const ready = state.status === 'ready';
  useEffect(() => {
    if (!ready) return;
    const check = refresh;
    const onVisible = () => {
      if (document.visibilityState === 'visible') check();
    };
    const timer = window.setInterval(check, REFRESH_MS);
    window.addEventListener('focus', check);
    document.addEventListener('visibilitychange', onVisible);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener('focus', check);
      document.removeEventListener('visibilitychange', onVisible);
    };
  }, [ready, refresh]);

  const value = useMemo<AuthApi>(
    () => ({
      state,
      updateAvailable,
      login: async (identifier, password) => {
        let body;
        try {
          body = await api.login(identifier, password);
        } catch (e) {
          if (e instanceof ApiProblem && e.code === 'client.upgrade_required') setUpdateAvailable(true);
          throw e;
        }
        if (body.mfa_required && body.challenge !== null && body.challenge !== undefined) {
          return { status: 'mfa', challenge: body.challenge };
        }
        apply(await api.me());
        return { status: 'ok' };
      },
      verifyTotp: async (challenge, code) => {
        await api.totpVerify(challenge, code);
        apply(await api.me());
      },
      reload: async () => {
        apply(await api.me());
      },
      csrf: () => csrf.current ?? '',
      confirmed: async <T,>(action: (token: string) => Promise<T>): Promise<T> => {
        try {
          return await action(csrf.current ?? '');
        } catch (e) {
          if (!(e instanceof ApiProblem && e.code === 'auth.reauth_required')) throw e;
        }
        await new Promise<void>((resolve, reject) => {
          setAsking({
            resolve,
            reject: () => {
              reject(new ConfirmCancelled());
            },
          });
        });
        return action(csrf.current ?? '');
      },
      changePassword: async (current, next) => {
        await api.changePassword(current, next, csrf.current ?? '');
        apply(await api.me());
      },
      selectOrganization: async (id) => {
        apply(await api.selectOrganization(id, csrf.current ?? ''));
      },
      logout: async () => {
        try {
          await api.logout(csrf.current);
        } catch {
          // la session est de toute façon perdue côté navigateur : on revient à la connexion
        }
        expire(undefined);
      },
      logoutAll: async () => {
        try {
          await api.logoutAll(csrf.current ?? '');
        } catch {
          // idem
        }
        expire(undefined);
      },
    }),
    [state, updateAvailable, apply, expire],
  );

  return (
    <Context.Provider value={value}>
      {children}
      {asking !== undefined && (
        <ConfirmDialog
          onConfirm={async (password) => {
            await api.reauth(password, csrf.current ?? '');
            asking.resolve();
            setAsking(undefined);
          }}
          onCancel={() => {
            asking.reject();
            setAsking(undefined);
          }}
        />
      )}
    </Context.Provider>
  );
}

export function useAuth(): AuthApi {
  const value = useContext(Context);
  if (value === undefined) throw new Error('useAuth hors de AuthProvider');
  return value;
}
