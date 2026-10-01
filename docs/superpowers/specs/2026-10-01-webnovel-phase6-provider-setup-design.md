# Phase 6: Provider Setup and Credential Boundary

## Status

Approved implementation slice on `phase6/implementation`.

## Goal

Give users a typed provider setup surface that can register and remove an
OpenAI-compatible provider through the existing Rust runtime, while keeping
credentials out of canonical SQLite data, React state after submission, and
provider list responses. Establish the platform credential seam required for
Windows Credential Manager and Android Keystore without coupling the UI to an
operating-system API.

## Scope

This phase includes:

- a provider settings route and navigation entry;
- a setup form for one OpenAI-compatible provider and one or more model
  profiles;
- provider discovery and removal in the settings page;
- safe form validation and secret-field clearing after a successful configure;
- a Rust credential-store factory boundary that makes the persistence mode
  explicit and keeps the current session-only store available for headless
  development;
- tests for the form payload, secret clearing, provider removal, and the
  credential backend selection contract;
- architecture and README documentation for the native secure-store handoff.

This phase does not include model routing, orchestration, semantic retrieval,
provider-specific UI, cloud sync, or a claim that the Linux cloud runner is a
production credential backend.

## User flow

1. The user opens **Providers** from the left navigation.
2. The page lists configured provider descriptors and their model IDs. It never
   displays credential values or credential IDs.
3. The user opens **Add provider**, enters a display name, provider ID, base
   URL, credential ID, credential value, and a model profile.
4. The client validates required fields locally and sends the typed
   `provider_configure` command.
5. Rust validates the input, stores the secret through the injected credential
   store, and registers the provider as one lifecycle.
6. On success the form closes, the credential input is cleared, and the list
   refreshes. On failure the error is normalized without showing raw provider
   or storage details.
7. Removing a provider requires an explicit action and refreshes the list only
   after the Rust runtime has removed both the adapter and credential reference.

## Credential boundary

`CredentialStore` remains the only runtime dependency that can read or write a
secret. Phase 6 adds an explicit `CredentialStoreKind` and factory contract:

- `Ephemeral` is used by headless tests and the current cloud runner. It is
  process-only and is cleared on restart.
- `PlatformSecure` is the native integration point. The factory returns a
  typed unavailable result until the Windows Credential Manager and Android
  Keystore adapters are supplied in a platform build. It never silently falls
  back to SQLite or a plaintext file.

Application startup currently selects the explicit session-only kind so the
cloud and headless runtime remain deterministic. A target build may select a
platform adapter only after its Windows Credential Manager or Android Keystore
implementation is available; the selected mode is exposed to diagnostics
without exposing any secret.

## Validation

The setup form rejects blank IDs, names, base URLs, credential values and model
IDs, and rejects non-positive context/output limits. Rust remains the final
authority for provider and model validation. Model profiles are submitted as
typed values and are not serialized into prompts or database migrations.

## Out of scope and follow-up

Model selection/routing, quality modes, multi-provider failover, native secure
storage adapters, and encrypted export are later work. The native adapter must
be implemented and tested per target before the application can advertise
persisted credentials on Windows or Android.
