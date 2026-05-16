"""Shared helpers for fitness plugin scripts."""

import subprocess
import sys
from pathlib import Path

BUILD_TIMEOUT_S = 300


def cargo_build(cwd: Path, args: list[str], *, label: str, timeout: int = BUILD_TIMEOUT_S) -> None:
    """Run `cargo build <args>` in `cwd`; exit 1 with a contextual error on failure."""
    result = subprocess.run(
        ["cargo", "build", *args],
        cwd=cwd,
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    if result.returncode != 0:
        print(f"error: cargo build ({label}) failed:\n{result.stderr}", file=sys.stderr)
        sys.exit(1)


def run_binary(cmd: list[str], *, label: str, timeout: int) -> subprocess.CompletedProcess:
    """Run `cmd`; exit 1 with a contextual error on non-zero return or timeout."""
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired as e:
        print(f"error: {label} timed out after {e.timeout}s", file=sys.stderr)
        sys.exit(1)
    if result.returncode != 0:
        print(f"error: {label} failed (exit {result.returncode}):\n{result.stderr}", file=sys.stderr)
        sys.exit(1)
    return result
