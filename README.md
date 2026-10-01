# Webnovel AI Studio

Webnovel AI Studio is a provider-independent workspace for planning and writing long-form fiction. The repository contains a Tauri 2 shell, React/TypeScript client, Rust application core, local SQLite persistence, structured character memory, revision safety, provider execution contracts and the Phase 7 native credential boundary.

## Current scope

The current branch supports creating, listing, opening, renaming and archiving projects; structured Character and CharacterState memory; revision history and proposals; an OpenAI-compatible provider adapter; typed provider configure/remove commands; and a `/settings/providers` setup route. The responsive workspace shell, typed Tauri command boundary and SQLite migrations are implemented.

Model routing, orchestration, semantic retrieval, continuity analysis, cloud novel-data sync and manuscript export remain later phases. Windows builds use Credential Manager and Android builds use Keystore-backed encrypted SharedPreferences through the Rust credential boundary. The cloud/headless runtime intentionally uses process-only credential storage and never writes API keys to SQLite.

## Local setup

Requirements:

- Node.js 24 or another current LTS release
- Rust stable with `rustfmt` and `clippy`
- Tauri 2 platform prerequisites for the desktop target

Install dependencies and prepare the pinned lockfiles:

```bash
export NPM_CONFIG_CACHE=/workspace/.npm-cache
npm ci
cargo fetch
```

On Windows PowerShell, if script execution blocks `npm.ps1`, use the command
shim instead:

```powershell
npm.cmd ci
npm.cmd run dev
```

Use `npm.cmd run tauri:dev` after installing Rust stable, the MSVC toolchain,
Visual Studio Build Tools with Desktop C++, the Windows SDK and WebView2.

The Rust crate is a workspace member under `src-tauri`. In environments where the shell profile is read-only, set `CARGO_HOME=/workspace/.cargo`, `RUSTUP_HOME=/workspace/.rustup` and prepend `/workspace/.cargo/bin` to `PATH`.

## Commands

```bash
npm run dev                 # Vite client
npm run build               # production frontend bundle
npm test -- --run           # Vitest UI tests
npm run typecheck
npm run lint
npm run format:check
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
npm run tauri:dev           # Tauri desktop shell (native prerequisites required)
npm run tauri:build         # desktop bundle (native target required)
```

The default Rust test profile excludes the optional `tauri-app` feature so domain, SQLite and command-helper tests can run in headless CI. `tauri:dev` and `tauri:build` enable that feature and compile the native webview runtime.

## Data and security

Project data is stored in the platform app-data directory as `webnovel.sqlite`. Migrations live in `migrations/` and are tracked in the `_migrations` table. Runtime SQLite files, API keys and local credentials must never be committed.

The frontend never opens SQLite. All project operations use the typed client in `apps/client/src/lib/commands.ts`, which invokes Rust commands and maps typed errors to safe user messages.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the data boundaries and [docs/CLOUD_DEVELOPMENT.md](docs/CLOUD_DEVELOPMENT.md) for the Codex Cloud workflow.

Development work stays on a feature branch such as `phase6/implementation`. Do not merge feature branches into `main` until the changes have been reviewed and the full verification matrix is available.
