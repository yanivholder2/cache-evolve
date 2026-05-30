"""Per-op-time fitness plugin."""

from __future__ import annotations

import json
import statistics
import subprocess
import tempfile
from pathlib import Path

from evolve._io import read_utf8
from evolve.fitness._contract import validate_plugin_dict
from evolve.fitness.plugins._common import cargo_build, cargo_env
from evolve.paths import Layout
from evolve.subject import load_subject

RUN_TIMEOUT_S = 120
METADATA_TIMEOUT_S = 60


def _resolve_target_dir(bench_dir: Path, target_dir: Path | None) -> Path:
    """Returns the bench crate's target directory.

    When target_dir is set it is authoritative (cargo_build runs with
    CARGO_TARGET_DIR=target_dir), so return it without spawning cargo. Fall back
    to `cargo metadata` only when target_dir is None, because a workspace
    member's target lives at the workspace root, not under bench/.
    """

    if target_dir is not None:
        return target_dir

    result = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=bench_dir,
        capture_output=True,
        text=True,
        timeout=METADATA_TIMEOUT_S,
        check=True,
        env=cargo_env(target_dir),
    )
    return Path(json.loads(result.stdout)["target_directory"])


def _run_once(binary: Path, binary_name: str) -> dict:
    """Runs the microbench binary once and returns its validated record."""

    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
        tmp_out = Path(f.name)
    try:
        subprocess.run(
            [str(binary), "--output", str(tmp_out)],
            capture_output=True,
            text=True,
            timeout=RUN_TIMEOUT_S,
            check=True,
        )
        data = json.loads(read_utf8(tmp_out))
        validate_plugin_dict(data, source=binary_name)
    finally:
        tmp_out.unlink(missing_ok=True)
    return data


def _aggregate_runs(runs: list[dict], *, source: str) -> dict:
    """Aggregate the per-run records: median p50/p99 per op plus min/max spread,
    median summary, and the dim/higher_is_better carried from the runs."""

    first = runs[0]
    by_op: dict[str, list[dict]] = {}
    order: list[str] = []
    for run in runs:
        for entry in run["values"]:
            op = entry["op"]
            if op not in by_op:
                by_op[op] = []
                order.append(op)
            by_op[op].append(entry)

    values = []
    for op in order:
        entries = by_op[op]
        p50s = [entry["p50_ns"] for entry in entries]
        p99s = [entry["p99_ns"] for entry in entries]
        merged = {**entries[0]}
        merged["p50_ns"] = statistics.median(p50s)
        merged["p99_ns"] = statistics.median(p99s)
        merged["p50_ns_spread"] = [min(p50s), max(p50s)]
        merged["p99_ns_spread"] = [min(p99s), max(p99s)]
        values.append(merged)

    aggregated = {
        "dim": first["dim"],
        "values": values,
        "summary": statistics.median([run["summary"] for run in runs]),
        "higher_is_better": first["higher_is_better"],
    }
    validate_plugin_dict(aggregated, source=source)
    return aggregated


def compute(*, layout: Layout, target_dir: Path | None = None, repeats: int) -> dict:
    """Builds the selected subject's microbench once, runs it `repeats` times,
    and returns the aggregated record (median p50/p99 per op plus spread).

    Re-running absorbs sub-noise deltas so a single jittery run does not decide
    the Pareto front. The subject must declare a `[bench]` table in its manifest
    (`runner` only schedules this plugin when it does)."""

    subject = load_subject(layout.sources.subject_dir)
    if subject.bench is None:
        raise RuntimeError(f"subject {subject.name!r} declares no [bench]; per-op-time is unavailable")
    binary_name = subject.bench.binary_name

    bench_dir = layout.sources.bench_dir
    cargo_build(bench_dir, ["--release"], label=binary_name, target_dir=target_dir)
    binary = _resolve_target_dir(bench_dir, target_dir) / "release" / binary_name

    runs = [_run_once(binary, binary_name) for _ in range(repeats)]
    return _aggregate_runs(runs, source=binary_name)
