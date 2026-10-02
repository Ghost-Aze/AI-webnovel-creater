import { userEvent } from "@testing-library/user-event";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

const getProjectMock = vi.hoisted(() => vi.fn());
const updateProjectMock = vi.hoisted(() => vi.fn());
const archiveProjectMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/commands", () => ({
  getProject: getProjectMock,
  updateProject: updateProjectMock,
  archiveProject: archiveProjectMock,
}));

vi.mock("../features/projects/CharacterPanel", () => ({
  CharacterPanel: () => <div>Characters panel</div>,
}));

vi.mock("../features/manuscripts/ChapterPanel", () => ({
  ChapterPanel: () => <div>Chapters panel</div>,
}));

import { ProjectWorkspacePage } from "../features/projects/ProjectWorkspacePage";

const project = {
  id: "project-1",
  name: "The Long Night",
  description: "A courier crosses a divided city.",
  status: "active" as const,
  created_at: "2026-10-01T00:00:00Z",
  updated_at: "2026-10-01T00:00:00Z",
  revision: 1,
};

function renderWorkspace() {
  return render(
    <MemoryRouter initialEntries={["/projects/project-1"]}>
      <Routes>
        <Route path="/projects/:projectId" element={<ProjectWorkspacePage />} />
        <Route path="/projects" element={<p>Projects list</p>} />
      </Routes>
    </MemoryRouter>,
  );
}

describe("ProjectWorkspacePage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    getProjectMock.mockResolvedValue(project);
    updateProjectMock.mockResolvedValue({
      ...project,
      name: "The Longer Night",
      revision: 2,
    });
    archiveProjectMock.mockResolvedValue({ ...project, status: "archived" });
  });

  it("loads and saves project details", async () => {
    const user = userEvent.setup();
    renderWorkspace();

    expect(
      await screen.findByRole("heading", { name: "The Long Night" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Open Developer Chat" }),
    ).toHaveAttribute("href", "/projects/project-1/chat");
    const name = screen.getByLabelText("Project name");
    await user.clear(name);
    await user.type(name, "The Longer Night");
    await user.click(screen.getByRole("button", { name: "Save changes" }));

    await waitFor(() =>
      expect(updateProjectMock).toHaveBeenCalledWith("project-1", {
        name: "The Longer Night",
        description: "A courier crosses a divided city.",
      }),
    );
    expect(await screen.findByText("Saved")).toBeInTheDocument();
  });

  it("can retry a failed project load", async () => {
    const user = userEvent.setup();
    getProjectMock
      .mockRejectedValueOnce({ code: "storage" })
      .mockResolvedValueOnce(project);

    renderWorkspace();

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "We could not open this project.",
    );
    await user.click(screen.getByRole("button", { name: "Try again" }));
    expect(
      await screen.findByRole("heading", { name: "The Long Night" }),
    ).toBeInTheDocument();
    expect(getProjectMock).toHaveBeenCalledTimes(2);
  });

  it("archives the project and returns to the project list", async () => {
    const user = userEvent.setup();
    renderWorkspace();

    await screen.findByRole("heading", { name: "The Long Night" });
    await user.click(screen.getByRole("button", { name: "Archive project" }));

    await waitFor(() =>
      expect(archiveProjectMock).toHaveBeenCalledWith("project-1"),
    );
    expect(await screen.findByText("Projects list")).toBeInTheDocument();
  });
});
