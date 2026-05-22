"""Single source of truth for the evolve on-disk layout.

Every path string that names a file or directory under `evolve_root` (read-only
sources: config, subject, traces, bench) or `outputs_root` (read-write working
dir: cycles, ledger, logs, leaderboard, cargo target) lives here. Callers
build a `Layout` and ask it for paths instead of concatenating strings.

The dataclasses are frozen — a Layout is a value, not a registry.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class VariantLayout:
    """Paths under one variant directory (cycles/<id>/variants/v<NNN>)."""

    root: Path

    @property
    def id(self) -> str:
        return self.root.name

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
    def num(self) -> str:
        return self.root.name

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
    def cohort_md(self) -> Path:
        return self.root / "cohort.md"

    @property
    def cycle_insight_md(self) -> Path:
        return self.root / "cycle-insight.md"

    @property
    def sealed_marker(self) -> Path:
        return self.root / "SEALED"

    def hypothesis_file(self, i: int) -> Path:
        return self.hypotheses_dir / f"H{i:02d}.md"

    def variant(self, i: int) -> VariantLayout:
        return VariantLayout(root=self.variants_dir / f"v{i:03d}")


@dataclass(frozen=True)
class Layout:
    """Filesystem layout for one evolve invocation.

    `evolve_root` is the read-only source root (config, traces, subject,
    harness, bench). `outputs_root` is the read-write working dir (cycles,
    ledger, logs, leaderboard, cargo target). Either may be omitted when a
    caller only needs paths from the other half — accessing the missing
    subtree raises TypeError.
    """

    evolve_root: Path | None = None
    outputs_root: Path | None = None

    # outputs subtree (read-write)
    @property
    def cycles_dir(self) -> Path:
        return self.outputs_root / "cycles"

    @property
    def ledger(self) -> Path:
        return self.outputs_root / "ledger" / "cycles.jsonl"

    @property
    def leaderboard_md(self) -> Path:
        return self.outputs_root / "leaderboard.md"

    @property
    def synthesis_md(self) -> Path:
        return self.outputs_root / "synthesis" / "current.md"

    @property
    def rejection_log(self) -> Path:
        return self.outputs_root / "rejection-log.md"

    @property
    def target_dir(self) -> Path:
        return self.outputs_root / "target"

    @property
    def logs_dir(self) -> Path:
        return self.outputs_root / "logs"

    def cycle_log(self, cycle_num: str) -> Path:
        return self.logs_dir / f"cycle-{cycle_num}.log"

    def cycle(self, cycle_num: str) -> CycleLayout:
        return CycleLayout(root=self.cycles_dir / cycle_num)

    # evolve subtree (read-only sources)
    @property
    def extension_points_md(self) -> Path:
        return self.evolve_root / "subject" / "extension-points.md"

    @property
    def baseline(self) -> VariantLayout:
        """Permanent research baseline at evolve/subject/baseline/.

        Mirrors VariantLayout shape so cycle 1 reads the baseline
        with the same file conventions cycle 2+ uses for the previous cycle's
        variants.
        """

        return VariantLayout(root=self.evolve_root / "subject" / "baseline")

    @property
    def bench_dir(self) -> Path:
        return self.evolve_root / "bench"

    @property
    def good_segment_rs(self) -> Path:
        return self.evolve_root / "tests" / "fixtures" / "good" / "impl" / "segment.rs"

    # repo-relative ledger formatting
    def cycle_ledger_path(self, cycle_num: str) -> str:
        """Repo-relative path string for a cycle dir, suitable for ledger rows."""

        rel = self.cycle(cycle_num).root.relative_to(self.outputs_root)
        return f"evolve/outputs/{rel}"
