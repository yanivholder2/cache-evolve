"""Shared by both arms of the comparison: the one evaluation function and the
one scoring function. cache-evolve's own gates and fitness plugins do the
work; nothing here re-implements them, so a candidate scores identically no
matter which framework proposed it."""

from __future__ import annotations

import json
import shutil
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT / "src") not in sys.path:
    sys.path.insert(0, str(REPO_ROOT / "src"))

from evolve.aggregate.selectors.weighted import WeightedSelector  # noqa: E402
from evolve.config import load_configs  # noqa: E402
from evolve.fitness.runner import FitnessError, compute_fitness  # noqa: E402
from evolve.identifiers import CycleId, VariantId  # noqa: E402
from evolve.paths import CycleVariant, Layout  # noqa: E402
from evolve.subject import load_subject  # noqa: E402
from evolve.variant.gates import run_gates  # noqa: E402

# Score of a candidate that fails a gate or cannot be measured. The baseline
# scores 0 and the weighted score is a fraction of the baseline, so -1 sits
# below any plausible regression without distorting OpenEvolve's selection.
FAILED_SCORE = -1.0

TAIL_CHARS = 4000


def default_evolve_file(subject: str) -> str:
    """The single implementation file OpenEvolve evolves for `subject`, relative
    to the subject crate. OpenEvolve rewrites one file, so subjects whose logic
    spans several extension-point files need an explicit --evolve-file."""
    candidate = REPO_ROOT / "inputs" / "subjects" / subject / "src" / subject / "mod.rs"
    if not candidate.is_file():
        raise SystemExit(
            f"no default evolve file for subject {subject!r}; pass --evolve-file "
            "(path relative to the subject crate)"
        )
    return candidate.relative_to(REPO_ROOT / "inputs" / "subjects" / subject).as_posix()


@dataclass
class Outcome:
    passed: bool
    score: float
    stage: str  # "ok" | "gate:<name>" | "fitness" | "precheck"
    detail: str = ""
    dimensions: dict[str, float] = field(default_factory=dict)


def _tail(text: str) -> str:
    return text[-TAIL_CHARS:]


def evaluate_source(
    source: str, *, subject: str, evolve_file: str, work_dir: Path, index: int
) -> Outcome:
    """Gate and score one candidate version of `evolve_file`.

    The source is laid out as a cache-evolve overlay (one file over the pristine
    subject crate), run through the same gate list and fitness plugins a
    cache-evolve variant gets, and reduced to the weighted score over the
    checked-in baseline that cache-evolve's `weighted` selector uses."""

    cfg = load_configs(REPO_ROOT, subject=subject)
    layout = Layout(repo_root=REPO_ROOT, outputs_root=work_dir, subject=subject)
    subj = load_subject(layout.sources.subject_dir)

    rel = Path(evolve_file)
    if subj.is_pinned_overlay_path(rel):
        return Outcome(False, FAILED_SCORE, "precheck", f"{evolve_file} is gate-pinned")
    for protected in subj.protected_files:
        # A protected JNI module declared from the evolved file must stay wired in.
        if protected.parent != rel.parent:
            continue
        module = protected.stem
        if module not in source:
            return Outcome(
                False, FAILED_SCORE, "precheck",
                f"the program must keep declaring `pub mod {module};` (protected JNI boundary)",
            )

    variant = CycleVariant(CycleId(1), VariantId(index))
    variant_dir = variant.resolve(layout)
    target = variant_dir.attempt(1).overlay_dir / rel
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(source, encoding="utf-8")

    result = run_gates(variant, layout=layout, repo_root=REPO_ROOT)
    if not result.all_passed:
        gate = result.first_failed_gate
        rejection = variant_dir.rejection_md
        detail = rejection.read_text(encoding="utf-8") if rejection.is_file() else ""
        return Outcome(False, FAILED_SCORE, f"gate:{gate}", _tail(detail))

    try:
        compute_fitness(variant, layout=layout, repo_root=REPO_ROOT, cfg=cfg)
    except FitnessError as exception:
        return Outcome(False, FAILED_SCORE, "fitness", _tail(str(exception)))

    fitness_vector = json.loads(variant_dir.fitness_vector_json.read_text(encoding="utf-8"))
    baseline = json.loads(layout.sources.baseline.fitness_vector_json.read_text(encoding="utf-8"))
    score = score_fitness_vector(cfg, fitness_vector, baseline)
    dims = {d["dim"]: float(d["summary"]) for d in fitness_vector["dimensions"]}
    return Outcome(True, score, "ok", dimensions=dims)


def score_fitness_vector(cfg, fitness_vector: dict, baseline: dict) -> float:
    """Weighted percentage improvement over the baseline (0 = baseline)."""
    if cfg.selection.method != "weighted":
        raise SystemExit(
            f"comparison needs a scalar score; config.toml selection.method is "
            f"{cfg.selection.method!r}, expected 'weighted'"
        )
    selector = WeightedSelector.from_config(
        winners_count=cfg.selection.winners_count,
        dimensions=cfg.fitness.dimensions,
        weights=cfg.selection.weights,
    )
    selected = selector.select([fitness_vector], baseline=baseline)
    return selected.scores[fitness_vector["variant"]]


def evaluate_in_tempdir(source: str, *, subject: str, evolve_file: str, index: int = 1) -> Outcome:
    work_dir = Path(tempfile.mkdtemp(prefix="comparison-eval-"))
    try:
        return evaluate_source(
            source, subject=subject, evolve_file=evolve_file, work_dir=work_dir, index=index
        )
    finally:
        shutil.rmtree(work_dir, ignore_errors=True)
