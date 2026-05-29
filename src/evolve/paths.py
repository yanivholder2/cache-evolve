"""Single source of truth for the evolve on-disk layout."""

from __future__ import annotations

import logging
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Union

from evolve.identifiers import CycleId, HypothesisId, VariantId

BASELINE_STR = "baseline"
ATTEMPT_ID_WIDTH = 2

log = logging.getLogger(__name__)

_CDYLIB_FILENAME_FORMATS = {
    "linux": "lib{name}.so",
    "win32": "{name}.dll",
    "darwin": "lib{name}.dylib",
}


def cdylib_filename(platform: str, cdylib_name: str) -> str:
    try:
        template = _CDYLIB_FILENAME_FORMATS[platform]
    except KeyError:
        raise RuntimeError(
            f"unsupported platform {platform!r}; supported: {sorted(_CDYLIB_FILENAME_FORMATS)}"
        ) from None
    return template.format(name=cdylib_name)


@dataclass(frozen=True)
class CaffeineOutputDir:
    """Per-variant caffeine output directory. Producer (`caffeine_simulator`)
    and consumers (tests) reference subpaths only through these properties."""

    root: Path

    @property
    def rust_logs_dir(self) -> Path:
        return self.root / "rust-logs"

    @property
    def gradle_stdout_log(self) -> Path:
        return self.root / "gradle.stdout.log"

    @property
    def gradle_stderr_log(self) -> Path:
        return self.root / "gradle.stderr.log"

    def hit_rate_csv(self, size: int) -> Path:
        return self.root / f"hit_rate_{size}.csv"


@dataclass(frozen=True)
class VariantDir:
    """A variant-shaped on-disk directory. `root.name` need not be a valid
    `VariantId` — `baseline/` and prose templates carrying the literal
    `v<NNN>` placeholder share this layout. Use `CycleVariantDir` when the id
    matters.

    Variants are overlay-by-path: source overrides live under
    `overlay/<path-within-subject>`. An empty/missing `overlay/` means "no
    overlay" (used by the baseline)."""

    root: Path

    @property
    def overlay_dir(self) -> Path:
        return self.root / "overlay"

    @property
    def task_md(self) -> Path:
        return self.root / "task.md"

    @property
    def eval_dir(self) -> Path:
        return self.root / "eval"

    @property
    def fitness_dir(self) -> Path:
        return self.eval_dir / "fitness"

    @property
    def fitness_vector_json(self) -> Path:
        return self.eval_dir / "fitness-vector.json"

    @property
    def insight_md(self) -> Path:
        return self.root / "insight.md"

    @property
    def rejection_md(self) -> Path:
        return self.root / "rejection.md"

    @property
    def caffeine_dir(self) -> CaffeineOutputDir:
        # Shared eval substrate, alongside the fitness artifacts under eval/.
        return CaffeineOutputDir(root=self.eval_dir / "caffeine")


@dataclass(frozen=True)
class CargoWorkspace:
    """A subject-copy cargo workspace prepared for one variant build.

    Materialised by `evolve.variant.build.prepare_variant_workspace`: a fresh
    tempdir containing a full copy of the subject crate, the variant's `overlay/`
    overlay applied, and the `framework` path dependency rewritten to an
    absolute path. Callers own the lifecycle and must `shutil.rmtree(root)` once
    they no longer need the artifacts."""

    root: Path
    cdylib_name: str

    @property
    def cargo_toml(self) -> Path:
        return self.root / "Cargo.toml"

    @property
    def cdylib(self) -> Path:
        return self.root / "target" / "release" / cdylib_filename(sys.platform, self.cdylib_name)


@dataclass(frozen=True)
class AttemptDir(VariantDir):
    """One implementation attempt of a cycle variant
    (cycles/<id>/variants/v<NNN>/attempts/NN). Holds the per-attempt overlay,
    `task.md`, and the agent log. Per-variant artifacts (rejection.md, eval/,
    insight.md) live at the variant root, not here."""


@dataclass(frozen=True)
class CycleVariantDir(VariantDir):
    """A concrete cycle variant (cycles/<id>/variants/v<NNN>); `root.name`
    must parse as a `VariantId`.

    The overlay and `task.md` are per-attempt: a cycle variant materialises
    one or more attempts under `attempts/NN/`, and `overlay_dir`/`task_md`
    resolve to the latest one. Everything else (eval/, rejection.md,
    insight.md) is evaluated once and stays at the variant root."""

    @property
    def variant_id(self) -> VariantId:
        return VariantId.from_str(self.root.name)

    @property
    def attempts_dir(self) -> Path:
        return self.root / "attempts"

    def attempt(self, n: int) -> AttemptDir:
        return AttemptDir(root=self.attempts_dir / f"{n:0{ATTEMPT_ID_WIDTH}d}")

    @property
    def latest_attempt(self) -> AttemptDir:
        """The highest-numbered existing attempt, or attempt 1 if none exist."""
        if self.attempts_dir.is_dir():
            numbered = [
                int(p.name)
                for p in self.attempts_dir.iterdir()
                if p.is_dir() and p.name.isdigit()
            ]
            if numbered:
                return self.attempt(max(numbered))
        return self.attempt(1)

    @property
    def overlay_dir(self) -> Path:
        return self.latest_attempt.overlay_dir

    @property
    def task_md(self) -> Path:
        return self.latest_attempt.task_md


@dataclass(frozen=True)
class CycleLayout:
    """Paths under one cycle directory (cycles/<id>)."""

    root: Path

    @property
    def cycle_id(self) -> CycleId:
        return CycleId.from_str(self.root.name)

    @property
    def hypotheses_dir(self) -> Path:
        return self.root / "hypotheses"

    @property
    def hypotheses_task_md(self) -> Path:
        return self.hypotheses_dir / "task.md"

    @property
    def variants_dir(self) -> Path:
        return self.root / "variants"

    @property
    def winners_json(self) -> Path:
        return self.root / "winners.json"

    @property
    def cycle_insight_md(self) -> Path:
        return self.root / "cycle-insight.md"

    @property
    def sealed_marker(self) -> Path:
        return self.root / "SEALED"

    def hypothesis_file(self, hypothesis_id: HypothesisId) -> Path:
        return self.hypotheses_dir / f"{hypothesis_id}.md"

    def variant(self, variant_id: VariantId) -> CycleVariantDir:
        return CycleVariantDir(root=self.variants_dir / str(variant_id))

    @property
    def variant_template(self) -> VariantDir:
        """Prose-only variant directory whose `root` ends in the literal
        placeholder `v<NNN>`. Use to document where variant artefacts will
        live without binding to a concrete variant id.

        Note: `overlay_dir` on this template is wrong for cycle variants — a
        cycle variant's overlay is per-attempt. Use `variant_overlay_template`
        for the overlay path; root-level artefacts (`fitness_vector_json`,
        `insight_md`) are correct here."""
        return VariantDir(root=self.variants_dir / "v<NNN>")

    @property
    def variant_overlay_template(self) -> Path:
        """Prose-only overlay path for a cycle variant, carrying the literal
        placeholders `v<NNN>` (variant) and `<NN>` (attempt). A cycle variant's
        overlay lives under its `attempts/<NN>/overlay`, one per attempt — not
        a flat `v<NNN>/overlay` (that layout is only the baseline/prose
        template)."""
        return self.variant_template.root / "attempts" / "<NN>" / "overlay"


@dataclass(frozen=True)
class SourcesLayout:
    """Paths in the read-only evolve sources subtree.

    `subject` may be None for callers that need only subject-independent paths
    (config.toml, agent prompts, the framework crate). Subject-scoped paths
    (`subject_dir` and everything derived from it) raise if `subject` is unset,
    so a missing subject fails loudly instead of silently defaulting.
    """

    root: Path
    subject: str | None = None

    @property
    def config_toml(self) -> Path:
        return self.root / "config.toml"

    @property
    def subject_dir(self) -> Path:
        if self.subject is None:
            raise ValueError("SourcesLayout.subject is not set; this path requires a selected subject")
        return self.root / "inputs" / "subjects" / self.subject

    @property
    def agents_dir(self) -> Path:
        return self.root / "src" / "evolve" / "agents"

    def agent_prompt(self, agent_name: str) -> Path:
        return self.agents_dir / f"{agent_name}.md"

    @property
    def baseline(self) -> VariantDir:
        # Mirrors the variant layout so the baseline reads with the same file
        # conventions as any cycle variant. Returns `VariantDir` (not
        # `CycleVariantDir`) because `"baseline"` isn't a valid `VariantId`.
        return VariantDir(root=self.subject_dir / "baseline")

    @property
    def bench_dir(self) -> Path:
        return self.subject_dir / "bench"

    @property
    def subject_src(self) -> Path:
        return self.subject_dir / "src"

    @property
    def subject_manifest(self) -> Path:
        return self.subject_dir / "manifest.toml"

    @property
    def subject_cargo_toml(self) -> Path:
        return self.subject_dir / "Cargo.toml"

    @property
    def framework_crate_dir(self) -> Path:
        return self.root / "framework-rs"


@dataclass(frozen=True)
class OutputsLayout:
    """Paths in the read-write outputs subtree."""

    root: Path

    @property
    def cycles_dir(self) -> Path:
        return self.root / "cycles"

    @property
    def ledger(self) -> Path:
        return self.root / "cycles-ledger.jsonl"

    @property
    def leaderboard_md(self) -> Path:
        return self.root / "leaderboard.md"

    @property
    def cross_cycle_insights_md(self) -> Path:
        return self.root / "cross-cycle-insights.md"

    @property
    def target_dir(self) -> Path:
        return self.root / "target"

    @property
    def logs_dir(self) -> Path:
        return self.root / "logs"

    def cycle_log(self, cycle_id: CycleId) -> Path:
        return self.logs_dir / f"cycle-{cycle_id}.log"

    def cycle(self, cycle_id: CycleId) -> CycleLayout:
        return CycleLayout(root=self.cycles_dir / str(cycle_id))


@dataclass(frozen=True)
class Layout:
    """Filesystem layout for one evolve invocation.

    `repo_root` is the read-only source root; `outputs_root` is the
    read-write working dir.
    """

    repo_root: Path
    outputs_root: Path
    subject: str | None = None

    @property
    def sources(self) -> SourcesLayout:
        return SourcesLayout(root=self.repo_root, subject=self.subject)

    @property
    def outputs(self) -> OutputsLayout:
        return OutputsLayout(root=self.outputs_root)

    # repo-relative ledger formatting
    def cycle_ledger_path(self, cycle_id: CycleId) -> str:
        """Repo-relative path string for a cycle dir, anchored to `repo_root`."""

        anchor = self.repo_root
        cycle_dir = self.outputs.cycle(cycle_id).root
        try:
            rel = cycle_dir.relative_to(anchor)
        except ValueError as e:
            raise ValueError(
                f"cycle dir {cycle_dir} is not under repo_root {anchor}; "
                "outputs_root must live inside the repo (repo-relative paths only)"
            ) from e
        return str(rel)


# Addressable variant references — either the permanent baseline or a
# past/current cycle variant. Used as the parent in hypotheses and as the
# identifier passed through the build, gate, and fitness layers.


@dataclass(frozen=True)
class Baseline:
    """The permanent research baseline."""

    def resolve(self, layout: Layout) -> VariantDir:
        return layout.sources.baseline

    @property
    def label(self) -> str:
        """Short form used as the fitness-vector key (cycle implied by path)."""
        return BASELINE_STR

    def __str__(self) -> str:
        return BASELINE_STR


@dataclass(frozen=True)
class CycleVariant:
    """A variant from a specific cycle."""

    cycle_id: CycleId
    variant_id: VariantId

    def resolve(self, layout: Layout) -> CycleVariantDir:
        return layout.outputs.cycle(self.cycle_id).variant(self.variant_id)

    @property
    def label(self) -> str:
        """Short form used as the fitness-vector key (cycle implied by path)."""
        return str(self.variant_id)

    def __str__(self) -> str:
        return f"cycle/{self.cycle_id}/{self.variant_id}"


Variant = Union[Baseline, CycleVariant]
