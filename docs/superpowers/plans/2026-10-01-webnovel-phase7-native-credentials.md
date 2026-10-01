# Phase 7 implementation plan: native secure credentials

## 1. Document the boundary

- Add the Phase 7 design and record the target-selection and error-normalizing
  decisions in the architecture documentation.
- Keep all work on `phase7/implementation`; never merge `main`.

## 2. Add target-native adapters (TDD)

- Add target-specific Cargo dependencies for keyring-core, Windows Credential
  Manager, and Android Keystore-backed storage.
- Implement a small native adapter module behind `CredentialStore`.
- Keep unsupported targets typed and unavailable; never use plaintext fallback.
- Normalize missing credentials and platform failures to the existing domain
  errors without leaking provider values or platform error text.

## 3. Wire runtime selection and status (TDD)

- Delegate `CredentialStoreStatus` to the selected store's real availability.
- Select native secure storage for Windows/Android Tauri builds and ephemeral
  storage for headless/unsupported builds.
- Add unit tests for status delegation, selection, duplicate handling, and
  normalized failures.

## 4. Verify the full slice

- Run Rust format, clippy, unit/integration tests, and dependency resolution.
- Run frontend format, typecheck, lint, tests, and build to catch contract
  regressions.
- Run `git diff --check`, inspect the branch, commit the slice, and push
  `phase7/implementation` without merging `main`.
- Report native build limitations separately when Windows/Android SDKs are not
  present in the cloud environment.
