"""Caffeine simulator invocation core.

Library + CLI for invoking `gradlew simulator:simulate` against the
consolidated Dash policy (`dash`). Owns invocation only; parsing is each
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


DEFAULT_CAFFEINE_DIR = Path.home() / "projects" / "caffeine"

# Geometry defaults (used when caller does not override). Match the
# dash block in application.conf: 1 × (2+6) × 64 = 512.
DEFAULT_NUM_OF_NORMAL_BUCKETS = 2
DEFAULT_NUM_OF_STASH_BUCKETS = 6
DEFAULT_BUCKET_SIZE = 64

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


@dataclass(frozen=True)
class TargetSpec:
    """Per-target wiring: cdylib name, policy class, config-key prefix."""

    name: str
    cdylib_basename: str
    config_prefix: str
    policy_key: str          # config key holding the eviction-policy list (e.g. "evictionPolicies")
    policy_spec_name: str    # value for caffeine.simulator.policies.0


TARGETS: dict[str, TargetSpec] = {
    "dash": TargetSpec(
        name="dash",
        cdylib_basename="dash",
        config_prefix="caffeine.simulator.dash",
        policy_key="evictionPolicies",
        policy_spec_name="dash.DashRust",
    ),
}


@dataclass
class Invocation:
    """Single gradlew simulator:simulate invocation."""

    target: TargetSpec
    library_path: Path
    trace: str
    sizes: list[int]
    policies: list[str]
    output_dir: Path
    num_of_normal_buckets: int = DEFAULT_NUM_OF_NORMAL_BUCKETS
    num_of_stash_buckets: int = DEFAULT_NUM_OF_STASH_BUCKETS
    bucket_size: int = DEFAULT_BUCKET_SIZE
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
        prefix = self.target.config_prefix
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
            f"-Dcaffeine.simulator.policies.0={self.target.policy_spec_name}",
            f"-D{prefix}.numOfNormalBuckets={self.num_of_normal_buckets}",
            f"-D{prefix}.numOfStashBuckets={self.num_of_stash_buckets}",
            f"-D{prefix}.bucketSize={self.bucket_size}",
            f"-D{prefix}.debugMode={self.debug_mode}",
        ]
        for i, policy in enumerate(self.policies):
            cmd.append(f"-D{prefix}.{self.target.policy_key}.{i}={policy}")
        if self.log_path is not None:
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
    target: str,
    library_path: Path,
    trace: str,
    size: int,
    policies: list[str],
    timeout_s: int,
    output_dir: Path,
    num_of_normal_buckets: int = DEFAULT_NUM_OF_NORMAL_BUCKETS,
    num_of_stash_buckets: int = DEFAULT_NUM_OF_STASH_BUCKETS,
    bucket_size: int = DEFAULT_BUCKET_SIZE,
    debug_mode: int = 0,
    log_path: Path | None = None,
    caffeine_dir: Path = DEFAULT_CAFFEINE_DIR,
) -> RunResult:
    """Single trace × single size."""

    return simulate(
        target=target,
        library_path=library_path,
        trace=trace,
        sizes=[size],
        policies=policies,
        timeout_s=timeout_s,
        num_of_normal_buckets=num_of_normal_buckets,
        num_of_stash_buckets=num_of_stash_buckets,
        bucket_size=bucket_size,
        debug_mode=debug_mode,
        log_path=log_path,
        output_dir=output_dir,
        caffeine_dir=caffeine_dir,
    )


def simulate(
    *,
    target: str,
    library_path: Path,
    trace: str,
    sizes: list[int],
    policies: list[str],
    timeout_s: int,
    output_dir: Path,
    num_of_normal_buckets: int = DEFAULT_NUM_OF_NORMAL_BUCKETS,
    num_of_stash_buckets: int = DEFAULT_NUM_OF_STASH_BUCKETS,
    bucket_size: int = DEFAULT_BUCKET_SIZE,
    debug_mode: int = 0,
    log_path: Path | None = None,
    caffeine_dir: Path = DEFAULT_CAFFEINE_DIR,
) -> RunResult:
    """Single trace × multiple sizes."""

    if target not in TARGETS:
        raise CaffeineError(f"unknown target {target!r}; expected one of {sorted(TARGETS)}")
    if not sizes:
        raise CaffeineError("simulate(): sizes must be non-empty")
    if not policies:
        raise CaffeineError("simulate(): policies must be non-empty")
    invocation = Invocation(
        target=TARGETS[target],
        library_path=library_path,
        trace=trace,
        sizes=list(sizes),
        policies=list(policies),
        num_of_normal_buckets=num_of_normal_buckets,
        num_of_stash_buckets=num_of_stash_buckets,
        bucket_size=bucket_size,
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
        description="Invoke the Caffeine simulator with the Dash / DashVariant policy.",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    def add_common(p: argparse.ArgumentParser) -> None:
        p.add_argument("--target", choices=sorted(TARGETS), default="dash")
        p.add_argument("--library-path", type=Path, required=True,
                       help="Dir containing the cdylib (dash.dll / libdash.so / libdash.dylib).")
        p.add_argument("--trace", required=True, help="<format>:<path> (e.g. lirs:multi1.trace.gz)")
        p.add_argument("--policies", required=True,
                       help="Comma-separated eviction policies (e.g. classiclru,lfu).")
        p.add_argument("--normal-buckets", type=int, default=DEFAULT_NUM_OF_NORMAL_BUCKETS)
        p.add_argument("--stash-buckets", type=int, default=DEFAULT_NUM_OF_STASH_BUCKETS)
        p.add_argument("--bucket-size", type=int, default=DEFAULT_BUCKET_SIZE)
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


def _main(argv: list[str] | None = None) -> int:
    args = _parse_args(argv)
    policies = [p.strip() for p in args.policies.split(",") if p.strip()]
    common_kwargs = dict(
        target=args.target,
        library_path=args.library_path,
        trace=args.trace,
        policies=policies,
        timeout_s=args.timeout_s,
        num_of_normal_buckets=args.normal_buckets,
        num_of_stash_buckets=args.stash_buckets,
        bucket_size=args.bucket_size,
        debug_mode=args.debug_mode,
        log_path=args.log_path,
        output_dir=args.output_dir,
        caffeine_dir=args.caffeine_dir,
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
