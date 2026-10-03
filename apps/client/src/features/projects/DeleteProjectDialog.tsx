import { useState } from "react";

import { normalizeCommandError } from "../../lib/command-error";
import { deleteProject } from "../../lib/commands";
import type { Project } from "../../types/project";

interface DeleteProjectDialogProps {
  project: Project;
  onClose: () => void;
  onDeleted: () => void;
}

export function DeleteProjectDialog({
  project,
  onClose,
  onDeleted,
}: DeleteProjectDialogProps) {
  const [error, setError] = useState<string | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  async function handleDelete() {
    setIsDeleting(true);
    setError(null);
    try {
      await deleteProject(project.id);
      onDeleted();
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsDeleting(false);
    }
  }

  return (
    <div
      className="dialog-backdrop"
      role="presentation"
      onMouseDown={(event) => event.target === event.currentTarget && onClose()}
    >
      <section
        className="dialog destructive-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="delete-project-title"
      >
        <div className="dialog-header">
          <div>
            <p className="eyebrow">Permanent action</p>
            <h2 id="delete-project-title">Delete project?</h2>
          </div>
          <button
            className="icon-button"
            type="button"
            onClick={onClose}
            aria-label="Close dialog"
            disabled={isDeleting}
          >
            ×
          </button>
        </div>
        <p className="destructive-dialog-copy">
          <strong>{project.name}</strong> and its chapters, chats, characters,
          revisions and project memory will be permanently deleted.
        </p>
        {error && (
          <p className="form-error" role="alert">
            {error}
          </p>
        )}
        <div className="dialog-actions">
          <button
            className="button button-ghost"
            type="button"
            onClick={onClose}
            disabled={isDeleting}
          >
            Cancel
          </button>
          <button
            className="button button-danger"
            type="button"
            onClick={() => void handleDelete()}
            disabled={isDeleting}
          >
            {isDeleting ? "Deleting…" : "Delete permanently"}
          </button>
        </div>
      </section>
    </div>
  );
}
