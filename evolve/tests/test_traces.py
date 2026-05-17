"""Trace suite invariants.

These tests pin contracts the rest of the pipeline relies on:
- dev/holdout directories are populated and parse cleanly
- every trace has an accompanying *.profile.md with a hidden flag
- holdout profiles are marked hidden: true (the AI must not see them)
- trace files contain only u64 keys + comments + blanks
"""

import re
from pathlib import Path

TRACES_ROOT = Path(__file__).parent.parent / "traces"


def _parse_frontmatter(text: str) -> dict:
    if not text.startswith("---"):
        return {}
    end = text.find("\n---", 3)
    if end == -1:
        return {}
    block = text[3:end].strip()
    out: dict[str, str] = {}
    for line in block.splitlines():
        if ":" not in line:
            continue
        k, _, v = line.partition(":")
        out[k.strip()] = v.strip()
    return out


def _traces_in(suite: str) -> list[Path]:
    return sorted((TRACES_ROOT / suite).glob("*.trace"))


def test_dev_suite_has_multiple_traces():
    traces = _traces_in("dev")
    assert 4 <= len(traces) <= 6, f"dev suite should have 4-6 traces, got {len(traces)}"


def test_holdout_suite_has_multiple_traces():
    traces = _traces_in("holdout")
    assert 2 <= len(traces) <= 4, f"holdout suite should have 2-3 traces, got {len(traces)}"


def test_every_trace_has_profile():
    for suite in ("dev", "holdout"):
        for trace in _traces_in(suite):
            profile = trace.with_suffix(".profile.md")
            assert profile.is_file(), f"missing profile for {trace}"


def test_dev_profiles_are_not_hidden():
    for trace in _traces_in("dev"):
        profile = trace.with_suffix(".profile.md")
        fm = _parse_frontmatter(profile.read_text())
        assert fm.get("hidden") == "false", f"{profile} must declare hidden: false"


def test_holdout_profiles_are_hidden():
    for trace in _traces_in("holdout"):
        profile = trace.with_suffix(".profile.md")
        fm = _parse_frontmatter(profile.read_text())
        assert fm.get("hidden") == "true", f"{profile} must declare hidden: true"


def test_at_least_one_profile_has_signoff():
    """AC: trace-profile sign-off workflow has produced at least one signed profile."""
    signed = []
    for suite in ("dev", "holdout"):
        for trace in _traces_in(suite):
            fm = _parse_frontmatter(trace.with_suffix(".profile.md").read_text())
            if fm.get("signed_off_by"):
                signed.append(trace.name)
    assert signed, "no trace profile carries a maintainer sign-off"


def test_trace_files_are_well_formed():
    key_re = re.compile(r"^\d+$")
    for suite in ("dev", "holdout"):
        for trace in _traces_in(suite):
            lines = trace.read_text().splitlines()
            key_lines = [ln for ln in lines if ln and not ln.startswith("#")]
            assert key_lines, f"{trace} has no key lines"
            for ln in key_lines:
                assert key_re.match(ln), f"{trace}: non-u64 key line {ln!r}"


def test_signoff_workflow_documented():
    """The sign-off workflow must be discoverable from the trace suite README."""
    readme = TRACES_ROOT / "README.md"
    assert readme.is_file(), "evolve/traces/README.md must document the sign-off workflow"
    text = readme.read_text().lower()
    assert "sign-off" in text or "signed_off_by" in text, "README must describe sign-off"
    assert "hidden" in text, "README must describe held-out hidden flag"
