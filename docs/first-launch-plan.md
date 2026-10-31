# First-launch data-folder setup

Implement in this workspace; leave changes uncommitted. Native Rust builds and execution stay on GitHub runners.

## Contract and approach

Replace the initial Yes/No/Cancel message box with a single Aevrik setup window. Show the real default data folder, an editable path, Browse, Use default folder, Exit and Open Aevrik. Validate on confirmation, show errors inline and preserve input on cancellation/failure. Existing preferences, legacy workspaces and AEVRIK_DATA_DIR bypass setup. Main engine initialization and polling begin only after a confirmed selection. The main WebView uses the selected data folder; the setup WebView uses a separate Windows cache location.

Keep the existing iris/graphite and pearl surfaces, Segoe UI, compact Windows controls and standard native window frame. No redesign or additional dependencies.

## Steps

- [x] Add a failing browser test for first-launch selection, picker cancellation, retry, busy state, default reset and exit.
- [x] Split storage discovery from activation and add a small native startup module; keep preference/data locks and native validation.
- [x] Add an isolated setup frontend entry, with no workspace polling, and preserve the normal application entry.
- [x] Update hosted native coverage for first-launch exit, invalid selection, custom selection and cold restart.
- [x] Run frontend/browser/Python checks and Rust formatting, inspect light/dark/compact captures, update installation docs.

## Verification

41 Python tests, 7 Node tests, 8 browser scripts and the TypeScript/Vite production build passed locally. The new browser test failed against the old startup page before implementation, then passed. Light, dark, compact and validation-error captures were inspected; the initial logo-loading defect was corrected and covered by an assertion. Rust formatting and diff whitespace checks passed.

Rust compilation/clippy and actual first-launch WebView handoff, persistence and restart tests await the existing hosted installer check. No native build, commit or push was performed locally.

## Review focus

No database created before confirmation; setup cannot race two activations; cancelled picker retains input; invalid/unwritable/in-use/app-overlapping folders retain the setup screen; existing data is preserved; startup errors allow retry; settings changes still apply on next launch. Native validation and actual WebView startup remain hosted checks.
