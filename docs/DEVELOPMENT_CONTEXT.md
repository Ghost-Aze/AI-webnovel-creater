# Development context

This file preserves the project decisions needed to continue work without
reconstructing the Codex session. Read it with the approved phase documents
before changing the repository.

## Product boundary

Webnovel AI Studio is a provider-independent, local-first workspace for
planning and writing long-form fiction. Canonical project state belongs to
the application database. Chat output remains reviewable and cannot silently
mutate project memory, characters or manuscript text.

The current approved work is the Jan-inspired chat and settings foundation on
`phase13/implementation`. The design and implementation plan are under
`docs/superpowers/`.

## Current implementation state

- Branch: `phase13/implementation`; implementation work has not been merged
  into `main`.
- Migration `0011_conversation_runtime_settings.sql` stores non-secret
  assistant, provider, model, quality and temperature choices per conversation.
- Rust `ChatRuntimeService` owns runtime snapshot loading, setting validation,
  provider/model selection checks, archived-scope guards and chat sends.
- Tauri exposes typed `chat_runtime_load`, `chat_runtime_update` and
  `chat_send` commands. Existing developer/chapter commands remain adapters.
- React uses one reducer-driven runtime for Developer Chat and Chapter Chat.
  Conversations, retry-safe drafts, runtime selection and chapter proposal
  actions remain typed at the command boundary.
- The shared workspace has a conversation sidebar, model selector, runtime
  menu, scrollable message region and bottom composer. Archived scopes are
  read-only.
- `/settings` redirects to `/settings/providers`. The settings shell exposes
  `Model Providers` as its first functional section, with provider discovery,
  model lists, secure-store status, connection test and remove/update flows.
- The application shell now has a compact dark rail, project navigation,
  context panel, responsive mobile navigation and Jan-inspired chat surfaces.

SQL remains behind Rust repositories and provider credentials remain in the
existing secure credential boundary. Runtime settings and messages contain no
API keys.

## Deferred scope

Local model download, Hub, MCP, tools, agents, attachments, semantic
retrieval, automatic memory extraction, cloud sync, collaboration and export
are deferred until a separately approved phase. The Chat event union defines
an event boundary, but the current provider path still returns completed chat
results rather than claiming streaming support.

## Verification baseline

The final verification results for this implementation are recorded in
`docs/CHAT_CONTEXT.md` and the SDD ledger at
`.superpowers/sdd/2026-10-03-jan-style-chat-runtime/progress.md`: frontend
typecheck, lint, 102 Vitest tests and production build pass; Rust formatting,
clippy, 117 unit tests and 39 integration tests pass.

The repository-wide Prettier check still reports pre-existing client format
drift; unrelated files are not mass-formatted. Generated `src-tauri/gen/`,
`src-tauri/icons/`, Rust targets, SQLite runtime data, credentials and
`node_modules` stay outside commits.

## Local continuation

Use `npm.cmd` in PowerShell when execution policy blocks `npm.ps1`:

```powershell
npm.cmd ci
npm.cmd run dev
npm.cmd run tauri:dev
```

Configure provider credentials locally through the provider settings boundary;
never commit them.

## Continuation protocol

1. Confirm the branch and read this file plus the relevant phase documents.
2. Keep changes inside the approved phase unless a new phase is explicitly
   approved and has its own design and plan.
3. Run the required checks after source changes and record the result.
4. Preserve local changes and do not merge implementation work into `main`.
