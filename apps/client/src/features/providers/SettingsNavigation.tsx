import { NavLink } from "react-router-dom";

const deferredSections = ["Chat", "Appearance", "Storage"];

export function SettingsNavigation() {
  return (
    <nav className="settings-navigation" aria-label="Settings navigation">
      <p className="eyebrow">Settings</p>
      <div className="settings-navigation-list">
        <NavLink
          className="settings-navigation-link"
          to="/settings/providers"
          end
        >
          Model Providers
        </NavLink>
        {deferredSections.map((section) => (
          <span
            className="settings-navigation-link is-disabled"
            aria-disabled="true"
            key={section}
          >
            {section}
          </span>
        ))}
      </div>
    </nav>
  );
}
