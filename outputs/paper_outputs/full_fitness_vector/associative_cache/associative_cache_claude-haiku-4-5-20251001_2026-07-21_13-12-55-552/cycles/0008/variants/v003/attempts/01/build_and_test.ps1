# Build and test the cache variant

$ErrorActionPreference = "Stop"

# Paths
$subjectPath = "C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\associative_cache"
$overlayPath = "C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_13-12-55-552\cycles\0008\variants\v003\attempts\01\overlay"
$testDir = "$env:TEMP\associative_cache_variant_test"

Write-Host "Step 1: Cleaning and copying subject crate to temp location..."
if (Test-Path $testDir) {
    Remove-Item -Recurse -Force $testDir
}
Copy-Item -Recurse $subjectPath -Destination $testDir

Write-Host "Step 2: Applying overlay files..."
Copy-Item -Force "$overlayPath\src\associative_cache\mod.rs" "$testDir\src\associative_cache\mod.rs"
Copy-Item -Force "$overlayPath\src\associative_cache\associative_cache_bucket.rs" "$testDir\src\associative_cache\associative_cache_bucket.rs"

Write-Host "Step 3: Building with test features..."
Set-Location $testDir
cargo build --tests --features test-utils 2>&1 | Tee-Object -Variable buildOutput
if ($LASTEXITCODE -ne 0) {
    Write-Host "BUILD FAILED" -ForegroundColor Red
    exit 1
}

Write-Host "`nStep 4: Running contract_gate tests..."
cargo test --test contract_gate 2>&1 | Tee-Object -Variable contractOutput
$contractExit = $LASTEXITCODE

Write-Host "`nStep 5: Running universal_invariant_gate tests..."
cargo test --test universal_invariant_gate 2>&1 | Tee-Object -Variable universalOutput
$universalExit = $LASTEXITCODE

Write-Host "`nStep 6: Running invariant_gate tests..."
cargo test --test invariant_gate --features test-utils 2>&1 | Tee-Object -Variable invariantOutput
$invariantExit = $LASTEXITCODE

Write-Host "`n========== TEST SUMMARY ==========" -ForegroundColor Cyan
if ($contractExit -eq 0) {
    Write-Host "contract_gate: PASSED" -ForegroundColor Green
} else {
    Write-Host "contract_gate: FAILED" -ForegroundColor Red
}

if ($universalExit -eq 0) {
    Write-Host "universal_invariant_gate: PASSED" -ForegroundColor Green
} else {
    Write-Host "universal_invariant_gate: FAILED" -ForegroundColor Red
}

if ($invariantExit -eq 0) {
    Write-Host "invariant_gate: PASSED" -ForegroundColor Green
} else {
    Write-Host "invariant_gate: FAILED" -ForegroundColor Red
}

if ($contractExit -eq 0 -and $universalExit -eq 0 -and $invariantExit -eq 0) {
    Write-Host "`nAll tests PASSED!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "`nSome tests FAILED" -ForegroundColor Red
    exit 1
}
