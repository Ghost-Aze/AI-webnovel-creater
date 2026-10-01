# Phase 0–1 architecture

The repository is split into a browser-facing client and a provider-independent Rust core. The client is intentionally unaware of SQLite and of any AI provider.

Approved cross-phase defaults are recorded in [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md).

```text
apps/client                 React + TypeScript + Vite
  features/projects         Project and character workspace routes
  lib/commands              typed Tauri invoke client and safe errors
src-tauri
  commands.rs               Tauri command adapters
  projects/service.rs       validation, normalization and policy
  projects/repository.rs    SQL and row mapping
  characters/service.rs     character and state policy
  characters/repository.rs  character and state SQL and row mapping
  db.rs                     SQLite connection and migration runner
  domain/project.rs         provider-independent Project types
  domain/character.rs       provider-independent Character types
migrations/                 ordered SQL files
```

## Command boundary

The project commands are `project_create`, `project_list`, `project_get`, `project_update` and `project_archive`. Phase 1 adds `character_create`, `character_list`, `character_get`, `character_update`, `character_archive`, `character_state_get` and `character_state_update`. Phase 2 requires `expected_revision` on Character and CharacterState mutations so stale device state cannot overwrite a newer snapshot. They accept and return serde types matching the TypeScript definitions. Expected failures use serializable codes (`validation`, `not_found`, `archived_project`, `archived_character`, `duplicate_name`, `conflict`, `locked_canon`, `invalid_proposal`, `storage` and `internal`). The frontend maps those codes to user-facing messages and never displays SQL or Rust stack traces.

The Tauri feature is enabled only for the native shell. Headless tests compile the same domain, repository, service and command-helper code without loading a Linux webview. Native runs initialize one `ProjectService` and one `CharacterService` in Tauri managed state over the same SQLite connection.

## Project data model

`projects` is the only canonical Phase 0 entity:

| Column | Type and policy |
| --- | --- |
| `id` | UUID text primary key |
| `name` | required, trimmed text |
| `description` | required text, default empty |
| `status` | `active` or `archived` |
| `created_at` | UTC ISO-8601 text |
| `updated_at` | UTC ISO-8601 text |

Active projects are listed by most recent `updated_at`. Archived projects are omitted by default and may be requested explicitly. Updates to archived projects are rejected. Archive is idempotent so a repeated action does not corrupt state.

`migrations/0001_create_projects.sql` creates the project table and an index. `migrations/0002_create_characters.sql` adds the `characters` table and the one-to-one `character_states` table. Character state is stored as named columns so each field remains queryable and independently updateable; it is not a JSON blob. `db::run_migrations` records each migration name in `_migrations` and skips it on later starts.

## Phase 1 character memory

`characters` belongs to a project and has a normalized name, summary, role and active/archived status. Names are unique within a project case-insensitively. Character writes require an active project, while reads can still list records for an archived project. Archiving is idempotent and archived characters cannot be edited.

`character_states` has one row per character. The service creates the row on the first state update and merges partial updates into the existing row. The current UI edits location, emotional state, goals and arc role while preserving the remaining structured fields for later slices.

## Phase 2 canon and revision memory

Migration `0003_create_memory_revisions.sql` adds `revision` and `canon_status` to the typed `characters` and `character_states` snapshots. Canonical values remain queryable columns; `canon` and `locked_canon` are the only stored snapshot statuses.

The generic `memory_revisions` table is an append-only log owned by `src-tauri/src/revisions/repository.rs`. Each accepted Character or CharacterState mutation records its actor, base revision, operation, source and structured before/after snapshots in the same SQLite transaction as the current-row update. `RevisionService` provides typed history, restore and canon-status operations. A stale base returns `conflict`; writes to a locked snapshot return `locked_canon` without changing the row or log.

Unaccepted changes live in `memory_proposals`, not in canonical tables. `ProposalService` validates typed Character update proposals, promotes a matching draft through the same transactional revision path, and supports idempotent rejection. Proposal payloads never become visible through ordinary Character or CharacterState reads until promotion succeeds.

The command boundary exposes `memory_history_list`, `memory_restore`, `memory_set_canon_status` and the proposal lifecycle commands. React consumes these through typed client functions. `RevisionHistoryPanel` renders metadata and field-level diffs, preserves parent form input when a restore conflicts, and disables edits/restores for locked canon.

## UI shell

`AppShell` owns the left navigation, center route outlet, right context placeholders and bottom status bar. `/projects` handles list/create/filter states. `/projects/:projectId` handles project details, rename and archive. The CSS switches to a compact navigation row and hides the context panel on narrow screens.

## Phase boundary

The domain currently covers `Project`, `Character` and `CharacterState`, with generic revision/proposal infrastructure for those entities. Provider adapters, orchestration, manuscript revisions, semantic search, relationships and sync remain outside this slice and require separate approved phases. New memory entities must remain structured and provider-independent.
