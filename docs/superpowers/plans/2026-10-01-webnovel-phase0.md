# WEBNOVEL AI STUDIO Phase 0 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** GitHub’daki WEBNOVEL AI STUDIO repository’sini Tauri 2 + React + TypeScript + Rust + SQLite temelli, Windows build’i ve Android scaffold’ı bulunan çalışan bir Phase 0 uygulamasına dönüştürmek.

**Architecture:** React istemcisi yalnızca typed Tauri command client üzerinden Rust application service katmanına erişecek. Rust service, SQLite repository’sini ve ayrı migration dosyalarını kullanacak; domain tipleri provider bağımsız kalacak. GitHub `main` kaynak gerçek olacak, Codex Cloud environment ise cihazlar arası geliştirme çalışma alanı olarak belgelenip CI ile doğrulanacak.

**Tech Stack:** Tauri 2, Rust, React, TypeScript, Vite, `react-router`, rusqlite (bundled SQLite), Vitest, ESLint, Prettier, Cargo fmt/clippy/test, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-10-01-webnovel-phase0-design.md`

## Global Constraints

- Phase 0 yalnızca Project CRUD, temel shell, SQLite persistence, test/tooling, Windows build ve Android scaffold içerir.
- AI provider, prompt, memory extraction, semantic search, continuity AI, cloud novel-data sync ve export özellikleri bu fazda eklenmez.
- UI veritabanına doğrudan erişmez; bütün veri işlemleri typed Tauri command katmanından geçer.
- Project alanları `id`, `name`, `description`, `status`, `created_at`, `updated_at` değerleridir.
- Status değerleri yalnızca `active` ve `archived` olabilir; arşivleme geri döndürülebilir bir durum değişikliğidir.
- SQLite runtime verisi, API key ve yerel credential repository’ye commit edilmez.
- `main` yalnızca doğrulanmış değişiklikleri alır; geliştirme görevleri branch veya izole worktree üzerinde yürütülür.

## Review Focus

- Boş veya yalnızca whitespace olan proje adı kullanıcıya açık doğrulama hatası vermeli; veritabanına yazılmamalıdır. (Task 2)
- Migration ikinci kez çalıştırıldığında tabloyu veya veriyi bozmamalıdır. (Task 2)
- Arşivli proje varsayılan aktif listede görünmemeli ve update işlemi reddedilmelidir. (Task 2)
- Tauri command hatası UI’da ham Rust/SQLite ayrıntısı yerine güvenli bir mesaj göstermelidir. (Task 3)
- Uygulama yeniden açıldığında SQLite’daki projeler aynı alanlarla listelenmelidir. (Task 2 ve Task 3)

---

### Task 1: Workspace ve platform scaffold’ı

**Files:**
- Create: `package.json`
- Create: `package-lock.json`
- Create: `tsconfig.base.json`
- Create: `.gitignore`
- Create: `apps/client/package.json`
- Create: `apps/client/index.html`
- Create: `apps/client/src/main.tsx`
- Create: `apps/client/src/vite-env.d.ts`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/icons/` (Tauri-generated placeholder assets)

**Interfaces:**
- Produces root npm scripts: `dev`, `build`, `test`, `typecheck`, `lint`, `format:check`, `tauri:dev`, `tauri:build`.
- Produces a Tauri application whose frontend dev URL and production asset path are configured for `apps/client`.

- [ ] **Step 1: Create the workspace manifests and ignore rules**

Use npm workspaces with `apps/client` as the only workspace. Ignore `node_modules`, `dist`, `target`, `.env*`, SQLite runtime files, IDE files and Tauri generated build output.

- [ ] **Step 2: Scaffold the React/Vite entry point**

Create the minimal `main.tsx` mount and a typed Vite environment declaration. Keep business logic out of this entry point.

- [ ] **Step 3: Scaffold the Tauri 2 Rust crate**

Configure a desktop-capable Tauri crate with a `main` entry that delegates to `lib::run()`. Keep command registration in `lib.rs` so Android and Windows use the same application bootstrap.

- [ ] **Step 4: Run baseline tooling**

Run `npm install`, `npm run typecheck`, `npm run lint`, `cargo fmt --all -- --check`, and `cargo test --manifest-path src-tauri/Cargo.toml`.

Expected: commands complete successfully with only the scaffold present.

- [ ] **Step 5: Commit the scaffold**

```bash
git add package.json package-lock.json tsconfig.base.json .gitignore apps/client src-tauri
git commit -m "chore: scaffold Tauri React workspace"
```

### Task 2: Domain, SQLite migration and Project application service

**Files:**
- Create: `migrations/0001_create_projects.sql`
- Create: `src-tauri/src/domain/mod.rs`
- Create: `src-tauri/src/domain/project.rs`
- Create: `src-tauri/src/db.rs`
- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/projects/repository.rs`
- Create: `src-tauri/src/projects/service.rs`
- Create: `src-tauri/tests/project_flow.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- `Project { id: String, name: String, description: String, status: ProjectStatus, created_at: String, updated_at: String }`
- `ProjectStatus::{Active, Archived}` serialized as `active` and `archived`.
- `CreateProjectInput { name: String, description: Option<String> }`.
- `UpdateProjectInput { name: String, description: Option<String> }`.
- `ProjectListFilter { include_archived: bool }`.
- `ProjectRepository::{create, get, list, update, archive}`.
- `ProjectService::{create, get, list, update, archive}`.

- [ ] **Step 1: Write failing domain validation tests**

Add tests for whitespace-only names, trimming accepted names, default empty descriptions, valid status serialization, and UTC timestamp shape.

- [ ] **Step 2: Run the focused Rust tests and verify they fail**

Run `cargo test --manifest-path src-tauri/Cargo.toml domain::project`.

Expected: failure because the domain types and validation do not exist.

- [ ] **Step 3: Implement the Project domain types and typed errors**

Implement validation and serde serialization in `domain/project.rs`. Define serializable application errors for validation, not-found, invalid-status and storage failures in `error.rs`.

- [ ] **Step 4: Write the migration and idempotent migration runner**

Create `projects` and migration metadata tables in `migrations/0001_create_projects.sql`. The runner records applied migration names and safely skips an already applied migration.

- [ ] **Step 5: Write repository and service tests**

Use a temporary SQLite database to assert migration idempotency, create/list/get/update/archive, active-only default filtering, archive state persistence and update timestamp changes.

- [ ] **Step 6: Implement the SQLite repository and service**

Keep SQL inside the repository module. The service owns normalization, validation, active/archive policy and timestamp creation; the repository maps rows to `Project`.

- [ ] **Step 7: Run the full Rust test set**

Run `cargo fmt --all`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, and `cargo test --manifest-path src-tauri/Cargo.toml`.

Expected: all domain and integration tests pass with no clippy warnings.

- [ ] **Step 8: Commit the data layer**

```bash
git add migrations src-tauri
git commit -m "feat: add SQLite project domain and persistence"
```

### Task 3: Tauri commands and typed frontend client

**Files:**
- Create: `src-tauri/src/commands.rs`
- Create: `apps/client/src/types/project.ts`
- Create: `apps/client/src/lib/commands.ts`
- Create: `apps/client/src/lib/command-error.ts`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Tauri commands: `project_create`, `project_list`, `project_get`, `project_update`, `project_archive`.
- Frontend functions: `createProject`, `listProjects`, `getProject`, `updateProject`, `archiveProject`.
- Frontend functions return typed `Project` values and throw a normalized `CommandError`.

- [ ] **Step 1: Write command contract tests**

Test that invalid input returns a serializable validation error, missing IDs return a safe not-found error, and successful commands return the exact `Project` shape.

- [ ] **Step 2: Register commands and initialize the database**

Initialize the SQLite path under the Tauri app data directory, run migrations during startup, build the service, and register all five commands.

- [ ] **Step 3: Implement the typed frontend command client**

Wrap `@tauri-apps/api/core` `invoke` calls in one module. Map backend error codes to user-safe messages without exposing raw SQL or stack traces.

- [ ] **Step 4: Verify the command boundary**

Run Rust tests and `npm run typecheck`. Expected: the frontend compiles against the exact command input/output types.

- [ ] **Step 5: Commit the command boundary**

```bash
git add src-tauri apps/client/src/types apps/client/src/lib
git commit -m "feat: expose typed project commands"
```

### Task 4: Application shell, navigation and Project UI

**Files:**
- Create: `apps/client/src/app/App.tsx`
- Create: `apps/client/src/app/routes.tsx`
- Create: `apps/client/src/components/layout/AppShell.tsx`
- Create: `apps/client/src/components/layout/Sidebar.tsx`
- Create: `apps/client/src/components/layout/ContextPanel.tsx`
- Create: `apps/client/src/components/layout/StatusBar.tsx`
- Create: `apps/client/src/features/projects/ProjectsPage.tsx`
- Create: `apps/client/src/features/projects/ProjectWorkspacePage.tsx`
- Create: `apps/client/src/features/projects/NewProjectDialog.tsx`
- Create: `apps/client/src/features/projects/project-view-model.ts`
- Create: `apps/client/src/styles.css`
- Create: `apps/client/src/test/ProjectsPage.test.tsx`
- Modify: `apps/client/src/main.tsx`
- Modify: `apps/client/package.json`

**Interfaces:**
- Routes: `/projects` and `/projects/:projectId`.
- `ProjectsPage` consumes `listProjects` and `createProject`.
- `ProjectWorkspacePage` consumes `getProject`, `updateProject` and `archiveProject`.

- [ ] **Step 1: Write failing UI tests**

Add tests for empty project state, project list rendering, whitespace name rejection, successful creation, safe command error display, and navigation to a project workspace.

- [ ] **Step 2: Implement the shell and routes**

Create the responsive three-region shell: left project navigation, center route content, right context placeholder, and bottom status bar. Use a drawer/bottom navigation breakpoint for narrow screens.

- [ ] **Step 3: Implement the Projects screen and dialog**

Render active projects, an archived filter, loading and retry states, and a modal with name/description validation. On creation, refresh the list and navigate to the workspace.

- [ ] **Step 4: Implement the project workspace**

Show the current project, rename form, archive action and right-side placeholders for Context, Characters, Timeline, Plot Threads, Memory, Continuity and Project Bible.

- [ ] **Step 5: Run frontend verification**

Run `npm test -- --run`, `npm run typecheck`, `npm run lint`, and `npm run build`.

Expected: all UI tests pass and a production frontend bundle is generated.

- [ ] **Step 6: Commit the application UI**

```bash
git add apps/client
git commit -m "feat: add project navigation and workspace shell"
```

### Task 5: Logging, documentation, Cloud setup and CI

**Files:**
- Create: `docs/ARCHITECTURE.md`
- Create: `docs/CLOUD_DEVELOPMENT.md`
- Create: `AGENTS.md`
- Create: `.github/workflows/ci.yml`
- Modify: `README.md`
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- `README.md` documents local setup, supported commands and Phase 0 limits.
- `docs/CLOUD_DEVELOPMENT.md` documents GitHub → Codex Cloud environment setup, publish/reopen flow, branch discipline and mobile continuation.
- CI runs frontend typecheck/lint/test/build and Rust fmt/clippy/test.

- [ ] **Step 1: Add structured Rust logging**

Initialize `tracing` with a Tauri-compatible subscriber. Log command lifecycle and failures without API keys, credentials or manuscript content.

- [ ] **Step 2: Write architecture and cloud documents**

Document the command/data boundaries, SQLite location, migration policy, branch/worktree policy, Codex Cloud environment setup and how to resume the same task from mobile.

- [ ] **Step 3: Add repository guidance**

Create `AGENTS.md` with the Phase 0 scope boundary, required verification commands, no-secret rule and instruction to keep provider-specific logic out of the domain.

- [ ] **Step 4: Add GitHub Actions validation**

Configure CI for Node and Rust toolchains. Keep Android build as a documented environment check unless the runner has Android SDK/NDK configured.

- [ ] **Step 5: Run documentation and CI-equivalent checks**

Run every command listed in `README.md` and the workflow locally where the installed environment permits. Record unavailable Windows/Android checks rather than hiding them.

- [ ] **Step 6: Commit documentation and automation**

```bash
git add README.md AGENTS.md docs .github src-tauri/Cargo.toml
git commit -m "docs: document architecture cloud workflow and CI"
```

### Task 6: Windows build, Android scaffold and final verification

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Create or modify: `src-tauri/gen/android/` (generated by Tauri CLI only)
- Modify: `docs/CLOUD_DEVELOPMENT.md`

- [ ] **Step 1: Verify the Windows development and build commands**

Run `npm run tauri:dev` for a smoke launch and `npm run tauri:build` for the Windows bundle. Record the exact result and any missing SDK/toolchain.

- [ ] **Step 2: Initialize Android project files**

Run the Tauri Android initialization command and verify the generated Android project is tracked while build artifacts remain ignored.

- [ ] **Step 3: Run the Android check**

Run the repository’s documented Android build/check command. If Android SDK/NDK is unavailable, report `environment-limited` with the missing component and keep the scaffold intact.

- [ ] **Step 4: Execute the complete verification matrix**

Run Rust fmt/check/clippy/test, frontend typecheck/lint/test/build, Windows build if available, and Android scaffold/build check. Confirm project persistence manually with a temporary app data directory.

- [ ] **Step 5: Commit the verified platform setup**

```bash
git add src-tauri docs/CLOUD_DEVELOPMENT.md
git commit -m "chore: verify Windows build and prepare Android target"
```

## Completion report

At the end, report `IMPLEMENTED`, `ARCHITECTURE DECISIONS`, `FILES CREATED / CHANGED`, `DATABASE`, `TESTS`, `WINDOWS STATUS`, `ANDROID STATUS`, `KNOWN LIMITATIONS` and `NEXT RECOMMENDED PHASE`. Do not begin Phase 1 without a new user approval.

