import type { ProviderDescriptor } from "../../types/provider";

interface ProviderNavigationProps {
  providers: ProviderDescriptor[];
  configuredProviderIds?: string[];
  selectedProviderId: string | null;
  onSelect: (providerId: string) => void;
  onAdd: () => void;
}

export function ProviderNavigation({
  providers,
  configuredProviderIds = [],
  selectedProviderId,
  onSelect,
  onAdd,
}: ProviderNavigationProps) {
  return (
    <aside className="provider-settings-sidebar">
      <div className="provider-settings-sidebar-heading">
        <p className="eyebrow">Catalog</p>
        <strong>Model Providers</strong>
      </div>
      <nav
        className="provider-settings-navigation"
        aria-label="Provider catalog navigation"
      >
        <a href="#provider-details" aria-current="page">
          Provider details
        </a>
        <a href="#models">Models</a>
        <a href="#credentials">Credentials</a>
      </nav>
      <div className="provider-settings-provider-list">
        <div className="provider-settings-list-heading">
          <span className="toolbar-label">Model providers</span>
          <button
            className="icon-button"
            type="button"
            aria-label="Add provider to settings navigation"
            onClick={onAdd}
          >
            +
          </button>
        </div>
        {providers.length > 0 ? (
          providers.map((provider) => (
            <button
              className="provider-settings-provider"
              type="button"
              aria-label={provider.display_name}
              key={provider.id}
              aria-pressed={selectedProviderId === provider.id}
              onClick={() => onSelect(provider.id)}
            >
              <span className="provider-settings-provider-mark" aria-hidden="true">
                {provider.display_name.slice(0, 1).toUpperCase()}
              </span>
              <span>{provider.display_name}</span>
              {configuredProviderIds.includes(provider.id) && (
                <small className="provider-settings-provider-status">
                  Connected
                </small>
              )}
            </button>
          ))
        ) : (
          <p className="provider-settings-no-providers">No providers yet.</p>
        )}
      </div>
    </aside>
  );
}
