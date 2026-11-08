# Releasing Planetes

GitHub Releases attaches release notes and downloads to a version tag, such as `v0.1.0`.
The tag identifies the source revision; `Planetes-win-Setup.exe` (or `Planetes-win-Portable.zip`) is the application
users download, and the `.nupkg` files with `releases.win.json` are the update feed installed copies read. GitHub's automatic
"Source code" archives are not compiled apps.

## Prepare and review

1. Update the workspace version in `Cargo.toml` and add `docs/releases/vVERSION.md`.
2. Merge the release-preparation changes after CI passes. CI runs Windows tests, Clippy, packaging and a macOS
   compile check. Windows produces a `windows-release` workflow artifact with the Velopack output: installer,
   portable ZIP, full (and, after the first Velopack release, delta) update packages, feed and `SHA256SUMS.txt`.
3. In GitHub Actions, run **Prepare draft release** on the source revision to release, supplying the matching
   version (without `v`). The workflow repeats the checks and creates a draft with the tested Windows assets
   (`vpk upload github`, then the notes and `SHA256SUMS.txt`).
4. Inspect the draft's notes and assets. Install with `Planetes-win-Setup.exe`, verify version information, and check
   Steam/Epic startup, search, Escape, refresh and repeated application launches.
5. Publish the reviewed draft when ready. Publishing is what delivers the update: installed copies only see
   published releases.

The workflow intentionally creates drafts and fails if that release already exists; it never overwrites a published
release. An existing draft can be edited on the Releases page. If its source changes, rebuild and replace its assets
before publication, and make sure its target still matches the tested source commit.

The repository is public, which installed copies rely on: they read releases without a token (GitHub allows 60
unauthenticated API requests an hour per IP; Planetes makes one a day).

See GitHub's [release documentation](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)
and [repository visibility documentation](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/managing-repository-settings/setting-repository-visibility).

## Local packaging

On Windows with the pinned Rust toolchain, C++ Build Tools, the .NET 8 SDK and Velopack's CLI
(`dotnet tool install -g vpk --version 1.2.161`, matching the `velopack` crate):

```powershell
pwsh -File scripts/package-windows.ps1
```

Packaging requires a clean checkout and takes its version from Cargo. Use `-AllowDirty` only for a local preview;
`BUILD-INFO.json` records that condition. The script includes only the executable, portable instructions, licenses,
dependency notices and build information, then runs `vpk pack` into `dist/velopack`: `Planetes-win-Setup.exe`,
`Planetes-win-Portable.zip`, `Planetes-VERSION-full.nupkg`, `releases.win.json` and `SHA256SUMS.txt`. Run
`vpk download github --repoUrl https://github.com/matteomejia/Planetes --outputDir dist/velopack` first to also get a
delta update from the latest release.

To test an update locally: install one version from `dist/velopack`, package a higher version into the same folder,
then start the installed `planetes.exe --background` with `PLANETES_UPDATE_SOURCE` set to that folder. It updates
about 30 seconds later and restarts hidden. Uninstall from Windows Settings or with `Update.exe uninstall`.
Machine-specific artifacts, benchmarks, debug symbols and source manifests are excluded.

The x64 executable statically links the C runtime and embeds its icon, version data and per-monitor DPI manifest.
Build-path remapping keeps this machine's user-folder paths out of the shipped executable. The app is unsigned;
code signing is future work (`vpk pack` supports signing options).

Some crates omit license texts from their published package. `packaging/licenses` contains copies from their exact
published Git revisions, with source URLs. The notices generator checks these copies against the locked version;
dependency upgrades may require reviewing a new copy. Bundled Rune font licenses are included as well.

The orbit icon source is `app/assets/planetes.svg`; the Windows renderer in `scripts/generate-icon.ps1` reproduces
the same mark as a PNG and a multi-resolution ICO. Generated icon assets are committed so CI needs no image tools.

## Performance measurements

```powershell
cargo build --locked --release -p planetes
pwsh -File scripts/measure-windows.ps1 -Runs 5
```

The opt-in benchmark opens a native window, discovers the real local library, idles briefly, measures search
filtering and exits. Results stay in ignored `artifacts/benchmark-summary.json`. Ordinary sessions have no benchmark
timers or logging. The environment variable `PLANETES_BENCHMARK_OUTPUT` enables this mode; do not set it for normal use.

Startup records begin at Rust `main` entry and end at Rune's CPU frame callback. They omit loader time and do not
measure GPU completion or physical scanout. Each run is a fresh process with OS and driver caches retained; these
are not controlled cold-cache measurements. Search timing covers CPU filtering, not keyboard-to-display latency.
Memory sampling is external and includes the framework/graphics driver, with private bytes reported separately
from sampled working-set peaks. This is a baseline on one Windows desktop, not a cross-hardware performance claim.
