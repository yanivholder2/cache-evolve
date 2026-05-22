"""LLMAdapter base class

The base reads two prompt sources:

- `evolve/scripts/agents/<agent_name>.md` — system prompt (the agent's role,
  contract, and constraints).
- `<output_dir>/task.md` — user prompt (this invocation's assignment;
  written by the framework before calling `run`).

Subclasses implement only `_call_llm(system_prompt, user_prompt)` — whatever mechanism
they need (subprocess, SDK, HTTP). Adapters never see file paths.
"""

from __future__ import annotations

import logging
from pathlib import Path

AGENTS_DIR = Path(__file__).resolve().parent.parent / "agents"

log = logging.getLogger("llm-adapter")


class LLMAdapter:
    """Decouples the framework from any particular LLM backend.

    Lets the framework call out to an LLM without knowing which one, and
    keeps backend-specific concerns (auth, flags, permissions) out of the
    call sites.
    """

    def run(self, agent_name: str, output_dir: Path, *, read_only: bool) -> None:
        """Single entry point the framework uses to invoke an agent.

        Centralises prompt sourcing so call sites stay agnostic of where
        prompts live. `read_only` expresses the caller's intent about
        whether the agent is allowed to mutate the workspace.
        """
        agent_file = AGENTS_DIR / f"{agent_name}.md"
        if not agent_file.is_file():
            raise FileNotFoundError(f"agent prompt not found: {agent_file}")
        task_file = output_dir / "task.md"
        if not task_file.is_file():
            raise FileNotFoundError(f"task.md not found: {task_file}")

        system_prompt = agent_file.read_text(encoding="utf-8")
        user_prompt = task_file.read_text(encoding="utf-8")
        log.debug("dispatch %s -> %s (read_only=%s)", agent_name, type(self).__name__, read_only)
        self._call_llm(system_prompt, user_prompt, read_only=read_only)

    def _call_llm(self, system_prompt: str, user_prompt: str, *, read_only: bool) -> None:
        """Subclass extension point for talking to a specific backend.

        Receives the caller's `read_only` intent and is responsible for
        translating it into whatever permission model the backend exposes.
        """
        raise NotImplementedError
