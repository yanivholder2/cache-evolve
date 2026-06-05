"""Antigravity SDK adapter — uses `google.antigravity.Agent` in-process."""

from __future__ import annotations

import asyncio
import json
import logging
import time
from datetime import datetime, timezone
from enum import StrEnum
from pathlib import Path
from typing import Any, TextIO

from evolve.llm.base_adapter import LLMAdapter, write_timeline
from evolve.llm.token_usage import Usage

log = logging.getLogger(__name__)


def _format_args(args: Any) -> str:
    try:
        return "```json\n" + json.dumps(args, indent=2, ensure_ascii=False) + "\n```"
    except (TypeError, ValueError) as exception:
        log.warning("tool args not json serialisable, falling back to repr: %s", exception)
        return f"`{args!r}`"


def _stamp() -> tuple[float, str]:
    return time.monotonic(), datetime.now(timezone.utc).strftime("%H:%M:%S")


def _usage_from_metadata(um: Any) -> Usage | None:
    """Normalise an antigravity UsageMetadata into the base adapter's keys."""
    if um is None:
        return None
    key_map = {
        "prompt_token_count": "input_tokens",
        "candidates_token_count": "output_tokens",
        "cached_content_token_count": "cache_read_tokens",
        "thoughts_token_count": "thinking_tokens",
        "total_token_count": "total_tokens",
    }
    usage: Usage = {}
    for src, dst in key_map.items():
        value = getattr(um, src, None)
        if value is not None:
            usage[dst] = value
    return usage or None


class Model(StrEnum):
    GEMINI_3_PRO = "gemini-3-pro-preview"
    GEMINI_3_FLASH = "gemini-3-flash-preview"
    GEMINI_3_1_PRO = "gemini-3.1-pro-preview"
    GEMINI_3_1_FLASH_LITE = "gemini-3.1-flash-lite"
    GEMINI_3_5_FLASH = "gemini-3.5-flash"
    ANTIGRAVITY = "antigravity-preview-05-2026"


class AntigravitySdkAdapter(LLMAdapter):
    MODELS = Model

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
        from google.antigravity import CapabilitiesConfig, LocalAgentConfig
        from google.antigravity.agent import Agent
        from google.antigravity.hooks import policy
        from google.antigravity.types import BuiltinTools

        enabled = BuiltinTools.read_only()
        if allow_edits:
            enabled = enabled + [BuiltinTools.CREATE_FILE, BuiltinTools.EDIT_FILE]

        readable_resolved = [r.resolve() for r in readable_roots]

        def _inside_readable(tc) -> bool:
            raw = tc.canonical_path or ""
            if not raw:
                return False
            abs_path = Path(raw).resolve()
            return any(abs_path.is_relative_to(r) for r in readable_resolved)

        policies = [
            policy.deny("create_file", when=_inside_readable, name="readable_root_read_only"),
            policy.deny("edit_file", when=_inside_readable, name="readable_root_read_only"),
        ]

        config = LocalAgentConfig(
            model=self.model,
            system_instructions=system_prompt,
            capabilities=CapabilitiesConfig(enabled_tools=enabled),
            workspaces=[str(workspace), *(str(r) for r in readable_resolved)],
            policies=policies,
        )

        # (monotonic, ts_hms, kind, body) — monotonic orders events emitted by
        # the three concurrent collectors below; ts_hms is for display only.
        # The three iterators are independent cursors over a shared buffer
        # (see ChatResponse docstring); concurrent gather is the SDK-blessed
        # usage and is serialised internally by a pull lock.
        events: list[tuple[float, str, str, str]] = []

        async def collect_text(response) -> None:
            async for token in response:
                mono, ts = _stamp()
                events.append((mono, ts, "chat", token))

        async def collect_tools(response) -> None:
            async for call in response.tool_calls:
                mono, ts = _stamp()
                events.append((mono, ts, f"tool: {call.name}", _format_args(call.args)))

        async def collect_thoughts(response) -> None:
            async for thought in response.thoughts:
                mono, ts = _stamp()
                events.append((mono, ts, "thinking", thought))

        async with Agent(config) as agent:
            response = await agent.chat(user_prompt)
            await asyncio.gather(
                collect_text(response),
                collect_tools(response),
                collect_thoughts(response),
            )
            usage = _usage_from_metadata(response.usage_metadata)

        # The text/thoughts streams yield tokens, not blocks. Coalesce
        # consecutive same-kind events so the timeline shows one entry per
        # logical block (a tool call between two chat runs splits them).
        events.sort(key=lambda e: e[0])
        merged: list[tuple[str, str, str]] = []
        for _, ts, kind, body in events:
            if merged and merged[-1][1] == kind and kind in ("chat", "thinking"):
                prev_ts, prev_kind, prev_body = merged[-1]
                merged[-1] = (prev_ts, prev_kind, prev_body + body)
            else:
                merged.append((ts, kind, body))

        write_timeline(log_io, merged)
        return usage
