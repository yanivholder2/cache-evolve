"""Parses H<NN>.md hypothesis files into typed values."""

from __future__ import annotations

import logging
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Union

from src.identifiers import (
    CYCLE_ID_WIDTH,
    VARIANT_ID_WIDTH,
    CycleId,
    VariantId,
)
from src.paths import Layout

log = logging.getLogger("hypothesis-parser")

BASELINE_STR = "baseline"
_PARENT_CYCLE_RE = re.compile(
    rf"^cycle/(\d{{{CYCLE_ID_WIDTH}}})/v(\d{{{VARIANT_ID_WIDTH}}})$"
)
_SECTION_HEADER_RE = re.compile(r"^##\s+(.+?)\s*$")


@dataclass(frozen=True)
class Baseline:
    """The permanent research baseline. Always a valid parent variant."""

    def resolve(self, layout: Layout) -> Path:
        return layout.baseline.root

    def __str__(self) -> str:
        return BASELINE_STR


@dataclass(frozen=True)
class CycleVariant:
    """A past variant from a prior cycle."""

    cycle_id: CycleId
    variant_id: VariantId

    def resolve(self, layout: Layout) -> Path:
        path = layout.cycle(self.cycle_id).variant(self.variant_id).root
        if not path.is_dir():
            raise FileNotFoundError(f"parent variant dir not found: {path}")
        return path

    def __str__(self) -> str:
        return f"cycle/{self.cycle_id}/{self.variant_id}"


ParentVariant = Union[Baseline, CycleVariant]


@dataclass(frozen=True)
class Hypothesis:
    """Parsed contents of an H<NN>.md file."""

    claim: str
    """The testable claim — one-line summary (first non-empty, non-header line of the preamble)."""

    parent_variant: ParentVariant
    """Parent variant — either `Baseline()` or `CycleVariant(...)`. Resolve to a directory with `.resolve(layout)`."""

    changes: str
    """Free-form prose body of the ``## Changes`` section, describing the modifications the implementer must apply."""


def parse(path: Path) -> Hypothesis:
    log.debug("parsing hypothesis file %s", path)
    text = path.read_text(encoding="utf-8")
    sections = _split_sections(text)
    log.debug("found %d sections in %s: %s", len(sections), path, sorted(sections))

    claim = _extract_claim(sections.get("", ""))
    if claim is None:
        raise ValueError(f"{path}: missing claim")
    log.debug("claim for %s: %r", path, claim)

    parent_body = sections.get("Parent")
    if parent_body is None:
        raise ValueError(f"{path}: missing '## Parent' section")
    parent_line = _first_non_empty(parent_body)
    if parent_line is None:
        raise ValueError(f"{path}: '## Parent' section is empty")
    try:
        parent_variant = _parse_parent_variant(parent_line)
    except ValueError as e:
        raise ValueError(f"{path}: {e}") from e
    log.debug("parent variant for %s: %s", path, parent_variant)

    changes_body = sections.get("Changes")
    if changes_body is None:
        raise ValueError(f"{path}: missing '## Changes' section")
    changes = changes_body.strip()
    if not changes:
        raise ValueError(f"{path}: '## Changes' section is empty")

    log.info("parsed hypothesis %s claim=%r parent=%s changes=%d chars",
             path.name, claim, parent_variant, len(changes))
    return Hypothesis(claim=claim, parent_variant=parent_variant, changes=changes)


def _parse_parent_variant(line: str) -> ParentVariant:
    if line == BASELINE_STR:
        return Baseline()
    m = _PARENT_CYCLE_RE.match(line)
    if m:
        return CycleVariant(
            cycle_id=CycleId(int(m.group(1))),
            variant_id=VariantId(int(m.group(2))),
        )
    raise ValueError(
        f"malformed parent-variant identifier {line!r}; "
        f"expected '{BASELINE_STR}' or 'cycle/<NNNN>/v<NNN>'"
    )


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
