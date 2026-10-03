import { NavLink, useLocation } from "react-router-dom";

export function Sidebar() {
  const { pathname } = useLocation();
  const projectId = pathname.match(/^\/projects\/([^/]+)/)?.[1];
  const chatPath = projectId ? `/projects/${projectId}/chat` : "/chat";
  const manuscriptsPath = projectId
    ? `/projects/${projectId}/manuscripts`
    : "/manuscripts";

  return (
    <aside className="sidebar" aria-label="Application sidebar">
      <div className="brand-lockup">
        <span className="brand-mark" aria-hidden="true">
          W
        </span>
        <div>
          <p className="eyebrow">Webnovel</p>
          <strong>AI Studio</strong>
        </div>
      </div>
      <nav className="primary-nav" aria-label="Primary">
        <NavLink className="nav-link" to="/projects" end>
          <span aria-hidden="true">▦</span>
          Projects
        </NavLink>
        <NavLink className="nav-link" to="/settings/providers">
          <span aria-hidden="true">⌘</span>
          Providers
        </NavLink>
      </nav>
      {projectId && (
        <div className="project-nav-block">
          <p className="eyebrow project-nav-heading">Current project</p>
          <nav className="project-nav" aria-label="Project navigation">
            <NavLink className="nav-link" to={`/projects/${projectId}`} end>
              <span aria-hidden="true">◌</span>
              Overview
            </NavLink>
            <NavLink className="nav-link" to={chatPath}>
              <span aria-hidden="true">✦</span>
              Developer Chat
            </NavLink>
            <NavLink className="nav-link" to={manuscriptsPath}>
              <span aria-hidden="true">◈</span>
              Manuscripts
            </NavLink>
          </nav>
        </div>
      )}
      <div className="sidebar-note">
        <span className="status-dot" aria-hidden="true" />
        <span>Local workspace</span>
      </div>
    </aside>
  );
}
