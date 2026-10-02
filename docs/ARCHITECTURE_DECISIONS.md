# Architecture decisions

**Status:** Approved baseline

**Date:** 2026-10-01

This document records the product and architecture defaults approved after the
initial product prompt review. It guides later phases without expanding the
current Phase 1 implementation.

## Product boundary

- Phase 0 establishes the Tauri, React, Rust, SQLite and project workspace foundation.
- Phase 1 establishes structured `Character` and `CharacterState` memory.
- Phase 3 establishes provider-independent capability contracts, runtime model profiles, an in-memory registry and a deterministic structured-memory Context Compiler. It does not connect a provider.
- Phase 4 establishes one OpenAI-compatible HTTP adapter, an injected transport boundary, SSE streaming parsing and a typed provider-generate command. It does not persist credentials or add provider settings UI.
- Phase 5 establishes a process-memory provider runtime and credential-store abstraction. It supports configure/remove lifecycle commands without persisting secrets.
- Phase 6 establishes the typed provider setup route and explicit credential-store status boundary. The cloud/headless runtime uses session-only ephemeral storage.
- Phase 7 supplies target-specific Windows Credential Manager and Android Keystore-backed adapters. Native targets select those adapters at startup; unsupported and headless targets remain explicit and never fall back to plaintext storage. Provider descriptors and model profiles are still process-local until a later persistence phase.
- Phase 8 supplies a pure `ModelRouter` and `model_route` command. Quality modes and logical task tags select among registered profiles without making provider calls; orchestration remains a separate phase.
- Phase 9 supplies a sequential Narrative Orchestrator. It creates deterministic quality-mode plans, routes each step, compiles scoped context and executes through the provider contract without writing canonical memory or adding chat UI.
- Phase 10 supplies durable project-scoped Developer Chat conversations and a proposal-backed memory-tool boundary. Chat history is working context, not canonical memory; AI character changes remain draft proposals until explicit promotion.
- Phase 11 supplies structured chapters, one-to-one manuscript documents, optimistic-concurrency saves, immutable manuscript snapshots and a focused plain-text editor route. Rich text, autosave, collaboration, sync and export remain separate phases.
- Provider integrations, orchestration, semantic retrieval, manuscript tooling and sync remain separate phases and require their own implementation approval.

## Local-first and sync

- Local project work is available without an account or an internet connection.
- SQLite is the local source of truth. The client never connects directly to PostgreSQL.
- Future sync uses an authenticated backend API and an operation/revision history rather than blind last-write-wins for canonical data.
- Metadata may use field-level merges. Manuscript conflicts always remain visible to the user and support keeping both versions.
- Cloud sync is opt-in; local-only projects must remain fully usable.

## Canonical memory and revisions

- Project memory is stored as structured, queryable entities. A single project-wide JSON blob is not canonical storage.
- AI changes enter as `DRAFT` proposals by default. A user action promotes a proposal to `CANON`.
- `LOCKED_CANON` can be changed only through an explicit user-authorized flow. AI tools may propose a change and explain its impact, but cannot silently apply it.
- Every canonical mutation is transactional and creates an append-only revision record with actor, timestamp, entity identity and before/after structured values.
- Undo, restore and merge create new revisions; they do not rewrite history.

### Phase 2 implementation decisions

- `migrations/0003_create_memory_revisions.sql` keeps typed Character and CharacterState snapshots as the current canonical state and adds the generic `memory_revisions` and `memory_proposals` tables.
- Revision numbers are checked optimistically inside SQLite transactions. A stale base returns `conflict`; a `locked_canon` snapshot rejects update, archive, restore and proposal promotion. An explicit canon-status command is the user-controlled lock/unlock path.
- Proposal drafts are separate from canonical snapshots. Proposal `entity_id` remains nullable for future create operations; Phase 2 promotes only typed Character update proposals, and promotion changes the snapshot, marks the proposal accepted and appends one `promote` revision atomically.
- The Tauri command boundary owns history, restore, canon status and proposal lifecycle calls. TypeScript maps `conflict`, `locked_canon` and `invalid_proposal` to safe messages and never exposes SQL or serialized payloads.
- Character history UI uses immutable revision snapshots for field-level diffs. A failed restore leaves the parent form state untouched so the user can reload or retry deliberately.

## Character state

- `CharacterState` keeps the current structured snapshot for fast reads.
- State changes are also revisioned. When a chapter or scene is available, the change records may reference it as their source.
- Partial updates must preserve fields that were not part of the update. Locked canon and active-project policies are enforced in the Rust service layer.

## AI and providers

- The UI communicates with the Rust core through typed commands. Provider-specific code does not enter the domain or React components.
- Provider calls are routed through a common capability-aware abstraction. Unsupported features such as tools, vision, embeddings or structured output are reported explicitly.
- `ProviderRegistry` stores `Arc<dyn AIProvider>` adapters in memory and resolves models by provider id plus model id. `ModelProfile` is runtime metadata with context window, output reserve, capability flags and model strategy hints; it is not canonical SQLite state.
- Phase 3 includes only the async contract, safe provider error mapping and a deterministic mock used by tests. No production provider, HTTP SDK, API key or local-model runtime is registered.
- Phase 4's `OpenAiCompatibleProvider` translates the existing generate/stream contracts to `/chat/completions` and keeps the bearer token in the adapter instance only. `ReqwestTransport` is production transport; tests inject a fake transport without network access. Non-2xx, malformed JSON and malformed SSE never expose response bodies through `ProviderError` or logs.
- API keys are stored in the platform secure credential store and are excluded from SQLite backups and cloud sync by default.
- AI memory writes use the same transaction and revision path as user writes. A failed tool sequence cannot leave a half-applied canonical mutation.

## Orchestration and context

- Logical agents are roles in an execution plan, not a promise of one API call per role.
- FAST, BALANCED and DEEP are policy presets. The orchestrator may combine compatible roles into one model call and may route different steps to different model profiles.
- The Context Compiler selects structured memory first, then optional semantic results. It applies a per-task token budget and never sends the entire project memory or chat history automatically.
- Phase 3's compiler reads Project and explicitly selected Character/CharacterState records through `ContextSource`, accepts transient working blocks, uses model profile budget metadata and returns omission/truncation details without writing memory.
- A missing provider capability or retrieval service results in a visible degraded path, not an implicit change to canonical state.
- `provider_generate` resolves a model through `ProviderRegistry` and calls the async provider contract. The registry remains empty at startup until a later secure-credential setup registers an adapter.
- `ProviderRuntime` owns the shared registry and a `CredentialStore`. Phase 5's `EphemeralCredentialStore` is intentionally cleared on restart and is only a bridge to platform backends. Configure failures roll back the credential and registry mapping; remove deletes both runtime entries.
- Phase 6 exposes `CredentialStoreKind` and `provider_credential_status` without returning secrets. The provider settings form uses typed configure/remove commands, validates model budget metadata locally, and clears the API-key input after a successful submission.
- Phase 7's `PlatformSecureCredentialStore` delegates to Windows Credential Manager or the Android-native Keystore-backed store behind a target-specific Rust boundary. Initialization and platform failures are normalized to typed availability/errors; missing entries map to `credential_not_found`, and no raw keyring detail or plaintext fallback is exposed.
- Phase 8 adds `ModelTier`, `ModelTask`, `QualityMode` and deterministic capability/context-aware routing. Explicit user model choices remain authoritative when valid, and no suitable candidate returns a safe typed error rather than silently selecting an incompatible model.
- Phase 9 adds typed logical-agent execution steps and temporary working-memory handoff. The orchestrator owns sequencing and provider invocation, while canonical memory writes remain outside the execution boundary.
- Phase 10 adds ordered `Conversation`/`ConversationMessage` persistence and the `developer_chat_send`/`memory_tool_propose` command boundaries. User turns survive provider failures, while assistant turns are appended only after a successful run.
- Phase 11 adds `Chapter`, `Manuscript` and `ManuscriptRevision` persistence. A manuscript save requires an expected revision and appends a snapshot; restore creates a new revision rather than rewriting history. This makes local editor updates safe without coupling prose to chat or provider execution.

## Manuscript and chat

- Conversations and manuscript documents are separate entities.
- A chapter is a structured manuscript document with revisions; chat messages are working context and are not the canonical chapter body.
- Targeted edits produce a patch or a new revision. Concurrent manuscript edits create a user-visible conflict instead of silently overwriting content.

## Security and data lifecycle

- Archive is the default user-facing removal operation. Permanent deletion is a separate, explicit data-retention operation with revision and sync implications.
- Diagnostic logs must omit provider secrets, manuscript contents and raw database errors unless the user explicitly enables a secure diagnostic export.
- Backup and restore must preserve migration history and revision history before sync is enabled.

## Deferred choices

These defaults do not block the current foundation but must be specified before the relevant phase starts:

- authentication provider and hosted PostgreSQL deployment,
- minimum Windows and Android versions,
- first-class UI languages and generated-language defaults,
- exact provider capability matrix and model-profile schema,
- semantic index technology and retention policy,
- manuscript document format and editor package,
- backup encryption and account recovery policy.
