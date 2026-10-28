"""Evolve-only Caffeine wrapper: build the variant cdylib, invoke the
Caffeine simulator once per variant, parse the per-size CSV, and aggregate
into `CaffeineResults` for plugin consumption.
"""

from __future__ import annotations

import csv
import logging
import shutil
import subprocess
import sys
import threading
from dataclasses import dataclass
from pathlib import Path

# scripts/ is a top-level package at repo root. Bootstrap so `from scripts
# import caffeine_runner` works when the evolve package is invoked as a CLI.
_REPO_ROOT = Path(__file__).resolve().parents[3]
if str(_REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT))

from scripts import caffeine_runner  # noqa: E402

from evolve._io import read_utf8, write_utf8
from evolve.config import Configs
from evolve.paths import CaffeineOutputDir, Layout, Variant
from evolve.subject import load_subject
from evolve.variant.build import compile_cdylib

log = logging.getLogger(__name__)

# Variants no longer declare an eviction policy; the baseline runs ClassicLRU
# and that's what the simulator drives. A variant changes policy behavior by
# overlaying the eviction-policy extension point (src/shared/eviction_policy.rs),
# not by selecting a different policy here; factory.rs is contract-protected.
DEFAULT_EVICTION_POLICY = "ClassicLRU"

_CAFFEINE_LOCK = threading.Lock()

# Caffeine emits Hit Rate / Miss Rate / Admit rate as percentages (0–100).
# Normalise to fractions (0–1) at parse time so plugin output stays consistent
# with the historical hit-rate contract and with the baseline fixture.
def _pct(value: str) -> float:
    return float(value) / 100.0


_CSV_COLUMNS = {
    "Policy": ("policy", str),
    "Hit Rate": ("hit_rate", _pct),
    "Miss Rate": ("miss_rate", _pct),
    "Hits": ("hits", int),
    "Misses": ("misses", int),
    "Requests": ("requests", int),
    "Evictions": ("evictions", int),
    "Admit rate": ("admit_rate", _pct),
    "Requests Weight": ("requests_weight", float),
    "Weighted Hit Rate": ("weighted_hit_rate", float),
    "Weighted Miss Rate": ("weighted_miss_rate", float),
    "Adaption": ("adaption", float),
    "Average Miss Penalty": ("average_miss_penalty", float),
    "Average Penalty": ("average_penalty", float),
    "Steps": ("steps", int),
    "Time": ("time_ms", float),
}


@dataclass(frozen=True)
class CaffeineResult:
    """One (trace, size, policy) point's full caffeine output."""

    trace: str
    size: int
    policy: str
    hit_rate: float
    miss_rate: float
    hits: int
    misses: int
    requests: int
    evictions: int
    admit_rate: float | None = None
    requests_weight: float | None = None
    weighted_hit_rate: float | None = None
    weighted_miss_rate: float | None = None
    adaption: float | None = None
    average_miss_penalty: float | None = None
    average_penalty: float | None = None
    steps: int | None = None
    time_ms: float | None = None


@dataclass(frozen=True)
class CaffeineResults:
    """Flat list of per-(trace,size,policy) caffeine outputs for one variant."""

    results: list[CaffeineResult]


class CaffeineSimulatorError(RuntimeError):
    """Raised when the per-variant caffeine pipeline (build, invoke, parse) fails."""


# Python log levels -> rust debug_mode (see src/shared/logging.rs::debug_mode_to_level).
# Rust has no Error-only filter; WARNING/ERROR/CRITICAL all map to Warn (0).
_LOG_LEVEL_TO_DEBUG_MODE = {
    "DEBUG": 2,
    "INFO": 1,
    "WARNING": 0,
    "ERROR": 0,
    "CRITICAL": 0,
}


def _per_size_csv(caffeine_out: CaffeineOutputDir, size: int) -> Path:
    candidate = caffeine_out.hit_rate_csv(size)
    if not candidate.is_file():
        reports_dir = caffeine_out.root
        raise CaffeineSimulatorError(
            f"caffeine did not produce {candidate.name} under {reports_dir} "
            f"(contents: {sorted(p.name for p in reports_dir.iterdir()) if reports_dir.is_dir() else 'missing'})"
        )
    return candidate


# Caffeine emits one row per declared policy at each size. We pass exactly
# one policy (the variant's), so the CSV has exactly one data row.
def _parse_csv(csv_path: Path, *, trace: str, size: int) -> list[CaffeineResult]:
    text = read_utf8(csv_path)
    reader = csv.reader(text.splitlines())
    rows = list(reader)
    if not rows:
        raise CaffeineSimulatorError(f"empty CSV at {csv_path}")
    header = rows[0]
    results: list[CaffeineResult] = []
    for row in rows[1:]:
        if not row:
            continue
        kwargs: dict[str, object] = {}
        for col_name, cell in zip(header, row):
            mapping = _CSV_COLUMNS.get(col_name.strip())
            if mapping is None:
                continue
            field_name, caster = mapping
            cell = cell.strip()
            if cell == "":
                # Caffeine emits "" for zero-valued rates/counts (see
                # simulator Metrics.java). Treat as the caster's zero for
                # numeric fields; leave string fields (e.g. policy) missing
                # so the required-fields check below still catches them.
                if caster is str:
                    continue
                kwargs[field_name] = caster("0")
                continue
            try:
                kwargs[field_name] = caster(cell)
            except ValueError as e:
                raise CaffeineSimulatorError(
                    f"{csv_path.name}: column {col_name!r} value {cell!r} not parseable as {caster.__name__}"
                ) from e
        missing = [name for name in ("policy", "hit_rate", "miss_rate", "hits", "misses", "requests", "evictions") if name not in kwargs]
        if missing:
            raise CaffeineSimulatorError(
                f"{csv_path.name}: row missing required fields {missing}; header={header}"
            )
        results.append(CaffeineResult(trace=trace, size=size, **kwargs))  # type: ignore[arg-type]
    if not results:
        raise CaffeineSimulatorError(f"{csv_path.name}: no data rows")
    return results


def simulate_variant(
    variant: Variant,
    *,
    layout: Layout,
    repo_root: Path,
    cfg: Configs,
    target_dir: Path | None = None,
) -> CaffeineResults:
    """Build the variant cdylib, invoke caffeine once, parse and aggregate.

    Persists caffeine outputs under `<variant_dir>/caffeine/` for inspection.
    Raises `CaffeineSimulatorError` on any failure (build, invocation,
    timeout, missing/malformed CSV). `target_dir`, when given, shares one cargo
    target with the variant's bench-bin builds so dependency crates compile once.
    """

    variant_dir = variant.resolve(layout)
    policy_name = DEFAULT_EVICTION_POLICY
    subject = load_subject(layout.sources.subject_dir)
    caffeine = subject.caffeine
    evaluation = subject.evaluation_for(layout.evaluation_profile)
    traces = evaluation.traces
    sizes = evaluation.sizes
    debug_mode = _LOG_LEVEL_TO_DEBUG_MODE.get(cfg.framework.log_level, 0)

    caffeine_out = variant_dir.caffeine_dir
    caffeine_out.root.mkdir(parents=True, exist_ok=True)
    caffeine_out.rust_logs_dir.mkdir(parents=True, exist_ok=True)

    try:
        workspace, cdylib_path = compile_cdylib(
            variant, layout=layout, repo_root=repo_root, target_dir=target_dir
        )
    except (RuntimeError, subprocess.CalledProcessError) as exception:
        raise CaffeineSimulatorError(f"cdylib build failed for {variant}: {exception}") from exception

    try:
        results: list[CaffeineResult] = []
        gradle_stdout: list[str] = []
        gradle_stderr: list[str] = []
        # One native invocation per trace sweeps every size at once, emitting a
        # hit_rate_<size>.csv per size. Accumulate all (trace, size) points.
        for trace in traces:
            # Log the policy caffeine actually runs (the subject's registered
            # spec name), not the internal eviction-selector default. They differ:
            # caffeine drives `policy_spec_name`; `policy_name` only feeds the
            # internal selector for subjects that declare a `policy-key`.
            log.info(
                "[%s] caffeine simulate trace=%s sizes=%s policy=%s",
                variant, trace, list(sizes), caffeine.policy_spec_name,
            )
            try:
                with _CAFFEINE_LOCK:
                    run_result = caffeine_runner.simulate(
                        library_path=cdylib_path.parent,
                        trace=trace,
                        sizes=list(sizes),
                        policies=[policy_name.lower()],
                        timeout_s=cfg.simulator.timeout_s,
                        **caffeine.as_runner_kwargs(),
                        debug_mode=debug_mode,
                        log_path=caffeine_out.rust_logs_dir,
                        output_dir=caffeine_out.root,
                    )
            except caffeine_runner.CaffeineError as exception:
                _write_gradle_logs(
                    caffeine_out,
                    stdout="\n".join(gradle_stdout),
                    stderr="\n".join((*gradle_stderr, str(exception))),
                )
                raise CaffeineSimulatorError(str(exception)) from exception

            gradle_stdout.append(run_result.stdout)
            gradle_stderr.append(run_result.stderr)
            for size in sizes:
                csv_path = _per_size_csv(caffeine_out, size)
                results.extend(_parse_csv(csv_path, trace=trace, size=size))

        _write_gradle_logs(
            caffeine_out,
            stdout="\n".join(gradle_stdout),
            stderr="\n".join(gradle_stderr),
        )
        return CaffeineResults(results=results)
    finally:
        shutil.rmtree(
            workspace.root,
            onexc=lambda fn, path, exception: log.warning(
                "cdylib workspace cleanup failed for %s: %s", path, exception
            ),
        )


def _write_gradle_logs(dest: CaffeineOutputDir, *, stdout: str, stderr: str) -> None:
    write_utf8(dest.gradle_stdout_log, stdout)
    write_utf8(dest.gradle_stderr_log, stderr)
