import { NavLink, Outlet, useLocation } from "react-router-dom";

import { ContextPanel } from "./ContextPanel";
import { Sidebar } from "./Sidebar";
import { StatusBar } from "./StatusBar";

export function AppShell() {
  const { pathname } = useLocation();
  const projectId = pathname.match(/^\/projects\/([^/]+)/)?.[1];
  const chatPath = projectId ? `/projects/${projectId}/chat` : "/chat";
  const manuscriptsPath = projectId
    ? `/projects/${projectId}/manuscripts`
    : "/manuscripts";

  return (
    <div className="app-shell jan-shell">
      <header
        className="mobile-shell-header"
        aria-label="Mobile workspace header"
      >
        <div className="mobile-shell-title">
          <span className="brand-mark" aria-hidden="true">
            W
          </span>
          <div>
            <p className="eyebrow">AI workspace</p>
            <strong>Webnovel Studio</strong>
          </div>
        </div>
        <span className="mobile-shell-state">
          <i className="status-dot" aria-hidden="true" />
          Local
        </span>
      </header>
      <Sidebar />
      <main className="app-main">
        <Outlet />
      </main>
      <ContextPanel />
      <StatusBar />
      <nav className="mobile-bottom-nav" aria-label="Mobile navigation">
        <NavLink className="mobile-bottom-nav-link" to="/projects" end>
          <span aria-hidden="true">◈</span>
          <small>Workspace</small>
        </NavLink>
        <NavLink className="mobile-bottom-nav-link" to={chatPath}>
          <span aria-hidden="true">✦</span>
          <small>Chat</small>
        </NavLink>
        {projectId && (
          <NavLink
            className="mobile-bottom-nav-link"
            to={manuscriptsPath}
          >
            <span aria-hidden="true">◈</span>
            <small>Manuscripts</small>
          </NavLink>
        )}
        <NavLink className="mobile-bottom-nav-link" to="/settings/providers">
          <span aria-hidden="true">✦</span>
          <small>Models</small>
        </NavLink>
      </nav>
    </div>
  );
}
