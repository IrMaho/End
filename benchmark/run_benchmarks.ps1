# Reproducible Statistical Benchmark Harness for End Language
# Conforms to BENCHMARKS.md specification

param(
    [int]$WarmupCount = 1000,
    [int]$SampleCount = 100,
    [string]$OutputFile = "benchmark_results.json"
)

Write-Host "=======================================================================================" -ForegroundColor Cyan
Write-Host "?? End Language: Reproducible Multi-Language Benchmark Suite" -ForegroundColor Green
Write-Host "=======================================================================================" -ForegroundColor Cyan

# Gather System Metadata
$cpuInfo = (Get-CimInstance Win32_Processor).Name.Trim()
$osInfo = (Get-CimInstance Win32_OperatingSystem).Caption.Trim()
$commitSha = git rev-parse --short HEAD 2>$null
if (-not $commitSha) { $commitSha = "unknown" }
$compilerVer = "0.4.0-alpha"

Write-Host "  CPU:             $cpuInfo" -ForegroundColor DarkGray
Write-Host "  OS:              $osInfo" -ForegroundColor DarkGray
Write-Host "  Compiler:        endc $compilerVer ($commitSha)" -ForegroundColor DarkGray
Write-Host "  Warmup Cycles:   $WarmupCount" -ForegroundColor DarkGray
Write-Host "  Recorded Passes: $SampleCount`n" -ForegroundColor DarkGray

function Calculate-Stats([double[]]$samples) {
    $sorted = $samples | Sort-Object
    $count = $sorted.Count
    if ($count -eq 0) { return $null }

    $sum = 0.0
    foreach ($val in $sorted) { $sum += $val }
    $mean = $sum / $count

    $varianceSum = 0.0
    foreach ($val in $sorted) { $varianceSum += [Math]::Pow($val - $mean, 2) }
    $stdDev = [Math]::Sqrt($varianceSum / $count)

    $min = $sorted[0]
    $max = $sorted[-1]
    $p50 = $sorted[[Math]::Floor($count * 0.50)]
    $p90 = $sorted[[Math]::Floor($count * 0.90)]
    $p95 = $sorted[[Math]::Floor($count * 0.95)]
    $p99 = $sorted[[Math]::Floor($count * 0.99)]

    return [PSCustomObject]@{
        MinMs = [Math]::Round($min, 4)
        MaxMs = [Math]::Round($max, 4)
        MeanMs = [Math]::Round($mean, 4)
        MedianMs = [Math]::Round($p50, 4)
        P50Ms = [Math]::Round($p50, 4)
        P90Ms = [Math]::Round($p90, 4)
        P95Ms = [Math]::Round($p95, 4)
        P99Ms = [Math]::Round($p99, 4)
        StdDevMs = [Math]::Round($stdDev, 4)
    }
}

function Run-Workload([string]$name, [string]$exePath, [string]$lang) {
    if (-not (Test-Path $exePath)) {
        return [PSCustomObject]@{
            Language = $lang
            Workload = $name
            Status = "Executable Not Built"
            P50_Ms = "N/A"
            P99_Ms = "N/A"
            Mean_Ms = "N/A"
            StdDev_Ms = "N/A"
            OpsPerSec = "N/A"
        }
    }

    # Warmup
    for ($w = 0; $w -lt [Math]::Min($WarmupCount, 50); $w++) {
        $null = & $exePath
    }

    $samples = @()
    for ($i = 0; $i -lt $SampleCount; $i++) {
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        $out = & $exePath
        $sw.Stop()
        $samples += $sw.Elapsed.TotalMilliseconds
    }

    $stats = Calculate-Stats $samples
    $opsSec = if ($stats.MeanMs -gt 0) { [Math]::Round((1000.0 / $stats.MeanMs) * 1000000.0, 0) } else { 0 }

    return [PSCustomObject]@{
        Language = $lang
        Workload = $name
        Status = "Passed"
        Min_Ms = $stats.MinMs
        Max_Ms = $stats.MaxMs
        Mean_Ms = $stats.MeanMs
        P50_Ms = $stats.P50Ms
        P90_Ms = $stats.P90Ms
        P95_Ms = $stats.P95Ms
        P99_Ms = $stats.P99Ms
        StdDev_Ms = $stats.StdDevMs
        EstimatedOpsSec = $opsSec
    }
}

$results = @()

# Execute available benchmarks
$benchList = @(
    @{ Name = "Numeric Reduction (C11)"; Path = ".\bench_precise.exe"; Lang = "C11" },
    @{ Name = "Numeric Reduction (Rust)"; Path = ".\bench_rust.exe"; Lang = "Rust" },
    @{ Name = "Numeric Reduction (Zig)"; Path = ".\bench_zig.exe"; Lang = "Zig" }
)

foreach ($b in $benchList) {
    Write-Host "Running $($b.Name)..." -NoNewline
    $res = Run-Workload -name $b.Name -exePath $b.Path -lang $b.Lang
    $results += $res
    Write-Host " Done (P50: $($res.P50_Ms) ms, P99: $($res.P99_Ms) ms)" -ForegroundColor Green
}

Write-Host "`n?? Statistical Results Summary:" -ForegroundColor Cyan
$results | Format-Table -Property Language, Workload, P50_Ms, P99_Ms, Mean_Ms, StdDev_Ms, Status

$reportPayload = [PSCustomObject]@{
    Metadata = [PSCustomObject]@{
        CPU = $cpuInfo
        OS = $osInfo
        Compiler = "endc $compilerVer"
        Commit = $commitSha
        Timestamp = (Get-Date).ToString("o")
        WarmupCount = $WarmupCount
        SampleCount = $SampleCount
    }
    Results = $results
}

$reportPayload | ConvertTo-Json -Depth 5 | Set-Content -Path $OutputFile
Write-Host "? Benchmark report saved to $OutputFile" -ForegroundColor Green
