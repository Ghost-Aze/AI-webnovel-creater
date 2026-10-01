import { userEvent } from "@testing-library/user-event";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

const listProvidersMock = vi.hoisted(() => vi.fn());
const listModelsMock = vi.hoisted(() => vi.fn());
const getCredentialStoreStatusMock = vi.hoisted(() => vi.fn());
const configureProviderMock = vi.hoisted(() => vi.fn());
const removeProviderMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/commands", () => ({
  listProviders: listProvidersMock,
  listModels: listModelsMock,
  getCredentialStoreStatus: getCredentialStoreStatusMock,
  configureProvider: configureProviderMock,
  removeProvider: removeProviderMock,
}));

import { ProviderSettingsPage } from "../features/providers/ProviderSettingsPage";

const provider = {
  id: "openai",
  display_name: "OpenAI",
};

const model = {
  provider_id: "openai",
  model_id: "gpt-4o-mini",
  display_name: "GPT-4o mini",
  context_window_tokens: 8192,
  default_output_tokens: 1024,
  strengths: [],
  weaknesses: [],
  strategy: [],
  tier: "medium",
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
  });

  it("shows the explicit session-only credential status", async () => {
    renderProviders();

    expect(await screen.findByTestId("empty-providers")).toBeInTheDocument();
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

    expect(await screen.findByTestId("empty-providers")).toBeInTheDocument();
    expect(
      screen.getByText("Credentials use platform secure storage."),
    ).toBeInTheDocument();
  });

  it("validates required fields before calling configure", async () => {
    const user = userEvent.setup();
    renderProviders();
    await screen.findByTestId("empty-providers");
    await user.click(
      screen.getByRole("button", { name: "Configure your first provider" }),
    );
    await user.click(screen.getByRole("button", { name: "Save provider" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Provider ID is required.",
    );
    expect(configureProviderMock).not.toHaveBeenCalled();
  });

  it("submits typed metadata and removes the secret field after success", async () => {
    const user = userEvent.setup();
    renderProviders();
    await screen.findByTestId("empty-providers");
    await user.click(screen.getByRole("button", { name: "Add provider" }));

    await user.type(screen.getByLabelText("Provider ID"), "openai");
    await user.type(screen.getByPlaceholderText("OpenAI"), "OpenAI");
    await user.type(screen.getByLabelText("Credential ID"), "openai-primary");
    await user.type(screen.getByLabelText("API key"), "secret-token");
    await user.type(screen.getByLabelText("Model ID"), "gpt-4o-mini");
    await user.type(
      screen.getByLabelText("Model display name 1"),
      "GPT-4o mini",
    );
    await user.click(screen.getByRole("button", { name: "Save provider" }));

    await waitFor(() => expect(configureProviderMock).toHaveBeenCalledTimes(1));
    expect(configureProviderMock).toHaveBeenCalledWith({
      descriptor: { id: "openai", display_name: "OpenAI" },
      base_url: "https://api.example.com/v1",
      credential_id: "openai-primary",
      credential_value: "secret-token",
      models: [model],
    });
    expect(screen.queryByLabelText("API key")).not.toBeInTheDocument();
  });

  it("removes a configured provider through the typed command", async () => {
    const user = userEvent.setup();
    listProvidersMock.mockResolvedValueOnce([provider]);
    listModelsMock.mockResolvedValueOnce([model]);
    renderProviders();

    expect(await screen.findByText("OpenAI")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Remove OpenAI" }));
    await waitFor(() =>
      expect(removeProviderMock).toHaveBeenCalledWith("openai"),
    );
  });
});
