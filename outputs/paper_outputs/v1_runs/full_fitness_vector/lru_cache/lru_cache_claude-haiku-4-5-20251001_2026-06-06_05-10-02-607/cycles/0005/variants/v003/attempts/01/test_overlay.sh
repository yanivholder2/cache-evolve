#!/bin/bash
set -e

# Create temp directory
TMPDIR=/tmp/lru_test_$$
rm -rf "$TMPDIR"
mkdir -p "$TMPDIR"

# Copy subject crate
cp -r /home/user/cache-evolve/inputs/subjects/lru_cache "$TMPDIR/lru_cache"
cd "$TMPDIR/lru_cache"

# Copy overlay to replace mod.rs
cp /home/user/cache-evolve/outputs/lru_cache_2026-06-06_05-10-02-607/cycles/0005/variants/v003/attempts/01/overlay/src/lru_cache/mod.rs src/lru_cache/mod.rs

echo "===== Test Setup Complete ====="
echo "Working directory: $PWD"
echo "Cargo.toml exists: $([ -f Cargo.toml ] && echo 'yes' || echo 'no')"
echo "Overlay applied to: src/lru_cache/mod.rs"
echo ""

# Test 1: cargo build --tests --features test-utils
echo "===== Test 1: cargo build --tests --features test-utils ====="
if cargo build --tests --features test-utils 2>&1; then
    echo "PASS: Compilation successful"
    TEST1_PASS=1
else
    echo "FAIL: Compilation failed"
    TEST1_PASS=0
fi
echo ""

# Test 2: cargo test --test contract_gate
echo "===== Test 2: cargo test --test contract_gate ====="
if cargo test --test contract_gate 2>&1; then
    echo "PASS: contract_gate test passed"
    TEST2_PASS=1
else
    echo "FAIL: contract_gate test failed"
    TEST2_PASS=0
fi
echo ""

# Test 3: cargo test --test universal_invariant_gate
echo "===== Test 3: cargo test --test universal_invariant_gate ====="
if cargo test --test universal_invariant_gate 2>&1; then
    echo "PASS: universal_invariant_gate test passed"
    TEST3_PASS=1
else
    echo "FAIL: universal_invariant_gate test failed"
    TEST3_PASS=0
fi
echo ""

# Test 4: cargo test --test invariant_gate --features test-utils
echo "===== Test 4: cargo test --test invariant_gate --features test-utils ====="
if cargo test --test invariant_gate --features test-utils 2>&1; then
    echo "PASS: invariant_gate test passed"
    TEST4_PASS=1
else
    echo "FAIL: invariant_gate test failed"
    TEST4_PASS=0
fi
echo ""

# Summary
echo "===== Summary ====="
echo "Test 1 (build --tests --features test-utils): $([ $TEST1_PASS -eq 1 ] && echo 'PASS' || echo 'FAIL')"
echo "Test 2 (contract_gate): $([ $TEST2_PASS -eq 1 ] && echo 'PASS' || echo 'FAIL')"
echo "Test 3 (universal_invariant_gate): $([ $TEST3_PASS -eq 1 ] && echo 'PASS' || echo 'FAIL')"
echo "Test 4 (invariant_gate --features test-utils): $([ $TEST4_PASS -eq 1 ] && echo 'PASS' || echo 'FAIL')"

if [ $TEST1_PASS -eq 1 ] && [ $TEST2_PASS -eq 1 ] && [ $TEST3_PASS -eq 1 ] && [ $TEST4_PASS -eq 1 ]; then
    echo ""
    echo "===== ALL TESTS PASSED ====="
    exit 0
else
    echo ""
    echo "===== SOME TESTS FAILED ====="
    exit 1
fi
