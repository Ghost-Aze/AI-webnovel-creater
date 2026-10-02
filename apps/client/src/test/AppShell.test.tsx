import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { describe, expect, it } from "vitest";

import { AppShell } from "../components/layout/AppShell";

describe("AppShell mobile navigation", () => {
  it("exposes an AI workspace header and bottom navigation", () => {
    render(
      <MemoryRouter initialEntries={["/projects"]}>
        <AppShell />
      </MemoryRouter>,
    );

    expect(
      screen.getByRole("banner", { name: "Mobile workspace header" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("navigation", { name: "Mobile navigation" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Workspace" })).toHaveAttribute(
      "href",
      "/projects",
    );
    expect(screen.getByRole("link", { name: "Chat" })).toHaveAttribute(
      "href",
      "/chat",
    );
    expect(screen.getByRole("link", { name: "Models" })).toHaveAttribute(
      "href",
      "/settings/providers",
    );
  });

  it("keeps project chat and manuscript entries actionable", () => {
    render(
      <MemoryRouter initialEntries={["/projects/project-1"]}>
        <AppShell />
      </MemoryRouter>,
    );

    expect(
      screen.getByRole("link", { name: "Developer Chat" }),
    ).toHaveAttribute("href", "/projects/project-1/chat");
    expect(screen.getByRole("link", { name: "Manuscripts" })).toHaveAttribute(
      "href",
      "/projects/project-1/manuscripts",
    );
  });
});
