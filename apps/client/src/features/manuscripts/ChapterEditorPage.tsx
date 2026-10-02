import { type FormEvent, useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";

import { normalizeCommandError } from "../../lib/command-error";
import {
  getChapter,
  getManuscript,
  listManuscriptRevisions,
  restoreManuscript,
  saveManuscript,
  updateChapter,
} from "../../lib/commands";
import type {
  Chapter,
  ChapterStatus,
  Manuscript,
  ManuscriptRevision,
} from "../../types/manuscript";

export function ChapterEditorPage() {
  const { projectId, chapterId } = useParams<{
    projectId: string;
    chapterId: string;
  }>();
  const [chapter, setChapter] = useState<Chapter | null>(null);
  const [manuscript, setManuscript] = useState<Manuscript | null>(null);
  const [revisions, setRevisions] = useState<ManuscriptRevision[]>([]);
  const [title, setTitle] = useState("");
  const [synopsis, setSynopsis] = useState("");
  const [status, setStatus] = useState<ChapterStatus>("draft");
  const [content, setContent] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [isSavingMetadata, setIsSavingMetadata] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  const loadEditor = useCallback(async () => {
    if (!chapterId) return;
    setIsLoading(true);
    setError(null);
    try {
      const [loadedChapter, loadedManuscript, loadedRevisions] =
        await Promise.all([
          getChapter(chapterId),
          getManuscript(chapterId),
          listManuscriptRevisions(chapterId),
        ]);
      setChapter(loadedChapter);
      setManuscript(loadedManuscript);
      setRevisions(loadedRevisions);
      setTitle(loadedChapter.title);
      setSynopsis(loadedChapter.synopsis);
      setStatus(loadedChapter.status);
      setContent(loadedManuscript.content);
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsLoading(false);
    }
  }, [chapterId]);

  useEffect(() => {
    void loadEditor();
  }, [loadEditor]);

  async function handleMetadataSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!chapter) return;
    setIsSavingMetadata(true);
    setError(null);
    try {
      const updated = await updateChapter(
        chapter.id,
        { number: chapter.number, title, synopsis, status },
        chapter.revision,
      );
      setChapter(updated);
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsSavingMetadata(false);
    }
  }

  async function handleContentSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!chapter || !manuscript || chapter.status === "archived") return;
    setIsSaving(true);
    setSaved(false);
    setError(null);
    try {
      const updated = await saveManuscript(chapter.id, {
        content,
        label: `Draft ${manuscript.revision + 1}`,
        expected_revision: manuscript.revision,
      });
      setManuscript(updated);
      setRevisions(await listManuscriptRevisions(chapter.id));
      setSaved(true);
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsSaving(false);
    }
  }

  async function handleRestore(revision: number) {
    if (!chapter || !manuscript || chapter.status === "archived") return;
    setError(null);
    try {
      const restored = await restoreManuscript(
        chapter.id,
        revision,
        manuscript.revision,
      );
      setManuscript(restored);
      setContent(restored.content);
      setRevisions(await listManuscriptRevisions(chapter.id));
      setSaved(true);
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    }
  }

  if (isLoading)
    return (
      <p className="loading-state page-content" role="status">
        Loading chapter…
      </p>
    );
  if (error && (!chapter || !manuscript)) {
    return (
      <section className="page-content error-page" role="alert">
        <p className="eyebrow">Chapter unavailable</p>
        <h1>We could not open this chapter.</h1>
        <p>{error}</p>
        <Link
          className="button button-secondary"
          to={`/projects/${projectId ?? ""}`}
        >
          Back to project
        </Link>
      </section>
    );
  }
  if (!chapter || !manuscript) return null;

  const archived = chapter.status === "archived";
  return (
    <section className="page-content editor-page">
      <Link
        className="back-link"
        to={`/projects/${projectId ?? chapter.project_id}`}
      >
        ← Project workspace
      </Link>
      <header className="editor-header">
        <div>
          <p className="eyebrow">Chapter {chapter.number}</p>
          <h1>{chapter.title}</h1>
          <span className={`project-status status-${chapter.status}`}>
            {chapter.status}
          </span>
        </div>
        {saved && (
          <span className="saved-message" role="status">
            Saved revision {manuscript.revision}
          </span>
        )}
      </header>
      <div className="editor-grid">
        <div className="editor-primary">
          <form
            className="editor-manuscript-form"
            onSubmit={handleContentSubmit}
          >
            <label className="field-label" htmlFor="chapter-content">
              Manuscript
              <textarea
                id="chapter-content"
                className="manuscript-editor"
                value={content}
                onChange={(event) => setContent(event.target.value)}
                disabled={archived}
                placeholder="Begin the chapter…"
              />
            </label>
            {error && (
              <p className="form-error" role="alert">
                {error}
              </p>
            )}
            <div className="editor-actions">
              <span className="editor-revision">
                Revision {manuscript.revision}
              </span>
              <button
                className="button button-primary"
                type="submit"
                disabled={archived || isSaving}
              >
                {isSaving ? "Saving…" : "Save manuscript"}
              </button>
            </div>
          </form>
        </div>
        <aside className="editor-side-stack">
          <form
            className="workspace-primary editor-metadata-form"
            onSubmit={handleMetadataSubmit}
          >
            <div className="section-heading">
              <div>
                <p className="eyebrow">Chapter setup</p>
                <h2>Metadata</h2>
              </div>
            </div>
            <label className="field-label" htmlFor="chapter-title">
              Title
              <input
                id="chapter-title"
                value={title}
                onChange={(event) => setTitle(event.target.value)}
                disabled={archived}
              />
            </label>
            <label className="field-label" htmlFor="chapter-synopsis">
              Synopsis
              <textarea
                id="chapter-synopsis"
                rows={5}
                value={synopsis}
                onChange={(event) => setSynopsis(event.target.value)}
                disabled={archived}
              />
            </label>
            <label className="field-label" htmlFor="chapter-status">
              Status
              <select
                id="chapter-status"
                value={status}
                onChange={(event) =>
                  setStatus(event.target.value as ChapterStatus)
                }
                disabled={archived}
              >
                <option value="draft">Draft</option>
                <option value="final">Final</option>
                <option value="archived">Archived</option>
              </select>
            </label>
            {!archived && (
              <button
                className="button button-secondary"
                type="submit"
                disabled={isSavingMetadata}
              >
                {isSavingMetadata ? "Saving…" : "Save metadata"}
              </button>
            )}
          </form>
          <section
            className="workspace-primary editor-revisions"
            aria-labelledby="revision-heading"
          >
            <div className="section-heading">
              <div>
                <p className="eyebrow">History</p>
                <h2 id="revision-heading">Revisions</h2>
              </div>
            </div>
            <div className="editor-revision-list">
              {revisions.map((revision) => (
                <div className="editor-revision-item" key={revision.id}>
                  <div>
                    <strong>Revision {revision.revision}</strong>
                    <small>{revision.label}</small>
                  </div>
                  <button
                    className="button button-ghost button-small"
                    type="button"
                    onClick={() => void handleRestore(revision.revision)}
                    disabled={
                      archived || revision.revision === manuscript.revision
                    }
                  >
                    Restore
                  </button>
                </div>
              ))}
            </div>
          </section>
        </aside>
      </div>
    </section>
  );
}
