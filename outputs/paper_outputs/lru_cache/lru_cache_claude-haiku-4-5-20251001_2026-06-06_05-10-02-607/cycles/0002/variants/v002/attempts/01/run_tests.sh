#!/bin/bash
set -e

echo "=== Setting up LRU Cache test environment ==="
cd /tmp
rm -rf lru_cache_test
cp -r /home/user/cache-evolve/inputs/subjects/lru_cache lru_cache_test
cd lru_cache_test

echo "=== Applying overlay ==="
cp /home/user/cache-evolve/outputs/lru_cache_2026-06-06_05-10-02-607/cycles/0002/variants/v002/attempts/01/overlay/src/lru_cache/mod.rs src/lru_cache/mod.rs

echo "=== Build gate (cargo build --tests --features test-utils) ==="
if cargo build --tests --features test-utils 2>&1; then
    echo "✓ BUILD GATE PASSED"
    BUILD_RESULT="PASS"
else
    echo "✗ BUILD GATE FAILED"
    BUILD_RESULT="FAIL"
fi

echo ""
echo "=== Contract gate (cargo test --test contract_gate) ==="
if cargo test --test contract_gate 2>&1; then
    echo "✓ CONTRACT GATE PASSED"
    CONTRACT_RESULT="PASS"
else
    echo "✗ CONTRACT GATE FAILED"
    CONTRACT_RESULT="FAIL"
fi

echo ""
echo "=== Universal Invariant gate (cargo test --test universal_invariant_gate) ==="
if cargo test --test universal_invariant_gate 2>&1; then
    echo "✓ UNIVERSAL INVARIANT GATE PASSED"
    UNIVERSAL_RESULT="PASS"
else
    echo "✗ UNIVERSAL INVARIANT GATE FAILED"
    UNIVERSAL_RESULT="FAIL"
fi

echo ""
echo "=== Invariant gate (cargo test --test invariant_gate --features test-utils) ==="
if cargo test --test invariant_gate --features test-utils 2>&1; then
    echo "✓ INVARIANT GATE PASSED"
    INVARIANT_RESULT="PASS"
else
    echo "✗ INVARIANT GATE FAILED"
    INVARIANT_RESULT="FAIL"
fi

echo ""
echo "============================================"
echo "SUMMARY:"
echo "  Build gate: $BUILD_RESULT"
echo "  Contract gate: $CONTRACT_RESULT"
echo "  Universal invariant gate: $UNIVERSAL_RESULT"
echo "  Invariant gate: $INVARIANT_RESULT"
echo "============================================"
