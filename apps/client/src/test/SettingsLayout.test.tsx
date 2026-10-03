import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { describe, expect, it, vi } from "vitest";

vi.mock("../components/layout/AppShell", async () => {
  const { Outlet } = await import("react-router-dom");
  return { AppShell: () => <Outlet /> };
});
vi.mock("../features/providers/ProviderSettingsPage", () => ({
  ProviderSettingsPage: () => <div data-testid="provider-route">Providers route</div>,
}));

import { SettingsLayout } from "../features/providers/SettingsLayout";
import { AppRoutes } from "../app/routes";

const providers = [
  { id: "openrouter", display_name: "OpenRouter" },
];

describe("SettingsLayout", () => {
  it("renders the model provider section around typed settings content", () => {
    render(
      <MemoryRouter initialEntries={["/settings/providers"]}>
        <SettingsLayout
          providers={providers}
          configuredProviderIds={["openrouter"]}
          selectedProviderId="openrouter"
          onSelectProvider={vi.fn()}
          onAddProvider={vi.fn()}
        >
          <div data-testid="settings-content">Provider content</div>
        </SettingsLayout>
      </MemoryRouter>,
    );

    expect(
      screen.getByRole("navigation", { name: "Settings navigation" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Model Providers" }),
    ).toHaveAttribute("aria-current", "page");
    expect(screen.getByText("Chat")).toHaveAttribute("aria-disabled", "true");
    expect(screen.getByText("Appearance")).toHaveAttribute("aria-disabled", "true");
    expect(screen.getByText("Storage")).toHaveAttribute("aria-disabled", "true");
    expect(screen.getByTestId("settings-content")).toBeInTheDocument();
  });

  it("redirects the settings root to the model providers section", () => {
    render(
      <MemoryRouter initialEntries={["/settings"]}>
        <AppRoutes />
      </MemoryRouter>,
    );

    expect(screen.getByTestId("provider-route")).toBeInTheDocument();
  });
});
