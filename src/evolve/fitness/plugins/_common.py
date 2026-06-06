"""Shared helpers for fitness plugin scripts."""

import subprocess
from pathlib import Path

from evolve.variant.build import cargo_env

BUILD_TIMEOUT_S = 300


class PluginError(RuntimeError):
    """Raised when a plugin's subprocess (cargo build, binary run) fails."""


def cargo_build(
    cwd: Path,
    args: list[str],
    *,
    label: str,
    timeout: int = BUILD_TIMEOUT_S,
    target_dir: Path | None = None,
) -> None:
    """Runs cargo build; raises `PluginError` on failure.

    Overriding CARGO_TARGET_DIR keeps parallel evolve runs off the same cargo lock.
    """

    try:
        result = subprocess.run(
            ["cargo", "build", *args],
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=timeout,
            env=cargo_env(target_dir),
        )
    except subprocess.TimeoutExpired as exception:
        raise PluginError(
            f"cargo build ({label}) timed out after {exception.timeout}s"
        ) from exception
    if result.returncode != 0:
        raise PluginError(f"cargo build ({label}) failed:\n{result.stderr}")


def run_binary(cmd: list[str], *, label: str, timeout: int) -> subprocess.CompletedProcess:
    """Runs a subprocess; raises `PluginError` on non-zero return or timeout."""

    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired as e:
        raise PluginError(f"{label} timed out after {e.timeout}s") from e
    if result.returncode != 0:
        raise PluginError(f"{label} failed (exit {result.returncode}):\n{result.stderr}")
    return result
