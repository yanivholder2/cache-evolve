"""Base adapter contract that decouples the framework from any particular LLM backend."""

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
    """Backend-agnostic interface for invoking an LLM agent."""

    def run(
        self,
        agent_name: str,
        workspace: Path,
        *,
        readable_roots: list[Path],
        read_only: bool,
    ) -> None:
        """Invoke `agent_name` with prompts sourced from `evolve/src/agents/<agent_name>.md` and `<workspace>/task.md`.

        `workspace` is the only directory the agent may write into when
        `read_only` is False. `readable_roots` lists additional read-only
        scope (dirs or single files). Writes a markdown log to
        `<workspace>/<agent_name>.log.md`.
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
                except BaseException as exception:
                    status = f"error: {type(exception).__name__}: {exception}"
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
        """Subclass extension point: drive one agent invocation against a specific backend.

        Adapters must enforce that writes stay inside `workspace` and reads
        stay inside `workspace` or `readable_roots`. Stream all
        backend-side output (chat, thoughts, tool calls, diagnostics) into
        `log_io`; do not close it.

        Returns a usage dict appended to the log footer. Recommended keys
        (omit what the backend doesn't expose): `input_tokens`,
        `output_tokens`, `cache_read_tokens`, `cache_write_tokens`,
        `thinking_tokens`, `total_tokens`, `cost_usd`.
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
    """Render `(ts, kind, body)` events into `log_io` under a `## timeline` section."""
    log_io.write("## timeline\n\n")
    if not events:
        log_io.write("_(no events)_\n")
        return
    for ts, kind, body in events:
        body = body.rstrip("\n")
        log_io.write(f"### {ts} {kind}\n\n{body}\n\n")
