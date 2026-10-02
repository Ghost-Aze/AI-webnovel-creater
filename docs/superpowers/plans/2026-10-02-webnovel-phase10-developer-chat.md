# Phase 10 implementation plan: Developer Chat

## 1. Document the boundary

- Add the Phase 10 design and architecture decisions.
- Keep work on `phase10/implementation`; do not merge `main`.

## 2. Persist conversations

- Add the ordered conversation/message migration and typed domain models.
- Implement project-scoped repository/service operations with archive checks.

## 3. Connect Developer Chat to orchestration

- Persist user turns, pass bounded recent history to the Phase 9
  orchestrator, and persist the final assistant response.
- Keep failures safe and leave no partial assistant turn.

## 4. Add proposal-backed memory tools

- Define a typed character-update tool request.
- Delegate to `ProposalService` as an AI draft; never mutate canonical memory
  in the tool boundary.

## 5. Verify and publish

- Run migration, Rust, frontend and Vite smoke checks.
- Commit and push only `phase10/implementation`.
