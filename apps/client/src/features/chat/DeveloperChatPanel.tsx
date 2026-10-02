import { useCallback, useEffect, useState } from "react";

import { normalizeCommandError } from "../../lib/command-error";
import {
  createConversation,
  listConversationMessages,
  listConversations,
  sendDeveloperChat,
} from "../../lib/commands";
import type {
  Conversation,
  ConversationMessage,
  DeveloperChatSendRequest,
} from "../../types/conversation";

interface DeveloperChatPanelProps {
  projectId: string;
  disabled?: boolean;
}

const emptyCapabilities = {
  streaming: false,
  embeddings: false,
  tools: false,
  vision: false,
  structured_output: false,
  prompt_caching: false,
};

export function DeveloperChatPanel({
  projectId,
  disabled = false,
}: DeveloperChatPanelProps) {
  const [conversation, setConversation] = useState<Conversation | null>(null);
  const [messages, setMessages] = useState<ConversationMessage[]>([]);
  const [draft, setDraft] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [isSending, setIsSending] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const loadChat = useCallback(async () => {
    setIsLoading(true);
    setLoadError(null);
    setError(null);
    try {
      const existing = (
        await listConversations(projectId, { kind: "developer_chat" })
      )[0];
      const active =
        existing ??
        (disabled
          ? null
          : await createConversation({
              project_id: projectId,
              chapter_id: null,
              kind: "developer_chat",
              title: "Developer Chat",
            }));
      if (!active) {
        setConversation(null);
        setMessages([]);
        return;
      }
      setConversation(active);
      setMessages(await listConversationMessages(active.id));
    } catch (commandError) {
      setLoadError(normalizeCommandError(commandError).message);
    } finally {
      setIsLoading(false);
    }
  }, [disabled, projectId]);

  useEffect(() => {
    void loadChat();
  }, [loadChat]);

  async function handleSend() {
    const message = draft.trim();
    if (!conversation || !message || disabled) return;
    setIsSending(true);
    setError(null);
    try {
      const request: DeveloperChatSendRequest = {
        conversation_id: conversation.id,
        task: "main_writing",
        quality: "balanced",
        preferred_model: null,
        required_capabilities: emptyCapabilities,
        minimum_context_window_tokens: null,
        system_instructions:
          "You are the Developer Chat assistant. Respect the project's canon and keep all canonical changes explicit and reviewable.",
        character_ids: [],
        include_character_states: false,
        context_budget: {
          output_reserve_tokens: null,
          safety_margin_tokens: 256,
        },
        message,
        temperature: null,
      };
      const result = await sendDeveloperChat(request);
      setMessages((current) => [
        ...current,
        result.user_message,
        result.assistant_message,
      ]);
      setDraft("");
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsSending(false);
    }
  }

  return (
    <section
      className="workspace-primary developer-chat-panel"
      aria-labelledby="developer-chat-heading"
    >
      <div className="section-heading">
        <div>
          <p className="eyebrow">Project workspace</p>
          <h2 id="developer-chat-heading">Developer Chat</h2>
        </div>
        <span className="chat-scope-label">Project scoped</span>
      </div>
      {isLoading ? (
        <p className="loading-state" role="status">
          Loading chat…
        </p>
      ) : loadError ? (
        <div className="error-state" role="alert">
          <strong>Developer Chat could not be loaded.</strong>
          <span>{loadError}</span>
          <button
            className="button button-ghost"
            type="button"
            onClick={() => void loadChat()}
          >
            Try again
          </button>
        </div>
      ) : (
        <>
          <div className="developer-chat-messages" aria-live="polite">
            {messages.length === 0 && (
              <p className="empty-state developer-chat-empty">
                Ask about your world, characters or the next story decision.
              </p>
            )}
            {messages.map((message) => (
              <article
                className={`developer-chat-message message-${message.role}`}
                key={message.id}
              >
                <small>{message.role}</small>
                <p>{message.content}</p>
              </article>
            ))}
          </div>
          {disabled ? (
            <p className="chat-readonly-note">
              Archived projects can be viewed but cannot receive new messages.
            </p>
          ) : (
            <div className="developer-chat-composer">
              <textarea
                aria-label="Developer Chat message"
                rows={4}
                value={draft}
                onChange={(event) => setDraft(event.target.value)}
                disabled={isSending}
                placeholder="Ask about this project…"
              />
              <button
                className="button button-primary"
                type="button"
                onClick={() => void handleSend()}
                disabled={isSending || !draft.trim()}
              >
                {isSending ? "Sending…" : "Send"}
              </button>
            </div>
          )}
        </>
      )}
      {error && !loadError && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
    </section>
  );
}
