import { userEvent } from "@testing-library/user-event";
import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const listCharactersMock = vi.hoisted(() => vi.fn());
const createCharacterMock = vi.hoisted(() => vi.fn());
const getCharacterStateMock = vi.hoisted(() => vi.fn());
const updateCharacterMock = vi.hoisted(() => vi.fn());
const updateCharacterStateMock = vi.hoisted(() => vi.fn());
const archiveCharacterMock = vi.hoisted(() => vi.fn());
const listMemoryHistoryMock = vi.hoisted(() => vi.fn());
const restoreMemoryMock = vi.hoisted(() => vi.fn());
const setMemoryCanonStatusMock = vi.hoisted(() => vi.fn());

vi.mock("../lib/commands", () => ({
  listCharacters: listCharactersMock,
  createCharacter: createCharacterMock,
  getCharacterState: getCharacterStateMock,
  updateCharacter: updateCharacterMock,
  updateCharacterState: updateCharacterStateMock,
  archiveCharacter: archiveCharacterMock,
  listMemoryHistory: listMemoryHistoryMock,
  restoreMemory: restoreMemoryMock,
  setMemoryCanonStatus: setMemoryCanonStatusMock,
}));

import { CharacterPanel } from "../features/projects/CharacterPanel";

const character = {
  id: "character-1",
  project_id: "project-1",
  name: "Mara Voss",
  summary: "A courier carrying a dangerous secret.",
  role: "Protagonist",
  status: "active" as const,
  created_at: "2026-10-01T00:00:00Z",
  updated_at: "2026-10-01T00:00:00Z",
  revision: 1,
  canon_status: "canon" as const,
};

const state = {
  character_id: "character-1",
  current_location: "North station",
  physical_condition: "Tired",
  injuries: "",
  emotional_state: "Wary",
  goals: "Reach the river before dawn.",
  beliefs: "",
  knowledge: "",
  secrets_known: "",
  current_conflicts: "",
  possessions: "",
  promises: "",
  last_appearance: "",
  current_arc_role: "Reluctant guide",
  updated_at: "2026-10-01T00:00:00Z",
  revision: 1,
  canon_status: "canon" as const,
};

describe("CharacterPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listCharactersMock.mockResolvedValue([character]);
    getCharacterStateMock.mockResolvedValue(state);
    updateCharacterMock.mockResolvedValue(character);
    updateCharacterStateMock.mockResolvedValue(state);
    listMemoryHistoryMock.mockResolvedValue([]);
    restoreMemoryMock.mockResolvedValue({ revision: 2 });
    setMemoryCanonStatusMock.mockResolvedValue({ revision: 2 });
  });

  it("loads a character profile and continuity state", async () => {
    render(<CharacterPanel projectId="project-1" projectArchived={false} />);

    expect(
      await screen.findByRole("heading", { name: "Mara Voss" }),
    ).toBeInTheDocument();
    expect(
      await screen.findByDisplayValue("North station"),
    ).toBeInTheDocument();
    expect(
      await screen.findByDisplayValue("Reach the river before dawn."),
    ).toBeInTheDocument();
    expect(getCharacterStateMock).toHaveBeenCalledWith("character-1");
    expect(listMemoryHistoryMock).toHaveBeenCalledWith(
      "character",
      "character-1",
    );
    expect(listMemoryHistoryMock).toHaveBeenCalledWith(
      "character_state",
      "character-1",
    );
  });

  it("validates a new character before calling the backend", async () => {
    const user = userEvent.setup();
    render(<CharacterPanel projectId="project-1" projectArchived={false} />);
    await screen.findByRole("heading", { name: "Mara Voss" });
    await user.click(screen.getByRole("button", { name: "Add" }));
    await user.click(screen.getByRole("button", { name: "Create character" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Character name is required.",
    );
    expect(createCharacterMock).not.toHaveBeenCalled();
  });

  it("saves edited continuity fields for the selected character", async () => {
    const user = userEvent.setup();
    render(<CharacterPanel projectId="project-1" projectArchived={false} />);
    await screen.findByRole("heading", { name: "Mara Voss" });
    await screen.findByDisplayValue("North station");
    const location = screen.getByLabelText("Current location");
    await user.clear(location);
    await user.type(location, "The river crossing");
    await user.click(screen.getByRole("button", { name: "Save state" }));

    await waitFor(() => expect(updateCharacterStateMock).toHaveBeenCalled());
    expect(updateCharacterStateMock).toHaveBeenCalledWith(
      "character-1",
      expect.objectContaining({ current_location: "The river crossing" }),
    );
  });
});
