# Agent prompts

One `<agent_name>.md` per agent that the evolve framework invokes via the
LLMAdapter. The body of the file is used verbatim as the **system prompt** —
no frontmatter, no auto-discovery. The framework follows the open-source
agent-skills convention (markdown body = role + contract + constraints).

Add a new prompt here when you add a new framework call site that needs LLM
judgment.
