# Execute from inside the Coverage folder. All paths relative to Coverage.
# Usage: cd crates\envs\ce-security\src\Coverage; .\run_coverage.ps1 [options]

param(
    [int]$Iterations = 200,

    [ValidateSet("new", "old", "compare")]
    [string]$Mode = "new",

    [string]$OutputDir = "",

    [uint64]$Seed = 0
)

$CoverageDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ResultsDir = Join-Path $CoverageDir "results"
$WorkspaceRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $CoverageDir))))

if ([string]::IsNullOrEmpty($OutputDir)) {
    $OutputDir = $ResultsDir
}

Set-Location $WorkspaceRoot

Write-Host "Building coverage_runner..." -ForegroundColor Cyan
cargo build -p ce-security --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed. Aborting."
    exit 1
}
Write-Host "Build succeeded." -ForegroundColor Green

$Binary = Join-Path $WorkspaceRoot "target\release\coverage_runner.exe"
if (-not $IsWindows) {
    $Binary = Join-Path $WorkspaceRoot "target/release/coverage_runner"
}

Write-Host "Running coverage_runner in $Mode mode (seed=$Seed) ..." -ForegroundColor Cyan
& $Binary `
    --iterations $Iterations `
    --mode $Mode `
    --output-dir $OutputDir `
    --seed $Seed

if ($LASTEXITCODE -ne 0) {
    Write-Error "coverage_runner exited with code $LASTEXITCODE."
    exit $LASTEXITCODE
}

Write-Host ""
Write-Host "Results saved to: $OutputDir" -ForegroundColor Green
