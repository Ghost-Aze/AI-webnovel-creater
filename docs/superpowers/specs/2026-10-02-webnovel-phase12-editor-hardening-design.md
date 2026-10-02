# Phase 12: Rich Text, Autosave and Manuscript Conflicts

## Status

Approved implementation slice on `phase12/implementation`.

## Goal

Make the Phase 11 manuscript workspace safer for continuous writing without
coupling it to an editor vendor or silently overwriting another device's work.

## Scope

- a migration that labels existing manuscript bodies as plain text and adds an
  explicit `plain_text`/`html` content format;
- format-aware manuscript and revision command contracts;
- a small provider-independent rich-text editor surface with bold, italic,
  headings, lists, quote and undo/redo controls;
- sanitized HTML serialization at the client boundary;
- debounced autosave with explicit saved/unsaved status;
- visible optimistic-concurrency conflict state with `Use server copy` and
  `Keep my local copy` actions;
- tests for format persistence, autosave/conflict behavior and editor commands.

## Non-goals

- collaborative cursors or real-time synchronization;
- automatic merge algorithms for prose;
- AI writing, streaming, semantic retrieval or memory extraction;
- export formats, media embeds or a full TipTap/ProseMirror schema;
- background autosave while the app is closed.

## Data contract

`Manuscript.content_format` and `ManuscriptRevision.content_format` are typed
values. Existing rows migrate to `plain_text`; new editor saves use `html`.
The backend stores the body opaquely and continues to enforce expected-revision
checks in one transaction.

Autosave is a normal `manuscript_save` with a debounce. If the expected revision
is stale, the editor fetches the current server document and stops writing until
the user explicitly chooses a resolution. Keeping the local copy submits it as
a new revision against the fetched server revision; using the server copy
replaces local edits without creating a misleading revision.

## Validation

- migration is idempotent and preserves all existing manuscript content;
- Rust tests cover format round-trips, format-aware snapshots and stale saves;
- frontend tests cover toolbar serialization, debounced save and conflict
  resolution actions;
- existing Rust, frontend, lint and build checks remain required.
