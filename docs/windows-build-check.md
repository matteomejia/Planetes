# Rune Windows build check

Verified on 2026-11-07. This checks upstream Rune before creating the Planetes Cargo workspace.

## Pinned baseline

- Rune `dev` revision: `7256c7c2afb605862ccfe600a469df7330509efb`.
- Rust and Cargo: `1.99.1`, host `x86_64-pc-windows-msvc`. Rune's toolchain file requests this Rust version.
- Visual Studio Build Tools 2022: `17.14.44`, C++ workload with recommended components.
- Windows SDK: `10.0.26100.0`.
- Native renderer: wgpu DirectX 12 on NVIDIA GeForce RTX 5090.

Rust was installed using rustup's default profile, including rustfmt and Clippy. Cargo's bin directory is
registered in the user PATH. Existing terminal/app processes may need restarting to inherit it.

## Results

- `hello` and `kit-gallery` built with the upstream lockfile and no source changes.
- `hello`'s switch interaction and spring animation test passed (1 test).
- Headless GPU snapshots and element trees succeeded for hello's `on` scene and the gallery's
  `ui-button--variants` scene. Both snapshots were visually inspected: text, switch, buttons and icons rendered.
- Both native Windows examples completed 120 presented frames with zero failed surface acquisitions or skipped
  frames, then exited successfully. No reboot or upstream workaround was needed.

This is a build and rendering smoke check, not evidence of 120 Hz performance. Runs used debug builds and small
scenes. Large-library scrolling, memory usage, real input, accessibility, DPI changes and macOS remain unverified.

## Reproduce in PowerShell

Install Rust through the official [rustup installer](https://rust-lang.org/tools/install/) and Microsoft's C++
Build Tools with the Windows SDK first. The commands below keep upstream source and generated artifacts outside
the Planetes checkout. Use a fresh directory if the temporary checkout already exists.

```powershell
$runeDir = Join-Path $env:TEMP 'planetes-rune-windows'
$outDir = Join-Path $env:TEMP 'planetes-rune-evidence'
$rev = '7256c7c2afb605862ccfe600a469df7330509efb'

git init $runeDir
git -C $runeDir remote add origin https://github.com/anomalyco/rune.git
git -C $runeDir fetch --depth 1 origin $rev
git -C $runeDir checkout --detach FETCH_HEAD
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

Push-Location $runeDir
try {
    cargo build --locked -p hello -p kit-gallery
    cargo test --locked -p hello

    & .\target\debug\hello.exe --scene on `
        --snap (Join-Path $outDir 'hello-on.png') `
        --tree (Join-Path $outDir 'hello-on.json')
    & .\target\debug\kit-gallery.exe --scene ui-button--variants `
        --snap (Join-Path $outDir 'gallery-buttons.png') `
        --tree (Join-Path $outDir 'gallery-buttons.json') --theme dark --size 720x300

    # These open native windows briefly and close automatically.
    $env:RUNE_GPU_LOG = '1'
    & .\target\debug\hello.exe --native `
        --trace (Join-Path $outDir 'hello-native.jsonl') --frames 120
    & .\target\debug\kit-gallery.exe --scene ui-button--variants --native `
        --trace (Join-Path $outDir 'gallery-native.jsonl') --frames 120
} finally {
    Remove-Item Env:RUNE_GPU_LOG -ErrorAction SilentlyContinue
    Pop-Location
}
```

Check each command's exit code before continuing. Generated snapshots, trees and traces are temporary evidence,
not repository assets. The application should pin the same Rune revision when its Cargo workspace is created.
