# Phase 14 implementation plan: local user profile and global preferences

## Goal

Implement the approved Phase 14 design in small, testable slices. Add durable
local user profile and global writing preferences without introducing auth,
sync, project overrides or a settings UI.

## Constraints

- Work on `phase13/implementation`; do not merge `main`.
- Keep SQL behind the Rust repository boundary.
- Keep the singleton local identity provider-independent and local-only.
- Preserve optimistic revision safety and safe command errors.
- Run the required frontend and Rust checks after source changes.

## Task 1: Add the idempotent user migration and database wiring

Files:

- Create `migrations/0008_create_user_profile_preferences.sql`.
- Modify `src-tauri/src/db.rs`.
- Add or extend database tests in `src-tauri/src/db.rs` or a focused
  integration test.

Test first:

- Add a migration test that runs `db::in_memory()` twice through the migration
  path and verifies both singleton rows exist with id `local_user`.
- Assert the profile default is `display_name = "Writer"` and
  `preferred_language = "en"`.
- Assert the preference defaults are `third_person`, `limited`, `2000`, `600`,
  `40`, `standard`, `balanced`, and `avoid_repetition = true`.
- Assert the migration marker is not duplicated when migrations are rerun.

Implementation:

- Create `user_profile` and `user_preferences` tables with the exact columns,
  singleton primary key and checks from the approved spec.
- Insert one default row for each table using `INSERT ... SELECT ... WHERE NOT
  EXISTS` or an equivalent idempotent statement.
- Append migration `0008_create_user_profile_preferences.sql` to the ordered
  `MIGRATIONS` list.
- Keep SQLite booleans represented as `0`/`1` only inside the repository.

Verification: `cargo test db::` and the focused integration test pass, with no
duplicate rows after a repeated migration run.

## Task 2: Add typed user domain, repository and service boundaries

Files:

- Create `src-tauri/src/domain/user.rs`.
- Create `src-tauri/src/users/mod.rs`.
- Create `src-tauri/src/users/repository.rs`.
- Create `src-tauri/src/users/service.rs`.
- Modify `src-tauri/src/domain/mod.rs` and `src-tauri/src/lib.rs`.

Interfaces:

- `UserProfile { id, display_name, preferred_language, created_at, updated_at, revision }`.
- `UpdateUserProfileInput { display_name, preferred_language }`.
- `UserPreferences { id, preferred_narrator, preferred_pov, chapter_length, scene_length, dialogue_density, prose_level, pacing, avoid_repetition, created_at, updated_at, revision }`.
- `UpdateUserPreferencesInput { preferred_narrator, preferred_pov, chapter_length, scene_length, dialogue_density, prose_level, pacing, avoid_repetition }`.
- `UserRepository::new(connection: SharedConnection) -> Self`.
- `UserRepository::get_profile() -> AppResult<UserProfile>`.
- `UserRepository::update_profile(input: &UpdateUserProfileInput, expected_revision: u64, updated_at: &str) -> AppResult<UserProfile>`.
- `UserRepository::get_preferences() -> AppResult<UserPreferences>`.
- `UserRepository::update_preferences(input: &UpdateUserPreferencesInput, expected_revision: u64, updated_at: &str) -> AppResult<UserPreferences>`.
- `UserService::new(repository: UserRepository) -> Self`.
- `UserService::get_profile() -> AppResult<UserProfile>`.
- `UserService::update_profile(input: UpdateUserProfileInput, expected_revision: u64) -> AppResult<UserProfile>`.
- `UserService::get_preferences() -> AppResult<UserPreferences>`.
- `UserService::update_preferences(input: UpdateUserPreferencesInput, expected_revision: u64) -> AppResult<UserPreferences>`.

Test first:

- Domain tests reject blank display names and blank text preference fields,
  reject zero lengths, reject dialogue density outside 0..=100, trim text and
  preserve boolean values.
- Service tests read the deterministic migration defaults, update each record
  to revision 2, and reject a stale expected revision with `AppError::Conflict`
  without changing the stored row.
- Repository tests verify `avoid_repetition` maps `1` to `true` and `0` to
  `false` in both directions.

Implementation:

- Keep all normalization in the domain/service boundary, before SQL writes.
- Use one transaction per update and update by `id = 'local_user' AND revision
  = expected_revision`.
- Return `AppError::Conflict` when the guarded update affects no row.
- Map malformed stored values to the existing safe storage/status errors; do
  not expose SQL text.

Verification: `cargo test domain::user users::` passes.

## Task 3: Expose typed Tauri command adapters and application state

Files:

- Modify `src-tauri/src/commands.rs`.
- Modify `src-tauri/src/lib.rs`.
- Extend `src-tauri/src/commands.rs` command tests.

Interfaces:

- `get_user_profile(service: &UserService) -> AppResult<UserProfile>`.
- `update_user_profile(service: &UserService, input: UpdateUserProfileInput, expected_revision: u64) -> AppResult<UserProfile>`.
- `get_user_preferences(service: &UserService) -> AppResult<UserPreferences>`.
- `update_user_preferences(service: &UserService, input: UpdateUserPreferencesInput, expected_revision: u64) -> AppResult<UserPreferences>`.

Implementation:

- Add `user_service: UserService` to `AppState`.
- Construct it from the startup SQLite connection in `run()`.
- Register `user_profile_get`, `user_profile_update`,
  `user_preferences_get` and `user_preferences_update` in
  `tauri::generate_handler![]`.
- Keep command logging on operation name and safe error code only.

Test first:

- Assert helper delegation returns the profile/preferences types.
- Assert command definitions use `expected_revision` and snake_case payload
  names.
- Assert stale updates map to the existing `conflict` error without leaking
  SQL or internal details.

Verification: `cargo test commands::` passes.

## Task 4: Add TypeScript user contracts and command wrappers

Files:

- Create `apps/client/src/types/user.ts`.
- Modify `apps/client/src/lib/commands.ts`.
- Modify `apps/client/src/lib/commands.test.ts`.

Interfaces:

- Add TypeScript `UserProfile`, `UpdateUserProfileInput`, `UserPreferences`
  and `UpdateUserPreferencesInput` with snake_case fields matching serde.
- Add `getUserProfile()` and `getUserPreferences()`.
- Add `updateUserProfile(input, expectedRevision)` sending
  `{ input, expected_revision }` to `user_profile_update`.
- Add `updateUserPreferences(input, expectedRevision)` sending
  `{ input, expected_revision }` to `user_preferences_update`.

Test first:

- Assert each wrapper invokes the expected command name.
- Assert update payloads use `expected_revision` and preserve boolean values.
- Assert normalized backend errors continue to use the existing safe error
  normalizer.

Implementation:

- Keep these wrappers UI-independent; do not add a settings screen in Phase
  14.
- Export the types through the existing client type organization if required
  by current imports.

Verification: `npm test -- --run`, `npm run typecheck` and `npm run lint` pass.

## Task 5: Update durable project documentation

Files:

- Modify `docs/DEVELOPMENT_CONTEXT.md` to record Phase 14 completion and the
  local-only boundary.
- Modify `docs/ARCHITECTURE.md` with the user service, migration and typed
  command boundary.
- Modify `docs/ARCHITECTURE_DECISIONS.md` with the no-auth/no-sync decision for
  this phase.
- Update `README.md` only if its phase summary needs the new foundation.

Test/verification:

- Search documentation for contradictory claims that Phase 14 user data is
  cloud-synced or authenticated.
- Keep the deferred list explicit: account auth, sync, project overrides and
  settings UI remain future work.

## Task 6: Full verification and native smoke check

Run from the repository root:

```text
npm run typecheck
npm run lint
npm test -- --run
npm run build
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Start `npm run tauri:dev` with the Rust toolchain on `PATH`, verify the Vite
endpoint returns HTTP 200, and confirm the native process starts. Do not add
SQLite runtime data, credentials, `target`, generated Tauri output or
`node_modules` to Git.

## Review focus

- Migration reruns cannot duplicate singleton rows.
- A stale profile or preferences update writes neither partial fields nor a
  new revision.
- Boolean preferences retain their value across the Rust/SQLite boundary.
- The command boundary uses snake_case and never exposes SQL or credentials.
- Phase 14 does not accidentally introduce auth, sync, project overrides or a
  settings UI.
