# Agent prompts

One `<agent_name>.md` per agent that the evolve framework invokes via the
LLMAdapter. The body of the file is used verbatim as the **system prompt** —
no frontmatter, no auto-discovery. The framework follows the open-source
agent-skills convention (markdown body = role + contract + constraints).

`evolve/scripts/adapters/base_llm_adapter.py` reads `<agent_name>.md` from this
directory and `<output_dir>/task.md` (written by the caller) and hands both
strings to the adapter subclass via `call_llm_and_log(system_prompt, user_prompt)`.

Today `hypotheses-writer.md` and `variant-implementer.md` are wired. Add a
new prompt here when you add a new framework call site that needs LLM
judgment (e.g. `smoke-reviewer`, `insight-writer`).
