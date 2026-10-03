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

import { DeveloperChatPanel } from "../features/chat/DeveloperChatPanel";

const conversation = {
  id: "conversation-1",
  project_id: "project-1",
  chapter_id: null,
  kind: "developer_chat" as const,
  title: "Developer Chat",
  created_at: "2026-10-01T00:00:00Z",
  updated_at: "2026-10-01T00:00:00Z",
};

const model = {
  provider_id: "openrouter",
  model_id: "deepseek/deepseek-v4-flash-0731",
  display_name: "DeepSeek V4 Flash 0731",
  context_window_tokens: 1310720,
  default_output_tokens: 393216,
  strengths: [],
  weaknesses: [],
  strategy: [],
  tier: "medium" as const,
  capabilities: {
    streaming: true,
    embeddings: false,
    tools: false,
    vision: false,
    structured_output: false,
    prompt_caching: false,
  },
};

const settings = {
  assistant_id: "general-assistant",
  provider_id: null,
  model_id: null,
  quality: "balanced" as const,
  temperature: null,
  updated_at: "2026-10-01T00:00:00Z",
};

const snapshot = {
  conversations: [conversation],
  conversation,
  messages: [],
  settings,
  assistants: [
    { id: "general-assistant", label: "General Assistant", description: "A balanced assistant." },
    { id: "world-builder", label: "World Builder", description: "A canon assistant." },
    { id: "continuity-reviewer", label: "Continuity Reviewer", description: "A continuity assistant." },
    { id: "writing-coach", label: "Writing Coach", description: "A prose assistant." },
  ],
  models: [],
};

const result = {
  conversation,
  user_message: {
    id: "message-user",
    conversation_id: conversation.id,
    sequence: 1,
    role: "user" as const,
    content: "Make the opening more tense.",
    model: null,
    created_at: "2026-10-01T00:00:00Z",
  },
  assistant_message: {
    id: "message-assistant",
    conversation_id: conversation.id,
    sequence: 2,
    role: "assistant" as const,
    content: "Start with a decision under pressure.",
    model: null,
    created_at: "2026-10-01T00:00:00Z",
  },
  orchestration: { steps: [] },
};

describe("DeveloperChatPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    loadChatRuntimeMock.mockResolvedValue(snapshot);
    createConversationMock.mockResolvedValue(conversation);
    updateChatRuntimeMock.mockImplementation(async (_conversationId, input) => ({
      ...input,
      updated_at: settings.updated_at,
    }));
    sendChatMock.mockResolvedValue(result);
  });

  it("loads the project chat and sends a durable developer message", async () => {
    const user = userEvent.setup();
    render(<DeveloperChatPanel projectId="project-1" />);

    const input = await screen.findByRole("textbox", {
      name: "Developer Chat message",
    });
    await user.type(input, "Make the opening more tense.");
    await user.click(screen.getByRole("button", { name: "Send" }));

    await waitFor(() =>
      expect(sendChatMock).toHaveBeenCalledWith(
        expect.objectContaining({
          conversation_id: conversation.id,
          task: "developer_chat",
          message: "Make the opening more tense.",
          retry_attempt: false,
        }),
      ),
    );
    expect(
      await screen.findByText("Start with a decision under pressure."),
    ).toBeInTheDocument();
  });

  it("creates the project's main chat when none exists", async () => {
    loadChatRuntimeMock.mockResolvedValueOnce({
      ...snapshot,
      conversations: [],
      conversation: null,
    });
    const user = userEvent.setup();
    render(<DeveloperChatPanel projectId="project-1" />);

    expect(
      await screen.findByRole("textbox", { name: "Developer Chat message" }),
    ).toBeInTheDocument();
    expect(createConversationMock).toHaveBeenCalledWith({
      project_id: "project-1",
      chapter_id: null,
      kind: "developer_chat",
      title: "Developer Chat",
    });
    await user.click(screen.getByRole("button", { name: "Send" }));
  });

  it("sends the selected assistant, quality and model to the runtime boundary", async () => {
    const user = userEvent.setup();
    loadChatRuntimeMock.mockResolvedValueOnce({ ...snapshot, models: [model] });
    render(<DeveloperChatPanel projectId="project-1" />);

    await user.click(await screen.findByRole("button", { name: "Runtime settings" }));
    await user.selectOptions(screen.getByLabelText("Assistant"), "continuity-reviewer");
    await user.selectOptions(screen.getByLabelText("Provider"), "openrouter");
    await user.selectOptions(
      screen.getByLabelText("Model"),
      "openrouter:deepseek/deepseek-v4-flash-0731",
    );
    await user.selectOptions(screen.getByLabelText("Quality"), "deep");
    await user.type(
      await screen.findByRole("textbox", { name: "Developer Chat message" }),
      "Check the timeline.",
    );
    await user.click(screen.getByRole("button", { name: "Send" }));

    await waitFor(() =>
      expect(sendChatMock).toHaveBeenCalledWith(
        expect.objectContaining({
          runtime: expect.objectContaining({
            assistant_id: "continuity-reviewer",
            quality: "deep",
            provider_id: "openrouter",
            model_id: "deepseek/deepseek-v4-flash-0731",
          }),
          system_instructions: expect.stringContaining("continuity reviewer"),
        }),
      ),
    );
  });

  it("can retry a failed chat load", async () => {
    const user = userEvent.setup();
    loadChatRuntimeMock
      .mockRejectedValueOnce({ code: "storage" })
      .mockResolvedValueOnce(snapshot);

    render(<DeveloperChatPanel projectId="project-1" />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Developer Chat could not be loaded.",
    );
    await user.click(screen.getByRole("button", { name: "Try again" }));
    expect(
      await screen.findByRole("textbox", { name: "Developer Chat message" }),
    ).toBeInTheDocument();
    expect(loadChatRuntimeMock).toHaveBeenCalledTimes(2);
  });

  it("preserves the draft and retries a rate-limited attempt without duplication", async () => {
    const user = userEvent.setup();
    sendChatMock
      .mockRejectedValueOnce({ code: "provider_rate_limited" })
      .mockResolvedValueOnce(result);
    render(<DeveloperChatPanel projectId="project-1" />);

    const input = await screen.findByRole("textbox", {
      name: "Developer Chat message",
    });
    await user.type(input, "Try again later.");
    await user.click(screen.getByRole("button", { name: "Send" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "rate limit was reached",
    );
    expect(input).toHaveValue("Try again later.");
    await user.click(screen.getByRole("button", { name: "Retry" }));

    await waitFor(() =>
      expect(sendChatMock).toHaveBeenCalledWith(
        expect.objectContaining({
          message: "Try again later.",
          retry_attempt: true,
        }),
      ),
    );
  });

  it("links unauthorized chat failures to provider settings", async () => {
    const user = userEvent.setup();
    sendChatMock.mockRejectedValueOnce({ code: "provider_unauthorized" });
    render(<DeveloperChatPanel projectId="project-1" />);

    const input = await screen.findByRole("textbox", {
      name: "Developer Chat message",
    });
    await user.type(input, "Check credentials.");
    await user.click(screen.getByRole("button", { name: "Send" }));

    expect(
      await screen.findByRole("link", { name: "Open Provider settings" }),
    ).toHaveAttribute("href", "/settings/providers");
    expect(screen.queryByRole("button", { name: "Retry" })).not.toBeInTheDocument();
  });
});
