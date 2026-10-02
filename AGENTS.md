# Repository guidance

This repository is the durable source of truth for the Webnovel AI Studio
implementation. Read `docs/DEVELOPMENT_CONTEXT.md`, `docs/PRODUCT_SPEC.md`
and the relevant phase design/plan before making changes. Chat history from a
Codex session is not assumed to be available in a local session.

Phase 13 is the currently approved and completed implementation scope on
`phase13/implementation`. It includes Chapter Chat, chapter-scoped
conversations, explicit manuscript proposals and revision-safe promotion. Do
not begin Phase 14 or expand the product scope without explicit user approval.

Preserve the provider-independent boundary, structured SQLite data and
explicit user-controlled canonical writes. Do not add automatic memory
extraction, semantic retrieval, continuity AI, novel-data sync, collaboration
or export unless a later phase is approved. Keep SQL behind repository
boundaries and keep business logic out of React components.

Required checks for source changes:

```bash
npm run typecheck
npm run lint
npm test -- --run
npm run build
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Do not commit API keys, credentials, SQLite runtime data, `node_modules`, Rust targets or Tauri generated build output. Work on the approved phase branch or a feature branch; do not merge into `main` from an implementation task.
