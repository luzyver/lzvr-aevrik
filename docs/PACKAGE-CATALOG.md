# Runtime catalog, version 0.1.0-beta-1

The bundled catalog contains **14 products and 140 selectable releases**. It is available offline; installing requires HTTPS access to the official publisher. This is a pinned selection, not every historical release or automatic upstream discovery.

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
| MariaDB | 14 | 12.2, 12.1, 12.0, 11.8, 11.4, 10.11, 10.6 |
| MongoDB | 8 | 9.0, 8.3, 8.0, 7.0 |
| DuckDB | 6 | 1.5, 1.4 (Windows ZIP releases) |
| SQLite | 2 | 3.54, 3.53 |
| Mailpit | 6 | 1.32, 1.31 |

All builds target Windows x64 except nginx, whose official Windows build is x86 and runs on x64 Windows. Older PHP/Node/Python/Go branches are provided for compatibility; their presence does not imply upstream support. Python uses the official CPython NuGet distribution, with pip. Java uses Eclipse Temurin JDKs; Bun uses the baseline x64 build.

Exact URLs, digests, architecture and expected executables are in `crates/aevrik-engine/src/packages/catalog.json`. The adjacent `catalog.sources.json` records provenance per release. Publisher SHA-256 metadata is preferred; where unavailable, the catalog pins a SHA-256 computed from the official HTTPS archive. A checksum pin is not a verified publisher signature.

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
- SHA-256 verification before extraction. ZIP paths reject traversal, Windows aliases, duplicate case-insensitive paths, links and special files. Limits: 40,000 entries and 8 GiB expanded content.
- Extraction occurs in a private temporary directory on the destination filesystem. The expected executable and completion marker must exist before atomic publication.
- Existing destinations are never overwritten. An OS file lock serializes install/uninstall of the same package within the same metadata directory. A separate lock serializes atomic location-index updates.
- Completed installs are discovered from their catalog-matching marker and executable on every launch. Failures never become Installed. Forced exits can leave unused download/staging files; they are not treated as completed packages.
- Process control uses a separate engine lock, so downloads do not block service snapshots or controls.

Uninstall requires explicit confirmation and a matching ownership marker. It removes only the selected version directory, including any data/configuration stored inside it. Registered service executables, working directories, absolute path arguments and registered project folders protect referenced installations. Remove stopped service registrations first; this unlinks projects but preserves their files. Junctions, symlinks and nested managed installations block removal. Other versions and files outside the selected directory remain untouched.

Cancel/resume, in-place upgrades, automatic catalog refresh, database initialization, graceful shutdown and automatic per-project version switching remain unavailable. MySQL, PostgreSQL and Redis are not in this release.

## Maintaining the catalog

Run `python tools/sync_catalog.py --output .tooling/catalog-candidate.json` to produce a candidate and provenance from official metadata. The standard-library source adapters live in `tools/catalog_sources/`. Downloads/metadata are cached under `.tooling/catalog-source-cache`; remove the cache entry for a changed metadata URL to refresh it. Existing shipped release pins are preserved.

Run `python tools/verify_catalog.py .tooling/catalog-candidate.json` before promoting the candidate. It checks every URL's ZIP signature and compressed size; it does not download/hash/extract every archive. Review both JSON files, copy them together into the bundled catalog, then run Rust and UI checks. Use `cargo run -p aevrik-engine --example install_smoke -- <absolute-isolated-directory> <release-id> ...` for actual digest/extraction validation. With no IDs, the smoke test installs the original four releases only.

Release evidence distinguishes all-URL checks from full installations of representative releases. All 140 URLs passed the local header/ZIP-signature check. Full installations of the newly added products are deferred to the GitHub runner. The native smoke test installs two Caddy versions into a chosen folder, configures the selected path, checks a real HTTP response, reopens the app and tests guarded uninstall. Tagged CI runs must also install MongoDB, DuckDB and SQLite and successfully query each executable's version before publishing.
