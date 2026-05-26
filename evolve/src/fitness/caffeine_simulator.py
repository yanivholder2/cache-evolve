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
import tomllib
from dataclasses import dataclass
from pathlib import Path

# scripts/ is a top-level package at repo root. Tests register sys.path via
# conftest; evolve.py runs from `evolve/`, so bootstrap repo_root here too.
_REPO_ROOT = Path(__file__).resolve().parents[3]
if str(_REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT))

from scripts import caffeine_runner  # noqa: E402

from src._io import read_utf8, write_utf8
from src.config import Configs
from src.paths import CaffeineOutputDir, Layout, Variant
from src.variant.build import build_cdylib

log = logging.getLogger("fitness.caffeine")

# Issue-85 hardcoded fitness point. #83 replaces these with a trace suite.
HARDCODED_TRACE = "lirs:multi1.trace.gz"
HARDCODED_SIZE = 512
TARGET = "dashvariant"

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


def _read_eviction_policy(manifest_toml: Path) -> str:
    if not manifest_toml.is_file():
        raise CaffeineSimulatorError(f"manifest not found at {manifest_toml}")
    with open(manifest_toml, "rb") as f:
        data = tomllib.load(f)
    variant = data.get("variant")
    if not isinstance(variant, dict) or "eviction_policy" not in variant:
        raise CaffeineSimulatorError(
            f"[variant] eviction_policy missing from {manifest_toml}"
        )
    return str(variant["eviction_policy"])


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
) -> CaffeineResults:
    """Build the variant cdylib, invoke caffeine once, parse and aggregate.

    Persists caffeine outputs under `<variant_dir>/caffeine/` for inspection.
    Raises `CaffeineSimulatorError` on any failure (build, invocation,
    timeout, missing/malformed CSV).
    """

    variant_dir = variant.resolve(layout)
    policy_name = _read_eviction_policy(variant_dir.manifest_toml)
    debug_mode = _LOG_LEVEL_TO_DEBUG_MODE.get(cfg.framework.log_level, 0)

    caffeine_out = variant_dir.caffeine_dir
    caffeine_out.root.mkdir(parents=True, exist_ok=True)
    caffeine_out.rust_logs_dir.mkdir(parents=True, exist_ok=True)

    try:
        workspace, cdylib_path = build_cdylib(variant, layout=layout, repo_root=repo_root)
    except (RuntimeError, subprocess.CalledProcessError) as exception:
        raise CaffeineSimulatorError(f"cdylib build failed for {variant}: {exception}") from exception

    try:
        log.info("[%s] caffeine simulate trace=%s size=%d policy=%s", variant, HARDCODED_TRACE, HARDCODED_SIZE, policy_name)
        try:
            with _CAFFEINE_LOCK:
                run_result = caffeine_runner.run(
                    target=TARGET,
                    library_path=cdylib_path.parent,
                    trace=HARDCODED_TRACE,
                    size=HARDCODED_SIZE,
                    policies=[policy_name.lower()],
                    timeout_s=cfg.simulator.timeout_s,
                    debug_mode=debug_mode,
                    log_path=caffeine_out.rust_logs_dir,
                    output_dir=caffeine_out.root,
                )
        except caffeine_runner.CaffeineError as exception:
            _write_gradle_logs(caffeine_out, stdout="", stderr=str(exception))
            raise CaffeineSimulatorError(str(exception)) from exception

        _write_gradle_logs(caffeine_out, stdout=run_result.stdout, stderr=run_result.stderr)
        csv_path = _per_size_csv(caffeine_out, HARDCODED_SIZE)
        results = _parse_csv(csv_path, trace=HARDCODED_TRACE, size=HARDCODED_SIZE)
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
