import { userEvent } from "@testing-library/user-event";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

const listChaptersMock = vi.hoisted(() => vi.fn());
const createChapterMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/commands", () => ({
  listChapters: listChaptersMock,
  createChapter: createChapterMock,
}));

import { ChapterPanel } from "../features/manuscripts/ChapterPanel";

const chapter = {
  id: "chapter-1",
  project_id: "project-1",
  number: 1,
  title: "Opening",
  synopsis: "Arrival",
  status: "draft" as const,
  revision: 1,
  created_at: "2026-10-01T00:00:00Z",
  updated_at: "2026-10-01T00:00:00Z",
};

describe("ChapterPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listChaptersMock.mockResolvedValue([]);
    createChapterMock.mockResolvedValue({
      ...chapter,
      id: "chapter-2",
      number: 2,
    });
  });

  it("loads chapters and creates a new chapter with typed fields", async () => {
    const user = userEvent.setup();
    listChaptersMock.mockResolvedValueOnce([chapter]);
    render(
      <MemoryRouter>
        <ChapterPanel projectId="project-1" projectArchived={false} />
      </MemoryRouter>,
    );

    expect(await screen.findByText("Opening")).toBeInTheDocument();
    await user.type(screen.getByLabelText("Title"), " Second");
    await user.click(screen.getByRole("button", { name: "New chapter" }));
    await waitFor(() => expect(createChapterMock).toHaveBeenCalled());
    expect(createChapterMock).toHaveBeenCalledWith("project-1", {
      number: 1,
      title: " Second",
    });
  });

  it("does not show mutation controls for archived projects", async () => {
    render(
      <MemoryRouter>
        <ChapterPanel projectId="project-1" projectArchived />
      </MemoryRouter>,
    );
    expect(
      await screen.findByText(
        "No chapters yet. Give the story its first page.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "New chapter" }),
    ).not.toBeInTheDocument();
  });
});
