import { useEffect, useState } from 'react';
import { AuthProvider, useAuth } from './auth';
import { fr } from './texts/fr';
import { AuditScreen } from './screens/AuditScreen';
import { ChooseOrganization } from './screens/ChooseOrganization';
import { Login } from './screens/Login';
import { InvitePage, ResetPage, readPublicLink, type PublicLink } from './screens/PublicLinks';
import { SettingsScreen } from './screens/SettingsScreen';
import { ComingSoon, Denied, Home, NotFound } from './screens/Simple';
import { Users } from './screens/Users';
import { trackScreen } from './observation/track';
import { currentPath, usePath } from './router';
import { NAV_ENTRIES, isRole, isVisible } from './shell/nav';
import { Shell } from './shell/Shell';

function Screens() {
  const { state } = useAuth();
  const [path, navigate] = usePath();
  // lien d'invitation ou de réinitialisation : lu une fois, puis retiré de l'adresse et de l'historique (jeton à usage unique)
  const [link, setLink] = useState<PublicLink | undefined>(() => readPublicLink(currentPath()));
  useEffect(() => {
    if (link !== undefined && window.location.pathname !== '/') window.history.replaceState(null, '', '/');
  }, [link]);

  // mesure d'usage : un écran affiché (seulement une fois la session ouverte et l'organisation choisie)
  const shown =
    state.status === 'ready' && state.me.organization !== null && state.me.organization !== undefined;
  useEffect(() => {
    if (shown) trackScreen(path, window.innerWidth);
  }, [shown, path]);

  if (link?.kind === 'reset') {
    return (
      <ResetPage
        token={link.token}
        onDone={() => {
          setLink(undefined);
          navigate('/');
        }}
      />
    );
  }
  if (link?.kind === 'invite' && state.status !== 'loading') {
    return (
      <InvitePage
        token={link.token}
        onDone={() => {
          setLink(undefined);
          navigate('/');
        }}
      />
    );
  }

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
  } else if (entry.id === 'users') {
    content = <Users organizationName={me.organization.name} plan={me.organization.plan} />;
  } else if (entry.id === 'settings') {
    content = <SettingsScreen permissions={me.permissions} />;
  } else if (entry.id === 'audit') {
    content = <AuditScreen />;
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
