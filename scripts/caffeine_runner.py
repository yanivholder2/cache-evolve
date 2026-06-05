"""Caffeine simulator invocation core.

Library + CLI for invoking `gradlew simulator:simulate` against a subject's
Caffeine policy. Subject-agnostic: the per-subject wiring (config prefix,
policy spec name, geometry, policy-list key) is passed in by the caller, which
reads it from the subject manifest. Owns invocation only; parsing is each
caller's concern.

See docs/caffeine-simulator.md for the full invocation reference.
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path


def _default_caffeine_dir() -> Path:
    """Locate the Caffeine checkout without hardcoding an absolute path.

    Resolution order:
      1. The ``CAFFEINE_DIR`` environment variable, if set (``~`` expanded).
      2. A ``caffeine/`` sibling of the cache-evolve repo root.

    The sibling default keeps the path anchored to the repo rather than to a
    fixed home directory: it resolves to ``~/projects/caffeine`` for the local
    ``~/projects/cache-evolve`` layout and to ``<container>/caffeine`` in a
    remote session where the repo lives at ``<container>/cache-evolve``.
    """

    override = os.environ.get("CAFFEINE_DIR")
    if override:
        return Path(override).expanduser()
    repo_root = Path(__file__).resolve().parents[1]
    return repo_root.parent / "caffeine"


DEFAULT_CAFFEINE_DIR = _default_caffeine_dir()

STDERR_TAIL_LINES = 40
DEFAULT_TIMEOUT_S = 600


class CaffeineError(RuntimeError):
    """Raised when a Caffeine invocation fails (non-zero exit or timeout)."""


@dataclass(frozen=True)
class RunResult:
    """Outcome of a single Caffeine invocation."""

    output_dir: Path
    stdout: str
    stderr: str


@dataclass
class Invocation:
    """Single gradlew simulator:simulate invocation.

    The `config_prefix`, `policy_spec_name`, `geometry`, `policy_key`, and
    `supports_log_path` fields are the subject's Caffeine wiring (read from its
    manifest by the caller). `geometry` maps each config-key suffix (under
    `config_prefix`) to its value; an empty map means the subject is sized
    solely by `--maximumSize`. A scalar value is emitted as `key=value`; a list
    value is emitted in indexed `key.0=…` form, matching a Java `get*List` read.
    `policy_key` is the config key holding the eviction-policy list, or None
    when the subject declares no policy list.
    """

    library_path: Path
    trace: str
    sizes: list[int]
    policies: list[str]
    output_dir: Path
    config_prefix: str
    policy_spec_name: str
    geometry: dict[str, int | list[int]]
    policy_key: str | None
    supports_log_path: bool
    debug_mode: int = 0
    log_path: Path | None = None
    title: str | None = None
    caffeine_dir: Path = field(default_factory=lambda: DEFAULT_CAFFEINE_DIR)
    extra_jvm_args: tuple[str, ...] = ()

    def gradle_cmd(self) -> list[str]:
        gradlew = "gradlew.bat" if os.name == "nt" else "gradlew"
        sizes_str = ",".join(f"{s:,}".replace(",", "_") for s in self.sizes)
        title = self.title or _trace_display_name(self.trace)
        jvm_args = ",".join((
            "-XX:+UseParallelGC",
            "-Xmx8g",
            f"-Djava.library.path={self.library_path}",
            *self.extra_jvm_args,
        ))
        prefix = self.config_prefix
        cmd: list[str] = [
            str(self.caffeine_dir / gradlew),
            "-p", str(self.caffeine_dir),
            "simulator:simulate", "-q",
            f"--maximumSize={sizes_str}",
            f"--title={title}",
            f"--outputDir={self.output_dir}",
            f"-PjvmArgs={jvm_args}",
            "-Dcaffeine.simulator.trace.source=files",
            f"-Dcaffeine.simulator.files.paths.0={self.trace}",
            f"-Dcaffeine.simulator.policies.0={self.policy_spec_name}",
        ]
        for config_key, value in self.geometry.items():
            if isinstance(value, list):
                for i, item in enumerate(value):
                    cmd.append(f"-D{prefix}.{config_key}.{i}={item}")
            else:
                cmd.append(f"-D{prefix}.{config_key}={value}")
        cmd.append(f"-D{prefix}.debugMode={self.debug_mode}")
        if self.policy_key is not None:
            for i, policy in enumerate(self.policies):
                cmd.append(f"-D{prefix}.{self.policy_key}.{i}={policy}")
        if self.supports_log_path and self.log_path is not None:
            cmd.append(f"-D{prefix}.logPath={self.log_path}")
        return cmd


def _trace_display_name(trace: str) -> str:
    """`lirs:multi1.trace.gz` -> `multi1_lirs`."""
    fmt, _, filename = trace.partition(":")
    basename = os.path.basename(filename or fmt)
    name = basename.split(".")[0]
    return f"{name}_{fmt}" if filename else name


def _invoke(invocation: Invocation, *, timeout_s: int) -> RunResult:
    cmd = invocation.gradle_cmd()
    try:
        proc = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout_s,
        )
    except subprocess.TimeoutExpired as e:
        raise CaffeineError(
            f"caffeine simulator timed out after {e.timeout}s (trace={invocation.trace}, "
            f"sizes={invocation.sizes}, policies={invocation.policies})"
        ) from e
    if proc.returncode != 0:
        stderr_tail = "\n".join(proc.stderr.splitlines()[-STDERR_TAIL_LINES:])
        raise CaffeineError(
            f"caffeine simulator failed (exit {proc.returncode}, trace={invocation.trace}):\n"
            f"{stderr_tail}"
        )
    return RunResult(
        output_dir=invocation.output_dir,
        stdout=proc.stdout,
        stderr=proc.stderr,
    )


def run(
    *,
    library_path: Path,
    trace: str,
    size: int,
    policies: list[str],
    timeout_s: int,
    output_dir: Path,
    config_prefix: str,
    policy_spec_name: str,
    geometry: dict[str, int | list[int]],
    policy_key: str | None,
    supports_log_path: bool,
    debug_mode: int = 0,
    log_path: Path | None = None,
    caffeine_dir: Path = DEFAULT_CAFFEINE_DIR,
) -> RunResult:
    """Single trace × single size."""

    return simulate(
        library_path=library_path,
        trace=trace,
        sizes=[size],
        policies=policies,
        timeout_s=timeout_s,
        config_prefix=config_prefix,
        policy_spec_name=policy_spec_name,
        geometry=geometry,
        policy_key=policy_key,
        supports_log_path=supports_log_path,
        debug_mode=debug_mode,
        log_path=log_path,
        output_dir=output_dir,
        caffeine_dir=caffeine_dir,
    )


def simulate(
    *,
    library_path: Path,
    trace: str,
    sizes: list[int],
    policies: list[str],
    timeout_s: int,
    output_dir: Path,
    config_prefix: str,
    policy_spec_name: str,
    geometry: dict[str, int | list[int]],
    policy_key: str | None,
    supports_log_path: bool,
    debug_mode: int = 0,
    log_path: Path | None = None,
    caffeine_dir: Path = DEFAULT_CAFFEINE_DIR,
) -> RunResult:
    """Single trace × multiple sizes."""

    if not sizes:
        raise CaffeineError("simulate(): sizes must be non-empty")
    if not policies:
        raise CaffeineError("simulate(): policies must be non-empty")
    invocation = Invocation(
        library_path=library_path,
        trace=trace,
        sizes=list(sizes),
        policies=list(policies),
        config_prefix=config_prefix,
        policy_spec_name=policy_spec_name,
        geometry=dict(geometry),
        policy_key=policy_key,
        supports_log_path=supports_log_path,
        debug_mode=debug_mode,
        log_path=log_path,
        output_dir=output_dir,
        caffeine_dir=caffeine_dir,
    )
    return _invoke(invocation, timeout_s=timeout_s)


# ---------------------------------------------------------------------------
# CLI

def _parse_args(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="caffeine_runner.py",
        description="Invoke the Caffeine simulator against a subject's policy.",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    def add_common(p: argparse.ArgumentParser) -> None:
        p.add_argument("--subject-dir", type=Path, required=True,
                       help="Subject directory containing manifest.toml (source of the Caffeine wiring).")
        p.add_argument("--library-path", type=Path, required=True,
                       help="Dir containing the subject's cdylib.")
        p.add_argument("--trace", required=True, help="<format>:<path> (e.g. lirs:multi1.trace.gz)")
        p.add_argument("--policies", required=True,
                       help="Comma-separated eviction policies (e.g. classiclru,lfu).")
        p.add_argument("--debug-mode", type=int, default=0)
        p.add_argument("--log-path", type=Path, default=None,
                       help="Override the cdylib's rust-logs dir.")
        p.add_argument("--output-dir", type=Path, required=True,
                       help="Caffeine reports dir (passed as --outputDir to simulate).")
        p.add_argument("--caffeine-dir", type=Path, default=DEFAULT_CAFFEINE_DIR)
        p.add_argument("--timeout-s", type=int, default=DEFAULT_TIMEOUT_S)

    run_p = sub.add_parser("run", help="Single trace × single size.")
    add_common(run_p)
    run_p.add_argument("--size", type=int, required=True)

    sim_p = sub.add_parser("simulate", help="Single trace × multiple sizes.")
    add_common(sim_p)
    sim_p.add_argument("--sizes", required=True,
                       help="Comma-separated cache sizes (e.g. 512,1024,4096).")

    return parser.parse_args(argv)


def _load_caffeine_kwargs(subject_dir: Path) -> dict:
    """Read the subject manifest's Caffeine wiring as `simulate()` kwargs."""

    sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))
    from evolve.subject import load_subject  # noqa: E402

    return load_subject(subject_dir).caffeine.as_runner_kwargs()


def _main(argv: list[str] | None = None) -> int:
    args = _parse_args(argv)
    policies = [p.strip() for p in args.policies.split(",") if p.strip()]
    common_kwargs = dict(
        library_path=args.library_path,
        trace=args.trace,
        policies=policies,
        timeout_s=args.timeout_s,
        debug_mode=args.debug_mode,
        log_path=args.log_path,
        output_dir=args.output_dir,
        caffeine_dir=args.caffeine_dir,
        **_load_caffeine_kwargs(args.subject_dir),
    )
    try:
        if args.command == "run":
            result = run(size=args.size, **common_kwargs)
        else:
            sizes = [int(s) for s in args.sizes.split(",")]
            result = simulate(sizes=sizes, **common_kwargs)
    except CaffeineError as exception:
        print(f"error: {exception}", file=sys.stderr)
        return 1
    print(f"output_dir={result.output_dir}")
    return 0


if __name__ == "__main__":
    sys.exit(_main())
