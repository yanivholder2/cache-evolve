#!/bin/bash

# Segment-based LRU cache variant build and test script
# This script copies overlay files and runs all test gates

set -e  # Exit on error

SUBJECT_DIR="/c/Users/yaniv/Projects/cache-evolve/inputs/subjects/associative_cache"
OVERLAY_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/overlay"
LOG_FILE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/build_test.log"

{
echo "============================================================"
echo "Segment-Based LRU Cache Variant Build and Test"
echo "============================================================"
echo "Timestamp: $(date)"
echo "Subject:  $SUBJECT_DIR"
echo "Overlay:  $OVERLAY_DIR"
echo ""

# Step 1: Copy overlay files
echo "[1/5] Copying overlay files..."
echo "      Copying: associative_cache_bucket.rs"
cp "$OVERLAY_DIR/src/associative_cache/associative_cache_bucket.rs" \
   "$SUBJECT_DIR/src/associative_cache/associative_cache_bucket.rs"

echo "      Copying: geometry.rs"
cp "$OVERLAY_DIR/src/shared/geometry.rs" \
   "$SUBJECT_DIR/src/shared/geometry.rs"
echo "      [SUCCESS] Overlay files copied"
echo ""

# Step 2: Build with tests
echo "[2/5] Building with test utilities..."
cd "$SUBJECT_DIR"
if cargo build --tests --features test-utils 2>&1; then
    echo "      [SUCCESS] Build completed"
else
    echo "      [FAILED] Build failed"
    exit 1
fi
echo ""

# Step 3: Run contract gate tests
echo "[3/5] Running contract_gate tests..."
if cargo test --test contract_gate 2>&1; then
    echo "      [SUCCESS] contract_gate tests passed"
else
    echo "      [FAILED] contract_gate tests failed"
    exit 1
fi
echo ""

# Step 4: Run universal invariant gate tests
echo "[4/5] Running universal_invariant_gate tests..."
if cargo test --test universal_invariant_gate 2>&1; then
    echo "      [SUCCESS] universal_invariant_gate tests passed"
else
    echo "      [FAILED] universal_invariant_gate tests failed"
    exit 1
fi
echo ""

# Step 5: Run invariant gate tests
echo "[5/5] Running invariant_gate tests (with test-utils feature)..."
if cargo test --test invariant_gate --features test-utils 2>&1; then
    echo "      [SUCCESS] invariant_gate tests passed"
else
    echo "      [FAILED] invariant_gate tests failed"
    exit 1
fi
echo ""

echo "============================================================"
echo "ALL TESTS PASSED SUCCESSFULLY"
echo "============================================================"
echo "Timestamp: $(date)"

} | tee "$LOG_FILE"

echo "Log saved to: $LOG_FILE"
