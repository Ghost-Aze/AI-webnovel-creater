# WEBNOVEL AI STUDIO Phase 1 Implementation Plan

**Goal:** Project Memory’nin ilk yapılandırılmış entity’lerini, `Character` ve `CharacterState`, provider bağımsız biçimde eklemek.

**Branch:** `phase1/implementation`

**Global boundary:** AI/provider, semantic retrieval, sync, relationship graph ve manuscript özellikleri bu planda yoktur.

## Task 1: Domain and migration

- [x] Add `migrations/0002_create_characters.sql`.
- [x] Add Rust domain types, typed inputs/errors and repository/service modules.
- [x] Extend migration runner with ordered migration registration.
- [x] Add unit and temporary SQLite integration tests for create/list/get/update/archive and state upsert.
- [x] Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings` and `cargo test`.

## Task 2: Tauri commands and frontend client

- [x] Add the seven character/state command adapters.
- [x] Add TypeScript types and typed client functions.
- [x] Extend safe error mapping for duplicate names and archived mutations.
- [x] Add command contract tests and run Rust/frontend checks.

## Task 3: Character workspace UI

- [x] Add a character list to the project workspace.
- [x] Add create/edit character dialog and state editor.
- [x] Handle loading, empty, archived and safe error states.
- [x] Add frontend tests for validation, rendering and state updates.

## Task 4: Documentation and verification

- [x] Document the Phase 1 schema and command boundary.
- [x] Run the complete frontend and Rust verification matrix.
- [x] Keep native Windows/Android checks separate from the headless data-layer result.

Phase 2 planning must wait for a separate user decision.
