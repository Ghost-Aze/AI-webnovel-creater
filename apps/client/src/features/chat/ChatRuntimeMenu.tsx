import { useState } from "react";

import type {
  ChatAssistantDescriptor,
  ChatRuntimeSettingsInput,
} from "../../types/conversation";
import type { ModelProfile } from "../../types/provider";

interface ChatRuntimeMenuProps {
  assistants: ChatAssistantDescriptor[];
  settings: ChatRuntimeSettingsInput;
  models: ModelProfile[];
  onChange: (settings: ChatRuntimeSettingsInput) => void;
  disabled?: boolean;
}

function modelKey(providerId: string, modelId: string) {
  return `${providerId}:${modelId}`;
}

export function ChatRuntimeMenu({
  assistants,
  settings,
  models,
  onChange,
  disabled = false,
}: ChatRuntimeMenuProps) {
  const [isOpen, setIsOpen] = useState(false);
  const providers = Array.from(new Set(models.map((model) => model.provider_id))).sort();
  const selectedModel =
    settings.provider_id && settings.model_id
      ? modelKey(settings.provider_id, settings.model_id)
      : "";
  const visibleModels = settings.provider_id
    ? models.filter((model) => model.provider_id === settings.provider_id)
    : models;

  return (
    <div className="chat-runtime-menu">
      <button
        className="button button-ghost chat-runtime-menu-trigger"
        type="button"
        aria-expanded={isOpen}
        aria-label="Runtime settings"
        onClick={() => setIsOpen((current) => !current)}
        disabled={disabled}
      >
        Runtime settings
      </button>
      {isOpen && (
        <div className="chat-runtime-menu-panel" role="region" aria-label="Runtime settings panel">
          <label className="chat-runtime-field">
            <span>Assistant</span>
            <select
              aria-label="Assistant"
              value={settings.assistant_id}
              onChange={(event) =>
                onChange({ ...settings, assistant_id: event.target.value })
              }
              disabled={disabled}
            >
              {assistants.map((assistant) => (
                <option value={assistant.id} key={assistant.id}>
                  {assistant.label}
                </option>
              ))}
            </select>
          </label>
          <label className="chat-runtime-field">
            <span>Provider</span>
            <select
              aria-label="Provider"
              value={settings.provider_id ?? ""}
              onChange={(event) => {
                const providerId = event.target.value || null;
                const firstModel = models.find(
                  (model) => model.provider_id === providerId,
                );
                onChange({
                  ...settings,
                  provider_id: providerId,
                  model_id: firstModel?.model_id ?? null,
                });
              }}
              disabled={disabled}
            >
              <option value="">Automatic routing</option>
              {providers.map((provider) => (
                <option value={provider} key={provider}>
                  {provider}
                </option>
              ))}
            </select>
          </label>
          <label className="chat-runtime-field">
            <span>Model</span>
            <select
              aria-label="Model"
              value={selectedModel}
              onChange={(event) => {
                const model = visibleModels.find(
                  (item) => modelKey(item.provider_id, item.model_id) === event.target.value,
                );
                onChange({
                  ...settings,
                  provider_id: model?.provider_id ?? null,
                  model_id: model?.model_id ?? null,
                });
              }}
              disabled={disabled || models.length === 0}
            >
              <option value="">Automatic routing</option>
              {visibleModels.map((model) => (
                <option
                  value={modelKey(model.provider_id, model.model_id)}
                  key={modelKey(model.provider_id, model.model_id)}
                >
                  {model.display_name} · {model.provider_id}
                </option>
              ))}
            </select>
          </label>
          <label className="chat-runtime-field">
            <span>Quality</span>
            <select
              aria-label="Quality"
              value={settings.quality}
              onChange={(event) =>
                onChange({
                  ...settings,
                  quality: event.target.value as ChatRuntimeSettingsInput["quality"],
                })
              }
              disabled={disabled}
            >
              <option value="fast">Fast</option>
              <option value="balanced">Balanced</option>
              <option value="deep">Deep</option>
            </select>
          </label>
        </div>
      )}
    </div>
  );
}
