# Chat Runtime Controls Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the Developer Chat and Chapter Chat runtime controls functional by wiring assistant presets, provider/model selection and quality settings to the existing typed orchestration boundary.

**Architecture:** Add a provider-independent React control component that loads model profiles through `model_list`, exposes a small set of chat assistant presets, and emits a typed `ModelRef` plus `QualityMode`. Each chat panel keeps the selected runtime options local to the active chat view and sends them as the existing `preferred_model`, `quality` and `system_instructions` fields. No provider SDK, credential or SQL logic enters the client.

**Tech Stack:** React, TypeScript, Vitest, existing Tauri typed commands, Rust model routing/orchestration.

**Spec:** Existing Phase 8 model routing, Phase 10 Developer Chat and Phase 13 Chapter Chat designs; this is a bounded UI/runtime integration on `phase13/implementation`.

## Global Constraints

- Preserve the provider-independent boundary and explicit canonical writes.
- Do not add streaming, automatic memory extraction, semantic retrieval, sync or new provider adapters in this slice.
- Keep API keys out of the chat state and UI controls.
- Run the repository-required frontend and Rust checks after source changes.

## Review Focus

- No configured models: chat remains usable through automatic routing and links to provider setup.
- Stale or unavailable selected model: the request must surface the existing typed provider error and never mutate canonical data.
- Switching assistant or quality mode: the next request must carry the chosen instructions and routing preference.
- Archived chats: controls and sending remain disabled as today.
- Chapter Chat proposal behavior: runtime controls must not bypass explicit proposal creation or promotion.

### Task 1: Shared chat runtime controls

**Files:**
- Create: `apps/client/src/features/chat/ChatRuntimeControls.tsx`
- Create: `apps/client/src/test/ChatRuntimeControls.test.tsx`
- Modify: `apps/client/src/styles.css`

**Interfaces:**
- Consumes `listModels`, `ModelProfile`, `ModelRef`, `QualityMode`.
- Produces selected assistant instructions, quality and preferred model through callbacks.

- [x] Write tests for assistant selection, provider/model selection, quality selection and the empty model state.
- [x] Run the focused test and observe the expected failure before implementation.
- [x] Implement the shared controls with loading/error states and a provider setup link.
- [x] Run the focused test and the existing frontend suite.

### Task 2: Developer Chat integration

**Files:**
- Modify: `apps/client/src/features/chat/DeveloperChatPanel.tsx`
- Modify: `apps/client/src/test/DeveloperChatPanel.test.tsx`

**Interfaces:**
- Uses `ChatRuntimeControls` and passes its selected `ModelRef | null`, `QualityMode` and assistant instructions into `DeveloperChatSendRequest`.

- [x] Add a failing test proving the selected model and assistant instructions are sent.
- [x] Implement the panel integration without changing durable conversation semantics.
- [x] Run focused and full frontend tests.

### Task 3: Chapter Chat integration

**Files:**
- Modify: `apps/client/src/features/manuscripts/ChapterChatPanel.tsx`
- Modify: `apps/client/src/test/ChapterChatPanel.test.tsx`

**Interfaces:**
- Uses the same controls and keeps proposal creation/promotion unchanged.

- [x] Add a failing test proving chapter chat sends the selected runtime options.
- [x] Implement the integration and preserve archived/read-only behavior.
- [x] Run focused and full frontend tests.

### Task 4: Documentation and required verification

**Files:**
- Modify: `docs/ARCHITECTURE.md`
- Modify: `docs/DEVELOPMENT_CONTEXT.md`

- [x] Document the chat runtime control boundary and automatic routing fallback.
- [x] Run typecheck, lint, frontend tests, build, cargo fmt, clippy, cargo test and `git diff --check`.
- [x] Report exact results without merging `main` or starting another phase.
