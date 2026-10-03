import { useEffect, useState } from "react";

import type {
  CredentialStoreStatus,
  ModelProfile,
  ProviderDescriptor,
  ProviderSettings,
} from "../../types/provider";
import type { ProviderPreset } from "./provider-presets";
import { ProviderModelList } from "./ProviderModelList";

interface ProviderDetailsPanelProps {
  provider: ProviderDescriptor;
  preset: ProviderPreset;
  configured: boolean;
  models: ModelProfile[];
  settings: ProviderSettings | null;
  storeStatus: CredentialStoreStatus | null;
  testResult?: string;
  testing: boolean;
  savingApiKey: boolean;
  onSaveApiKey: (value: string) => void;
  onTest: () => void;
  onRemove: () => void;
}

export function ProviderDetailsPanel({
  provider,
  preset,
  configured,
  models,
  settings,
  storeStatus,
  testResult,
  testing,
  savingApiKey,
  onSaveApiKey,
  onTest,
  onRemove,
}: ProviderDetailsPanelProps) {
  const [apiKey, setApiKey] = useState("");
  const storageUnavailable = storeStatus?.available === false;

  useEffect(() => {
    setApiKey("");
  }, [provider.id]);

  return (
    <div className="provider-details-workspace" id="provider-details">
      <section
        className="provider-details-panel"
        aria-labelledby="provider-details-heading"
      >
        <div className="provider-details-header">
          <div>
            <p className="eyebrow">Model provider</p>
            <h2 id="provider-details-heading">{provider.display_name}</h2>
            <p className="provider-details-description">{preset.description}</p>
          </div>
          {configured && (
            <div className="provider-card-actions">
              <button
                className="button button-ghost button-small"
                type="button"
                onClick={onTest}
                disabled={testing}
                aria-label={`Test ${provider.display_name} connection`}
              >
                {testing ? "Testing…" : "Test"}
              </button>
              <button
                className="button button-danger-ghost button-small"
                type="button"
                onClick={onRemove}
                aria-label={`Remove ${provider.display_name}`}
              >
                Remove
              </button>
            </div>
          )}
        </div>

        <section
          className="provider-key-panel"
          aria-labelledby="provider-key-heading"
        >
          <div>
            <p className="eyebrow">Runtime credential</p>
            <h3 id="provider-key-heading">API key</h3>
            {configured ? (
              <p>
                <strong>API key saved securely.</strong> Enter a new key only
                when you want to replace it.
              </p>
            ) : (
              <p>
                Enter your API key. Provider settings and starter models are
                already prepared.
              </p>
            )}
          </div>
          <form
            className="provider-key-form"
            onSubmit={(event) => {
              event.preventDefault();
              if (apiKey.trim() && !storageUnavailable) onSaveApiKey(apiKey.trim());
            }}
          >
            <label className="field-label" htmlFor={`${provider.id}-api-key`}>
              <span className="sr-only">{provider.display_name} API key</span>
              <input
                id={`${provider.id}-api-key`}
                aria-label={`${provider.display_name} API key`}
                type="password"
                autoComplete="new-password"
                placeholder={
                  configured ? "Enter a new API key" : "Enter primary API key"
                }
                value={apiKey}
                onChange={(event) => setApiKey(event.target.value)}
                disabled={storageUnavailable || savingApiKey}
              />
            </label>
            <button
              className="button button-primary"
              type="submit"
              disabled={!apiKey.trim() || storageUnavailable || savingApiKey}
            >
              {savingApiKey
                ? "Saving…"
                : configured
                  ? `Update ${provider.display_name} API key`
                  : `Connect ${provider.display_name}`}
            </button>
          </form>
          {storageUnavailable && (
            <p className="provider-metadata-notice" role="status">
              Credentials are unavailable; provider metadata is still visible.
            </p>
          )}
        </section>

        <div className="provider-metadata-grid" id="credentials">
          <div>
            <span className="toolbar-label">Base URL</span>
            <code>{settings?.base_url ?? preset.baseUrl}</code>
          </div>
          <div>
            <span className="toolbar-label">Credential ID</span>
            <code>{settings?.credential_id ?? preset.credentialId}</code>
          </div>
        </div>
        {testResult && (
          <p className="provider-test-result" role="status">
            {testResult}
          </p>
        )}
      </section>
      <ProviderModelList models={models} />
      <p className="provider-chat-model-note">
        Choose the model from the chat composer.
      </p>
    </div>
  );
}
