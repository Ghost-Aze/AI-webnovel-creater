import { ChatRuntimeControls, type ChatAssistantPreset } from "./ChatRuntimeControls";
import type { ModelRef, QualityMode } from "../../types/provider";

export interface ChatComposerProps {
  draft: string;
  onDraftChange: (value: string) => void;
  onSubmit: () => void;
  isSending: boolean;
  disabled?: boolean;
  assistants: readonly ChatAssistantPreset[];
  selectedAssistantId: string;
  onAssistantChange: (assistantId: string) => void;
  selectedModel: ModelRef | null;
  onModelChange: (model: ModelRef | null) => void;
  quality: QualityMode;
  onQualityChange: (quality: QualityMode) => void;
  placeholder?: string;
  messageLabel?: string;
}

export function ChatComposer({
  draft,
  onDraftChange,
  onSubmit,
  isSending,
  disabled = false,
  assistants,
  selectedAssistantId,
  onAssistantChange,
  selectedModel,
  onModelChange,
  quality,
  onQualityChange,
  placeholder = "Ask me anything…",
  messageLabel = "Chat message",
}: ChatComposerProps) {
  const isDisabled = disabled || isSending;

  return (
    <form
      className="chat-composer"
      onSubmit={(event) => {
        event.preventDefault();
        if (!isDisabled && draft.trim()) onSubmit();
      }}
    >
      <textarea
        aria-label={messageLabel}
        rows={3}
        value={draft}
        onChange={(event) => onDraftChange(event.target.value)}
        disabled={isDisabled}
        placeholder={placeholder}
      />
      <ChatRuntimeControls
        assistants={assistants}
        selectedAssistantId={selectedAssistantId}
        onAssistantChange={onAssistantChange}
        selectedModel={selectedModel}
        onModelChange={onModelChange}
        quality={quality}
        onQualityChange={onQualityChange}
        disabled={isDisabled}
      />
      <div className="chat-composer-actions">
        <span className="chat-composer-hint">Model and quality are adjustable per message.</span>
        <button
          className="button button-primary"
          type="submit"
          disabled={isDisabled || !draft.trim()}
        >
          {isSending ? "Sending" : "Send"}
        </button>
      </div>
    </form>
  );
}
