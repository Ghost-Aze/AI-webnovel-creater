import { useEffect, useMemo, useState } from "react";

import { normalizeCommandError } from "../../lib/command-error";
import { listModels } from "../../lib/commands";
import type {
  ModelProfile,
  ModelRef,
  QualityMode,
} from "../../types/provider";

export interface ChatAssistantPreset {
  id: string;
  label: string;
  description: string;
  systemInstructions: string;
}

interface ChatRuntimeControlsProps {
  assistants: readonly ChatAssistantPreset[];
  selectedAssistantId: string;
  onAssistantChange: (assistantId: string) => void;
  selectedModel: ModelRef | null;
  onModelChange: (model: ModelRef | null) => void;
  quality: QualityMode;
  onQualityChange: (quality: QualityMode) => void;
  disabled?: boolean;
}

function modelKey(model: Pick<ModelProfile, "provider_id" | "model_id">) {
  return model.provider_id + ":" + model.model_id;
}

export function ChatRuntimeControls({
  assistants,
  selectedAssistantId,
  onAssistantChange,
  selectedModel,
  onModelChange,
  quality,
  onQualityChange,
  disabled = false,
}: ChatRuntimeControlsProps) {
  const [models, setModels] = useState<ModelProfile[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    setIsLoading(true);
    setError(null);
    void listModels()
      .then((profiles) => {
        if (active) setModels(profiles);
      })
      .catch((commandError) => {
        if (active) setError(normalizeCommandError(commandError).message);
      })
      .finally(() => {
        if (active) setIsLoading(false);
      });
    return () => {
      active = false;
    };
  }, []);

  const providers = useMemo(
    () =>
      Array.from(new Set(models.map((model) => model.provider_id))).sort(),
    [models],
  );
  const selectedProviderId = selectedModel?.provider_id ?? "";
  const visibleModels = selectedProviderId
    ? models.filter((model) => model.provider_id === selectedProviderId)
    : models;
  const selectedAssistant = assistants.find(
    (assistant) => assistant.id === selectedAssistantId,
  );

  function handleProviderChange(providerId: string) {
    if (!providerId) {
      onModelChange(null);
      return;
    }
    const firstModel = models.find((model) => model.provider_id === providerId);
    onModelChange(
      firstModel
        ? { provider_id: firstModel.provider_id, model_id: firstModel.model_id }
        : null,
    );
  }

  function handleModelChange(value: string) {
    const selected = models.find((model) => modelKey(model) === value);
    onModelChange(
      selected
        ? { provider_id: selected.provider_id, model_id: selected.model_id }
        : null,
    );
  }

  return (
    <div className="chat-runtime-controls" aria-label="Chat runtime">
      <div className="chat-runtime-heading">
        <div>
          <p className="eyebrow">Runtime</p>
          <p className="chat-runtime-summary">
            {selectedAssistant?.description ??
              "Choose how the assistant should respond."}
          </p>
        </div>
        {isLoading && (
          <span className="chat-runtime-status" role="status">
            Loading models…
          </span>
        )}
      </div>

      <div className="chat-runtime-grid">
        <label className="chat-runtime-field">
          <span>Assistant</span>
          <select
            aria-label="Assistant"
            value={selectedAssistantId}
            onChange={(event) => onAssistantChange(event.target.value)}
            disabled={disabled || assistants.length === 0}
          >
            {assistants.map((assistant) => (
              <option key={assistant.id} value={assistant.id}>
                {assistant.label}
              </option>
            ))}
          </select>
        </label>

        <label className="chat-runtime-field">
          <span>Provider</span>
          <select
            aria-label="Provider"
            value={selectedProviderId}
            onChange={(event) => handleProviderChange(event.target.value)}
            disabled={disabled || isLoading}
          >
            <option value="">Automatic routing</option>
            {providers.map((providerId) => (
              <option key={providerId} value={providerId}>
                {providerId}
              </option>
            ))}
          </select>
        </label>

        <label className="chat-runtime-field">
          <span>Model</span>
          <select
            aria-label="Model"
            value={selectedModel ? modelKey(selectedModel) : ""}
            onChange={(event) => handleModelChange(event.target.value)}
            disabled={disabled || isLoading || models.length === 0}
          >
            <option value="">Automatic routing</option>
            {visibleModels.map((model) => (
              <option key={modelKey(model)} value={modelKey(model)}>
                {model.display_name} · {model.provider_id}
              </option>
            ))}
          </select>
        </label>

        <label className="chat-runtime-field">
          <span>Quality</span>
          <select
            aria-label="Quality"
            value={quality}
            onChange={(event) =>
              onQualityChange(event.target.value as QualityMode)
            }
            disabled={disabled}
          >
            <option value="fast">Fast</option>
            <option value="balanced">Balanced</option>
            <option value="deep">Deep</option>
          </select>
        </label>
      </div>

      {error ? (
        <p className="chat-runtime-note" role="alert">
          Model list could not be loaded. Automatic routing may still work.
          <span>{error}</span>
        </p>
      ) : models.length === 0 && !isLoading ? (
        <p className="chat-runtime-note">
          No configured models. Automatic routing will be used.{" "}
          <a href="/settings/providers">Configure providers</a>
        </p>
      ) : null}
    </div>
  );
}
