import { type FormEvent, useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";

import { normalizeCommandError } from "../../lib/command-error";
import { createChapter, listChapters } from "../../lib/commands";
import type { Chapter } from "../../types/manuscript";

interface ChapterPanelProps {
  projectId: string;
  projectArchived: boolean;
}

export function ChapterPanel({
  projectId,
  projectArchived,
}: ChapterPanelProps) {
  const [chapters, setChapters] = useState<Chapter[]>([]);
  const [number, setNumber] = useState(1);
  const [title, setTitle] = useState("");
  const [isCreating, setIsCreating] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadChapters = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      setChapters(await listChapters(projectId));
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsLoading(false);
    }
  }, [projectId]);

  useEffect(() => {
    void loadChapters();
  }, [loadChapters]);

  async function handleCreate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (projectArchived || !title.trim()) return;
    setIsCreating(true);
    setError(null);
    try {
      const created = await createChapter(projectId, {
        number,
        title,
      });
      setChapters((current) =>
        [...current, created].sort((a, b) => a.number - b.number),
      );
      setNumber((current) => current + 1);
      setTitle("");
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsCreating(false);
    }
  }

  return (
    <section className="chapter-panel" aria-labelledby="chapter-panel-heading">
      <div className="chapter-panel-header">
        <div>
          <p className="eyebrow">Manuscript</p>
          <h2 id="chapter-panel-heading">Chapters</h2>
        </div>
        <span className="phase-chip">{chapters.length}</span>
      </div>
      {isLoading && <p className="loading-state">Loading chapters…</p>}
      {!isLoading && chapters.length === 0 && (
        <p className="chapter-empty">
          No chapters yet. Give the story its first page.
        </p>
      )}
      {!isLoading && chapters.length > 0 && (
        <div className="chapter-list" aria-label="Chapter list">
          {chapters.map((chapter) => (
            <Link
              className="chapter-list-item"
              to={`/projects/${projectId}/chapters/${chapter.id}`}
              key={chapter.id}
            >
              <span className="chapter-number">
                {String(chapter.number).padStart(2, "0")}
              </span>
              <span>
                <strong>{chapter.title}</strong>
                <small>{chapter.status}</small>
              </span>
              <span aria-hidden="true">→</span>
            </Link>
          ))}
        </div>
      )}
      {!projectArchived && (
        <form className="chapter-create-form" onSubmit={handleCreate}>
          <div className="chapter-create-fields">
            <label className="field-label" htmlFor="new-chapter-number">
              No.
              <input
                id="new-chapter-number"
                min={1}
                type="number"
                value={number}
                onChange={(event) => setNumber(Number(event.target.value))}
              />
            </label>
            <label className="field-label" htmlFor="new-chapter-title">
              Title
              <input
                id="new-chapter-title"
                placeholder="Chapter title"
                value={title}
                onChange={(event) => setTitle(event.target.value)}
              />
            </label>
          </div>
          {error && (
            <p className="form-error" role="alert">
              {error}
            </p>
          )}
          <button
            className="button button-secondary button-small"
            type="submit"
            disabled={isCreating || !title.trim()}
          >
            {isCreating ? "Creating…" : "New chapter"}
          </button>
        </form>
      )}
      {projectArchived && error && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
    </section>
  );
}
