# Codex Cloud development

The repository checkout is the working directory for the cloud task. Continue work on the selected branch and keep `main` reserved for reviewed changes. Do not create a worktree for routine onboarding; the cloud environment already provides an isolated checkout.

## Reproducible install

The following setup uses only writable workspace paths and does not modify a shell profile:

```bash
set -eu
cd /workspace/AI-webnovel-creater
export CARGO_HOME=/workspace/.cargo
export RUSTUP_HOME=/workspace/.rustup
export NPM_CONFIG_CACHE=/workspace/.npm-cache
export PATH="$CARGO_HOME/bin:$PATH"

if ! command -v rustup >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init
  chmod 700 /tmp/rustup-init
  /tmp/rustup-init -y --no-modify-path --default-toolchain stable --profile minimal
fi
rustup component add rustfmt clippy
npm ci
cargo fetch
```

The native Tauri desktop prerequisites are platform-specific; a Linux cloud runner without GTK/WebKit development packages can still run the headless Rust tests and frontend checks.

## Verification

```bash
npm run typecheck
npm run lint
npm test -- --run
npm run build
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

`npm run tauri:dev` and `npm run tauri:build` require a desktop display and the target platform's native webview toolchain. Windows builds should be run on a Windows-capable runner. Android source generation uses the Tauri CLI and requires Android SDK, NDK and Java; if those are unavailable, report the check as `environment-limited` and keep generated source separate from build artifacts.

## Continue from another device

The cloud environment is the shared development workspace. A mobile Codex client can reopen the same task and inspect the branch, test output and saved configuration; it does not run the Tauri desktop binary locally. Publishing the environment snapshots the prepared checkout, while live processes and runtime authentication are recreated on the next task.

No provider key, novel content or SQLite runtime file belongs in environment scripts or source control. Phase 0 does not configure a provider secret or cloud sync destination.
