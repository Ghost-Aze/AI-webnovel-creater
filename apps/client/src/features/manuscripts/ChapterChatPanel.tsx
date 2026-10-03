import { useCallback, useEffect, useState } from "react";

import { normalizeCommandError, type CommandError } from "../../lib/command-error";
import {
  chapterChatSend,
  createConversation,
  createManuscriptProposal,
  listConversationMessages,
  listConversations,
} from "../../lib/commands";
import { ChatComposer, type ChatComposerProps } from "../chat/ChatComposer";
import { ChatEmptyState } from "../chat/ChatEmptyState";
import { ChatErrorBanner } from "../chat/ChatErrorBanner";
import { ChatHeader } from "../chat/ChatHeader";
import { ChatMessageList } from "../chat/ChatMessageList";
import type {
  Conversation,
  ConversationMessage,
  DeveloperChatSendRequest,
} from "../../types/conversation";
import type { ModelRef, QualityMode } from "../../types/provider";

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

const chapterAssistants = [
  {
    id: "chapter-editor",
    label: "Chapter editor",
    description: "Shape the chapter while preserving its current intent.",
    systemInstructions:
      "You are the Chapter Chat assistant. Discuss and propose manuscript changes without changing canonical data automatically.",
  },
  {
    id: "continuity-reviewer",
    label: "Continuity reviewer",
    description: "Check the chapter against the project's established canon.",
    systemInstructions:
      "You are the Chapter Chat continuity reviewer. Check the current chapter against the project's canon, then discuss and propose manuscript changes without changing canonical data automatically.",
  },
  {
    id: "scene-coach",
    label: "Scene coach",
    description: "Improve tension, pacing and scene-level choices.",
    systemInstructions:
      "You are the Chapter Chat scene coach. Give concrete guidance on tension, pacing and scene craft without changing canonical data automatically.",
  },
] as const;

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
  const [loadError, setLoadError] = useState<string | null>(null);
  const [error, setError] = useState<CommandError | null>(null);
  const [failedMessage, setFailedMessage] = useState<string | null>(null);
  const [proposalId, setProposalId] = useState<string | null>(null);
  const [selectedAssistantId, setSelectedAssistantId] = useState<string>(
    chapterAssistants[0].id,
  );
  const [selectedModel, setSelectedModel] = useState<ModelRef | null>(null);
  const [quality, setQuality] = useState<QualityMode>("balanced");

  const loadChat = useCallback(async () => {
    setIsLoading(true);
    setLoadError(null);
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
      setLoadError(normalizeCommandError(commandError).message);
    } finally {
      setIsLoading(false);
    }
  }, [chapterId, disabled, projectId]);

  useEffect(() => {
    void loadChat();
  }, [loadChat]);

  async function handleSend(retryAttempt = false) {
    const message = (retryAttempt ? failedMessage ?? draft : draft).trim();
    if (!conversation || !message || disabled) return;
    setIsSending(true);
    setError(null);
    try {
      const assistant =
        chapterAssistants.find(
          (preset) => preset.id === selectedAssistantId,
        ) ?? chapterAssistants[0];
      const request: DeveloperChatSendRequest = {
        conversation_id: conversation.id,
        task: "developer_chat",
        quality,
        preferred_model: selectedModel,
        required_capabilities: emptyCapabilities,
        minimum_context_window_tokens: null,
        system_instructions: assistant.systemInstructions,
        character_ids: [],
        include_character_states: false,
        context_budget: {
          output_reserve_tokens: null,
          safety_margin_tokens: 256,
        },
        message,
        temperature: null,
        retry_attempt: retryAttempt,
      };
      const result = await chapterChatSend(chapterId, request);
      setMessages((current) => [
        ...current,
        result.user_message,
        result.assistant_message,
      ]);
      setDraft("");
      setFailedMessage(null);
    } catch (commandError) {
      const normalized = normalizeCommandError(commandError);
      setError(normalized);
      setFailedMessage(normalized.retryable ? message : null);
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
      setError(normalizeCommandError(commandError));
    }
  }

  const composerProps: ChatComposerProps = {
    draft,
    onDraftChange: setDraft,
    onSubmit: () => void handleSend(),
    isSending,
    disabled: disabled || isLoading,
    assistants: chapterAssistants,
    selectedAssistantId,
    onAssistantChange: setSelectedAssistantId,
    selectedModel,
    onModelChange: setSelectedModel,
    quality,
    onQualityChange: setQuality,
    placeholder: "Ask about this chapter…",
    messageLabel: "Chapter Chat message",
  };

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
      ) : loadError ? (
        <div className="error-state" role="alert">
          <strong>Chapter Chat could not be loaded.</strong>
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
        messages.length === 0 ? (
          <>
            <ChatEmptyState
              title="How can I help you today?"
              description="Ask for a scene idea or revision."
              composerProps={composerProps}
            />
            {error && (
              <ChatErrorBanner
                error={error}
                onRetry={() => void handleSend(true)}
              />
            )}
          </>
        ) : (
          <div className="chat-active-surface">
            <ChatHeader
              scopeTitle="Chapter Chat"
              selectedModel={selectedModel}
            />
            <ChatMessageList
              messages={messages}
              onAssistantAction={(message) => void handlePropose(message.content)}
              assistantActionLabel="Propose revision"
              disabled={disabled}
            />
            {disabled ? (
              <p className="chat-readonly-note">
                Archived chapters can be viewed but cannot receive new messages.
              </p>
            ) : (
              <div className="chat-composer-dock">
                <ChatComposer {...composerProps} />
              </div>
            )}
            {error && (
              <ChatErrorBanner
                error={error}
                onRetry={() => void handleSend(true)}
              />
            )}
          </div>
        )
      )}
    </section>
  );
}
