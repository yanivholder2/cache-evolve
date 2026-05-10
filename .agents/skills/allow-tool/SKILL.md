---
name: allow-tool
description: Add an allowed-tool entry to both .claude/settings.json and .gemini/settings.json so Claude Code and Gemini CLI auto-approve the same operation. Use when the user says "allow X", "add permission for X", "auto-approve X". Triggers on "allow tool", "allow bash", "add permission".
argument-hint: "<Claude-style entry, e.g. Bash(cargo bench*) or Skill(my-skill) or Edit(path/**)>"
allowed-tools: Bash(python *allow_tool.py*), Bash(python3 *allow_tool.py*), Read
---

Add an allowed-tool entry to both Claude and Gemini settings in one shot, with
the necessary translation between formats.

## Usage

`$ARGUMENTS` is a Claude-style allow entry, e.g.:

- `Bash(cargo bench*)` — auto-approve a shell command prefix
- `Bash(npm run *)`
- `Skill(my-skill)` — auto-approve a skill (Claude only; Gemini has no per-skill allow-list)
- `Edit(path/**)` — auto-approve edits under a path (Claude only)

## Steps

1. Run the helper script with `$ARGUMENTS` as a single quoted argument:
   ```bash
   python3 "${CLAUDE_SKILL_DIR:-.agents/skills/allow-tool}/scripts/allow_tool.py" '$ARGUMENTS'
   ```
   On Gemini CLI, the path resolves the same way relative to the repo root.

2. Report the script's output verbatim. It tells the user what was added (or
   was already present) on each side, and which entries had no Gemini
   equivalent and were therefore skipped on the Gemini side.

## Translation rules used by the script

| Claude entry | Gemini equivalent |
|---|---|
| `Bash(<prefix>*)` | `ShellTool(<prefix>)` (in `tools.coreTools`) |
| `Skill(<name>)` | none — skipped |
| `Edit(<glob>)`, `Read(...)`, `Write(...)` | none — skipped |

The script is idempotent: re-running with an entry that's already present is a
no-op. It writes settings back with standard JSON formatting (so any
hand-grouped blank lines inside arrays will be lost).

When an entry is skipped on the Gemini side (no equivalent), the script also
appends it to a tracked list in `.gemini/SETTINGS_NOTES.md` between
`<!-- allow-tool:claude-only-begin -->` / `<!-- allow-tool:claude-only-end -->`
markers, so the notes file stays in sync with which Claude entries have no
Gemini counterpart. The section is created on first need; do not edit the
content between the markers by hand.
