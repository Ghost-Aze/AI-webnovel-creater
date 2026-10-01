# Phase 6 implementation plan: provider setup

## 1. Document the boundary

- Add the provider setup design and update architecture/decision records.
- Keep the phase on `phase6/implementation`; do not merge `main`.

## 2. Add the credential factory seam (TDD)

- Add the explicit credential-store kind and factory contract in Rust.
- Cover ephemeral selection and the typed platform-unavailable result.
- Keep secrets non-serializable and out of diagnostics.

## 3. Build the provider settings UI (TDD)

- Add the `/settings/providers` route and sidebar navigation.
- Add typed form state, validation, configure/remove actions, loading and
  normalized error states.
- Clear the credential field after a successful configure and never render it
  from provider list data.

## 4. Verify the full slice

- Run frontend format, typecheck, lint, tests and build.
- Run Rust formatting, clippy and tests when the toolchain is available.
- Run `git diff --check`, inspect the final branch, and push
  `phase6/implementation` without merging `main`.
