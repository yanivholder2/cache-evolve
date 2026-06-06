#!/usr/bin/env python3
"""
Test runner for LRU Cache LFU variant
Copies the subject, applies overlay, and runs all test gates
"""

import subprocess
import sys
import shutil
from pathlib import Path
import tempfile

def run_command(cmd, description, cwd=None):
    """Run a shell command and return success/failure status"""
    print(f"\n{'='*70}")
    print(f"{description}")
    print(f"{'='*70}")
    print(f"Command: {' '.join(cmd)}")
    print()

    try:
        result = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=False,
            text=True,
            timeout=600  # 10 minute timeout
        )
        success = result.returncode == 0
        status = "✓ PASSED" if success else "✗ FAILED"
        print(f"\n{status}")
        return success
    except subprocess.TimeoutExpired:
        print(f"\n✗ TIMEOUT (10 minutes)")
        return False
    except Exception as e:
        print(f"\n✗ ERROR: {e}")
        return False

def main():
    print("""
╔════════════════════════════════════════════════════════════════════╗
║          LRU Cache LFU Variant Test Suite                         ║
╚════════════════════════════════════════════════════════════════════╝
    """)

    # Setup paths
    subject_path = Path("/home/user/cache-evolve/inputs/subjects/lru_cache")
    overlay_path = Path("/home/user/cache-evolve/outputs/lru_cache_2026-06-06_05-10-02-607/cycles/0002/variants/v002/attempts/01/overlay/src/lru_cache/mod.rs")

    # Create temp test directory
    test_dir = Path("/tmp/lru_cache_test")
    if test_dir.exists():
        shutil.rmtree(test_dir)

    print(f"Setting up test environment...")
    print(f"  Subject: {subject_path}")
    print(f"  Overlay: {overlay_path}")
    print(f"  Test dir: {test_dir}")

    # Copy subject
    shutil.copytree(subject_path, test_dir)

    # Apply overlay
    overlay_dest = test_dir / "src/lru_cache/mod.rs"
    shutil.copy(overlay_path, overlay_dest)
    print(f"✓ Setup complete\n")

    # Track results
    results = {}

    # Test 1: Build gate
    results["build"] = run_command(
        ["cargo", "build", "--tests", "--features", "test-utils"],
        "TEST 1: Build Gate (cargo build --tests --features test-utils)",
        cwd=test_dir
    )

    # Test 2: Contract gate
    results["contract"] = run_command(
        ["cargo", "test", "--test", "contract_gate"],
        "TEST 2: Contract Gate (cargo test --test contract_gate)",
        cwd=test_dir
    )

    # Test 3: Universal Invariant gate
    results["universal_invariant"] = run_command(
        ["cargo", "test", "--test", "universal_invariant_gate"],
        "TEST 3: Universal Invariant Gate (cargo test --test universal_invariant_gate)",
        cwd=test_dir
    )

    # Test 4: Invariant gate
    results["invariant"] = run_command(
        ["cargo", "test", "--test", "invariant_gate", "--features", "test-utils"],
        "TEST 4: Invariant Gate (cargo test --test invariant_gate --features test-utils)",
        cwd=test_dir
    )

    # Summary
    print(f"\n{'='*70}")
    print("SUMMARY")
    print(f"{'='*70}")

    all_passed = True
    for gate, passed in results.items():
        status = "PASS" if passed else "FAIL"
        symbol = "✓" if passed else "✗"
        print(f"  {symbol} {gate.upper():25} {status}")
        if not passed:
            all_passed = False

    print(f"{'='*70}\n")

    if all_passed:
        print("All tests PASSED!")
        return 0
    else:
        print("Some tests FAILED!")
        return 1

if __name__ == "__main__":
    sys.exit(main())
