# Jan-style chat runtime and settings workspace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Jan-inspired application shell, settings workspace and shared typed chat runtime for Developer Chat and Chapter Chat.

**Architecture:** Keep SQLite, provider credentials, routing, context compilation and canonical write policy behind Rust services and repositories. Add a conversation-scoped runtime settings store and a shared `ChatRuntimeService`; expose typed load, update and send commands that feed a reducer-driven React workspace.

**Tech Stack:** Rust, SQLite, Tauri 2, serde, rusqlite, React, TypeScript, React Router, Vite, Vitest and CSS.

**Spec:** `docs/superpowers/specs/2026-10-03-jan-style-chat-runtime-design.md`

## Global Constraints

- Jan is a behavioral and interaction reference only; do not copy Jan source code, its local inference engine, Hub, MCP ecosystem or agent platform.
- Preserve the project's local-first, provider-independent domain boundaries.
- Keep SQL in Rust repositories and policy in Rust services; React components consume typed commands and do not access SQLite or provider SDKs.
- Keep API keys and credentials in the existing secure credential boundary; never put secrets in runtime settings, SQLite, TypeScript contracts or messages.
- Chat output must not silently write canonical project memory, manuscripts, characters or canon.
- Include only the shared chat/settings foundation; defer local model download, Hub, MCP, tools, agents, attachments, semantic retrieval, automatic memory extraction, cloud sync, collaboration and export.
- Keep generated `src-tauri/gen/`, `src-tauri/icons/`, target directories, SQLite runtime data, credentials and `node_modules` outside commits.
- Run the repository-required checks after source changes: `npm run typecheck`, `npm run lint`, `npm test -- --run`, `npm run build`, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`.

## Review Focus

- Existing conversations without a runtime row load with `general-assistant`, automatic routing, `balanced` quality and `null` temperature; the migration and load test must pin these defaults.
- A removed or invalid provider/model selection is rejected by Rust, leaves the last confirmed client selection visible and never reaches provider execution; runtime validation tests must cover this.
- A retry after a provider failure preserves the draft and appends no duplicate user message; Rust and React retry tests must cover this.
- An archived project or chapter is rejected before provider execution or message mutation; the shared runtime integration test must cover this.
- Restart restores non-secret runtime metadata while credential values remain outside serialized settings and messages; persistence and command contract tests must cover this.

---

### Task 1: Persist typed conversation runtime settings

**Files:**
- Create: `migrations/0011_conversation_runtime_settings.sql`
- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/domain/conversation.rs`
- Modify: `src-tauri/src/conversations/repository.rs`
- Modify: `src-tauri/src/conversations/service.rs`
- Test: `src-tauri/src/db.rs` and `src-tauri/src/conversations/service.rs` unit tests

**Interfaces:**
- Produces `ChatRuntimeSettings { assistant_id: String, provider_id: Option<String>, model_id: Option<String>, quality: QualityMode, temperature: Option<f32>, updated_at: String }`.
- Produces `ChatRuntimeSettingsInput { assistant_id: String, provider_id: Option<String>, model_id: Option<String>, quality: QualityMode, temperature: Option<f32> }`.
- `ConversationRepository::get_runtime_settings(&self, conversation_id: &str) -> AppResult<Option<ChatRuntimeSettings>>`.
- `ConversationRepository::upsert_runtime_settings(&self, conversation_id: &str, input: ChatRuntimeSettingsInput) -> AppResult<ChatRuntimeSettings>`.
- `ConversationService::runtime_settings(&self, conversation_id: &str) -> AppResult<ChatRuntimeSettings>` returns deterministic defaults when no row exists.
- `ConversationService::update_runtime_settings(&self, conversation_id: &str, input: ChatRuntimeSettingsInput) -> AppResult<ChatRuntimeSettings>` validates non-empty assistant id, quality and temperature range before repository persistence.

- [ ] **Step 1: Write the failing migration and service tests**

  Add tests proving that `db::in_memory()` applies migration `0011`, repeated migration runs are idempotent, a new conversation gets the exact default settings, an update persists provider/model/quality/temperature values, and a missing conversation returns `not_found`.

- [ ] **Step 2: Run the focused Rust tests and verify they fail**

  Run `cargo test runtime_settings -- --nocapture`.
  Expected: compilation or assertion failure because the migration, types and service methods do not exist yet.

- [ ] **Step 3: Add `0011_conversation_runtime_settings.sql` and register it in `db.rs`**

  Create one row per conversation with a foreign key to `conversations`, nullable provider/model columns, a `quality` check for `fast`, `balanced` and `deep`, an optional temperature, an `updated_at` value and an index only if the repository query needs it. Register the migration after `0010_provider_settings.sql`.

- [ ] **Step 4: Implement the typed domain and repository/service methods**

  Keep provider/model ids as separate nullable columns. Map SQLite rows to typed values and use `now_utc()` for updates. Apply `general-assistant`, `balanced`, `None` provider/model and `None` temperature when the row is absent; do not materialize a row during a read.

- [ ] **Step 5: Run the focused Rust tests and verify they pass**

  Run `cargo test runtime_settings -- --nocapture`.
  Expected: all migration, default, update, validation and not-found tests pass.

- [ ] **Step 6: Commit the persistence slice**

  ```bash
  git add migrations/0011_conversation_runtime_settings.sql src-tauri/src/db.rs src-tauri/src/domain/conversation.rs src-tauri/src/conversations/repository.rs src-tauri/src/conversations/service.rs
  git commit -m "feat: persist conversation runtime settings"
  ```

### Task 2: Add the shared Rust chat runtime and typed Tauri commands

**Files:**
- Create: `src-tauri/src/conversations/runtime.rs`
- Create: `src-tauri/tests/chat_runtime_flow.rs`
- Modify: `src-tauri/src/conversations/mod.rs`
- Modify: `src-tauri/src/conversations/chat.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/domain/conversation.rs`
- Modify: `src-tauri/src/error.rs` only if a narrowly scoped runtime error code is required

**Interfaces:**
- `ChatRuntimeLoadRequest { project_id: String, chapter_id: Option<String>, conversation_id: Option<String>, kind: ConversationKind }`.
- `ChatRuntimeSnapshot { conversations: Vec<Conversation>, conversation: Option<Conversation>, messages: Vec<ConversationMessage>, settings: ChatRuntimeSettings, assistants: Vec<ChatAssistantDescriptor>, models: Vec<ModelProfile> }`; an empty project scope returns `conversation: None` and an empty message list so the client can create the first conversation with the requested project/chapter scope.
- `ChatAssistantDescriptor { id: String, label: String, description: String }` with stable ids for `general-assistant`, `world-builder`, `continuity-reviewer` and `writing-coach`.
- `ChatSendRequest { conversation_id: String, task: ModelTask, runtime: ChatRuntimeSettingsInput, system_instructions: String, character_ids: Vec<String>, include_character_states: bool, context_budget: ContextBudget, message: String, retry_attempt: bool }`.
- `ChatSendResult` preserves the current conversation, user message, assistant message and orchestration result fields.
- `ChatRuntimeService::load(&self, request: ChatRuntimeLoadRequest) -> AppResult<ChatRuntimeSnapshot>`.
- `ChatRuntimeService::update_settings(&self, conversation_id: &str, input: ChatRuntimeSettingsInput) -> AppResult<ChatRuntimeSettings>`.
- `ChatRuntimeService::send(&self, request: ChatSendRequest) -> AppResult<ChatSendResult>` as `pub async fn`.
- Tauri commands: `chat_runtime_load`, `chat_runtime_update`, `chat_send`.

- [ ] **Step 1: Write failing integration tests for the shared runtime**

  Add `chat_runtime_flow.rs` tests for default snapshot loading, model/provider validation, one successful send, provider failure followed by retry without duplicate user rows, and archived project/chapter rejection before provider execution. Use the public mock provider types and a local integration-test fixture; do not depend on private unit-test helpers from `conversations/chat.rs`.

- [ ] **Step 2: Run the new integration tests and verify they fail**

  Run `cargo test --test chat_runtime_flow -- --nocapture`.
  Expected: compilation failure for the missing runtime service, types and commands.

- [ ] **Step 3: Implement `ChatRuntimeService` in `conversations/runtime.rs`**

  Compose the existing `ConversationService`, `ContextCompiler`, `ServiceContextSource`, `ProviderRegistry`, `ManuscriptService` and existing chat send logic. Load the project-scoped conversations and messages, return the assistant catalog and model list, validate the persisted runtime choice, then delegate send work to the existing provider-independent orchestration path. Read the conversation’s chapter scope instead of trusting a duplicate client field.

- [ ] **Step 4: Make existing chat commands delegate to the shared service**

  Keep `developer_chat_send` and `chapter_chat_send` as compatibility adapters for current callers and tests. Convert their existing request into `ChatSendRequest`, preserve chapter ownership checks and current manuscript context behavior, and return the existing result shape.

- [ ] **Step 5: Register `ChatRuntimeService` and expose typed commands**

  Add the service to `AppState`, initialize it in `lib.rs`, register `chat_runtime_load`, `chat_runtime_update` and `chat_send` in the Tauri invoke handler, and keep all command arguments under `rename_all = "snake_case"`.

- [ ] **Step 6: Run the integration tests and verify they pass**

  Run `cargo test --test chat_runtime_flow conversations:: -- --nocapture`.
  Expected: snapshot defaults, runtime validation, send, retry deduplication and archived-scope tests pass.

- [ ] **Step 7: Commit the shared runtime slice**

  ```bash
  git add src-tauri/src/conversations src-tauri/src/commands.rs src-tauri/src/lib.rs src-tauri/src/domain/conversation.rs src-tauri/src/error.rs src-tauri/tests/chat_runtime_flow.rs
  git commit -m "feat: add shared chat runtime boundary"
  ```

### Task 3: Add TypeScript runtime contracts, commands and reducer tests

**Files:**
- Modify: `apps/client/src/types/conversation.ts`
- Modify: `apps/client/src/types/provider.ts` only for shared model/event types
- Modify: `apps/client/src/lib/commands.ts`
- Modify: `apps/client/src/lib/commands.test.ts`
- Create: `apps/client/src/features/chat/chat-runtime.ts`
- Create: `apps/client/src/features/chat/chat-runtime.test.ts`

**Interfaces:**
- Mirror the Rust `ChatRuntimeSettings`, `ChatRuntimeSettingsInput`, `ChatRuntimeLoadRequest`, `ChatAssistantDescriptor`, `ChatRuntimeSnapshot`, `ChatSendRequest` and `ChatSendResult` shapes in snake_case.
- Add `ChatEvent` as a discriminated union for `accepted`, `delta`, `completed`, `retryable_error`, `terminal_error` and `cancelled`.
- Add `loadChatRuntime(request: ChatRuntimeLoadRequest): Promise<ChatRuntimeSnapshot>`.
- Add `updateChatRuntime(conversationId: string, input: ChatRuntimeSettingsInput): Promise<ChatRuntimeSettings>`.
- Add `sendChat(request: ChatSendRequest): Promise<ChatSendResult>`.
- Add `ChatRuntimeState`, `ChatRuntimeAction` and `reduceChatRuntime(state, action)` in `chat-runtime.ts`.

- [ ] **Step 1: Write failing command and reducer tests**

  Assert exact snake_case invoke payloads for load, update and send. Assert that reducer transitions preserve the draft on retryable error, do not append a duplicate user message on retry, restore the last confirmed runtime setting on update failure, and expose terminal errors without secret or SQL text.

- [ ] **Step 2: Run focused Vitest tests and verify they fail**

  Run `npm test -- --run apps/client/src/features/chat/chat-runtime.test.ts apps/client/src/lib/commands.test.ts`.
  Expected: missing exports and reducer failures.

- [ ] **Step 3: Implement typed command wrappers and reducer**

  Reuse the existing `call<T>` normalization path. Keep the reducer pure and make every message append come from `ChatSendResult`; a retry action must never synthesize a second user message locally.

- [ ] **Step 4: Run focused Vitest tests and verify they pass**

  Run the command from Step 2.
  Expected: command payload and reducer tests pass.

- [ ] **Step 5: Commit the client contract slice**

  ```bash
  git add apps/client/src/types/conversation.ts apps/client/src/types/provider.ts apps/client/src/lib/commands.ts apps/client/src/lib/commands.test.ts apps/client/src/features/chat/chat-runtime.ts apps/client/src/features/chat/chat-runtime.test.ts
  git commit -m "feat: add typed chat runtime client contracts"
  ```

### Task 4: Build the shared Jan-style chat workspace

**Files:**
- Create: `apps/client/src/features/chat/ChatConversationSidebar.tsx`
- Create: `apps/client/src/features/chat/ChatRuntimeMenu.tsx`
- Create: `apps/client/src/features/chat/ChatWorkspace.tsx`
- Create: `apps/client/src/features/chat/useChatRuntime.ts`
- Modify: `apps/client/src/features/chat/DeveloperChatPanel.tsx`
- Modify: `apps/client/src/features/manuscripts/ChapterChatPanel.tsx`
- Modify: `apps/client/src/features/chat/ChatHeader.tsx`
- Modify: `apps/client/src/features/chat/ChatComposer.tsx`
- Modify: `apps/client/src/features/chat/ChatMessageList.tsx`
- Create or modify: `apps/client/src/test/ChatWorkspace.test.tsx`
- Modify: `apps/client/src/test/DeveloperChatPanel.test.tsx`
- Modify: `apps/client/src/test/ChapterChatPanel.test.tsx`
- Modify: `apps/client/src/test/ChatRuntimeControls.test.tsx`
- Modify: `apps/client/src/test/ChatSurface.test.tsx`

**Interfaces:**
- `ChatWorkspaceProps { projectId: string; chapterId?: string | null; kind: ConversationKind; title: string; scopeLabel: string; disabled?: boolean; task: ModelTask }`.
- `useChatRuntime(props: ChatWorkspaceProps): { state: ChatRuntimeState; send(): Promise<void>; retry(): Promise<void>; updateSettings(input: ChatRuntimeSettingsInput): Promise<void>; selectConversation(id: string): Promise<void>; createConversation(): Promise<void>; setDraft(value: string): void }`.
- Child components receive typed state and callbacks; no child calls `invoke` or imports command wrappers.

- [ ] **Step 1: Write failing component tests for the shared workspace**

  Assert the conversation sidebar renders the active conversation and new-chat action, the header exposes the selected model, the runtime menu keeps assistant/provider/model/quality choices one click away, the composer preserves drafts during failures, the retry action does not duplicate a user turn, and archived scopes disable sending.

- [ ] **Step 2: Run focused Vitest tests and verify they fail**

  Run `npm test -- --run apps/client/src/test/ChatWorkspace.test.tsx apps/client/src/test/DeveloperChatPanel.test.tsx apps/client/src/test/ChapterChatPanel.test.tsx`.
  Expected: missing shared components/hook and changed behavior assertions.

- [ ] **Step 3: Implement the pure `useChatRuntime` orchestration hook**

  Load one runtime snapshot per scope, keep the draft local, dispatch reducer actions for load/update/send/retry, and restore the confirmed settings after an update failure. Use the existing project/chapter disabled rules before invoking send.

- [ ] **Step 4: Implement the shared workspace components**

  Move common conversation, header, message and composer behavior into `ChatWorkspace`. Use the header model selector for the frequent model choice and render the advanced runtime controls in `ChatRuntimeMenu`. Keep Chapter Chat’s manuscript/proposal behavior outside the shared workspace boundary.

- [ ] **Step 5: Convert Developer Chat and Chapter Chat to workspace adapters**

  Supply scope, kind, title, task and disabled state from each feature page. Preserve existing developer assistant presets and chapter manuscript context while routing all chat state through the shared hook.

- [ ] **Step 6: Run focused Vitest tests and verify they pass**

  Run the command from Step 2.
  Expected: shared workspace, developer chat, chapter chat, retry and archived-scope tests pass.

- [ ] **Step 7: Commit the shared chat workspace slice**

  ```bash
  git add apps/client/src/features/chat apps/client/src/features/manuscripts/ChapterChatPanel.tsx apps/client/src/test/ChatWorkspace.test.tsx apps/client/src/test/DeveloperChatPanel.test.tsx apps/client/src/test/ChapterChatPanel.test.tsx apps/client/src/test/ChatRuntimeControls.test.tsx apps/client/src/test/ChatSurface.test.tsx
  git commit -m "feat: build shared Jan-style chat workspace"
  ```

### Task 5: Restructure the settings workspace around provider and model discovery

**Files:**
- Create: `apps/client/src/features/providers/SettingsNavigation.tsx`
- Modify: `apps/client/src/features/providers/SettingsLayout.tsx`
- Modify: `apps/client/src/features/providers/ProviderSettingsPage.tsx`
- Modify: `apps/client/src/features/providers/ProviderNavigation.tsx`
- Modify: `apps/client/src/features/providers/ProviderDetailsPanel.tsx`
- Modify: `apps/client/src/features/providers/ProviderModelList.tsx`
- Modify: `apps/client/src/app/routes.tsx`
- Modify: `apps/client/src/test/ProviderSettingsPage.test.tsx`
- Create or modify: `apps/client/src/test/SettingsLayout.test.tsx`

**Interfaces:**
- `SettingsLayout` renders the section navigation around its typed child content; the existing provider page remains the first functional settings section.
- `SettingsNavigation` exposes `Model Providers` as the first active section and keeps Chat, Appearance and Storage out of the route until they have real behavior.
- `ProviderSettingsPage` continues to use `listProviders`, `listModels`, `getProviderSettings`, `configureProvider`, `updateProvider`, `testProvider`, `removeProvider` and `getCredentialStoreStatus` without moving credential values into React persistence.

- [ ] **Step 1: Write failing settings tests**

  Assert that the settings shell renders the provider section, configured and unconfigured providers remain distinguishable, models are grouped under the selected provider, secure-store status is visible without secret values, and edit/test/remove actions preserve existing typed commands.

- [ ] **Step 2: Run focused Vitest tests and verify they fail**

  Run `npm test -- --run apps/client/src/test/SettingsLayout.test.tsx apps/client/src/test/ProviderSettingsPage.test.tsx`.
  Expected: missing settings navigation and changed layout assertions.

- [ ] **Step 3: Implement the settings shell and provider/model layout**

  Keep provider navigation as the catalog inside the `Model Providers` section. Put provider details and model information in the main panel, and retain explicit connection test, remove and secure-store status behavior.

- [ ] **Step 4: Add the `/settings` route and preserve `/settings/providers`**

  Route `/settings` to the settings shell and redirect its default section to `/settings/providers`. Update links from chat runtime notes and the application shell to use the settings root or provider section as appropriate.

- [ ] **Step 5: Run focused Vitest tests and verify they pass**

  Run the command from Step 2.
  Expected: settings shell and provider lifecycle tests pass with no secret leakage.

- [ ] **Step 6: Commit the settings workspace slice**

  ```bash
  git add apps/client/src/features/providers apps/client/src/app/routes.tsx apps/client/src/test/SettingsLayout.test.tsx apps/client/src/test/ProviderSettingsPage.test.tsx
  git commit -m "feat: add Jan-style settings workspace"
  ```

### Task 6: Apply the Jan-inspired application shell and responsive visual system

**Files:**
- Modify: `apps/client/src/components/layout/AppShell.tsx`
- Modify: `apps/client/src/components/layout/Sidebar.tsx`
- Modify: `apps/client/src/components/layout/ContextPanel.tsx` only where chat focus requires it
- Modify: `apps/client/src/components/layout/StatusBar.tsx` only where shell state requires it
- Modify: `apps/client/src/styles.css`
- Modify: `apps/client/src/test/AppShell.test.tsx`
- Modify: `apps/client/src/test/ChatSurface.test.tsx`

**Interfaces:**
- Keep route paths and `NavLink` semantics stable for `/projects`, project chat, project manuscripts and `/settings/providers`.
- Preserve accessible labels for the primary rail, settings navigation, conversation list, model selector and composer.

- [ ] **Step 1: Write failing shell and responsive tests**

  Assert the primary rail exposes Projects, Chat and Settings, project-scoped navigation remains available, the chat surface has a scrollable message region and bottom composer, and narrow layouts expose the mobile navigation without hiding the active route.

- [ ] **Step 2: Run focused Vitest tests and verify they fail**

  Run `npm test -- --run apps/client/src/test/AppShell.test.tsx apps/client/src/test/ChatSurface.test.tsx`.
  Expected: changed navigation and layout assertions.

- [ ] **Step 3: Implement shell markup and CSS tokens**

  Establish a compact dark Jan-inspired shell with a narrow rail, secondary sidebar, bordered workspace surfaces, consistent muted text, selected navigation state, responsive collapse rules and a bottom-pinned composer. Keep the Webnovel Studio identity and existing project context semantics.

- [ ] **Step 4: Run focused Vitest tests and verify they pass**

  Run the command from Step 2.
  Expected: shell, accessibility and responsive behavior tests pass.

- [ ] **Step 5: Start the native Tauri app and perform a visual smoke check**

  Run `npm.cmd run tauri:dev` from the nested `phase13/implementation` checkout. If the checkout lacks generated `src-tauri/icons/`, create only the local icon asset required by Tauri and keep it untracked. Verify the Projects route, a project chat route, `/settings/providers`, model selector and responsive shell render without a runtime error. Keep generated icons and `src-tauri/gen/` untracked.

- [ ] **Step 6: Commit the shell and visual slice**

  ```bash
  git add apps/client/src/components/layout apps/client/src/styles.css apps/client/src/test/AppShell.test.tsx apps/client/src/test/ChatSurface.test.tsx
  git commit -m "feat: apply Jan-inspired application shell"
  ```

### Task 7: Update durable context and run the complete verification gate

**Files:**
- Modify: `docs/DEVELOPMENT_CONTEXT.md`
- Modify: `docs/CHAT_CONTEXT.md`
- Test: repository-wide commands below

**Interfaces:**
- Documentation must state the actual branch, completed Jan-style runtime/settings scope, deferred Jan ecosystem features and the generated-output boundary.

- [ ] **Step 1: Update the durable context documents**

  Record the new migration, shared runtime boundary, settings/chat surfaces, persistence rules, current verification baseline and remaining deferred scope. Do not claim streaming provider support if only the event boundary is implemented.

- [ ] **Step 2: Run all required verification commands**

  ```bash
  npm run typecheck
  npm run lint
  npm test -- --run
  npm run build
  cargo fmt --all -- --check
  cargo clippy --all-targets -- -D warnings
  cargo test
  ```

  Expected: every command exits successfully. If an existing formatting drift is reported, identify whether it is pre-existing before changing unrelated files.

- [ ] **Step 3: Inspect the final diff and generated-file boundary**

  Run `git status --short --branch`, `git diff --check origin/phase13/implementation...HEAD` and `git diff --stat origin/phase13/implementation...HEAD`. Confirm only intended source, test and context files are tracked; generated icons, `src-tauri/gen/`, targets, runtime databases and credentials remain uncommitted.

- [ ] **Step 4: Commit durable context and verification results**

  ```bash
  git add docs/DEVELOPMENT_CONTEXT.md docs/CHAT_CONTEXT.md
  git commit -m "docs: record Jan-style chat runtime completion"
  ```

## Execution Notes

Implement tasks in order because each task publishes typed interfaces used by
the next one. Keep each task's focused test run green before moving forward.
Use the current `phase13/implementation` branch or a feature branch from it;
do not merge implementation work into `main`.
