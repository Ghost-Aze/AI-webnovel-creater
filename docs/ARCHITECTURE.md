# Phase 0 architecture

The repository is split into a browser-facing client and a provider-independent Rust core. The client is intentionally unaware of SQLite and of any AI provider.

```text
apps/client                 React + TypeScript + Vite
  features/projects         Project list, dialog and workspace routes
  lib/commands              typed Tauri invoke client and safe errors
src-tauri
  commands.rs               Tauri command adapters
  projects/service.rs       validation, normalization and policy
  projects/repository.rs    SQL and row mapping
  db.rs                     SQLite connection and migration runner
  domain/project.rs         provider-independent Project types
migrations/                 ordered SQL files
```

## Command boundary

The five commands are `project_create`, `project_list`, `project_get`, `project_update` and `project_archive`. They accept and return serde types matching the TypeScript definitions. Expected failures use serializable codes (`validation`, `not_found`, `archived_project`, `storage` and `internal`). The frontend maps those codes to user-facing messages and never displays SQL or Rust stack traces.

The Tauri feature is enabled only for the native shell. Headless tests compile the same domain, repository, service and command-helper code without loading a Linux webview. Native runs initialize one `ProjectService` in Tauri managed state.

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

`migrations/0001_create_projects.sql` creates the table and an index. `db::run_migrations` records the migration name in `_migrations` and skips it on later starts.

## UI shell

`AppShell` owns the left navigation, center route outlet, right context placeholders and bottom status bar. `/projects` handles list/create/filter states. `/projects/:projectId` handles project details, rename and archive. The CSS switches to a compact navigation row and hides the context panel on narrow screens.

## Phase boundary

The domain is deliberately limited to `Project`. Character state, narrative memory, provider adapters, orchestration, manuscript revisions, semantic search and sync need their own modules and migrations in later approved phases. They must not be added as JSON blobs or provider-specific fields to this schema.
