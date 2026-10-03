import { useCallback, useEffect, useReducer } from "react";

import { normalizeCommandError } from "../../lib/command-error";
import {
  createConversation,
  loadChatRuntime as loadChatRuntimeCommand,
  sendChat,
  updateChatRuntime,
} from "../../lib/commands";
import type {
  ChatRuntimeSettings,
  ChatRuntimeSettingsInput,
  ChatRuntimeSnapshot,
  ChatSendRequest,
} from "../../types/conversation";
import {
  createChatRuntimeState,
  reduceChatRuntime,
  type ChatRuntimeAction,
  type ChatRuntimeState,
} from "./chat-runtime";
import type { ChatWorkspaceProps } from "./ChatWorkspace";

const emptySnapshot: ChatRuntimeSnapshot = {
  conversations: [],
  conversation: null,
  messages: [],
  settings: {
    assistant_id: "general-assistant",
    provider_id: null,
    model_id: null,
    quality: "balanced",
    temperature: null,
    updated_at: "",
  },
  assistants: [],
  models: [],
};

function initialState(): ChatRuntimeState {
  return { ...createChatRuntimeState(emptySnapshot), status: "loading" };
}

function inputFromSettings(settings: ChatRuntimeSettings): ChatRuntimeSettingsInput {
  return {
    assistant_id: settings.assistant_id,
    provider_id: settings.provider_id,
    model_id: settings.model_id,
    quality: settings.quality,
    temperature: settings.temperature,
  };
}

function scopeRequest(
  projectId: string,
  chapterId: string | null | undefined,
  kind: ChatWorkspaceProps["kind"],
  conversationId: string | null = null,
) {
  return {
    project_id: projectId,
    chapter_id: chapterId ?? null,
    conversation_id: conversationId,
    kind,
  };
}

export function useChatRuntime(props: ChatWorkspaceProps) {
  const {
    assistantInstructions,
    chapterId,
    disabled = false,
    kind,
    projectId,
    task,
    title,
  } = props;
  const [state, dispatch] = useReducer(
    (current: ChatRuntimeState, action: ChatRuntimeAction) =>
      reduceChatRuntime(current, action),
    undefined,
    initialState,
  );

  const load = useCallback(async () => {
    try {
      const snapshot = await loadChatRuntimeCommand({
        project_id: projectId,
        chapter_id: chapterId ?? null,
        conversation_id: null,
        kind,
      });
      if (snapshot.conversation || disabled) {
        dispatch({ type: "load_succeeded", snapshot });
        return;
      }
      const conversation = await createConversation({
        project_id: projectId,
        chapter_id: chapterId ?? null,
        kind,
        title,
      });
      dispatch({
        type: "load_succeeded",
        snapshot: {
          ...snapshot,
          conversations: [conversation],
          conversation,
        },
      });
    } catch (error) {
      dispatch({ type: "load_failed", error });
    }
  }, [chapterId, disabled, kind, projectId, title]);

  useEffect(() => {
    void load();
  }, [load]);

  const submit = useCallback(
    async (request: ChatSendRequest) => {
      dispatch({ type: "send_started", request });
      try {
        const result = await sendChat(request);
        dispatch({ type: "event", event: { type: "completed", result } });
      } catch (error) {
        const normalized = normalizeCommandError(error);
        dispatch({
          type: "event",
          event: {
            type: normalized.retryable ? "retryable_error" : "terminal_error",
            error: normalized,
          },
        });
      }
    },
    [],
  );

  const send = useCallback(async () => {
    const conversation = state.conversation;
    const message = state.draft.trim();
    if (!conversation || !message || disabled || state.status === "sending") return;
    const assistant = state.assistants.find(
      (item) => item.id === state.settings.assistant_id,
    );
    await submit({
      conversation_id: conversation.id,
      task,
      runtime: inputFromSettings(state.settings),
      system_instructions:
        assistantInstructions?.(state.settings.assistant_id) ??
        `You are the ${assistant?.label ?? "General Assistant"}. Respect the project's canon.`,
      character_ids: [],
      include_character_states: false,
      context_budget: { output_reserve_tokens: null, safety_margin_tokens: 256 },
      message,
      retry_attempt: false,
    });
  }, [assistantInstructions, disabled, state, submit, task]);

  const retry = useCallback(async () => {
    if (!state.last_request || disabled) return;
    await submit({ ...state.last_request, retry_attempt: true });
  }, [disabled, state.last_request, submit]);

  const updateSettings = useCallback(
    async (input: ChatRuntimeSettingsInput) => {
      if (!state.conversation || disabled) return;
      const optimistic: ChatRuntimeSettings = {
        ...input,
        updated_at: state.settings.updated_at,
      };
      dispatch({ type: "settings_changed", settings: optimistic });
      try {
        const confirmed = await updateChatRuntime(state.conversation.id, input);
        dispatch({ type: "settings_confirmed", settings: confirmed });
      } catch (error) {
        dispatch({ type: "settings_failed", error });
      }
    },
    [disabled, state.conversation, state.settings.updated_at],
  );

  const selectConversation = useCallback(
    async (conversationId: string) => {
      try {
        const snapshot = await loadChatRuntimeCommand(
          scopeRequest(projectId, chapterId, kind, conversationId),
        );
        dispatch({ type: "load_succeeded", snapshot });
      } catch (error) {
        dispatch({ type: "load_failed", error });
      }
    },
    [chapterId, kind, projectId],
  );

  const create = useCallback(async () => {
    if (disabled) return;
    try {
      const conversation = await createConversation({
        project_id: projectId,
        chapter_id: chapterId ?? null,
        kind,
        title,
      });
      const snapshot = await loadChatRuntimeCommand(
        scopeRequest(projectId, chapterId, kind, conversation.id),
      );
      dispatch({ type: "load_succeeded", snapshot });
    } catch (error) {
      dispatch({ type: "load_failed", error });
    }
  }, [chapterId, disabled, kind, projectId, title]);

  const setDraft = useCallback((value: string) => {
    dispatch({ type: "draft_changed", value });
  }, []);

  return {
    state,
    send,
    retry,
    updateSettings,
    selectConversation,
    createConversation: create,
    setDraft,
    reload: load,
  };
}
