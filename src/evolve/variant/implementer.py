"""Materializes one Variant's overlay from a Hypothesis via the variant-implementer agent.

The agent-side contract lives in `src/evolve/agents/variant-implementer.md`.
"""

from __future__ import annotations

import logging
from pathlib import Path

from evolve._io import write_utf8
from evolve.config import AuthoringScope
from evolve.hypothesis import parser
from evolve.llm import LLMAdapter
from evolve.paths import CycleVariantDir, Layout, Variant, VariantDir
from evolve.subject import load_subject

AGENT_NAME = "variant-implementer"

log = logging.getLogger("variant-implementer")


def _write_task_md(
    variant_dir: CycleVariantDir,
    hypothesis_file: Path,
    parent_dir: VariantDir,
    variant: Variant,
    layout: Layout,
    authoring_scope: AuthoringScope,
) -> None:
    body = (
        f"# Task: variant-implementer\n\n"
        f"Variant: {variant_dir.variant_id}\n"
        f"Parent variant: {variant} ({parent_dir.root})\n"
        f"Hypothesis: {hypothesis_file}\n"
        f"Output dir: {variant_dir.files_dir} (overlay-by-path)\n\n"
        f"## Paths\n\n"
        f"- Parent files/:         {parent_dir.files_dir}\n"
        f"- Parent fitness-vector: {parent_dir.fitness_vector_json}\n"
        f"- Parent insight.md:     {parent_dir.insight_md}\n"
        f"- Subject src/:          {layout.sources.subject_src}\n"
        f"- Subject manifest:      {layout.sources.subject_manifest}\n\n"
        f"## Authoring scope\n\n"
        f"{authoring_scope}\n\n"
        f"## Write\n\n"
        f"Place every changed file under `files/<path-within-subject>`; the overlay\n"
        f"is applied verbatim to a copy of the subject crate at build time.\n"
    )
    write_utf8(variant_dir.task_md, body)


def implement_variant(
    hypothesis_file: Path,
    variant_dir: CycleVariantDir,
    *,
    layout: Layout,
    adapter: LLMAdapter,
    authoring_scope: AuthoringScope,
) -> None:
    """Materialize one Variant's overlay from a single Hypothesis. Raises on failure."""

    if not hypothesis_file.is_file():
        raise FileNotFoundError(f"hypothesis file not found: {hypothesis_file}")

    hypothesis = parser.parse(hypothesis_file)
    parent_dir = hypothesis.parent_variant.resolve(layout)
    subject = load_subject(layout.sources.subject_dir)

    files_dir = variant_dir.files_dir
    files_dir.mkdir(parents=True, exist_ok=True)
    _write_task_md(variant_dir, hypothesis_file, parent_dir, hypothesis.parent_variant, layout, authoring_scope)
    log.debug("wrote implementer task.md at %s", variant_dir.task_md)

    adapter.run(
        agent_name=AGENT_NAME,
        workspace=files_dir,
        readable_roots=[hypothesis_file, parent_dir.root, layout.sources.subject_src, layout.sources.subject_dir],
        allow_edits=True,
    )

    overlay_files = [p for p in files_dir.rglob("*") if p.is_file()]
    if not overlay_files:
        # An empty overlay would build but be indistinguishable from the
        # baseline, so treat "agent wrote nothing" as a failure.
        raise FileNotFoundError(
            f"adapter produced no overlay files under {files_dir}"
        )

    # The manifest's contract-protected files are the fixed seam between the
    # Rust cache and the Caffeine Java side: the JNI entry points and the
    # factory that constructs the cache. Their function signatures and exported
    # symbol names are a binding contract — if a variant changes them, Caffeine
    # still loads the library but silently fails to call into the cache, so
    # every request misses. No authoring scope (not even full-autonomy) may
    # touch them.
    protected = sorted(
        str(p.relative_to(files_dir))
        for p in overlay_files
        if p.relative_to(files_dir) in subject.protected_files
    )
    if protected:
        raise ValueError(
            f"variant wrote contract-protected files (the fixed JNI/factory seam "
            f"between the cache and Caffeine, writable by no scope): {protected}"
        )

    if authoring_scope is AuthoringScope.EXTENSION_POINTS:
        whitelist = subject.ep_scope_whitelist
        outside = sorted(
            str(p.relative_to(files_dir))
            for p in overlay_files
            if p.relative_to(files_dir) not in whitelist
        )
        if outside:
            raise ValueError(
                f"authoring scope '{AuthoringScope.EXTENSION_POINTS}' permits writing only the manifest "
                f"extension-point impl files; variant wrote outside that set: {outside}"
            )

    log.info("variant materialized at %s", files_dir)
