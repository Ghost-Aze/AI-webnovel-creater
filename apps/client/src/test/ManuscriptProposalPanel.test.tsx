import { userEvent } from "@testing-library/user-event";
import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const listProposalsMock = vi.hoisted(() => vi.fn());
const promoteProposalMock = vi.hoisted(() => vi.fn());
const rejectProposalMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/commands", () => ({
  listManuscriptProposals: listProposalsMock,
  promoteManuscriptProposal: promoteProposalMock,
  rejectManuscriptProposal: rejectProposalMock,
}));

import { ManuscriptProposalPanel } from "../features/manuscripts/ManuscriptProposalPanel";

const proposal = {
  id: "proposal-1",
  project_id: "project-1",
  chapter_id: "chapter-1",
  base_revision: 2,
  proposed_content: "A sharper opening.",
  content_format: "plain_text" as const,
  rationale: "Tighten pacing",
  status: "draft" as const,
  actor_type: "ai" as const,
  actor_id: "chapter-chat",
  created_at: "2026-10-01T00:00:00Z",
  updated_at: "2026-10-01T00:00:00Z",
};

describe("ManuscriptProposalPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listProposalsMock.mockResolvedValue([proposal]);
    promoteProposalMock.mockResolvedValue({
      id: "manuscript-1",
      chapter_id: "chapter-1",
      content: "A sharper opening.",
      content_format: "plain_text",
      revision: 3,
      created_at: "2026-10-01T00:00:00Z",
      updated_at: "2026-10-01T00:00:00Z",
    });
    rejectProposalMock.mockResolvedValue({ ...proposal, status: "rejected" });
  });

  it("promotes a draft with the currently loaded revision", async () => {
    const user = userEvent.setup();
    const onPromoted = vi.fn();
    render(
      <ManuscriptProposalPanel
        chapterId="chapter-1"
        currentRevision={2}
        onPromoted={onPromoted}
      />,
    );

    await user.click(
      await screen.findByRole("button", { name: "Apply revision" }),
    );
    await waitFor(() =>
      expect(promoteProposalMock).toHaveBeenCalledWith("proposal-1", 2),
    );
    expect(onPromoted).toHaveBeenCalledWith(
      expect.objectContaining({ revision: 3 }),
    );
    expect(
      await screen.findByText("No pending proposals."),
    ).toBeInTheDocument();
  });

  it("can retry a failed proposal load", async () => {
    const user = userEvent.setup();
    listProposalsMock
      .mockRejectedValueOnce({ code: "storage" })
      .mockResolvedValueOnce([proposal]);

    render(
      <ManuscriptProposalPanel
        chapterId="chapter-1"
        currentRevision={2}
        onPromoted={vi.fn()}
      />,
    );

    expect(await screen.findByRole("alert")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Try again" }));
    expect(await screen.findByText("A sharper opening.")).toBeInTheDocument();
    expect(listProposalsMock).toHaveBeenCalledTimes(2);
  });
});
