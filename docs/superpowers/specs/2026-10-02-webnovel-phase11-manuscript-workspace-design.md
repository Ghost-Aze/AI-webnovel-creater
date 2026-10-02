# Phase 11: Manuscript Workspace and Chapter Editor

## Status

Approved implementation slice on `phase11/implementation`.

## Goal

Give each project a durable manuscript workspace where chapters are managed as
structured records and their prose is stored in a dedicated document with
revision-safe saves. Chat messages remain separate from manuscript content.

## Scope

- SQLite migrations for projects, chapters, manuscript documents and immutable
  manuscript revision snapshots;
- typed chapter/manuscript domain, repository and service operations;
- optimistic-concurrency saves using an expected manuscript revision;
- chapter and manuscript Tauri commands plus TypeScript command contracts;
- project workspace chapter list/create affordance;
- a focused chapter editor route with title, synopsis and manuscript content;
- revision list and restore action for manuscript snapshots;
- Rust, frontend and command-boundary tests.

## Non-goals

- rich-text/ProseMirror integration, autosave or collaborative editing;
- AI writing, chat streaming, automatic memory extraction or continuity checks;
- arcs, scenes, sync/conflict transport or export formats;
- provider-specific manuscript behavior.

## Data contract

`Chapter` belongs to a project and has a unique positive number within that
project. Chapter metadata is versioned by `updated_at` and can be archived
without deleting its manuscript.

`Manuscript` is a one-to-one document for a chapter. Its current content and
monotonic `revision` are stored separately from chat history. Every successful
save creates an immutable `ManuscriptRevision` snapshot containing content,
label and actor metadata. A save must provide the revision the editor last
loaded; stale saves return a conflict and never overwrite newer prose.

Restoring a snapshot is an explicit save that creates a new revision, preserving
the full history rather than rewriting old snapshots.

## Validation

- migration is idempotent and enforces project ownership, positive chapter
  numbers and one manuscript per chapter;
- repository/service tests cover chapter lifecycle, archived projects,
  revision ordering, stale-save conflicts and restore behavior;
- frontend tests cover chapter loading, editing, saving and restore feedback;
- existing Rust, frontend, lint and build checks remain required.
