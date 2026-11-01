import { AuthProvider, useAuth } from './auth';
import { fr } from './texts/fr';
import { ChooseOrganization } from './screens/ChooseOrganization';
import { Login } from './screens/Login';
import { ComingSoon, Denied, Home, NotFound } from './screens/Simple';
import { usePath } from './router';
import { NAV_ENTRIES, isRole, isVisible } from './shell/nav';
import { Shell } from './shell/Shell';

function Screens() {
  const { state } = useAuth();
  const [path, navigate] = usePath();

  if (state.status === 'loading') {
    return (
      <main className="login">
        <p role="status" className="muted">
          {fr.common.loading}
        </p>
      </main>
    );
  }
  if (state.status === 'anonymous') return <Login notice={state.notice} />;

  const me = state.me;
  if (me.organization === null || me.organization === undefined) return <ChooseOrganization me={me} />;

  const role = me.membership?.role ?? '';
  const home = () => {
    navigate('/');
  };
  let content;
  const entry = NAV_ENTRIES.find((e) => e.path === path);
  if (entry === undefined) {
    content = <NotFound onHome={home} />;
  } else if (!isRole(role) || !isVisible(entry, role, me.permissions)) {
    // adresse tapée à la main pour une entrée que le rôle n'a pas : « Accès refusé » (le serveur refuse de toute façon)
    content = <Denied onHome={home} />;
  } else if (entry.id === 'home') {
    content = <Home me={me} />;
  } else {
    content = <ComingSoon title={fr.nav.entries[entry.id]} />;
  }
  return (
    <Shell me={me} path={path} navigate={navigate}>
      {content}
    </Shell>
  );
}

export function App() {
  return (
    <AuthProvider>
      <Screens />
    </AuthProvider>
  );
}
