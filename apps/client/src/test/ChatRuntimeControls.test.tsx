import { userEvent } from "@testing-library/user-event";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

const listModelsMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/commands", () => ({
  listModels: listModelsMock,
}));

import { ChatRuntimeControls } from "../features/chat/ChatRuntimeControls";

const assistants = [
  {
    id: "general",
    label: "General assistant",
    description: "Open-ended project discussion.",
    systemInstructions: "Be a helpful story assistant.",
  },
  {
    id: "continuity",
    label: "Continuity reviewer",
    description: "Look for contradictions before suggesting changes.",
    systemInstructions: "Review continuity carefully.",
  },
] as const;

const models = [
  {
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
  },
  {
    provider_id: "openai",
    model_id: "gpt-4o-mini",
    display_name: "GPT-4o mini",
    context_window_tokens: 128000,
    default_output_tokens: 4096,
    strengths: [],
    weaknesses: [],
    strategy: [],
    tier: "small" as const,
    capabilities: {
      streaming: true,
      embeddings: false,
      tools: false,
      vision: false,
      structured_output: false,
      prompt_caching: false,
    },
  },
];

describe("ChatRuntimeControls", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listModelsMock.mockResolvedValue(models);
  });

  it("loads assistant, provider/model and quality choices into typed callbacks", async () => {
    const user = userEvent.setup();
    const onAssistantChange = vi.fn();
    const onModelChange = vi.fn();
    const onQualityChange = vi.fn();

    render(
      <ChatRuntimeControls
        assistants={assistants}
        selectedAssistantId="general"
        onAssistantChange={onAssistantChange}
        selectedModel={null}
        onModelChange={onModelChange}
        quality="balanced"
        onQualityChange={onQualityChange}
      />,
    );

    expect(await screen.findByLabelText("Model")).toBeInTheDocument();
    await user.selectOptions(
      screen.getByLabelText("Assistant"),
      "continuity",
    );
    await user.selectOptions(screen.getByLabelText("Provider"), "openrouter");
    await user.selectOptions(
      screen.getByLabelText("Model"),
      "openrouter:deepseek/deepseek-v4-flash-0731",
    );
    await user.selectOptions(screen.getByLabelText("Quality"), "deep");

    expect(onAssistantChange).toHaveBeenCalledWith("continuity");
    expect(onModelChange).toHaveBeenCalledWith({
      provider_id: "openrouter",
      model_id: "deepseek/deepseek-v4-flash-0731",
    });
    expect(onQualityChange).toHaveBeenCalledWith("deep");
  });

  it("keeps automatic routing available when no provider models are configured", async () => {
    listModelsMock.mockResolvedValueOnce([]);

    render(
      <MemoryRouter>
        <ChatRuntimeControls
          assistants={assistants}
          selectedAssistantId="general"
          onAssistantChange={vi.fn()}
          selectedModel={null}
          onModelChange={vi.fn()}
          quality="balanced"
          onQualityChange={vi.fn()}
        />
      </MemoryRouter>,
    );

    await waitFor(() =>
      expect(
        screen.getByText("No configured models. Automatic routing will be used."),
      ).toBeInTheDocument(),
    );
    expect(screen.getByRole("link", { name: "Configure providers" })).toHaveAttribute(
      "href",
      "/settings/providers",
    );
  });
});
