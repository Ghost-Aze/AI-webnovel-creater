# Jan-style chat runtime and settings workspace

## Status

Approved design for the next implementation slice. This document describes
the product and architecture boundary; implementation details belong in the
follow-up plan.

## Goal

Give Webnovel AI Studio a Jan-inspired desktop experience for chat and
settings while preserving the project's local-first, provider-independent
domain boundaries. The change covers both the visible workspace and the
shared runtime that loads conversations, validates model choices, routes
provider calls, persists messages and exposes retry-safe errors.

Jan is a behavioral and interaction reference only. The project does not copy
Jan source code, its local inference engine, its Hub, MCP ecosystem or agent
platform. The current Jan reference is the public `janhq/jan` project and its
0.8.4 settings model:

- https://github.com/janhq/jan
- https://github.com/janhq/jan/releases/tag/v0.8.4
- https://www.jan.ai/docs/desktop/settings

## User outcome

A writer opens a project and sees one coherent workspace. A narrow navigation
rail leads to projects, chat and settings. Chat has a conversation list, a
clear header with the active model, a scrollable message region and a bottom
pinned composer. Provider and model choices are discoverable from chat and
fully manageable from a settings workspace. Changing a model, retrying a
failed response or reopening the app does not lose the draft, duplicate a
message or bypass canonical project write rules.

## Scope

### Included

- Jan-inspired application shell and settings navigation.
- Shared chat runtime for developer chat and chapter chat.
- Conversation-scoped assistant, provider, model and quality settings.
- Typed runtime load, update and send boundaries.
- Safe loading, empty, retryable-error and terminal-error states.
- A streaming-ready event model with the existing completed-response path as
  the first provider implementation.
- Provider/model catalog and model selector integration.
- SQLite persistence for non-secret conversation runtime settings.
- Rust and React tests for the new state transitions and boundaries.

### Deferred

- Local model download and Hub workflows.
- MCP servers, tools, agents, attachments and autonomous actions.
- Automatic memory extraction, semantic retrieval, continuity AI, cloud sync,
  collaboration and export.
- Copying Jan's local engine, provider adapters or UI source.
- Silent writes to project memory, manuscripts, characters or canon.

## Architectural design

### Rust runtime boundary

Add a shared `ChatRuntimeService` above the existing conversation,
context, routing and provider services. It owns the request lifecycle:

1. Load the requested conversation and project/chapter scope.
2. Validate the runtime settings against the available assistant and model
   contracts.
3. Build the provider-independent context request.
4. Resolve a model through the existing routing policy unless a valid
   preferred model is supplied.
5. Invoke the provider runtime.
6. Persist the user and assistant messages through the conversation
   repository boundary.
7. Return a typed result or a safe typed error.

`developer_chat_send` and `chapter_chat_send` remain compatibility adapters
for their existing command callers, but delegate their shared work to the new
runtime service. A unified `chat_send` request is the preferred client
boundary for new UI code.

The service must not expose credentials, provider secrets, SQL errors or
prompts in error payloads. It must not write canonical memory or manuscript
content as a side effect of producing an answer.

### Runtime settings

Introduce a typed conversation runtime settings value containing:

- assistant preset id;
- optional provider id;
- optional model id;
- quality mode;
- optional temperature and future generation parameters.

The provider and model references are validated in Rust. Secrets remain in the
existing credential store and are never included in the runtime settings row,
TypeScript contract or serialized conversation message.

Runtime settings are persisted per conversation so that reopening a chat
restores the writer's last choice. The migration must use a stable
conversation foreign key and preserve existing conversations with deterministic
defaults. Existing conversations remain readable if the new settings row is
missing; the service supplies the safe default and can materialize it on the
next explicit update.

### Streaming-ready response model

Define a provider-independent chat event shape that can represent:

- request accepted;
- assistant text delta;
- completed assistant response;
- retryable failure;
- terminal failure;
- cancellation.

The first implementation may return one completed response because the current
provider boundary is primarily non-streaming. React state must consume the
same event reducer so a later streaming provider can add deltas without
rebuilding the chat surface.

## Client design

### Application shell

Refactor `AppShell` into three responsibilities:

- a narrow primary navigation rail for Projects, Chat and Settings;
- a contextual sidebar for conversations or settings sections;
- the main route outlet for the active workspace.

The existing project context panel remains a project feature, but it should
not compete with the chat composer. Chat keeps its message area and composer
as the primary focus; context details can remain available through the
project-scoped navigation and a responsive secondary panel.

### Chat workspace

Create a shared `ChatWorkspace` surface used by Developer Chat and Chapter
Chat. Its functional parts are:

- `ChatConversationSidebar`: list, active selection and new conversation;
- `ChatHeader`: conversation title, scope and active model selector;
- `ChatMessageList`: user, assistant, reserved tool and error states;
- `ChatRuntimeMenu`: assistant, provider, model and quality controls;
- `ChatComposer`: bottom-pinned draft editor and send/stop/retry actions;
- `ChatErrorBanner`: safe error message with retry when the error is
  retryable.

The runtime menu replaces the current always-visible four-select grid. The
most common action remains choosing the model from the chat header; advanced
runtime choices stay one click away and do not consume the entire composer.

`ChatWorkspace` owns one reducer-driven runtime state. Child components receive
typed values and callbacks and do not call Tauri commands directly.

### Settings workspace

Add a settings shell with a Jan-inspired vertical navigation. The first fully
functional section remains `Model Providers`:

- provider catalog with configured and unconfigured states;
- provider details and endpoint metadata;
- secure credential status, edit, test and remove actions;
- model list grouped by provider;
- model profile details for context size, capabilities and routing metadata.

The shell leaves clear boundaries for future Chat, Appearance and Storage
sections. Those sections are not added as empty sections in this slice.

Existing provider commands and secure-store behavior remain the source of
truth. The visual change must not move keys into React persistence or SQLite.

## Data flow

1. A chat route requests the typed runtime snapshot for its project or
   chapter scope.
2. The backend returns the conversation list, active conversation, messages,
   validated runtime settings and available model descriptors.
3. The client reducer renders the snapshot and keeps the current draft local
   until the user sends it.
4. A runtime choice updates the reducer immediately and persists through one
   typed runtime update command. If persistence fails, the reducer restores
   the last confirmed value and shows an inline error.
5. Send uses one typed chat request. The backend validates scope and model,
   compiles transient context, routes the provider call and persists messages
   through the repository boundary.
6. A successful result replaces the pending state with the assistant message.
   A retryable result keeps the draft or failed message available and never
   appends a duplicate user turn.
7. Explicit proposal or promotion commands remain the only path to canonical
   memory and manuscript changes.

## Error and persistence rules

- Use existing serializable error codes and add only narrowly scoped codes if
  the runtime requires them.
- Map errors to user-facing messages without exposing SQL, prompts,
  credentials or Rust traces.
- Preserve drafts across provider failure, route refresh and retry.
- Treat missing runtime settings as safe defaults.
- Reject archived project/chapter scopes before provider execution.
- Keep credential ids and provider metadata separate from secret values.
- Keep all SQL in repositories and all policy in Rust services.

## Testing and verification

Rust tests must cover runtime settings defaults and migration behavior,
provider/model validation, shared send lifecycle, retry deduplication,
archived-scope rejection, typed error mapping and event reducer inputs.

React tests must cover the shell navigation, settings navigation, provider
catalog states, model selector, runtime menu, draft preservation, retry
behavior, empty/loading/error states and responsive layout assumptions.

After implementation, run the repository-required checks:

```bash
npm run typecheck
npm run lint
npm test -- --run
npm run build
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The generated `src-tauri/gen/`, `src-tauri/icons/`, target directories,
SQLite runtime data, credentials and `node_modules` remain outside the commit.

## Risks and decisions

- A full Jan clone would pull in local inference, Hub, MCP and agent scope, so
  the design intentionally stops at the reusable chat/settings foundation.
- Persisting runtime settings per conversation adds a migration, but it gives
  the model selector stable behavior across route changes and restarts.
- Streaming is represented at the boundary before it is required by the first
  provider path. This avoids locking the UI to a completed-response API while
  keeping the initial implementation deterministic.
- The project keeps its own provider-independent domain and canonical write
  policy even where Jan offers broader provider and agent features.
