# Windows build and release

Workflow: [windows-release.yml](../.github/workflows/windows-release.yml).

Source version: **0.1.0-beta-1**. Application identifier: **id.luzyver.aevrik.desktop**.
No hosted run has happened yet: this workspace has no Git remote configured. Old local build artifacts, including the 0.3.0 binaries, were removed during cleanup. A 0.1.0-beta-1 executable is produced only after the workflow passes.

## First build

1. Put this source, including `.github/workflows/windows-release.yml`, into your GitHub repository using your repository UI or Git client. Keep `node_modules`, `target`, `.tooling`, `release` and `deliverables` excluded; `.gitignore` covers them.
2. Push `dev`, or open **Actions → Windows build and release → Run workflow** and select `dev`. Manual runs become available once the workflow exists on the default branch. `master` is also supported for the later stable phase.
3. Download **aevrik-windows-x64** from the successful run. It contains the portable EXE, ZIP, SHA-256 checksums, release notes and build metadata. **windows-verification** contains test evidence.

The Windows runner installs the pinned Rust toolchain, Node 24, Python 3.13 and test tooling. It checks the frontend, Rust, packaging and native application before uploading distributable files. Its Cargo cache contains dependency sources only. No long-lived GitHub secret or signing certificate is required; the EXE is unsigned.

Manual inputs can check all package URLs or perform full downloads of the three newly added products. Full package smoke includes a large MongoDB archive and runs automatically for release tags.

## Rolling development release

Every successful push to `dev` updates the same GitHub prerelease: **Aevrik dev**, tag **dev**. A manual workflow run on `dev` does the same. The first run creates it; subsequent runs replace these four assets in place:

- `Aevrik-dev-windows-x64.exe`
- `Aevrik-dev-windows-x64.zip`
- `SHA256SUMS.txt`
- `build-info.json`

The tag moves to the built commit. The prerelease is never marked as the latest stable release. Its notes record the actual application version, commit and workflow URL. The application version does not increment automatically on each push.

Dev runs are serialized, and publication checks that the built commit is still the current `dev` branch head. An older queued build cannot overwrite a newer build's release. A failed build/test stage leaves the existing release unchanged. Asset replacement uses GitHub's upload API and is not an atomic multi-file transaction; a failed upload step must be rerun to finish replacement.

The automatic `GITHUB_TOKEN` needs `contents: write` only in the publication job. Repository rules must allow updating the `dev` tag and replacing assets on this mutable prerelease. Versioned release tags are handled separately and are never overwritten by this job.

## Versioned milestone releases

Current development branch: **dev**. Version sequence: `v0.1.0-beta-1`, `v0.1.0-beta-2`, `v0.1.0-beta-3`, and so on. Manifests omit the leading `v`. For each numbered beta milestone, update both Cargo manifests, their workspace entries in Cargo.lock, package.json, the root entries of package-lock.json, tauri.conf.json, visible version labels and release notes before tagging. Builds do not increment the beta counter automatically.

Numbered beta tags publish separate GitHub **prereleases** and explicitly do not become the latest stable release. They retain milestone downloads independently of the rolling `dev` release. When ready, development can move to `master` with a stable version such as `0.1.0` and tag `v0.1.0`.

Once the source is committed and pushed through your Git client, create the matching version tag. For this source, the CMD commands are:

```cmd
git tag v0.1.0-beta-1
git push origin v0.1.0-beta-1
```

The workflow rejects numbered tags that do not match the version in package.json, package-lock.json, both Cargo manifests and tauri.conf.json. A successful tag run publishes **Aevrik v0.1.0-beta-1**, attaching the EXE, ZIP, checksums and build metadata. PR and `master` branch builds upload artifacts without publishing a release; push/manual builds on `dev` update the rolling release. Artifact retention is 14 days; verification retention is 7 days. Published versioned release assets remain until removed; `dev` assets are replaced on successful updates.

The build job has read-only repository access. Only publication jobs receive `contents: write`, using the automatic `GITHUB_TOKEN`. Repository policy must permit that permission and the pinned official actions. If a release with a numbered version tag already exists, publication fails rather than replacing its assets; only `dev` is overwritten.

## Verification boundaries

Local source validation does not prove that a hosted workflow or newly compiled native executable has passed. The workflow syntax is checked locally with actionlint. Windows UI automation, full archive downloads and release compilation must pass on GitHub before 0.1.0-beta-1 is considered delivered. CI networking or the runner's WebView2/desktop environment can still cause a run to fail; inspect the uploaded verification evidence and step logs.

To keep release publication gated, native and full package tests are not marked `continue-on-error`. If any required test fails, no release is published.
