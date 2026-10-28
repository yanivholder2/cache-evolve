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
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]


def _resolve_trace(trace: str) -> str:
    """Anchor the path part of a ``<format>:<path>`` trace to the repo root.

    A relative path is resolved against the repo root so manifests stay
    portable across machines (mirrors :func:`_default_caffeine_dir` rather than
    hardcoding an absolute home directory); an absolute path is left untouched.
    The ``<format>`` prefix (e.g. ``lirs``) is preserved.
    """

    fmt, sep, path = trace.partition(":")
    if not sep or not path:
        return trace
    resolved = Path(path).expanduser()
    if not resolved.is_absolute():
        resolved = REPO_ROOT / resolved
    return f"{fmt}:{resolved}"


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
    return REPO_ROOT.parent / "caffeine"


DEFAULT_CAFFEINE_DIR = _default_caffeine_dir()

STDERR_TAIL_LINES = 40
DEFAULT_TIMEOUT_S = 600


class CaffeineError(RuntimeError):
    """Raised when a Caffeine invocation fails: non-zero exit, timeout, or a
    zero exit that left an expected per-size CSV unwritten (Caffeine/picocli
    can print a policy-constructor exception and still exit 0)."""


_ANSI_ESCAPE = re.compile(r"\x1b\[[0-9;]*m")
# A Java throwable header line, e.g. `java.lang.IllegalArgumentException: msg`
# or `Caused by: java.lang.RuntimeException: msg` (stack-frame `at ...` lines excluded).
_THROWABLE_LINE = re.compile(r"^(Caused by: )?[\w$.]+(Exception|Error)(: .*)?$")


def _root_cause(stderr: str) -> str | None:
    """The Java throwable line(s) in Caffeine's stderr, ANSI-stripped, or None.

    Returns the first throwable header plus its last `Caused by:` (the deepest
    cause), so the error names the real failure rather than a stack-frame tail.
    """

    lines = [_ANSI_ESCAPE.sub("", line).strip() for line in stderr.splitlines()]
    throwables = [line for line in lines if _THROWABLE_LINE.match(line)]
    if not throwables:
        return None
    picked = [throwables[0]]
    caused = [line for line in throwables if line.startswith("Caused by: ")]
    if caused and caused[-1] != throwables[0]:
        picked.append(caused[-1])
    return "\n".join(picked)


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
    when the subject declares no policy list. `size_divisors` maps a config-key
    suffix to a divisor; each emits `key=size // divisor` (e.g. dash's
    `numOfSegments = size / per-segment capacity`), so an invocation carrying
    any must hold exactly one size (`simulate()` splits per size).
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
    size_divisors: dict[str, int] = field(default_factory=dict)

    def gradle_cmd(self) -> list[str]:
        if self.size_divisors and len(self.sizes) != 1:
            raise CaffeineError(
                f"{self.policy_spec_name}: size-derived keys {sorted(self.size_divisors)} "
                f"need one size per invocation, got {self.sizes}"
            )
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
            f"-Dcaffeine.simulator.files.paths.0={_resolve_trace(self.trace)}",
            f"-Dcaffeine.simulator.policies.0={self.policy_spec_name}",
        ]
        for config_key, value in self.geometry.items():
            if isinstance(value, list):
                for i, item in enumerate(value):
                    cmd.append(f"-D{prefix}.{config_key}.{i}={item}")
            else:
                cmd.append(f"-D{prefix}.{config_key}={value}")
        for config_key, divisor in self.size_divisors.items():
            derived = _derive(self.sizes[0], divisor, config_key, self.policy_spec_name)
            cmd.append(f"-D{prefix}.{config_key}={derived}")
        cmd.append(f"-D{prefix}.debugMode={self.debug_mode}")
        if self.policy_key is not None:
            for i, policy in enumerate(self.policies):
                cmd.append(f"-D{prefix}.{self.policy_key}.{i}={policy}")
        if self.supports_log_path and self.log_path is not None:
            cmd.append(f"-D{prefix}.logPath={self.log_path}")
        return cmd


def _derive(size: int, divisor: int, config_key: str, policy_spec_name: str) -> int:
    """`size // divisor`, refusing a size that is not an exact positive multiple."""

    if divisor <= 0 or size <= 0 or size % divisor:
        raise CaffeineError(
            f"{policy_spec_name}: size {size} is not a positive multiple of the "
            f"{config_key} divisor {divisor} (size-divisors in the manifest [caffeine] table)"
        )
    return size // divisor


def _trace_display_name(trace: str) -> str:
    """`lirs:multi1.trace.gz` -> `multi1_lirs`."""
    fmt, _, filename = trace.partition(":")
    basename = os.path.basename(filename or fmt)
    name = basename.split(".")[0]
    return f"{name}_{fmt}" if filename else name


def _invoke(invocation: Invocation, *, timeout_s: int) -> RunResult:
    cmd = invocation.gradle_cmd()
    what = (
        f"policy={invocation.policy_spec_name}, trace={invocation.trace}, "
        f"sizes={invocation.sizes}"
    )
    # Drop stale per-size reports first, so a failed call can't pass the
    # missing-CSV check below on a previous run's file.
    for size in invocation.sizes:
        (invocation.output_dir / f"hit_rate_{size}.csv").unlink(missing_ok=True)
    try:
        proc = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout_s,
        )
    except subprocess.TimeoutExpired as e:
        raise CaffeineError(
            f"caffeine simulator timed out after {e.timeout}s ({what}, "
            f"policies={invocation.policies})"
        ) from e
    cause = _root_cause(proc.stderr)
    cause_text = f"\ncause: {cause}" if cause else "\ncause: unknown (no Java exception in stderr)"
    if proc.returncode != 0:
        stderr_tail = "\n".join(proc.stderr.splitlines()[-STDERR_TAIL_LINES:])
        raise CaffeineError(
            f"caffeine simulator failed (exit {proc.returncode}, {what}){cause_text}\n"
            f"{stderr_tail}"
        )
    # Caffeine can print a policy exception and still exit 0; a missing
    # per-size report is the reliable failure signal.
    missing = [
        size for size in invocation.sizes
        if not (invocation.output_dir / f"hit_rate_{size}.csv").is_file()
    ]
    if missing:
        raise CaffeineError(
            f"caffeine simulator exited 0 but wrote no hit_rate CSV for sizes {missing} "
            f"({what}, output_dir={invocation.output_dir})"
            f"{cause_text}"
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
    size_divisors: dict[str, int] | None = None,
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
        size_divisors=size_divisors,
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
    size_divisors: dict[str, int] | None = None,
) -> RunResult:
    """Single trace × multiple sizes.

    One Caffeine call sweeps every size, unless the subject declares
    `size_divisors`: their derived keys vary with the size, so each size gets
    its own call (all writing `hit_rate_<size>.csv` into `output_dir`).
    """

    if not sizes:
        raise CaffeineError("simulate(): sizes must be non-empty")
    if not policies:
        raise CaffeineError("simulate(): policies must be non-empty")
    size_divisors = dict(size_divisors or {})
    # Validate every size up front so a bad grid fails before any Caffeine call.
    for size in sizes:
        for config_key, divisor in size_divisors.items():
            _derive(size, divisor, config_key, policy_spec_name)
    batches = [[size] for size in sizes] if size_divisors else [list(sizes)]
    stdout: list[str] = []
    stderr: list[str] = []
    for batch in batches:
        invocation = Invocation(
            library_path=library_path,
            trace=trace,
            sizes=batch,
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
            size_divisors=size_divisors,
        )
        result = _invoke(invocation, timeout_s=timeout_s)
        stdout.append(result.stdout)
        stderr.append(result.stderr)
    return RunResult(output_dir=output_dir, stdout="\n".join(stdout), stderr="\n".join(stderr))


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
        print(f"error: subject {args.subject_dir.name}: {exception}", file=sys.stderr)
        return 1
    print(f"output_dir={result.output_dir}")
    return 0


if __name__ == "__main__":
    sys.exit(_main())
