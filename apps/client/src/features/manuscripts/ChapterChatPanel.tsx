import { useCallback, useEffect, useState } from "react";

import { normalizeCommandError } from "../../lib/command-error";
import {
  chapterChatSend,
  createConversation,
  createManuscriptProposal,
  listConversationMessages,
  listConversations,
} from "../../lib/commands";
import type {
  Conversation,
  ConversationMessage,
  DeveloperChatSendRequest,
} from "../../types/conversation";

interface ChapterChatPanelProps {
  projectId: string;
  chapterId: string;
  currentRevision: number;
  disabled?: boolean;
  onProposalCreated?: () => void;
}

const emptyCapabilities = {
  streaming: false,
  embeddings: false,
  tools: false,
  vision: false,
  structured_output: false,
  prompt_caching: false,
};

export function ChapterChatPanel({
  projectId,
  chapterId,
  currentRevision,
  disabled = false,
  onProposalCreated,
}: ChapterChatPanelProps) {
  const [conversation, setConversation] = useState<Conversation | null>(null);
  const [messages, setMessages] = useState<ConversationMessage[]>([]);
  const [draft, setDraft] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [isSending, setIsSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [proposalId, setProposalId] = useState<string | null>(null);

  const loadChat = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const existing = (
        await listConversations(projectId, {
          kind: "chapter_chat",
        })
      ).find((item) => item.chapter_id === chapterId);
      const active =
        existing ??
        (disabled
          ? null
          : await createConversation({
              project_id: projectId,
              chapter_id: chapterId,
              kind: "chapter_chat",
              title: "Chapter Chat",
            }));
      if (!active) {
        setConversation(null);
        setMessages([]);
        return;
      }
      setConversation(active);
      setMessages(await listConversationMessages(active.id));
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsLoading(false);
    }
  }, [chapterId, disabled, projectId]);

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
          "You are the Chapter Chat assistant. Discuss and propose manuscript changes without changing canonical data automatically.",
        character_ids: [],
        include_character_states: false,
        context_budget: {
          output_reserve_tokens: null,
          safety_margin_tokens: 256,
        },
        message,
        temperature: null,
      };
      const result = await chapterChatSend(chapterId, request);
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

  async function handlePropose(content: string) {
    if (disabled) return;
    setError(null);
    try {
      const proposal = await createManuscriptProposal(projectId, {
        chapter_id: chapterId,
        base_revision: currentRevision,
        proposed_content: content,
        content_format: "plain_text",
        rationale: "Proposed from Chapter Chat",
        actor_type: "ai",
        actor_id: "chapter-chat",
      });
      setProposalId(proposal.id);
      onProposalCreated?.();
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    }
  }

  return (
    <section
      className="workspace-primary chapter-chat-panel"
      aria-labelledby="chapter-chat-heading"
    >
      <div className="section-heading">
        <div>
          <p className="eyebrow">Chapter workspace</p>
          <h2 id="chapter-chat-heading">Chapter Chat</h2>
        </div>
        {proposalId && <span className="saved-message">Proposal saved</span>}
      </div>
      {isLoading ? (
        <p className="loading-state">Loading chat…</p>
      ) : (
        <>
          <div className="chapter-chat-messages" aria-live="polite">
            {messages.length === 0 && (
              <p className="empty-state">Ask for a scene idea or revision.</p>
            )}
            {messages.map((message) => (
              <article
                className={`chapter-chat-message message-${message.role}`}
                key={message.id}
              >
                <small>{message.role}</small>
                <p>{message.content}</p>
                {message.role === "assistant" && message.content.trim() && (
                  <button
                    className="button button-ghost button-small"
                    type="button"
                    onClick={() => void handlePropose(message.content)}
                    disabled={disabled}
                  >
                    Propose revision
                  </button>
                )}
              </article>
            ))}
          </div>
          <div className="chapter-chat-composer">
            <textarea
              aria-label="Chapter Chat message"
              rows={3}
              value={draft}
              onChange={(event) => setDraft(event.target.value)}
              disabled={disabled || isSending}
              placeholder="Ask about this chapter…"
            />
            <button
              className="button button-primary"
              type="button"
              onClick={() => void handleSend()}
              disabled={disabled || isSending || !draft.trim()}
            >
              {isSending ? "Sending…" : "Send"}
            </button>
          </div>
        </>
      )}
      {error && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
    </section>
  );
}
