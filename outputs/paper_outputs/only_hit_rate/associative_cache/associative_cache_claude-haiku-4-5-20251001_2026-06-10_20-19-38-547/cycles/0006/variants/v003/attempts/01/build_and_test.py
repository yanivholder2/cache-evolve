#!/usr/bin/env python3
import subprocess
import shutil
import sys
import os

# Copy overlay files into the subject directory
overlay_file = "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-38-547/cycles/0006/variants/v003/attempts/01/overlay/src/associative_cache/mod.rs"
target_file = "/home/user/cache-evolve/inputs/subjects/associative_cache/src/associative_cache/mod.rs"

print("Copying overlay files...")
try:
    shutil.copy2(overlay_file, target_file)
    print(f"Successfully copied {overlay_file} to {target_file}")
except Exception as e:
    print(f"ERROR copying file: {e}", file=sys.stderr)
    sys.exit(1)

# Navigate to the subject directory and build
subject_dir = "/home/user/cache-evolve/inputs/subjects/associative_cache"
print(f"\nBuilding in {subject_dir}...")

try:
    result = subprocess.run(
        ["cargo", "build", "--tests", "--features", "test-utils"],
        cwd=subject_dir,
        capture_output=True,
        text=True,
        timeout=300
    )

    print("STDOUT:")
    print(result.stdout)

    if result.stderr:
        print("\nSTDERR:")
        print(result.stderr)

    if result.returncode != 0:
        print(f"\nBuild failed with return code {result.returncode}")
        sys.exit(1)
    else:
        print("\nBuild completed successfully!")

except subprocess.TimeoutExpired:
    print("Build timed out after 300 seconds")
    sys.exit(1)
except Exception as e:
    print(f"ERROR during build: {e}", file=sys.stderr)
    sys.exit(1)
