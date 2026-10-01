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
  provider/                 capability-aware provider contracts and registry
  context/                  budgeted structured-memory compiler and source
  db.rs                     SQLite connection and migration runner
  domain/project.rs         provider-independent Project types
  domain/character.rs       provider-independent Character types
migrations/                 ordered SQL files
```

## Command boundary

The project commands are `project_create`, `project_list`, `project_get`, `project_update` and `project_archive`. Phase 1 adds `character_create`, `character_list`, `character_get`, `character_update`, `character_archive`, `character_state_get` and `character_state_update`. Phase 2 requires `expected_revision` on Character and CharacterState mutations so stale device state cannot overwrite a newer snapshot. Phase 3 adds `provider_list`, `model_list` and `context_compile`; provider/model discovery and context preview remain typed command operations rather than provider-specific UI logic. They accept and return serde types matching the TypeScript definitions. Expected failures use serializable codes (`validation`, `not_found`, `archived_project`, `archived_character`, `duplicate_name`, `conflict`, `locked_canon`, `invalid_proposal`, `provider_not_found`, `model_not_found`, `no_suitable_model`, `unsupported_capability`, `invalid_provider_request`, `provider_failure`, `storage` and `internal`). The frontend maps those codes to user-facing messages and never displays SQL, prompts or Rust stack traces.

The Tauri feature is enabled only for the native shell. Headless tests compile the same domain, repository, service and command-helper code without loading a Linux webview. Native runs initialize one `ProjectService` and one `CharacterService` in Tauri managed state over the same SQLite connection.

## Project data model

`projects` is the only canonical Phase 0 entity:

| Column        | Type and policy              |
| ------------- | ---------------------------- |
| `id`          | UUID text primary key        |
| `name`        | required, trimmed text       |
| `description` | required text, default empty |
| `status`      | `active` or `archived`       |
| `created_at`  | UTC ISO-8601 text            |
| `updated_at`  | UTC ISO-8601 text            |

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

## Phase 3 provider and context foundation

`src-tauri/src/provider` defines the provider-independent async `AIProvider`
contract, capability metadata, runtime `ModelProfile` values and an in-memory
`ProviderRegistry`. Profiles and credentials are not canonical SQLite data. The
registry rejects duplicate provider/model identities and resolves a model before
any future provider call. Streaming and embedding operations have typed
unsupported-capability results; Phase 3 does not register a real network or
local-model adapter.

`src-tauri/src/context` defines `ContextSource`, `ContextCompileRequest` and a
deterministic `ContextCompiler`. The service-backed source reads only typed
Project, explicitly selected Character records and optional CharacterState
records. Caller-supplied working blocks are transient input. The compiler uses
the selected model's context window, output reserve and safety margin to produce
ordered blocks, truncation metadata and omissions. It never loads all project
memory or chat history and never mutates canonical memory.

The React client mirrors provider and context serde types in `types/provider.ts`
and `types/context.ts`; no provider SDK or SQL access enters the client. Future
orchestration, semantic retrieval, credentials and provider adapters must use
these boundaries and remain separate phases.

## Phase 4 provider execution

`src-tauri/src/provider/http.rs` implements the first production adapter:
`OpenAiCompatibleProvider` maps the provider-independent generate contract to
an OpenAI-compatible `/chat/completions` endpoint. `ReqwestTransport` owns
network I/O, while an injected `HttpTransport` keeps request/response and SSE
tests deterministic. Request validation happens before transport execution;
non-success HTTP responses, malformed JSON and malformed SSE map to the safe
`provider_failure` result without exposing response bodies or prompts.

Streaming responses are parsed as chunk-safe `data:` events. Content deltas
become typed `StreamEvent` values and `data: [DONE]` becomes the single typed
completion event. The bearer credential is an in-memory adapter input and is
not serializable, logged, stored in SQLite or included in sync.

The typed `provider_generate` command resolves the requested model through the
runtime registry and awaits `AIProvider::generate`. The application registry
is intentionally empty at startup until the later secure credential and
provider setup phase registers a configured adapter. React exposes the same
boundary through `generateProvider`; it does not know the HTTP wire format.

## Phase 5 provider runtime

`ProviderRuntime` owns the registry used by list/model/generate commands and a
`CredentialStore` abstraction. The phase's `EphemeralCredentialStore` keeps
`SecretValue` instances in process memory only; values are neither serializable
nor printable through `Debug`. Configure writes the credential, builds an
OpenAI-compatible adapter and registers it as one lifecycle. Any validation,
adapter or duplicate failure removes the temporary credential before returning.

`provider_configure` and `provider_remove` expose this lifecycle through typed
snake_case commands. Configure responses contain descriptors and model
profiles only. Startup creates an empty runtime, and remove deletes both the
provider registry entry and its credential reference. Windows Credential
Manager, Android Keystore, provider settings UI and model routing are deferred
until a platform secure-storage phase.

## Phase 6 provider setup and credential boundary

`CredentialStoreKind` makes the credential persistence choice explicit. The
cloud and headless test runtime select `Ephemeral`; its secrets live only in
the process and are cleared on restart. `PlatformSecure` is a typed native
integration seam for Windows Credential Manager and Android Keystore. Until a
target adapter is supplied, it returns `secure_store_unavailable` and never
falls back to SQLite or a plaintext file.

`provider_credential_status` exposes only the selected store kind, persistence
flag and availability; it never returns a credential or credential ID. The
React `/settings/providers` route uses the typed `provider_list`, `model_list`,
`provider_configure` and `provider_remove` commands. The setup form validates
required fields, submits one typed model profile, clears the API-key field
after success and renders provider descriptors without secrets. Model routing
and orchestration remain later phases.

## Phase 7 native secure credentials

`PlatformSecureCredentialStore` now delegates to target-specific adapters. On
Windows it initializes Windows Credential Manager; on Android it initializes
the Android-native keyring store, which encrypts SharedPreferences data with
the Android Keystore. The adapter uses the opaque credential ID as the store
username and keeps the application service namespace internal to Rust. Native
initialization is lazy and the status command reports only typed availability.

Unsupported and headless targets keep the unavailable platform implementation
or select `Ephemeral` through the application-store factory. No target falls
back to SQLite, a plaintext file, or an environment variable. Platform errors
are normalized to `secure_store_unavailable`, while a missing entry maps to
`credential_not_found` without exposing keyring error text.

The Tauri startup path selects native secure storage on Windows and Android and
the deterministic process-only store elsewhere. Provider descriptors and model
profiles are still process-local; persisting and restoring that metadata is a
later phase, independent of the native secret storage boundary.

## Phase 8 model routing

`ModelProfile` carries a typed `ModelTier` (`local`, `small`, `medium` or
`large`). `ModelRouter` consumes the in-memory registry profiles and a typed
`RoutingRequest` containing a logical `ModelTask`, `QualityMode`, optional
preferred model, required capabilities and an optional minimum context window.
It filters candidates before scoring task-affinity tags and quality/tier
policy, then uses provider/model IDs as a deterministic tie-break. Explicit
user preferences win when they satisfy the constraints.

The `model_route` command returns only a `RouteDecision` with the selected
model/profile and a typed reason. It never calls a provider, loads credentials
or returns prompt/response data. `FAST` favors local/small helper models,
`BALANCED` favors medium/large models with fallbacks, and `DEEP` favors
large/medium models. No candidate returns `no_suitable_model`. Narrative
orchestration and multi-call execution remain a later phase.

## UI shell

`AppShell` owns the left navigation, center route outlet, right context placeholders and bottom status bar. `/projects` handles list/create/filter states. `/projects/:projectId` handles project details, rename and archive. The CSS switches to a compact navigation row and hides the context panel on narrow screens.

## Phase boundary

The domain currently covers `Project`, `Character` and `CharacterState`, with generic revision/proposal infrastructure for those entities. Phase 3 adds provider contracts, runtime profiles and read-only context compilation. Phase 4 adds one provider adapter and the typed generate boundary. Phase 5 adds an ephemeral runtime configure/remove boundary. Phase 6 adds the provider setup route and explicit secure-store boundary. Phase 7 supplies Windows/Android native secure adapters. Phase 8 supplies deterministic model routing; orchestration, manuscript revisions, semantic search, relationships and sync remain later work. New memory entities must remain structured and provider-independent.
