# Application updates and installer data selection

> **For agentic workers:** Implement in order with test-driven-development and verification-before-completion. Leave changes uncommitted.

**Goal:** Download and install a release from Settings, and select persistent data storage during Setup.

**Architecture:** Reuse published build-info artifact hashes and NSIS Setup. The engine validates metadata and streams a verified installer; the desktop launches it only for an idle installed application. An NSIS data page calls the installed binary in a headless configuration mode, sharing the engine's validation and atomic preferences.

**Tech Stack:** Existing Rust, Tauri, TypeScript, NSIS, Python/Playwright.

**Spec:** User approved the preceding proposal: in-app Download & install update; initial Setup chooses data; upgrades reuse saved data.

## Constraints and review focus

- Preserve channel routing, application identifier, portable setup fallback, and all existing data.
- Never execute a partial download, wrong checksum, unexpected artifact, or downgrade.
- Recheck release identity at install time; fail safely during rolling release replacement.
- Keep existing preferences, including inaccessible drives, rather than silently resetting them.
- Block duplicate update requests and installation while services or runtime operations are active.
- Build/release installers on hosted CI; leave changes uncommitted.

## Tasks

- [x] Add engine tests for installer selection, byte/hash verification, and install-time data preferences; observe failures.
- [x] Implement `updates::download_app_update` and `data_location::configure_installation`, preserving existing locations under a preference lock.
- [x] Add desktop install IPC, exclusive update state, idle checks, installer launch and orderly app exit.
- [x] Add NSIS data folder page and headless configuration entry point; extend hosted smoke tests for selection and upgrade persistence (execution awaits CI).
- [x] Extend Settings interaction tests for busy/error/retry/install and connect the action; preserve layout at 800px.
- [x] Run frontend/Python tests, build, Rust formatting and available Rust checks; inspect UI and document hosted-only verification limits.

## Verification

- `cargo test --workspace --locked`: 61 tests passed, including 8 data-location tests, 2 handoff coordination tests and 2 installer error-reporting tests.
- `cargo check -p aevrik --locked`, workspace Clippy with warnings denied, and Rust formatting passed.
- `npm test`: 8 tests passed; `npm run build` passed.
- Python unit suite: 41 tests passed; changed native test scripts compile.
- All 9 browser scripts passed. Update checks also cover duplicate requests, retry after failure, and unavailable/current/ahead states. Reviewed light and compact screenshots; design detector found no issues.
- Windows Setup creation, interactive NSIS execution and end-to-end installer restart remain hosted-CI checks. No release installer was built, run or published locally.
- Review fix: `/S` forwards `--silent` to configuration, returning failure without a dialog. The hosted installer test checks an invalid silent data path exits nonzero within 60 seconds without interaction; execution awaits CI.
