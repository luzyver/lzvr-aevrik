# Runtime catalog expansion and categories

> **For agentic workers:** Implement in order with test-driven-development and verification-before-completion. Leave changes uncommitted.

**Goal:** Add Java JRE, Luau and Wasmtime Windows packages and classify packages by their actual purpose.

**Architecture:** Keep existing package identities and installer behavior. Share a family-to-category JSON map between catalog generation and the UI so cached catalogs also display corrected categories.

**Tech Stack:** TypeScript/Vite, Python source adapters, existing Rust installer.

**Spec:** User request to expand runtime choices and correct categories; preserve existing visual design.

## Constraints and review focus

- Official Windows ZIP releases only; record digests and provenance.
- Preserve every shipped ID, URL, version, digest and executable path.
- Exclude draft/prerelease releases and wrong architectures.
- Distinguish Java JDK from JRE; preserve unknown future categories.
- Category filtering and search must work with older cached catalog labels.
- Verify archive executables, category controls and 800px layout.

## Tasks

- [x] Add failing category/filter regression tests in `tests/package-categories.test.mjs` and adapter tests in `tools/test_catalog_runtimes.py`.
- [x] Add `tools/catalog_sources/runtimes.py`; integrate adapters and shared `catalog.categories.json` into `tools/sync_catalog.py`.
- [x] Add `desktop/package-categories.ts`; use it in package grouping and rendering. Clarify the page heading.
- [x] Generate new entries from publisher metadata, preserving shipped package identities.
- [x] Run Node/Python/Rust tests, frontend build, archive installation smoke checks and browser checks. Inspect screenshots and diff.

## Verification

- `npm test`: 10 passed; `npm run build`: passed.
- `python -m unittest discover -s tools -p 'test_*.py'`: 44 passed.
- `cargo test --workspace --locked`: 61 passed.
- Category browser checks passed at 1280px and 800px, with dark/light screenshots. Existing multiversion browser test passed after updating its expected category label.
- All 11 added archives passed full SHA-256, executable-path and PE x64 checks; all 179 existing release identities remain unchanged. Evidence: `previews/desktop/runtime-catalog-expansion.json`.
- `python tools/check_catalog_additions.py --families java-jre luau wasmtime`: latest release of each installed through the Rust package manager and executed successfully in an isolated temporary directory. Evidence: `previews/desktop/catalog-additions-smoke-0.1.0-beta-1.json`.
- UI detector and `git diff --check`: clean. Generated Tauri schema changes from compilation were discarded; no installer or updater source changes.
- No release installer was built or published; changes remain uncommitted.
