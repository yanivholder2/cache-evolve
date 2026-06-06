"""Persist a run's resolved inputs into its output folder.

At run start the runner snapshots config.toml and the subject manifest into the
output root so each run folder is self-describing for reviewers and downstream
tooling (e.g. plots/build_data.py reads the run setup from here). CLI flags that
won over config.toml are overlaid onto the config snapshot so it reflects the
values the run actually used rather than the on-disk defaults the flags overrode.
"""

from __future__ import annotations

import argparse
import re
import shutil
from collections.abc import Mapping

from evolve._io import read_utf8
from evolve.paths import Layout


def snapshot_run_inputs(layout: Layout, config_overrides: Mapping[str, Mapping[str, object]]) -> None:
    """Copy the run's config.toml and subject manifest.toml into the output
    root. `config_overrides` carries the CLI flags that won over config.toml;
    they are applied to the config snapshot so it reflects the values the run
    actually used rather than the on-disk defaults the flags overrode.

    Write-if-absent: a resumed run (`--continue`) already has both snapshots
    from its first invocation, and they are the run's identity — never overwrite
    them with the current repo state."""

    layout.outputs.root.mkdir(parents=True, exist_ok=True)
    if not layout.outputs.config_toml.exists():
        source = read_utf8(layout.sources.config_toml)
        layout.outputs.config_toml.write_text(overlay_config(source, config_overrides), encoding="utf-8")
    if not layout.outputs.manifest_toml.exists():
        shutil.copy2(layout.sources.subject_manifest, layout.outputs.manifest_toml)


def flag_overrides(args: argparse.Namespace) -> dict[str, dict[str, object]]:
    """Map the CLI flags that were given to their config.toml (section, key) so
    the run's config snapshot can reflect the resolved values. Mirrors the
    override fields threaded into `load_configs`; flags left unset are omitted."""
    by_section = {
        "framework": {
            "subject": args.subject,
            "cycles": args.cycles,
            "cohort_size": args.cohort_size,
            "ai_agent_tool": args.ai_agent_tool,
            "model": args.model,
            "log_level": args.log_level,
            "max_workers": args.max_workers,
            "authoring_scope": args.authoring_scope,
        },
        "simulator": {"timeout_s": args.simulator_timeout_s},
    }
    overrides: dict[str, dict[str, object]] = {}
    for section, values in by_section.items():
        given = {key: value for key, value in values.items() if value is not None}
        if given:
            overrides[section] = given
    return overrides


def overlay_config(source: str, overrides: Mapping[str, Mapping[str, object]]) -> str:
    """Return config.toml `source` with `overrides` (section to key to value)
    applied, preserving comments and untouched lines. Keys present in their
    section are rewritten in place; keys absent are appended to the section
    (the section header is created if missing)."""

    pending = {
        (section, key): value
        for section, keys in overrides.items()
        for key, value in keys.items()
    }
    if not pending:
        return source

    lines = source.splitlines(keepends=True)
    section: str | None = None
    for index, line in enumerate(lines):
        header = re.match(r"\s*\[([^\]]+)\]", line)
        if header:
            section = header.group(1)
            continue
        if line.lstrip().startswith("#"):
            continue
        assignment = re.match(r"\s*([A-Za-z0-9_]+)\s*=", line)
        if assignment is None:
            continue
        key = assignment.group(1)
        if (section, key) in pending:
            value = pending.pop((section, key))
            trailer = _line_trailer(line)
            lines[index] = f"{key} = {_toml_value(value)}{trailer}"

    for (section, key), value in pending.items():
        _append_key(lines, section, key, value)
    return "".join(lines)


def _line_trailer(line: str) -> str:
    """Return the trailing part of an assignment `line` to carry onto a rewritten
    value: an inline comment with the whitespace before it, plus the line ending.
    A `#` inside the quoted value is not a comment, so quoted spans are skipped."""
    newline = line[len(line.rstrip("\n")):]
    body = line.rstrip("\n")
    in_string = False
    for index, char in enumerate(body):
        if char == '"':
            in_string = not in_string
        elif char == "#" and not in_string:
            start = index
            while start > 0 and body[start - 1] in " \t":
                start -= 1
            return body[start:] + newline
    return newline


def _toml_value(value: object) -> str:
    """Render `value` as a TOML scalar for the config snapshot. Covers only the
    flag-overridable types: bool, int, and string-likes (str, StrEnum)."""
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    return f'"{value}"'


def _append_key(lines: list[str], section: str, key: str, value: object) -> None:
    """Insert `key = value` into `section`'s block, creating the section at the
    end of `lines` if its header is absent."""
    rendered = f"{key} = {_toml_value(value)}\n"
    start = next(
        (i for i, line in enumerate(lines) if re.match(rf"\s*\[{re.escape(section)}\]", line)),
        None,
    )
    if start is None:
        if lines and not lines[-1].endswith("\n"):
            lines[-1] += "\n"
        lines.append(f"[{section}]\n")
        lines.append(rendered)
        return
    insert_at = next(
        (i for i in range(start + 1, len(lines)) if re.match(r"\s*\[", lines[i])),
        len(lines),
    )
    # Land the key with the section's other keys, before any trailing blank lines.
    while insert_at > start + 1 and not lines[insert_at - 1].strip():
        insert_at -= 1
    if insert_at > 0 and not lines[insert_at - 1].endswith("\n"):
        lines[insert_at - 1] += "\n"
    lines.insert(insert_at, rendered)
