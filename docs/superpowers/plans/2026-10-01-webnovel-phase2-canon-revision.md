# Canon and Revision Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add provider-independent canonical revision history, proposal lifecycle, optimistic concurrency and history/diff/restore UI for `Character` and `CharacterState`.

**Architecture:** Keep the existing typed canonical tables as current snapshots. Record each accepted mutation in a generic immutable `memory_revisions` log, while unaccepted changes live in a separate `memory_proposals` table. Character and state services enforce revision checks and canon locks inside SQLite transactions; the React workspace consumes typed history and restore commands.

**Tech Stack:** Rust 2021, SQLite/rusqlite bundled migrations, serde/serde_json, Tauri 2 typed commands, React 19, TypeScript, Vitest and Testing Library.

**Spec:** `docs/superpowers/specs/2026-10-01-webnovel-phase2-canon-revision-design.md`

## Global Constraints

- Work only on `phase2/implementation`; do not merge into `main`.
- Preserve the approved snapshot + immutable log architecture; do not convert canonical storage into event sourcing or a project-wide JSON blob.
- Canonical snapshots contain only `canon` or `locked_canon`; `draft` values stay in `memory_proposals`.
- Every accepted mutation, restore and proposal promotion updates the snapshot and appends exactly one revision in the same SQLite transaction.
- Every update, restore and canon-status mutation checks `expected_revision`; stale writes return `conflict` without changing data.
- Provider integrations, AI tool calling, memory extraction, semantic search, graph data, sync, auth, PostgreSQL, Developer Chat and manuscript editing are out of scope.
- Keep SQL behind repository/service boundaries and expose only typed Tauri commands to the client.
- After each task run that task’s tests and commit the task before starting the next one.
- Finish with `npm run typecheck`, `npm run lint`, `npm test -- --run`, `npm run build`, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`.

## Review Focus

- A stale `expected_revision` must leave both the canonical snapshot and revision log unchanged. **Pinned by:** Task 2 integration test `stale_character_update_does_not_write_revision`.
- A `locked_canon` Character or CharacterState must reject update, restore and proposal promotion while preserving its history. **Pinned by:** Task 2 `locked_character_rejects_mutation` and Task 3 `locked_proposal_cannot_promote`.
- A missing CharacterState must report revision `0`, then create revision `1` on its first partial upsert without losing untouched fields. **Pinned by:** Task 2 `state_upsert_creates_first_revision_and_preserves_fields`.
- A proposal based on an old revision must be rejectable without side effects, while a valid promotion creates one canonical revision and marks the proposal accepted. **Pinned by:** Task 3 `proposal_promotion_checks_base_revision`.
- Restoring an older revision from the UI must create a new revision and keep unsaved form input when the server returns a conflict. **Pinned by:** Task 5 `restores_selected_revision` and `keeps_form_input_on_conflict`.

## File Map

The implementation follows the repository’s existing boundaries:

- `migrations/0003_create_memory_revisions.sql` owns revision/proposal tables and snapshot columns.
- `src-tauri/src/domain/revision.rs` owns typed canon, entity, actor, operation, revision and proposal values.
- `src-tauri/src/revisions/repository.rs` owns revision/proposal SQL and row mapping; `src-tauri/src/revisions/service.rs` owns generic history, restore, lock and proposal policy.
- `src-tauri/src/characters/repository.rs` and `src-tauri/src/characters/service.rs` keep Character/CharacterState mutations transactional and call the revision persistence helpers.
- `src-tauri/src/commands.rs` and `src-tauri/src/lib.rs` expose the typed command boundary.
- `apps/client/src/types/revision.ts` and `apps/client/src/lib/commands.ts` mirror the command contract.
- `apps/client/src/features/projects/RevisionHistoryPanel.tsx` and `RevisionDiff.tsx` own history selection, field diffs and restore actions; `CharacterPanel.tsx` only composes them.

### Task 1: Revision domain and migration

**Files:**

- Create: `migrations/0003_create_memory_revisions.sql`
- Create: `src-tauri/src/domain/revision.rs`
- Modify: `src-tauri/src/domain/mod.rs`
- Modify: `src-tauri/src/domain/character.rs`
- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/error.rs`
- Test: `src-tauri/src/domain/revision.rs` unit tests and `src-tauri/tests/project_flow.rs`

**Interfaces:**

- Produces `CanonStatus::{Canon, LockedCanon}`, `MemoryEntityType::{Character, CharacterState}`, `RevisionOperation::{Create, Update, Archive, Restore, CanonStatus, Promote}`, `ActorType::{User, Ai, System}`, `ProposalStatus::{Draft, Accepted, Rejected}` and serde-backed `MemoryRevision`/`MemoryProposal` values.
- `MemoryRevision` fields are `id`, `project_id`, `entity_type`, `entity_id`, `revision: u64`, `operation`, `actor_type`, `actor_id: Option<String>`, `base_revision: u64`, `previous_value: Option<serde_json::Value>`, `new_value: serde_json::Value`, `source_type: Option<String>`, `source_id: Option<String>`, and `created_at`.
- `Character` and `CharacterState` gain `revision: u64` and `canon_status: CanonStatus`; `CharacterState::empty` returns revision `0` and `Canon`.
- `AppError` gains safe `Conflict`, `LockedCanon` and `InvalidProposal` variants and matching lowercase codes.

- [x] **Step 1: Write the failing domain and migration tests**

  Add tests for lowercase serde round-trips, invalid enum values, `CharacterState::empty` defaults, migration registration count `3`, and the existence of `memory_revisions`/`memory_proposals` with `canon_status` defaults.

- [x] **Step 2: Run the focused tests to verify they fail**

  Run: `cargo test domain::revision::tests project_flow -- --nocapture`

  Expected: FAIL because the new typed values, migration and snapshot fields do not exist.

- [x] **Step 3: Implement the typed revision domain and `0003` migration**

  Define the enums and serde structs in `domain/revision.rs`; add the migration to the ordered `MIGRATIONS` list. Add `revision INTEGER NOT NULL DEFAULT 0` and `canon_status TEXT NOT NULL DEFAULT 'canon'` with checks to both canonical tables. Create indexed revision/proposal tables with foreign-key project ownership, entity/revision uniqueness and status/operation checks.

- [x] **Step 4: Run the focused tests to verify they pass**

  Run: `cargo test domain::revision::tests project_flow -- --nocapture`

  Expected: all selected unit and migration tests pass and existing project data remains readable.

- [x] **Step 5: Commit the domain and migration slice**

  ```bash
  git add migrations/0003_create_memory_revisions.sql src-tauri/src/domain src-tauri/src/db.rs src-tauri/src/error.rs src-tauri/tests/project_flow.rs
  git commit -m "feat: add canonical revision schema"
  ```

### Task 2: Transactional Character revisions and history service

**Files:**

- Create: `src-tauri/src/revisions/mod.rs`
- Create: `src-tauri/src/revisions/repository.rs`
- Create: `src-tauri/src/revisions/service.rs`
- Modify: `src-tauri/src/characters/repository.rs`
- Modify: `src-tauri/src/characters/service.rs`
- Modify: `src-tauri/src/domain/character.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/revision_flow.rs`

**Interfaces:**

- `RevisionRepository::list(entity_type: MemoryEntityType, entity_id: &str) -> AppResult<Vec<MemoryRevision>>` returns ascending revision order.
- `RevisionRepository::get(entity_type: MemoryEntityType, entity_id: &str, revision: u64) -> AppResult<MemoryRevision>` returns `NotFound` when absent.
- `RevisionService::history(entity_type, entity_id) -> AppResult<Vec<MemoryRevision>>` delegates typed history reads.
- `CharacterService::create`, `update`, `archive` and `update_state` append revisions with actor `User`, source `manual`, and the exact base revision used by the mutation.
- `RevisionService::restore(entity_type, entity_id, target_revision: u64, expected_revision: u64) -> AppResult<MemoryRevision>` applies the selected snapshot as a new revision.
- `RevisionService::set_canon_status(entity_type, entity_id, status: CanonStatus, expected_revision: u64) -> AppResult<MemoryRevision>` enforces explicit lock policy.

- [x] **Step 1: Write failing integration tests for revision sequencing and concurrency**

  Add `revision_flow.rs` tests named `character_create_and_update_append_ordered_revisions`, `stale_character_update_does_not_write_revision`, `state_upsert_creates_first_revision_and_preserves_fields`, `locked_character_rejects_mutation`, and `restore_creates_new_revision_without_mutating_history`. Assert revision numbers, before/after snapshots, unchanged row values after conflict, and safe error codes.

- [x] **Step 2: Run the focused integration tests to verify they fail**

  Run: `cargo test --test revision_flow -- --nocapture`

  Expected: FAIL because revision persistence, expected-revision arguments and service methods are not implemented.

- [x] **Step 3: Implement transactional revision persistence**

  Add row mapping and insert/list helpers in `revisions/repository.rs`. Update Character and CharacterState repository writes to acquire one SQLite transaction, validate the current revision/canon status, write the typed snapshot, insert exactly one `memory_revisions` row and commit atomically. Keep partial CharacterState updates as merges and serialize only revision snapshots, never as canonical storage.

- [x] **Step 4: Implement history, restore and canon-status policy**

  Add `RevisionService` dispatch for `Character` and `CharacterState`; reject stale bases with `Conflict`, reject writes to `LockedCanon` with `LockedCanon`, and make restore use the target revision’s `new_value` while recording `operation = restore` and `source_type = restore`.

- [x] **Step 5: Run the focused integration tests to verify they pass**

  Run: `cargo test --test revision_flow -- --nocapture`

  Expected: all revision sequencing, state upsert, lock, conflict and restore tests pass.

- [x] **Step 6: Commit the transactional revision slice**

  ```bash
  git add src-tauri/src/revisions src-tauri/src/characters src-tauri/src/domain/character.rs src-tauri/src/lib.rs src-tauri/tests/revision_flow.rs
  git commit -m "feat: add transactional character revisions"
  ```

### Task 3: Proposal lifecycle

**Files:**

- Modify: `src-tauri/src/revisions/repository.rs`
- Modify: `src-tauri/src/revisions/service.rs`
- Modify: `src-tauri/src/error.rs`
- Test: `src-tauri/tests/proposal_flow.rs`

**Interfaces:**

- `CreateProposalInput { project_id: String, entity_type: MemoryEntityType, entity_id: Option<String>, operation: RevisionOperation, payload: serde_json::Value, base_revision: u64, actor_type: ActorType, actor_id: Option<String> }`.
- `ProposalService::create(input) -> AppResult<MemoryProposal>` validates the project and payload shape without changing canonical tables.
- `ProposalService::list(project_id: &str, status: Option<ProposalStatus>) -> AppResult<Vec<MemoryProposal>>` returns newest proposals first.
- `ProposalService::promote(id: &str, expected_revision: u64) -> AppResult<MemoryRevision>` validates proposal status/base revision, applies the typed payload and records one `promote` revision atomically.
- `ProposalService::reject(id: &str) -> AppResult<MemoryProposal>` is idempotent for rejected proposals and rejects accepted proposals with `InvalidProposal`.

- [x] **Step 1: Write failing proposal lifecycle tests**

  Add tests named `proposal_create_is_non_canonical`, `proposal_promotion_checks_base_revision`, `proposal_reject_is_idempotent`, and `locked_proposal_cannot_promote`. Assert draft values do not appear in Character reads, stale promotion leaves both proposal and snapshot unchanged, and accepted/rejected transitions are enforced.

- [x] **Step 2: Run proposal tests to verify they fail**

  Run: `cargo test --test proposal_flow -- --nocapture`

  Expected: FAIL because proposal repository/service methods do not exist.

- [x] **Step 3: Implement proposal persistence and transitions**

  Add typed row mapping and service validation. Keep proposals separate from canonical queries; on promote, use the same transaction helper as Character revisions and set proposal status to `accepted` only after the canonical revision commits.

- [x] **Step 4: Run proposal tests to verify they pass**

  Run: `cargo test --test proposal_flow -- --nocapture`

  Expected: all proposal lifecycle and lock/concurrency tests pass.

- [x] **Step 5: Commit the proposal slice**

  ```bash
  git add src-tauri/src/revisions src-tauri/src/error.rs src-tauri/tests/proposal_flow.rs
  git commit -m "feat: add canonical memory proposals"
  ```

### Task 4: Tauri commands and typed frontend client

**Files:**

- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `apps/client/src/lib/command-error.ts`
- Modify: `apps/client/src/lib/commands.ts`
- Modify: `apps/client/src/types/character.ts`
- Create: `apps/client/src/types/revision.ts`
- Test: `src-tauri/src/commands.rs` and `apps/client/src/lib/commands.test.ts`

**Interfaces:**

- Rust wrappers expose `memory_history_list`, `memory_restore`, `memory_set_canon_status`, `memory_proposal_create`, `memory_proposal_list`, `memory_proposal_promote` and `memory_proposal_reject` with snake_case Tauri argument names.
- Frontend exports `listMemoryHistory(entityType, entityId)`, `restoreMemory(entityType, entityId, revision, expectedRevision)`, `setMemoryCanonStatus(entityType, entityId, status, expectedRevision)`, and typed proposal functions.
- `Character`, `CharacterState`, `Revision` and `Proposal` TypeScript types mirror Rust serde names, including `revision`, `canon_status`, `previous_value`, `new_value` and safe nullable fields.
- `normalizeCommandError` maps `conflict`, `locked_canon` and `invalid_proposal` to safe user-facing messages without raw SQL or serialized payloads.

- [x] **Step 1: Write failing command contract tests**

  Extend Rust command tests for history and restore helper delegation. Extend `commands.test.ts` to assert snake_case `entity_type`, `entity_id`, `expected_revision` and `character_id` arguments, plus safe conflict/locked error mapping.

- [x] **Step 2: Run command tests to verify they fail**

  Run: `cargo test commands::tests` and `npm test -- --run src/lib/commands.test.ts`

  Expected: FAIL because wrappers, client functions and error mappings are absent.

- [x] **Step 3: Implement Rust command adapters and registration**

  Add typed helper functions and `#[tauri::command]` wrappers in `commands.rs`; register every wrapper in `tauri::generate_handler![]`. Preserve the existing `report` safe logging pattern.

- [x] **Step 4: Implement TypeScript types, client functions and safe errors**

  Add `revision.ts`, extend Character/State types and call `invoke` with exact Rust snake_case argument names. Use `normalizeCommandError` for all new command failures.

- [x] **Step 5: Run command tests to verify they pass**

  Run: `cargo test commands::tests` and `npm test -- --run src/lib/commands.test.ts`

  Expected: all command contract tests pass and no backend details appear in error messages.

- [x] **Step 6: Commit the command boundary slice**

  ```bash
  git add src-tauri/src/commands.rs src-tauri/src/lib.rs apps/client/src/lib apps/client/src/types
  git commit -m "feat: expose canonical revision commands"
  ```

### Task 5: History, diff and restore UI

**Files:**

- Create: `apps/client/src/features/projects/RevisionHistoryPanel.tsx`
- Create: `apps/client/src/features/projects/RevisionDiff.tsx`
- Modify: `apps/client/src/features/projects/CharacterPanel.tsx`
- Modify: `apps/client/src/styles.css`
- Test: `apps/client/src/test/RevisionHistoryPanel.test.tsx` and `apps/client/src/test/CharacterPanel.test.tsx`

**Interfaces:**

- `RevisionHistoryPanelProps` accepts `entityType`, `entityId`, `currentRevision`, `canonStatus`, `onRestored` and `disabled`.
- `RevisionDiff` accepts one `Revision` and renders only fields whose before/after values differ; null before values render as an empty initial state.
- The history panel calls `listMemoryHistory`, `restoreMemory` and `setMemoryCanonStatus`; it never mutates Character state locally without a successful command result.

- [x] **Step 1: Write failing component tests**

  Add `RevisionHistoryPanel.test.tsx` tests named `renders_revision_metadata_and_field_diff`, `restores_selected_revision`, `disables_mutations_for_locked_canon`, and `keeps_form_input_on_conflict`. Update CharacterPanel command mocks and assert the selected Character and CharacterState panels pass current revision/status values.

- [x] **Step 2: Run frontend tests to verify they fail**

  Run: `npm test -- --run src/test/RevisionHistoryPanel.test.tsx src/test/CharacterPanel.test.tsx`

  Expected: FAIL because the history components and command calls do not exist.

- [x] **Step 3: Implement `RevisionDiff` and `RevisionHistoryPanel`**

  Render loading/empty/error states, revision metadata, field-level diffs, restore confirmation and locked controls. On `conflict`, retain the parent form’s unsaved values and expose a reload/retry path without displaying serialized error details.

- [x] **Step 4: Integrate history into CharacterPanel**

  Add history controls for Character profile and CharacterState, refresh the selected snapshot after a successful restore/status update, and disable edits/restores when `canon_status === 'locked_canon'`.

- [x] **Step 5: Add focused CSS and run frontend tests**

  Add styles for the history list, diff rows, status badges and conflict state without changing the existing project workspace layout. Run: `npm test -- --run src/test/RevisionHistoryPanel.test.tsx src/test/CharacterPanel.test.tsx`

  Expected: all new and existing focused component tests pass.

- [x] **Step 6: Commit the UI slice**

  ```bash
  git add apps/client/src/features/projects apps/client/src/styles.css apps/client/src/types
  git commit -m "feat: add character revision history UI"
  ```

### Task 6: Documentation and complete verification

**Files:**

- Modify: `docs/ARCHITECTURE.md`
- Modify: `docs/ARCHITECTURE_DECISIONS.md`
- Modify: `docs/superpowers/plans/2026-10-01-webnovel-phase2-canon-revision.md`
- Test/verification: repository-wide frontend and Rust commands

**Interfaces:**

- Documentation names migration `0003`, revision/proposal ownership, command boundary, lock/conflict semantics and the Phase 2 boundary.
- The final plan records every completed checkbox, commit and verification command.

- [x] **Step 1: Update architecture documentation**

  Document the generic revision/proposal tables, Character/CharacterState snapshot relationship and safe conflict/lock behavior. Keep AI provider, sync and manuscript work explicitly outside this phase.

- [x] **Step 2: Run the complete verification matrix**

  Run:

  ```bash
  npm run format:check
  npm run typecheck
  npm run lint
  npm test -- --run
  npm run build
  cargo fmt --all -- --check
  cargo clippy --all-targets -- -D warnings
  cargo test
  ```

  Expected: all existing and new frontend tests pass; all Rust unit/integration tests pass; formatting, lint, clippy and production build pass. Native Tauri/Android checks remain environment-limited as documented.

- [x] **Step 3: Self-review the full diff and branch state**

  Verify `git diff --check`, no generated artifacts or secrets, a clean `phase2/implementation` worktree, and that `main` has not moved or been merged.

- [x] **Step 4: Commit documentation and final verification record**

  ```bash
  git add docs/ARCHITECTURE.md docs/ARCHITECTURE_DECISIONS.md docs/superpowers/plans/2026-10-01-webnovel-phase2-canon-revision.md
  git commit -m "docs: finalize canon revision phase"
  git push -u origin phase2/implementation
  ```

## Completion record

- `066eab4` — canonical revision domain and migration.
- `c604bd2` — transactional Character and CharacterState revision history.
- `9a215e4` — proposal persistence and lifecycle.
- `11589d7` — Tauri command and typed client boundary.
- `0bdbc05` — revision history, diff and restore UI.
- Final verification: `npm run format:check`, `npm run typecheck`, `npm run lint`, `npm test -- --run` (18 tests), `npm run build`, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` (11 unit tests, 17 integration/doc tests) passed.
- Native `cargo check --features tauri-app` remains environment-limited because the cloud image lacks `glib-2.0 >= 2.70`; this is documented in `docs/CLOUD_DEVELOPMENT.md`.
