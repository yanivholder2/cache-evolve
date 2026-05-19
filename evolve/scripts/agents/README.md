# LLM CLI Adapters

Per-CLI shell adapters implementing the agent-agnostic contract from
[ADR-0005](../../../docs/adr/0005-agent-agnostic-llm-cli-contract.md):

```
$EVOLVE_LLM_CMD <agent-name> <output-dir> [--writable <path>...]
```

The framework script writes `<output-dir>/task.md` before invocation. The
adapter loads `.agents/agents/<agent-name>.md` as the agent's system prompt,
hands control to the underlying CLI, and the agent reads `task.md` to discover
its assignment. Success = the expected files exist under `<output-dir>`.

Select an adapter via the `EVOLVE_LLM_CMD` env var. Default is `gemini.sh`.

## Files

- `gemini.sh` — default real adapter. Concatenates the agent body + `task.md`
  and pipes the result to `gemini -p`. Write-scope enforcement is currently
  prompt-level only (the agent's `.md` says "only write under `$OUTPUT_DIR`");
  CLI-policy plumbing is tracked as a TODO inside the script.
- New CLI? Drop in `claude.sh`, `cursor.sh`, etc. matching the contract and
  point `EVOLVE_LLM_CMD` at it. Framework scripts know about no specific CLI.

The test stub lives at `../../tests/_mocks/llm-stub.sh`; it dispatches by
`<agent-name>` and writes canned files instead of invoking a real model.
