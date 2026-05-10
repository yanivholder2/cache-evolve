# Gemini settings translation notes

`.gemini/settings.json` mirrors `.claude/settings.json` where Gemini CLI has an
equivalent. This file documents the mapping and the keys that had to be
dropped, since standard JSON has no comments.

## Mapping

| Claude `.claude/settings.json` | Gemini `.gemini/settings.json` |
|---|---|
| `permissions.allow: ["Bash(<cmd>*)"]` | `tools.coreTools: ["ShellTool(<cmd>)"]` |
| `permissions.additionalDirectories: [...]` | `context.includeDirectories: [...]` |
| (implicit: skills are enabled) | `skills.enabled: true` |

`ShellTool(<prefix>)` matches commands beginning with the given prefix, the
same way `Bash(<prefix>*)` does on the Claude side.

## Dropped (no Gemini equivalent)

- `effortLevel: "high"` — Claude-only thinking-effort dial.
- `permissions.allow: ["Edit(../caffeine/**)"]` — Gemini has no path-scoped
  edit allowlist. The same effect is approximated by listing `../caffeine` in
  `context.includeDirectories`, which is already covered.
- `permissions.allow: ["Skill(...)"]` — Gemini has no per-skill auto-allow.
  Skills are toggled globally via `skills.enabled`; Gemini will prompt on
  first invocation per session. For frictionless launches, run
  `gemini --allowed-tools "..."` instead.
- `env.CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD` — Claude-specific env
  var that opts the included directories into CLAUDE.md discovery. Gemini
  reads `GEMINI.md` / `AGENTS.md` from `includeDirectories` automatically.

## Claude-only allow entries (auto-tracked)

Concrete `permissions.allow` entries from `.claude/settings.json` that have
no Gemini equivalent. Maintained by `.agents/skills/allow-tool` — do not edit
between the markers by hand.

<!-- allow-tool:claude-only-begin -->
- `Edit(../caffeine/**)`
- `Skill(caffeine-sweep)`
- `Skill(caffeine-sim)`
- `Skill(sim)`
- `Skill(allow-tool)`
<!-- allow-tool:claude-only-end -->

## Skills location

Skills live canonically under `.agents/skills/`, which Gemini CLI discovers
natively (it is a built-in alias of `.gemini/skills/` and takes precedence
within the same tier). Claude Code does not yet support `.agents/skills/`, so
`.claude/skills` is a symlink to `../.agents/skills`. Run `.agents/setup.sh`
after a fresh clone to (re)create the symlink. The script works on Linux,
macOS, and Windows Git Bash (falling back to a directory junction when
symlinks aren't available).

## Cross-platform paths

All paths in this repo's agent config are relative. The Caffeine simulator is
expected as a sibling directory at `../caffeine`. There are no longer any
hardcoded absolute paths.
