# WEBNOVEL AI STUDIO Phase 1 Implementation Plan

**Goal:** Project Memory’nin ilk yapılandırılmış entity’lerini, `Character` ve `CharacterState`, provider bağımsız biçimde eklemek.

**Branch:** `phase1/implementation`

**Global boundary:** AI/provider, semantic retrieval, sync, relationship graph ve manuscript özellikleri bu planda yoktur.

## Task 1: Domain and migration

- Add `migrations/0002_create_characters.sql`.
- Add Rust domain types, typed inputs/errors and repository/service modules.
- Extend migration runner with ordered migration registration.
- Add unit and temporary SQLite integration tests for create/list/get/update/archive and state upsert.
- Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings` and `cargo test`.

## Task 2: Tauri commands and frontend client

- Add the seven character/state command adapters.
- Add TypeScript types and typed client functions.
- Extend safe error mapping for duplicate names and archived mutations.
- Add command contract tests and run Rust/frontend checks.

## Task 3: Character workspace UI

- Add a character list to the project workspace.
- Add create/edit character dialog and state editor.
- Handle loading, empty, archived and safe error states.
- Add frontend tests for validation, rendering and state updates.

## Task 4: Documentation and verification

- Document the Phase 1 schema and command boundary.
- Run the complete frontend and Rust verification matrix.
- Keep native Windows/Android checks separate from the headless data-layer result.

Phase 2 planning must wait for a separate user decision.
