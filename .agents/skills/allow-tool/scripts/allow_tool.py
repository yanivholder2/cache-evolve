#!/usr/bin/env python3
"""Add an allowed-tool entry to both .claude/settings.json and .gemini/settings.json.

The argument is the Claude-style entry. The script writes that entry to
.claude/settings.json and translates it to the Gemini equivalent (where one
exists) for .gemini/settings.json.

Translation table:
    Bash(<prefix>*)  -> ShellTool(<prefix>)
    Skill(<name>)    -> (no Gemini equivalent; skipped)
    Edit(<glob>)     -> (no Gemini equivalent; skipped)
    Read / Write     -> (no Gemini equivalent; skipped)

Idempotent: re-running with an entry that already exists is a no-op.

Note: re-serializes settings.json with consistent JSON formatting, so any
hand-grouped blank lines inside arrays will be lost.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path


def repo_root() -> Path:
    out = subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True)
    return Path(out.strip())


def load_json(path: Path) -> dict:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def save_json(path: Path, data: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def add_unique(items: list, value) -> bool:
    if value in items:
        return False
    items.append(value)
    return True


def claude_to_gemini(entry: str) -> str | None:
    """Translate a Claude-style allow entry to a Gemini coreTools entry, or None
    if Gemini has no equivalent.
    """
    m = re.fullmatch(r"\s*Bash\(\s*(.*?)\s*\*?\s*\)\s*", entry)
    if m:
        prefix = m.group(1).rstrip().rstrip("*").rstrip()
        if not prefix:
            return None
        return f"ShellTool({prefix})"
    return None


def update_claude(root: Path, entry: str) -> tuple[Path, bool]:
    path = root / ".claude" / "settings.json"
    data = load_json(path)
    allow = data.setdefault("permissions", {}).setdefault("allow", [])
    added = add_unique(allow, entry)
    if added:
        save_json(path, data)
    return path, added


def update_gemini(root: Path, claude_entry: str) -> tuple[Path | None, bool, str]:
    gemini_entry = claude_to_gemini(claude_entry)
    if gemini_entry is None:
        return None, False, "no Gemini equivalent"
    path = root / ".gemini" / "settings.json"
    data = load_json(path)
    core = data.setdefault("tools", {}).setdefault("coreTools", [])
    added = add_unique(core, gemini_entry)
    if added:
        save_json(path, data)
    return path, added, gemini_entry


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("entry", help='Claude-style allow entry, e.g. "Bash(cargo bench*)" or "Skill(my-skill)"')
    parser.add_argument("--repo-root", type=Path, default=None, help="Override repo root (default: git rev-parse --show-toplevel)")
    args = parser.parse_args()

    root = args.repo_root or repo_root()

    cpath, cadded = update_claude(root, args.entry)
    print(f"allow_tool: .claude/settings.json: {'added' if cadded else 'already present'}: {args.entry}")

    gpath, gadded, info = update_gemini(root, args.entry)
    if gpath is None:
        print(f"allow_tool: .gemini/settings.json: skipped ({info}; Gemini has no equivalent for this entry shape)")
    else:
        print(f"allow_tool: .gemini/settings.json: {'added' if gadded else 'already present'}: {info}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
