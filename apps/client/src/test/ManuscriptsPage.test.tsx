import { render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

const getProjectMock = vi.hoisted(() => vi.fn());
const listProjectsMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/commands", () => ({
  getProject: getProjectMock,
  listProjects: listProjectsMock,
}));

vi.mock("../features/manuscripts/ChapterPanel", () => ({
  ChapterPanel: ({ projectId }: { projectId: string }) => (
    <div>Chapter panel for {projectId}</div>
  ),
}));

import { ManuscriptsPage } from "../features/manuscripts/ManuscriptsPage";

const project = {
  id: "project-1",
  name: "The Long Night",
  description: "A courier crosses a divided city.",
  status: "active" as const,
  created_at: "2026-10-01T00:00:00Z",
  updated_at: "2026-10-01T00:00:00Z",
};

describe("ManuscriptsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    getProjectMock.mockResolvedValue(project);
    listProjectsMock.mockResolvedValue([project]);
  });

  it("opens the manuscript workspace for a project", async () => {
    render(
      <MemoryRouter initialEntries={["/projects/project-1/manuscripts"]}>
        <Routes>
          <Route
            path="/projects/:projectId/manuscripts"
            element={<ManuscriptsPage />}
          />
        </Routes>
      </MemoryRouter>,
    );

    expect(
      await screen.findByRole("heading", { name: "Manuscripts" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Chapter panel for project-1")).toBeInTheDocument();
  });

  it("offers a project picker from the global manuscript entry", async () => {
    render(
      <MemoryRouter initialEntries={["/manuscripts"]}>
        <Routes>
          <Route path="/manuscripts" element={<ManuscriptsPage />} />
        </Routes>
      </MemoryRouter>,
    );

    expect(
      await screen.findByRole("heading", { name: "Manuscripts" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: /The Long Night/ }),
    ).toHaveAttribute("href", "/projects/project-1/manuscripts");
  });
});
