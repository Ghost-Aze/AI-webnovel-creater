# Repository guidance

Implement only the approved phase. Phase 0 is limited to the Project CRUD lifecycle, SQLite persistence and migrations, typed Tauri commands, the React workspace shell, tests/tooling and platform scaffolding.

Do not add AI provider integrations, prompt libraries, memory extraction, semantic retrieval, continuity AI, novel-data sync or export until a new phase is approved. Keep the domain provider-independent and keep SQL behind the repository boundary.

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

Do not commit API keys, credentials, SQLite runtime data, `node_modules`, Rust targets or Tauri generated build output. Work on `phase0/implementation` or a feature branch; do not merge into `main` from an implementation task.
