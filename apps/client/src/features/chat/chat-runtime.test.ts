import { describe, expect, it } from "vitest";

import type {
  ChatRuntimeSettings,
  ChatRuntimeSnapshot,
  ChatSendResult,
  ConversationMessage,
} from "../../types/conversation";
import {
  createChatRuntimeState,
  reduceChatRuntime,
} from "./chat-runtime";

const settings: ChatRuntimeSettings = {
  assistant_id: "general-assistant",
  provider_id: null,
  model_id: null,
  quality: "balanced",
  temperature: null,
  updated_at: "2026-10-03T00:00:00Z",
};

const userMessage: ConversationMessage = {
  id: "user-1",
  conversation_id: "conversation-1",
  sequence: 1,
  role: "user",
  content: "Keep this draft.",
  model: null,
  created_at: "2026-10-03T00:00:01Z",
};

const assistantMessage: ConversationMessage = {
  id: "assistant-1",
  conversation_id: "conversation-1",
  sequence: 2,
  role: "assistant",
  content: "A safe response.",
  model: { provider_id: "mock", model_id: "writer" },
  created_at: "2026-10-03T00:00:02Z",
};

const snapshot: ChatRuntimeSnapshot = {
  conversations: [],
  conversation: null,
  messages: [],
  settings,
  assistants: [
    {
      id: "general-assistant",
      label: "General Assistant",
      description: "A balanced assistant.",
    },
  ],
  models: [],
};

function result(): ChatSendResult {
  return {
    conversation: {
      id: "conversation-1",
      project_id: "project-1",
      chapter_id: null,
      kind: "developer_chat",
      title: "Chat",
      created_at: "2026-10-03T00:00:00Z",
      updated_at: "2026-10-03T00:00:02Z",
    },
    user_message: userMessage,
    assistant_message: assistantMessage,
    orchestration: { plan: { task: "developer_chat", quality: "balanced", steps: [] }, steps: [] },
  };
}

describe("chat runtime reducer", () => {
  it("preserves the draft and returns a safe retryable error", () => {
    const state = createChatRuntimeState(snapshot, "Keep this draft.");

    const next = reduceChatRuntime(state, {
      type: "event",
      event: {
        type: "retryable_error",
        error: { code: "provider_unavailable", details: { sql: "secret" } },
      },
    });

    expect(next.draft).toBe("Keep this draft.");
    expect(next.status).toBe("error");
    expect(next.error?.retryable).toBe(true);
    expect(next.error?.message).not.toContain("secret");
    expect(next.error?.message).not.toContain("sql");
  });

  it("does not append a duplicate user turn when retry completes", () => {
    const state = createChatRuntimeState(snapshot, "Keep this draft.");
    const withFailedTurn = { ...state, messages: [userMessage] };

    const next = reduceChatRuntime(withFailedTurn, {
      type: "event",
      event: { type: "completed", result: result() },
    });

    expect(next.messages).toEqual([userMessage, assistantMessage]);
    expect(next.messages.filter((message) => message.role === "user")).toHaveLength(1);
    expect(next.draft).toBe("");
  });

  it("restores the confirmed settings after an optimistic update fails", () => {
    const optimistic = reduceChatRuntime(
      createChatRuntimeState(snapshot),
      {
        type: "settings_changed",
        settings: {
          ...settings,
          assistant_id: "writing-coach",
          quality: "deep",
        },
      },
    );

    const next = reduceChatRuntime(optimistic, {
      type: "settings_failed",
      error: { code: "storage", details: { sql: "secret" } },
    });

    expect(next.settings).toEqual(settings);
    expect(next.confirmed_settings).toEqual(settings);
    expect(next.error?.message).not.toContain("secret");
  });

  it("accepts deltas without changing persisted messages", () => {
    const state = createChatRuntimeState(snapshot);
    const next = reduceChatRuntime(state, {
      type: "event",
      event: { type: "delta", conversation_id: "conversation-1", text: "partial" },
    });

    expect(next.streaming_text).toBe("partial");
    expect(next.messages).toEqual([]);
  });
});
