"""Drives the hypotheses-writer agent and verifies its output."""

from __future__ import annotations

import json
import logging
import re
from pathlib import Path

from evolve._io import read_utf8, write_utf8
from evolve.config import AuthoringScope
from evolve.hypothesis import parser
from evolve.llm import LLMAdapter
from evolve.identifiers import HYPOTHESIS_ID_WIDTH, CycleId
from evolve.paths import Layout
from evolve.subject import Subject, load_subject

AGENT_NAME = "hypotheses-writer"
HYPOTHESIS_FILE_RE = re.compile(rf"H\d{{{HYPOTHESIS_ID_WIDTH}}}\.md")
COLD_START_CYCLE = CycleId.cold_start()

log = logging.getLogger(__name__)


def _task_md_header(cycle_id: CycleId, hypotheses_num: int, hypotheses_dir: Path, *, title_suffix: str = "") -> str:
    return (
        f"# Task: hypotheses-writer (cycle {cycle_id}{title_suffix})\n\n"
        f"Cycle: {cycle_id}\n"
        f"Number of hypotheses: {hypotheses_num}\n"
        f"Output dir: {hypotheses_dir}\n\n"
    )


def _authoring_scope_section(authoring_scope: AuthoringScope, layout: Layout) -> str:
    """Tell the proposer which overlay files its hypotheses may edit, so it does
    not propose changes outside the writable surface. Derived subject-agnostically
    from the subject manifest."""
    subject = load_subject(layout.sources.subject_dir)
    pinned = ", ".join(subject.pinned_overlay_display)
    lines = ["## Authoring scope\n\n"]
    if authoring_scope is AuthoringScope.EXTENSION_POINTS:
        whitelist = sorted(str(path) for path in subject.ep_scope_whitelist)
        lines.append(
            "Hypotheses must confine their edits to the extension-point impl files "
            "below. Propose only ideas expressible within these files:\n"
        )
        for path in whitelist:
            lines.append(f"- {path}\n")
        lines.append(
            f"\nThe gate-integrity surface is writable by no scope ({pinned}). "
            "Do not propose ideas that require editing it.\n\n"
        )
    else:
        lines.append(
            "Any overlay file may be edited, except the gate-integrity surface, "
            f"writable by no scope ({pinned}). Do not propose ideas that require "
            "editing it.\n\n"
        )
    return "".join(lines)


def _load_cross_cycle_winners(layout: Layout) -> dict | None:
    """The cross-cycle winners artifact, or None when absent."""
    cross_cycle_winners_json = layout.outputs.cross_cycle_winners_json
    if not cross_cycle_winners_json.is_file():
        return None
    return json.loads(read_utf8(cross_cycle_winners_json))


def _incumbents_section(cross_cycle_winners: dict) -> str:
    """The cross-cycle Winners as measured anchors plus a hard parent quota."""
    front = cross_cycle_winners["winners"]
    by_variant = {
        fitness_vector["variant"]: fitness_vector
        for fitness_vector in cross_cycle_winners["fitness_vectors"]
    }
    lines = ["## Incumbents (cross-cycle winners)\n\n"]
    for variant_id in front:
        fitness_vector = by_variant.get(variant_id)
        if fitness_vector is None:
            lines.append(f"- {variant_id}\n")
            continue
        summaries = ", ".join(
            f"{dimension['dim']}={dimension['summary']}"
            for dimension in fitness_vector["dimensions"]
        )
        lines.append(f"- {variant_id}: {summaries}\n")
    non_baseline = [variant_id for variant_id in front if variant_id != "baseline"]
    quota = (
        "At least one of the hypotheses MUST cite a cross-cycle winner above as "
        "its `## Parent`."
    )
    if non_baseline:
        quota += (
            " Prefer the non-baseline incumbents — they are the proven best-so-far. "
        )
    else:
        quota += " "
    quota += (
        "The remaining hypotheses may explore from `baseline` or prior losers.\n\n"
    )
    lines.append("\n")
    lines.append(quota)
    return "".join(lines)


def _write_cold_start_task_md(
    hypotheses_num: int, layout: Layout, authoring_scope: AuthoringScope | None
) -> None:
    cycle_layout = layout.outputs.cycle(COLD_START_CYCLE)
    baseline_layout = layout.sources.baseline
    body = _task_md_header(
        COLD_START_CYCLE, hypotheses_num, cycle_layout.hypotheses_dir,
        title_suffix=" — cold start",
    ) + (
        f"## Framing\n\n"
        f"This is the first cycle. No prior variants exist. You are proposing the "
        f"first batch of {hypotheses_num} hypotheses against the permanent research "
        f"baseline below. Prefer breadth — distinct strategies for bucket-movement — "
        f"over depth on any single idea.\n\n"
        f"Every hypothesis in this cycle MUST cite `baseline` as its parent variant. "
        f"No other parent variant is valid in cycle {COLD_START_CYCLE}.\n\n"
    )
    if authoring_scope is not None:
        body += _authoring_scope_section(authoring_scope, layout)
    body += (
        f"## Read for context\n\n"
        f"- {layout.sources.subject_src} — subject Rust source. The code your hypotheses propose to overlay.\n"
        f"- {layout.sources.subject_manifest} — subject manifest (extension points, baseline insight, fitness vector path).\n"
        f"- {baseline_layout.fitness_vector_json} — baseline's measured fitness vector. Numeric anchor your hypotheses target dimensions of.\n"
        f"- {baseline_layout.insight_md} — short prose describing what the baseline does.\n"
    )
    write_utf8(cycle_layout.hypotheses_task_md, body)


def _write_continuation_task_md(
    hypotheses_num: int,
    cycle_id: CycleId,
    layout: Layout,
    authoring_scope: AuthoringScope | None,
) -> dict | None:
    """Write the continuation task.md; return the cross_cycle_winners artifact (or None)
    so the caller can enforce the parent-anchor quota on the produced files."""
    cycle_layout = layout.outputs.cycle(cycle_id)
    prev_cycle_id = cycle_id.prev()
    prev_cycle_layout = layout.outputs.cycle(prev_cycle_id)
    if not prev_cycle_layout.variants_dir.is_dir():
        raise FileNotFoundError(
            f"cycle {cycle_id} expects previous cycle at {prev_cycle_layout.root}, "
            f"but {prev_cycle_layout.variants_dir} is missing"
        )
    cross_cycle_winners = _load_cross_cycle_winners(layout)
    has_front = cross_cycle_winners is not None and bool(cross_cycle_winners["winners"])
    prev_variant_template = prev_cycle_layout.variant_template
    body = _task_md_header(
        cycle_id, hypotheses_num, cycle_layout.hypotheses_dir,
    ) + (
        f"## Framing\n\n"
        f"You are proposing the next batch of {hypotheses_num} hypotheses. Each "
        f"hypothesis MUST cite exactly one parent variant: either `baseline` or a past "
        f"variant `cycle/<NNNN>/v<NNN>` from any prior cycle (1..{prev_cycle_id.value}). "
    )
    if has_front:
        body += "Aim for distinct strategies — near-duplicates collapse the cohort's winners.\n\n"
    else:
        body += (
            "Prefer refining recent winners or diversifying from recent losers; aim for "
            "distinct strategies — near-duplicates collapse the cohort's winners.\n\n"
        )
    if authoring_scope is not None:
        body += _authoring_scope_section(authoring_scope, layout)
    if has_front:
        body += _incumbents_section(cross_cycle_winners)
    body += (
        f"## Read for context\n\n"
        f"### Previous cycle ({prev_cycle_id})\n\n"
        f"- {prev_cycle_layout.variants_dir} — every variant directory `v<NNN>/` under this path. For each variant read:\n"
        f"  - {prev_cycle_layout.variant_overlay_template} — the variant's overlay (subject files it replaced); read its latest attempt.\n"
        f"  - {prev_variant_template.fitness_vector_json} — its measured fitness vector.\n"
        f"  - {prev_variant_template.insight_md} — one-line fitness summary.\n"
        f"- {prev_cycle_layout.cycle_winners_json} — Winners + per-variant fitness vectors for the cycle.\n"
        f"- {prev_cycle_layout.cycle_insight_md} — cycle-level summary.\n\n"
        f"### Cross-cycle history\n\n"
        f"- {layout.outputs.leaderboard_md} — current winner variants and fitness summaries across all cycles.\n"
        f"- {layout.outputs.ledger} — one JSON row per completed cycle (cohort size, winners, insights).\n"
        f"- {layout.outputs.cross_cycle_insights_md} — long-running narrative of what has been learned across cycles. Read if present.\n\n"
        f"### Baseline\n\n"
        f"- {layout.sources.baseline.root} — the permanent baseline (always a valid parent variant).\n"
    )
    write_utf8(cycle_layout.hypotheses_task_md, body)
    return cross_cycle_winners if has_front else None


def write_hypotheses(
    hypotheses_num: int,
    *,
    layout: Layout,
    cycle_id: CycleId,
    adapter: LLMAdapter,
    authoring_scope: AuthoringScope | None = None,
    attempts: int = 1,
) -> list[Path]:
    """Produce `hypotheses_num` H<NN>.md files; return their paths.

    When `authoring_scope` is given, the task.md states which overlay files the
    proposer's hypotheses may edit. Under the extension-points scope, a produced
    batch whose `## Changes` names a file outside the writable surface is
    regenerated (up to `attempts` total) with a corrective directive appended to
    the task.md — catching the out-of-scope proposal before it costs an
    implementer call. On a continuation cycle with non-empty cross-cycle
    winners, at least one produced hypothesis must cite a winner variant as its
    parent (anchoring the cohort on the proven best-so-far).

    Raises:
        ValueError: `hypotheses_num` < 1, adapter wrote the wrong file count,
            hypotheses still name out-of-scope files after `attempts`, or no
            produced hypothesis anchors on a required winner variant.
    """

    if hypotheses_num < 1:
        raise ValueError(f"hypotheses_num must be >= 1, got {hypotheses_num}")

    cycle_layout = layout.outputs.cycle(cycle_id)
    hypotheses_dir = cycle_layout.hypotheses_dir
    hypotheses_dir.mkdir(parents=True, exist_ok=True)
    log.info("generating %d hypotheses for cycle %s into %s", hypotheses_num, cycle_id, hypotheses_dir)

    cross_cycle_winners: dict | None = None
    if cycle_id == COLD_START_CYCLE:
        _write_cold_start_task_md(hypotheses_num, layout, authoring_scope)
    else:
        cross_cycle_winners = _write_continuation_task_md(hypotheses_num, cycle_id, layout, authoring_scope)
    log.debug("wrote task spec: %s", cycle_layout.hypotheses_task_md)

    subject = (
        load_subject(layout.sources.subject_dir)
        if authoring_scope is AuthoringScope.EXTENSION_POINTS
        else None
    )

    files: list[Path] = []
    for attempt in range(1, attempts + 1):
        adapter.run(
            agent_name=AGENT_NAME,
            workspace=hypotheses_dir,
            readable_roots=[layout.outputs_root, layout.sources.subject_src, layout.sources.subject_dir],
            allow_edits=True,
        )
        files = _collect_hypothesis_files(hypotheses_dir, hypotheses_num)
        violations = _scope_violations(files, subject, authoring_scope)
        if not violations:
            break
        log.warning(
            "hypotheses name out-of-scope files on attempt %d of %d: %s",
            attempt, attempts, violations,
        )
        if attempt == attempts:
            raise ValueError(
                f"hypotheses still name files outside the authoring scope after {attempts} "
                f"attempts: {violations}; writable surface: "
                f"{sorted(str(path) for path in subject.ep_scope_whitelist)}"
            )
        write_utf8(
            cycle_layout.hypotheses_task_md,
            read_utf8(cycle_layout.hypotheses_task_md) + _scope_correction(violations, subject),
        )
    log.info("adapter produced %d hypothesis files in %s", len(files), hypotheses_dir)

    if cycle_id != COLD_START_CYCLE and cross_cycle_winners is not None:
        _enforce_parent_quota(files, cross_cycle_winners)
    return files


def _collect_hypothesis_files(hypotheses_dir: Path, hypotheses_num: int) -> list[Path]:
    """The `hypotheses_num` H<NN>.md files the adapter wrote; raise on any other count."""
    entries = sorted(p for p in hypotheses_dir.iterdir() if p.is_file())
    files = [p for p in entries if HYPOTHESIS_FILE_RE.fullmatch(p.name)]
    if len(files) != hypotheses_num:
        agent_log = hypotheses_dir / f"{AGENT_NAME}.log.md"
        other = [p.name for p in entries if not HYPOTHESIS_FILE_RE.fullmatch(p.name)]
        raise ValueError(
            f"adapter wrote {len(files)} H<NN>.md files in {hypotheses_dir}, expected {hypotheses_num}; "
            f"matched={[p.name for p in files]}; other_files={other}; agent_log={agent_log}"
        )
    return files


def _scope_violations(
    files: list[Path], subject: Subject | None, authoring_scope: AuthoringScope | None
) -> dict[str, list[str]]:
    """Per-hypothesis, the out-of-scope source files its `## Changes` names.

    Only the extension-points scope constrains the writable surface, so any other
    scope yields no violations. A referenced file is out of scope when its
    basename is not among the extension-point whitelist's. Matching by basename
    keeps an in-scope proposal from being rejected whether it writes the path in
    full or bare; the implementer's exact-path check is the backstop."""
    if authoring_scope is not AuthoringScope.EXTENSION_POINTS or subject is None:
        return {}
    whitelist = subject.ep_scope_whitelist
    allowed = {path.name for path in whitelist}
    suffixes = frozenset(path.suffix for path in whitelist)
    violations: dict[str, list[str]] = {}
    for path in files:
        referenced = parser.referenced_files(parser.parse(path).changes, suffixes)
        outside = sorted({token for token in referenced if Path(token).name not in allowed})
        if outside:
            violations[path.name] = outside
    return violations


def _scope_correction(violations: dict[str, list[str]], subject: Subject) -> str:
    """A task.md directive steering regeneration back inside the writable surface."""
    whitelist = sorted(str(path) for path in subject.ep_scope_whitelist)
    lines = [
        "\n## Scope correction\n\n",
        "The previous attempt named files outside the writable surface. Rewrite "
        "every hypothesis so its `## Changes` edits only these extension-point "
        "impl files:\n",
    ]
    for path in whitelist:
        lines.append(f"- {path}\n")
    lines.append("\nOut-of-scope files named last attempt:\n")
    for name, outside in sorted(violations.items()):
        lines.append(f"- {name}: {outside}\n")
    lines.append(
        "\nExpress the same idea through one of the in-scope files instead.\n"
    )
    return "".join(lines)


def _enforce_parent_quota(files: list[Path], cross_cycle_winners: dict) -> None:
    """At least one produced hypothesis must parent on a required winner anchor:
    the non-baseline incumbents when any exist, else the full winner set."""
    front = cross_cycle_winners["winners"]
    non_baseline = {variant_id for variant_id in front if variant_id != "baseline"}
    required_anchors = non_baseline if non_baseline else set(front)
    parents = [str(parser.parse(path).parent_variant) for path in files]
    if not any(parent in required_anchors for parent in parents):
        raise ValueError(
            f"no produced hypothesis cites a required winner anchor; "
            f"parents seen={parents}; required anchors={sorted(required_anchors)}"
        )
