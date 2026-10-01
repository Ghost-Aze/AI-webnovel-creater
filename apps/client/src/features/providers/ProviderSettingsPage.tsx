import { type FormEvent, useCallback, useEffect, useState } from "react";

import { normalizeCommandError } from "../../lib/command-error";
import {
  configureProvider,
  getCredentialStoreStatus,
  listModels,
  listProviders,
  removeProvider,
} from "../../lib/commands";
import type {
  CredentialStoreStatus,
  ModelProfile,
  ProviderConfigureInput,
  ProviderDescriptor,
} from "../../types/provider";

interface ModelDraft {
  modelId: string;
  displayName: string;
  contextWindowTokens: string;
  defaultOutputTokens: string;
  streaming: boolean;
}

interface ProviderDraft {
  id: string;
  displayName: string;
  baseUrl: string;
  credentialId: string;
  credentialValue: string;
  models: ModelDraft[];
}

function createModelDraft(): ModelDraft {
  return {
    modelId: "",
    displayName: "",
    contextWindowTokens: "8192",
    defaultOutputTokens: "1024",
    streaming: true,
  };
}

function createDraft(): ProviderDraft {
  return {
    id: "",
    displayName: "",
    baseUrl: "https://api.example.com/v1",
    credentialId: "",
    credentialValue: "",
    models: [createModelDraft()],
  };
}

function toModelProfile(providerId: string, model: ModelDraft): ModelProfile {
  return {
    provider_id: providerId.trim(),
    model_id: model.modelId.trim(),
    display_name: model.displayName.trim(),
    context_window_tokens: Number(model.contextWindowTokens),
    default_output_tokens: Number(model.defaultOutputTokens),
    strengths: [],
    weaknesses: [],
    strategy: [],
    capabilities: {
      streaming: model.streaming,
      embeddings: false,
      tools: false,
      vision: false,
      structured_output: false,
      prompt_caching: false,
    },
  };
}

function validateDraft(draft: ProviderDraft): string | null {
  if (!draft.id.trim()) return "Provider ID is required.";
  if (!draft.displayName.trim()) return "Provider name is required.";
  if (!draft.baseUrl.trim()) return "Base URL is required.";
  if (!draft.credentialId.trim()) return "Credential ID is required.";
  if (!draft.credentialValue.trim()) return "API key is required.";
  if (draft.models.length === 0) return "At least one model is required.";
  for (const model of draft.models) {
    if (!model.modelId.trim()) return "Model ID is required.";
    if (!model.displayName.trim()) return "Model display name is required.";

    const contextWindow = Number(model.contextWindowTokens);
    const outputTokens = Number(model.defaultOutputTokens);
    if (
      !Number.isInteger(contextWindow) ||
      !Number.isInteger(outputTokens) ||
      outputTokens <= 0 ||
      contextWindow <= outputTokens
    ) {
      return "Model context must be larger than its positive output limit.";
    }
  }
  return null;
}

export function ProviderSettingsPage() {
  const [providers, setProviders] = useState<ProviderDescriptor[]>([]);
  const [models, setModels] = useState<ModelProfile[]>([]);
  const [storeStatus, setStoreStatus] = useState<CredentialStoreStatus | null>(
    null,
  );
  const [draft, setDraft] = useState<ProviderDraft>(createDraft);
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);

  const loadProviders = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const [descriptors, profiles] = await Promise.all([
        listProviders(),
        listModels(),
      ]);
      setProviders(descriptors);
      setModels(profiles);
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

  function openForm() {
    setDraft(createDraft());
    setFormError(null);
    setIsFormOpen(true);
  }

  function closeForm() {
    if (!isSubmitting) {
      setIsFormOpen(false);
      setFormError(null);
    }
  }

  function updateModel(index: number, update: Partial<ModelDraft>) {
    setDraft((current) => ({
      ...current,
      models: current.models.map((model, modelIndex) =>
        modelIndex === index ? { ...model, ...update } : model,
      ),
    }));
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const validationError = validateDraft(draft);
    if (validationError) {
      setFormError(validationError);
      return;
    }

    setFormError(null);
    setIsSubmitting(true);
    const input: ProviderConfigureInput = {
      descriptor: {
        id: draft.id.trim(),
        display_name: draft.displayName.trim(),
      },
      base_url: draft.baseUrl.trim(),
      credential_id: draft.credentialId.trim(),
      credential_value: draft.credentialValue,
      models: draft.models.map((model) => toModelProfile(draft.id, model)),
    };
    try {
      await configureProvider(input);
      setDraft(createDraft());
      setIsFormOpen(false);
      await loadProviders();
    } catch (commandError) {
      setFormError(normalizeCommandError(commandError).message);
    } finally {
      setIsSubmitting(false);
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

  return (
    <section className="page-content providers-page">
      <header className="page-header">
        <div>
          <p className="eyebrow">Runtime connections</p>
          <h1>Providers</h1>
          <p className="page-lede">
            Connect a model provider without putting credentials in story
            memory.
          </p>
        </div>
        <button
          className="button button-primary"
          type="button"
          onClick={openForm}
        >
          <span aria-hidden="true">+</span> Add provider
        </button>
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

      {isLoading && (
        <p className="loading-state" role="status">
          Loading providers…
        </p>
      )}
      {!isLoading && error && (
        <div className="error-state" role="alert">
          <strong>Providers could not be loaded.</strong>
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
      {!isLoading && !error && providers.length === 0 && (
        <div
          className="empty-state providers-empty"
          data-testid="empty-providers"
        >
          <div className="empty-orbit" aria-hidden="true">
            ◌
          </div>
          <h2>No providers configured</h2>
          <p>
            Add a provider to make model execution available to later workflows.
          </p>
          <button
            className="button button-secondary"
            type="button"
            onClick={openForm}
          >
            Configure your first provider
          </button>
        </div>
      )}
      {!isLoading && !error && providers.length > 0 && (
        <div className="provider-grid" aria-label="Configured providers">
          {providers.map((provider) => {
            const providerModels = models.filter(
              (model) => model.provider_id === provider.id,
            );
            return (
              <article className="provider-card" key={provider.id}>
                <div className="provider-card-header">
                  <div>
                    <p className="eyebrow">Provider</p>
                    <h2>{provider.display_name}</h2>
                  </div>
                  <button
                    className="button button-danger-ghost button-small"
                    type="button"
                    onClick={() => void handleRemove(provider)}
                    aria-label={`Remove ${provider.display_name}`}
                  >
                    Remove
                  </button>
                </div>
                <code className="provider-id">{provider.id}</code>
                <div className="provider-models">
                  <span className="toolbar-label">Models</span>
                  {providerModels.length > 0 ? (
                    <ul>
                      {providerModels.map((model) => (
                        <li key={`${model.provider_id}:${model.model_id}`}>
                          <strong>{model.display_name}</strong>
                          <span>{model.model_id}</span>
                        </li>
                      ))}
                    </ul>
                  ) : (
                    <span className="provider-no-models">
                      No models reported.
                    </span>
                  )}
                </div>
              </article>
            );
          })}
        </div>
      )}

      {isFormOpen && (
        <div
          className="dialog-backdrop"
          role="presentation"
          onMouseDown={(event) =>
            event.target === event.currentTarget && closeForm()
          }
        >
          <section
            className="dialog provider-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="provider-dialog-title"
          >
            <div className="dialog-header">
              <div>
                <p className="eyebrow">Runtime setup</p>
                <h2 id="provider-dialog-title">Add provider</h2>
              </div>
              <button
                className="icon-button"
                type="button"
                onClick={closeForm}
                aria-label="Close dialog"
              >
                ×
              </button>
            </div>
            <form onSubmit={handleSubmit}>
              <div className="form-grid form-grid-two">
                <label className="field-label" htmlFor="provider-id">
                  Provider ID
                  <input
                    id="provider-id"
                    value={draft.id}
                    onChange={(event) =>
                      setDraft((current) => ({
                        ...current,
                        id: event.target.value,
                      }))
                    }
                    placeholder="openai"
                  />
                </label>
                <label className="field-label" htmlFor="provider-name">
                  Display name
                  <input
                    id="provider-name"
                    value={draft.displayName}
                    onChange={(event) =>
                      setDraft((current) => ({
                        ...current,
                        displayName: event.target.value,
                      }))
                    }
                    placeholder="OpenAI"
                  />
                </label>
              </div>
              <label className="field-label" htmlFor="provider-base-url">
                Base URL
                <input
                  id="provider-base-url"
                  type="url"
                  value={draft.baseUrl}
                  onChange={(event) =>
                    setDraft((current) => ({
                      ...current,
                      baseUrl: event.target.value,
                    }))
                  }
                  placeholder="https://api.openai.com/v1"
                />
              </label>
              <div className="form-grid form-grid-two">
                <label className="field-label" htmlFor="provider-credential-id">
                  Credential ID
                  <input
                    id="provider-credential-id"
                    value={draft.credentialId}
                    onChange={(event) =>
                      setDraft((current) => ({
                        ...current,
                        credentialId: event.target.value,
                      }))
                    }
                    placeholder="openai-primary"
                  />
                </label>
                <label
                  className="field-label"
                  htmlFor="provider-credential-value"
                >
                  API key
                  <input
                    id="provider-credential-value"
                    type="password"
                    autoComplete="new-password"
                    value={draft.credentialValue}
                    onChange={(event) =>
                      setDraft((current) => ({
                        ...current,
                        credentialValue: event.target.value,
                      }))
                    }
                    placeholder="Stored by the runtime"
                  />
                </label>
              </div>

              <div className="provider-model-form">
                <div className="provider-model-form-header">
                  <div>
                    <span className="toolbar-label">Model profiles</span>
                    <p>Model routing remains outside this phase.</p>
                  </div>
                  <button
                    className="button button-ghost button-small"
                    type="button"
                    onClick={() =>
                      setDraft((current) => ({
                        ...current,
                        models: [...current.models, createModelDraft()],
                      }))
                    }
                  >
                    Add model
                  </button>
                </div>
                {draft.models.map((model, index) => (
                  <div className="provider-model-entry" key={`model-${index}`}>
                    <div className="form-grid form-grid-two">
                      <label
                        className="field-label"
                        htmlFor={`model-id-${index}`}
                      >
                        Model ID
                        <input
                          id={`model-id-${index}`}
                          value={model.modelId}
                          onChange={(event) =>
                            updateModel(index, { modelId: event.target.value })
                          }
                          placeholder="gpt-4o-mini"
                        />
                      </label>
                      <label
                        className="field-label"
                        htmlFor={`model-name-${index}`}
                      >
                        Display name
                        <input
                          id={`model-name-${index}`}
                          aria-label={`Model display name ${index + 1}`}
                          value={model.displayName}
                          onChange={(event) =>
                            updateModel(index, {
                              displayName: event.target.value,
                            })
                          }
                          placeholder="GPT-4o mini"
                        />
                      </label>
                    </div>
                    <div className="form-grid form-grid-two">
                      <label
                        className="field-label"
                        htmlFor={`model-context-${index}`}
                      >
                        Context tokens
                        <input
                          id={`model-context-${index}`}
                          type="number"
                          min="1"
                          value={model.contextWindowTokens}
                          onChange={(event) =>
                            updateModel(index, {
                              contextWindowTokens: event.target.value,
                            })
                          }
                        />
                      </label>
                      <label
                        className="field-label"
                        htmlFor={`model-output-${index}`}
                      >
                        Output tokens
                        <input
                          id={`model-output-${index}`}
                          type="number"
                          min="1"
                          value={model.defaultOutputTokens}
                          onChange={(event) =>
                            updateModel(index, {
                              defaultOutputTokens: event.target.value,
                            })
                          }
                        />
                      </label>
                    </div>
                    <label
                      className="checkbox-field"
                      htmlFor={`model-streaming-${index}`}
                    >
                      <input
                        id={`model-streaming-${index}`}
                        type="checkbox"
                        checked={model.streaming}
                        onChange={(event) =>
                          updateModel(index, {
                            streaming: event.target.checked,
                          })
                        }
                      />
                      Supports streaming
                    </label>
                    {draft.models.length > 1 && (
                      <button
                        className="button button-danger-ghost button-small"
                        type="button"
                        onClick={() =>
                          setDraft((current) => ({
                            ...current,
                            models: current.models.filter(
                              (_, modelIndex) => modelIndex !== index,
                            ),
                          }))
                        }
                      >
                        Remove model
                      </button>
                    )}
                  </div>
                ))}
              </div>

              {formError && (
                <p className="form-error" role="alert">
                  {formError}
                </p>
              )}
              <div className="dialog-actions">
                <button
                  className="button button-ghost"
                  type="button"
                  onClick={closeForm}
                >
                  Cancel
                </button>
                <button
                  className="button button-primary"
                  type="submit"
                  disabled={isSubmitting || storeStatus?.available === false}
                >
                  {isSubmitting ? "Saving…" : "Save provider"}
                </button>
              </div>
            </form>
          </section>
        </div>
      )}
    </section>
  );
}
