"""Single source of truth for the evolve on-disk layout."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from src.identifiers import CycleId, HypothesisId, VariantId


@dataclass(frozen=True)
class VariantLayout:
    """Paths under one variant directory (cycles/<id>/variants/v<NNN>)."""

    root: Path

    @property
    def variant_id(self) -> VariantId:
        return VariantId.from_str(self.root.name)

    @property
    def impl_dir(self) -> Path:
        return self.root / "impl"

    @property
    def impl_task_md(self) -> Path:
        return self.impl_dir / "task.md"

    @property
    def manifest_toml(self) -> Path:
        return self.impl_dir / "manifest.toml"

    @property
    def segment_rs(self) -> Path:
        return self.impl_dir / "segment.rs"

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

    def variant(self, variant_id: VariantId) -> VariantLayout:
        return VariantLayout(root=self.variants_dir / str(variant_id))


@dataclass(frozen=True)
class SourcesLayout:
    """Paths in the read-only evolve sources subtree."""

    root: Path

    @property
    def subject_dir(self) -> Path:
        return self.root / "subject"

    @property
    def extension_points_md(self) -> Path:
        return self.subject_dir / "extension-points.md"

    @property
    def baseline(self) -> VariantLayout:
        # Mirrors VariantLayout shape so the baseline reads with the same file
        # conventions as any cycle variant.
        return VariantLayout(root=self.subject_dir / "baseline")

    @property
    def bench_dir(self) -> Path:
        return self.root / "bench"

    @property
    def dash_src(self) -> Path:
        return self.root.parent / "src"


@dataclass(frozen=True)
class OutputsLayout:
    """Paths in the read-write outputs subtree."""

    root: Path

    @property
    def cycles_dir(self) -> Path:
        return self.root / "cycles"

    @property
    def ledger(self) -> Path:
        return self.root / "ledger" / "cycles.jsonl"

    @property
    def leaderboard_md(self) -> Path:
        return self.root / "leaderboard.md"

    @property
    def synthesis_md(self) -> Path:
        return self.root / "synthesis" / "current.md"

    @property
    def rejection_log(self) -> Path:
        return self.root / "rejection-log.md"

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

    `evolve_root` is the read-only source root; `outputs_root` is the
    read-write working dir.
    """

    evolve_root: Path
    outputs_root: Path

    @property
    def sources(self) -> SourcesLayout:
        return SourcesLayout(root=self.evolve_root)

    @property
    def outputs(self) -> OutputsLayout:
        return OutputsLayout(root=self.outputs_root)

    # outputs subtree (read-write)
    @property
    def cycles_dir(self) -> Path:
        return self.outputs.cycles_dir

    @property
    def ledger(self) -> Path:
        return self.outputs.ledger

    @property
    def leaderboard_md(self) -> Path:
        return self.outputs.leaderboard_md

    @property
    def synthesis_md(self) -> Path:
        return self.outputs.synthesis_md

    @property
    def rejection_log(self) -> Path:
        return self.outputs.rejection_log

    @property
    def target_dir(self) -> Path:
        return self.outputs.target_dir

    @property
    def logs_dir(self) -> Path:
        return self.outputs.logs_dir

    def cycle_log(self, cycle_id: CycleId) -> Path:
        return self.outputs.cycle_log(cycle_id)

    def cycle(self, cycle_id: CycleId) -> CycleLayout:
        return self.outputs.cycle(cycle_id)

    # evolve subtree (read-only sources)
    @property
    def subject_dir(self) -> Path:
        return self.sources.subject_dir

    @property
    def extension_points_md(self) -> Path:
        return self.sources.extension_points_md

    @property
    def baseline(self) -> VariantLayout:
        return self.sources.baseline

    @property
    def bench_dir(self) -> Path:
        return self.sources.bench_dir

    @property
    def dash_src(self) -> Path:
        return self.sources.dash_src

    # repo-relative ledger formatting
    def cycle_ledger_path(self, cycle_id: CycleId) -> str:
        """Repo-relative path string for a cycle dir (ADR-0003).

        Anchored to `evolve_root.parent`.
        """

        anchor = self.evolve_root.parent
        cycle_dir = self.cycle(cycle_id).root
        try:
            rel = cycle_dir.relative_to(anchor)
        except ValueError as e:
            raise ValueError(
                f"cycle dir {cycle_dir} is not under evolve_root.parent {anchor}; "
                "outputs_root must live inside the repo (ADR-0003: repo-relative paths only)"
            ) from e
        return str(rel)
