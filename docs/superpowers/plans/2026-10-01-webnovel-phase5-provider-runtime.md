# WEBNOVEL AI STUDIO Phase 5 Provider Runtime Implementation Plan

> **Execution note:** Keep secrets ephemeral and keep platform-specific secure
> storage out of this phase. Follow TDD and run required checks after each task.

## Goal

Add a typed provider configure/remove lifecycle that stores credentials only in
process memory and shares the configured registry with Phase 4 commands.

## Task 1: Branch and contracts

- Create `phase5/implementation` from the pushed Phase 4 commit.
- Add this design and plan.
- Add `CredentialStore`, `SecretValue`, `EphemeralCredentialStore`,
  `ProviderConfigureInput` and `ProviderConfigureResult`.
- Write redaction and validation tests before implementation.

## Task 2: Registry/runtime lifecycle

- Add `ProviderRegistry::unregister`.
- Implement `ProviderRuntime::configure` using the credential store and
  `OpenAiCompatibleProvider`.
- Roll back stored credentials on adapter or registry failure.
- Implement remove and unknown-id behavior with safe provider errors.
- Add lifecycle, duplicate, rollback and restart-equivalent tests.

## Task 3: Tauri and TypeScript command boundary

- Add `provider_configure` and `provider_remove` command helpers and Tauri
  commands with snake_case argument naming.
- Share the runtime registry with existing provider list/model/generate commands.
- Add frontend typed request/result contracts and invoke wrappers.
- Test exact payloads and safe error normalization.

## Task 4: Documentation and verification

- Update architecture and decisions documents with the ephemeral runtime
  boundary and explicit platform-store deferral.
- Run frontend format/typecheck/lint/test/build.
- Run Rust fmt/clippy/test and native feature check.
- Perform one read-only review/fix pass, then commit and push
  `phase5/implementation` without merging `main`.

## Non-goals

- No persistent credential backend, provider settings UI, model routing,
  orchestration, retries, sync or new provider protocol.
