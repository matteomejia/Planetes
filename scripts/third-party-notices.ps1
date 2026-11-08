param([Parameter(Mandatory)]$Metadata, [Parameter(Mandatory)][string]$Destination)
$ErrorActionPreference = 'Stop'
$packages = @{}; $nodes = @{}
foreach ($package in $Metadata.packages) { $packages[$package.id] = $package }
foreach ($node in $Metadata.resolve.nodes) { $nodes[$node.id] = $node }
$app = $Metadata.packages | Where-Object name -eq 'planetes'
$queue = [Collections.Generic.Queue[string]]::new(); $queue.Enqueue($app.id)
$seen = [Collections.Generic.HashSet[string]]::new()
while ($queue.Count) {
    $id = $queue.Dequeue()
    if (-not $seen.Add($id)) { continue }
    foreach ($dep in $nodes[$id].deps) {
        if (@($dep.dep_kinds | Where-Object kind -ne 'dev').Count) { $queue.Enqueue($dep.pkg) }
    }
}
$text = [Text.StringBuilder]::new()
[void]$text.AppendLine('Planetes third-party notices (runtime and build dependencies)')
$missing = @()
foreach ($package in ($seen | ForEach-Object { $packages[$_] } | Where-Object source | Sort-Object name,version)) {
    [void]$text.AppendLine("`n================================================================")
    [void]$text.AppendLine("$($package.name) $($package.version)")
    [void]$text.AppendLine("License: $($package.license)")
    [void]$text.AppendLine("Source: $($package.repository)")
    $directory = Split-Path $package.manifest_path
    $files = @(Get-ChildItem -LiteralPath $directory -File | Where-Object Name -Match '^(LICENSE|LICENCE|COPYING|COPYRIGHT|NOTICE)([._-]|$)')
    if ($package.license_file -and (Test-Path -LiteralPath $package.license_file)) { $files += Get-Item -LiteralPath $package.license_file }
    # Git workspace crates (Rune) inherit the repository license two levels above.
    if (-not $files.Count -and $package.source.StartsWith('git+')) {
        $parent = $directory
        for ($level=0; $level -lt 3 -and -not $files.Count; $level++) {
            $parent = Split-Path $parent
            $files = @(Get-ChildItem -LiteralPath $parent -File | Where-Object Name -Match '^(LICENSE|LICENCE|COPYING|NOTICE)([._-]|$)')
        }
    }
    foreach ($subdir in @('licenses','LICENSES','assets')) {
        $path = Join-Path $directory $subdir
        if (Test-Path -LiteralPath $path) {
            $files += Get-ChildItem -LiteralPath $path -Recurse -File | Where-Object Name -Match '(LICENSE|LICENCE|COPYING|NOTICE|OFL)'
        }
    }
    # A few published crates omit upstream license files. Reviewed copies are versioned here.
    if (-not $files.Count) {
        $override = Join-Path (Split-Path $PSScriptRoot) "packaging/licenses/$($package.name)"
        if (Test-Path -LiteralPath $override) {
            $source = Get-Content -LiteralPath (Join-Path $override 'SOURCE.txt') -Raw
            $identity = [regex]::Escape("$($package.name) $($package.version)")
            if ($source -notmatch "$identity(?:\r?\n|:|$)") { throw "Update the reviewed license copy for $($package.name) $($package.version)." }
            $files = @(Get-ChildItem -LiteralPath $override -File)
        }
    }
    if (-not $files.Count) { $missing += "$($package.name) $($package.version)"; continue }
    foreach ($file in ($files | Sort-Object FullName -Unique)) {
        [void]$text.AppendLine("`n--- $($file.Name) ---")
        [void]$text.AppendLine([IO.File]::ReadAllText($file.FullName))
    }
}
if ($missing.Count) { throw "License texts need review before packaging: $($missing -join ', ')" }
[IO.File]::WriteAllText($Destination, $text.ToString(), [Text.UTF8Encoding]::new($false))
