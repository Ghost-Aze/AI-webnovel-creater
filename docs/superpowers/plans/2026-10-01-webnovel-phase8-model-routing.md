# Phase 8 implementation plan: model routing

## 1. Document the boundary

- Add the Phase 8 design and update architecture/decision records.
- Keep work on `phase8/implementation`; do not merge `main`.

## 2. Extend typed model metadata (TDD)

- Add `ModelTier` and attach it to `ModelProfile` with a safe default for
  existing provider definitions.
- Update provider setup types/forms and all Rust fixtures without changing the
  provider wire contract.

## 3. Implement the pure routing policy (TDD)

- Add `ModelTask`, `QualityMode`, `RoutingRequest`, `RouteDecision` and
  `ModelRouter`.
- Filter capabilities/context, honor explicit preferences, score task/tier
  affinity, and use stable tie-breaking.
- Add the typed `no_suitable_model` error without exposing request content.

## 4. Expose the command contract

- Add `model_route` to Rust/Tauri and the TypeScript command client.
- Verify command serialization and routing-tier form payloads.
- Keep `provider_generate` explicit; do not start orchestrator execution.

## 5. Verify and publish

- Run Rust format, clippy, unit/integration tests and locked dependency checks.
- Run frontend format, typecheck, lint, tests and build.
- Run a Vite smoke check, inspect `git diff --check`, commit and push only
  `phase8/implementation`.
