import { normalizeCommandError } from "../../lib/command-error";
import type {
  ChatEvent,
  ChatRuntimeSettings,
  ChatRuntimeSnapshot,
  ChatSendRequest,
  Conversation,
  ConversationMessage,
} from "../../types/conversation";
import type { CommandErrorAction, CommandErrorCode } from "../../lib/command-error";

export type ChatRuntimeStatus =
  | "idle"
  | "loading"
  | "ready"
  | "updating"
  | "sending"
  | "error";

export interface ChatRuntimeError {
  code: CommandErrorCode;
  message: string;
  retryable: boolean;
  action: CommandErrorAction;
}

export interface ChatRuntimeState {
  status: ChatRuntimeStatus;
  conversations: Conversation[];
  conversation: Conversation | null;
  messages: ConversationMessage[];
  settings: ChatRuntimeSettings;
  confirmed_settings: ChatRuntimeSettings;
  assistants: ChatRuntimeSnapshot["assistants"];
  models: ChatRuntimeSnapshot["models"];
  draft: string;
  streaming_text: string;
  error: ChatRuntimeError | null;
  last_request: ChatSendRequest | null;
}

export type ChatRuntimeAction =
  | { type: "load_succeeded"; snapshot: ChatRuntimeSnapshot }
  | { type: "draft_changed"; value: string }
  | { type: "settings_changed"; settings: ChatRuntimeSettings }
  | { type: "settings_confirmed"; settings: ChatRuntimeSettings }
  | { type: "settings_failed"; error: unknown }
  | { type: "send_started"; request: ChatSendRequest }
  | { type: "event"; event: ChatEvent };

export function createChatRuntimeState(
  snapshot: ChatRuntimeSnapshot,
  draft = "",
): ChatRuntimeState {
  return {
    status: "ready",
    conversations: snapshot.conversations,
    conversation: snapshot.conversation,
    messages: snapshot.messages,
    settings: snapshot.settings,
    confirmed_settings: snapshot.settings,
    assistants: snapshot.assistants,
    models: snapshot.models,
    draft,
    streaming_text: "",
    error: null,
    last_request: null,
  };
}

export function reduceChatRuntime(
  state: ChatRuntimeState,
  action: ChatRuntimeAction,
): ChatRuntimeState {
  switch (action.type) {
    case "load_succeeded":
      return {
        ...createChatRuntimeState(action.snapshot, state.draft),
        last_request: state.last_request,
      };
    case "draft_changed":
      return { ...state, draft: action.value };
    case "settings_changed":
      return {
        ...state,
        settings: action.settings,
        status: "updating",
        error: null,
      };
    case "settings_confirmed":
      return {
        ...state,
        settings: action.settings,
        confirmed_settings: action.settings,
        status: "ready",
        error: null,
      };
    case "settings_failed":
      return {
        ...state,
        settings: state.confirmed_settings,
        status: "error",
        error: safeError(action.error),
      };
    case "send_started":
      return {
        ...state,
        status: "sending",
        streaming_text: "",
        error: null,
        last_request: action.request,
      };
    case "event":
      return reduceEvent(state, action.event);
  }
}

function reduceEvent(state: ChatRuntimeState, event: ChatEvent): ChatRuntimeState {
  switch (event.type) {
    case "accepted":
      return { ...state, status: "sending", error: null };
    case "delta":
      return {
        ...state,
        status: "sending",
        streaming_text: state.streaming_text + event.text,
      };
    case "completed":
      return {
        ...state,
        status: "ready",
        conversation: event.result.conversation,
        messages: mergeMessages(state.messages, [
          event.result.user_message,
          event.result.assistant_message,
        ]),
        draft: "",
        streaming_text: "",
        error: null,
      };
    case "retryable_error":
      return {
        ...state,
        status: "error",
        error: safeError(event.error),
      };
    case "terminal_error":
      return {
        ...state,
        status: "error",
        error: safeError(event.error),
      };
    case "cancelled":
      return { ...state, status: "ready", streaming_text: "" };
  }
}

function mergeMessages(
  existing: ConversationMessage[],
  incoming: ConversationMessage[],
): ConversationMessage[] {
  const bySequence = new Map<string, ConversationMessage>();
  for (const message of [...existing, ...incoming]) {
    bySequence.set(`${message.conversation_id}:${message.sequence}`, message);
  }
  return [...bySequence.values()].sort((left, right) => left.sequence - right.sequence);
}

function safeError(error: unknown): ChatRuntimeError {
  const normalized = normalizeCommandError(error);
  return {
    code: normalized.code,
    message: normalized.message,
    retryable: normalized.retryable,
    action: normalized.action,
  };
}
