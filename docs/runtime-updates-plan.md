# Runtime catalog and update checks

Implement in this workspace, without committing or pushing. Keep native Rust builds and runtime execution on GitHub runners.

## Contract

- Add .NET SDK, .NET Runtime, ASP.NET Core Runtime, Zig and Nim from publisher metadata. Stable Windows x64 ZIPs only; no global PATH changes. Nim still needs a C compiler.
- Add SHA-512 verification for Microsoft's published hashes; preserve all previously shipped SHA-256 package identities.
- Check for runtime updates using the catalog asset of this application's GitHub release channel. Cache additions atomically in the selected data folder. Preserve every known version and reject changed download pins, unsafe paths, invalid metadata and oversized responses.
- Show available newer versions for installed products. Selecting an update opens the existing installation-folder dialog. Do not change service registrations or migrate databases.
- Settings checks app releases for the compiled channel; rolling builds compare commit identity. Open the fixed repository's release page in the browser, with no silent installer execution.
- Package the catalog with each successful release. Existing releases without the asset report an actionable error while the offline catalog remains usable.

## Implementation and verification

- [x] Tests written for source adapters, update detection, channel routing, invalid feed/pin changes, cache persistence and SHA-512 extraction. Rust tests await CI.
- [x] Publisher source adapters and expanded pinned catalog/provenance: 19 products, 179 releases, all 140 previous identities retained.
- [x] Engine network/catalog cache and release checking; Tauri commands and build identity.
- [x] Existing Runtimes/Settings controls, loading/error states and version selection.
- [x] Release asset packaging, hosted smoke coverage and user documentation.
- [x] Local verification: 41 Python tests, 7 Node tests, TypeScript/Vite build, formatting and 7 browser scripts passed. All 39 added URLs passed ZIP-header checks. Impeccable detector returned no findings.
- [x] Review correction: select exact .NET asset names, with regression coverage for apphost appearing first and missing runtime assets. Corrected 10.0.12–10.0.14 runtime URLs/hashes; all 27 .NET entries match publisher metadata. The three corrected archives passed full SHA-512, ZIP CRC and `dotnet.exe` presence checks without execution.
- [ ] Hosted verification: Rust compilation, clippy, Rust tests, full runtime downloads/extraction/execution, native IPC and release publication. No native build, commit or push was performed locally.

Workflow lint passed with only the existing `concurrency.queue` diagnostic filtered: installed actionlint 1.7.12 does not recognize that GitHub feature. No repository lint ignore was added.

Review focus: offline startup; malformed and stale catalogs; versions installed on custom drives; concurrent installation/checking; rolling channels with unchanged version strings; older stable releases must not be offered as upgrades.
