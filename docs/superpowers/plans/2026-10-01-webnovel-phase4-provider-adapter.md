# WEBNOVEL AI STUDIO Phase 4 Provider Adapter Implementation Plan

> **Execution note:** Follow the repository's TDD and verification workflow.
> Keep this phase limited to the provider adapter and typed generate boundary.

## Goal

Connect the Phase 3 `AIProvider` contract to an OpenAI-compatible HTTP adapter
with deterministic fake transport tests, without adding credential persistence,
orchestration, or UI.

## Task 1: Establish phase branch and contracts

- Create `phase4/implementation` from the pushed Phase 3 commit.
- Add the Phase 4 design and this plan.
- Add `HttpTransport`, request/response transport types and non-serializable
  secret handling in `src-tauri/src/provider/http.rs`.
- Add `ProviderError` variants only where the safe public mapping requires it.
- Write validation and secret-redaction tests before implementation.

## Task 2: Implement the OpenAI-compatible JSON adapter

- Add `OpenAiCompatibleProvider` with descriptor, profiles, normalized base URL,
  in-memory bearer token and injected transport.
- Serialize `PromptMessage` roles and generation options to chat completion JSON.
- Parse the first choice and usage into the existing `GenerateResponse`.
- Map HTTP and malformed response failures to `ProviderFailure`.
- Use fake transport tests for request shape, auth header and safe failures.

## Task 3: Implement streaming SSE parsing

- Add the production Reqwest transport using the existing async provider
  boundary.
- Parse chunk-split SSE lines and `data: [DONE]` into `StreamEvent` values.
- Keep unsupported/no-content events out of the output stream.
- Add fake chunk-stream RED/GREEN tests for split lines, done events and bad
  JSON.

## Task 4: Connect the typed command boundary

- Add a non-Tauri `generate` command helper that resolves the model through the
  registry and awaits `AIProvider::generate`.
- Add the async `provider_generate` Tauri command and register it in `run()`.
- Add frontend `GenerateRequest`, `GenerateResponse` types and a typed
  `generateProvider` invoke wrapper.
- Test snake_case payloads and safe error mapping.

## Task 5: Documentation and verification

- Update architecture decisions and architecture boundary docs.
- Keep the registry empty at startup because secure credential setup is later.
- Run frontend format/typecheck/lint/test/build.
- Run Rust fmt/clippy/test and the native feature check, recording any cloud
  prerequisite failure.
- Perform a read-only review, fix Critical/Important findings once, rerun the
  full verification matrix, commit and push `phase4/implementation`.

## Non-goals

- No API key storage or sync.
- No provider settings screen.
- No orchestration, model routing, embeddings, tools, semantic retrieval,
  manuscript UI or continuity checks.
