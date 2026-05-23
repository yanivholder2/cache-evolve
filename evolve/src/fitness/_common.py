"""Shared helpers for fitness plugin scripts."""

import os
import subprocess
import sys
from pathlib import Path

BUILD_TIMEOUT_S = 300


def cargo_env(target_dir: Path | None) -> dict[str, str] | None:
    """Returns an env overlay setting CARGO_TARGET_DIR, or None to inherit."""

    if target_dir is None:
        return None
    return {**os.environ, "CARGO_TARGET_DIR": str(target_dir)}


def cargo_build(
    cwd: Path,
    args: list[str],
    *,
    label: str,
    timeout: int = BUILD_TIMEOUT_S,
    target_dir: Path | None = None,
) -> None:
    """Runs cargo build; exits 1 with a contextual error on failure.

    Overriding CARGO_TARGET_DIR keeps parallel evolve runs off the same cargo lock.
    """

    result = subprocess.run(
        ["cargo", "build", *args],
        cwd=cwd,
        capture_output=True,
        text=True,
        timeout=timeout,
        env=cargo_env(target_dir),
    )
    if result.returncode != 0:
        print(f"error: cargo build ({label}) failed:\n{result.stderr}", file=sys.stderr)
        sys.exit(1)


def run_binary(cmd: list[str], *, label: str, timeout: int) -> subprocess.CompletedProcess:
    """Runs a subprocess; exits 1 with a contextual error on non-zero return or timeout."""

    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired as e:
        print(f"error: {label} timed out after {e.timeout}s", file=sys.stderr)
        sys.exit(1)
    if result.returncode != 0:
        print(f"error: {label} failed (exit {result.returncode}):\n{result.stderr}", file=sys.stderr)
        sys.exit(1)
    return result
