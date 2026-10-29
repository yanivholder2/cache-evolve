"""Shared by both arms of the comparison: the one evaluation function and the
one scoring function. cache-evolve's own gates and fitness plugins do the
work; nothing here re-implements them, so a candidate scores identically no
matter which framework proposed it."""

from __future__ import annotations

import json
import os
import shutil
import sys
import tempfile
from dataclasses import asdict, dataclass, field
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


SETUP_ENV = "COMPARISON_SETUP"


@dataclass(frozen=True)
class Setup:
    """What is being scored: the subject plus the evaluation settings that both arms
    must share. `evaluation_profile` names a manifest `[evaluation.profiles.NAME]`
    grid (e.g. "msr" for the SNIA traces) and its baseline anchor; None is the
    default grid. `dimensions` / `weights` / `simulator_timeout_s` override
    config.toml the way the `evolve` flags of the same name do. The comparison
    needs a scalar score, so selection is always "weighted"; a single active
    dimension gets weight 1.0."""

    subject: str
    evaluation_profile: str | None = None
    dimensions: tuple[str, ...] | None = None
    weights: dict[str, float] | None = None
    simulator_timeout_s: int | None = None

    def configs(self):
        weights = self.weights
        if weights is None and self.dimensions is not None and len(self.dimensions) == 1:
            weights = {self.dimensions[0]: 1.0}
        return load_configs(
            REPO_ROOT,
            subject=self.subject,
            evaluation_profile=self.evaluation_profile,
            dimensions=self.dimensions,
            selection_method="weighted",
            weights=weights,
            simulator_timeout_s=self.simulator_timeout_s,
        )

    def layout(self, outputs_root: Path) -> Layout:
        return Layout(
            repo_root=REPO_ROOT, outputs_root=outputs_root, subject=self.subject,
            evaluation_profile=self.evaluation_profile,
        )

    def baseline(self) -> dict:
        path = self.layout(REPO_ROOT).sources.baseline.fitness_vector_json
        return json.loads(path.read_text(encoding="utf-8"))

    def validate(self) -> None:
        """Fail before any LLM time is spent: unknown profile, missing or mismatched anchor."""
        subj = load_subject(self.layout(REPO_ROOT).sources.subject_dir)
        subj.validate_profile_baseline(self.evaluation_profile)
        self.configs()

    @classmethod
    def add_arguments(cls, parser) -> None:
        """The evaluation flags both entry points share (same names as `evolve`'s)."""
        parser.add_argument("--evaluation-profile", default=None, metavar="NAME",
                            help="score on the manifest's [evaluation.profiles.NAME] grid and baseline "
                                 "anchor, e.g. `msr` for the SNIA traces (default: the default grid)")
        parser.add_argument("--dimensions", nargs="+", default=None, metavar="DIM",
                            help="active fitness dimensions (default: config.toml); one dimension "
                                 "scores with weight 1.0")
        parser.add_argument("--weights", nargs="+", default=None, metavar="DIM=W",
                            help="per-dimension weights when several dimensions are active")
        parser.add_argument("--simulator-timeout-s", type=int, default=None,
                            help="per-Caffeine-call timeout (default: config.toml)")

    @classmethod
    def from_args(cls, args) -> "Setup":
        weights = None
        if args.weights:
            try:
                weights = {k: float(v) for k, _, v in (item.partition("=") for item in args.weights)}
            except ValueError:
                raise SystemExit(f"--weights expects DIM=W pairs, got {args.weights!r}")
        return cls(
            subject=args.subject,
            evaluation_profile=args.evaluation_profile,
            dimensions=tuple(args.dimensions) if args.dimensions else None,
            weights=weights,
            simulator_timeout_s=args.simulator_timeout_s,
        )

    def evolve_flags(self) -> list[str]:
        """The `evolve` CLI flags that give the cache-evolve arm this same setup."""
        cfg = self.configs()
        flags = ["--selection-method", "weighted", "--dimensions", *cfg.fitness.dimensions,
                 "--weights", *(f"{d}={w}" for d, w in cfg.selection.weights.items())]
        if self.evaluation_profile:
            flags += ["--evaluation-profile", self.evaluation_profile]
        if self.simulator_timeout_s:
            flags += ["--simulator-timeout-s", str(self.simulator_timeout_s)]
        return flags

    def to_env(self) -> dict[str, str]:
        return {SETUP_ENV: json.dumps(asdict(self))}

    @classmethod
    def from_env(cls) -> "Setup":
        raw = json.loads(os.environ[SETUP_ENV])
        if raw.get("dimensions") is not None:
            raw["dimensions"] = tuple(raw["dimensions"])
        return cls(**raw)


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
    source: str, *, setup: Setup, evolve_file: str, work_dir: Path, index: int
) -> Outcome:
    """Gate and score one candidate version of `evolve_file`.

    The source is laid out as a cache-evolve overlay (one file over the pristine
    subject crate), run through the same gate list and fitness plugins a
    cache-evolve variant gets, and reduced to the weighted score over the
    checked-in baseline that cache-evolve's `weighted` selector uses."""

    cfg = setup.configs()
    layout = setup.layout(work_dir)
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
    baseline = setup.baseline()
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


def evaluate_in_tempdir(source: str, *, setup: Setup, evolve_file: str, index: int = 1) -> Outcome:
    work_dir = Path(tempfile.mkdtemp(prefix="comparison-eval-"))
    try:
        return evaluate_source(
            source, setup=setup, evolve_file=evolve_file, work_dir=work_dir, index=index
        )
    finally:
        shutil.rmtree(work_dir, ignore_errors=True)
