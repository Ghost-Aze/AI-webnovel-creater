# WEBNOVEL AI STUDIO Phase 5 Provider Runtime Implementation Plan

> **Execution note:** Keep secrets ephemeral and keep platform-specific secure
> storage out of this phase. Follow TDD and run required checks after each task.

## Goal

Add a typed provider configure/remove lifecycle that stores credentials only in
process memory and shares the configured registry with Phase 4 commands.

## Task 1: Branch and contracts

- [x] Create `phase5/implementation` from the pushed Phase 4 commit.
- [x] Add this design and plan.
- [x] Add `CredentialStore`, `SecretValue`, `EphemeralCredentialStore`,
  `ProviderConfigureInput` and `ProviderConfigureResult`.
- [x] Write redaction and validation tests before implementation.

## Task 2: Registry/runtime lifecycle

- [x] Add `ProviderRegistry::unregister`.
- [x] Implement `ProviderRuntime::configure` using the credential store and
  `OpenAiCompatibleProvider`.
- [x] Roll back stored credentials on adapter or registry failure.
- [x] Implement remove and unknown-id behavior with safe provider errors.
- [x] Add lifecycle, duplicate, rollback and restart-equivalent tests.

## Task 3: Tauri and TypeScript command boundary

- [x] Add `provider_configure` and `provider_remove` command helpers and Tauri
  commands with snake_case argument naming.
- [x] Share the runtime registry with existing provider list/model/generate commands.
- [x] Add frontend typed request/result contracts and invoke wrappers.
- [x] Test exact payloads and safe error normalization.

## Task 4: Documentation and verification

- [x] Update architecture and decisions documents with the ephemeral runtime
  boundary and explicit platform-store deferral.
- [x] Run frontend format/typecheck/lint/test/build.
- [x] Run Rust fmt/clippy/test and native feature check.
- [x] Perform one read-only review/fix pass, then commit and push
  `phase5/implementation` without merging `main`.

## Non-goals

- No persistent credential backend, provider settings UI, model routing,
  orchestration, retries, sync or new provider protocol.

## Completion record

- Branch: `phase5/implementation`
- Implementation commit: `d398143` (`feat: add ephemeral provider runtime lifecycle`).
- Frontend: format, typecheck, lint and build passed; 26 tests passed.
- Rust: format, Clippy and tests passed; 50 unit tests, 21 integration tests,
  and 0 doc-test failures.
- Review: credential ownership collision, concurrent lifecycle and deletion
  rollback findings fixed with regression tests.
- Native Tauri feature check remains environment-limited by missing cloud
  `glib-2.0`, `gobject-2.0`, `gio-2.0` and `gdk-3.0` packages.
