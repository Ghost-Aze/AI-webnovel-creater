import type { ModelRef } from "../../types/provider";

interface ChatHeaderProps {
  scopeTitle: string;
  selectedModel: ModelRef | null;
  modelLabel?: string | null;
}

export function ChatHeader({
  scopeTitle,
  selectedModel,
  modelLabel,
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
      <span className="chat-model-badge" aria-label={`Selected model: ${label}`}>
        {label}
      </span>
    </header>
  );
}
