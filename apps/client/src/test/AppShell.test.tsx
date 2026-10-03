import { render, screen, within } from "@testing-library/react";
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
      screen.getByRole("navigation", { name: "Project navigation" }),
    ).toBeInTheDocument();
    const projectNavigation = screen.getByRole("navigation", {
      name: "Project navigation",
    });
    expect(within(projectNavigation).getByRole("link", { name: "Overview" })).toHaveAttribute(
      "href",
      "/projects/project-1",
    );
    expect(
      within(projectNavigation).getByRole("link", { name: "Developer Chat" }),
    ).toHaveAttribute("href", "/projects/project-1/chat");
    expect(
      within(projectNavigation).getByRole("link", { name: "Manuscripts" }),
    ).toHaveAttribute("href", "/projects/project-1/manuscripts");
    const mobileNavigation = screen.getByRole("navigation", {
      name: "Mobile navigation",
    });
    expect(
      within(mobileNavigation).getByRole("link", { name: "Manuscripts" }),
    ).toHaveAttribute("href", "/projects/project-1/manuscripts");
  });

  it("keeps provider settings out of the project navigation rail", () => {
    render(
      <MemoryRouter initialEntries={["/settings/providers"]}>
        <AppShell />
      </MemoryRouter>,
    );

    const sidebar = screen.getByRole("complementary", {
      name: "Application sidebar",
    });
    expect(within(sidebar).getByRole("link", { name: "Providers" })).toHaveAttribute(
      "href",
      "/settings/providers",
    );
    expect(
      within(sidebar).queryByRole("link", { name: "Developer Chat" }),
    ).not.toBeInTheDocument();
    expect(
      within(sidebar).queryByRole("link", { name: "Manuscripts" }),
    ).not.toBeInTheDocument();
  });
});
