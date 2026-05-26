"""Materializes one Variant attempt's impl from a Hypothesis via the
variant-implementer agent.

Each call writes a fresh `attempts/NN/impl/` under the variant dir. On a
repair attempt (`prev_attempt_dir` set), the previous attempt's
segment/manifest serve as the parent and the latest rejection.md entry is
embedded into task.md so the agent can fix what failed.

The agent-side contract lives in `evolve/src/agents/variant-implementer.md`.
"""

from __future__ import annotations

import logging
from pathlib import Path

from src._io import read_utf8, write_utf8
from src.hypothesis import parser
from src.adapters import LLMAdapter
from src.paths import AttemptDir, CycleVariantDir, Layout, Variant, VariantDir

AGENT_NAME = "variant-implementer"
REQUIRED_OUTPUTS = ("segment.rs", "manifest.toml")

log = logging.getLogger("variant-implementer")


def _initial_task_md(
    attempt_dir: AttemptDir,
    variant_dir: CycleVariantDir,
    hypothesis_file: Path,
    parent_dir: VariantDir,
    parent_variant: Variant,
    layout: Layout,
) -> str:
    return (
        f"# Task: variant-implementer (attempt 1)\n\n"
        f"Variant: {variant_dir.variant_id}\n"
        f"Parent variant: {parent_variant} ({parent_dir.root})\n"
        f"Hypothesis: {hypothesis_file}\n"
        f"Output dir: {attempt_dir.impl_dir}\n\n"
        f"## Paths\n\n"
        f"- Parent segment.rs:     {parent_dir.segment_rs}\n"
        f"- Parent manifest.toml:  {parent_dir.manifest_toml}\n"
        f"- Parent fitness-vector: {parent_dir.fitness_vector_json}\n"
        f"- Parent insight.md:     {parent_dir.insight_md}\n"
        f"- Extension points:      {layout.sources.extension_points_md}\n"
        f"- Baseline segment.rs:   {layout.sources.baseline.segment_rs}\n"
        f"- Rust scaffold dir:     {layout.sources.rust_scaffold_dir}\n"
        f"  (contract gate + invariant gate test sources; read on demand)\n\n"
        f"## Write\n\n"
        f"Exactly two files under the output dir: `segment.rs` and `manifest.toml`.\n"
    )


def _latest_rejection_entry(rejection_md: Path) -> str:
    """Return the body of the last `## ...` entry in rejection.md, or an
    empty string when no entry exists yet. Used to brief the repair turn."""

    if not rejection_md.is_file():
        return ""
    content = read_utf8(rejection_md)
    marker = "\n## "
    idx = content.rfind(marker)
    if idx == -1:
        # First entry has no preceding newline before `##`.
        idx = content.find("## ")
        if idx == -1:
            return ""
    return content[idx:].lstrip("\n")


def _repair_task_md(
    attempt: int,
    attempt_dir: AttemptDir,
    variant_dir: CycleVariantDir,
    hypothesis_file: Path,
    prev_attempt_dir: AttemptDir,
    parent_variant: Variant,
    parent_dir: VariantDir,
    layout: Layout,
) -> str:
    rejection_entry = _latest_rejection_entry(variant_dir.rejection_md)
    return (
        f"# Task: variant-implementer (attempt {attempt}, repair)\n\n"
        f"Variant: {variant_dir.variant_id}\n"
        f"Parent variant: {parent_variant} ({parent_dir.root})\n"
        f"Hypothesis: {hypothesis_file}\n"
        f"Output dir: {attempt_dir.impl_dir}\n\n"
        f"## Paths\n\n"
        f"- Previous attempt segment.rs:    {prev_attempt_dir.segment_rs}\n"
        f"- Previous attempt manifest.toml: {prev_attempt_dir.manifest_toml}\n"
        f"- Original parent segment.rs:     {parent_dir.segment_rs}\n"
        f"- Original parent manifest.toml:  {parent_dir.manifest_toml}\n"
        f"- Extension points:               {layout.sources.extension_points_md}\n"
        f"- Rust scaffold dir:              {layout.sources.rust_scaffold_dir}\n"
        f"  (contract gate + invariant gate test sources; read on demand)\n\n"
        f"## Failing gate\n\n"
        f"The previous attempt was rejected by a gate. Read the rejection entry "
        f"below, identify the failing test, and fix `segment.rs` / `manifest.toml`.\n\n"
        f"{rejection_entry}\n"
        f"## Write\n\n"
        f"Exactly two files under the output dir: `segment.rs` and `manifest.toml`.\n"
    )


def implement_variant(
    hypothesis_file: Path,
    variant_dir: CycleVariantDir,
    *,
    attempt: int,
    prev_attempt_dir: AttemptDir | None = None,
    layout: Layout,
    adapter: LLMAdapter,
) -> AttemptDir:
    """Materialize one Variant attempt's impl/ from a single Hypothesis.

    Returns the AttemptDir that was just written. Raises on failure.
    """

    if not hypothesis_file.is_file():
        raise FileNotFoundError(f"hypothesis file not found: {hypothesis_file}")
    if attempt < 1:
        raise ValueError(f"attempt must be >= 1, got: {attempt}")
    if attempt > 1 and prev_attempt_dir is None:
        raise ValueError(f"prev_attempt_dir is required for attempt={attempt}")

    hypothesis = parser.parse(hypothesis_file)
    parent_dir = hypothesis.parent_variant.resolve(layout)

    attempt_dir = variant_dir.attempt(attempt)
    impl_dir = attempt_dir.impl_dir
    impl_dir.mkdir(parents=True, exist_ok=True)

    if attempt == 1:
        task_body = _initial_task_md(
            attempt_dir, variant_dir, hypothesis_file, parent_dir, hypothesis.parent_variant, layout,
        )
    else:
        assert prev_attempt_dir is not None
        task_body = _repair_task_md(
            attempt, attempt_dir, variant_dir, hypothesis_file,
            prev_attempt_dir, hypothesis.parent_variant, parent_dir, layout,
        )
    write_utf8(attempt_dir.impl_task_md, task_body)
    log.debug("wrote implementer task.md at %s", attempt_dir.impl_task_md)

    readable_roots = [
        hypothesis_file,
        parent_dir.root,
        layout.sources.dash_src,
        layout.sources.subject_dir,
        layout.sources.rust_scaffold_dir,
    ]
    if prev_attempt_dir is not None:
        readable_roots.append(prev_attempt_dir.root)

    adapter.run(
        agent_name=AGENT_NAME,
        workspace=impl_dir,
        readable_roots=readable_roots,
        allow_edits=True,
    )

    missing = [n for n in REQUIRED_OUTPUTS if not (impl_dir / n).is_file()]
    if missing:
        raise FileNotFoundError(
            f"adapter did not produce {missing} in {impl_dir}"
        )
    log.info("variant attempt %d materialized at %s", attempt, impl_dir)
    return attempt_dir
