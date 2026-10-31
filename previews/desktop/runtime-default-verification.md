# Runtime defaults verification — 2026-10-31

Implemented scope: Set as default / Remove from PATH for installed Node.js,
Python, PHP, Bun, Deno, Zig and Nim. Explicit scope confirmation, live default
status, permission/stale-plan failures, and uninstall protection.

Local evidence:

- Regression-first: new Node IPC test failed for missing previewDefault;
  new browser contract failed for missing Terminal default state before UI changes.
- `npm test`: 8 passed.
- `npm run build`: TypeScript and Vite passed.
- `python -m unittest discover -s tools -p 'test_*.py'`: 41 passed.
- `python tools/run_ui_checks.py`: all 9 suites passed; default suite rerun after
  keeping permission errors visible in compact dialogs and final copy changes.
- `cargo fmt --all --check` and `git diff --check`: passed.
- Impeccable detector on package-view.ts, package-dialogs.ts and native.css: `[]`.

14 new Rust tests are included in the existing Windows workflow. They cover
PATH ordering/removal, registry value types and Unicode, scope, PATHEXT,
environment expansion, ownership, stale confirmation and uninstall guards.
The registry roundtrip uses a disposable HKCU Software test key, never the real
Environment key. Rust compilation, Clippy, Rust tests and real executable smoke
were not run locally: native builds remain on GitHub Actions as requested.
Local browser tests mock IPC; the host PATH was not changed.

Review correction: added missing ALLUSERSPROFILE and PUBLIC bootstrap expansion.
A child-process regression test supplies temporary profile paths without changing
the parent environment. It covers machine-scope planning, the current executable,
uninstall protection through both PATH scopes, mixed-case variable names, sibling
folder exclusion and REG_SZ literal preservation. This Rust regression has not
been executed locally; its red/green verification remains pending on Windows CI.

## Scoped UI finish review

Reviewed directly in-thread against the incumbent Windows utility UI. Product,
design and mockup documents previously removed by the user were not recreated.
No new comp, concept seed or quality-bar card applies to this scoped extension.

disposition: ship (UI scope only; native verification remains pending)

### persistence

Pass: implementation, feature documentation, automated checks and captures are present.

### fidelity

Windows frame, graphite/pearl surfaces, iris accent, Segoe typography and existing
catalog rows: match. Default action/status, scope confirmation and error state:
authorized additions. Compact 800×600 and desktop 1280×820 captures: match.

### ceiling

Existing Windows utility character retained; no additional native-device classes apply.

### material_fixes

None remaining in the reviewed UI. Earlier clipped permission message was moved
outside scrolling details and is now fully visible above the dialog actions.

### keep

Keep explicit scope, readable errors, default/version labels and guarded uninstall.
