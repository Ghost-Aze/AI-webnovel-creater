import { userEvent } from "@testing-library/user-event";
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ChatComposer } from "../features/chat/ChatComposer";
import { ChatEmptyState } from "../features/chat/ChatEmptyState";
import { ChatErrorBanner } from "../features/chat/ChatErrorBanner";
import { ChatHeader } from "../features/chat/ChatHeader";
import { ChatMessageList } from "../features/chat/ChatMessageList";
import type { CommandError } from "../lib/command-error";

const assistant = {
  id: "general-assistant",
  label: "General assistant",
  description: "Open-ended project discussion.",
  systemInstructions: "Stay within canon.",
} as const;

const composerProps = {
  draft: "",
  onDraftChange: vi.fn(),
  onSubmit: vi.fn(),
  isSending: false,
  assistants: [assistant],
  selectedAssistantId: assistant.id,
  onAssistantChange: vi.fn(),
  selectedModel: null,
  onModelChange: vi.fn(),
  quality: "balanced" as const,
  onQualityChange: vi.fn(),
};

describe("shared chat surface", () => {
  it("centers the empty state composer and keeps the welcome prompt visible", () => {
    render(
      <ChatEmptyState
        title="How can I help you today?"
        description="Shape the next story decision."
        composerProps={composerProps}
      />,
    );

    expect(screen.getByTestId("chat-empty-state")).toHaveClass(
      "chat-empty-state",
    );
    expect(screen.getByText("How can I help you today?")).toBeInTheDocument();
    expect(
      screen.getByRole("textbox", { name: "Chat message" }),
    ).toBeInTheDocument();
  });

  it("renders the selected model in the active header and exposes a scroll region", () => {
    render(
      <>
        <ChatHeader
          scopeTitle="Developer Chat"
          selectedModel={{
            provider_id: "openrouter",
            model_id: "deepseek/deepseek-v4-flash-0731",
          }}
          modelLabel="DeepSeek V4 Flash 0731"
        />
        <ChatMessageList
          messages={[
            {
              id: "message-1",
              conversation_id: "conversation-1",
              sequence: 1,
              role: "user",
              content: "Hello",
              model: null,
              created_at: "2026-10-01T00:00:00Z",
            },
          ]}
        />
      </>,
    );

    expect(screen.getByText("DeepSeek V4 Flash 0731")).toBeInTheDocument();
    expect(screen.getByTestId("chat-message-list")).toHaveClass(
      "chat-message-list",
    );
    expect(screen.getByTestId("chat-message-list")).toHaveClass(
      "chat-message-scroll",
    );
    expect(screen.getByText("Hello")).toBeInTheDocument();
  });

  it("disables duplicate submission while the composer is pending", async () => {
    const user = userEvent.setup();
    render(<ChatComposer {...composerProps} isSending />);

    const input = screen.getByRole("textbox", { name: "Chat message" });
    expect(input).toBeDisabled();
    expect(screen.getByRole("button", { name: "Sending" })).toBeDisabled();
    await user.click(screen.getByRole("button", { name: "Sending" }));
    expect(composerProps.onSubmit).not.toHaveBeenCalled();
  });

  it("offers only the safe recovery action encoded by the command error", () => {
    const retryError = new (class extends Error {
      code = "provider_rate_limited" as const;
      retryable = true;
      action = "retry" as const;
    })("The provider rate limit was reached.") as CommandError;
    render(<ChatErrorBanner error={retryError} onRetry={vi.fn()} />);

    expect(screen.getByRole("alert")).toHaveTextContent("rate limit");
    expect(screen.getByRole("button", { name: "Retry" })).toBeInTheDocument();
    expect(
      screen.queryByRole("link", { name: "Open Provider settings" }),
    ).not.toBeInTheDocument();
  });
});
