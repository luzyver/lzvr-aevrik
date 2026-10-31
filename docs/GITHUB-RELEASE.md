# Windows build and release

Workflow: [windows-release.yml](../.github/workflows/windows-release.yml).

Source version: **0.1.0-beta-1**. Application identifier: **id.luzyver.aevrik.desktop**.
The first hosted run compiled the executable but failed the native package test's Windows path comparison; the test correction awaits a new run. Old local build artifacts, including the 0.3.0 binaries, were removed during cleanup. A downloadable release is published only after the workflow passes.

## First build

1. Put this source, including `.github/workflows/windows-release.yml`, into your GitHub repository using your repository UI or Git client. Keep `node_modules`, `target`, `.tooling`, `release` and `deliverables` excluded; `.gitignore` covers them.
2. Push `dev` or `beta`, or open **Actions â†’ Windows build and release â†’ Run workflow** and select either branch. Each branch must contain the workflow and application source first. Manual runs become available once the workflow exists on the default branch, `master`; an empty `master` does not expose that button.
3. Download **aevrik-windows-x64** from the successful run. It contains the Setup EXE, portable EXE, ZIP, SHA-256 checksums, release notes and build metadata. **windows-verification** contains test evidence.

The Windows runner installs the pinned Rust toolchain, Node 24, Python 3.13 and test tooling. It checks the frontend, Rust, packaging and native application, then uses `tauri bundle --bundles nsis --no-binary-patching` to create Setup from the tested executable without another Rust compilation. The installer smoke check covers the directory picker, custom installation path, data-folder persistence and uninstall before uploading distributable files. No long-lived GitHub secret or signing certificate is required; the EXE is unsigned.

The pinned [Rust Cache action](https://github.com/Swatinem/rust-cache) caches Cargo dependencies and their compiled artifacts under the workspace's `target` directory, covering both test and release profiles. Its cache key accounts for the Rust toolchain, Cargo manifests, lockfile and compiler environment. Cache saving remains enabled when a later test fails. Application crates are rebuilt, and all verification steps still run. The first run fills the new cache; subsequent runs can reuse it. Toolchain/dependency changes, cache eviction and download/upload time affect the benefit; no hosted speedup has been measured yet.

Manual inputs can check all package URLs or perform full downloads of the three newly added products. Full package smoke includes a large MongoDB archive and runs automatically for release tags.

## Branches and releases

| Event | Build | Publication |
| --- | --- | --- |
| Push to `dev` | Yes | Replace prerelease `dev` |
| Push to `beta` | Yes | Replace prerelease `beta` |
| Push to `master` | Yes | Automatic stable tag/release when a new version is due |
| Push to `feat/*`, `fix/*` or other branches | No | None |
| Pull request opened, reopened or updated | Yes | Artifacts only |
| Push numbered `v*` tag | Yes | Separate versioned release |
| Manual run | Yes | Selected dev/beta channel, or automatic stable release on `master` |

Promote source through feature/fix branch â†’ `dev` â†’ `beta` â†’ `master`. This workflow does not merge branches automatically. Stable tags must point to commits already included in `master`; tags containing a prerelease suffix produce prereleases instead.

## Automatic stable versions on master

Pushes and manual runs on `master` read stable tags reachable from the build commit, their published GitHub releases, and commit messages since the latest completed stable version. Unpublished tags and drafts must be recovered before later commits can allocate a version. The first stable release removes the suffix from the source version: `0.1.0-beta-1` becomes `v0.1.0`. Subsequent releases use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/):

| Commit | Change from `1.2.3` |
| --- | --- |
| `fix: repair installation` or `perf: reduce startup time` | `1.2.4` |
| `feat: add a runtime` | `1.3.0` |
| `feat!: remove an incompatible API` or a `BREAKING CHANGE:` footer | `2.0.0` |
| `docs:`, `chore:`, `ci:`, `test:`, other non-release types, or unrecognized subjects | No release by themselves |

The highest change across all included commits wins, and breaking changes increment major even before `1.0.0`. Ordinary merge commits are ignored while their underlying commits are analyzed. For squash merges, use a Conventional Commit title and preserve any breaking-change footer in the final commit message. Commit labels must describe the actual compatibility impact; the workflow does not infer it from the diff.

Before compilation, `prepare_stable.py` stamps the selected version into npm/Tauri/Cargo manifests, both local Cargo.lock entries and visible application labels in the disposable checkout. It generates release notes from commit subjects. No version-bump commit is pushed: the development manifests stay unchanged. The stable tag points to the tested source commit, and `build-info.json` records the generated version. A rebuild triggered by a stable tag stamps that tag's version again before validation and compilation.

Master and version-tag runs share a serialized queue with `queue: max` and cancellation disabled. GitHub retains up to 100 pending runs per group instead of replacing the previous pending run; additional runs are canceled if that queue fills ([GitHub concurrency rules](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency)). After every required test (including full package downloads) passes, the publication job verifies artifact hashes, version and commit. A new tag requires the commit to still be the latest `master`. It creates the immutable tag, uploads files to a draft release, and publishes only after upload completes. An already published release is left unchanged. Conflicting tags or newer stable tags outside the current history stop publication instead of being moved or reused.

If publication fails after creating a tag or draft, subsequent master runs stop during version planning with an explicit unpublished-tag error, even for documentation-only pushes. To recover, open the **original failed run** in Actions and choose **Re-run failed jobs**. Its publication job uses the original artifacts and SHA; it can finish the matching draft even after master advances, provided that SHA remains an ancestor of master and no newer stable release has been published. If artifacts have expired, choose **Re-run all jobs** on that original run to rebuild and test the same SHA and version. After recovery, rerun any newer runs stopped by the unpublished-tag error. Recovery is an explicit rerun, not an automatic retry on the newer push; never move or delete the reserved tag to retry an upload.

Publication uses the built artifacts directly. Tags created with `GITHUB_TOKEN` do not trigger another push workflow, so there is no second compilation for automatic releases ([GitHub workflow triggering rules](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow)). Repository rules must allow the publication job to create version tags and releases.

## Rolling dev and beta releases

Every successful push or manual run on `dev` or `beta` updates that branch's GitHub prerelease: **Aevrik dev** (tag `dev`) or **Aevrik beta** (tag `beta`). The first run creates it; subsequent runs replace these five assets in place, where `<channel>` is `dev` or `beta`:

- `Aevrik-<channel>-windows-x64.exe`
- `Aevrik-<channel>-windows-x64.zip`
- `Aevrik-<channel>-windows-x64-setup.exe`
- `SHA256SUMS.txt`
- `build-info.json`

The tag moves to the built commit. The prerelease is never marked as the latest stable release. Its notes record the actual application version, commit and workflow URL. The application version does not increment automatically on each push.

Runs are serialized within each channel, and publication checks that the built commit is still that branch's current head. An older queued build cannot overwrite a newer build's release. Channels use separate tags and releases. A failed build/test stage leaves the existing release unchanged. Asset replacement uses GitHub's upload API and is not an atomic multi-file transaction; a failed upload step must be rerun to finish replacement.

The automatic `GITHUB_TOKEN` needs `contents: write` only in the publication job. Repository rules must allow updating the `dev` and `beta` tags and replacing assets on these mutable prereleases. Versioned release tags are handled separately and are never overwritten by this job.

## Versioned milestone releases

Current development branch: **dev**. Version sequence: `v0.1.0-beta-1`, `v0.1.0-beta-2`, `v0.1.0-beta-3`, and so on. Manifests omit the leading `v`. For each numbered beta milestone, update both Cargo manifests, their workspace entries in Cargo.lock, package.json, the root entries of package-lock.json, tauri.conf.json, visible version labels and release notes before tagging. Builds do not increment the beta counter automatically.

Numbered beta tags publish separate GitHub **prereleases** and explicitly do not become the latest stable release. They retain milestone downloads independently of the rolling `dev` and `beta` releases. Stable releases normally use the automatic master flow above; manually pushed stable tags are also supported when their commits belong to `master`.

Once the source is committed and pushed through your Git client, create the matching version tag. For this source, the CMD commands are:

```cmd
git tag v0.1.0-beta-1
git push origin v0.1.0-beta-1
```

Prerelease tags must match the source manifests. Stable tag builds stamp the tag's version first; all builds then validate agreement between package.json, package-lock.json, both Cargo manifests and tauri.conf.json. A successful tag run publishes a release named after that version, attaching the Setup EXE, portable EXE, ZIP, checksums and build metadata. PR builds only upload artifacts. Master builds publish when version planning calls for a release; dev/beta builds update their rolling releases. Artifact retention is 14 days; verification retention is 7 days. Published versioned release assets remain until removed; rolling assets are replaced on successful updates.

The build job has read-only repository access. Only publication jobs receive `contents: write`, using the automatic `GITHUB_TOKEN`. Repository policy must permit that permission, the pinned official actions and `Swatinem/rust-cache`. Automatic master retries skip already published stable releases; manually pushed numbered tags fail if their release already exists. Only rolling dev/beta releases and unfinished stable drafts allow asset replacement.

## Verification boundaries

Local source validation does not prove that a hosted workflow or newly compiled native executable has passed. Actionlint 1.7.12 checks the workflow but does not yet recognize GitHub's documented `concurrency.queue` property; the local review checked that property against GitHub's documentation and suppressed only that specific diagnostic for the remaining lint checks. Windows UI automation, full archive downloads and release compilation must pass on GitHub before 0.1.0-beta-1 is considered delivered. CI networking or the runner's WebView2/desktop environment can still cause a run to fail; inspect the uploaded verification evidence and step logs.

To keep release publication gated, native and full package tests are not marked `continue-on-error`. If any required test fails, no release is published.
