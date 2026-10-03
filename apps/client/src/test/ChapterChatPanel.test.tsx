import { userEvent } from "@testing-library/user-event";
import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const listConversationsMock = vi.hoisted(() => vi.fn());
const listMessagesMock = vi.hoisted(() => vi.fn());
const chapterChatSendMock = vi.hoisted(() => vi.fn());
const createProposalMock = vi.hoisted(() => vi.fn());
const listModelsMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/commands", () => ({
  listConversations: listConversationsMock,
  listConversationMessages: listMessagesMock,
  chapterChatSend: chapterChatSendMock,
  createManuscriptProposal: createProposalMock,
  listModels: listModelsMock,
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

describe("ChapterChatPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listConversationsMock.mockResolvedValue([conversation]);
    listMessagesMock.mockResolvedValue([]);
    listModelsMock.mockResolvedValue([]);
    chapterChatSendMock.mockResolvedValue({
      conversation,
      user_message: {
        id: "message-user",
        conversation_id: conversation.id,
        sequence: 1,
        role: "user",
        content: "Make it tenser.",
        model: null,
        created_at: "2026-10-01T00:00:00Z",
      },
      assistant_message: {
        id: "message-assistant",
        conversation_id: conversation.id,
        sequence: 2,
        role: "assistant",
        content: "Use shorter sentences.",
        model: null,
        created_at: "2026-10-01T00:00:00Z",
      },
      orchestration: { steps: [] },
    });
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
      expect(chapterChatSendMock).toHaveBeenCalledWith(
        "chapter-1",
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
    listConversationsMock
      .mockRejectedValueOnce({ code: "storage" })
      .mockResolvedValueOnce([conversation]);

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
    expect(listConversationsMock).toHaveBeenCalledTimes(2);
  });

  it("sends selected chapter assistant settings without changing proposal flow", async () => {
    const user = userEvent.setup();
    listModelsMock.mockResolvedValueOnce([model]);
    render(
      <ChapterChatPanel
        projectId="project-1"
        chapterId="chapter-1"
        currentRevision={3}
      />,
    );

    await user.selectOptions(
      await screen.findByLabelText("Assistant"),
      "continuity-reviewer",
    );
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
      expect(chapterChatSendMock).toHaveBeenCalledWith(
        "chapter-1",
        expect.objectContaining({
          quality: "fast",
          preferred_model: {
            provider_id: "openrouter",
            model_id: "deepseek/deepseek-v4-flash-0731",
          },
          system_instructions: expect.stringContaining(
            "continuity reviewer",
          ),
        }),
      ),
    );
  });

  it("preserves the draft and retries a chapter provider failure", async () => {
    const user = userEvent.setup();
    chapterChatSendMock.mockRejectedValueOnce({ code: "provider_timeout" });
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
      expect(chapterChatSendMock).toHaveBeenCalledWith(
        "chapter-1",
        expect.objectContaining({
          message: "Retry this chapter request.",
          retry_attempt: true,
        }),
      ),
    );
  });
});
