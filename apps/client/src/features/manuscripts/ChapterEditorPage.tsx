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
import { RichTextEditor } from "./rich-text";
import { manuscriptToEditorHtml, sanitizeHtml } from "./rich-text-utils";
import { ChapterChatPanel } from "./ChapterChatPanel";
import { ManuscriptProposalPanel } from "./ManuscriptProposalPanel";

const AUTOSAVE_DELAY_MS = 900;

type SaveStatus = "saved" | "unsaved" | "saving" | "conflict";

interface ConflictState {
  server: Manuscript;
}

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
  const [isDirty, setIsDirty] = useState(false);
  const [saveStatus, setSaveStatus] = useState<SaveStatus>("saved");
  const [conflict, setConflict] = useState<ConflictState | null>(null);
  const [isResolving, setIsResolving] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [isSavingMetadata, setIsSavingMetadata] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [proposalRefreshKey, setProposalRefreshKey] = useState(0);

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
      setContent(
        manuscriptToEditorHtml(
          loadedManuscript.content,
          loadedManuscript.content_format,
        ),
      );
      setIsDirty(false);
      setSaveStatus("saved");
      setConflict(null);
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setIsLoading(false);
    }
  }, [chapterId]);

  useEffect(() => {
    void loadEditor();
  }, [loadEditor]);

  const persistContent = useCallback(
    async (base: Manuscript, nextContent: string, label: string) => {
      if (!chapter || chapter.status === "archived") return false;
      setIsSaving(true);
      setError(null);
      setSaveStatus("saving");
      try {
        const updated = await saveManuscript(chapter.id, {
          content: sanitizeHtml(nextContent),
          content_format: "html",
          label,
          expected_revision: base.revision,
        });
        setManuscript(updated);
        setRevisions(await listManuscriptRevisions(chapter.id));
        setIsDirty(false);
        setSaveStatus("saved");
        return true;
      } catch (commandError) {
        const normalized = normalizeCommandError(commandError);
        if (normalized.code === "conflict") {
          try {
            const server = await getManuscript(chapter.id);
            setConflict({ server });
            setSaveStatus("conflict");
          } catch (reloadError) {
            setError(normalizeCommandError(reloadError).message);
          }
        } else {
          setError(normalized.message);
          setSaveStatus("unsaved");
        }
        return false;
      } finally {
        setIsSaving(false);
      }
    },
    [chapter],
  );

  const saveContent = useCallback(
    async (label: string) => {
      if (!manuscript || !isDirty || conflict) return false;
      return persistContent(manuscript, content, label);
    },
    [conflict, content, isDirty, manuscript, persistContent],
  );

  useEffect(() => {
    if (!isDirty || conflict || !manuscript || chapter?.status === "archived") {
      return;
    }
    const timer = window.setTimeout(() => {
      void saveContent(`Autosave ${manuscript.revision + 1}`);
    }, AUTOSAVE_DELAY_MS);
    return () => window.clearTimeout(timer);
  }, [chapter?.status, conflict, isDirty, manuscript, saveContent]);

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
    if (!manuscript) return;
    await saveContent(`Draft ${manuscript.revision + 1}`);
  }

  function handleContentChange(nextContent: string) {
    setContent(nextContent);
    setIsDirty(true);
    setSaveStatus("unsaved");
    setError(null);
  }

  function useServerCopy() {
    if (!conflict) return;
    setManuscript(conflict.server);
    setContent(
      manuscriptToEditorHtml(
        conflict.server.content,
        conflict.server.content_format,
      ),
    );
    setConflict(null);
    setIsDirty(false);
    setSaveStatus("saved");
    setError(null);
  }

  async function keepLocalCopy() {
    if (!conflict) return;
    setIsResolving(true);
    const resolved = await persistContent(
      conflict.server,
      content,
      `Conflict resolution ${conflict.server.revision + 1}`,
    );
    if (resolved) setConflict(null);
    setIsResolving(false);
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
      setContent(
        manuscriptToEditorHtml(restored.content, restored.content_format),
      );
      setRevisions(await listManuscriptRevisions(chapter.id));
      setIsDirty(false);
      setSaveStatus("saved");
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    }
  }

  function handleProposalPromoted(updated: Manuscript) {
    setManuscript(updated);
    setContent(manuscriptToEditorHtml(updated.content, updated.content_format));
    setRevisions((current) => [
      {
        id: `revision-${updated.revision}`,
        manuscript_id: updated.id,
        revision: updated.revision,
        content: updated.content,
        content_format: updated.content_format,
        label: "AI proposal",
        actor_type: "ai",
        actor_id: "chapter-chat",
        created_at: updated.updated_at,
      },
      ...current,
    ]);
    setIsDirty(false);
    setSaveStatus("saved");
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
        <div className="form-actions">
          <button
            className="button button-primary"
            type="button"
            onClick={() => void loadEditor()}
          >
            Try again
          </button>
          <Link
            className="button button-secondary"
            to={`/projects/${projectId ?? ""}`}
          >
            Back to project
          </Link>
        </div>
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
        <span className={`editor-save-status is-${saveStatus}`} role="status">
          {saveStatus === "saving" && "Saving…"}
          {saveStatus === "unsaved" && "Unsaved changes"}
          {saveStatus === "conflict" && "Conflict needs review"}
          {saveStatus === "saved" && `Saved revision ${manuscript.revision}`}
        </span>
      </header>
      {conflict && (
        <section className="editor-conflict" role="alert">
          <div>
            <p className="eyebrow">Concurrent edit detected</p>
            <h2>
              Your local copy is based on revision {manuscript.revision}, but
              the server is now at revision {conflict.server.revision}.
            </h2>
            <p>Choose a copy before writing again. Nothing was overwritten.</p>
          </div>
          <div className="editor-conflict-actions">
            <button
              className="button button-ghost"
              type="button"
              onClick={useServerCopy}
              disabled={isResolving}
            >
              Use server copy
            </button>
            <button
              className="button button-primary"
              type="button"
              onClick={() => void keepLocalCopy()}
              disabled={isResolving}
            >
              {isResolving ? "Saving local copy…" : "Keep my local copy"}
            </button>
          </div>
        </section>
      )}
      <div className="editor-grid">
        <div className="editor-primary">
          <form
            className="editor-manuscript-form"
            onSubmit={(event) => void handleContentSubmit(event)}
          >
            <RichTextEditor
              value={content}
              disabled={archived || Boolean(conflict)}
              onChange={handleContentChange}
            />
            {error && (
              <p className="form-error" role="alert">
                {error}
              </p>
            )}
            <div className="editor-actions">
              <span className="editor-revision">
                Revision {manuscript.revision} · HTML
              </span>
              <button
                className="button button-primary"
                type="submit"
                disabled={archived || isSaving || !isDirty || Boolean(conflict)}
              >
                {isSaving ? "Saving…" : "Save manuscript"}
              </button>
            </div>
          </form>
          <ChapterChatPanel
            projectId={chapter.project_id}
            chapterId={chapter.id}
            currentRevision={manuscript.revision}
            disabled={archived}
            onProposalCreated={() => setProposalRefreshKey((key) => key + 1)}
          />
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
                    <small>
                      {revision.label} · {revision.content_format}
                    </small>
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
          <ManuscriptProposalPanel
            chapterId={chapter.id}
            currentRevision={manuscript.revision}
            disabled={archived}
            refreshKey={proposalRefreshKey}
            onPromoted={handleProposalPromoted}
          />
        </aside>
      </div>
    </section>
  );
}
