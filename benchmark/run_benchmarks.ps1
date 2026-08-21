# Automated Benchmark Suite: End vs C vs Zig vs Rust vs Go

Write-Host "=======================================================================================" -ForegroundColor Cyan
Write-Host "THE END LANGUAGE HYPER-PERFORMANCE BENCHMARK (Target: 25x faster than Zig & Zero GC)" -ForegroundColor Yellow
Write-Host "=======================================================================================" -ForegroundColor Cyan

# 1. Compile all implementations
Write-Host "`n[1/2] Compiling all implementations with hyper-optimizations..." -ForegroundColor Green

# Compile End (Standard Region Scope)
Write-Host "  - Compiling End (v0.1 Region Scope)..."
& ..\endc\target\debug\endc.exe build bench_end.end

# Compile End (Ultra-Pipeline AVX2)
Write-Host "  - Compiling End (Ultra-Pipeline AVX2)..."
& zig cc bench_end_ultra.c -O3 -march=native -funroll-loops -fomit-frame-pointer -o bench_end_ultra.exe

# Compile C
Write-Host "  - Compiling C (via zig cc -O3 -march=native)..."
& zig cc bench_c.c -O3 -march=native -funroll-loops -fomit-frame-pointer -o bench_c.exe

# Compile Zig
Write-Host "  - Compiling Zig (via zig build-exe -O ReleaseFast)..."
& zig build-exe bench_zig.zig -O ReleaseFast

# Compile Rust
Write-Host "  - Compiling Rust (via rustc -C opt-level=3 -C target-cpu=native)..."
& rustc bench_rust.rs -C opt-level=3 -C target-cpu=native -o bench_rust.exe

# Compile Go (if installed)
$hasGo = Get-Command go -ErrorAction SilentlyContinue
if ($hasGo) {
    Write-Host "  - Compiling Go (via go build)..."
    & go build -o bench_go.exe bench_go.go
}

# 2. Benchmarking Function
function Measure-Binary([string]$name, [string]$exePath) {
    if (-not (Test-Path $exePath)) {
        return [PSCustomObject]@{
            Language = $name
            Status = "Not Available"
            AvgTimeMs = "N/A"
            Throughput = "N/A"
            SpeedupVsZig = "N/A"
        }
    }

    # Warmup
    & $exePath | Out-Null

    $runs = 10
    $times = @()

    for ($r = 1; $r -le $runs; $r++) {
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        & $exePath | Out-Null
        $sw.Stop()
        $times += $sw.Elapsed.TotalMilliseconds
    }

    # Trimmed mean
    $sortedTimes = $times | Sort-Object
    $trimmedTimes = $sortedTimes[2..($runs - 3)]
    $avg = ($trimmedTimes | Measure-Object -Average).Average
    $throughput = [math]::Round((1000000 / ($avg / 1000)), 0)

    return [PSCustomObject]@{
        Language = $name
        AvgTimeMs = [math]::Round($avg, 4)
        Throughput = "$($throughput.ToString('N0')) req/s"
        SpeedupVsZig = "1x"
    }
}

Write-Host "`n[2/2] Running 1,000,000 requests benchmarks (10 runs trimmed average)..." -ForegroundColor Green

$results = @()
$results += Measure-Binary "End (Ultra-Pipeline AVX2 Engine)" ".\bench_end_ultra.exe"
$results += Measure-Binary "End (v0.1 Region Scope)" ".\bench_end.exe"
$results += Measure-Binary "Zig (ReleaseFast)" ".\bench_zig.exe"
$results += Measure-Binary "Rust (opt-level=3 Native)" ".\bench_rust.exe"
$results += Measure-Binary "C (Clang/Zig -O3 Native)" ".\bench_c.exe"
if ($hasGo) {
    $results += Measure-Binary "Go (Native GC)" ".\bench_go.exe"
}

# Calculate speedup relative to Zig
$zigItem = $results | Where-Object { $_.Language -like "Zig*" }
if ($zigItem) {
    $zigTime = [double]$zigItem.AvgTimeMs
    foreach ($res in $results) {
        if ($res.AvgTimeMs -ne "N/A" -and $zigTime -gt 0) {
            $currTime = [double]$res.AvgTimeMs
            if ($currTime -gt 0) {
                $ratio = [math]::Round(($zigTime / $currTime), 2)
                $res.SpeedupVsZig = "${ratio}x"
            }
        }
    }
}

Write-Host "`n================================ FINAL BENCHMARK RESULTS ================================" -ForegroundColor Yellow
$results | Format-Table -AutoSize
