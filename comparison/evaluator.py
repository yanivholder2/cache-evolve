"""OpenEvolve evaluator: scores a candidate with cache-evolve's own gates and
fitness plugins (see common.evaluate_source).

OpenEvolve imports this file and calls `evaluate(program_path)`; the setup (subject,
profile, dimensions) and file under evolution come from the environment because that call carries no
other arguments. Every evaluation is appended to $COMPARISON_LOG_DIR so the
collector can reconstruct the run without parsing OpenEvolve's checkpoints."""

from __future__ import annotations

import json
import os
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from comparison.common import GATE_FEEDBACK_ENV, Setup, evaluate_in_tempdir  # noqa: E402
from openevolve.evaluation_result import EvaluationResult  # noqa: E402


def _record(log_dir: Path, source: str, outcome) -> int:
    log_dir.mkdir(parents=True, exist_ok=True)
    (log_dir / "candidates").mkdir(exist_ok=True)
    log = log_dir / "evaluations.jsonl"
    index = sum(1 for _ in log.open()) if log.is_file() else 0
    (log_dir / "candidates" / f"{index:04d}.rs").write_text(source, encoding="utf-8")
    row = {
        "index": index,
        "time": time.time(),
        "passed": outcome.passed,
        "score": outcome.score,
        "stage": outcome.stage,
        "dimensions": outcome.dimensions,
    }
    with log.open("a", encoding="utf-8") as f:
        f.write(json.dumps(row) + "\n")
    return index


def evaluate(program_path: str) -> EvaluationResult:
    setup = Setup.from_env()
    evolve_file = os.environ["COMPARISON_EVOLVE_FILE"]
    log_dir = Path(os.environ["COMPARISON_LOG_DIR"])

    source = Path(program_path).read_text(encoding="utf-8")
    outcome = evaluate_in_tempdir(source, setup=setup, evolve_file=evolve_file)
    _record(log_dir, source, outcome)

    metrics = {"combined_score": outcome.score, "passed": float(outcome.passed)}
    metrics.update({dim.replace("-", "_"): value for dim, value in outcome.dimensions.items()})
    artifacts = {}
    if not outcome.passed and os.environ.get(GATE_FEEDBACK_ENV, "full") == "full":
        # The analogue of cache-evolve's self-repair feedback: OpenEvolve feeds
        # artifacts of the parent into the next prompt. With --gate-feedback none
        # the gates are a binary filter (score -1, no explanation), so OpenEvolve
        # gets none of cache-evolve's rejection text.
        artifacts["failure_stage"] = outcome.stage
        artifacts["failure_detail"] = outcome.detail
    return EvaluationResult(metrics=metrics, artifacts=artifacts)
