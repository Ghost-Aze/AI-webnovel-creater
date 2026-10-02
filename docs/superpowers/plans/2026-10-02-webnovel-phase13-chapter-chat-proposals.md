# Phase 13 implementation plan

## 1. Chapter Chat boundary

- Add chapter ownership to conversation persistence and typed contracts.
- Reuse the provider-independent orchestrator with bounded chat and manuscript
  working memory.
- Expose a `chapter_chat_send` command without canonical manuscript writes.

## 2. Manuscript proposals

- Add a proposal table and typed repository/service lifecycle.
- Create draft proposals from user-selected AI/chat output.
- Promote or reject explicitly with optimistic-concurrency checks.

## 3. UI and verification

- Add Chapter Chat UI to the chapter editor route.
- Add draft proposal list and promote/reject controls.
- Run Rust/frontend tests, lint, formatting, build and smoke checks, then push
  only `phase13/implementation`.
