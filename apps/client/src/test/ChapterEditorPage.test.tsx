import { userEvent } from "@testing-library/user-event";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

const getChapterMock = vi.hoisted(() => vi.fn());
const getManuscriptMock = vi.hoisted(() => vi.fn());
const listRevisionsMock = vi.hoisted(() => vi.fn());
const saveManuscriptMock = vi.hoisted(() => vi.fn());
const restoreManuscriptMock = vi.hoisted(() => vi.fn());
const updateChapterMock = vi.hoisted(() => vi.fn());
const listConversationsMock = vi.hoisted(() => vi.fn());
const createConversationMock = vi.hoisted(() => vi.fn());
const listConversationMessagesMock = vi.hoisted(() => vi.fn());
const listProposalsMock = vi.hoisted(() => vi.fn());
const chapterChatSendMock = vi.hoisted(() => vi.fn());
const createProposalMock = vi.hoisted(() => vi.fn());
const promoteProposalMock = vi.hoisted(() => vi.fn());
const rejectProposalMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/commands", () => ({
  getChapter: getChapterMock,
  getManuscript: getManuscriptMock,
  listManuscriptRevisions: listRevisionsMock,
  saveManuscript: saveManuscriptMock,
  restoreManuscript: restoreManuscriptMock,
  updateChapter: updateChapterMock,
  listConversations: listConversationsMock,
  createConversation: createConversationMock,
  listConversationMessages: listConversationMessagesMock,
  listManuscriptProposals: listProposalsMock,
  chapterChatSend: chapterChatSendMock,
  createManuscriptProposal: createProposalMock,
  promoteManuscriptProposal: promoteProposalMock,
  rejectManuscriptProposal: rejectProposalMock,
}));

import { ChapterEditorPage } from "../features/manuscripts/ChapterEditorPage";
import { CommandError } from "../lib/command-error";

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

const manuscript = {
  id: "manuscript-1",
  chapter_id: "chapter-1",
  content: "The gate waited.",
  content_format: "plain_text" as const,
  revision: 1,
  created_at: "2026-10-01T00:00:00Z",
  updated_at: "2026-10-01T00:00:00Z",
};

const initialRevision = {
  id: "revision-1",
  manuscript_id: "manuscript-1",
  revision: 1,
  content: "The gate waited.",
  content_format: "plain_text" as const,
  label: "Initial draft",
  actor_type: "user" as const,
  actor_id: null,
  created_at: "2026-10-01T00:00:00Z",
};

function renderEditor() {
  return render(
    <MemoryRouter initialEntries={["/projects/project-1/chapters/chapter-1"]}>
      <Routes>
        <Route
          path="/projects/:projectId/chapters/:chapterId"
          element={<ChapterEditorPage />}
        />
      </Routes>
    </MemoryRouter>,
  );
}

describe("ChapterEditorPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    getChapterMock.mockResolvedValue(chapter);
    getManuscriptMock.mockResolvedValue(manuscript);
    listRevisionsMock.mockResolvedValue([initialRevision]);
    saveManuscriptMock.mockResolvedValue({
      ...manuscript,
      content: "A new dawn.",
      revision: 2,
    });
    restoreManuscriptMock.mockResolvedValue({
      ...manuscript,
      revision: 3,
      content: "The gate waited.",
    });
    updateChapterMock.mockResolvedValue({ ...chapter, revision: 2 });
    listConversationsMock.mockResolvedValue([
      {
        id: "conversation-1",
        project_id: "project-1",
        chapter_id: "chapter-1",
        kind: "chapter_chat",
        title: "Chapter Chat",
        created_at: "2026-10-01T00:00:00Z",
        updated_at: "2026-10-01T00:00:00Z",
      },
    ]);
    listConversationMessagesMock.mockResolvedValue([]);
    listProposalsMock.mockResolvedValue([]);
  });

  it("loads prose, saves a new revision and exposes restore", async () => {
    const user = userEvent.setup();
    renderEditor();
    const editor = await screen.findByRole("textbox", { name: "Manuscript" });
    expect(editor).toHaveTextContent("The gate waited.");
    await user.clear(editor);
    await user.type(editor, "A new dawn.");
    await user.click(screen.getByRole("button", { name: "Save manuscript" }));
    await waitFor(() =>
      expect(saveManuscriptMock).toHaveBeenCalledWith(
        "chapter-1",
        expect.objectContaining({
          content: "A new dawn.",
          expected_revision: 1,
        }),
      ),
    );

    listRevisionsMock.mockResolvedValueOnce([
      { ...initialRevision, revision: 2, id: "revision-2", label: "Draft 2" },
      initialRevision,
    ]);
    await user.click(await screen.findByRole("button", { name: "Restore" }));
    await waitFor(() =>
      expect(restoreManuscriptMock).toHaveBeenCalledWith("chapter-1", 1, 2),
    );
    expect(await screen.findByText("Saved revision 3")).toBeInTheDocument();
  });

  it("shows a conflict and lets the user keep the local copy explicitly", async () => {
    const user = userEvent.setup();
    const server = {
      ...manuscript,
      content: "Server copy.",
      content_format: "html" as const,
      revision: 4,
    };
    getManuscriptMock
      .mockReset()
      .mockResolvedValueOnce(manuscript)
      .mockResolvedValueOnce(server);
    saveManuscriptMock
      .mockReset()
      .mockRejectedValueOnce(new CommandError("conflict", "stale"))
      .mockResolvedValueOnce({
        ...manuscript,
        content: "<p>Local copy.</p>",
        content_format: "html" as const,
        revision: 5,
      });

    renderEditor();
    const editor = await screen.findByRole("textbox", { name: "Manuscript" });
    await user.clear(editor);
    await user.type(editor, "Local copy.");
    await user.click(screen.getByRole("button", { name: "Save manuscript" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Concurrent edit detected",
    );
    await user.click(
      screen.getByRole("button", { name: "Keep my local copy" }),
    );
    await waitFor(() =>
      expect(saveManuscriptMock).toHaveBeenLastCalledWith(
        "chapter-1",
        expect.objectContaining({ expected_revision: 4 }),
      ),
    );
    expect(await screen.findByText("Saved revision 5")).toBeInTheDocument();
  });

  it("autosaves dirty prose after the debounce window", async () => {
    const user = userEvent.setup();
    renderEditor();
    const editor = await screen.findByRole("textbox", { name: "Manuscript" });
    await user.clear(editor);
    await user.type(editor, "Autosaved prose.");
    await waitFor(
      () =>
        expect(saveManuscriptMock).toHaveBeenCalledWith(
          "chapter-1",
          expect.objectContaining({
            label: "Autosave 2",
            content_format: "html",
          }),
        ),
      { timeout: 2_000 },
    );
  });

  it("can retry a failed chapter load", async () => {
    const user = userEvent.setup();
    getChapterMock
      .mockRejectedValueOnce({ code: "storage" })
      .mockResolvedValueOnce(chapter);

    renderEditor();

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "We could not open this chapter.",
    );
    await user.click(screen.getByRole("button", { name: "Try again" }));
    expect(
      await screen.findByRole("textbox", { name: "Manuscript" }),
    ).toBeInTheDocument();
    expect(getChapterMock).toHaveBeenCalledTimes(2);
  });
});
