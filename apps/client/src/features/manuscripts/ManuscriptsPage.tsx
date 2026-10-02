import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";

import { normalizeCommandError } from "../../lib/command-error";
import { getProject, listProjects } from "../../lib/commands";
import type { Project } from "../../types/project";
import { ChapterPanel } from "./ChapterPanel";

export function ManuscriptsPage() {
  const { projectId } = useParams<{ projectId: string }>();
  const [project, setProject] = useState<Project | null>(null);
  const [projects, setProjects] = useState<Project[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      if (projectId) {
        setProject(await getProject(projectId));
      } else {
        setProjects(await listProjects({ include_archived: false }));
      }
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsLoading(false);
    }
  }, [projectId]);

  useEffect(() => {
    void load();
  }, [load]);

  if (isLoading) {
    return (
      <p className="loading-state page-content" role="status">
        Loading manuscripts…
      </p>
    );
  }

  if (error) {
    return (
      <section className="page-content error-page" role="alert">
        <p className="eyebrow">Manuscripts unavailable</p>
        <h1>We could not open the manuscripts.</h1>
        <p>{error}</p>
        <button
          className="button button-primary"
          type="button"
          onClick={() => void load()}
        >
          Try again
        </button>
      </section>
    );
  }

  if (!projectId) {
    return (
      <section className="page-content manuscripts-picker-page">
        <header className="page-header">
          <div>
            <p className="eyebrow">Writing workspace</p>
            <h1>Manuscripts</h1>
            <p className="page-lede">
              Choose a project to manage chapters and open its editor.
            </p>
          </div>
        </header>
        {projects.length === 0 ? (
          <div className="empty-state">
            <div className="empty-orbit" aria-hidden="true">
              ◈
            </div>
            <h2>No active projects yet</h2>
            <p>Create a project before opening a manuscript.</p>
            <Link className="button button-secondary" to="/projects">
              Go to projects
            </Link>
          </div>
        ) : (
          <div
            className="project-grid manuscripts-project-grid"
            aria-label="Manuscript projects"
          >
            {projects.map((item) => (
              <Link
                className="project-card"
                to={`/projects/${item.id}/manuscripts`}
                key={item.id}
              >
                <div className="project-card-topline">
                  <span className="project-status status-active">Active</span>
                  <span className="project-arrow" aria-hidden="true">
                    ↗
                  </span>
                </div>
                <h2>{item.name}</h2>
                <p>{item.description || "Open the chapter workspace."}</p>
                <span className="project-updated">Open manuscript</span>
              </Link>
            ))}
          </div>
        )}
      </section>
    );
  }

  if (!project) return null;
  const archived = project.status === "archived";
  return (
    <section className="page-content manuscripts-page">
      <Link className="back-link" to={`/projects/${project.id}`}>
        ← Project workspace
      </Link>
      <header className="page-header manuscripts-header">
        <div>
          <p className="eyebrow">
            {archived ? "Archived project" : "Writing workspace"}
          </p>
          <h1>Manuscripts</h1>
          <p className="page-lede">{project.name}</p>
        </div>
        <span className={`project-status status-${project.status}`}>
          {archived ? "Archived" : "Active"}
        </span>
      </header>
      <ChapterPanel projectId={project.id} projectArchived={archived} />
    </section>
  );
}
