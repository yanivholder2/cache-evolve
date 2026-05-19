# Agent-agnostic LLM CLI contract via `EVOLVE_LLM_CMD`

The Evolve framework needs to invoke an LLM from non-interactive scripts
(hypothesis-writer for #91, variant-implementer for #90, smoke-reviewer and
insight-writer later). Each LLM CLI (Claude Code, Gemini CLI, etc.) has its own
flags, headless-mode conventions, and per-invocation tool-permissioning syntax.
We need one contract that all framework scripts use, without baking a specific
CLI in.

**Decision: one env var, `EVOLVE_LLM_CMD`, points at a per-CLI adapter script.
The adapter takes `<agent-name> <output-dir> [--writable <path>...]`. The
caller writes `<output-dir>/task.md` before invocation; the agent reads it and
writes artifacts to known paths under `<output-dir>` (plus any additional
`--writable` scopes). Success = the expected files exist on disk; fail loudly
otherwise.**

Default value: `evolve/scripts/agents/gemini.sh`. Override by exporting
`EVOLVE_LLM_CMD` (e.g. `EVOLVE_LLM_CMD=evolve/scripts/agents/claude.sh`).

```sh
$EVOLVE_LLM_CMD <agent-name> <output-dir> [--writable <path>...]
```

- `<agent-name>` resolves to `.agents/agents/<name>.md` (project-scoped,
  agent-agnostic — plain markdown, no Claude-Code-specific frontmatter).
- `<output-dir>` doubles as the task-file location and the default writable
  scope. The caller writes `<output-dir>/task.md` before invocation.
- `--writable` is additive: variant-implementer (#90) will pass both its
  `impl/` dir and a scratch dir, for example.

Tests stub the LLM by pointing `EVOLVE_LLM_CMD` at
`evolve/tests/_mocks/llm-stub.sh`, an adapter that `mkdir -p` + writes canned
files instead of invoking a real model.

## Considered options

- **Per-CLI logic inside framework scripts.** A `case` on the CLI name selects
  the right flags. Couples framework scripts to specific runtimes; new CLIs
  require code changes. Rejected.
- **Single env var + prompt as final argv (Round 1).** `$EVOLVE_LLM_CMD
  "$prompt"`, stdout = response. Trivial but breaks the moment a subagent
  needs to write multiple files (variant-implementer, #90), and bakes prompt
  assembly into framework scripts instead of letting the agent read its own
  context. Rejected on the second iteration.
- **Per-CLI adapter scripts (chosen).** Each user writes a thin shell adapter
  per CLI; the framework defines a positional argv + file-output protocol. The
  Round-1 rationale for rejecting this — friction for a single-user research
  tool — is outweighed by the need for per-invocation tool-permissioning,
  which is too divergent across CLIs to express via env-var flags. The
  adapter is the one place that knows about the underlying CLI; framework
  scripts know nothing.
- **Separate env vars for determinism knobs** (`EVOLVE_LLM_SEED`,
  `EVOLVE_LLM_TEMPERATURE`). Framework would translate them into CLI-specific
  flags. Re-introduces per-CLI logic. Rejected — flags bake into the adapter
  script itself.

## Consequences

- Adding a new LLM CLI = drop in another adapter under
  `evolve/scripts/agents/` and swap `EVOLVE_LLM_CMD`.
- Subagents are tool-using and file-out. They `Read` whatever context they
  need (leaderboard, ledger, extension-points) via the underlying CLI's file
  tools and `Write` artifacts to `<output-dir>`. Framework scripts do not
  pre-assemble prompts.
- Tests are deterministic by pointing `EVOLVE_LLM_CMD` at a stub adapter that
  writes canned files. No network, no real LLM, no flakiness in CI.
- Tool-permissioning (write-scope enforcement) is the adapter's job. v1 ships
  with prompt-level constraints only (each agent `.md` says "only write under
  `$OUTPUT_DIR`"); a future hardening pass will plumb CLI-specific allowlists
  through the adapter (Gemini policy / Claude `--allowedTools`).
- Determinism is the user's responsibility: bake `--temperature 0` or a seed
  flag into the adapter as needed. The framework does not enforce it.
- Framework scripts that need LLM judgment must use `EVOLVE_LLM_CMD` and only
  `EVOLVE_LLM_CMD`. No script reaches for `claude` or `gemini` directly.
- Token-frugality (ADR-0002) is preserved by the script/agent split: scripts
  do deterministic prep (scaffold dirs, write `task.md`, count files);
  agents do only judgment work.
