#!/bin/bash
set -e

# Copy overlay files into the subject directory
echo "Copying overlay files..."
cp -f /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-38-547/cycles/0006/variants/v003/attempts/01/overlay/src/associative_cache/mod.rs /home/user/cache-evolve/inputs/subjects/associative_cache/src/associative_cache/mod.rs

# Navigate to the subject directory
cd /home/user/cache-evolve/inputs/subjects/associative_cache

# Build with tests
echo "Building with cargo build --tests --features test-utils..."
cargo build --tests --features test-utils 2>&1

echo "Build completed successfully!"
