import { userEvent } from "@testing-library/user-event";
import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const loadChatRuntimeMock = vi.hoisted(() => vi.fn());
const createConversationMock = vi.hoisted(() => vi.fn());
const updateChatRuntimeMock = vi.hoisted(() => vi.fn());
const sendChatMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/commands", () => ({
  loadChatRuntime: loadChatRuntimeMock,
  createConversation: createConversationMock,
  updateChatRuntime: updateChatRuntimeMock,
  sendChat: sendChatMock,
}));

import { ChatWorkspace } from "../features/chat/ChatWorkspace";

const conversation = {
  id: "conversation-1",
  project_id: "project-1",
  chapter_id: null,
  kind: "developer_chat" as const,
  title: "Main chat",
  created_at: "2026-10-03T00:00:00Z",
  updated_at: "2026-10-03T00:00:00Z",
};

const model = {
  provider_id: "mock",
  model_id: "writer",
  display_name: "Mock Writer",
  context_window_tokens: 8192,
  default_output_tokens: 512,
  strengths: ["chat"],
  weaknesses: [],
  strategy: [],
  tier: "medium" as const,
  capabilities: {
    streaming: false,
    embeddings: false,
    tools: false,
    vision: false,
    structured_output: false,
    prompt_caching: false,
  },
};

const snapshot = {
  conversations: [conversation],
  conversation,
  messages: [],
  settings: {
    assistant_id: "general-assistant",
    provider_id: null,
    model_id: null,
    quality: "balanced" as const,
    temperature: null,
    updated_at: "2026-10-03T00:00:00Z",
  },
  assistants: [
    {
      id: "general-assistant",
      label: "General Assistant",
      description: "A balanced assistant.",
    },
    {
      id: "writing-coach",
      label: "Writing Coach",
      description: "A prose assistant.",
    },
  ],
  models: [model],
};

const result = {
  conversation,
  user_message: {
    id: "user-1",
    conversation_id: conversation.id,
    sequence: 1,
    role: "user" as const,
    content: "Keep this draft.",
    model: null,
    created_at: "2026-10-03T00:00:01Z",
  },
  assistant_message: {
    id: "assistant-1",
    conversation_id: conversation.id,
    sequence: 2,
    role: "assistant" as const,
    content: "A response.",
    model,
    created_at: "2026-10-03T00:00:02Z",
  },
  orchestration: { plan: { task: "developer_chat", quality: "balanced", steps: [] }, steps: [] },
};

describe("ChatWorkspace", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    loadChatRuntimeMock.mockResolvedValue(snapshot);
    createConversationMock.mockResolvedValue(conversation);
    updateChatRuntimeMock.mockResolvedValue(snapshot.settings);
    sendChatMock.mockResolvedValue(result);
  });

  it("renders the conversation sidebar, active model and one-click runtime menu", async () => {
    const user = userEvent.setup();
    render(
      <ChatWorkspace
        projectId="project-1"
        kind="developer_chat"
        title="Developer Chat"
        scopeLabel="Project scoped"
        task="developer_chat"
      />,
    );

    expect(await screen.findByRole("button", { name: "New chat" })).toBeInTheDocument();
    expect(screen.getAllByText("Main chat")).not.toHaveLength(0);
    expect(screen.getByLabelText("Selected model")).toHaveTextContent("Automatic routing");
    await user.click(screen.getByRole("button", { name: "Runtime settings" }));
    expect(screen.getByLabelText("Assistant")).toBeInTheDocument();
    expect(screen.getByLabelText("Quality")).toBeInTheDocument();
  });

  it("preserves a failed draft and retries without duplicating the user turn", async () => {
    const user = userEvent.setup();
    sendChatMock
      .mockRejectedValueOnce({ code: "provider_rate_limited" })
      .mockResolvedValueOnce(result);
    render(
      <ChatWorkspace
        projectId="project-1"
        kind="developer_chat"
        title="Developer Chat"
        scopeLabel="Project scoped"
        task="developer_chat"
      />,
    );

    const input = await screen.findByRole("textbox", { name: "Chat message" });
    await user.type(input, "Keep this draft.");
    await user.click(screen.getByRole("button", { name: "Send" }));
    expect(input).toHaveValue("Keep this draft.");
    await user.click(await screen.findByRole("button", { name: "Retry" }));

    await waitFor(() => expect(sendChatMock).toHaveBeenCalledTimes(2));
    expect(sendChatMock).toHaveBeenLastCalledWith(
      expect.objectContaining({ message: "Keep this draft.", retry_attempt: true }),
    );
    expect(screen.getAllByText("Keep this draft.")).toHaveLength(1);
  });

  it("disables sending for an archived scope", async () => {
    render(
      <ChatWorkspace
        projectId="project-1"
        kind="developer_chat"
        title="Developer Chat"
        scopeLabel="Archived project"
        task="developer_chat"
        disabled
      />,
    );

    const input = await screen.findByRole("textbox", { name: "Chat message" });
    expect(input).toBeDisabled();
    expect(screen.getByText(/cannot receive new messages/i)).toBeInTheDocument();
    expect(sendChatMock).not.toHaveBeenCalled();
  });
});
