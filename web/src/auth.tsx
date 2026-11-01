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

export type AuthState =
  { status: 'loading' } | { status: 'anonymous'; notice: string | undefined } | { status: 'ready'; me: Me };

interface AuthApi {
  state: AuthState;
  updateAvailable: boolean;
  login: (identifier: string, password: string) => Promise<void>;
  selectOrganization: (id: string) => Promise<void>;
  logout: () => Promise<void>;
  logoutAll: () => Promise<void>;
}

const Context = createContext<AuthApi | undefined>(undefined);

/** Intervalle de contrôle de la session pendant que l'application est ouverte (expiration par inactivité côté serveur). */
const REFRESH_MS = 60_000;

export function AuthProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<AuthState>({ status: 'loading' });
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const csrf = useRef<string | undefined>(undefined);

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
        try {
          await api.login(identifier, password);
        } catch (e) {
          if (e instanceof ApiProblem && e.code === 'client.upgrade_required') setUpdateAvailable(true);
          throw e;
        }
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

  return <Context.Provider value={value}>{children}</Context.Provider>;
}

export function useAuth(): AuthApi {
  const value = useContext(Context);
  if (value === undefined) throw new Error('useAuth hors de AuthProvider');
  return value;
}
