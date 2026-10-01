import { NavLink } from "react-router-dom";

export function Sidebar() {
  return (
    <aside className="sidebar" aria-label="Project navigation">
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
        <NavLink className="nav-link" to="/projects">
          <span aria-hidden="true">▦</span>
          Projects
        </NavLink>
        <span className="nav-link nav-link-muted" aria-disabled="true">
          <span aria-hidden="true">✦</span>
          Developer Chat
          <small>Phase 1</small>
        </span>
        <span className="nav-link nav-link-muted" aria-disabled="true">
          <span aria-hidden="true">◈</span>
          Manuscripts
          <small>Phase 1</small>
        </span>
      </nav>
      <div className="sidebar-note">
        <span className="status-dot" aria-hidden="true" />
        <span>Local workspace</span>
      </div>
    </aside>
  );
}
