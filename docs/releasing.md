# Releasing Planetes

GitHub Releases attaches release notes and downloads to a version tag, such as `v0.1.0`.
The tag identifies the source revision; the portable ZIP is the application users download. GitHub's automatic
"Source code" archives are not compiled apps.

## Prepare and review

1. Update the workspace version in `Cargo.toml` and add `docs/releases/vVERSION.md`.
2. Merge the release-preparation changes after CI passes. CI runs Windows tests, Clippy, packaging and a macOS
   compile check. Windows produces a `windows-portable` workflow artifact with the ZIP and SHA-256 file.
3. In GitHub Actions, run **Prepare draft release** on the source revision to release, supplying the matching
   version (without `v`). The workflow repeats the checks and creates a draft with the tested Windows assets.
4. Inspect the draft's notes and assets. Extract and run the ZIP, verify version information, and check Steam/Epic
   startup, search, Escape, refresh and repeated application launches.
5. Publish the reviewed draft when ready. This makes that release available to people who can access the repository.

The workflow intentionally creates drafts and fails if that release already exists; it never overwrites a published
release. An existing draft can be edited on the Releases page. If its source changes, rebuild and replace its assets
before publication, and make sure its target still matches the tested source commit.

Repository visibility is a separate decision. A published release in a private repository remains private to people
with repository access. Making the repository public exposes its source history and published releases. For v0.1,
keep the repository private and the release drafted until the owner approves publication and the visibility change.

See GitHub's [release documentation](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)
and [repository visibility documentation](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/managing-repository-settings/setting-repository-visibility).

## Local packaging

On Windows with the pinned Rust toolchain and C++ Build Tools:

```powershell
pwsh -File scripts/package-windows.ps1
```

Packaging requires a clean checkout and takes its version from Cargo. Use `-AllowDirty` only for a local preview;
`BUILD-INFO.json` records that condition. The script includes only the executable, portable instructions, licenses,
dependency notices and build information. It produces `dist/Planetes-vVERSION-windows-x64.zip` and `.zip.sha256`.
Machine-specific artifacts, benchmarks, debug symbols and source manifests are excluded.

The x64 executable statically links the C runtime and embeds its icon, version data and per-monitor DPI manifest.
Build-path remapping keeps this machine's user-folder paths out of the shipped executable. The app is unsigned;
signing and automatic updates are future work.

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
