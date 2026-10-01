# Phase 3 Provider Abstraction and Context Compiler Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a provider-independent Rust abstraction and deterministic structured-memory Context Compiler without connecting to a real AI provider.

**Architecture:** `provider` owns typed capability-aware provider contracts, model profiles and an in-memory registry. `context` owns a source boundary over existing Project/Character services, a token-budget compiler and typed context blocks. Tauri exposes only safe typed discovery and preview commands; React receives serde-matching types but gains no provider-specific logic or new screen in this phase.

**Tech Stack:** Rust 2021, serde/serde_json, async-trait, futures-core, `Arc<RwLock<...>>`, existing Tauri command adapters, React/TypeScript typed invoke client, Vitest and Cargo tests.

**Spec:** `docs/superpowers/specs/2026-10-01-webnovel-phase3-provider-context-design.md`

## Global Constraints

- No OpenAI, Anthropic, Gemini, OpenRouter, Ollama, llama.cpp or other real provider adapter.
- No API key, secure credential store, provider credential sync, semantic retrieval, relationship graph, chat/manuscript entity or persistent working memory.
- Provider-specific HTTP/SDK code must remain outside the provider-independent domain and canonical entities.
- Model profiles are runtime metadata and are not stored in SQLite.
- The Context Compiler must never automatically load all Project Memory or chat history; only explicitly selected records and caller-supplied working blocks may enter the result.
- Canonical memory writes remain behind the existing proposal/revision path; context compilation is read-only.
- Raw prompts, context contents, credentials and provider stack traces must not be logged or exposed in safe errors.
- Existing project checks remain required: `npm run format:check`, `npm run typecheck`, `npm run lint`, `npm test -- --run`, `npm run build`, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.

## Review Focus

- Duplicate provider IDs or model IDs must be rejected without replacing an existing adapter; covered by registry tests in Task 2.
- A model whose context window is smaller than its output reserve must fail validation without integer underflow; covered by profile/budget tests in Task 3.
- A selected character from another project must not enter a compiled context; covered by source/compiler integration tests in Task 4.
- A zero budget or empty system instruction must return a safe validation error and never produce an empty successful context; covered by compiler tests in Task 3.
- Unsupported stream/embed operations must return a capability error without invoking a provider or leaking request content; covered by mock-provider tests in Task 2 and command/error tests in Task 5.

---

### Task 1: Provider domain contracts and safe errors

**Files:**
- Create: `src-tauri/src/provider/mod.rs`
- Create: `src-tauri/src/provider/types.rs`
- Create: `src-tauri/src/provider/error.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/error.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `Cargo.lock`
- Test: `src-tauri/src/provider/types.rs` and `src-tauri/src/provider/error.rs`

**Interfaces:**
- `ProviderDescriptor { id: String, display_name: String }`.
- `ProviderCapabilities { streaming: bool, embeddings: bool, tools: bool, vision: bool, structured_output: bool, prompt_caching: bool }`.
- `ModelProfile { provider_id: String, model_id: String, display_name: String, context_window_tokens: u32, default_output_tokens: u32, strengths: Vec<String>, weaknesses: Vec<String>, strategy: Vec<String>, capabilities: ProviderCapabilities }` with `validate() -> ProviderResult<()>`.
- `ModelRef { provider_id: String, model_id: String }`.
- `PromptRole`, `PromptMessage`, `GenerateRequest`, `GenerateResponse`, `EmbedRequest`, `EmbedResponse`, `StreamEvent` and `ProviderStream` serde/runtime types.
- `AIProvider` async trait: `descriptor()`, `capabilities()`, `list_models()`, `generate()`, `stream()`, `embed()`; stream/embed default behavior returns `UnsupportedCapability` when not implemented.
- `ProviderError` variants map to safe `AppError` codes `provider_not_found`, `model_not_found`, `unsupported_capability`, `invalid_provider_request` and `provider_failure`.

- [ ] **Step 1: Write failing provider contract tests**

  Add tests for model profile rejection when ids are blank or `context_window_tokens <= default_output_tokens`, serde round trips for capabilities and model refs, and safe provider-error code mapping that does not include request text.

- [ ] **Step 2: Run focused tests to verify they fail**

  Run: `CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup PATH=/workspace/.cargo/bin:$PATH cargo test provider::`

  Expected: FAIL because the provider module, dependency and error variants do not exist.

- [ ] **Step 3: Implement the typed provider contracts**

  Add `async-trait` and direct `futures-core` dependencies. Keep provider requests owned, make `ProviderStream` a boxed `futures_core::Stream`, validate ids/context/output values in `ModelProfile::validate`, and expose only safe error codes through `AppError`.

- [ ] **Step 4: Run focused tests to verify they pass**

  Run: `CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup PATH=/workspace/.cargo/bin:$PATH cargo test provider::`

  Expected: all provider type, validation and safe-error tests pass.

- [ ] **Step 5: Commit the provider contract slice**

  ```bash
  git add src-tauri/src/provider src-tauri/src/lib.rs src-tauri/src/error.rs src-tauri/Cargo.toml Cargo.lock
  git commit -m "feat: add provider abstraction contracts"
  ```

### Task 2: Provider registry and mock capability behavior

**Files:**
- Create: `src-tauri/src/provider/registry.rs`
- Create: `src-tauri/src/provider/mock.rs`
- Modify: `src-tauri/src/provider/mod.rs`
- Test: `src-tauri/src/provider/registry.rs` and `src-tauri/src/provider/mock.rs`

**Interfaces:**
- `ProviderRegistry::new() -> Self`, `register(Arc<dyn AIProvider>) -> ProviderResult<()>`, `list_providers() -> ProviderResult<Vec<ProviderDescriptor>>`, `list_models(provider_id: Option<&str>) -> ProviderResult<Vec<ModelProfile>>`, and `resolve(provider_id: &str, model_id: &str) -> ProviderResult<ResolvedModel>`.
- `ResolvedModel { provider: Arc<dyn AIProvider>, profile: ModelProfile }`.
- `MockProvider::new(descriptor, profiles) -> Self` records a safe generation call count and returns deterministic `GenerateResponse`; stream/embed return `UnsupportedCapability` unless the test explicitly enables those capabilities.

- [ ] **Step 1: Write failing registry and mock tests**

  Add tests named `rejects_duplicate_provider_id`, `lists_and_resolves_models`, `returns_safe_not_found_errors`, `mock_generate_is_deterministic`, and `mock_unsupported_operations_do_not_execute`.

- [ ] **Step 2: Run focused tests to verify they fail**

  Run: `CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup PATH=/workspace/.cargo/bin:$PATH cargo test provider::`

  Expected: FAIL because registry storage, resolution and mock provider behavior are absent.

- [ ] **Step 3: Implement registry and mock provider**

  Store providers in `Arc<RwLock<BTreeMap<String, Arc<dyn AIProvider>>>>`, reject duplicate provider ids and duplicate model ids within a provider, return sorted deterministic lists, and keep the mock provider behind test-only construction so production runtime has no fake provider.

- [ ] **Step 4: Run focused tests to verify they pass**

  Run: `CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup PATH=/workspace/.cargo/bin:$PATH cargo test provider::`

  Expected: all registry, resolution, deterministic mock and unsupported-capability tests pass.

- [ ] **Step 5: Commit the registry slice**

  ```bash
  git add src-tauri/src/provider
  git commit -m "feat: add provider registry and mock"
  ```

### Task 3: Pure Context Compiler, estimator and budget

**Files:**
- Create: `src-tauri/src/context/mod.rs`
- Create: `src-tauri/src/context/types.rs`
- Create: `src-tauri/src/context/compiler.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/context/compiler.rs` and `src-tauri/src/context/types.rs`

**Interfaces:**
- `ContextTask` enum: `DeveloperChat`, `ArcPlanning`, `ChapterPlanning`, `ScenePlanning`, `Writing`, `ContinuityCheck`, `MemoryExtraction`, `Custom(String)`.
- `ContextBlockKind` enum: `System`, `Project`, `Character`, `CharacterState`, `WorkingMemory`.
- `WorkingMemoryBlock { id: String, label: String, content: String }`.
- `ContextBudget { output_reserve_tokens: Option<u32>, safety_margin_tokens: u32 }`.
- `ContextCompileRequest { project_id: String, task: ContextTask, model: ModelRef, system_instructions: String, character_ids: Vec<String>, include_character_states: bool, working_memory: Vec<WorkingMemoryBlock>, budget: ContextBudget }`.
- `TokenEstimator` trait with `estimate(&self, text: &str) -> u32`; `CharTokenEstimator` implements `ceil(char_count / 4)` and returns zero for empty text.
- `ContextBlock { kind, source_id: Option<String>, content: String, estimated_tokens: u32, priority: u8, truncated: bool }`.
- `ContextOmission { source_id: Option<String>, kind: ContextBlockKind, reason: OmissionReason }` and `CompiledContext { project_id, task, model, blocks, omissions, input_budget_tokens, estimated_input_tokens }`.
- `ContextCompiler<E>::compile(&self, request: ContextCompileRequest, profile: &ModelProfile, source: &dyn ContextSource) -> AppResult<CompiledContext>`; `ContextSource` is declared by the context module and supplies typed Project, Character and CharacterState values.

- [ ] **Step 1: Write failing compiler tests**

  Add tests named `rejects_empty_system_instruction`, `rejects_zero_or_insufficient_budget`, `preserves_priority_while_truncating_lower_blocks`, `reports_omitted_blocks`, `includes_explicit_working_memory_only`, and `uses_model_window_for_input_budget`. Assert exact budget arithmetic, UTF-8-safe truncation, source ids and omission reasons.

- [ ] **Step 2: Run focused tests to verify they fail**

  Run: `CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup PATH=/workspace/.cargo/bin:$PATH cargo test context::`

  Expected: FAIL because context request, block, estimator and compiler types do not exist.

- [ ] **Step 3: Implement the pure compiler**

  Build blocks in this order: system, project, selected characters, selected states, working memory. Compute `context_window_tokens - output_reserve - safety_margin` with checked arithmetic, include complete blocks when they fit, prefix-truncate a block to the remaining character budget when possible, and add every omitted/truncated item to metadata. Do not perform SQL or provider calls in this module.

- [ ] **Step 4: Run focused tests to verify they pass**

  Run: `CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup PATH=/workspace/.cargo/bin:$PATH cargo test context::`

  Expected: all validation, ordering, budget and omission tests pass.

- [ ] **Step 5: Commit the compiler slice**

  ```bash
  git add src-tauri/src/context src-tauri/src/lib.rs
  git commit -m "feat: add deterministic context compiler"
  ```

### Task 4: Service-backed ContextSource and SQLite integration

**Files:**
- Create: `src-tauri/src/context/source.rs`
- Modify: `src-tauri/src/context/mod.rs`
- Test: `src-tauri/tests/context_flow.rs`

**Interfaces:**
- `ServiceContextSource::new(projects: ProjectService, characters: CharacterService) -> Self`.
- `ContextSource::load_project(&self, project_id: &str) -> AppResult<Project>`.
- `ContextSource::load_character(&self, project_id: &str, character_id: &str) -> AppResult<Character>`; it verifies project ownership before returning.
- `ContextSource::load_character_state(&self, character_id: &str) -> AppResult<CharacterState>`; missing physical state returns the existing empty typed state.

- [ ] **Step 1: Write failing SQLite-backed context tests**

  Add `context_flow.rs` tests named `compiles_selected_character_and_state`, `rejects_character_from_another_project`, `does_not_load_unselected_characters`, and `does_not_persist_or_mutate_memory`. Create projects/characters through existing services, compile a request, and assert only selected structured records appear.

- [ ] **Step 2: Run focused integration tests to verify they fail**

  Run: `CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup PATH=/workspace/.cargo/bin:$PATH cargo test --test context_flow -- --nocapture`

  Expected: FAIL because the service-backed source and context module registration do not exist.

- [ ] **Step 3: Implement `ServiceContextSource`**

  Delegate to existing services, compare each Character's `project_id` with the request project, return `AppError::NotFound` for a mismatch, and preserve the empty-state behavior already used by `CharacterService`.

- [ ] **Step 4: Run focused integration tests to verify they pass**

  Run: `CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup PATH=/workspace/.cargo/bin:$PATH cargo test --test context_flow -- --nocapture`

  Expected: all selected-memory, cross-project, omission and read-only assertions pass.

- [ ] **Step 5: Commit the source integration slice**

  ```bash
  git add src-tauri/src/context src-tauri/tests/context_flow.rs
  git commit -m "feat: connect context compiler to structured memory"
  ```

### Task 5: Tauri boundary and typed frontend contracts

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `apps/client/src/types/provider.ts`
- Create: `apps/client/src/types/context.ts`
- Modify: `apps/client/src/lib/commands.ts`
- Modify: `apps/client/src/lib/command-error.ts`
- Test: `src-tauri/src/commands.rs`, `apps/client/src/lib/commands.test.ts`, `apps/client/src/lib/command-error.test.ts`

**Interfaces:**
- Rust helpers `list_providers(registry: &ProviderRegistry)`, `list_models(registry: &ProviderRegistry, provider_id: Option<String>)`, and `compile_context(registry: &ProviderRegistry, compiler: &ContextCompiler, source: &ServiceContextSource, request: ContextCompileRequest) -> AppResult<CompiledContext>`.
- Tauri commands `provider_list`, `model_list`, and `context_compile` use snake_case argument names and safe `AppError` serialization.
- TypeScript exports `ProviderDescriptor`, `ProviderCapabilities`, `ModelProfile`, `ModelRef`, `ContextCompileRequest`, `ContextBlock`, `ContextOmission`, `CompiledContext` and `WorkingMemoryBlock` matching Rust serde names.
- Client functions `listProviders()`, `listModels(providerId?: string)`, and `compileContext(input)` call `invoke` through the existing normalizer.

- [ ] **Step 1: Write failing command/client contract tests**

  Assert Rust helper delegation and registration, frontend payloads `{ provider_id }` and `{ request }`, nullable provider filtering, and safe normalization for `provider_not_found`, `model_not_found`, `unsupported_capability`, `invalid_provider_request` and `provider_failure`.

- [ ] **Step 2: Run focused tests to verify they fail**

  Run: `CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup PATH=/workspace/.cargo/bin:$PATH cargo test commands::tests` and `npm test -- --run src/lib/commands.test.ts src/lib/command-error.test.ts`

  Expected: FAIL because provider/context commands, frontend types and error mappings are absent.

- [ ] **Step 3: Implement Rust command adapters and managed state**

  Add `provider_registry`, `context_compiler` and `context_source` to `AppState`; initialize an empty registry and service-backed source in native setup; register all three commands in `generate_handler![]`. `context_compile` resolves the requested model through the registry before compiling and never logs request content.

- [ ] **Step 4: Implement TypeScript types, invoke functions and safe errors**

  Mirror serde field names exactly, keep `source_id` and optional provider filtering nullable, and map provider failures to short user-facing messages without raw details. Do not add a provider UI in this phase.

- [ ] **Step 5: Run focused tests to verify they pass**

  Run: `CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup PATH=/workspace/.cargo/bin:$PATH cargo test commands::tests` and `npm test -- --run src/lib/commands.test.ts src/lib/command-error.test.ts`

  Expected: all command registration, payload and safe-error tests pass.

- [ ] **Step 6: Commit the boundary slice**

  ```bash
  git add src-tauri/src/commands.rs src-tauri/src/lib.rs apps/client/src/lib apps/client/src/types
  git commit -m "feat: expose provider and context commands"
  ```

### Task 6: Documentation, full verification and branch handoff

**Files:**
- Modify: `docs/ARCHITECTURE.md`
- Modify: `docs/ARCHITECTURE_DECISIONS.md`
- Modify: `docs/superpowers/plans/2026-10-01-webnovel-phase3-provider-context.md`
- Test/verification: repository-wide frontend and Rust commands

**Interfaces:**
- Architecture docs describe provider registry/profile boundaries, capability-safe errors, ContextSource selection and budget metadata.
- The completed plan records task checkboxes, commit ids and environment-limited native checks.

- [ ] **Step 1: Update architecture documentation**

  Add the Phase 3 provider/context modules and explicitly keep real adapters, orchestration, semantic retrieval and credentials outside this phase.

- [ ] **Step 2: Run the complete verification matrix**

  Run:

  ```bash
  npm run format:check
  npm run typecheck
  npm run lint
  npm test -- --run
  npm run build
  CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup PATH=/workspace/.cargo/bin:$PATH cargo fmt --all -- --check
  CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup PATH=/workspace/.cargo/bin:$PATH cargo clippy --all-targets -- -D warnings
  CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup PATH=/workspace/.cargo/bin:$PATH cargo test
  ```

  Expected: frontend and headless Rust checks pass. Native `cargo check --features tauri-app` may remain blocked by the documented cloud `glib` prerequisite and must be reported with its exact result.

- [ ] **Step 3: Self-review branch and generated files**

  Run `git diff --check`, inspect `git status --short`, confirm no secrets/node_modules/targets/generated native output are tracked, and verify `main` remains untouched.

- [ ] **Step 4: Commit documentation and verification record**

  ```bash
  git add docs/ARCHITECTURE.md docs/ARCHITECTURE_DECISIONS.md docs/superpowers/plans/2026-10-01-webnovel-phase3-provider-context.md
  git commit -m "docs: finalize provider context phase"
  ```

- [ ] **Step 5: Push the implementation branch**

  ```bash
  git push -u origin phase3/implementation
  ```

  Keep the branch separate from `main`; do not merge without a new explicit integration request.
