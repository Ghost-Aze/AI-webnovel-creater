import { useEffect, useState } from "react";

import { normalizeCommandError } from "../../lib/command-error";
import {
  listMemoryHistory,
  restoreMemory,
  setMemoryCanonStatus,
} from "../../lib/commands";
import type { CanonStatus } from "../../types/character";
import type { MemoryEntityType, Revision } from "../../types/revision";
import { RevisionDiff } from "./RevisionDiff";

export interface RevisionHistoryPanelProps {
  entityType: MemoryEntityType;
  entityId: string;
  currentRevision: number;
  canonStatus: CanonStatus;
  onRestored: (revision: Revision) => void;
  disabled: boolean;
}

export function RevisionHistoryPanel({
  entityType,
  entityId,
  currentRevision,
  canonStatus,
  onRestored,
  disabled,
}: RevisionHistoryPanelProps) {
  const [revisions, setRevisions] = useState<Revision[]>([]);
  const [selected, setSelected] = useState<Revision | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [reloadToken, setReloadToken] = useState(0);
  const [confirmRestore, setConfirmRestore] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setIsLoading(true);
    setError(null);
    void listMemoryHistory(entityType, entityId)
      .then((loaded) => {
        if (cancelled) return;
        setRevisions(loaded);
        setSelected(loaded.at(-1) ?? null);
      })
      .catch((commandError) => {
        if (!cancelled) setError(normalizeCommandError(commandError).message);
      })
      .finally(() => {
        if (!cancelled) setIsLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [entityType, entityId, currentRevision, reloadToken]);

  async function handleRestore() {
    if (!selected || disabled || canonStatus === "locked_canon") return;
    setIsSaving(true);
    setError(null);
    try {
      const restored = await restoreMemory(
        entityType,
        entityId,
        selected.revision,
        currentRevision,
      );
      setConfirmRestore(false);
      onRestored(restored);
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsSaving(false);
    }
  }

  async function handleCanonStatus() {
    if (disabled) return;
    setIsSaving(true);
    setError(null);
    try {
      const updated = await setMemoryCanonStatus(
        entityType,
        entityId,
        canonStatus === "locked_canon" ? "canon" : "locked_canon",
        currentRevision,
      );
      onRestored(updated);
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <section className="revision-history" aria-label="Revision history">
      <div className="revision-history-heading">
        <div>
          <p className="eyebrow">Canon</p>
          <h4>Revision history</h4>
        </div>
        <span className={`canon-status canon-status-${canonStatus}`}>
          {canonStatus === "locked_canon" ? "Locked canon" : "Canon"}
        </span>
      </div>
      {error && (
        <div className="error-state revision-conflict" role="alert">
          <span>{error}</span>
          <button
            className="button button-ghost button-small"
            type="button"
            onClick={() => setReloadToken((current) => current + 1)}
            disabled={isSaving}
          >
            Try again
          </button>
        </div>
      )}
      {isLoading ? (
        <p className="loading-state">Loading history…</p>
      ) : revisions.length === 0 ? (
        <p className="revision-history-empty">No revisions yet.</p>
      ) : (
        <>
          <div className="revision-history-list" role="list">
            {revisions.map((revision) => (
              <button
                className={`revision-history-item${selected?.id === revision.id ? " is-selected" : ""}`}
                key={revision.id}
                type="button"
                onClick={() => {
                  setSelected(revision);
                  setConfirmRestore(false);
                }}
              >
                <span>Revision {revision.revision}</span>
                <small>{revision.operation}</small>
              </button>
            ))}
          </div>
          {selected && (
            <div className="revision-history-detail">
              <p className="revision-metadata">
                Revision {selected.revision} · {selected.operation} ·{" "}
                {selected.actor_type}
              </p>
              <RevisionDiff revision={selected} />
              <div className="revision-history-actions">
                {confirmRestore ? (
                  <button
                    className="button button-primary button-small"
                    type="button"
                    onClick={() => void handleRestore()}
                    disabled={
                      isSaving || disabled || canonStatus === "locked_canon"
                    }
                  >
                    Confirm restore
                  </button>
                ) : (
                  <button
                    className="button button-secondary button-small"
                    type="button"
                    onClick={() => setConfirmRestore(true)}
                    disabled={
                      isSaving || disabled || canonStatus === "locked_canon"
                    }
                  >
                    Restore revision
                  </button>
                )}
                <button
                  className="button button-secondary button-small"
                  type="button"
                  onClick={() => void handleCanonStatus()}
                  disabled={isSaving || disabled}
                >
                  {canonStatus === "locked_canon"
                    ? "Unlock canon"
                    : "Lock canon"}
                </button>
              </div>
            </div>
          )}
        </>
      )}
    </section>
  );
}
