"""Materializes one Variant's overlay from a Hypothesis via the variant-implementer agent.

The agent-side contract lives in `src/evolve/agents/variant-implementer.md`.

Each invocation materialises one *attempt* under the variant's
`attempts/NN/` directory: the agent's workspace is the attempt dir, into which
it writes its overlay under `overlay/` (sibling of `task.md` and the agent
log). On a repair attempt the previous attempt's overlay is the parent the
agent fixes, and the failing gate's rejection is embedded in `task.md`.
"""

from __future__ import annotations

import logging
import re
from pathlib import Path

from evolve._io import read_utf8, write_utf8
from evolve.config import AuthoringScope
from evolve.hypothesis import parser
from evolve.llm import LLMAdapter
from evolve.paths import AttemptDir, CycleVariantDir, Layout, Variant, VariantDir
from evolve.subject import Subject, load_subject
from evolve.variant.gates import CargoTestGate, build_gate_list

AGENT_NAME = "variant-implementer"

log = logging.getLogger(__name__)


def _latest_rejection_entry(rejection_md: Path) -> str | None:
    """The text of the most recent `## …` entry in rejection.md, or None."""
    if not rejection_md.is_file():
        return None
    text = read_utf8(rejection_md)
    idx = text.rfind("\n## ")
    entry = text[idx:] if idx != -1 else text
    entry = entry.strip()
    return entry or None


def _gate_name_from_entry(entry: str) -> str | None:
    match = re.search(r"\*\*Gate\*\*: (.+)", entry)
    if not match:
        return None
    return match.group(1).strip().removesuffix(" (harness error)")


def _scaffold_for_gate(subject: Subject, gate_name: str, layout: Layout) -> Path | None:
    """The subject test scaffold backing `gate_name`, or None (e.g. build gate)."""
    for gate in build_gate_list(subject):
        if gate.name == gate_name and isinstance(gate, CargoTestGate):
            return layout.sources.subject_dir / "tests" / f"{gate.test_name}.rs"
    return None


def _repair_section(
    rejection_md: Path,
    prev_attempt_dir: AttemptDir,
    subject: Subject,
    layout: Layout,
) -> str:
    entry = _latest_rejection_entry(rejection_md)
    lines = [
        "## Failing gate\n",
        "The previous attempt's overlay was rejected. Diagnose and fix it:\n",
        f"- Previous overlay (the parent you are fixing): {prev_attempt_dir.overlay_dir}\n",
    ]
    if entry:
        gate_name = _gate_name_from_entry(entry)
        if gate_name:
            scaffold = _scaffold_for_gate(subject, gate_name, layout)
            if scaffold is not None:
                lines.append(f"- Failing gate's test scaffold: {scaffold}\n")
        lines.append("\nLatest rejection entry:\n\n")
        lines.append(entry)
        lines.append("\n")
    return "".join(lines) + "\n"


def _write_task_md(
    attempt_dir: AttemptDir,
    variant_dir: CycleVariantDir,
    hypothesis_file: Path,
    parent_dir: VariantDir,
    variant: Variant,
    layout: Layout,
    authoring_scope: AuthoringScope,
    subject: Subject,
    prev_attempt_dir: AttemptDir | None,
) -> None:
    body = (
        f"# Task: variant-implementer\n\n"
        f"Variant: {variant_dir.variant_id}\n"
        f"Parent variant: {variant} ({parent_dir.root})\n"
        f"Hypothesis: {hypothesis_file}\n"
        f"Output dir: {attempt_dir.overlay_dir} (overlay-by-path)\n\n"
        f"## Paths\n\n"
        f"- Parent overlay/:       {parent_dir.overlay_dir}\n"
        f"- Parent fitness-vector: {parent_dir.fitness_vector_json}\n"
        f"- Parent insight.md:     {parent_dir.insight_md}\n"
        f"- Subject src/:          {layout.sources.subject_src}\n"
        f"- Subject manifest:      {layout.sources.subject_manifest}\n\n"
        f"## Authoring scope\n\n"
        f"{authoring_scope}\n\n"
    )
    if prev_attempt_dir is not None:
        body += _repair_section(variant_dir.rejection_md, prev_attempt_dir, subject, layout)
    body += (
        f"## Write\n\n"
        f"Place every changed file under `overlay/<path-within-subject>`; the overlay\n"
        f"is applied verbatim to a copy of the subject crate at build time.\n"
    )
    write_utf8(attempt_dir.task_md, body)


def implement_variant(
    hypothesis_file: Path,
    variant_dir: CycleVariantDir,
    *,
    layout: Layout,
    adapter: LLMAdapter,
    authoring_scope: AuthoringScope,
    attempt: int = 1,
    prev_attempt_dir: AttemptDir | None = None,
) -> None:
    """Materialize one attempt of a Variant's overlay from a single Hypothesis.

    Writes to `variant_dir.attempt(attempt).overlay_dir`. When `prev_attempt_dir`
    is given (a repair turn), the previous attempt's overlay is added to the
    agent's readable scope and the failing gate is embedded in `task.md`.
    Raises on failure.
    """

    if not hypothesis_file.is_file():
        raise FileNotFoundError(f"hypothesis file not found: {hypothesis_file}")

    hypothesis = parser.parse(hypothesis_file)
    parent_dir = hypothesis.parent_variant.resolve(layout)
    subject = load_subject(layout.sources.subject_dir)

    attempt_dir = variant_dir.attempt(attempt)
    overlay_dir = attempt_dir.overlay_dir
    overlay_dir.mkdir(parents=True, exist_ok=True)
    _write_task_md(
        attempt_dir, variant_dir, hypothesis_file, parent_dir, hypothesis.parent_variant,
        layout, authoring_scope, subject, prev_attempt_dir,
    )
    log.debug("wrote implementer task.md at %s", attempt_dir.task_md)

    readable_roots = [
        hypothesis_file, parent_dir.root, layout.sources.subject_src, layout.sources.subject_dir,
    ]
    if prev_attempt_dir is not None:
        readable_roots.append(prev_attempt_dir.overlay_dir)

    adapter.run(
        agent_name=AGENT_NAME,
        workspace=attempt_dir.root,
        readable_roots=readable_roots,
        allow_edits=True,
    )

    overlay_files = [p for p in overlay_dir.rglob("*") if p.is_file()]
    if not overlay_files:
        # An empty overlay would build but be indistinguishable from the
        # baseline, so treat "agent wrote nothing" as a failure.
        raise FileNotFoundError(
            f"adapter produced no overlay files under {overlay_dir}"
        )

    # The manifest's contract-protected files are the fixed seam between the
    # Rust cache and the Caffeine Java side: the JNI entry points and the
    # factory that constructs the cache. Their function signatures and exported
    # symbol names are a binding contract — if a variant changes them, Caffeine
    # still loads the library but silently fails to call into the cache, so
    # every request misses. No authoring scope (not even full-autonomy) may
    # touch them.
    protected = sorted(
        str(p.relative_to(overlay_dir))
        for p in overlay_files
        if p.relative_to(overlay_dir) in subject.protected_files
    )
    if protected:
        raise ValueError(
            f"variant wrote contract-protected files (the fixed JNI/factory seam "
            f"between the cache and Caffeine, writable by no scope): {protected}"
        )

    if authoring_scope is AuthoringScope.EXTENSION_POINTS:
        whitelist = subject.ep_scope_whitelist
        outside = sorted(
            str(p.relative_to(overlay_dir))
            for p in overlay_files
            if p.relative_to(overlay_dir) not in whitelist
        )
        if outside:
            raise ValueError(
                f"authoring scope '{AuthoringScope.EXTENSION_POINTS}' permits writing only the manifest "
                f"extension-point impl files; variant wrote outside that set: {outside}"
            )

    log.info("variant materialized at %s", overlay_dir)
