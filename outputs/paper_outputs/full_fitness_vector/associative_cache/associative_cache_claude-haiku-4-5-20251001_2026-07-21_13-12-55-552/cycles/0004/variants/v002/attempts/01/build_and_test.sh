#!/bin/bash
set -e

SUBJECT_DIR="/c/Users/yaniv/Projects/cache-evolve/inputs/subjects/associative_cache"
OVERLAY_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/overlay"

echo "=== Segment-Based LRU Cache Variant Build and Test ==="
echo ""

# Step 1: Copy overlay files to subject
echo "Step 1: Copying overlay files to subject..."
echo "  From: $OVERLAY_DIR"
echo "  To:   $SUBJECT_DIR"

cp "$OVERLAY_DIR/src/associative_cache/associative_cache_bucket.rs" "$SUBJECT_DIR/src/associative_cache/associative_cache_bucket.rs"
echo "  [OK] Copied associative_cache_bucket.rs"

cp "$OVERLAY_DIR/src/shared/geometry.rs" "$SUBJECT_DIR/src/shared/geometry.rs"
echo "  [OK] Copied geometry.rs"

echo ""

# Step 2: Build with tests
echo "Step 2: Building with test utilities..."
cd "$SUBJECT_DIR"
cargo build --tests --features test-utils
echo "  [OK] Build succeeded"

echo ""

# Step 3: Run contract gate tests
echo "Step 3: Running contract_gate tests..."
cd "$SUBJECT_DIR"
cargo test --test contract_gate
echo "  [OK] contract_gate tests passed"

echo ""

# Step 4: Run universal invariant gate tests
echo "Step 4: Running universal_invariant_gate tests..."
cd "$SUBJECT_DIR"
cargo test --test universal_invariant_gate
echo "  [OK] universal_invariant_gate tests passed"

echo ""

# Step 5: Run invariant gate tests
echo "Step 5: Running invariant_gate tests..."
cd "$SUBJECT_DIR"
cargo test --test invariant_gate --features test-utils
echo "  [OK] invariant_gate tests passed"

echo ""
echo "=== All Tests Passed Successfully ==="
