import type { ReactNode } from "react";

import type { ProviderDescriptor } from "../../types/provider";
import { ProviderNavigation } from "./ProviderNavigation";
import { SettingsNavigation } from "./SettingsNavigation";

interface SettingsLayoutProps {
  providers: ProviderDescriptor[];
  configuredProviderIds?: string[];
  selectedProviderId: string | null;
  onSelectProvider: (providerId: string) => void;
  onAddProvider: () => void;
  children: ReactNode;
}

export function SettingsLayout({
  providers,
  configuredProviderIds,
  selectedProviderId,
  onSelectProvider,
  onAddProvider,
  children,
}: SettingsLayoutProps) {
  return (
    <div className="settings-layout">
      <SettingsNavigation />
      <div className="settings-layout-section">
        <div className="settings-section-header">
          <p className="eyebrow">Model routing</p>
          <h2>Model Providers</h2>
          <p>
            Connect providers and review the models available to every chat.
          </p>
        </div>
        <div className="provider-settings-layout">
          <ProviderNavigation
            providers={providers}
            configuredProviderIds={configuredProviderIds}
            selectedProviderId={selectedProviderId}
            onSelect={onSelectProvider}
            onAdd={onAddProvider}
          />
          <div className="provider-settings-main">{children}</div>
        </div>
      </div>
    </div>
  );
}
