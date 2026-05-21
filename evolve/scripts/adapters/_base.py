"""LLMAdapter base class

The base reads two prompt sources:

- `evolve/scripts/agents/<agent_name>.md` — system prompt (the agent's role,
  contract, and constraints).
- `<output_dir>/task.md` — user prompt (this invocation's assignment;
  written by the framework before calling `run`).

Subclasses implement only `_call_llm(system, user)` — whatever mechanism
they need (subprocess, SDK, HTTP). Adapters never see file paths.
"""

from __future__ import annotations

import logging
from pathlib import Path

AGENTS_DIR = Path(__file__).resolve().parent.parent / "agents"

log = logging.getLogger("llm-adapter")


class LLMAdapter:
    """Read agent + task prompts, hand off to subclass `_call_llm`.

    `read_only=True` signals to the adapter that the agent should not be
    granted file-write permission. Adapters carry the intent; per-CLI
    enforcement (e.g. `--disallowedTools Write,Edit`) is the subclass's job.
    """

    def run(self, agent_name: str, output_dir: Path, *, read_only: bool = False) -> None:
        agent_file = AGENTS_DIR / f"{agent_name}.md"
        task_file = output_dir / "task.md"
        if not agent_file.is_file():
            raise FileNotFoundError(f"agent prompt not found: {agent_file}")
        if not task_file.is_file():
            raise FileNotFoundError(f"task.md not found: {task_file}")

        system = agent_file.read_text(encoding="utf-8")
        user = task_file.read_text(encoding="utf-8")
        log.debug("dispatch %s -> %s (read_only=%s)", agent_name, type(self).__name__, read_only)
        self._call_llm(system, user, read_only=read_only)

    def _call_llm(self, system: str, user: str, *, read_only: bool) -> None:
        raise NotImplementedError
