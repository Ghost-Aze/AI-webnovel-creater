# Development chat context

This file preserves the implementation context from the Codex session that
produced the current working tree. It is a handoff record, not canonical
product data or a transcript of private conversation.

## User priorities and boundaries

- Work remains on `phase13/implementation`; no implementation work is merged
  into `main`.
- Backend and core behavior are completed before a later visual redesign.
- Chapter Chat, Developer Chat and Manuscripts must use the provider-independent
  Rust boundaries and explicit canonical writes.
- AI output cannot silently mutate canonical memory or manuscript content.
- Authentication, cloud sync, collaboration, semantic retrieval, automatic
  memory extraction, project preference overrides, export and a visual
  settings redesign remain deferred until separately approved.

## Implemented during this session

- Added functional Developer Chat and Manuscripts routes and panels, typed
  command wrappers, responsive shell updates and regression tests.
- Added explicit project edit/open/delete actions with a confirmation dialog,
  a cascading `project_delete` command, and a project-scoped second sidebar
  navigation for Overview, Developer Chat and Manuscripts.
- Added provider settings lifecycle controls: edit metadata without re-entering
  an existing key, test a connection through the selected model, remove a
  provider, and persist non-secret settings in SQLite while keeping API keys
  in the platform secure store.
- Hardened Chapter Chat and manuscript proposal behavior for archived chapters
  and archived projects.
- Hardened character revision, restore, canon-status and proposal promotion
  transactions with project/character status checks and conflict guards.
- Added Phase 14 local user profile and global writing-preferences foundation:
  - migration `0008_create_user_profile_preferences.sql`;
  - singleton `local_user` profile and preferences rows with deterministic
    defaults;
  - typed Rust `UserProfile` and `UserPreferences` domain models;
  - repository/service validation, SQLite boolean mapping and optimistic
    revision updates;
  - Tauri commands `user_profile_get`, `user_profile_update`,
    `user_preferences_get` and `user_preferences_update`;
  - TypeScript contracts and command wrappers;
  - migration, normalization, conflict, storage and command-boundary tests.
- Added the approved Phase 14 design and implementation plan under
  `docs/superpowers/`.
- Updated the README and architecture/development context documents with the
  local-only Phase 14 boundary.
- Added Jan-inspired client surfaces for the provider workspace and Developer /
  Chapter Chat. Provider settings now use data-driven provider descriptors,
  secure API-key-only editing, connection testing, model profiles and chat
  runtime controls for assistant, provider, model and quality selection.
- Added typed provider failure classification and retry-safe chat state. Chat
  errors preserve the draft, distinguish credential/endpoint failures from
  retryable provider failures and keep retry attempts from appending duplicate
  user turns.
- Inspected the Jan open-source reference implementation. The relevant design
  patterns are `ChatInput` plus `DropdownModelProvider`, a secondary
  `SettingsMenu` provider rail, provider detail/model discovery through
  `/models`, and a separate scrollable conversation region with a bottom-pinned
  composer. These are reference patterns only; Jan source is not copied into
  this repository.

## Approved design decisions

- The local profile uses the stable opaque key `local_user`.
- Default profile: display name `Writer`, language `en`.
- Default writing preferences: `third_person`, `limited`, chapter length
  `2000`, scene length `600`, dialogue density `40`, prose level `standard`,
  pacing `balanced`, repetition prevention enabled.
- Profile and preference writes require `expected_revision`; stale writes
  return `conflict` and do not partially change a row.
- Missing singleton rows map to the safe `storage` error.

## Verification evidence

- Frontend: typecheck, ESLint, production build and 92 Vitest tests pass.
- Rust: `cargo fmt --all -- --check`, clippy, 113 unit tests plus the project
  integration suites pass.
- Windows native smoke test starts `webnovel-ai-studio`; Vite at
  `http://localhost:1420/` returns HTTP 200.
- Repository-wide `npm run format:check` still reports pre-existing
  formatting drift in 34 client files; those unrelated changes were not
  mass-formatted.

## Current handoff

The working tree contains the Phase 13 functional work and the previously
recorded local Phase 14/15 foundations. No further phase expansion is approved
in this handoff. The Jan inspection is a reference for the next explicitly
approved UI iteration; it does not authorize new scope. Before pushing,
generated `src-tauri/gen/` and `src-tauri/icons/` output must remain untracked,
and API keys must stay outside Git.
