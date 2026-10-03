import type { ConversationMessage } from "../../types/conversation";

interface ChatMessageListProps {
  messages: ConversationMessage[];
  className?: string;
  onAssistantAction?: (message: ConversationMessage) => void;
  assistantActionLabel?: string;
  disabled?: boolean;
}

export function ChatMessageList({
  messages,
  className = "",
  onAssistantAction,
  assistantActionLabel = "Use response",
  disabled = false,
}: ChatMessageListProps) {
  return (
    <div
      className={`chat-message-list ${className}`.trim()}
      data-testid="chat-message-list"
      aria-live="polite"
    >
      {messages.map((message) => (
        <article className={`chat-message chat-message-${message.role}`} key={message.id}>
          <small>{message.role}</small>
          <p>{message.content}</p>
          {message.role === "assistant" && onAssistantAction && (
            <button
              className="button button-ghost button-small"
              type="button"
              onClick={() => onAssistantAction(message)}
              disabled={disabled || !message.content.trim()}
            >
              {assistantActionLabel}
            </button>
          )}
        </article>
      ))}
    </div>
  );
}
