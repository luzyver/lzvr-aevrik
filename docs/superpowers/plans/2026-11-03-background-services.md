# Background services implementation plan

Implement directly in order, using test-driven-development and verification-before-completion.

**Goal:** Closing the main window keeps projects and services running, with a tray icon to reopen or quit.

**Architecture:** Keep the existing engine and Windows Job ownership alive by hiding the main window. A dedicated tray module handles restore and explicit exit; existing exit cleanup remains authoritative.

**Tech Stack:** Rust, existing Tauri 2 tray feature, TypeScript, native Python/Playwright smoke checks.

**Spec:** User request: services/projects continue running after closing Aevrik's window.

## Constraints and review focus

- Keep changes modular and uncommitted on the current branch.
- Setup cancellation and updater exit must still exit fully.
- Create the tray before allowing the main window to hide; never leave an inaccessible background process on tray failure.
- Restore hidden/minimized windows; retain service PIDs and logs.
- Explicit exit must release owned process trees and workspace locks.
- Update cold-restart tests and data-folder instructions for the distinction between close and exit.

## Task: background lifecycle

- [x] Extend `tools/check_native.py` with real close/hide, service connection, tray restore and explicit-exit assertions. Add OS helpers to `tools/native_lifecycle.py`.
- [x] Run against the existing EXE and observe the close-keeps-running assertion fail.
- [x] Add `src-tauri/src/tray.rs`; enable `tray-icon` in `src-tauri/Cargo.toml`; wire startup, window close handling and explicit exit in `main.rs`/`startup.rs`.
- [x] Update Settings, caption tooltip, storage copy and README. Settings also exposes explicit exit.
- [x] Adapt native cold-restart tests to explicit exit.
- [x] Build the executable, rerun native lifecycle, Rust tests, clippy, formatting, frontend build and tests. Record limitations without claiming unrun hosted checks.

## Verification

- The previous EXE failed with `Closing the window stopped Aevrik`.
- `npm run desktop:build` produced `target/release/aevrik.exe`.
- `python tools/check_native.py` passed: hidden window, reachable service, same PID/logs after native tray restore, explicit exit releases the service port, and cold restart preserves registrations.
- `cargo test --workspace`: 69 passed, one opt-in npm test skipped; `cargo test -p aevrik-engine --test npm_process -- --ignored`: that test also passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --all --check`, frontend build, 10 JavaScript tests and all 12 UI scripts passed.
- Python syntax checks passed for the changed native test helpers. Hosted installer/data-folder lifecycle and HTTPS package smoke tests were updated but not rerun locally. The installed application has not been replaced.
