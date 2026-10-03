import type { ModelProfile, ModelRef } from "../../types/provider";

interface ChatHeaderProps {
  scopeTitle: string;
  selectedModel: ModelRef | null;
  modelLabel?: string | null;
  models?: ModelProfile[];
  onModelChange?: (model: ModelRef | null) => void;
  disabled?: boolean;
}

export function ChatHeader({
  scopeTitle,
  selectedModel,
  modelLabel,
  models = [],
  onModelChange,
  disabled = false,
}: ChatHeaderProps) {
  const label = modelLabel ||
    (selectedModel
      ? `${selectedModel.provider_id} · ${selectedModel.model_id}`
      : "Automatic routing");

  return (
    <header className="chat-header">
      <div>
        <p className="eyebrow">Conversation</p>
        <h2>{scopeTitle}</h2>
      </div>
      {onModelChange ? (
        <label className="chat-model-selector">
          <span className="sr-only">Selected model</span>
          <select
            aria-label="Selected model"
            value={selectedModel ? `${selectedModel.provider_id}:${selectedModel.model_id}` : ""}
            onChange={(event) => {
              const selected = models.find(
                (model) => `${model.provider_id}:${model.model_id}` === event.target.value,
              );
              onModelChange(
                selected
                  ? { provider_id: selected.provider_id, model_id: selected.model_id }
                  : null,
              );
            }}
            disabled={disabled}
          >
            <option value="">Automatic routing</option>
            {models.map((model) => (
              <option
                key={`${model.provider_id}:${model.model_id}`}
                value={`${model.provider_id}:${model.model_id}`}
              >
                {model.display_name}
              </option>
            ))}
          </select>
        </label>
      ) : (
        <span className="chat-model-badge" aria-label={`Selected model: ${label}`}>
          {label}
        </span>
      )}
    </header>
  );
}
