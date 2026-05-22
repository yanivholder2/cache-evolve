"""LLMAdapter base class

The base reads two prompt sources:

- `evolve/src/agents/<agent_name>.md` — system prompt (the agent's role,
  contract, and constraints).
- `<workspace>/task.md` — user prompt (this invocation's assignment;
  written by the framework before calling `run`).

Subclasses implement only `call_llm_and_log(...)` — an async coroutine that talks
to whatever backend they need (SDK, HTTP, subprocess). The base drives the
event loop via `asyncio.run`, so subclasses never call it themselves.

The adapter exposes two filesystem scopes to the agent:

- `workspace` — the single directory the agent is allowed to *write* into
  when `read_only=False`. Also where `task.md` lives.
- `readable_roots` — additional directories or files the agent may *read*
  but never write. Callers list whatever surfaces the agent needs (e.g.
  outputs_root, dash_src, evolve/subject, a specific past variant dir).
  Adapters that can enforce containment must do so; adapters that can't
  should at least steer the agent via prompt or accept the looser scope.

The base also owns the per-invocation log file at
`<workspace>/<agent_name>.log.md`. It writes a header before the call
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
from typing import Any, TextIO

Usage = dict[str, Any]

AGENTS_DIR = Path(__file__).resolve().parent.parent / "agents"

log = logging.getLogger("llm-adapter")


class LLMAdapter:
    """Decouples the framework from any particular LLM backend.

    Lets the framework call out to an LLM without knowing which one, and
    keeps backend-specific concerns (auth, flags, permissions) out of the
    call sites.
    """

    def run(
        self,
        agent_name: str,
        workspace: Path,
        *,
        readable_roots: list[Path],
        read_only: bool,
    ) -> None:
        """Single entry point the framework uses to invoke an agent.

        Centralises prompt sourcing, scope validation, and log-file
        lifecycle so call sites and adapters both stay simpler. `read_only`
        expresses the caller's intent about whether the agent is allowed
        to mutate the workspace.
        """
        agent_file = AGENTS_DIR / f"{agent_name}.md"
        if not agent_file.is_file():
            raise FileNotFoundError(f"agent prompt not found: {agent_file}")
        task_file = workspace / "task.md"
        if not task_file.is_file():
            raise FileNotFoundError(f"task.md not found: {task_file}")

        resolved_roots = _validate_readable_roots(readable_roots)

        system_prompt = agent_file.read_text(encoding="utf-8")
        user_prompt = task_file.read_text(encoding="utf-8")
        log.debug(
            "dispatch %s into %s (adapter=%s, read_only=%s, readable_roots=%s)",
            agent_name,
            type(self).__name__,
            workspace,
            read_only,
            resolved_roots,
        )

        log_path = workspace / f"{agent_name}.log.md"
        started_wall = datetime.now(timezone.utc)
        started_mono = time.monotonic()
        status = "ok"
        usage: Usage | None = None
        try:
            with log_path.open("w", encoding="utf-8") as log_io:
                _write_header(log_io, agent_name, workspace, read_only, started_wall, type(self).__name__)
                try:
                    usage = asyncio.run(
                        self.call_llm_and_log(
                            system_prompt,
                            user_prompt,
                            workspace=workspace,
                            readable_roots=resolved_roots,
                            read_only=read_only,
                            log_io=log_io,
                        )
                    )
                except BaseException as exc:
                    status = f"error: {type(exc).__name__}: {exc}"
                    raise
                finally:
                    _write_footer(log_io, status, time.monotonic() - started_mono, usage)
        finally:
            log.debug("agent log written into %s", log_path)

    async def call_llm_and_log(
        self,
        system_prompt: str,
        user_prompt: str,
        *,
        workspace: Path,
        readable_roots: list[Path],
        read_only: bool,
        log_io: TextIO,
    ) -> Usage | None:
        """Subclass extension point for talking to a specific backend.

        Implemented as an async coroutine; the base wraps it in
        `asyncio.run`, so subclasses don't manage the event loop.

        `workspace` is the only directory the backend should let the agent
        write to (and only when `read_only=False`). Adapters that can
        enforce this must do so.

        `readable_roots` is the additional read-only scope: dirs or single
        files the agent may browse but never write. The base has already
        validated existence and resolved relative paths; adapters should
        translate this list into whatever containment check their backend
        supports.

        `log_io` is an open, writable text file dedicated to this
        invocation. Adapters should write everything they would otherwise
        send to stdout (chat tokens, tool calls, subprocess output,
        diagnostics) into this handle. Do not close it — the base owns
        its lifecycle.

        Receives the caller's `read_only` intent and is responsible for
        translating it into whatever permission model the backend exposes.

        Returns an optional token-usage dict the base appends to the
        footer. Recommended keys (all optional, omit what the backend
        doesn't expose): `input_tokens`, `output_tokens`,
        `cache_read_tokens`, `cache_write_tokens`, `thinking_tokens`,
        `total_tokens`, `cost_usd`.
        """
        raise NotImplementedError


def _validate_readable_roots(roots: list[Path]) -> list[Path]:
    if not roots:
        raise ValueError("readable_roots must be non-empty")
    resolved = []
    for r in roots:
        if not r.exists():
            raise FileNotFoundError(f"readable_root does not exist: {r}")
        resolved.append(r.resolve())
    return resolved


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


def _write_footer(log_io: TextIO, status: str, duration_s: float, usage: Usage | None) -> None:
    log_io.write("\n---\n\n")
    log_io.write(f"- finished: {datetime.now(timezone.utc).isoformat(timespec='seconds')}\n")
    log_io.write(f"- duration: {duration_s:.2f}s\n")
    log_io.write(f"- status: {status}\n")
    if usage:
        for key, value in usage.items():
            log_io.write(f"- {key}: {value}\n")
    log_io.flush()


def write_timeline(log_io: TextIO, events: list[tuple[str, str, str]]) -> None:
    """Render `(ts, kind, body)` events into the adapter log under `## timeline`.

    Adapters collect events from their backend's streaming primitives and
    hand the normalised list off here so the on-disk format stays uniform.
    Trailing newlines on bodies are stripped to keep entries flush.
    """
    log_io.write("## timeline\n\n")
    if not events:
        log_io.write("_(no events)_\n")
        return
    for ts, kind, body in events:
        body = body.rstrip("\n")
        log_io.write(f"### {ts} {kind}\n\n{body}\n\n")
