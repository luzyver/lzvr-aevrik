# Aevrik — Windows local environment manager

Aevrik is a Windows desktop application built with Rust + Tauri 2, with a graphite/iris and pearl interface. The name is a working name; brand clearance is not established.

Version 0.1.0-beta-1 adds selectable runtime folders, guarded uninstall and a larger catalog to the runtime installer and foreground process manager. See the remaining limitations below. The source is prepared for hosted builds; old local EXE/ZIP artifacts have been removed during cleanup.

Current development branch: `dev`. Successful pushes update one rolling GitHub prerelease with tag `dev`, replacing its EXE/ZIP assets. The application version remains `0.1.0-beta-1`; numbered milestone tags follow `v0.1.0-beta-1`, `v0.1.0-beta-2`, and so on. `master` is reserved for the later stable phase.

## Run the desktop application

Download the EXE or ZIP from a successful GitHub Actions run or tagged GitHub Release. The workflow builds and tests on a Windows runner, so no release compilation is needed on your laptop. See [GitHub build and release](docs/GITHUB-RELEASE.md). This workspace has no Git remote yet; hosted verification has not run.

Development prerequisites:
- Rust **1.99.1**, target `x86_64-pc-windows-msvc`, installed with rustup. The repository pins this version.
- Microsoft Visual Studio 2022 Build Tools with **Desktop development with C++** and a Windows SDK.
- Microsoft Edge WebView2 Runtime.
- Node.js/npm. This workspace was built with Node 25.9.0 and npm 11.21.1.

Open a new terminal after installing rustup so Cargo is on PATH.

From the project folder:

```cmd
npm ci
npm run desktop:dev
```

The hosted build produces an unsigned portable executable and ZIP. The target computer needs WebView2, but does not need Rust or Node.js.

## What works

- Actual native window minimize, maximize, drag and close.
- Fixed desktop chrome and workspace; lists, properties and output scroll independently.
- An offline-visible catalog of 14 products and 140 selectable versions: Caddy, nginx, PHP, Node.js, Python, Go, Java (Temurin), Bun, Deno, MariaDB, MongoDB, DuckDB, SQLite and Mailpit, with HTTPS downloads, pinned SHA-256 verification, progress and retry.
- Multiple versions installed side by side, with independent progress/status and service registration prefilled from the selected executable. Existing services keep their paths.
- Choose the runtime installation folder for each new install, for example `D:\Aevrik\runtimes`; the last choice is remembered. Downloads stream to that drive.
- Uninstall a selected managed version after confirmation. Referenced runtimes and unsafe folder trees are protected. Remove a stopped service registration separately without deleting its files.
- Architecture labels: nginx is the official x86 Windows build; other catalog products are x64.
- Register existing project folders; unique project names and .test domain labels are validated.
- Register an existing .exe, working directory, argument list and optional expected TCP port.
- Start, stop and restart registered processes; inspect their real PID and exit code.
- Show stdout/stderr, bounded to the latest 800 log entries for the current session.
- Link a project to a managed service and save its runtime label.
- Persist registrations, project properties and appearance in SQLite.
- Native file/folder pickers, filtering, Ctrl+K search focus and graphite/pearl appearance.
- Visible validation, unavailable-port, IPC and startup errors.

Runtime labels and domain labels are metadata. They do not switch executable versions or configure DNS.

## Register a service

Open **Runtimes**, choose a product version, select **Install**, choose its folder and confirm. Then use **Configure service**. Supply the working directory, foreground launch arguments and optional port before registering. Installation alone does not start a service. MariaDB and MongoDB installation supply binaries only; database initialization and configuration are still manual. SQLite and DuckDB are command-line tools, not persistent servers. See [package catalog details](docs/PACKAGE-CATALOG.md).

Use **Register service**, select its .exe and working directory, and enter one argument per line. Arguments are passed directly without shell expansion; do not add quoting around paths with spaces. Selecting a file does not launch it. Press **Start** after registration.

Use a foreground process. Daemonized programs, self-installing Windows services and services managed by another application are outside this build's process ownership model.

An expected port is an availability preflight on 127.0.0.1; it does not configure the executable's listening port and is not a readiness check. **Running** means the process is alive.

**Stop, Restart and closing Aevrik terminate the owned process tree through a Windows Job Object.** Graceful shutdown protocols are not implemented. Shut down database servers through their own tools before closing Aevrik; do not use this build to manage important database workloads. Existing external processes are not killed to free occupied ports.

## Storage

Application identifier: `id.luzyver.aevrik.desktop`. Fresh metadata: `%APPDATA%\id.luzyver.aevrik.desktop\aevrik.sqlite`.

When the new location has no database but `%APPDATA%\dev.aevrik.desktop` contains the old database, Aevrik keeps using that legacy directory in place. Close older Aevrik windows before opening 0.1.0-beta-1. Existing runtime and project paths remain valid; nothing is copied or silently moved.

The initial runtime folder is `<metadata-directory>\runtimes`. Each installation can use a different absolute folder, including `D:\Aevrik\runtimes`, with files under `<chosen-folder>\<package-id>\<version>`. A small index records the last choice and every installed version's location. The executable itself can also be kept on D:. Choosing a runtime folder does not move metadata or existing versions.

No global PATH entries, system services or externally managed runtimes are changed. Installed packages remain available after reopening. Interrupted downloads can be retried; a forced exit can leave a `.download-*` or `.install-*` temporary file/folder, never an Installed entry.

**Uninstall deletes everything inside the selected version folder.** Keep databases and personal configuration outside runtime folders. Removal is blocked for service/project references, mismatched ownership markers, nested managed installations, symlinks and junctions. Stop the service and use **Remove registration** first; this preserves files and unlinks projects.

For an isolated or portable metadata directory, set `AEVRIK_DATA_DIR` to an absolute directory before launch. The application creates that directory. Service registrations persist, but processes do not start automatically after reopening. Logs are session-only. The app needs no administrator privileges for its implemented features.

## Not implemented yet

Automatic runtime updates and catalog refresh, cancel/resume, service-registration editing, automatic per-project configuration, DNS/hosts/proxy routing, HTTPS certificates, database initialization/administration, graceful service shutdown, system tray, start with Windows, installers, signing and automatic app updates. MySQL, PostgreSQL and Redis are not bundled. Corresponding native pages describe availability instead of simulating success.

## Frontend development

`npm run dev` serves the native frontend alone. Without Tauri it displays an engine-unavailable message; it never substitutes sample process state.

## Layout

- `crates/aevrik-engine/`: independent Rust validation, SQLite storage, process ownership, log capture and isolated package installation.
- `src-tauri/`: desktop lifecycle, IPC commands, window permissions and file dialogs.
- `desktop/`: typed IPC bridge, state, views, inspectors, forms and interactions.
- `styles/`: shared design tokens and presentation.
- `assets/mark.svg`: application logo. Desktop icon helpers live in `desktop/icons.js`.
- `tools/`: repeatable frontend and native smoke tests.
- `docs/REFERENCES.md`: design and naming research.
- `previews/desktop/`: captures and verification reports; report names distinguish mocked IPC from the real executable.

## Verification

```cmd
npm run build
npm test
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

With Python Playwright and Chromium installed, run `python tools/run_ui_checks.py` for all frontend contract, layout, catalog and version-selector checks. These intentionally mock IPC and start their own Vite server if needed.

The workflow runs `check_native.py` and `check_native_packages.py` against its freshly built 0.1.0-beta-1 executable, including two Caddy versions, a chosen folder, HTTP startup, viewport checks, persistence and guarded uninstall. Do not run the updated native test against the older 0.3 executable. Tag builds also fully download and query the versions of MongoDB, DuckDB and SQLite through `check_catalog_additions.py`; this does not start a database server.

To verify selected pinned archives independently, run `cargo run -p aevrik-engine --example install_smoke -- <absolute-isolated-directory>`. By default it downloads the original four packages and never starts a server. Append release IDs to test specific versions.

Native tests use temporary metadata/WebView2 profiles and controlled fixtures, and close their own application. Test directories remain in the runner's temporary folder until its disposal. The beta version/workflow checks are recorded in `previews/desktop/beta-version-verification.json`. The broader implementation checks before renaming this release remain in `previews/desktop/source-verification-0.4.0.json`; those historical filenames are retained. Hosted build/native results are still pending.
