import { userEvent } from "@testing-library/user-event";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { CommandError } from "../lib/command-error";

const listProjectsMock = vi.hoisted(() => vi.fn());
const createProjectMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/commands", () => ({
  listProjects: listProjectsMock,
  createProject: createProjectMock,
  getProject: vi.fn(),
  updateProject: vi.fn(),
  archiveProject: vi.fn(),
}));

import { ProjectsPage } from "../features/projects/ProjectsPage";

const project = {
  id: "project-1",
  name: "The Long Night",
  description: "A city waits for dawn.",
  status: "active" as const,
  created_at: "2026-10-01T00:00:00Z",
  updated_at: "2026-10-01T00:00:00Z",
};

function renderProjects() {
  return render(
    <MemoryRouter initialEntries={["/projects"]}>
      <Routes>
        <Route path="/projects" element={<ProjectsPage />} />
        <Route
          path="/projects/:projectId"
          element={<p>project workspace route</p>}
        />
      </Routes>
    </MemoryRouter>,
  );
}

describe("ProjectsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listProjectsMock.mockResolvedValue([]);
  });

  it("shows a useful empty project state", async () => {
    renderProjects();

    expect(await screen.findByTestId("empty-projects")).toBeInTheDocument();
    expect(
      screen.getByText("Your first story starts here"),
    ).toBeInTheDocument();
  });

  it("renders active projects returned by the command client", async () => {
    listProjectsMock.mockResolvedValueOnce([project]);
    renderProjects();

    expect(await screen.findByText("The Long Night")).toBeInTheDocument();
    expect(screen.getByText("A city waits for dawn.")).toBeInTheDocument();
  });

  it("rejects a whitespace-only name before calling the backend", async () => {
    const user = userEvent.setup();
    renderProjects();
    await screen.findByTestId("empty-projects");
    await user.click(screen.getByRole("button", { name: "New project" }));
    await user.click(screen.getByRole("button", { name: "Create project" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Project name is required.",
    );
    expect(createProjectMock).not.toHaveBeenCalled();
  });

  it("creates a project and navigates to its workspace", async () => {
    const user = userEvent.setup();
    createProjectMock.mockResolvedValueOnce(project);
    renderProjects();
    await screen.findByTestId("empty-projects");
    await user.click(
      screen.getByRole("button", { name: "Create your first project" }),
    );
    await user.type(screen.getByLabelText("Project name"), "The Long Night");
    await user.click(screen.getByRole("button", { name: "Create project" }));

    await waitFor(() =>
      expect(screen.getByText("project workspace route")).toBeInTheDocument(),
    );
    expect(createProjectMock).toHaveBeenCalledWith({
      name: "The Long Night",
      description: "",
    });
  });

  it("shows a safe message when listing fails", async () => {
    listProjectsMock.mockRejectedValueOnce(
      new CommandError("storage", "raw SQLite details"),
    );
    renderProjects();

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent(
      "The project could not be saved. Try again.",
    );
    expect(alert).not.toHaveTextContent("SQLite");
  });
});
