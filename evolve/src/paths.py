"""Single source of truth for the evolve on-disk layout."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Union

from src.identifiers import CycleId, HypothesisId, VariantId

BASELINE_STR = "baseline"


@dataclass(frozen=True)
class VariantDir:
    """A variant-shaped on-disk directory. `root.name` need not be a valid
    `VariantId` — `baseline/` and prose templates carrying the literal
    `v<NNN>` placeholder share this layout. Use `CycleVariantDir` when the id
    matters."""

    root: Path

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
class CargoWorkspaceDir:
    """A scaffolded cargo crate laid out under a tempdir for compiling one variant."""

    root: Path

    @property
    def cargo_toml(self) -> Path:
        return self.root / "Cargo.toml"

    @property
    def manifest_toml(self) -> Path:
        return self.root / "manifest.toml"

    @property
    def src_dir(self) -> Path:
        return self.root / "src"

    @property
    def lib_rs(self) -> Path:
        return self.src_dir / "lib.rs"

    @property
    def variant_rs(self) -> Path:
        return self.src_dir / "variant.rs"

    @property
    def jni_rs(self) -> Path:
        return self.src_dir / "jni.rs"

    @property
    def tests_dir(self) -> Path:
        return self.root / "tests"

    @property
    def g2_contract_rs(self) -> Path:
        return self.tests_dir / "g2_contract.rs"

    @property
    def g3_invariant_rs(self) -> Path:
        return self.tests_dir / "g3_invariant.rs"

    def cdylib(self, filename: str) -> Path:
        return self.root / "target" / "release" / filename


@dataclass(frozen=True)
class CycleVariantDir(VariantDir):
    """A concrete cycle variant (cycles/<id>/variants/v<NNN>); `root.name`
    must parse as a `VariantId`."""

    @property
    def variant_id(self) -> VariantId:
        return VariantId.from_str(self.root.name)


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
        live without binding to a concrete variant id."""
        return VariantDir(root=self.variants_dir / "v<NNN>")


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
    def baseline(self) -> VariantDir:
        # Mirrors the variant layout so the baseline reads with the same file
        # conventions as any cycle variant. Returns `VariantDir` (not
        # `CycleVariantDir`) because `"baseline"` isn't a valid `VariantId`.
        return VariantDir(root=self.subject_dir / "baseline")

    @property
    def bench_dir(self) -> Path:
        return self.root / "bench"

    @property
    def harness_dir(self) -> Path:
        return self.root / "harness"

    @property
    def g2_contract_rs(self) -> Path:
        return self.harness_dir / "g2_contract.rs"

    @property
    def g3_invariant_rs(self) -> Path:
        return self.harness_dir / "g3_invariant.rs"

    @property
    def caffeine_jni_dir(self) -> Path:
        return self.root / "caffeine_jni"

    @property
    def variant_jni_rs(self) -> Path:
        return self.caffeine_jni_dir / "variant_jni.rs"

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
    def baseline(self) -> VariantDir:
        return self.sources.baseline

    @property
    def bench_dir(self) -> Path:
        return self.sources.bench_dir

    @property
    def harness_dir(self) -> Path:
        return self.sources.harness_dir

    @property
    def g2_contract_rs(self) -> Path:
        return self.sources.g2_contract_rs

    @property
    def g3_invariant_rs(self) -> Path:
        return self.sources.g3_invariant_rs

    @property
    def caffeine_jni_dir(self) -> Path:
        return self.sources.caffeine_jni_dir

    @property
    def variant_jni_rs(self) -> Path:
        return self.sources.variant_jni_rs

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


# Addressable variant references — either the permanent baseline or a
# past/current cycle variant. Used as the parent in hypotheses and as the
# identifier passed through the build, gate, and fitness layers.


@dataclass(frozen=True)
class Baseline:
    """The permanent research baseline."""

    def resolve(self, layout: Layout) -> VariantDir:
        return layout.baseline

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
        variant_dir = layout.cycle(self.cycle_id).variant(self.variant_id)
        if not variant_dir.root.is_dir():
            raise FileNotFoundError(f"variant dir not found: {variant_dir.root}")
        return variant_dir

    @property
    def label(self) -> str:
        """Short form used as the fitness-vector key (cycle implied by path)."""
        return str(self.variant_id)

    def __str__(self) -> str:
        return f"cycle/{self.cycle_id}/{self.variant_id}"


Variant = Union[Baseline, CycleVariant]
