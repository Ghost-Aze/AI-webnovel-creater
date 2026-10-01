# WEBNOVEL AI STUDIO Phase 4 Provider Adapter Implementation Plan

> **Execution note:** Follow the repository's TDD and verification workflow.
> Keep this phase limited to the provider adapter and typed generate boundary.

## Goal

Connect the Phase 3 `AIProvider` contract to an OpenAI-compatible HTTP adapter
with deterministic fake transport tests, without adding credential persistence,
orchestration, or UI.

## Task 1: Establish phase branch and contracts

- [x] Create `phase4/implementation` from the pushed Phase 3 commit.
- [x] Add the Phase 4 design and this plan.
- [x] Add `HttpTransport`, request/response transport types and non-serializable
  secret handling in `src-tauri/src/provider/http.rs`.
- [x] Add `ProviderError` variants only where the safe public mapping requires it.
- [x] Write validation and secret-redaction tests before implementation.

## Task 2: Implement the OpenAI-compatible JSON adapter

- [x] Add `OpenAiCompatibleProvider` with descriptor, profiles, normalized base URL,
  in-memory bearer token and injected transport.
- [x] Serialize `PromptMessage` roles and generation options to chat completion JSON.
- [x] Parse the first choice and usage into the existing `GenerateResponse`.
- [x] Map HTTP and malformed response failures to `ProviderFailure`.
- [x] Use fake transport tests for request shape, auth header and safe failures.

## Task 3: Implement streaming SSE parsing

- [x] Add the production Reqwest transport using the existing async provider
  boundary.
- [x] Parse chunk-split SSE lines and `data: [DONE]` into `StreamEvent` values.
- [x] Keep unsupported/no-content events out of the output stream.
- [x] Add fake chunk-stream RED/GREEN tests for split lines, done events and bad
  JSON.

## Task 4: Connect the typed command boundary

- [x] Add a non-Tauri `generate` command helper that resolves the model through the
  registry and awaits `AIProvider::generate`.
- [x] Add the async `provider_generate` Tauri command and register it in `run()`.
- [x] Add frontend `GenerateRequest`, `GenerateResponse` types and a typed
  `generateProvider` invoke wrapper.
- [x] Test snake_case payloads and safe error mapping.

## Task 5: Documentation and verification

- [x] Update architecture decisions and architecture boundary docs.
- [x] Keep the registry empty at startup because secure credential setup is later.
- [x] Run frontend format/typecheck/lint/test/build.
- [x] Run Rust fmt/clippy/test and the native feature check, recording any cloud
  prerequisite failure.
- [x] Perform a read-only review, fix Critical/Important findings once, rerun the
  full verification matrix, commit and push `phase4/implementation`.

## Non-goals

- No API key storage or sync.
- No provider settings screen.
- No orchestration, model routing, embeddings, tools, semantic retrieval,
  manuscript UI or continuity checks.

## Completion record

- Branch: `phase4/implementation`
- Implementation commits: pending final commit after verification
- Frontend: format, typecheck, lint, build passed; 25 tests passed.
- Rust: format, Clippy and tests passed; 41 unit tests (including 10 HTTP
  adapter tests) and 21 integration tests passed; 0 doc-test failures.
- Review: Important streaming findings fixed in one pass with byte-level UTF-8
  buffering, blank-line SSE event framing and strict truncated-stream errors.
- Native Tauri feature check remains environment-limited by the cloud image's
  missing `glib-2.0 >= 2.70` prerequisite.
