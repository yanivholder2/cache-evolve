#!/usr/bin/env python3
"""
Build and test the segment-based LRU cache variant implementation.

This script:
1. Copies overlay files to the subject directory
2. Runs cargo build with test utilities feature
3. Runs the contract_gate test
4. Runs the universal_invariant_gate test
5. Runs the invariant_gate test
"""

import os
import sys
import shutil
import subprocess
from pathlib import Path
from datetime import datetime

def run_command(cmd, cwd=None, description=""):
    """Run a shell command and capture output."""
    print(f"\n>>> {description}")
    print(f"    Command: {' '.join(cmd)}")

    try:
        result = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=False,
            text=True,
            check=False
        )
        return result.returncode == 0
    except Exception as e:
        print(f"ERROR: Failed to run command: {e}")
        return False

def main():
    # Define paths
    script_dir = Path(__file__).parent.absolute()
    subject_dir = Path("C:/Users/yaniv/Projects/cache-evolve/inputs/subjects/associative_cache")
    overlay_dir = script_dir / "overlay"

    print("=" * 70)
    print("Segment-Based LRU Cache Variant Build and Test")
    print("=" * 70)
    print(f"Timestamp: {datetime.now().isoformat()}")
    print(f"Subject:   {subject_dir}")
    print(f"Overlay:   {overlay_dir}")
    print()

    # Step 1: Copy overlay files
    print("[1/5] Copying overlay files to subject...")

    bucket_src = overlay_dir / "src" / "associative_cache" / "associative_cache_bucket.rs"
    bucket_dst = subject_dir / "src" / "associative_cache" / "associative_cache_bucket.rs"

    if bucket_src.exists():
        shutil.copy2(bucket_src, bucket_dst)
        print(f"  [OK] Copied {bucket_src.name}")
    else:
        print(f"  [ERROR] File not found: {bucket_src}")
        return False

    geom_src = overlay_dir / "src" / "shared" / "geometry.rs"
    geom_dst = subject_dir / "src" / "shared" / "geometry.rs"

    if geom_src.exists():
        shutil.copy2(geom_src, geom_dst)
        print(f"  [OK] Copied {geom_src.name}")
    else:
        print(f"  [ERROR] File not found: {geom_src}")
        return False

    print()

    # Step 2: Build with tests
    print("[2/5] Building with test utilities...")
    if not run_command(
        ["cargo", "build", "--tests", "--features", "test-utils"],
        cwd=subject_dir,
        description="cargo build --tests --features test-utils"
    ):
        print("  [FAILED] Build failed")
        return False
    print("  [OK] Build succeeded")

    print()

    # Step 3: Run contract_gate tests
    print("[3/5] Running contract_gate tests...")
    if not run_command(
        ["cargo", "test", "--test", "contract_gate"],
        cwd=subject_dir,
        description="cargo test --test contract_gate"
    ):
        print("  [FAILED] contract_gate tests failed")
        return False
    print("  [OK] contract_gate tests passed")

    print()

    # Step 4: Run universal_invariant_gate tests
    print("[4/5] Running universal_invariant_gate tests...")
    if not run_command(
        ["cargo", "test", "--test", "universal_invariant_gate"],
        cwd=subject_dir,
        description="cargo test --test universal_invariant_gate"
    ):
        print("  [FAILED] universal_invariant_gate tests failed")
        return False
    print("  [OK] universal_invariant_gate tests passed")

    print()

    # Step 5: Run invariant_gate tests
    print("[5/5] Running invariant_gate tests...")
    if not run_command(
        ["cargo", "test", "--test", "invariant_gate", "--features", "test-utils"],
        cwd=subject_dir,
        description="cargo test --test invariant_gate --features test-utils"
    ):
        print("  [FAILED] invariant_gate tests failed")
        return False
    print("  [OK] invariant_gate tests passed")

    print()
    print("=" * 70)
    print("ALL TESTS PASSED SUCCESSFULLY")
    print("=" * 70)
    print(f"Timestamp: {datetime.now().isoformat()}")

    return True

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
