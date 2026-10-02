import { userEvent } from "@testing-library/user-event";
import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { CommandError } from "../lib/command-error";

const listMemoryHistoryMock = vi.hoisted(() => vi.fn());
const restoreMemoryMock = vi.hoisted(() => vi.fn());
const setMemoryCanonStatusMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/commands", () => ({
  listMemoryHistory: listMemoryHistoryMock,
  restoreMemory: restoreMemoryMock,
  setMemoryCanonStatus: setMemoryCanonStatusMock,
}));

import { RevisionHistoryPanel } from "../features/projects/RevisionHistoryPanel";

const revisionOne = {
  id: "revision-1",
  project_id: "project-1",
  entity_type: "character" as const,
  entity_id: "character-1",
  revision: 1,
  operation: "create" as const,
  actor_type: "user" as const,
  actor_id: null,
  base_revision: 0,
  previous_value: null,
  new_value: { name: "Mira", summary: "A courier", role: "protagonist" },
  source_type: "manual",
  source_id: null,
  created_at: "2026-10-01T00:00:00Z",
};

const revisionTwo = {
  ...revisionOne,
  id: "revision-2",
  revision: 2,
  operation: "update" as const,
  base_revision: 1,
  previous_value: revisionOne.new_value,
  new_value: { name: "Mira Vale", summary: "A courier", role: "protagonist" },
  created_at: "2026-10-01T00:01:00Z",
};

function renderPanel(
  overrides: Partial<React.ComponentProps<typeof RevisionHistoryPanel>> = {},
) {
  return render(
    <RevisionHistoryPanel
      entityType="character"
      entityId="character-1"
      currentRevision={2}
      canonStatus="canon"
      onRestored={vi.fn()}
      disabled={false}
      {...overrides}
    />,
  );
}

describe("RevisionHistoryPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listMemoryHistoryMock.mockResolvedValue([revisionOne, revisionTwo]);
    restoreMemoryMock.mockResolvedValue({ ...revisionTwo, revision: 3 });
    setMemoryCanonStatusMock.mockResolvedValue({ ...revisionTwo, revision: 3 });
  });

  it("renders revision metadata and field diff", async () => {
    renderPanel();

    expect(await screen.findByText("Revision 2")).toBeInTheDocument();
    expect(screen.getByText("Name")).toBeInTheDocument();
    expect(screen.getByText("Mira")).toBeInTheDocument();
    expect(screen.getByText("Mira Vale")).toBeInTheDocument();
    expect(
      screen.queryByText("A courier", { selector: "del" }),
    ).not.toBeInTheDocument();
  });

  it("restores the selected revision", async () => {
    const user = userEvent.setup();
    const onRestored = vi.fn();
    renderPanel({ onRestored });

    await screen.findByText("Revision 2");
    await user.click(screen.getByRole("button", { name: /Revision 1/ }));
    await user.click(screen.getByRole("button", { name: "Restore revision" }));
    await user.click(screen.getByRole("button", { name: "Confirm restore" }));

    await waitFor(() =>
      expect(restoreMemoryMock).toHaveBeenCalledWith(
        "character",
        "character-1",
        1,
        2,
      ),
    );
    expect(onRestored).toHaveBeenCalled();
  });

  it("disables mutations for locked canon", async () => {
    renderPanel({ canonStatus: "locked_canon" });

    await screen.findByText("Revision 2");
    expect(
      screen.getByRole("button", { name: "Restore revision" }),
    ).toBeDisabled();
  });

  it("keeps form input on conflict", async () => {
    const user = userEvent.setup();
    restoreMemoryMock.mockRejectedValueOnce(
      new CommandError("conflict", "raw serialized conflict"),
    );
    renderPanel();

    await screen.findByText("Revision 2");
    await user.click(screen.getByRole("button", { name: "Restore revision" }));
    await user.click(screen.getByRole("button", { name: "Confirm restore" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "This record changed. Reload and try again.",
    );
    expect(
      screen.queryByText("raw serialized conflict"),
    ).not.toBeInTheDocument();
  });

  it("can retry a failed revision history load", async () => {
    const user = userEvent.setup();
    listMemoryHistoryMock
      .mockRejectedValueOnce({ code: "storage" })
      .mockResolvedValueOnce([revisionOne, revisionTwo]);
    renderPanel();

    expect(await screen.findByRole("alert")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Try again" }));
    expect(await screen.findByText("Revision 2")).toBeInTheDocument();
    expect(listMemoryHistoryMock).toHaveBeenCalledTimes(2);
  });
});
