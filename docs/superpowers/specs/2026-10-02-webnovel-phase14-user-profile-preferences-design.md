# Phase 14: Local User Profile and Global Preferences

## Status

Approved design for the next implementation slice on `phase13/implementation`.

## Goal

Add a durable local user identity and structured global writing preferences so
the workspace has a stable owner-level settings boundary before later user
interface, project overrides or cloud features are introduced.

The local profile is an application-local identity. It is not an account,
authentication credential or sync identity.

## Scope

- one local user profile stored in SQLite;
- structured global writing preferences stored in SQLite;
- idempotent migration for the new records;
- typed Rust domain, repository, service and Tauri command boundaries;
- typed TypeScript command and data contracts;
- optimistic revision checks for preference updates;
- validation and safe error mapping;
- Rust and frontend command-contract tests.

## Non-goals

- cloud authentication, account creation or password management;
- multi-user access control or project membership;
- cloud sync, device identity or PostgreSQL;
- project-level preference overrides;
- automatic preference extraction from chats or manuscripts;
- AI provider-specific settings;
- a visual settings redesign or a new settings screen.

## Data model

SQLite migration `0008_create_user_profile_preferences.sql` creates two
singleton-scoped tables. The singleton key is the stable text value
`local_user`.

`user_profile` contains:

- `id TEXT PRIMARY KEY`;
- `display_name TEXT NOT NULL`;
- `preferred_language TEXT NOT NULL`;
- `created_at TEXT NOT NULL`;
- `updated_at TEXT NOT NULL`;
- `revision INTEGER NOT NULL CHECK (revision > 0)`.

`user_preferences` contains:

- `id TEXT PRIMARY KEY`;
- `preferred_narrator TEXT NOT NULL`;
- `preferred_pov TEXT NOT NULL`;
- `chapter_length INTEGER NOT NULL CHECK (chapter_length > 0)`;
- `scene_length INTEGER NOT NULL CHECK (scene_length > 0)`;
- `dialogue_density INTEGER NOT NULL CHECK (dialogue_density BETWEEN 0 AND 100)`;
- `prose_level TEXT NOT NULL`;
- `pacing TEXT NOT NULL`;
- `avoid_repetition INTEGER NOT NULL CHECK (avoid_repetition IN (0, 1))`;
- `created_at TEXT NOT NULL`;
- `updated_at TEXT NOT NULL`;
- `revision INTEGER NOT NULL CHECK (revision > 0)`.

The migration inserts one deterministic default row for each table. Existing
projects remain valid and require no ownership column in this phase. Project
preference overrides are intentionally deferred so this slice does not change
the existing project data contract.

## Domain and service boundary

`src-tauri/src/domain/user.rs` owns the serializable `UserProfile`,
`UserPreferences`, update inputs, default values and normalization rules.

`src-tauri/src/users/repository.rs` owns SQL reads and optimistic updates.
`src-tauri/src/users/service.rs` owns the singleton lookup, validation and
revision checks. The service exposes:

- `get_profile() -> AppResult<UserProfile>`;
- `update_profile(input: UpdateUserProfileInput, expected_revision: u64) -> AppResult<UserProfile>`;
- `get_preferences() -> AppResult<UserPreferences>`;
- `update_preferences(input: UpdateUserPreferencesInput, expected_revision: u64) -> AppResult<UserPreferences>`.

Each update is one SQLite transaction. A stale revision returns `conflict` and
does not write a partial row. Updates never call a provider or inspect project
content.

## Command contract

Rust exposes these typed commands through the existing command adapter:

- `user_profile_get`;
- `user_profile_update` with `{ input, expected_revision }`;
- `user_preferences_get`;
- `user_preferences_update` with `{ input, expected_revision }`.

The client adds matching functions in `apps/client/src/lib/commands.ts` and
serde-compatible types under `apps/client/src/types/user.ts`. Error handling
continues to use the existing safe command-error normalizer; raw SQL details,
stack traces and provider data are never returned.

## Defaults and validation

The default profile uses `Writer` as the display name and `en` as the preferred
language. The default preferences use `third_person`, `limited`, chapter length
`2000`, scene length `600`, dialogue density `40`, prose level `standard`,
`balanced` pacing and repetition prevention enabled.

Profile display names are trimmed and may not be blank after normalization.
Language, narrator, POV, prose level and pacing are trimmed non-empty values.
Lengths must be positive, and dialogue density must stay between 0 and 100.
Boolean values are represented as Rust/TypeScript booleans at the command
boundary and as SQLite integers behind the repository.

## Error behavior

- missing singleton rows are treated as `storage` errors after migration;
- invalid user input returns `validation`;
- stale updates return `conflict`;
- database failures remain mapped to the existing safe `storage` error.

No new authentication or provider error codes are introduced.

## Verification

Rust tests cover migration idempotence, deterministic defaults, normalization,
validation, profile update conflicts, preference update conflicts and boolean
round trips. Command tests assert snake_case payloads and safe error mapping.
Frontend tests cover command payloads and response typing without adding a
settings screen.

Required repository checks remain unchanged:

```text
npm run typecheck
npm run lint
npm test -- --run
npm run build
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Future compatibility

The local profile ID is deliberately opaque to the client so a later account
or sync phase can add a remote identity without changing project entities.
Global preferences are kept separate from projects so a later project override
can be layered by the Context Compiler rather than copied into canonical
project memory. No cloud or authentication behavior is implied by this phase.
