import type { Conversation } from "../../types/conversation";

interface ChatConversationSidebarProps {
  conversations: Conversation[];
  activeConversationId: string | null;
  onSelect: (conversationId: string) => void;
  onCreate: () => void;
  disabled?: boolean;
}

export function ChatConversationSidebar({
  conversations,
  activeConversationId,
  onSelect,
  onCreate,
  disabled = false,
}: ChatConversationSidebarProps) {
  return (
    <aside className="chat-conversation-sidebar" aria-label="Conversations">
      <div className="chat-sidebar-heading">
        <div>
          <p className="eyebrow">Workspace</p>
          <h3>Conversations</h3>
        </div>
        <button
          className="button button-ghost button-small"
          type="button"
          onClick={onCreate}
          disabled={disabled}
        >
          New chat
        </button>
      </div>
      {conversations.length === 0 ? (
        <p className="chat-sidebar-empty">No conversations yet.</p>
      ) : (
        <nav aria-label="Conversation list">
          {conversations.map((conversation) => (
            <button
              className={`chat-conversation-item ${
                conversation.id === activeConversationId ? "is-active" : ""
              }`.trim()}
              type="button"
              key={conversation.id}
              onClick={() => onSelect(conversation.id)}
              aria-current={
                conversation.id === activeConversationId ? "page" : undefined
              }
            >
              <strong>{conversation.title}</strong>
              <span>{conversation.kind.replaceAll("_", " ")}</span>
            </button>
          ))}
        </nav>
      )}
    </aside>
  );
}
