"""Base adapter contract that decouples the framework from any particular LLM backend."""

from __future__ import annotations

import asyncio
import logging
import time
from contextlib import contextmanager
from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import StrEnum
from pathlib import Path
from typing import ClassVar, Iterator, TextIO

from evolve._io import open_utf8_append, open_utf8_write, read_utf8
from evolve.llm.token_usage import (
    Usage,
    accumulate_usage,
    format_usage,
    raise_if_zero_token,
)
from evolve.paths import SourcesLayout

_SOURCES = SourcesLayout(root=Path(__file__).resolve().parents[3])

log = logging.getLogger(__name__)


class LLMAdapter:
    """Backend-agnostic interface for invoking an LLM agent."""

    # Subclasses set this to their `Model` StrEnum — the set of models the
    # backend can drive. The config boundary reads it to validate `framework.model`
    # against the chosen adapter (via `validate_model`). `None` on test doubles
    # that ignore the model and on open-namespace adapters that override
    # `validate_model` instead.
    MODELS: ClassVar[type[StrEnum] | None] = None

    def __init__(self, model: str | None = None) -> None:
        if model is not None:
            self.validate_model(model)
        self.model = model

    @classmethod
    def validate_model(cls, model: str) -> None:
        """Raise `ValueError` if this backend cannot drive `model`.

        Default: membership in `MODELS` (no check when it is `None`). Adapters
        with an open model namespace (claude-sdk) override this instead."""
        if cls.MODELS is None:
            return
        valid = {member.value for member in cls.MODELS}
        if model not in valid:
            raise ValueError(
                f"{cls.__name__} does not support model {model!r}; "
                f"valid models: {sorted(valid)}"
            )

    def identity(self) -> str:
        """Short string identifying adapter + model."""
        if self.model is None:
            return type(self).__name__
        return f"{type(self).__name__}(model={self.model})"

    def run(
        self,
        agent_name: str,
        workspace: Path,
        *,
        readable_roots: list[Path],
        allow_edits: bool,
    ) -> None:
        """Invoke `agent_name` with prompts sourced from `src/evolve/agents/<agent_name>.md` and `<workspace>/task.md`.

        `workspace` is the only directory the agent may write into when
        `allow_edits` is True. `readable_roots` lists additional read-only
        scope (dirs or single files). Writes a markdown log to
        `<workspace>/<agent_name>.log.md`.
        """
        system_prompt, user_prompt, resolved_roots = self._prepare_invocation(
            agent_name, workspace, readable_roots, allow_edits
        )

        log_path = workspace / f"{agent_name}.log.md"
        started_at = datetime.now(timezone.utc)
        started_monotonic = time.monotonic()
        outcome = _RunOutcome()
        try:
            with open_utf8_write(log_path) as log_io:
                _write_header(log_io, agent_name, workspace, allow_edits, started_at, self.identity())
                outcome.header_written = True
                with outcome.capture_status():
                    outcome.usage = asyncio.run(
                        self.call_llm_and_log(
                            system_prompt,
                            user_prompt,
                            workspace=workspace,
                            readable_roots=resolved_roots,
                            allow_edits=allow_edits,
                            log_io=log_io,
                        )
                    )
            # log_io is closed; safe to open log_path for reading on Windows.
            if outcome.ok:
                with outcome.capture_status():
                    raise_if_zero_token(agent_name, outcome.usage, log_path)
        finally:
            if outcome.header_written:
                with open_utf8_append(log_path) as log_io:
                    _write_footer(log_io, outcome.status, time.monotonic() - started_monotonic, outcome.usage)
            if outcome.ok:
                accumulate_usage(outcome.usage)
            log.info(
                "agent %s finished status=%s usage=%s log=%s",
                agent_name,
                outcome.status,
                format_usage(outcome.usage),
                log_path,
            )
            _log_produced_artifacts(agent_name, workspace, log_path)

    def _prepare_invocation(
        self,
        agent_name: str,
        workspace: Path,
        readable_roots: list[Path],
        allow_edits: bool,
    ) -> tuple[str, str, list[Path]]:
        """Validate inputs, load prompts, and log dispatch metadata."""
        agent_file = _SOURCES.agent_prompt(agent_name)
        if not agent_file.is_file():
            raise FileNotFoundError(f"agent prompt not found: {agent_file}")
        task_file = workspace / "task.md"
        if not task_file.is_file():
            raise FileNotFoundError(f"task.md not found: {task_file}")

        resolved_roots = _validate_readable_roots(readable_roots)
        system_prompt = read_utf8(agent_file)
        user_prompt = read_utf8(task_file)
        log.info(
            "dispatch agent=%s adapter=%s workspace=%s allow_edits=%s",
            agent_name,
            self.identity(),
            workspace,
            allow_edits,
        )
        log.debug("dispatch agent=%s readable_roots=%s", agent_name, resolved_roots)
        return system_prompt, user_prompt, resolved_roots

    async def call_llm_and_log(
        self,
        system_prompt: str,
        user_prompt: str,
        *,
        workspace: Path,
        readable_roots: list[Path],
        allow_edits: bool,
        log_io: TextIO,
    ) -> Usage | None:
        """Subclass extension point: drive one agent invocation against a specific backend.

        Adapters must enforce that writes stay inside `workspace` and reads
        stay inside `workspace` or `readable_roots`. Stream all
        backend-side output (chat, thoughts, tool calls, diagnostics) into
        `log_io`; do not close it.

        Returns a usage dict appended to the log footer.
        """
        raise NotImplementedError


@dataclass
class _RunOutcome:
    """Mutable status + usage tracker shared between the run body and its finally block."""

    status: str = "ok"
    usage: Usage | None = None
    header_written: bool = field(default=False)

    @property
    def ok(self) -> bool:
        return self.status == "ok"

    @contextmanager
    def capture_status(self) -> Iterator[None]:
        """Record any raised exception into `status`, then re-raise."""
        try:
            yield
        except BaseException as exception:
            self.status = f"error: {type(exception).__name__}: {exception}"
            raise


def _log_produced_artifacts(agent_name: str, workspace: Path, log_path: Path) -> None:
    """Log the files an adapter actually produced in `workspace`.

    Excludes `task.md` (the input prompt) and `log_path` (this very log file)
    so the count reflects real agent outputs. Distinguishes "adapter completed"
    from "produced the expected artifacts" so a later missing-output error
    has a breadcrumb.
    """

    excluded = {"task.md", log_path.name}
    try:
        entries = sorted(
            path.name for path in workspace.iterdir() if path.is_file() and path.name not in excluded
        )
    except OSError as exception:
        log.warning("agent %s: could not list %s: %s", agent_name, workspace, exception)
        return
    log.info("agent %s produced %d file(s) in %s: %s", agent_name, len(entries), workspace, entries)


def _validate_readable_roots(roots: list[Path]) -> list[Path]:
    if not roots:
        raise ValueError("readable_roots must be non-empty")
    resolved = []
    for root in roots:
        if not root.exists():
            raise FileNotFoundError(f"readable_root does not exist: {root}")
        resolved.append(root.resolve())
    return resolved


def _write_header(
    log_io: TextIO,
    agent_name: str,
    workspace: Path,
    allow_edits: bool,
    started: datetime,
    adapter_class: str,
) -> None:
    log_io.write(f"# Agent log: {agent_name}\n\n")
    log_io.write(f"- adapter: {adapter_class}\n")
    log_io.write(f"- workspace: {workspace}\n")
    log_io.write(f"- allow_edits: {allow_edits}\n")
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
    """Render `(timestamp, kind, body)` events into `log_io` under a `## timeline` section."""
    log_io.write("## timeline\n\n")
    if not events:
        log_io.write("_(no events)_\n")
        return
    for timestamp, kind, body in events:
        body = body.rstrip("\n")
        log_io.write(f"### {timestamp} {kind}\n\n{body}\n\n")
