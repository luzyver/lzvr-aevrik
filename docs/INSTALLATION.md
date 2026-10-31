# Windows installation and data folders

**Goal:** Setup → choose application folder → install; choose the data folder independently.

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

Download `Aevrik-<channel-or-version>-windows-x64-setup.exe` (for example `Aevrik-dev-windows-x64-setup.exe`). Open it normally, choose the application directory and install. `D:\Aevrik` is a suitable choice when that drive is available and writable. Setup installs for the current Windows user, creates shortcuts and an entry in **Settings → Apps → Installed apps**. It downloads WebView2 if the runtime is missing. The standalone EXE and ZIP are still portable; they do not show the Setup wizard.

The bundle uses the [standard Tauri NSIS installer](https://v2.tauri.app/distribute/windows-installer/), including its directory-selection page. Normal installation is interactive; `/S` is reserved for silent installs and automated checks.

On first launch, Aevrik asks whether to choose a data folder, use the Windows-profile default or cancel. Use a separate directory such as `D:\AevrikData`. Existing Aevrik installations retain their prior data folder. **Settings → Data folder → Change folder** schedules another folder for the next launch; the current folder stays active until exit. **Keep current folder** cancels a pending change.

The selected folder holds `aevrik.sqlite`, `runtimes` and `webview`. Previously selected per-runtime destinations remain specific to their workspace. Database-server data and project files stay wherever their launch settings put them. A small `%APPDATA%\id.luzyver.aevrik.desktop\data-location.json` file remembers the selected workspace; installing the application on another drive does not relocate this pointer file.

A folder switch opens an existing workspace or starts a fresh one; it does not migrate files. Select the previous folder to return to its registrations. `AEVRIK_DATA_DIR` takes precedence and disables changing the saved location in Settings for that session. A disconnected saved drive causes a startup error; reconnect it before launching, or supply the override to open a different workspace.

Uninstall Aevrik from Windows Settings or run `uninstall.exe` in its application directory. Default uninstall preserves data. The installer's optional **Delete app data** checkbox can remove the default Windows-profile data and preferences; it does not follow the pointer to delete custom data folders. Runtime removal remains available separately inside Aevrik.

## Implementation checkpoints

- [x] Storage implementation and integration tests: `crates/aevrik-engine/src/data_location.rs`, `src-tauri/src/storage.rs`, and `crates/aevrik-engine/tests/data_location.rs`. Commands `storage_info` and `set_data_directory` expose current and next-launch folders. Rust tests await hosted execution.
- [x] Settings: `desktop/storage.ts`, `bridge.ts` and `main.ts` show current/pending location, override state, validation errors and explicit no-migration copy. `tools/check_data_location_ui.py` passes with mocked IPC at 1280×820 and 800×600.
- [x] Installer configuration and packaging: NSIS target, bundle-only npm command, checksums and all three publication paths include Setup. Python packaging/publication tests pass. `tools/check_installer.py` and `tools/check_data_location_native.py` are wired into CI.
- [x] Local checks: 34 Python tests, 5 frontend unit tests, all 6 UI scripts, TypeScript/Vite build, Rust formatting and clean design-detector result. These do not prove native execution.
- [ ] Hosted verification: Rust compilation/tests/clippy, Setup creation, first-launch cancellation, directory picker, custom installation, real IPC across cold restarts, and uninstall preserving external data. No native build or installer was run on the local laptop.
