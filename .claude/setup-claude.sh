#!/usr/bin/env bash
# Make repo skills under .agents/skills/ visible to Claude Code by linking
# .claude/skills -> ../.agents/skills.
#
# Cross-platform: works on Linux/macOS bash and on Git Bash / MSYS2 / Cygwin
# on Windows. On Windows, if `ln -s` fails to produce a real symlink (no
# Developer Mode), the script falls back to a directory junction created via
# cmd.exe, which does not require admin rights.
#
# Idempotent: safe to re-run.

# Exit on error, undefined variables, and pipeline failures.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

target="../.agents/skills"
link=".claude/skills"

mkdir -p .claude

# If the link already points where we want, exit early (covers symlinks and
# junctions, which `[ -L ]` does not detect on every platform).
if [ -L "$link" ] && [ "$(readlink "$link")" = "$target" ]; then
    echo "$link already linked to $target"
    exit 0
fi

# Anything else at the path needs to go before we create the link.
if [ -L "$link" ] || [ -e "$link" ]; then
    if [ -L "$link" ]; then
        echo "Replacing stale symlink at $link"
        rm "$link"
    else
        echo "Error: $link exists and is not a symlink. Move/delete it first." >&2
        exit 1
    fi
fi

uname_s="$(uname -s 2>/dev/null || echo unknown)"
case "$uname_s" in
    MINGW*|MSYS*|CYGWIN*)
        is_windows=1 ;;
    *)
        is_windows=0 ;;
esac

if [ "$is_windows" -eq 0 ]; then
    ln -s "$target" "$link"
    echo "Linked $link -> $target (symlink)"
    exit 0
fi

# Windows path: try a real symlink first (works with Developer Mode), then
# fall back to a directory junction via cmd.exe.
if MSYS=winsymlinks:nativestrict ln -s "$target" "$link" 2>/dev/null && [ -L "$link" ]; then
    echo "Linked $link -> $target (symlink, Developer Mode)"
    exit 0
fi

# Junction fallback. mklink needs Windows-style absolute paths.
[ -e "$link" ] && rm -rf "$link"
if ! command -v cygpath >/dev/null 2>&1; then
    echo "Error: cygpath unavailable; cannot create junction fallback." >&2
    exit 1
fi
win_link="$(cygpath -w "$REPO_ROOT/$link")"
win_target="$(cygpath -w "$REPO_ROOT/.agents/skills")"
cmd.exe //c "mklink /J \"$win_link\" \"$win_target\"" >/dev/null
echo "Linked $link -> $target (junction)"
