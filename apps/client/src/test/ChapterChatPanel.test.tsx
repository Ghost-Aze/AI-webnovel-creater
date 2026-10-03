import { userEvent } from "@testing-library/user-event";
import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const loadChatRuntimeMock = vi.hoisted(() => vi.fn());
const createConversationMock = vi.hoisted(() => vi.fn());
const updateChatRuntimeMock = vi.hoisted(() => vi.fn());
const sendChatMock = vi.hoisted(() => vi.fn());
const createProposalMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/commands", () => ({
  loadChatRuntime: loadChatRuntimeMock,
  createConversation: createConversationMock,
  updateChatRuntime: updateChatRuntimeMock,
  sendChat: sendChatMock,
  createManuscriptProposal: createProposalMock,
}));

import { ChapterChatPanel } from "../features/manuscripts/ChapterChatPanel";

const conversation = {
  id: "conversation-1",
  project_id: "project-1",
  chapter_id: "chapter-1",
  kind: "chapter_chat" as const,
  title: "Chapter Chat",
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
    content: "Make it tenser.",
    model: null,
    created_at: "2026-10-01T00:00:00Z",
  },
  assistant_message: {
    id: "message-assistant",
    conversation_id: conversation.id,
    sequence: 2,
    role: "assistant" as const,
    content: "Use shorter sentences.",
    model: null,
    created_at: "2026-10-01T00:00:00Z",
  },
  orchestration: { steps: [] },
};

describe("ChapterChatPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    loadChatRuntimeMock.mockResolvedValue(snapshot);
    createConversationMock.mockResolvedValue(conversation);
    updateChatRuntimeMock.mockImplementation(async (_conversationId, input) => ({
      ...input,
      updated_at: settings.updated_at,
    }));
    sendChatMock.mockResolvedValue(result);
    createProposalMock.mockResolvedValue({ id: "proposal-1" });
  });

  it("sends chapter-scoped chat and creates an explicit proposal", async () => {
    const user = userEvent.setup();
    render(
      <ChapterChatPanel
        projectId="project-1"
        chapterId="chapter-1"
        currentRevision={3}
      />,
    );

    const input = await screen.findByRole("textbox", {
      name: "Chapter Chat message",
    });
    expect(screen.getByTestId("chat-empty-state")).toBeInTheDocument();
    await user.type(input, "Make it tenser.");
    await user.click(screen.getByRole("button", { name: "Send" }));
    await waitFor(() =>
      expect(sendChatMock).toHaveBeenCalledWith(
        expect.objectContaining({
          conversation_id: "conversation-1",
          task: "developer_chat",
          message: "Make it tenser.",
          retry_attempt: false,
        }),
      ),
    );
    expect(screen.getByTestId("chat-message-list")).toBeInTheDocument();
    expect(document.querySelector(".chat-composer-dock")).toBeInTheDocument();

    await user.click(
      await screen.findByRole("button", { name: "Propose revision" }),
    );
    await waitFor(() =>
      expect(createProposalMock).toHaveBeenCalledWith(
        "project-1",
        expect.objectContaining({
          chapter_id: "chapter-1",
          base_revision: 3,
          proposed_content: "Use shorter sentences.",
        }),
      ),
    );
    expect(await screen.findByText("Proposal saved")).toBeInTheDocument();
  });

  it("can retry a failed chat load", async () => {
    const user = userEvent.setup();
    loadChatRuntimeMock
      .mockRejectedValueOnce({ code: "storage" })
      .mockResolvedValueOnce(snapshot);

    render(
      <ChapterChatPanel
        projectId="project-1"
        chapterId="chapter-1"
        currentRevision={3}
      />,
    );

    expect(await screen.findByRole("alert")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Try again" }));
    expect(
      await screen.findByRole("textbox", { name: "Chapter Chat message" }),
    ).toBeInTheDocument();
    expect(loadChatRuntimeMock).toHaveBeenCalledTimes(2);
  });

  it("sends selected chapter assistant settings without changing proposal flow", async () => {
    const user = userEvent.setup();
    loadChatRuntimeMock.mockResolvedValueOnce({ ...snapshot, models: [model] });
    render(
      <ChapterChatPanel
        projectId="project-1"
        chapterId="chapter-1"
        currentRevision={3}
      />,
    );

    await user.click(await screen.findByRole("button", { name: "Runtime settings" }));
    await user.selectOptions(screen.getByLabelText("Assistant"), "continuity-reviewer");
    await user.selectOptions(screen.getByLabelText("Provider"), "openrouter");
    await user.selectOptions(
      screen.getByLabelText("Model"),
      "openrouter:deepseek/deepseek-v4-flash-0731",
    );
    await user.selectOptions(screen.getByLabelText("Quality"), "fast");
    await user.type(
      await screen.findByRole("textbox", { name: "Chapter Chat message" }),
      "Check this scene.",
    );
    await user.click(screen.getByRole("button", { name: "Send" }));

    await waitFor(() =>
      expect(sendChatMock).toHaveBeenCalledWith(
        expect.objectContaining({
          runtime: expect.objectContaining({
            assistant_id: "continuity-reviewer",
            quality: "fast",
            provider_id: "openrouter",
            model_id: "deepseek/deepseek-v4-flash-0731",
          }),
          system_instructions: expect.stringContaining("continuity reviewer"),
        }),
      ),
    );
  });

  it("preserves the draft and retries a chapter provider failure", async () => {
    const user = userEvent.setup();
    sendChatMock
      .mockRejectedValueOnce({ code: "provider_timeout" })
      .mockResolvedValueOnce(result);
    render(
      <ChapterChatPanel
        projectId="project-1"
        chapterId="chapter-1"
        currentRevision={3}
      />,
    );

    const input = await screen.findByRole("textbox", {
      name: "Chapter Chat message",
    });
    await user.type(input, "Retry this chapter request.");
    await user.click(screen.getByRole("button", { name: "Send" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("timed out");
    expect(input).toHaveValue("Retry this chapter request.");
    await user.click(screen.getByRole("button", { name: "Retry" }));

    await waitFor(() =>
      expect(sendChatMock).toHaveBeenCalledWith(
        expect.objectContaining({
          message: "Retry this chapter request.",
          retry_attempt: true,
        }),
      ),
    );
  });
});
