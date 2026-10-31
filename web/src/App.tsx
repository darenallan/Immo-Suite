import { fr } from './texts/fr';
import { appEnvironment, appVersion } from './version';

export function App() {
  return (
    <main className="page">
      <header className="page__header">
        <h1>{fr.app.name}</h1>
        <p className="muted">{fr.app.tagline}</p>
      </header>
      <section aria-labelledby="home-title" className="card">
        <h2 id="home-title">{fr.home.title}</h2>
        <p>{fr.home.intro}</p>
        <dl className="facts">
          <dt>{fr.home.versionLabel}</dt>
          <dd data-testid="version">{appVersion}</dd>
          <dt>{fr.home.environmentLabel}</dt>
          <dd data-testid="environment">{appEnvironment}</dd>
        </dl>
      </section>
    </main>
  );
}
