import { type FormEvent, useCallback, useEffect, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";

import { normalizeCommandError } from "../../lib/command-error";
import { archiveProject, getProject, updateProject } from "../../lib/commands";
import type { Project } from "../../types/project";

export function ProjectWorkspacePage() {
  const { projectId } = useParams<{ projectId: string }>();
  const navigate = useNavigate();
  const [project, setProject] = useState<Project | null>(null);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  const loadProject = useCallback(async () => {
    if (!projectId) return;
    setIsLoading(true);
    setError(null);
    try {
      const loaded = await getProject(projectId);
      setProject(loaded);
      setName(loaded.name);
      setDescription(loaded.description);
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsLoading(false);
    }
  }, [projectId]);

  useEffect(() => {
    void loadProject();
  }, [loadProject]);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!projectId || !project) return;
    setIsSaving(true);
    setError(null);
    setSaved(false);
    try {
      const updated = await updateProject(projectId, { name, description });
      setProject(updated);
      setName(updated.name);
      setDescription(updated.description);
      setSaved(true);
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsSaving(false);
    }
  }

  async function handleArchive() {
    if (!projectId || !project || project.status === "archived") return;
    setError(null);
    try {
      await archiveProject(projectId);
      navigate("/projects");
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    }
  }

  if (isLoading)
    return (
      <p className="loading-state page-content" role="status">
        Loading project…
      </p>
    );
  if (error && !project) {
    return (
      <section className="page-content error-page" role="alert">
        <p className="eyebrow">Project unavailable</p>
        <h1>We could not open this project.</h1>
        <p>{error}</p>
        <Link className="button button-secondary" to="/projects">
          Back to projects
        </Link>
      </section>
    );
  }
  if (!project) return null;

  const archived = project.status === "archived";
  return (
    <section className="page-content workspace-page">
      <Link className="back-link" to="/projects">
        ← All projects
      </Link>
      <header className="workspace-header">
        <div>
          <p className="eyebrow">Project workspace</p>
          <h1>{project.name}</h1>
          <span className={`project-status status-${project.status}`}>
            {archived ? "Archived" : "Active"}
          </span>
        </div>
        {!archived && (
          <button
            className="button button-danger-ghost"
            type="button"
            onClick={() => void handleArchive()}
          >
            Archive project
          </button>
        )}
      </header>

      <div className="workspace-grid">
        <div className="workspace-primary">
          <div className="section-heading">
            <div>
              <p className="eyebrow">Project details</p>
              <h2>Shape the starting point</h2>
            </div>
            {saved && (
              <span className="saved-message" role="status">
                Saved
              </span>
            )}
          </div>
          <form className="project-form" onSubmit={handleSubmit}>
            <label className="field-label" htmlFor="workspace-project-name">
              Project name
              <input
                id="workspace-project-name"
                value={name}
                onChange={(event) => setName(event.target.value)}
                disabled={archived}
              />
            </label>
            <label
              className="field-label"
              htmlFor="workspace-project-description"
            >
              Premise
              <textarea
                id="workspace-project-description"
                rows={7}
                value={description}
                onChange={(event) => setDescription(event.target.value)}
                disabled={archived}
              />
            </label>
            {error && (
              <p className="form-error" role="alert">
                {error}
              </p>
            )}
            {!archived && (
              <button
                className="button button-primary"
                type="submit"
                disabled={isSaving}
              >
                {isSaving ? "Saving…" : "Save changes"}
              </button>
            )}
          </form>
        </div>
        <div className="workspace-placeholder">
          <p className="eyebrow">Next steps</p>
          <h2>Build the foundation</h2>
          <p>
            Characters, arcs and manuscript tools will live here as the studio
            grows.
          </p>
          <span className="phase-chip">Available in Phase 1</span>
        </div>
      </div>
    </section>
  );
}
