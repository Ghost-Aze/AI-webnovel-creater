import { userEvent } from "@testing-library/user-event";
import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const listConversationsMock = vi.hoisted(() => vi.fn());
const createConversationMock = vi.hoisted(() => vi.fn());
const listMessagesMock = vi.hoisted(() => vi.fn());
const sendDeveloperChatMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/commands", () => ({
  listConversations: listConversationsMock,
  createConversation: createConversationMock,
  listConversationMessages: listMessagesMock,
  sendDeveloperChat: sendDeveloperChatMock,
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

describe("DeveloperChatPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listConversationsMock.mockResolvedValue([conversation]);
    createConversationMock.mockResolvedValue(conversation);
    listMessagesMock.mockResolvedValue([]);
    sendDeveloperChatMock.mockResolvedValue({
      conversation,
      user_message: {
        id: "message-user",
        conversation_id: conversation.id,
        sequence: 1,
        role: "user",
        content: "Make the opening more tense.",
        model: null,
        created_at: "2026-10-01T00:00:00Z",
      },
      assistant_message: {
        id: "message-assistant",
        conversation_id: conversation.id,
        sequence: 2,
        role: "assistant",
        content: "Start with a decision under pressure.",
        model: null,
        created_at: "2026-10-01T00:00:00Z",
      },
      orchestration: { steps: [] },
    });
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
      expect(sendDeveloperChatMock).toHaveBeenCalledWith(
        expect.objectContaining({
          conversation_id: conversation.id,
          message: "Make the opening more tense.",
        }),
      ),
    );
    expect(
      await screen.findByText("Start with a decision under pressure."),
    ).toBeInTheDocument();
  });

  it("creates the project's main chat when none exists", async () => {
    listConversationsMock.mockResolvedValueOnce([]);
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

  it("can retry a failed chat load", async () => {
    const user = userEvent.setup();
    listConversationsMock
      .mockRejectedValueOnce({ code: "storage" })
      .mockResolvedValueOnce([conversation]);

    render(<DeveloperChatPanel projectId="project-1" />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Developer Chat could not be loaded.",
    );
    await user.click(screen.getByRole("button", { name: "Try again" }));
    expect(
      await screen.findByRole("textbox", { name: "Developer Chat message" }),
    ).toBeInTheDocument();
    expect(listConversationsMock).toHaveBeenCalledTimes(2);
  });
});
