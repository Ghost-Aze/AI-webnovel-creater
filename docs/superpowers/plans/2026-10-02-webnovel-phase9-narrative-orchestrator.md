# Phase 9 implementation plan: Narrative Orchestrator

## 1. Document the boundary

- Add the Phase 9 design and architecture decision.
- Keep work on `phase9/implementation`; do not merge `main`.

## 2. Define the execution plan

- Add logical-agent steps and deterministic quality-mode presets.
- Extend task metadata only where the orchestrator needs explicit plot/style
  analysis roles.

## 3. Implement sequential execution

- Route each step through `ModelRouter`.
- Compile context through `ContextCompiler` with prior outputs as working
  memory.
- Execute through `AIProvider::generate` and return typed step results.

## 4. Expose the command contract

- Add the Rust/Tauri `orchestrator_run` command.
- Add TypeScript request/result types and a command client function.
- Keep orchestration out of React UI and keep canonical memory unchanged.

## 5. Verify and publish

- Run Rust format, clippy, unit/integration tests and frontend checks.
- Run a Vite smoke check, inspect the diff, commit and push only
  `phase9/implementation`.
