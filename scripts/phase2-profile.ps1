[CmdletBinding()]
param(
    [ValidateSet("all", "synthetic", "rural", "dense", "water", "stress", "stability")]
    [string]$Scenario = "all",
    [string]$Assets,
    [string]$Worldspace = "0x3c",
    [int]$RuralGridX = 0,
    [int]$RuralGridY = 0,
    [int]$DenseGridX = 0,
    [int]$DenseGridY = 0,
    [int]$WaterGridX = 0,
    [int]$WaterGridY = 0,
    [int]$Repetitions = 3,
    [int]$DurationSeconds = 120,
    [int]$StabilitySeconds = 1800,
    [string]$Baseline = "profiling-baseline.json",
    [switch]$UpdateBaseline,
    [switch]$Quick,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$repository = Split-Path -Parent $PSScriptRoot
$engine = Join-Path $repository "target\release\engine.exe"
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$cpu = (Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name) -replace '[^a-zA-Z0-9]+', '-'
$gpuInfo = Get-CimInstance Win32_VideoController | Select-Object -First 1
$gpu = $gpuInfo.Name -replace '[^a-zA-Z0-9]+', '-'
$driver = $gpuInfo.DriverVersion -replace '[^a-zA-Z0-9.]+', '-'
$hardware = "$cpu-$gpu-driver-$driver".Trim('-')
$campaign = Join-Path $repository "target\profiling\$stamp-$hardware"
$testTemp = Join-Path $repository "target\test-temp"
New-Item -ItemType Directory -Force -Path $campaign, $testTemp | Out-Null
$env:TEMP = $testTemp
$env:TMP = $testTemp

if ($Quick) {
    $DurationSeconds = 3
    $StabilitySeconds = 5
    $Repetitions = 1
}
if ($Repetitions -lt 1) { throw "Repetitions must be at least 1" }

Push-Location $repository
try {
    if (-not $SkipBuild -or -not (Test-Path -LiteralPath $engine -PathType Leaf)) {
        & cargo build --release -p engine -j1
        if ($LASTEXITCODE -ne 0) { throw "Release build failed with exit code $LASTEXITCODE" }
    }

    $commit = (& git rev-parse --short=12 HEAD 2>$null)
    if (-not $commit) { $commit = "unknown" }
    & git diff --quiet --ignore-submodules HEAD 2>$null
    $dirty = $LASTEXITCODE -ne 0
    $baselinePath = if ([IO.Path]::IsPathRooted($Baseline)) { $Baseline } else { Join-Path $repository $Baseline }
    $baselineData = if (Test-Path -LiteralPath $baselinePath) {
        Get-Content -LiteralPath $baselinePath -Raw | ConvertFrom-Json -AsHashtable
    } else { @{} }

    $resolvedAssets = $null
    if ($Assets) {
        $resolvedAssets = (Resolve-Path -LiteralPath $Assets).Path
        foreach ($required in @("conversion-manifest.json", "integration-report.json", "skyrim_world.db", "cell_cache.rkyv")) {
            if (-not (Test-Path -LiteralPath (Join-Path $resolvedAssets $required) -PathType Leaf)) {
                throw "Converted asset set is missing $required"
            }
        }
    }

    $requested = if ($Scenario -eq "all") {
        if ($resolvedAssets) { @("synthetic", "rural", "dense", "water", "stress", "stability") } else { @("synthetic") }
    } else { @($Scenario) }
    if ($requested.Where({ $_ -ne "synthetic" }).Count -gt 0 -and -not $resolvedAssets) {
        throw "Scenario '$Scenario' requires -Assets with a converted legal Skyrim asset set"
    }

    $results = @{}
    $regressionFailed = $false
    foreach ($name in $requested) {
        $runs = @()
        for ($run = 1; $run -le $Repetitions; $run++) {
            $runDirectory = Join-Path $campaign "$name\run-$run"
            New-Item -ItemType Directory -Force -Path $runDirectory | Out-Null
            $arguments = @(
                "--benchmark-duration", $(if ($name -eq "stability") { "$StabilitySeconds" } else { "$DurationSeconds" }),
                "--benchmark-output", (Join-Path $runDirectory "acceptance.json"),
                "--profile-output", $runDirectory,
                "--profile-scenario", $name,
                "--profile-run-id", "run-$run",
                "--profile-commit", $commit,
                "--profile-hardware", $hardware,
                "--accept-min-fps", "0",
                "--accept-p95-ms", "1000000",
                "--accept-max-memory-growth-gib", "1000000"
            )
            if ($dirty) { $arguments += "--profile-dirty-worktree" }
            if ($name -eq "synthetic") {
                $arguments += @("--benchmark-only", "--synthetic-instances", "250000")
            } else {
                $arguments += @("--assets", $resolvedAssets, "--worldspace", $Worldspace)
                if ($name -eq "water") {
                    $arguments += @("--grid-x", "$WaterGridX", "--grid-y", "$WaterGridY")
                } elseif ($name -in @("dense", "stability")) {
                    $arguments += @("--grid-x", "$DenseGridX", "--grid-y", "$DenseGridY")
                } else {
                    $arguments += @("--grid-x", "$RuralGridX", "--grid-y", "$RuralGridY")
                }
                if ($name -eq "stress") { $arguments += @("--stream-radius", "3", "--auto-fly-speed", "5000") }
                if ($name -eq "stability") { $arguments += @("--auto-fly-speed", "900") }
            }
            & $engine @arguments
            if ($LASTEXITCODE -ne 0) { throw "Profiling run $name/run-$run failed with exit code $LASTEXITCODE" }
            $runs += Get-Content -LiteralPath (Join-Path $runDirectory "frame-metrics.json") -Raw | ConvertFrom-Json
        }

        function Get-Median([double[]]$Values) {
            $ordered = @($Values | Sort-Object)
            $middle = [int][Math]::Floor($ordered.Count / 2)
            if ($ordered.Count % 2) { return $ordered[$middle] }
            return ($ordered[$middle - 1] + $ordered[$middle]) / 2.0
        }
        $median = [ordered]@{
            average_fps = Get-Median @($runs.average_fps)
            frame_ms_p95 = Get-Median @($runs.frame_ms_p95)
            frame_ms_p99 = Get-Median @($runs.frame_ms_p99)
            peak_process_memory_gib = Get-Median @($runs.peak_process_memory_gib)
            process_memory_growth_gib = Get-Median @($runs.process_memory_growth_gib)
        }
        $comparison = [ordered]@{ scenario = $name; repetitions = $Repetitions; median = $median; regressions = @() }
        if ($baselineData.ContainsKey($name)) {
            foreach ($metric in @("average_fps", "frame_ms_p95", "frame_ms_p99", "peak_process_memory_gib", "process_memory_growth_gib")) {
                $old = [double]$baselineData[$name][$metric]
                $new = [double]$median[$metric]
                if ($old -le 0) { continue }
                $regression = if ($metric -eq "average_fps") { ($old - $new) / $old * 100.0 } else { ($new - $old) / $old * 100.0 }
                $status = if ($regression -gt 10.0) { "fail" } elseif ($regression -gt 5.0) { "warn" } else { "pass" }
                if ($status -eq "fail") { $regressionFailed = $true }
                $comparison.regressions += [ordered]@{ metric = $metric; baseline = $old; current = $new; regression_percent = $regression; status = $status }
            }
        }
        $results[$name] = $median
        $comparison | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $campaign "$name\comparison.json")
    }

    [ordered]@{ format_version = 1; generated_at = (Get-Date).ToString("o"); commit = $commit; dirty = $dirty; hardware = $hardware; scenarios = $results } |
        ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $campaign "campaign.json")
    $summary = @(
        "# Phase 2 profiling campaign",
        "",
        "- Commit: $commit",
        "- Dirty worktree: $dirty",
        "- Hardware: $hardware",
        "- Repetitions: $Repetitions",
        "",
        "| Scenario | Median FPS | Frame P95 ms | Frame P99 ms | Peak GiB | Growth GiB |",
        "|---|---:|---:|---:|---:|---:|"
    )
    foreach ($name in $requested) {
        $value = $results[$name]
        $summary += "| $name | $([Math]::Round($value.average_fps, 2)) | $([Math]::Round($value.frame_ms_p95, 3)) | $([Math]::Round($value.frame_ms_p99, 3)) | $([Math]::Round($value.peak_process_memory_gib, 3)) | $([Math]::Round($value.process_memory_growth_gib, 3)) |"
    }
    $summary | Set-Content -LiteralPath (Join-Path $campaign "summary.md")
    if ($UpdateBaseline) {
        $results | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $baselinePath
        Write-Host "Baseline updated: $baselinePath"
    }
    if ($regressionFailed) { throw "Profiling regression exceeded 10%. See $campaign" }
    Write-Host "Profiling campaign complete: $campaign"
}
finally {
    Pop-Location
}
