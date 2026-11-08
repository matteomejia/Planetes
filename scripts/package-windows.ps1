param([string]$Version, [switch]$AllowDirty)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot
Push-Location $root
$previousFlags = $env:CARGO_ENCODED_RUSTFLAGS
try {
    if (-not $IsWindows) { throw 'Windows packaging requires Windows and the MSVC Rust toolchain.' }
    $metadataText = & cargo metadata --locked --format-version 1 --filter-platform x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw 'Cargo metadata failed.' }
    $metadata = $metadataText | ConvertFrom-Json
    $app = $metadata.packages | Where-Object name -eq 'planetes'
    if (-not $Version) { $Version = $app.version }
    if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw 'Version must be MAJOR.MINOR.PATCH.' }
    if ($app.version -ne $Version) { throw "Requested $Version but Cargo contains $($app.version)." }
    $hostInfo = & rustc -vV
    if ($hostInfo -notcontains 'host: x86_64-pc-windows-msvc') { throw 'Use the x86_64-pc-windows-msvc host toolchain.' }
    $dirty = @(& git status --porcelain).Count -gt 0
    if ($dirty -and -not $AllowDirty) { throw 'Commit source changes first (or use -AllowDirty for a local preview).' }
    $cargoDirectory = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE '.cargo' }
    $env:CARGO_ENCODED_RUSTFLAGS = @('-C','target-feature=+crt-static',"--remap-path-prefix=$env:USERPROFILE=user", "--remap-path-prefix=$root=.","--remap-path-prefix=$cargoDirectory=cargo") -join [char]31
    & cargo build --locked --release -p planetes
    if ($LASTEXITCODE -ne 0) { throw 'Release build failed.' }
    $name = "Planetes-v$Version-windows-x64"
    $dist = Join-Path $root 'dist'
    $stage = Join-Path $dist ('staging-' + [guid]::NewGuid().ToString('N'))
    $folder = Join-Path $stage $name
    New-Item -ItemType Directory -Path $folder -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $root 'target/release/planetes.exe') -Destination $folder
    Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $folder
    Copy-Item -LiteralPath (Join-Path $root 'packaging/README.txt') -Destination $folder
    & (Join-Path $PSScriptRoot 'third-party-notices.ps1') -Metadata $metadata -Destination (Join-Path $folder 'THIRD-PARTY-NOTICES.txt')
    $commit = (& git rev-parse HEAD).Trim()
    $info = @{ version = $Version; source_commit = $commit; source_dirty = $dirty; target = 'x86_64-pc-windows-msvc'; executable_sha256 = (Get-FileHash (Join-Path $folder 'planetes.exe') -Algorithm SHA256).Hash.ToLowerInvariant() }
    [IO.File]::WriteAllText((Join-Path $folder 'BUILD-INFO.json'), ($info | ConvertTo-Json), [Text.UTF8Encoding]::new($false))
    $archive = Join-Path $dist "$name.zip"
    Compress-Archive -LiteralPath $folder -DestinationPath $archive -CompressionLevel Optimal -Force
    $checksum = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    [IO.File]::WriteAllText("$archive.sha256", "$checksum  $name.zip`n", [Text.UTF8Encoding]::new($false))
    Write-Output $archive
    Write-Output "$archive.sha256"
} finally { $env:CARGO_ENCODED_RUSTFLAGS = $previousFlags; Pop-Location }
