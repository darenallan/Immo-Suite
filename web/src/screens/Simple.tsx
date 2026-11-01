import type { Me } from '../api/client';
import { fr, type RoleKey } from '../texts/fr';

export function Home({ me }: { me: Me }) {
  const role = me.membership?.role as RoleKey | undefined;
  return (
    <section aria-labelledby="home-title" className="card">
      <h1 id="home-title">{fr.home.title}</h1>
      <p>
        {fr.home.welcome} {me.user.display_name}
      </p>
      <dl className="facts">
        <dt>{fr.home.organization}</dt>
        <dd data-testid="org-name">{me.organization?.name}</dd>
        <dt>{fr.home.role}</dt>
        <dd data-testid="role">{role === undefined ? '' : fr.roles[role]}</dd>
      </dl>
      <p className="muted">{fr.home.comingSoon}</p>
    </section>
  );
}

export function ComingSoon({ title }: { title: string }) {
  return (
    <section aria-labelledby="soon-title" className="card">
      <h1 id="soon-title">{title}</h1>
      <p className="muted">{fr.nav.comingSoon}</p>
    </section>
  );
}

export function Denied({ onHome }: { onHome: () => void }) {
  return (
    <section aria-labelledby="denied-title" className="card" role="alert">
      <h1 id="denied-title">{fr.denied.title}</h1>
      <p>{fr.denied.body}</p>
      <button className="button" type="button" onClick={onHome}>
        {fr.denied.back}
      </button>
    </section>
  );
}

export function NotFound({ onHome }: { onHome: () => void }) {
  return (
    <section aria-labelledby="notfound-title" className="card">
      <h1 id="notfound-title">{fr.notFound.title}</h1>
      <p>{fr.notFound.body}</p>
      <button className="button" type="button" onClick={onHome}>
        {fr.notFound.back}
      </button>
    </section>
  );
}
