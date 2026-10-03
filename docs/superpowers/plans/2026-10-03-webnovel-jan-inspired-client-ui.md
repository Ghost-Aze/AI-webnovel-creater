# Jan-Inspired Provider and Chat Client UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (native execution) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make provider settings and Developer/Chapter Chat feel like one Jan-inspired AI workspace while exposing safe, actionable provider error causes and keeping failed chat retries duplicate-free.

**Architecture:** Keep all provider, conversation, SQLite and secure-credential behavior behind the existing Rust commands. Extend the typed provider error contract at the Rust HTTP boundary, normalize it once in the client, and pass typed state into focused chat and provider components. The chat panels retain loading/request/proposal state; presentational components own the empty, active, pending and error layouts.

**Tech Stack:** Rust 2021, Tauri commands, reqwest/rustls, SQLite repositories, React 18, TypeScript, React Router, Vitest, Testing Library, CSS media queries.

**Spec:** `docs/superpowers/specs/2026-10-03-webnovel-jan-inspired-client-ui-design.md`

## Global Constraints

- Work only on `phase13/implementation`; do not merge into `main` or start Phase 14.
- Preserve the provider-independent boundary, structured SQLite repositories, secure credential storage and explicit canonical writes.
- Provider settings are populated from `provider_list`; no hardcoded Jan provider catalog is introduced.
- Raw provider response bodies, prompts and API keys never enter serialized errors, rendered text or logs.
- Existing typed command wrappers remain the only client/backend boundary.
- Failed chat attempts remain available for retry without appending a duplicate user message.
- Run the repository-required checks after every source change: `npm run typecheck`, `npm run lint`, `npm test -- --run`, `npm run build`, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`.

## Review Focus

- HTTP 401/403 and missing secure credentials show a settings action, never a retry, and never expose the provider body (Task 1 tests).
- HTTP 404 distinguishes an unavailable provider endpoint/model from local provider registry errors (Task 1 tests).
- HTTP 408, request timeouts and network failures produce retryable safe errors (Task 1 tests).
- HTTP 429 and HTTP 5xx produce retryable errors while preserving the draft and message history (Tasks 1 and 3 tests).
- Retrying a failed user turn reuses the persisted last user message instead of appending a second identical turn (Task 2 and Task 3 tests).

### Task 1: Introduce typed provider failure classification

**Files:**
- Modify: `src-tauri/src/provider/error.rs`
- Modify: `src-tauri/src/provider/http.rs`
- Modify: `src-tauri/src/error.rs`
- Modify: `apps/client/src/lib/command-error.ts`
- Test: `src-tauri/src/provider/error.rs` and `src-tauri/src/provider/http.rs` unit tests
- Test: `apps/client/src/lib/command-error.test.ts`

**Interfaces:**
- `ProviderError` produces `Unauthorized`, `EndpointNotFound`, `RateLimited`, `Unavailable`, `Timeout`, `NetworkFailure`, and the existing generic `ProviderFailure` variants.
- `AppError` serializes stable codes `provider_unauthorized`, `provider_endpoint_not_found`, `provider_rate_limited`, `provider_unavailable`, `provider_timeout`, and `provider_network_failure`.
- `normalizeCommandError(value)` returns a `CommandError` with `code`, safe `message`, `retryable: boolean`, and `action: "retry" | "provider_settings" | null`.

- [ ] **Step 1: Write failing Rust classification tests.** Add table-driven adapter tests for 400/422 → `InvalidRequest`, 401/403 → `Unauthorized`, 404 → `EndpointNotFound`, 408 → `Timeout`, 429 → `RateLimited`, 500–599 → `Unavailable`, request timeout → `Timeout`, and other transport failures → `NetworkFailure`. Assert serialized errors contain only the safe code and never a fake response body, prompt or token.
- [ ] **Step 2: Run the focused Rust tests and verify they fail.** Run `cargo test provider::http::tests provider::error::tests`; expected failure is the missing variants/mapping.
- [ ] **Step 3: Implement the minimal Rust mapping.** Add the provider variants and an explicit status/transport mapper. Map reqwest timeout errors to `RequestTimeout`; keep malformed successful payloads and stream parse errors as generic `ProviderFailure`; discard response bodies after status classification.
- [ ] **Step 4: Write failing TypeScript normalization tests.** Assert each new code maps to stable Turkish-neutral English UI copy, retry metadata and settings/retry action; assert unknown codes remain the generic non-retryable fallback and raw `details` are ignored.
- [ ] **Step 5: Implement `CommandError` metadata and mappings.** Preserve existing codes and validation details, add the new provider codes, and make unauthorized/credential/endpoint errors point to Provider settings while rate-limit, timeout, network and temporary-unavailable errors are retryable.
- [ ] **Step 6: Run the full required verification suite.** Confirm all seven commands in Global Constraints pass before moving to the conversation contract.

### Task 2: Make chat retry attempts idempotent at the conversation boundary

**Files:**
- Modify: `src-tauri/src/conversations/chat.rs`
- Modify: `apps/client/src/types/conversation.ts`
- Modify: `apps/client/src/features/chat/DeveloperChatPanel.tsx`
- Modify: `apps/client/src/features/manuscripts/ChapterChatPanel.tsx`
- Test: `src-tauri/src/conversations/chat.rs` tests
- Test: `apps/client/src/test/DeveloperChatPanel.test.tsx`
- Test: `apps/client/src/test/ChapterChatPanel.test.tsx`

**Interfaces:**
- `DeveloperChatSendRequest` gains `retry_attempt: boolean`; normal sends pass `false`, and an explicit retry passes `true`.
- `DeveloperChatService::send_for_kind` reuses the latest matching user message when `retry_attempt` is true and no assistant message follows it; otherwise it returns `AppError::Validation` without creating another user turn.
- Chat panels keep `draft` and a failed-attempt flag after a provider command error and clear both only after a successful assistant response.

- [ ] **Step 1: Write failing backend retry tests.** Add tests that create a conversation with a last user message, make the first generation fail, retry with `retry_attempt: true`, and assert the conversation has one user message plus one assistant message. Add a mismatch test that rejects retry without changing stored messages.
- [ ] **Step 2: Run the focused backend tests and verify they fail.** Run `cargo test conversations::chat::tests`; expected failure is the missing request field and duplicate behavior.
- [ ] **Step 3: Implement retry matching.** Add the request field, inspect the latest conversation messages before append, reuse only the latest user message whose content equals the request message when retrying, and keep the existing chapter scope and orchestration inputs unchanged.
- [ ] **Step 4: Update TypeScript request construction.** Add `retry_attempt` to the shared type and send it from both panels; preserve it as `false` for normal submissions.
- [ ] **Step 5: Add failing panel tests for draft preservation and retry.** Assert a provider error leaves the text in the composer, renders a retry affordance only for retryable errors, calls the command with `retry_attempt: true`, and does not add duplicate visible user messages. Assert unauthorized errors render a Provider settings action instead.
- [ ] **Step 6: Implement panel retry state.** Store the last failed message locally, normalize the command error, route retry/settings callbacks to the new visual error component, and clear the failed state only after success. Keep archived/read-only and proposal behavior intact.
- [ ] **Step 7: Run the full required verification suite.** Confirm the retry contract passes Rust and frontend checks before extracting visual components.

### Task 3: Extract the shared Jan-style chat surface

**Files:**
- Create: `apps/client/src/features/chat/ChatHeader.tsx`
- Create: `apps/client/src/features/chat/ChatEmptyState.tsx`
- Create: `apps/client/src/features/chat/ChatMessageList.tsx`
- Create: `apps/client/src/features/chat/ChatComposer.tsx`
- Create: `apps/client/src/features/chat/ChatErrorBanner.tsx`
- Modify: `apps/client/src/features/chat/ChatRuntimeControls.tsx`
- Modify: `apps/client/src/features/chat/DeveloperChatPanel.tsx`
- Modify: `apps/client/src/styles.css`
- Test: `apps/client/src/test/ChatSurface.test.tsx`

**Interfaces:**
- `ChatHeader` accepts scope title and selected `ModelRef | null` plus the resolved model label.
- `ChatEmptyState` accepts the shared `ChatComposer` props and renders the centered welcome/composer state.
- `ChatMessageList` accepts `ConversationMessage[]`, a scroll-region class and an optional assistant-action callback for Chapter proposals.
- `ChatComposer` accepts `draft`, `onDraftChange`, `onSubmit`, `isSending`, `disabled`, assistant/model/quality selection props and renders the compact assistant/provider/model/quality affordances inside the composer.
- `ChatErrorBanner` accepts normalized `CommandError`, `onRetry`, and `onOpenProviderSettings`; it omits actions when metadata says no action is available.

- [ ] **Step 1: Write failing component tests.** Cover empty state centering, active state message-list scroll region plus bottom composer, selected model visibility in the header, pending send disabling, accessible status/alert semantics, and error action selection.
- [ ] **Step 2: Run the focused frontend test and verify it fails.** Run `npm test -- --run apps/client/src/test/ChatSurface.test.tsx`; expected failure is missing components/layout.
- [ ] **Step 3: Implement the focused presentational components.** Keep command calls and state transitions out of them; reuse the existing runtime control data contract and assistant presets.
- [ ] **Step 4: Switch `DeveloperChatPanel` to state-driven composition.** Render `ChatEmptyState` only when `messages.length === 0`, otherwise render `ChatHeader`, `ChatMessageList` and the bottom `ChatComposer`; keep the message list as the only scrolling region.
- [ ] **Step 5: Add Jan-style desktop and narrow-screen CSS.** Center the empty composer, pin the active composer within the chat surface, preserve safe-area padding, and ensure the message list remains scrollable while the composer stays visible.
- [ ] **Step 6: Run focused tests and the full required verification suite.** Confirm the state transition and accessibility checks pass at desktop and narrow viewport test sizes.

### Task 4: Reuse the chat surface for Chapter Chat

**Files:**
- Modify: `apps/client/src/features/manuscripts/ChapterChatPanel.tsx`
- Modify: `apps/client/src/styles.css`
- Test: `apps/client/src/test/ChapterChatPanel.test.tsx`

**Interfaces:**
- Chapter Chat passes the same `ChatHeader`, `ChatEmptyState`, `ChatMessageList`, `ChatComposer` and `ChatErrorBanner` props as Developer Chat while retaining chapter-scoped `chapterChatSend` and `onProposalCreated`.

- [ ] **Step 1: Extend failing Chapter Chat tests.** Assert centered empty composer, active bottom composer, preserved chapter proposal buttons, retry/settings actions and `retry_attempt` propagation.
- [ ] **Step 2: Run the focused test and verify the new assertions fail.** Run `npm test -- --run apps/client/src/test/ChapterChatPanel.test.tsx`.
- [ ] **Step 3: Replace Chapter Chat’s duplicated layout with the shared surface.** Keep chapter title/scope and proposal actions in typed callbacks; do not move proposal creation into presentational components.
- [ ] **Step 4: Run the full required verification suite.** Confirm Developer and Chapter Chat remain separate in command scope and share only visual behavior.

### Task 5: Build the Jan-style provider/settings workspace

**Files:**
- Create: `apps/client/src/features/providers/SettingsLayout.tsx`
- Create: `apps/client/src/features/providers/ProviderNavigation.tsx`
- Create: `apps/client/src/features/providers/ProviderModelList.tsx`
- Create: `apps/client/src/features/providers/ProviderDetailsPanel.tsx`
- Modify: `apps/client/src/features/providers/ProviderSettingsPage.tsx`
- Modify: `apps/client/src/styles.css`
- Test: `apps/client/src/test/ProviderSettingsPage.test.tsx`

**Interfaces:**
- `SettingsLayout` accepts the active settings route and provider navigation content.
- `ProviderNavigation` accepts `ProviderDescriptor[]`, `selectedProviderId`, `onSelect`, and `onAdd`.
- `ProviderModelList` accepts `ModelProfile[]`, `onEdit`, and `onRemove` only for actions supported by existing commands.
- `ProviderDetailsPanel` accepts `ProviderSettings | null`, secure-store status, loading/error/test state, and typed edit/test/remove/add-model callbacks.

- [ ] **Step 1: Write failing provider workspace tests.** Cover provider selection from `provider_list`, first-provider selection, OpenRouter draft when empty, model details/actions, safe connection result/error, blank API-key preservation on edit, and metadata visibility when secure credentials are unavailable.
- [ ] **Step 2: Run the focused provider tests and verify they fail.** Run `npm test -- --run apps/client/src/test/ProviderSettingsPage.test.tsx`.
- [ ] **Step 3: Implement the secondary settings/provider navigation.** Keep the provider list data-driven and make selection load `provider_get` without requiring the secret value.
- [ ] **Step 4: Implement model/details panels.** Reuse existing validation and command wrappers; keep API key fields password inputs and never echo the value.
- [ ] **Step 5: Add provider-specific responsive layout styling.** Desktop uses a settings rail plus provider rail and details pane; narrow screens collapse navigation into a drawer/sheet without losing the selected provider or form focus.
- [ ] **Step 6: Run the full required verification suite.** Confirm provider selection, editing, testing, removal and credential-unavailable behavior remain green.

### Task 6: Integrate responsive navigation and perform final verification

**Files:**
- Modify: `apps/client/src/components/layout/AppShell.tsx`
- Modify: `apps/client/src/components/layout/Sidebar.tsx`
- Modify: `apps/client/src/styles.css`
- Modify: relevant layout tests under `apps/client/src/test/`

**Interfaces:**
- Existing global/project navigation remains route-compatible; settings navigation is rendered inside the settings workspace, and mobile bottom navigation remains the primary route switcher.

- [ ] **Step 1: Write failing layout tests.** Assert project-scoped Chat/Manuscripts links stay in the project navigation, settings provider navigation is not duplicated in the global rail, and narrow layouts expose the drawer/bottom route controls.
- [ ] **Step 2: Implement the smallest navigation/CSS changes.** Preserve current routes and keyboard focus behavior while adding visible labels and drawer close/recovery behavior.
- [ ] **Step 3: Run the full required verification suite.** Run all seven commands from Global Constraints and record their output.
- [ ] **Step 4: Run a native Tauri smoke test.** Start the desktop app, verify provider settings and both chat states against the native command boundary, and record any environment-only limitation separately from source failures.
- [ ] **Step 5: Review the final diff.** Run `git diff --check`, confirm no credentials, SQLite runtime data, `node_modules`, Rust targets or generated Tauri build output are staged, and keep the branch on `phase13/implementation`.
