import { useCallback, useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";

import { normalizeCommandError } from "../../lib/command-error";
import { listProjects } from "../../lib/commands";
import type { Project } from "../../types/project";
import { NewProjectDialog } from "./NewProjectDialog";
import { projectStatusLabel, projectUpdatedLabel } from "./project-view-model";

export function ProjectsPage() {
  const navigate = useNavigate();
  const [projects, setProjects] = useState<Project[]>([]);
  const [includeArchived, setIncludeArchived] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isDialogOpen, setIsDialogOpen] = useState(false);

  const loadProjects = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      setProjects(await listProjects({ include_archived: includeArchived }));
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsLoading(false);
    }
  }, [includeArchived]);

  useEffect(() => {
    void loadProjects();
  }, [loadProjects]);

  function handleCreated(project: Project) {
    setProjects((current) => [project, ...current]);
    setIsDialogOpen(false);
    navigate(`/projects/${project.id}`);
  }

  return (
    <section className="page-content projects-page">
      <header className="page-header">
        <div>
          <p className="eyebrow">Your worlds</p>
          <h1>Projects</h1>
          <p className="page-lede">
            A quiet place to build stories that remember where they came from.
          </p>
        </div>
        <button
          className="button button-primary"
          type="button"
          onClick={() => setIsDialogOpen(true)}
        >
          <span aria-hidden="true">+</span> New project
        </button>
      </header>

      <div className="page-toolbar">
        <span className="toolbar-label">
          {includeArchived ? "All projects" : "Active projects"}
        </span>
        <button
          className="filter-button"
          type="button"
          aria-pressed={includeArchived}
          onClick={() => setIncludeArchived((current) => !current)}
        >
          {includeArchived ? "Hide archived" : "Show archived"}
        </button>
      </div>

      {isLoading && (
        <p className="loading-state" role="status">
          Loading projects…
        </p>
      )}
      {!isLoading && error && (
        <div className="error-state" role="alert">
          <strong>Projects could not be loaded.</strong>
          <span>{error}</span>
          <button
            className="button button-ghost"
            type="button"
            onClick={() => void loadProjects()}
          >
            Try again
          </button>
        </div>
      )}
      {!isLoading && !error && projects.length === 0 && (
        <div className="empty-state" data-testid="empty-projects">
          <div className="empty-orbit" aria-hidden="true">
            ✦
          </div>
          <h2>
            {includeArchived
              ? "No projects yet"
              : "Your first story starts here"}
          </h2>
          <p>
            Create a project to give your characters, worlds and ideas a home.
          </p>
          <button
            className="button button-secondary"
            type="button"
            onClick={() => setIsDialogOpen(true)}
          >
            Create your first project
          </button>
        </div>
      )}
      {!isLoading && !error && projects.length > 0 && (
        <div className="project-grid" aria-label="Projects list">
          {projects.map((project) => (
            <Link
              className={`project-card ${project.status === "archived" ? "project-card-archived" : ""}`}
              to={`/projects/${project.id}`}
              key={project.id}
            >
              <div className="project-card-topline">
                <span className={`project-status status-${project.status}`}>
                  {projectStatusLabel(project)}
                </span>
                <span className="project-arrow" aria-hidden="true">
                  ↗
                </span>
              </div>
              <h2>{project.name}</h2>
              <p>{project.description || "No premise added yet."}</p>
              <span className="project-updated">
                {projectUpdatedLabel(project)}
              </span>
            </Link>
          ))}
        </div>
      )}

      {isDialogOpen && (
        <NewProjectDialog
          onClose={() => setIsDialogOpen(false)}
          onCreated={handleCreated}
        />
      )}
    </section>
  );
}
