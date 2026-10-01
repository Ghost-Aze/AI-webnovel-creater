import {
  type FormEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";

import { normalizeCommandError } from "../../lib/command-error";
import {
  archiveCharacter,
  createCharacter,
  getCharacterState,
  listCharacters,
  updateCharacter,
  updateCharacterState,
} from "../../lib/commands";
import type {
  Character,
  CharacterState,
  CreateCharacterInput,
  UpdateCharacterStateInput,
} from "../../types/character";
import { RevisionHistoryPanel } from "./RevisionHistoryPanel";

const emptyState: CharacterState = {
  character_id: "",
  current_location: "",
  physical_condition: "",
  injuries: "",
  emotional_state: "",
  goals: "",
  beliefs: "",
  knowledge: "",
  secrets_known: "",
  current_conflicts: "",
  possessions: "",
  promises: "",
  last_appearance: "",
  current_arc_role: "",
  revision: 0,
  canon_status: "canon",
  updated_at: "",
};

interface CharacterPanelProps {
  projectId: string;
  projectArchived: boolean;
}

export function CharacterPanel({
  projectId,
  projectArchived,
}: CharacterPanelProps) {
  const [characters, setCharacters] = useState<Character[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [state, setState] = useState<CharacterState>(emptyState);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [isStateSaving, setIsStateSaving] = useState(false);
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const stateDirtyRef = useRef(false);
  const stateInteractionRef = useRef(false);

  const selectedCharacter = characters.find(
    (character) => character.id === selectedId,
  );
  const characterLocked = selectedCharacter?.canon_status === "locked_canon";
  const stateLocked = state.canon_status === "locked_canon";

  const loadCharacters = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const loaded = await listCharacters(projectId);
      setCharacters(loaded);
      setSelectedId((currentId) => {
        if (
          currentId &&
          loaded.some((character) => character.id === currentId)
        ) {
          return currentId;
        }
        return loaded[0]?.id ?? null;
      });
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsLoading(false);
    }
  }, [projectId]);

  useEffect(() => {
    void loadCharacters();
  }, [loadCharacters]);

  useEffect(() => {
    if (!selectedId) {
      stateDirtyRef.current = false;
      stateInteractionRef.current = false;
      setState(emptyState);
      return;
    }
    let cancelled = false;
    stateDirtyRef.current = false;
    stateInteractionRef.current = false;
    setSaved(false);
    void getCharacterState(selectedId)
      .then((loaded) => {
        if (
          !cancelled &&
          !stateDirtyRef.current &&
          !stateInteractionRef.current
        ) {
          setState(loaded);
        }
      })
      .catch((commandError) => {
        if (!cancelled) setError(normalizeCommandError(commandError).message);
      });
    return () => {
      cancelled = true;
    };
  }, [selectedId]);

  async function handleProfileSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selectedCharacter || projectArchived || characterLocked) return;
    setIsSaving(true);
    setError(null);
    setSaved(false);
    try {
      const updated = await updateCharacter(selectedCharacter.id, {
        name: selectedCharacter.name,
        summary: selectedCharacter.summary,
        role: selectedCharacter.role,
      });
      setCharacters((current) =>
        current.map((character) =>
          character.id === updated.id ? updated : character,
        ),
      );
      setSaved(true);
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsSaving(false);
    }
  }

  async function handleArchive() {
    if (!selectedCharacter || projectArchived || characterLocked) return;
    setError(null);
    try {
      await archiveCharacter(selectedCharacter.id);
      await loadCharacters();
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    }
  }

  async function handleStateSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selectedCharacter || projectArchived || stateLocked) return;
    setIsStateSaving(true);
    setError(null);
    setSaved(false);
    const { character_id, updated_at, ...input } = state;
    void character_id;
    void updated_at;
    try {
      setState(await updateCharacterState(selectedCharacter.id, input));
      stateDirtyRef.current = false;
      setSaved(true);
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsStateSaving(false);
    }
  }

  async function handleCreate(input: CreateCharacterInput) {
    setError(null);
    try {
      const created = await createCharacter(projectId, input);
      setCharacters((current) => [...current, created]);
      setSelectedId(created.id);
      setIsDialogOpen(false);
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    }
  }

  function setStateField(
    field: keyof UpdateCharacterStateInput,
    value: string,
  ) {
    stateDirtyRef.current = true;
    setState((current) => ({ ...current, [field]: value }));
  }

  function markStateEditing() {
    stateDirtyRef.current = true;
    stateInteractionRef.current = true;
  }

  return (
    <div className="character-panel">
      <div className="character-panel-header">
        <div>
          <p className="eyebrow">Cast</p>
          <h2>Characters</h2>
        </div>
        <button
          className="button button-secondary button-small"
          type="button"
          onClick={() => setIsDialogOpen(true)}
          disabled={projectArchived}
        >
          Add
        </button>
      </div>

      {error && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}

      {isLoading ? (
        <p className="loading-state" role="status">
          Loading characters…
        </p>
      ) : characters.length === 0 ? (
        <div className="character-empty">
          <p>No characters yet.</p>
          {!projectArchived && (
            <button
              className="button button-primary button-small"
              type="button"
              onClick={() => setIsDialogOpen(true)}
            >
              Create the first character
            </button>
          )}
        </div>
      ) : (
        <div className="character-layout">
          <div className="character-list" aria-label="Characters">
            {characters.map((character) => (
              <button
                className={`character-list-item${character.id === selectedId ? " is-selected" : ""}`}
                key={character.id}
                type="button"
                onClick={() => setSelectedId(character.id)}
              >
                <strong>{character.name}</strong>
                <span>{character.role || "Unassigned role"}</span>
              </button>
            ))}
          </div>

          {selectedCharacter && (
            <div className="character-editor">
              <form className="character-form" onSubmit={handleProfileSubmit}>
                <div className="section-heading">
                  <div>
                    <p className="eyebrow">Profile</p>
                    <h3>{selectedCharacter.name}</h3>
                  </div>
                  {saved && (
                    <span className="saved-message" role="status">
                      Saved
                    </span>
                  )}
                </div>
                <label className="field-label" htmlFor="character-name">
                  Name
                  <input
                    id="character-name"
                    value={selectedCharacter.name}
                    disabled={projectArchived || characterLocked}
                    onChange={(event) =>
                      setCharacters((current) =>
                        current.map((character) =>
                          character.id === selectedCharacter.id
                            ? { ...character, name: event.target.value }
                            : character,
                        ),
                      )
                    }
                  />
                </label>
                <label className="field-label" htmlFor="character-role">
                  Role
                  <input
                    id="character-role"
                    value={selectedCharacter.role}
                    disabled={projectArchived || characterLocked}
                    onChange={(event) =>
                      setCharacters((current) =>
                        current.map((character) =>
                          character.id === selectedCharacter.id
                            ? { ...character, role: event.target.value }
                            : character,
                        ),
                      )
                    }
                  />
                </label>
                <label className="field-label" htmlFor="character-summary">
                  Summary
                  <textarea
                    id="character-summary"
                    rows={3}
                    value={selectedCharacter.summary}
                    disabled={projectArchived || characterLocked}
                    onChange={(event) =>
                      setCharacters((current) =>
                        current.map((character) =>
                          character.id === selectedCharacter.id
                            ? { ...character, summary: event.target.value }
                            : character,
                        ),
                      )
                    }
                  />
                </label>
                {!projectArchived && !characterLocked && (
                  <div className="form-actions">
                    <button
                      className="button button-primary button-small"
                      type="submit"
                      disabled={isSaving}
                    >
                      {isSaving ? "Saving…" : "Save profile"}
                    </button>
                    <button
                      className="button button-danger-ghost button-small"
                      type="button"
                      onClick={() => void handleArchive()}
                    >
                      Archive
                    </button>
                  </div>
                )}
              </form>

              <RevisionHistoryPanel
                entityType="character"
                entityId={selectedCharacter.id}
                currentRevision={selectedCharacter.revision}
                canonStatus={selectedCharacter.canon_status}
                disabled={projectArchived}
                onRestored={() => void loadCharacters()}
              />

              <form
                className="character-state-form"
                onSubmit={handleStateSubmit}
              >
                <div className="section-heading">
                  <div>
                    <p className="eyebrow">Continuity</p>
                    <h3>Current state</h3>
                  </div>
                </div>
                <label className="field-label" htmlFor="character-location">
                  Current location
                  <input
                    id="character-location"
                    value={state.current_location}
                    disabled={projectArchived || stateLocked}
                    onFocus={markStateEditing}
                    onChange={(event) =>
                      setStateField("current_location", event.target.value)
                    }
                  />
                </label>
                <label
                  className="field-label"
                  htmlFor="character-emotional-state"
                >
                  Emotional state
                  <input
                    id="character-emotional-state"
                    value={state.emotional_state}
                    disabled={projectArchived || stateLocked}
                    onFocus={markStateEditing}
                    onChange={(event) =>
                      setStateField("emotional_state", event.target.value)
                    }
                  />
                </label>
                <label className="field-label" htmlFor="character-goals">
                  Goals
                  <textarea
                    id="character-goals"
                    rows={3}
                    value={state.goals}
                    disabled={projectArchived || stateLocked}
                    onFocus={markStateEditing}
                    onChange={(event) =>
                      setStateField("goals", event.target.value)
                    }
                  />
                </label>
                <label className="field-label" htmlFor="character-arc-role">
                  Current arc role
                  <input
                    id="character-arc-role"
                    value={state.current_arc_role}
                    disabled={projectArchived || stateLocked}
                    onFocus={markStateEditing}
                    onChange={(event) =>
                      setStateField("current_arc_role", event.target.value)
                    }
                  />
                </label>
                {!projectArchived && !stateLocked && (
                  <button
                    className="button button-primary button-small"
                    type="submit"
                    disabled={isStateSaving}
                  >
                    {isStateSaving ? "Saving…" : "Save state"}
                  </button>
                )}
              </form>

              <RevisionHistoryPanel
                entityType="character_state"
                entityId={selectedCharacter.id}
                currentRevision={state.revision}
                canonStatus={state.canon_status}
                disabled={projectArchived}
                onRestored={() => {
                  stateDirtyRef.current = false;
                  void getCharacterState(selectedCharacter.id).then(setState);
                }}
              />
            </div>
          )}
        </div>
      )}

      {isDialogOpen && (
        <CharacterDialog
          onCancel={() => setIsDialogOpen(false)}
          onSubmit={(input) => void handleCreate(input)}
        />
      )}
    </div>
  );
}

interface CharacterDialogProps {
  onCancel: () => void;
  onSubmit: (input: CreateCharacterInput) => void;
}

function CharacterDialog({ onCancel, onSubmit }: CharacterDialogProps) {
  const [name, setName] = useState("");
  const [role, setRole] = useState("");
  const [summary, setSummary] = useState("");
  const [error, setError] = useState<string | null>(null);

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!name.trim()) {
      setError("Character name is required.");
      return;
    }
    onSubmit({
      name: name.trim(),
      role: role.trim() || null,
      summary: summary.trim() || null,
    });
  }

  return (
    <div className="dialog-backdrop" role="presentation">
      <section
        aria-labelledby="character-dialog-title"
        aria-modal="true"
        className="dialog"
        role="dialog"
      >
        <p className="eyebrow">New character</p>
        <h2 id="character-dialog-title">Add to the cast</h2>
        <form className="project-form" onSubmit={handleSubmit}>
          <label className="field-label" htmlFor="new-character-name">
            Name
            <input
              autoFocus
              id="new-character-name"
              value={name}
              onChange={(event) => setName(event.target.value)}
            />
          </label>
          <label className="field-label" htmlFor="new-character-role">
            Role
            <input
              id="new-character-role"
              value={role}
              onChange={(event) => setRole(event.target.value)}
            />
          </label>
          <label className="field-label" htmlFor="new-character-summary">
            Summary
            <textarea
              id="new-character-summary"
              rows={3}
              value={summary}
              onChange={(event) => setSummary(event.target.value)}
            />
          </label>
          {error && (
            <p className="form-error" role="alert">
              {error}
            </p>
          )}
          <div className="dialog-actions">
            <button
              className="button button-secondary"
              type="button"
              onClick={onCancel}
            >
              Cancel
            </button>
            <button className="button button-primary" type="submit">
              Create character
            </button>
          </div>
        </form>
      </section>
    </div>
  );
}
