# Windows installation and data folders

**Goal:** Setup → choose data folder → choose application folder → install; upgrades retain the saved data folder.

**Architecture:** Use Tauri's NSIS bundle for installation and removal. Store the selected data path in a small per-user location file, outside the selected data folder. Rust resolves the path before opening SQLite or starting the runtime manager. Settings schedules a folder switch for the next launch; existing files are never moved or deleted by a switch.

**Stack:** Existing Tauri 2, Rust, TypeScript, NSIS and GitHub Actions.

**Request:** Add a Windows installer with folder selection and a separate data-folder setting.

## Constraints and review focus

- Keep `id.luzyver.aevrik.desktop` and the existing dev/beta/master release policy.
- Build native binaries and installers only on GitHub; leave local changes uncommitted.
- Retain portable EXE/ZIP downloads alongside a clearly named Setup EXE.
- Preserve existing and legacy data locations, explicit `AEVRIK_DATA_DIR`, and external runtime installation paths.
- Reject relative, inaccessible, occupied, nonempty unrelated, or missing saved folders without silently creating a replacement workspace.
- Keep app and data locations separate. A folder change switches workspaces after relaunch; it does not migrate databases or rewrite service paths.
- Test installer folder selection, install/uninstall, restart persistence, environment overrides, cancellation, write failures and unchanged previous data.

## Using Setup

Download `Aevrik-<channel-or-version>-windows-x64-setup.exe` (for example `Aevrik-dev-windows-x64-setup.exe`). Open it normally, choose the data directory on the first page, then choose the application directory and install. For example, use `D:\AevrikData` for data and `D:\Aevrik` for the application. Setup installs for the current Windows user, creates shortcuts and an entry in **Settings → Apps → Installed apps**. It downloads WebView2 if the runtime is missing. The standalone EXE and ZIP are still portable; they retain the application's first-launch data picker.

The bundle extends the [standard Tauri NSIS installer](https://v2.tauri.app/distribute/windows-installer/) through `installer/data-folder.nsh`. Setup saves the selected location through the installed application's headless `--configure-data-directory` entry point; it does not create a database or WebView. Cancelling before installation saves no preference. Upgrades skip the data page and preserve existing preferences, including legacy workspaces. Invalid or inaccessible saved locations cause an error rather than replacement with an empty workspace.

Silent installs accept `/S /DATA="D:\AevrikData" /D=D:\Aevrik`; `/D` must be last and unquoted. Without `/DATA`, a fresh silent install uses the normal Windows-profile data location. An existing preference always wins over `/DATA`.

Setup forwards silent mode to the configuration process. Configuration failures return a nonzero exit code without a dialog for `/S`; interactive installations show the error.

## Updating from Settings

Choose **Check for updates**, then **Download & install update**. Stop running services and finish runtime operations first. Aevrik rechecks the published build, downloads the Windows Setup artifact, and verifies its exact size and SHA-256 against the release's `build-info.json` over HTTPS. Changed rolling releases, missing metadata, interrupted downloads and checksum failures leave the running application in place and show a retryable error.

After verification, Aevrik launches Setup in passive update mode for the current application directory, exits, and asks Setup to reopen it after installation. The data preference and external workspace remain in place. A temporary installer is retained in the Windows temp directory for the handoff. The downloads remain unsigned; checksum verification does not replace publisher code signing. Portable copies can open the release page and install Setup first.

Portable copies without a saved workspace open **Choose your data folder** on first launch. Type a path or use **Browse…**, then select **Open Aevrik**. **Use default folder** restores the Windows-profile path; **Exit** closes setup without creating a workspace. Use a separate directory such as `D:\AevrikData`. Invalid paths show an inline error so you can correct the selection. Installed copies use the folder saved by Setup and go straight to the workspace. **Settings → Data folder → Change folder** schedules another folder for the next launch; the current folder stays active until exit. **Keep current folder** cancels a pending change.

The first-launch screen uses a separate WebView cache under `%LOCALAPPDATA%\id.luzyver.aevrik.desktop\setup-webview`. It does not initialize the default workspace. After confirmation, the main window's WebView cache, metadata and default runtimes use the selected data folder.

The selected folder holds `aevrik.sqlite`, `runtimes` and `webview`. Previously selected per-runtime destinations remain specific to their workspace. Database-server data and project files stay wherever their launch settings put them. A small `%APPDATA%\id.luzyver.aevrik.desktop\data-location.json` file remembers the selected workspace; installing the application on another drive does not relocate this pointer file.

A folder switch opens an existing workspace or starts a fresh one; it does not migrate files. Select the previous folder to return to its registrations. `AEVRIK_DATA_DIR` takes precedence and disables changing the saved location in Settings for that session. A disconnected saved drive causes a startup error; reconnect it before launching, or supply the override to open a different workspace.

Uninstall Aevrik from Windows Settings or run `uninstall.exe` in its application directory. Default uninstall preserves data. The installer's optional **Delete app data** checkbox can remove the default Windows-profile data and preferences; it does not follow the pointer to delete custom data folders. Runtime removal remains available separately inside Aevrik.

## Implementation checkpoints

- [x] Storage implementation and integration tests: `crates/aevrik-engine/src/data_location.rs`, `src-tauri/src/storage.rs`, and `crates/aevrik-engine/tests/data_location.rs`. Commands `storage_info` and `set_data_directory` expose current and next-launch folders. Rust tests await hosted execution.
- [x] Settings: `desktop/storage.ts`, `bridge.ts` and `main.ts` show current/pending location, override state, validation errors and explicit no-migration copy. `tools/check_data_location_ui.py` passes with mocked IPC at 1280×820 and 800×600.
- [x] Installer configuration and packaging: NSIS target, bundle-only npm command, checksums and all three publication paths include Setup. Python packaging/publication tests pass. `tools/check_installer.py` and `tools/check_data_location_native.py` are wired into CI.
- [x] Local checks: 34 Python tests, 5 frontend unit tests, all 6 UI scripts, TypeScript/Vite build, Rust formatting and clean design-detector result. These do not prove native execution.
- [ ] Hosted verification: Rust compilation/tests/clippy, Setup creation, first-launch cancellation, directory picker, custom installation, real IPC across cold restarts, and uninstall preserving external data. No native build or installer was run on the local laptop.
