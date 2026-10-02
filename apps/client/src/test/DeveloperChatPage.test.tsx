import { render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

const getProjectMock = vi.hoisted(() => vi.fn());
const listProjectsMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/commands", () => ({
  getProject: getProjectMock,
  listProjects: listProjectsMock,
}));

vi.mock("../features/chat/DeveloperChatPanel", () => ({
  DeveloperChatPanel: ({ projectId }: { projectId: string }) => (
    <div>Chat panel for {projectId}</div>
  ),
}));

import { DeveloperChatPage } from "../features/chat/DeveloperChatPage";

const project = {
  id: "project-1",
  name: "The Long Night",
  description: "A courier crosses a divided city.",
  status: "active" as const,
  created_at: "2026-10-01T00:00:00Z",
  updated_at: "2026-10-01T00:00:00Z",
};

describe("DeveloperChatPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    getProjectMock.mockResolvedValue(project);
    listProjectsMock.mockResolvedValue([project]);
  });

  it("opens the developer chat for the selected project", async () => {
    render(
      <MemoryRouter initialEntries={["/projects/project-1/chat"]}>
        <Routes>
          <Route
            path="/projects/:projectId/chat"
            element={<DeveloperChatPage />}
          />
        </Routes>
      </MemoryRouter>,
    );

    expect(
      await screen.findByRole("heading", { name: "Developer Chat" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Chat panel for project-1")).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: /Project workspace/ }),
    ).toHaveAttribute("href", "/projects/project-1");
  });

  it("offers a project picker from the global chat entry", async () => {
    render(
      <MemoryRouter initialEntries={["/chat"]}>
        <Routes>
          <Route path="/chat" element={<DeveloperChatPage />} />
        </Routes>
      </MemoryRouter>,
    );

    expect(
      await screen.findByRole("heading", { name: "Developer Chat" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: /The Long Night/ }),
    ).toHaveAttribute("href", "/projects/project-1/chat");
  });
});
