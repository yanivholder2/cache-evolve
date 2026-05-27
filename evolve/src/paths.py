"""Single source of truth for the evolve on-disk layout."""

from __future__ import annotations

import logging
import shutil
import sys
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from tempfile import mkdtemp
from typing import Iterator, Type, TypeVar, Union

from src.identifiers import CycleId, HypothesisId, VariantId

BASELINE_STR = "baseline"

log = logging.getLogger("paths")

_CDYLIB_FILENAMES = {
    "linux": "libdash_variant.so",
    "win32": "dash_variant.dll",
    "darwin": "libdash_variant.dylib",
}


def cdylib_filename(platform: str) -> str:
    try:
        return _CDYLIB_FILENAMES[platform]
    except KeyError:
        raise RuntimeError(
            f"unsupported platform {platform!r}; supported: {sorted(_CDYLIB_FILENAMES)}"
        ) from None


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

    @property
    def rejection_md(self) -> Path:
        return self.root / "rejection.md"

    @property
    def caffeine_dir(self) -> CaffeineOutputDir:
        return CaffeineOutputDir(root=self.root / "caffeine")


@dataclass(frozen=True)
class CargoWorkspaceDir:
    """Path layout of a cargo crate built for one variant.

    Does not own its root directory; callers manage cleanup.
    """

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


@dataclass(frozen=True)
class GatesWorkspace(CargoWorkspaceDir):
    """Cargo crate scaffolded to run the contract gate + invariant gate harness tests."""

    @property
    def tests_dir(self) -> Path:
        return self.root / "tests"

    @property
    def contract_gate_rs(self) -> Path:
        return self.tests_dir / "contract_gate.rs"

    @property
    def invariant_gate_rs(self) -> Path:
        return self.tests_dir / "invariant_gate.rs"


@dataclass(frozen=True)
class CdylibWorkspace(CargoWorkspaceDir):
    """Cargo crate scaffolded to build a Caffeine-loadable cdylib."""

    @property
    def jni_rs(self) -> Path:
        return self.src_dir / "jni.rs"

    @property
    def cdylib(self) -> Path:
        return self.root / "target" / "release" / cdylib_filename(sys.platform)


# Preserves the concrete subclass through `cargo_workspace_tempdir` so callers
# keep access to subclass-only paths (e.g. `GatesWorkspace.contract_gate_rs`).
_W = TypeVar("_W", bound=CargoWorkspaceDir)

@contextmanager
def cargo_workspace_tempdir(prefix: str, cls: Type[_W]) -> Iterator[_W]:
    """Create a tempdir-rooted workspace of `cls`. Removes the dir on exception
    only; on clean exit the caller owns the workspace lifetime."""

    root = Path(mkdtemp(prefix=prefix))
    workspace = cls(root=root)
    try:
        yield workspace
    except BaseException:
        shutil.rmtree(
            root,
            onexc=lambda fn, path, exception: log.warning(
                "workspace cleanup failed for %s: %s", path, exception
            ),
        )
        raise


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
    def config_toml(self) -> Path:
        return self.root / "config.toml"

    @property
    def subject_dir(self) -> Path:
        return self.root / "subject"

    @property
    def agents_dir(self) -> Path:
        return self.root / "src" / "agents"

    def agent_prompt(self, agent_name: str) -> Path:
        return self.agents_dir / f"{agent_name}.md"

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
    def rust_scaffold_dir(self) -> Path:
        return self.root / "src" / "variant" / "rust_scaffold"

    @property
    def contract_gate_rs(self) -> Path:
        return self.rust_scaffold_dir / "contract_gate.rs"

    @property
    def invariant_gate_rs(self) -> Path:
        return self.rust_scaffold_dir / "invariant_gate.rs"

    @property
    def variant_jni_rs(self) -> Path:
        return self.rust_scaffold_dir / "variant_jni.rs"

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

    # repo-relative ledger formatting
    def cycle_ledger_path(self, cycle_id: CycleId) -> str:
        """Repo-relative path string for a cycle dir, anchored to `evolve_root.parent`."""

        anchor = self.evolve_root.parent
        cycle_dir = self.outputs.cycle(cycle_id).root
        try:
            rel = cycle_dir.relative_to(anchor)
        except ValueError as e:
            raise ValueError(
                f"cycle dir {cycle_dir} is not under evolve_root.parent {anchor}; "
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
