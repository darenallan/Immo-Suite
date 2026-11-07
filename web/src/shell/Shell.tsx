import { useEffect, useRef, useState, type MouseEvent, type ReactNode } from 'react';
import type { Me } from '../api/client';
import { useAuth } from '../auth';
import { FeedbackDialog } from '../observation/FeedbackDialog';
import { SearchBox } from '../screens/Catalog';
import { fr, type RoleKey } from '../texts/fr';
import { NAV_ENTRIES, bottomBar, isRole, visibleEntries, type NavEntry } from './nav';

interface ShellProps {
  me: Me;
  path: string;
  navigate: (path: string) => void;
  children: ReactNode;
}

function Link({
  entry,
  current,
  navigate,
  onDone,
}: {
  entry: NavEntry;
  current: boolean;
  navigate: (p: string) => void;
  onDone?: () => void;
}) {
  const onClick = (e: MouseEvent) => {
    e.preventDefault();
    navigate(entry.path);
    onDone?.();
  };
  return (
    <a
      className={current ? 'nav__link nav__link--current' : 'nav__link'}
      href={entry.path}
      aria-current={current ? 'page' : undefined}
      onClick={onClick}
    >
      {fr.nav.entries[entry.id]}
    </a>
  );
}

/** Coquille de navigation : menu latéral de 248 px à partir de 1024 px ; sinon barre du bas à 4 entrées dont « Plus » (UI_UX_DESIGN.md). */
export function Shell({ me, path, navigate, children }: ShellProps) {
  const { logout, logoutAll, selectOrganization, updateAvailable, csrf } = useAuth();
  const [userOpen, setUserOpen] = useState(false);
  const [moreOpen, setMoreOpen] = useState(false);
  const [feedbackOpen, setFeedbackOpen] = useState(false);
  const [reference, setReference] = useState<string | undefined>(undefined);
  const [copied, setCopied] = useState(false);
  const userButton = useRef<HTMLButtonElement>(null);
  const roleText = me.membership?.role ?? '';
  const role: RoleKey | undefined = isRole(roleText) ? roleText : undefined;
  const permissions = me.permissions;
  const visible = role === undefined ? [] : visibleEntries(role, permissions);
  const bar = role === undefined ? [] : bottomBar(role, permissions);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setUserOpen(false);
        setMoreOpen(false);
      }
    };
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('keydown', onKey);
    };
  }, []);

  const switchable = me.organizations.length > 1;
  const roleLabel = role === undefined ? '' : fr.roles[role];

  return (
    <div className="shell">
      <a className="skip" href="#contenu">
        {fr.common.skipToContent}
      </a>
      {updateAvailable && (
        <div className="banner" role="status">
          <span>{fr.update.banner}</span>
          <button
            className="button button--small"
            type="button"
            onClick={() => {
              window.location.reload();
            }}
          >
            {fr.update.reload}
          </button>
        </div>
      )}
      <aside className="sidebar">
        <p className="sidebar__brand">{fr.app.name}</p>
        <nav aria-label={fr.nav.label}>
          <ul>
            {visible.map((e) => (
              <li key={e.id}>
                <Link entry={e} current={path === e.path} navigate={navigate} />
              </li>
            ))}
          </ul>
        </nav>
      </aside>
      <header className="topbar">
        <p className="topbar__title">{me.organization?.name}</p>
        {me.permissions.includes('search.use') && (
          <SearchBox navigate={navigate} canCreateTenant={me.permissions.includes('catalog.write')} />
        )}
        <div className="usermenu">
          <button
            ref={userButton}
            className="button button--secondary"
            type="button"
            aria-expanded={userOpen}
            aria-controls="user-panel"
            onClick={() => {
              setUserOpen(!userOpen);
            }}
          >
            {fr.menu.user} · {me.user.display_name}
          </button>
          {userOpen && (
            <div id="user-panel" className="panel" role="group" aria-label={fr.menu.user}>
              <p>
                <strong>{me.user.display_name}</strong>
                <br />
                <span className="muted">
                  {roleLabel} · {me.organization?.name}
                </span>
              </p>
              {switchable && (
                <ul className="panel__list">
                  {me.organizations
                    .filter((o) => o.id !== me.organization?.id)
                    .map((o) => (
                      <li key={o.id}>
                        <button
                          className="button button--secondary"
                          type="button"
                          onClick={() => {
                            setUserOpen(false);
                            void selectOrganization(o.id);
                          }}
                        >
                          {fr.menu.switchOrganization} : {o.name}
                        </button>
                      </li>
                    ))}
                </ul>
              )}
              <button
                className="button button--secondary"
                type="button"
                onClick={() => {
                  setUserOpen(false);
                  setFeedbackOpen(true);
                }}
              >
                {fr.menu.reportProblem}
              </button>
              <button
                className="button button--secondary"
                type="button"
                onClick={() => {
                  void logoutAll();
                }}
              >
                {fr.menu.logoutAll}
              </button>
              <button
                className="button"
                type="button"
                onClick={() => {
                  void logout();
                }}
              >
                {fr.menu.logout}
              </button>
            </div>
          )}
        </div>
      </header>
      <main id="contenu" className="content" tabIndex={-1}>
        {children}
      </main>
      <nav className="bottombar" aria-label={fr.nav.label}>
        <ul>
          {bar.map((id) => {
            const entry = NAV_ENTRIES.find((e) => e.id === id);
            return entry === undefined ? null : (
              <li key={id}>
                <Link entry={entry} current={path === entry.path} navigate={navigate} />
              </li>
            );
          })}
          <li>
            <button
              className="nav__link"
              type="button"
              aria-expanded={moreOpen}
              aria-controls="more-panel"
              onClick={() => {
                setMoreOpen(!moreOpen);
              }}
            >
              {fr.nav.more}
            </button>
          </li>
        </ul>
        {moreOpen && (
          <div id="more-panel" className="sheet" role="group" aria-label={fr.nav.moreTitle}>
            <ul>
              {visible
                .filter((e) => !bar.includes(e.id))
                .map((e) => (
                  <li key={e.id}>
                    <Link
                      entry={e}
                      current={path === e.path}
                      navigate={navigate}
                      onDone={() => {
                        setMoreOpen(false);
                      }}
                    />
                  </li>
                ))}
              <li>
                <button
                  className="nav__link"
                  type="button"
                  onClick={() => {
                    setMoreOpen(false);
                    setFeedbackOpen(true);
                  }}
                >
                  {fr.menu.reportProblem}
                </button>
              </li>
            </ul>
          </div>
        )}
      </nav>
      {feedbackOpen && (
        <FeedbackDialog
          csrf={csrf}
          onClose={() => {
            setFeedbackOpen(false);
          }}
          onSent={(ref) => {
            setFeedbackOpen(false);
            setCopied(false);
            setReference(ref);
          }}
        />
      )}
      {reference !== undefined && (
        <div className="toast" role="status">
          <span>{fr.feedback.sent(reference)}</span>
          <button
            className="button button--small"
            type="button"
            onClick={() => {
              void navigator.clipboard.writeText(reference).then(
                () => {
                  setCopied(true);
                },
                () => undefined,
              );
            }}
          >
            {copied ? fr.feedback.copied : fr.feedback.copy}
          </button>
          <button
            className="button button--small button--secondary"
            type="button"
            onClick={() => {
              setReference(undefined);
            }}
          >
            {fr.feedback.dismiss}
          </button>
        </div>
      )}
    </div>
  );
}
