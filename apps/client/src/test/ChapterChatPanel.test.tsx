import { userEvent } from "@testing-library/user-event";
import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const listConversationsMock = vi.hoisted(() => vi.fn());
const listMessagesMock = vi.hoisted(() => vi.fn());
const chapterChatSendMock = vi.hoisted(() => vi.fn());
const createProposalMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/commands", () => ({
  listConversations: listConversationsMock,
  listConversationMessages: listMessagesMock,
  chapterChatSend: chapterChatSendMock,
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

describe("ChapterChatPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listConversationsMock.mockResolvedValue([conversation]);
    listMessagesMock.mockResolvedValue([]);
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
    await user.type(input, "Make it tenser.");
    await user.click(screen.getByRole("button", { name: "Send" }));
    await waitFor(() =>
      expect(chapterChatSendMock).toHaveBeenCalledWith(
        "chapter-1",
        expect.objectContaining({
          conversation_id: "conversation-1",
          message: "Make it tenser.",
        }),
      ),
    );

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
});
