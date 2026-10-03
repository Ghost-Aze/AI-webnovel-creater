# Development chat context

This is a handoff record for the current implementation. It is not canonical
product data or a transcript of private conversation.

## User goal

Make Developer Chat, Chapter Chat and Settings feel like Jan while keeping a
provider-independent runtime foundation and the project's local-first policy.

## Approved boundaries

- Work on `phase13/implementation`; do not merge into `main`.
- Keep SQLite, routing, context compilation, credentials and canonical write
  policy behind Rust services and repositories.
- Chat output must not silently write memory, characters, canon or manuscript
  content. Chapter revisions require an explicit proposal and promotion flow.
- Defer local inference, Hub, MCP, tools, agents, attachments, semantic
  retrieval, automatic memory extraction, sync, collaboration and export.

## Implemented in this session

- Added conversation runtime settings with deterministic defaults:
  `general-assistant`, automatic provider/model routing, `balanced` quality and
  no temperature override.
- Added `ChatRuntimeService` plus typed Tauri load, update and send commands.
  Provider/model validation and archived project/chapter rejection happen
  before provider execution or message mutation.
- Added TypeScript runtime contracts, commands and a pure reducer. Retry keeps
  the draft and merges messages by conversation sequence so a user turn is not
  duplicated.
- Added the shared chat workspace with conversation creation/selection,
  selected model display, advanced assistant/provider/model/quality controls,
  retry-safe error handling and archived read-only behavior.
- Converted Developer Chat and Chapter Chat to workspace adapters. Chapter
  proposal creation remains outside the shared runtime boundary.
- Added the Jan-style settings shell. `Model Providers` is functional;
  Chat, Appearance and Storage are visible as deferred sections. `/settings`
  redirects to `/settings/providers`.
- Applied the dark responsive application shell with a primary rail,
  project navigation, context panel, mobile navigation and bottom-pinned
  chat composer.

## Verification evidence

- Frontend typecheck, ESLint, 21 Vitest files with 102 passing tests and the
  production build pass.
- Rust formatting and clippy pass. The full Rust suite passes with 117 unit
  tests and 39 integration tests.
- Native `tauri:dev` compiled and launched in the Windows checkout. The
  localhost browser preview rendered the settings shell; browser-only
  `invoke` calls show the expected unavailable-command fallback because they
  are outside the native Tauri runtime.
- The repository-wide Prettier check retains known baseline drift and is not
  used as a source gate for unrelated files.

## Generated-file boundary

Keep `src-tauri/gen/` and `src-tauri/icons/` untracked when they are produced
by local Tauri startup. Do not commit API keys, credentials, SQLite runtime
databases, `node_modules`, Rust targets or generated Tauri build output.
