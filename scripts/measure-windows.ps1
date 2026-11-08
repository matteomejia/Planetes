param([string]$Executable = 'target/release/planetes.exe', [ValidateRange(1,10)][int]$Runs = 5)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot
Push-Location $root
$previous = $env:PLANETES_BENCHMARK_OUTPUT
try {
    $exe = (Resolve-Path -LiteralPath $Executable).Path
    New-Item -ItemType Directory -Force -Path artifacts | Out-Null
    $results = @()
    for ($run=1; $run -le $Runs; $run++) {
        $output = Join-Path $root ('artifacts/benchmark-' + [guid]::NewGuid().ToString('N') + '.json')
        $env:PLANETES_BENCHMARK_OUTPUT = $output
        $process = Start-Process -FilePath $exe -PassThru -WindowStyle Hidden
        $deadline = [DateTime]::UtcNow.AddSeconds(25)
        $working = 0L; $private = 0L; $cpu = 0.0
        while (-not $process.HasExited) {
            if ([DateTime]::UtcNow -gt $deadline) { $process.Kill(); throw 'Benchmark exceeded 25 seconds.' }
            $process.Refresh()
            try {
                $working = [Math]::Max($working,$process.WorkingSet64)
                $private = $process.PrivateMemorySize64
                $cpu = $process.TotalProcessorTime.TotalMilliseconds
            } catch { if (-not $process.HasExited) { throw } }
            Start-Sleep -Milliseconds 100
        }
        if (-not (Test-Path -LiteralPath $output)) { throw "Benchmark failed with exit code $($process.ExitCode)." }
        $data = Get-Content -LiteralPath $output -Raw | ConvertFrom-Json
        $data | Add-Member run $run
        $data | Add-Member sampled_peak_working_mib ([Math]::Round($working/1MB,2))
        $data | Add-Member idle_private_mib ([Math]::Round($private/1MB,2))
        $data | Add-Member total_cpu_ms $cpu
        $results += $data
    }
    $results | ConvertTo-Json -Depth 5 | Set-Content -Encoding utf8 artifacts/benchmark-summary.json
    $results | Select-Object run,first_frame_cpu_ms,library_frame_cpu_ms,search_cpu_us_p50,idle_private_mib
} finally { $env:PLANETES_BENCHMARK_OUTPUT = $previous; Pop-Location }
