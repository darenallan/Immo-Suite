import { useEffect, useState } from 'react';
import { AuthProvider, useAuth } from './auth';
import { fr } from './texts/fr';
import { AuditScreen } from './screens/AuditScreen';
import { LeaseScreen, LeasesScreen } from './screens/Leases';
import { CollectScreen, PaymentScreen, PaymentsScreen } from './screens/Payments';
import { PropertiesScreen, PropertyScreen, TenantScreen, TenantsScreen, UnitScreen } from './screens/Catalog';
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
  // fiches (étape 05) : /biens/{id} et /lots/{id} relèvent de « Biens et lots », /locataires/{id} de « Locataires »
  const detail = /^\/(biens|lots|locataires|baux|paiements)\/([0-9a-f-]{36})$/.exec(path);
  const entryPath =
    detail === null
      ? path
      : detail[1] === 'locataires'
        ? '/locataires'
        : detail[1] === 'baux'
          ? '/baux'
          : detail[1] === 'paiements'
            ? '/paiements'
            : '/biens';
  const entry = NAV_ENTRIES.find((e) => e.path === entryPath);
  const canWrite = me.permissions.includes('catalog.write');
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
  } else if (detail !== null && detail[1] === 'biens') {
    content = <PropertyScreen id={detail[2] ?? ''} canWrite={canWrite} navigate={navigate} />;
  } else if (detail !== null && detail[1] === 'lots') {
    content = <UnitScreen id={detail[2] ?? ''} canWrite={canWrite} navigate={navigate} />;
  } else if (detail !== null && detail[1] === 'paiements') {
    content = <PaymentScreen id={detail[2] ?? ''} navigate={navigate} />;
  } else if (detail !== null && detail[1] === 'baux') {
    content = <LeaseScreen id={detail[2] ?? ''} canWrite={canWrite} navigate={navigate} />;
  } else if (detail !== null) {
    content = <TenantScreen id={detail[2] ?? ''} canWrite={canWrite} navigate={navigate} />;
  } else if (entry.id === 'properties') {
    content = (
      <PropertiesScreen
        canWrite={canWrite}
        canCreateOwner={me.permissions.includes('mandate.write')}
        ownerPlan={me.organization.plan === 'owner'}
        navigate={navigate}
      />
    );
  } else if (entry.id === 'leases') {
    content = <LeasesScreen canWrite={canWrite} navigate={navigate} />;
  } else if (entry.id === 'collect') {
    content = <CollectScreen navigate={navigate} />;
  } else if (entry.id === 'payments') {
    content = <PaymentsScreen navigate={navigate} />;
  } else if (entry.id === 'tenants') {
    content = <TenantsScreen canWrite={canWrite} navigate={navigate} />;
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
