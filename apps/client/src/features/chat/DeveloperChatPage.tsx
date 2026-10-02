import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";

import { normalizeCommandError } from "../../lib/command-error";
import { getProject, listProjects } from "../../lib/commands";
import type { Project } from "../../types/project";
import { DeveloperChatPanel } from "./DeveloperChatPanel";

export function DeveloperChatPage() {
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
        Loading Developer Chat…
      </p>
    );
  }

  if (error) {
    return (
      <section className="page-content error-page" role="alert">
        <p className="eyebrow">Developer Chat unavailable</p>
        <h1>We could not open the chat.</h1>
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
      <section className="page-content chat-picker-page">
        <header className="page-header">
          <div>
            <p className="eyebrow">Project workspace</p>
            <h1>Developer Chat</h1>
            <p className="page-lede">
              Choose a project to continue its durable assistant conversation.
            </p>
          </div>
        </header>
        {projects.length === 0 ? (
          <div className="empty-state">
            <div className="empty-orbit" aria-hidden="true">
              ✦
            </div>
            <h2>No active projects yet</h2>
            <p>Create a project before opening Developer Chat.</p>
            <Link className="button button-secondary" to="/projects">
              Go to projects
            </Link>
          </div>
        ) : (
          <div
            className="project-grid chat-project-grid"
            aria-label="Chat projects"
          >
            {projects.map((item) => (
              <Link
                className="project-card"
                to={`/projects/${item.id}/chat`}
                key={item.id}
              >
                <div className="project-card-topline">
                  <span className="project-status status-active">Active</span>
                  <span className="project-arrow" aria-hidden="true">
                    ↗
                  </span>
                </div>
                <h2>{item.name}</h2>
                <p>{item.description || "Open the project conversation."}</p>
                <span className="project-updated">Open chat</span>
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
    <section className="page-content developer-chat-page">
      <Link className="back-link" to={`/projects/${project.id}`}>
        ← Project workspace
      </Link>
      <header className="page-header developer-chat-header">
        <div>
          <p className="eyebrow">
            {archived ? "Archived project" : "Project workspace"}
          </p>
          <h1>Developer Chat</h1>
          <p className="page-lede">{project.name}</p>
        </div>
        <span className={`project-status status-${project.status}`}>
          {archived ? "Archived" : "Active"}
        </span>
      </header>
      <DeveloperChatPanel projectId={project.id} disabled={archived} />
    </section>
  );
}
