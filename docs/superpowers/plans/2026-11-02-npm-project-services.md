# npm project services

> Implement in this session with test-driven-development and verification-before-completion. Leave changes uncommitted.

**Goal:** From an existing project, select a package.json script and Node runtime, save a linked service, then use existing Start/Stop/Restart/Output controls.

**Architecture:** Add bounded, read-only package.json discovery and an optional npmScript field in the existing JSON service configuration (no schema migration). Run the selected Node executable with its bundled npm-cli.js, preserving npm lifecycle behavior. Prepend that Node directory to the child PATH only. Register and link in one transaction; reject an existing project link instead of replacing it implicitly.

**Scope:** npm only; no dependency installation, automatic start, deployment, or edits to project files. Keep manual executable registration. Node choices come from installed catalog packages and absolute PATH entries; allow browsing to another node.exe.

## Tasks

- [x] Add Rust tests for script discovery, invalid manifests, missing scripts/npm, persistence and atomic linking; add a real npm lifecycle test.
- [x] Implement npm discovery/configuration, process environment and project-service registration in separate engine modules, then expose two IPC commands.
- [x] Add a modular project npm dialog with script preview, Node selection, loading/error/empty states, and explicit Save service action.
- [x] Verify frontend contracts, browser flow, real npm Start/Stop/Restart/exit and selected Node inheritance; run workspace tests, build, formatting, clippy and diff checks.

## Review focus

- Paths containing spaces and Windows extended path prefixes must work through npm/cmd.
- Reading scripts never executes them. Missing/changed scripts fail before spawning.
- npm children use the selected Node even when global PATH differs.
- Old service JSON remains readable and manual launch behavior stays unchanged.
- Failed registration leaves no orphan service; existing linked services are preserved.
- Closing/reopening dialogs must not apply stale async results or double-submit.

## Verification

- `cargo test --workspace --locked`: 65 passed; the opt-in npm process test is run separately.
- `cargo test -p aevrik-engine --test npm_process --locked -- --ignored`: passed with real Node/npm, isolated project and runtime folders containing spaces, npm pre-hook, selected Node inheritance, Start/Stop/Restart, shutdown descendant cleanup and exit code 7 propagation. Added to Windows CI.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` and `cargo fmt --all -- --check`: passed.
- `npm test`: 10 passed. `npm run build`: passed.
- `python -m unittest discover -s tools -p 'test_*.py'`: 44 passed.
- `python tools/run_ui_checks.py check_desktop_ui.py check_npm_ui.py`: passed, including save/link, retry, empty scripts, stale dialog responses and 800px layout. New npm dialog screenshots retained in `previews/desktop`.
- Installed desktop application and real portfolio project were not modified or launched. Changes require a new application build.
