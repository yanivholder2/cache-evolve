"""LLMAdapter base class

The base reads two prompt sources:

- `evolve/src/agents/<agent_name>.md` — system prompt (the agent's role,
  contract, and constraints).
- `<output_dir>/task.md` — user prompt (this invocation's assignment;
  written by the framework before calling `run`).

Subclasses implement only `call_llm_and_log(...)` — an async coroutine that talks
to whatever backend they need (SDK, HTTP, subprocess). The base drives the
event loop via `asyncio.run`, so subclasses never call it themselves.
Adapters never see file paths beyond `workspace`.

The base also owns the per-invocation log file at
`<output_dir>/<agent_name>.log.md`. It writes a header before the call
and a footer after, and hands the open file to the adapter as `log_io`
so the adapter can stream its own content (chat tokens, thoughts, tool
calls, subprocess stdout, etc.) into the same file. Nothing is written
to the parent process's stdout/stderr from this layer.
"""

from __future__ import annotations

import asyncio
import logging
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import TextIO

AGENTS_DIR = Path(__file__).resolve().parent.parent / "agents"
REPO_SRC = Path(__file__).resolve().parents[3] / "src"

log = logging.getLogger("llm-adapter")


class LLMAdapter:
    """Decouples the framework from any particular LLM backend.

    Lets the framework call out to an LLM without knowing which one, and
    keeps backend-specific concerns (auth, flags, permissions) out of the
    call sites.
    """

    def run(self, agent_name: str, output_dir: Path, *, read_only: bool) -> None:
        """Single entry point the framework uses to invoke an agent.

        Centralises prompt sourcing and log-file lifecycle so call sites
        and adapters both stay simpler. `read_only` expresses the caller's
        intent about whether the agent is allowed to mutate the workspace.
        """
        agent_file = AGENTS_DIR / f"{agent_name}.md"
        if not agent_file.is_file():
            raise FileNotFoundError(f"agent prompt not found: {agent_file}")
        task_file = output_dir / "task.md"
        if not task_file.is_file():
            raise FileNotFoundError(f"task.md not found: {task_file}")

        system_prompt = agent_file.read_text(encoding="utf-8")
        user_prompt = task_file.read_text(encoding="utf-8")
        log.debug(
            "dispatch %s -> %s (workspace=%s, read_only=%s)",
            agent_name,
            type(self).__name__,
            output_dir,
            read_only,
        )

        log_path = output_dir / f"{agent_name}.log.md"
        started_wall = datetime.now(timezone.utc)
        started_mono = time.monotonic()
        status = "ok"
        try:
            with log_path.open("w", encoding="utf-8") as log_io:
                _write_header(log_io, agent_name, output_dir, read_only, started_wall, type(self).__name__)
                try:
                    asyncio.run(
                        self.call_llm_and_log(
                            system_prompt,
                            user_prompt,
                            workspace=output_dir,
                            repo_src=REPO_SRC,
                            read_only=read_only,
                            log_io=log_io,
                        )
                    )
                except BaseException as exc:
                    status = f"error: {type(exc).__name__}: {exc}"
                    raise
                finally:
                    _write_footer(log_io, status, time.monotonic() - started_mono)
        finally:
            log.debug("agent log written into %s", log_path)

    async def call_llm_and_log(
        self,
        system_prompt: str,
        user_prompt: str,
        *,
        workspace: Path,
        repo_src: Path,
        read_only: bool,
        log_io: TextIO,
    ) -> None:
        """Subclass extension point for talking to a specific backend.

        Implemented as an async coroutine; the base wraps it in
        `asyncio.run`, so subclasses don't manage the event loop.

        `workspace` is the only directory the backend should let the agent
        write to. Adapters that can enforce this must do so. Adapters
        that can't should at least steer the agent via prompt or accept
        the looser scope.

        `repo_src` is an additional read-only source the agent may browse
        for context (dash repo Rust code). Adapters should expose it for
        reading and deny writes.

        `log_io` is an open, writable text file dedicated to this
        invocation. Adapters should write everything they would otherwise
        send to stdout (chat tokens, tool calls, subprocess output,
        diagnostics) into this handle. Do not close it — the base owns
        its lifecycle.

        Receives the caller's `read_only` intent and is responsible for
        translating it into whatever permission model the backend exposes.
        """
        raise NotImplementedError


def _write_header(
    log_io: TextIO,
    agent_name: str,
    workspace: Path,
    read_only: bool,
    started: datetime,
    adapter_class: str,
) -> None:
    log_io.write(f"# Agent log: {agent_name}\n\n")
    log_io.write(f"- adapter: {adapter_class}\n")
    log_io.write(f"- workspace: {workspace}\n")
    log_io.write(f"- read_only: {read_only}\n")
    log_io.write(f"- started: {started.isoformat(timespec='seconds')}\n\n")
    log_io.flush()


def _write_footer(log_io: TextIO, status: str, duration_s: float) -> None:
    log_io.write("\n---\n\n")
    log_io.write(f"- finished: {datetime.now(timezone.utc).isoformat(timespec='seconds')}\n")
    log_io.write(f"- duration: {duration_s:.2f}s\n")
    log_io.write(f"- status: {status}\n")
    log_io.flush()
