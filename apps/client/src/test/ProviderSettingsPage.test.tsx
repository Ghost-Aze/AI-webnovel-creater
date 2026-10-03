import { userEvent } from "@testing-library/user-event";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

const listProvidersMock = vi.hoisted(() => vi.fn());
const listModelsMock = vi.hoisted(() => vi.fn());
const getCredentialStoreStatusMock = vi.hoisted(() => vi.fn());
const configureProviderMock = vi.hoisted(() => vi.fn());
const removeProviderMock = vi.hoisted(() => vi.fn());
const getProviderSettingsMock = vi.hoisted(() => vi.fn());
const updateProviderMock = vi.hoisted(() => vi.fn());
const testProviderMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/commands", () => ({
  listProviders: listProvidersMock,
  listModels: listModelsMock,
  getCredentialStoreStatus: getCredentialStoreStatusMock,
  configureProvider: configureProviderMock,
  removeProvider: removeProviderMock,
  getProviderSettings: getProviderSettingsMock,
  updateProvider: updateProviderMock,
  testProvider: testProviderMock,
}));

import { ProviderSettingsPage } from "../features/providers/ProviderSettingsPage";

const provider = {
  id: "openrouter",
  display_name: "OpenRouter",
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

function renderProviders() {
  return render(
    <MemoryRouter initialEntries={["/settings/providers"]}>
      <Routes>
        <Route path="/settings/providers" element={<ProviderSettingsPage />} />
      </Routes>
    </MemoryRouter>,
  );
}

describe("ProviderSettingsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listProvidersMock.mockResolvedValue([]);
    listModelsMock.mockResolvedValue([]);
    getCredentialStoreStatusMock.mockResolvedValue({
      kind: "ephemeral",
      persistent: false,
      available: true,
    });
    configureProviderMock.mockResolvedValue({
      descriptor: provider,
      models: [model],
    });
    removeProviderMock.mockResolvedValue(provider);
    getProviderSettingsMock.mockResolvedValue({
      descriptor: provider,
      base_url: "https://openrouter.ai/api/v1",
      credential_id: "openrouter-primary",
      models: [model],
    });
    updateProviderMock.mockResolvedValue({
      descriptor: provider,
      models: [model],
    });
    testProviderMock.mockResolvedValue({
      provider_id: provider.id,
      model_id: model.model_id,
      message: "Connection successful.",
    });
  });

  it("shows the explicit session-only credential status", async () => {
    renderProviders();

    expect(await screen.findByTestId("provider-catalog")).toBeInTheDocument();
    expect(
      screen.getByText(
        "Session-only credential storage is active on this build.",
      ),
    ).toBeInTheDocument();
  });

  it("shows persistent platform storage when the native adapter is available", async () => {
    getCredentialStoreStatusMock.mockResolvedValueOnce({
      kind: "platform_secure",
      persistent: true,
      available: true,
    });
    renderProviders();

    expect(await screen.findByTestId("provider-catalog")).toBeInTheDocument();
    expect(
      screen.getByText("Credentials use platform secure storage."),
    ).toBeInTheDocument();
  });

  it("renders known providers and opens an API-key-only connection form", async () => {
    const user = userEvent.setup();
    renderProviders();

    expect(await screen.findByTestId("provider-catalog")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "OpenAI" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Anthropic" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "OpenRouter" })).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "OpenRouter" }));
    expect(
      await screen.findByRole("heading", { name: "OpenRouter" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("OpenRouter API key")).toBeInTheDocument();
    expect(screen.queryByLabelText("Base URL")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Credential ID")).not.toBeInTheDocument();
  });

  it("configures OpenRouter from its preset and only sends the entered API key", async () => {
    const user = userEvent.setup();
    renderProviders();

    await screen.findByTestId("provider-catalog");
    await user.click(screen.getByRole("button", { name: "OpenRouter" }));
    await user.type(screen.getByLabelText("OpenRouter API key"), "secret-token");
    await user.click(screen.getByRole("button", { name: "Connect OpenRouter" }));

    await waitFor(() => expect(configureProviderMock).toHaveBeenCalledTimes(1));
    expect(configureProviderMock).toHaveBeenCalledWith(
      expect.objectContaining({
        descriptor: provider,
        base_url: "https://openrouter.ai/api/v1",
        credential_id: "openrouter-primary",
        credential_value: "secret-token",
        models: expect.arrayContaining([model]),
      }),
    );
  });

  it("shows preset models before connection and keeps model choice in chat runtime", async () => {
    const user = userEvent.setup();
    renderProviders();

    await screen.findByTestId("provider-catalog");
    await user.click(screen.getByRole("button", { name: "OpenRouter" }));
    expect(await screen.findByText("DeepSeek V4 Flash 0731")).toBeInTheDocument();
    expect(
      screen.getByText("Choose the model from the chat composer."),
    ).toBeInTheDocument();
  });

  it("shows a configured provider without exposing its stored key", async () => {
    const user = userEvent.setup();
    listProvidersMock.mockResolvedValueOnce([provider]);
    listModelsMock.mockResolvedValueOnce([model]);
    renderProviders();

    await screen.findByRole("button", { name: "OpenRouter" });
    expect(await screen.findByText("API key saved securely.")).toBeInTheDocument();
    expect(screen.getByLabelText("OpenRouter API key")).toHaveValue("");
    await user.type(screen.getByLabelText("OpenRouter API key"), "replacement");
    await user.click(screen.getByRole("button", { name: "Update OpenRouter API key" }));

    await waitFor(() =>
      expect(updateProviderMock).toHaveBeenCalledWith(
        expect.objectContaining({
          descriptor: provider,
          base_url: "https://openrouter.ai/api/v1",
          credential_id: "openrouter-primary",
          credential_value: "replacement",
          models: expect.arrayContaining([model]),
        }),
      ),
    );
  });

  it("removes a configured provider through the typed command", async () => {
    const user = userEvent.setup();
    listProvidersMock.mockResolvedValueOnce([provider]);
    listModelsMock.mockResolvedValueOnce([model]);
    renderProviders();

    await screen.findByText("API key saved securely.");
    await user.click(screen.getByRole("button", { name: "Remove OpenRouter" }));
    await waitFor(() =>
      expect(removeProviderMock).toHaveBeenCalledWith("openrouter"),
    );
  });

  it("tests a configured provider and renders the safe result", async () => {
    const user = userEvent.setup();
    listProvidersMock.mockResolvedValueOnce([provider]);
    listModelsMock.mockResolvedValueOnce([model]);
    renderProviders();

    await user.click(
      await screen.findByRole("button", {
        name: "Test OpenRouter connection",
      }),
    );
    await waitFor(() => expect(testProviderMock).toHaveBeenCalledWith("openrouter"));
    expect(await screen.findByText("Connection successful.")).toBeInTheDocument();
  });

  it("keeps provider metadata visible when secure credential storage is unavailable", async () => {
    listProvidersMock.mockResolvedValueOnce([provider]);
    listModelsMock.mockResolvedValueOnce([model]);
    getCredentialStoreStatusMock.mockResolvedValueOnce({
      kind: "platform_secure",
      persistent: true,
      available: false,
    });

    renderProviders();

    expect(
      await screen.findByText(
        "Credentials are unavailable; provider metadata is still visible.",
      ),
    ).toBeInTheDocument();
    expect(await screen.findByText("https://openrouter.ai/api/v1")).toBeInTheDocument();
    expect(await screen.findByText("openrouter-primary")).toBeInTheDocument();
  });
});
