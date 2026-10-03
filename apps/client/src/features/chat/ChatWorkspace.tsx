import type { ConversationMessage, ConversationKind } from "../../types/conversation";
import type { ModelRef, ModelTask } from "../../types/provider";
import { ChatConversationSidebar } from "./ChatConversationSidebar";
import { ChatComposer } from "./ChatComposer";
import { ChatEmptyState } from "./ChatEmptyState";
import { ChatErrorBanner } from "./ChatErrorBanner";
import { ChatHeader } from "./ChatHeader";
import { ChatMessageList } from "./ChatMessageList";
import { ChatRuntimeMenu } from "./ChatRuntimeMenu";
import { useChatRuntime } from "./useChatRuntime";

export interface ChatWorkspaceProps {
  projectId: string;
  chapterId?: string | null;
  kind: ConversationKind;
  title: string;
  scopeLabel: string;
  disabled?: boolean;
  task: ModelTask;
  assistantInstructions?: (assistantId: string) => string;
  placeholder?: string;
  messageLabel?: string;
  onAssistantAction?: (message: ConversationMessage) => void;
  assistantActionLabel?: string;
}

export function ChatWorkspace(props: ChatWorkspaceProps) {
  const runtime = useChatRuntime(props);
  const { state } = runtime;
  const selectedModel: ModelRef | null =
    state.settings.provider_id && state.settings.model_id
      ? {
          provider_id: state.settings.provider_id,
          model_id: state.settings.model_id,
        }
      : null;
  const settingsInput = {
    assistant_id: state.settings.assistant_id,
    provider_id: state.settings.provider_id,
    model_id: state.settings.model_id,
    quality: state.settings.quality,
    temperature: state.settings.temperature,
  };
  const composerProps = {
    draft: state.draft,
    onDraftChange: runtime.setDraft,
    onSubmit: () => void runtime.send(),
    isSending: state.status === "sending",
    disabled: Boolean(props.disabled || state.status === "loading" || !state.conversation),
    runtimeMenu: (
      <ChatRuntimeMenu
        assistants={state.assistants}
        settings={settingsInput}
        models={state.models}
        onChange={(settings) => void runtime.updateSettings(settings)}
        disabled={props.disabled || !state.conversation}
      />
    ),
    placeholder: props.placeholder ?? "Ask me anything…",
    messageLabel: props.messageLabel ?? "Chat message",
  };

  if (state.status === "loading") {
    return (
      <section className="workspace-primary chat-workspace" aria-label={props.title}>
        <p className="loading-state" role="status">Loading chat…</p>
      </section>
    );
  }

  if (state.status === "error" && !state.conversation && state.messages.length === 0) {
    return (
      <section className="workspace-primary chat-workspace" aria-label={props.title}>
        <div className="error-state" role="alert">
          <strong>{props.title} could not be loaded.</strong>
          <span>{state.error?.message}</span>
          <button className="button button-ghost" type="button" onClick={() => void runtime.reload()}>
            Try again
          </button>
        </div>
      </section>
    );
  }

  return (
    <section className="workspace-primary chat-workspace" aria-label={props.title}>
      <ChatConversationSidebar
        conversations={state.conversations}
        activeConversationId={state.conversation?.id ?? null}
        onSelect={(id) => void runtime.selectConversation(id)}
        onCreate={() => void runtime.createConversation()}
        disabled={props.disabled}
      />
      <div className="chat-workspace-main">
        <ChatHeader
          scopeTitle={state.conversation?.title ?? props.title}
          selectedModel={selectedModel}
          models={state.models}
          onModelChange={(model) =>
            void runtime.updateSettings({
              ...settingsInput,
              provider_id: model?.provider_id ?? null,
              model_id: model?.model_id ?? null,
            })
          }
        />
        <p className="chat-scope-label">{props.scopeLabel}</p>
        {state.messages.length === 0 ? (
          <>
            <ChatEmptyState
              title="How can I help you today?"
              description="Ask about your world, characters or the next story decision."
              composerProps={composerProps}
            />
            {props.disabled && (
              <p className="chat-readonly-note">
                Archived scopes can be viewed but cannot receive new messages.
              </p>
            )}
          </>
        ) : (
          <div className="chat-active-surface">
            <ChatMessageList
              messages={state.messages}
              className="chat-message-scroll"
              onAssistantAction={props.onAssistantAction}
              assistantActionLabel={props.assistantActionLabel}
              disabled={props.disabled}
            />
            {props.disabled ? (
              <p className="chat-readonly-note">
                Archived scopes can be viewed but cannot receive new messages.
              </p>
            ) : (
              <div className="chat-composer-dock">
                <ChatComposer {...composerProps} />
              </div>
            )}
          </div>
        )}
        {state.error && (
          <ChatErrorBanner error={state.error} onRetry={() => void runtime.retry()} />
        )}
      </div>
    </section>
  );
}
