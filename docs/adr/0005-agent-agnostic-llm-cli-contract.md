# Agent-agnostic LLM CLI contract via `EVOLVE_LLM_CMD`

The Evolve framework needs to invoke an LLM from non-interactive scripts
(hypothesis-writer for #91, variant-implementer for #90, smoke-reviewer and
insight-writer later). Each LLM CLI (Claude Code, Gemini CLI, etc.) has its own
flags, headless-mode conventions, and banner behaviour. We need one contract
that all framework scripts use, without baking a specific CLI in.

**Decision: one env var, `EVOLVE_LLM_CMD`, holds the full command + flags. The
framework script appends the prompt as the final argv and captures stdout.**

Default value: `"gemini -p"`. Override by exporting `EVOLVE_LLM_CMD` in the
environment (e.g. `EVOLVE_LLM_CMD="claude -p --model opus"`). The prompt is
assembled by the framework script (token-frugal: assembly is mechanical, so it
lives in `evolve/scripts/`) and passed as the final positional argument:

```sh
$EVOLVE_LLM_CMD "$prompt" > response.txt
```

Tests stub the LLM by pointing `EVOLVE_LLM_CMD` at a deterministic script that
emits canned output.

## Considered options

- **Per-CLI logic inside framework scripts.** A `case` on the CLI name selects
  the right flags. Couples framework scripts to specific runtimes; new CLIs
  require code changes. Rejected.
- **User-written adapter scripts (stdin → stdout contract).** Each user writes
  a ~3-line shell adapter per CLI; framework defines a stdin/stdout protocol.
  Cleanest in principle but adds friction for a single-user research tool.
  Rejected for v1; reachable later if more CLIs need integration.
- **Separate env vars for determinism knobs** (`EVOLVE_LLM_SEED`,
  `EVOLVE_LLM_TEMPERATURE`). Framework would translate them into CLI-specific
  flags. Re-introduces per-CLI logic. Rejected — flags bake into
  `EVOLVE_LLM_CMD` itself (`gemini -p --temperature 0`).
- **Single env var with prompt as final argv.** Chosen. Trivial to override,
  trivial to stub, no framework code knows about any CLI.

## Consequences

- Adding a new LLM CLI is purely a configuration change — set
  `EVOLVE_LLM_CMD` to whatever invocation works.
- Tests are deterministic by pointing `EVOLVE_LLM_CMD` at a stub script. No
  network, no real LLM, no flakiness in CI.
- Determinism is the user's responsibility: bake `--temperature 0` or a seed
  flag into `EVOLVE_LLM_CMD` as needed. The framework does not enforce it.
- The contract assumes all supported CLIs accept the prompt as their final
  positional argument in headless mode. True for Claude Code (`claude -p`) and
  Gemini CLI (`gemini -p`); if a future CLI doesn't fit, the user wraps it in
  an adapter and points `EVOLVE_LLM_CMD` at the adapter.
- Framework scripts that need LLM judgment must use `EVOLVE_LLM_CMD` and only
  `EVOLVE_LLM_CMD`. No script reaches for `claude` or `gemini` directly.
