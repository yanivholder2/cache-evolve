"""Parse H<NN>.md hypothesis files.

Expected format (see `agents/hypotheses-writer.md`):

    # H<NN>                   (optional)

    <claim>                   # first non-empty non-header line

    ## Parent

    baseline                  # or: cycle/<NNNN>/v<NNN>

    ## Changes

    <free-form prose>

The variant-implementer driver calls `parse()` to extract the parent
citation, then `resolve_parent_dir()` to compute the dir that must be
included in the agent's readable_roots.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

from src.paths import Layout

BASELINE = "baseline"
_PARENT_CYCLE_RE = re.compile(r"^cycle/(\d{4})/v(\d{3})$")
_SECTION_HEADER_RE = re.compile(r"^##\s+(.+?)\s*$")


@dataclass(frozen=True)
class Hypothesis:
    """Parsed contents of an H<NN>.md file."""

    claim: str
    """The testable claim — one-line summary (first non-empty, non-header line of the preamble)."""

    parent: str
    """Parent citation — either ``"baseline"`` or ``"cycle/<NNNN>/v<NNN>"``. Resolve to a directory with `resolve_parent_dir`."""

    changes: str
    """Free-form prose body of the ``## Changes`` section, describing the modifications the implementer must apply."""


def parse(path: Path) -> Hypothesis:
    text = path.read_text(encoding="utf-8")
    sections = _split_sections(text)

    claim = _extract_claim(sections.get("", ""))
    if claim is None:
        raise ValueError(f"{path}: missing claim")

    parent_body = sections.get("Parent")
    if parent_body is None:
        raise ValueError(f"{path}: missing '## Parent' section")
    parent = _first_non_empty(parent_body)
    if parent is None:
        raise ValueError(f"{path}: '## Parent' section is empty")
    if parent != BASELINE and not _PARENT_CYCLE_RE.match(parent):
        raise ValueError(
            f"{path}: malformed parent identifier {parent!r}; "
            f"expected '{BASELINE}' or 'cycle/<NNNN>/v<NNN>'"
        )

    changes_body = sections.get("Changes")
    if changes_body is None:
        raise ValueError(f"{path}: missing '## Changes' section")
    changes = changes_body.strip()
    if not changes:
        raise ValueError(f"{path}: '## Changes' section is empty")

    return Hypothesis(claim=claim, parent=parent, changes=changes)


def resolve_parent_dir(parent: str, layout: Layout) -> Path:
    if parent == BASELINE:
        return layout.baseline.root
    m = _PARENT_CYCLE_RE.match(parent)
    if not m:
        raise ValueError(
            f"malformed parent identifier {parent!r}; "
            f"expected '{BASELINE}' or 'cycle/<NNNN>/v<NNN>'"
        )
    cycle_num, variant_num = m.group(1), m.group(2)
    parent_dir = layout.cycle(cycle_num).variant(int(variant_num)).root
    if not parent_dir.is_dir():
        raise FileNotFoundError(f"parent variant dir not found: {parent_dir}")
    return parent_dir


def _split_sections(text: str) -> dict[str, str]:
    """Split markdown by `^## ` headers. Preamble (lines before the first
    `## `) lives under the empty-string key."""

    buckets: dict[str, list[str]] = {"": []}
    current = ""
    for line in text.splitlines():
        m = _SECTION_HEADER_RE.match(line)
        if m:
            current = m.group(1)
            buckets.setdefault(current, [])
        else:
            buckets[current].append(line)
    return {name: "\n".join(lines) for name, lines in buckets.items()}


def _extract_claim(preamble: str) -> str | None:
    """First non-empty, non-`#`-header, non-list-marker line."""

    for line in preamble.splitlines():
        s = line.strip()
        if not s or s.startswith("#"):
            continue
        return s
    return None


def _first_non_empty(body: str) -> str | None:
    for line in body.splitlines():
        s = line.strip()
        if s:
            return s
    return None
