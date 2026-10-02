# Phase 11 implementation plan

## 1. Persistence and domain

- Add `0005_create_manuscripts.sql` with chapters, manuscripts and immutable
  manuscript revisions.
- Add typed chapter/manuscript entities and repository/service boundaries.
- Enforce archived-project and optimistic-concurrency rules in the service.

## 2. Commands and contracts

- Expose chapter create/list/get/update/archive commands.
- Expose manuscript get/save/list-revisions/restore commands.
- Add TypeScript types and typed command wrappers with error normalization.

## 3. Workspace UI

- Add a chapter list/create panel to the project workspace.
- Add a chapter editor route with metadata fields, plain-text manuscript editor,
  save status and revision restore controls.
- Keep the editor intentionally plain-text until a later editor-focused phase.

## 4. Verification

- Add Rust unit/integration tests and frontend component/command tests.
- Run format, clippy, Rust tests, typecheck, lint, Vitest, production build and
  a Vite smoke request.
- Commit and push only `phase11/implementation`; do not merge `main`.
