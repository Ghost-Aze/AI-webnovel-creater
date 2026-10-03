import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { normalizeCommandError } from "../../lib/command-error";
import {
  configureProvider,
  getCredentialStoreStatus,
  getProviderSettings,
  listModels,
  listProviders,
  removeProvider,
  testProvider,
  updateProvider,
} from "../../lib/commands";
import type {
  CredentialStoreStatus,
  ModelProfile,
  ProviderConfigureInput,
  ProviderDescriptor,
  ProviderSettings,
  ProviderUpdateInput,
} from "../../types/provider";
import { ProviderDetailsPanel } from "./ProviderDetailsPanel";
import {
  providerPresetDescriptors,
  providerPresets,
  providerPresetFor,
  mergeProviderModels,
} from "./provider-presets";
import { SettingsLayout } from "./SettingsLayout";

export function ProviderSettingsPage() {
  const [configuredProviders, setConfiguredProviders] = useState<
    ProviderDescriptor[]
  >([]);
  const [configuredModels, setConfiguredModels] = useState<ModelProfile[]>([]);
  const [selectedProviderId, setSelectedProviderId] = useState<string>(
    providerPresets[0].id,
  );
  const [selectedSettings, setSelectedSettings] =
    useState<ProviderSettings | null>(null);
  const [storeStatus, setStoreStatus] = useState<CredentialStoreStatus | null>(
    null,
  );
  const [isLoading, setIsLoading] = useState(true);
  const [isSavingApiKey, setIsSavingApiKey] = useState(false);
  const [testingProviderId, setTestingProviderId] = useState<string | null>(
    null,
  );
  const [testResults, setTestResults] = useState<Record<string, string>>({});
  const [error, setError] = useState<string | null>(null);
  const syncingPresetModels = useRef(new Set<string>());

  const configuredProviderIds = useMemo(
    () => configuredProviders.map((provider) => provider.id),
    [configuredProviders],
  );
  const selectedPreset = providerPresetFor(selectedProviderId);
  const selectedProvider =
    providerPresetDescriptors.find(
      (provider) => provider.id === selectedProviderId,
    ) ?? null;
  const selectedIsConfigured = configuredProviderIds.includes(selectedProviderId);
  const selectedModels = selectedPreset
    ? selectedIsConfigured
      ? mergeProviderModels(
          configuredModels.filter(
            (model) => model.provider_id === selectedProviderId,
          ),
          selectedPreset.models,
        )
      : selectedPreset.models
    : [];

  const loadProviders = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const [descriptors, profiles] = await Promise.all([
        listProviders(),
        listModels(),
      ]);
      setConfiguredProviders(descriptors);
      setConfiguredModels(profiles);
      setSelectedProviderId((current) =>
        descriptors.some((provider) => provider.id === current)
          ? current
          : descriptors[0]?.id ?? providerPresets[0].id,
      );
      try {
        setStoreStatus(await getCredentialStoreStatus());
      } catch {
        setStoreStatus(null);
      }
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadProviders();
  }, [loadProviders]);

  useEffect(() => {
    if (!selectedIsConfigured) {
      setSelectedSettings(null);
      return;
    }
    let isCurrent = true;
    void getProviderSettings(selectedProviderId)
      .then((settings) => {
        if (isCurrent) setSelectedSettings(settings);
      })
      .catch(() => {
        if (isCurrent) setSelectedSettings(null);
      });
    return () => {
      isCurrent = false;
    };
  }, [selectedIsConfigured, selectedProviderId]);

  useEffect(() => {
    if (
      !selectedIsConfigured ||
      !selectedPreset ||
      !selectedSettings ||
      selectedSettings.descriptor.id !== selectedProviderId ||
      syncingPresetModels.current.has(selectedProviderId)
    ) {
      return;
    }
    const mergedModels = mergeProviderModels(
      selectedSettings.models,
      selectedPreset.models,
    );
    if (mergedModels.length === selectedSettings.models.length) return;

    syncingPresetModels.current.add(selectedProviderId);
    void updateProvider({
      descriptor: selectedSettings.descriptor,
      base_url: selectedSettings.base_url,
      credential_id: selectedSettings.credential_id,
      credential_value: null,
      models: mergedModels,
    })
      .then(() => {
        setSelectedSettings((current) =>
          current ? { ...current, models: mergedModels } : current,
        );
        setConfiguredModels((current) => [
          ...current.filter((model) => model.provider_id !== selectedProviderId),
          ...mergedModels,
        ]);
      })
      .catch(() => {
        syncingPresetModels.current.delete(selectedProviderId);
      });
  }, [selectedIsConfigured, selectedPreset, selectedProviderId, selectedSettings]);

  async function handleSaveApiKey(value: string) {
    if (!selectedPreset) return;
    setError(null);
    setIsSavingApiKey(true);
    const descriptor: ProviderDescriptor = {
      id: selectedPreset.id,
      display_name: selectedPreset.displayName,
    };
    try {
      if (selectedIsConfigured) {
        const input: ProviderUpdateInput = {
          descriptor,
          base_url: selectedPreset.baseUrl,
          credential_id: selectedPreset.credentialId,
          credential_value: value,
          models: selectedModels.length > 0 ? selectedModels : selectedPreset.models,
        };
        await updateProvider(input);
      } else {
        const input: ProviderConfigureInput = {
          descriptor,
          base_url: selectedPreset.baseUrl,
          credential_id: selectedPreset.credentialId,
          credential_value: value,
          models: selectedPreset.models,
        };
        await configureProvider(input);
      }
      await loadProviders();
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsSavingApiKey(false);
    }
  }

  async function handleRemove(provider: ProviderDescriptor) {
    setError(null);
    try {
      await removeProvider(provider.id);
      await loadProviders();
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    }
  }

  async function handleTest(provider: ProviderDescriptor) {
    setError(null);
    setTestingProviderId(provider.id);
    try {
      const result = await testProvider(provider.id);
      setTestResults((current) => ({ ...current, [provider.id]: result.message }));
    } catch (commandError) {
      setError(
        `${provider.display_name}: ${normalizeCommandError(commandError).message}`,
      );
    } finally {
      setTestingProviderId(null);
    }
  }

  return (
    <section className="page-content providers-page">
      <header className="page-header">
        <div>
          <p className="eyebrow">Runtime connections</p>
          <h1>Providers</h1>
          <p className="page-lede">
            Choose a provider, enter its API key, and select the model from chat.
          </p>
        </div>
        <span className="page-header-note">Preset configurations</span>
      </header>

      {storeStatus && (
        <div
          className={`credential-status credential-status-${storeStatus.kind}`}
          role="status"
        >
          <span className="status-dot" aria-hidden="true" />
          {!storeStatus.available
            ? "Secure credential storage is unavailable on this build."
            : storeStatus.persistent
              ? "Credentials use platform secure storage."
              : "Session-only credential storage is active on this build."}
        </div>
      )}

      {isLoading ? (
        <p className="loading-state" role="status">
          Loading providers…
        </p>
      ) : (
        <div data-testid="provider-catalog">
          {error && (
            <div className="error-state" role="alert">
              <strong>Provider operation failed.</strong>
              <span>{error}</span>
              <button
                className="button button-ghost"
                type="button"
                onClick={() => void loadProviders()}
              >
                Try again
              </button>
            </div>
          )}
          <SettingsLayout
            providers={providerPresetDescriptors}
            configuredProviderIds={configuredProviderIds}
            selectedProviderId={selectedProviderId}
            onSelectProvider={setSelectedProviderId}
            onAddProvider={() => setSelectedProviderId(providerPresets[0].id)}
          >
            {selectedPreset && selectedProvider && (
              <ProviderDetailsPanel
                provider={selectedProvider}
                preset={selectedPreset}
                configured={selectedIsConfigured}
                models={selectedModels}
                settings={selectedSettings}
                storeStatus={storeStatus}
                testResult={testResults[selectedProvider.id]}
                testing={testingProviderId === selectedProvider.id}
                savingApiKey={isSavingApiKey}
                onSaveApiKey={(value) => void handleSaveApiKey(value)}
                onTest={() => void handleTest(selectedProvider)}
                onRemove={() => void handleRemove(selectedProvider)}
              />
            )}
          </SettingsLayout>
        </div>
      )}
    </section>
  );
}
