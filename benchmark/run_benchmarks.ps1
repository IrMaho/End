# Automated Benchmark Runner for End, C, Zig, Rust, and Go

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "👑 THE END LANGUAGE BENCHMARK SUITE (vs C, Zig, Rust, Go)" -ForegroundColor Yellow
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Compile all implementations
Write-Host "`n[1/2] Compiling implementations with -O3 / ReleaseFast optimizations..." -ForegroundColor Green

# Compile End
Write-Host "  - Compiling End (via endc + native backend)..."
& ..\endc\target\debug\endc.exe build bench_end.end

# Compile C
Write-Host "  - Compiling C (via zig cc -O3)..."
& zig cc bench_c.c -O3 -o bench_c.exe

# Compile Zig
Write-Host "  - Compiling Zig (via zig build-exe -O ReleaseFast)..."
& zig build-exe bench_zig.zig -O ReleaseFast

# Compile Rust
Write-Host "  - Compiling Rust (via rustc -C opt-level=3)..."
& rustc bench_rust.rs -C opt-level=3 -o bench_rust.exe

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
        }
    }

    # Warmup
    & $exePath | Out-Null

    $runs = 5
    $times = @()

    for ($r = 1; $r -le $runs; $r++) {
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        & $exePath | Out-Null
        $sw.Stop()
        $times += $sw.Elapsed.TotalMilliseconds
    }

    $avg = ($times | Measure-Object -Average).Average
    $throughput = [math]::Round((1000000 / ($avg / 1000)), 0)

    return [PSCustomObject]@{
        Language = $name
        AvgTimeMs = [math]::Round($avg, 2)
        Throughput = "$($throughput.ToString('N0')) req/s"
        Runs = "$runs runs"
    }
}

Write-Host "`n[2/2] Running 1,000,000 requests benchmarks (5 runs average)..." -ForegroundColor Green

$results = @()
$results += Measure-Binary "End (v0.1 Region Scope)" ".\bench_end.exe"
$results += Measure-Binary "C (Clang/Zig -O3)" ".\bench_c.exe"
$results += Measure-Binary "Zig (ReleaseFast)" ".\bench_zig.exe"
$results += Measure-Binary "Rust (opt-level=3)" ".\bench_rust.exe"
if ($hasGo) {
    $results += Measure-Binary "Go (Native GC)" ".\bench_go.exe"
}

Write-Host "`n======================= FINAL RESULTS =======================" -ForegroundColor Yellow
$results | Format-Table -AutoSize
