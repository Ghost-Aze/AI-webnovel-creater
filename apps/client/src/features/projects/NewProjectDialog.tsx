import { type FormEvent, useState } from "react";

import { normalizeCommandError } from "../../lib/command-error";
import { createProject } from "../../lib/commands";
import type { CreateProjectInput, Project } from "../../types/project";

interface NewProjectDialogProps {
  onClose: () => void;
  onCreated: (project: Project) => void;
}

export function NewProjectDialog({
  onClose,
  onCreated,
}: NewProjectDialogProps) {
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!name.trim()) {
      setError("Project name is required.");
      return;
    }

    setError(null);
    setIsSubmitting(true);
    const input: CreateProjectInput = { name, description };
    try {
      const project = await createProject(input);
      onCreated(project);
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div
      className="dialog-backdrop"
      role="presentation"
      onMouseDown={(event) => event.target === event.currentTarget && onClose()}
    >
      <section
        className="dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="new-project-title"
      >
        <div className="dialog-header">
          <div>
            <p className="eyebrow">Start a new world</p>
            <h2 id="new-project-title">Create project</h2>
          </div>
          <button
            className="icon-button"
            type="button"
            onClick={onClose}
            aria-label="Close dialog"
          >
            ×
          </button>
        </div>
        <form onSubmit={handleSubmit}>
          <label className="field-label" htmlFor="project-name">
            Project name
            <input
              autoFocus
              id="project-name"
              name="name"
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder="The name of your story"
              aria-invalid={Boolean(error)}
            />
          </label>
          <label className="field-label" htmlFor="project-description">
            Premise <span className="field-optional">Optional</span>
            <textarea
              id="project-description"
              name="description"
              value={description}
              onChange={(event) => setDescription(event.target.value)}
              placeholder="A sentence to anchor the story"
              rows={4}
            />
          </label>
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
            >
              Cancel
            </button>
            <button
              className="button button-primary"
              type="submit"
              disabled={isSubmitting}
            >
              {isSubmitting ? "Creating…" : "Create project"}
            </button>
          </div>
        </form>
      </section>
    </div>
  );
}
