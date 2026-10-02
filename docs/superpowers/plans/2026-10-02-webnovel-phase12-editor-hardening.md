# Phase 12 implementation plan

## 1. Format-aware persistence

- Add `0006_manuscript_content_format.sql` with a safe plain-text default.
- Extend manuscript/revision domain values, repository snapshots and commands.
- Preserve optimistic-concurrency behavior and explicit restore semantics.

## 2. Editor interaction

- Add a sanitized HTML utility and a focused rich-text toolbar component.
- Replace the Phase 11 textarea with a contenteditable editor while retaining
  keyboard-accessible controls and a plain-text fallback for old documents.
- Add debounced autosave and clear save/conflict status.

## 3. Conflict handling

- Fetch the server document on a conflict and display both revision numbers.
- Provide explicit `Use server copy` and `Keep my local copy` actions.
- Ensure no conflict path silently overwrites content.

## 4. Verification

- Add Rust migration/service tests and frontend component/command tests.
- Run format, clippy, Rust tests, typecheck, lint, Vitest, production build and
  a Vite smoke request.
- Commit and push only `phase12/implementation`; do not merge `main`.
