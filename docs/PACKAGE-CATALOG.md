# Runtime catalog, version 0.1.0-beta-1

The bundled catalog contains **19 products and 179 selectable releases**. It is available offline; installing requires HTTPS access to the official publisher. This is a pinned selection, not every historical release. Check for updates retrieves the curated catalog from the published Aevrik release for the current channel.

| Product | Releases | Release lines |
| --- | ---: | --- |
| Caddy | 6 | 2.11 |
| nginx | 5 | 1.31, 1.30, 1.28, 1.26, 1.24 |
| PHP | 12 | 8.5, 8.4, 8.3, 8.2, 8.1, 8.0, 7.4, 7.3, 7.2, 7.1, 7.0, 5.6 |
| Node.js | 40 | 26, 24, 22, 20, 18, 16, 14, 12 |
| Python | 8 | 3.15, 3.14, 3.13, 3.12, 3.11, 3.10, 3.9, 3.8 |
| Go | 16 | 1.27, 1.26, 1.25, 1.24, 1.23, 1.22, 1.21, 1.20 |
| Java (Temurin) | 5 | 25, 21, 17, 11, 8u504-b01 |
| Bun | 6 | 1.4, 1.3 |
| Deno | 6 | 2.9 |
| .NET SDK | 9 | 8.0, 9.0, 10.0 |
| .NET Runtime | 9 | 8.0, 9.0, 10.0 |
| ASP.NET Core Runtime | 9 | 8.0, 9.0, 10.0 |
| Zig | 6 | Six recent stable Windows ZIP releases |
| Nim | 6 | Six recent stable Windows ZIP releases |
| MariaDB | 14 | 12.2, 12.1, 12.0, 11.8, 11.4, 10.11, 10.6 |
| MongoDB | 8 | 9.0, 8.3, 8.0, 7.0 |
| DuckDB | 6 | 1.5, 1.4 (Windows ZIP releases) |
| SQLite | 2 | 3.54, 3.53 |
| Mailpit | 6 | 1.32, 1.31 |

All builds target Windows x64 except nginx, whose official Windows build is x86 and runs on x64 Windows. Older PHP/Node/Python/Go branches are provided for compatibility; their presence does not imply upstream support. Python uses the official CPython NuGet distribution, with pip. Java uses Eclipse Temurin JDKs; Bun uses the baseline x64 build.

Exact URLs, digests, architecture and expected executables are in `crates/aevrik-engine/src/packages/catalog.json`. The adjacent `catalog.sources.json` records provenance per release. Microsoft packages use publisher SHA-512; other entries use SHA-256. Where publisher metadata is unavailable, SHA-256 is computed from the official HTTPS archive. A checksum pin is not a verified publisher signature.

.NET SDK includes its runtime; choose it to build C#/F#/Visual Basic projects. The standalone .NET and ASP.NET runtimes run existing applications. Nim requires a separately installed C compiler. Aevrik does not run Nim's `finish.exe` or change global PATH. Rust and Ruby need additional installer/dependency support and are not listed as managed packages.

## Update checks

**Runtimes → Check for updates** downloads `runtime-catalog.json` from the existing `dev`, `beta`, or latest stable GitHub release. No upstream requests are made on a timer. Known releases are retained even if an older feed omits them; changed download identities are rejected. Valid additions are written atomically to `<data>/runtimes/.catalog.json`, remaining available offline. Download failure leaves the saved catalog untouched. This feed is trusted through the fixed repository's HTTPS release endpoint, not a separate signing system.

An **Install update** button appears when a product has an installed version and a newer stable catalog entry. It opens the usual installation-folder dialog. Existing services and projects are not switched; configure the new executable and link the project when ready. Database migrations are not performed.

**Settings → Aevrik updates** compares stable versions numerically and rolling builds by commit and build time. It opens the release page for manual Setup download/installation. Local builds without a compiled channel cannot check releases. Releases predating this feed show an unavailable-feed error until a new release is published.

## Install and use

1. Open Runtimes. Choose a product's version from its dropdown, then Install. Browse or type an absolute destination such as `D:\Aevrik\runtimes`, then confirm. The folder is remembered for the next installation.
2. Versions install into separate directories. Changing selection shows that version's progress/error/Installed state; other active downloads remain labelled. Search accepts a product name or version.
3. Select an installed version and choose Configure service. Aevrik fills its exact executable and parent directory. Set the correct working directory and foreground arguments, then register it.
4. Open Services and Start. Installation alone never starts a process or changes an existing service's executable.

Existing 0.2 and 0.3 release IDs, download pins and installation paths are retained. Their completion markers remain readable. Existing service registrations retain their original executable paths; automatic project version switching is not implemented.

Caddy needs a Caddyfile or explicit arguments. nginx needs its extracted nginx folder as its working directory and foreground operation. PHP includes CLI/FastCGI and needs the matching Microsoft Visual C++ runtime. Node includes npm. Python/pip, Go and Java can be invoked using their installed paths; Aevrik does not modify global PATH. Mailpit requires appropriate listening addresses/ports. MariaDB and MongoDB supply binaries only: initialize/configure a separate data directory yourself. SQLite and DuckDB are CLI tools; they are not background servers. Stop/Restart/app exit force-terminate owned processes; shut down database servers using their own tools first.

## Installation contract

- Runtime destination: `<chosen root>/<id>/<version>`, initially `<Aevrik data directory>/runtimes`; no global PATH, hosts or system-service modifications. Per-release roots and the default are persisted in `.locations.json` under the original runtime metadata directory. Choosing a folder never moves existing installations.
- HTTPS-only downloads with a 20-second connection timeout, 30-minute request timeout and 2 GiB compressed-size limit. Downloads stream through a 64 KiB buffer to a temporary file on the chosen drive.
- SHA-256 or SHA-512 verification before extraction. ZIP paths reject traversal, Windows aliases, duplicate case-insensitive paths, links and special files. Limits: 40,000 entries and 8 GiB expanded content.
- Extraction occurs in a private temporary directory on the destination filesystem. The expected executable and completion marker must exist before atomic publication.
- Existing destinations are never overwritten. An OS file lock serializes install/uninstall of the same package within the same metadata directory. A separate lock serializes atomic location-index updates.
- Completed installs are discovered from their catalog-matching marker and executable on every launch. Failures never become Installed. Forced exits can leave unused download/staging files; they are not treated as completed packages.
- Process control uses a separate engine lock, so downloads do not block service snapshots or controls.

Uninstall requires explicit confirmation and a matching ownership marker. It removes only the selected version directory, including any data/configuration stored inside it. Registered service executables, working directories, absolute path arguments and registered project folders protect referenced installations. Remove stopped service registrations first; this unlinks projects but preserves their files. Junctions, symlinks and nested managed installations block removal. Other versions and files outside the selected directory remain untouched.

Cancel/resume, in-place upgrades, unattended catalog refresh, database initialization, graceful shutdown and automatic per-project version switching remain unavailable. MySQL, PostgreSQL and Redis are not in this release.

## Maintaining the catalog

Run `python tools/sync_catalog.py --refresh --output .tooling/catalog-candidate.json` to produce a fresh candidate and provenance from official metadata. Use `--products dotnet zig nim` to refresh only the added language sources. The standard-library source adapters live in `tools/catalog_sources/`. Downloads/metadata are cached under `.tooling/catalog-source-cache`. Existing shipped release pins and provenance are preserved. Review and promote the candidate through the normal release workflow; every release publishes the catalog asset, allowing older compatible apps to fetch it without installing a new app build.

Run `python tools/verify_catalog.py .tooling/catalog-candidate.json` before promoting the candidate. It checks every URL's ZIP signature and compressed size; it does not download/hash/extract every archive. Review both JSON files, copy them together into the bundled catalog, then run Rust and UI checks. Use `cargo run -p aevrik-engine --example install_smoke -- <absolute-isolated-directory> <release-id> ...` for actual digest/extraction validation. With no IDs, the smoke test installs the original four releases only.

Release evidence distinguishes URL checks from full installations. All 39 added URLs passed local header/ZIP-signature checks; this does not prove full archive checksums or execution. Full installation and version checks for .NET SDK, both .NET runtimes, Zig and Nim are added to the existing hosted smoke script alongside MongoDB, DuckDB and SQLite. They run for versioned/stable releases and when `full_package_smoke` is selected. Rust tests, compilation and these native checks remain pending until CI runs.
