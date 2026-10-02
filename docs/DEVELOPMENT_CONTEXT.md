# Development context

This file preserves the project decisions that would otherwise live only in a
Codex conversation. A local developer or a new assistant should read it before
continuing work.

## Product boundary

Webnovel AI Studio is a provider-independent, local-first workspace for
planning and writing long-form fiction. Canonical project state belongs to the
application database, not to an AI provider conversation. AI output must remain
reviewable and must not silently mutate canonical memory or manuscript text.

The product specification is in `docs/PRODUCT_SPEC.md`. Phase design and plan
documents are under `docs/superpowers/`.

## Implementation state

- Current approved branch: `phase13/implementation`
- Latest Phase 13 commit: `9c8cb5f feat: add chapter chat manuscript proposals`
- Phase 12 base commit: `12fa759`
- Phase 11 base commit: `76552e6`
- `main` has not been merged into by implementation work.
- Phase 0 through Phase 14 are implemented slices. Phase 14 adds the local
  user profile and global writing-preferences foundation; it does not add
  authentication, sync or a settings UI.

## Phase 13 decisions

- `Conversation.chapter_id` is optional in storage and required for
  `chapter_chat`; the service verifies that the chapter belongs to the same
  project.
- `chapter_chat_send` reuses the provider-independent orchestrator. It passes
  bounded conversation history and the current manuscript as transient working
  memory. It never writes the canonical manuscript automatically.
- `ManuscriptProposal` stores a complete proposed body with `draft`, `accepted`
  or `rejected` status and a `base_revision`.
- Promotion is explicit. One SQLite transaction checks the chapter/project and
  manuscript revision, writes the next `manuscript_revisions` snapshot and
  accepts the proposal. A stale base returns `conflict`.
- The React editor exposes Chapter Chat plus review controls. Proposal content
  is currently a complete plain-text body; line-level diff/merge is deferred.

## Phase 14 decisions

- `user_profile` and `user_preferences` are singleton local SQLite records with
  the stable `local_user` key and deterministic migration defaults.
- User profile and writing-preference updates normalize typed inputs and use
  optimistic revisions inside one SQLite transaction. Stale writes return
  `conflict` without partial changes.
- `UserService` owns the local identity boundary. Tauri and TypeScript expose
  typed profile/preferences commands while SQL, SQLite booleans and storage
  errors remain inside Rust.
- Authentication, cloud sync, multi-user access, project-level overrides,
  provider settings and a visual settings screen remain deferred.

## Verification baseline

The current Phase 14 baseline is:

- Rust: 95 unit tests plus 25 integration tests pass.
- Rust `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`
  pass.
- Frontend: 63 Vitest tests across 16 files pass.
- Frontend typecheck, ESLint and production build pass. The repository-wide
  Prettier check still reports existing formatting drift across the client and
  is not part of the required source verification gate.
- Windows native smoke test starts `webnovel-ai-studio` successfully and the
  Vite endpoint returns HTTP 200.
- Native `cargo check --features tauri-app` cannot complete in the cloud Linux
  environment because GTK/GObject/GIO/GDK development packages are absent.
  Windows native and Android builds must be verified on machines with their
  respective SDK/toolchain prerequisites.

## Local continuation

Clone the approved branch rather than copying the cloud filesystem:

```powershell
git clone --branch phase13/implementation --single-branch `
  https://github.com/Ghost-Aze/AI-webnovel-creater.git `
  AI-webnovel-creater
cd AI-webnovel-creater
npm.cmd ci
```

Use `npm.cmd` in PowerShell when execution policy blocks `npm.ps1`. Run
`npm.cmd run dev` for the Vite client or `npm.cmd run tauri:dev` for the native
desktop shell after installing Rust, Visual Studio C++ Build Tools, Windows SDK
and WebView2. API keys and platform credentials are intentionally not copied by
Git; configure them locally through the provider settings boundary.

## Continuation protocol

1. Confirm the branch and read this file plus the relevant phase documents.
2. Keep each change inside the approved phase unless the user explicitly
   approves a new phase.
3. Run the required checks after source changes and report results.
4. Preserve local changes; do not use destructive resets or merge into `main`.
5. Before a later phase, ask the user for approval and write a new
   design/plan. Keep Phase 14 local-only until a later scope is approved.
