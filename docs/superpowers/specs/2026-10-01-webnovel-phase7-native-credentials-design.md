# Phase 7: Native Secure Credential Storage

## Status

Approved implementation slice on `phase7/implementation`.

## Goal

Replace the Phase 6 platform-secure credential placeholder with target-native
secret storage for Windows and Android, while keeping the provider runtime
independent from operating-system APIs and preserving the explicit
session-only behavior used by headless and unsupported targets.

## Scope

This phase includes:

- a target-specific credential-store adapter boundary in the Rust provider
  module;
- Windows Credential Manager integration through the maintained keyring store
  implementation;
- Android Keystore-backed encrypted SharedPreferences integration through the
  maintained Android keyring store implementation;
- initialization/status reporting that does not expose platform error details
  or credential values;
- duplicate, missing, and delete semantics consistent with the existing
  `CredentialStore` contract;
- automatic application-store selection: native secure storage on Windows and
  Android, ephemeral storage on unsupported/headless targets;
- tests for store status, error normalization, application-store selection,
  and runtime status delegation;
- documentation of native runtime prerequisites and the remaining provider
  metadata limitation.

## Non-goals

This phase does not include:

- model routing, quality modes, orchestration, or provider failover;
- persisting provider descriptors/model profiles in SQLite;
- syncing credentials or provider configuration between devices;
- Linux Secret Service support in the cloud runner;
- API-key export, backup, or plaintext fallback;
- changes to canonical project memory.

## Credential boundary

`CredentialStore` remains the only code allowed to read or write provider
secrets. Native adapters use a fixed application service namespace and the
opaque credential ID as the store username. IDs and values never enter
serialized provider responses, logs, prompts, or SQLite migrations.

The adapter initializes the platform store lazily and reports only a typed
available/unavailable status. Any platform error other than a missing entry is
normalized to `SecureStoreUnavailable`; the underlying error is not returned to
the UI. Unsupported targets keep the typed unavailable implementation and do
not fall back to a file, SQLite, or environment variable.

On Windows the adapter uses Windows Credential Manager. On Android it uses
the Android-native keyring store, which encrypts SharedPreferences data with
the Android Keystore. The Android host must initialize the NDK application
context before creating the store; Tauri's mobile runtime is responsible for
that host integration.

## Application selection

The Tauri application selects the native store on `windows` and `android` and
the ephemeral store everywhere else. The Linux cloud/headless test runtime
therefore remains deterministic and does not require a desktop secret-service
daemon. The selected mode is exposed through the existing typed status
command.

Native credential persistence is independent from provider registry lifetime.
Provider descriptors and model profiles are still process-local until a later
phase adds provider metadata persistence and restore.

## Validation

- Native adapter code is compiled only for its target OS through Cargo target
  dependencies and `cfg` modules.
- Unsupported-target tests prove no plaintext fallback is available.
- Runtime tests verify status delegation and application-store selection.
- Frontend behavior remains covered by the Phase 6 provider settings tests.
- Linux checks run formatting, clippy, unit/integration tests, frontend
  checks, and a Vite smoke start. Windows/Android production builds remain
  dependent on their native SDK/toolchain environments.
