import { useCallback, useEffect, useState } from "react";

import { normalizeCommandError, type CommandError } from "../../lib/command-error";
import {
  createConversation,
  listConversationMessages,
  listConversations,
  sendDeveloperChat,
} from "../../lib/commands";
import { ChatComposer, type ChatComposerProps } from "./ChatComposer";
import { ChatEmptyState } from "./ChatEmptyState";
import { ChatErrorBanner } from "./ChatErrorBanner";
import { ChatHeader } from "./ChatHeader";
import { ChatMessageList } from "./ChatMessageList";
import type {
  Conversation,
  ConversationMessage,
  DeveloperChatSendRequest,
} from "../../types/conversation";
import type { ModelRef, QualityMode } from "../../types/provider";

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

const developerAssistants = [
  {
    id: "general-assistant",
    label: "General assistant",
    description: "Open-ended project discussion and next-step ideas.",
    systemInstructions:
      "You are the Developer Chat assistant. Respect the project's canon and keep all canonical changes explicit and reviewable.",
  },
  {
    id: "world-builder",
    label: "World builder",
    description: "Develop the world while preserving established facts.",
    systemInstructions:
      "You are the world-building assistant. Respect the project's canon, identify assumptions clearly, and keep all canonical changes explicit and reviewable.",
  },
  {
    id: "continuity-reviewer",
    label: "Continuity reviewer",
    description: "Look for contradictions before suggesting changes.",
    systemInstructions:
      "You are the continuity reviewer. Check the project's canon carefully, call out contradictions, and keep all canonical changes explicit and reviewable.",
  },
  {
    id: "writing-coach",
    label: "Writing coach",
    description: "Improve prose direction without silently rewriting canon.",
    systemInstructions:
      "You are the writing coach. Give concrete prose and scene guidance while respecting the project's canon and keeping all canonical changes explicit and reviewable.",
  },
] as const;

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
  const [error, setError] = useState<CommandError | null>(null);
  const [failedMessage, setFailedMessage] = useState<string | null>(null);
  const [selectedAssistantId, setSelectedAssistantId] = useState<string>(
    developerAssistants[0].id,
  );
  const [selectedModel, setSelectedModel] = useState<ModelRef | null>(null);
  const [quality, setQuality] = useState<QualityMode>("balanced");

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

  async function handleSend(retryAttempt = false) {
    const message = (retryAttempt ? failedMessage ?? draft : draft).trim();
    if (!conversation || !message || disabled) return;
    setIsSending(true);
    setError(null);
    try {
      const assistant =
        developerAssistants.find(
          (preset) => preset.id === selectedAssistantId,
        ) ?? developerAssistants[0];
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
      const result = await sendDeveloperChat(request);
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

  const composerProps: ChatComposerProps = {
    draft,
    onDraftChange: setDraft,
    onSubmit: () => void handleSend(),
    isSending,
    disabled: disabled || isLoading,
    assistants: developerAssistants,
    selectedAssistantId,
    onAssistantChange: setSelectedAssistantId,
    selectedModel,
    onModelChange: setSelectedModel,
    quality,
    onQualityChange: setQuality,
    placeholder: "Ask about this project…",
    messageLabel: "Developer Chat message",
  };

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
        messages.length === 0 ? (
          <>
            <ChatEmptyState
              title="How can I help you today?"
              description="Ask about your world, characters or the next story decision."
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
              scopeTitle="Developer Chat"
              selectedModel={selectedModel}
            />
            <ChatMessageList messages={messages} />
            {disabled ? (
              <p className="chat-readonly-note">
                Archived projects can be viewed but cannot receive new messages.
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
