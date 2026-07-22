# Build and test the segment-based LRU cache variant
# This script copies overlay files and runs the test gates

$ErrorActionPreference = "Stop"

$SUBJECT_DIR = "C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\associative_cache"
$OVERLAY_DIR = "$PSScriptRoot\overlay"

Write-Host "=== Segment-Based LRU Cache Variant Build and Test ===" -ForegroundColor Cyan
Write-Host ""

# Step 1: Copy overlay files to subject
Write-Host "Step 1: Copying overlay files to subject..." -ForegroundColor Yellow
Write-Host "  From: $OVERLAY_DIR"
Write-Host "  To:   $SUBJECT_DIR"

# Copy associative_cache_bucket.rs
$src_bucket = "$OVERLAY_DIR\src\associative_cache\associative_cache_bucket.rs"
$dest_bucket = "$SUBJECT_DIR\src\associative_cache\associative_cache_bucket.rs"
if (Test-Path $src_bucket) {
    Copy-Item $src_bucket $dest_bucket -Force
    Write-Host "  [OK] Copied associative_cache_bucket.rs"
} else {
    Write-Error "File not found: $src_bucket"
}

# Copy geometry.rs
$src_geom = "$OVERLAY_DIR\src\shared\geometry.rs"
$dest_geom = "$SUBJECT_DIR\src\shared\geometry.rs"
if (Test-Path $src_geom) {
    Copy-Item $src_geom $dest_geom -Force
    Write-Host "  [OK] Copied geometry.rs"
} else {
    Write-Error "File not found: $src_geom"
}

Write-Host ""

# Step 2: Build with tests
Write-Host "Step 2: Building with test utilities..." -ForegroundColor Yellow
Push-Location $SUBJECT_DIR
try {
    cargo build --tests --features test-utils 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Build failed with exit code $LASTEXITCODE"
    }
    Write-Host "  [OK] Build succeeded" -ForegroundColor Green
} finally {
    Pop-Location
}

Write-Host ""

# Step 3: Run contract gate tests
Write-Host "Step 3: Running contract_gate tests..." -ForegroundColor Yellow
Push-Location $SUBJECT_DIR
try {
    cargo test --test contract_gate 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Error "contract_gate tests failed with exit code $LASTEXITCODE"
    }
    Write-Host "  [OK] contract_gate tests passed" -ForegroundColor Green
} finally {
    Pop-Location
}

Write-Host ""

# Step 4: Run universal invariant gate tests
Write-Host "Step 4: Running universal_invariant_gate tests..." -ForegroundColor Yellow
Push-Location $SUBJECT_DIR
try {
    cargo test --test universal_invariant_gate 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Error "universal_invariant_gate tests failed with exit code $LASTEXITCODE"
    }
    Write-Host "  [OK] universal_invariant_gate tests passed" -ForegroundColor Green
} finally {
    Pop-Location
}

Write-Host ""

# Step 5: Run invariant gate tests
Write-Host "Step 5: Running invariant_gate tests..." -ForegroundColor Yellow
Push-Location $SUBJECT_DIR
try {
    cargo test --test invariant_gate --features test-utils 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Error "invariant_gate tests failed with exit code $LASTEXITCODE"
    }
    Write-Host "  [OK] invariant_gate tests passed" -ForegroundColor Green
} finally {
    Pop-Location
}

Write-Host ""
Write-Host "=== All Tests Passed Successfully ===" -ForegroundColor Green
