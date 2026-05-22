"""Antigravity SDK adapter — uses `google.antigravity.Agent` in-process.

A CLI-based counterpart (`AntigravityCliAdapter`) may be added later.
"""

from __future__ import annotations

import asyncio
import json
import os
import time
from datetime import datetime, timezone
from enum import StrEnum
from pathlib import Path
from typing import Any, TextIO

from .base_llm_adapter import LLMAdapter, Usage


def _format_args(args: Any) -> str:
    try:
        return "```json\n" + json.dumps(args, indent=2, ensure_ascii=False) + "\n```"
    except (TypeError, ValueError):
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


MODEL = Model.GEMINI_3_1_FLASH_LITE


class AntigravitySdkAdapter(LLMAdapter):
    async def call_llm_and_log(
        self,
        system_prompt: str,
        user_prompt: str,
        *,
        workspace: Path,
        repo_src: Path,
        read_only: bool,
        log_io: TextIO,
    ) -> Usage | None:
        from google.antigravity import CapabilitiesConfig, LocalAgentConfig
        from google.antigravity.agent import Agent
        from google.antigravity.hooks import policy
        from google.antigravity.types import BuiltinTools

        enabled = BuiltinTools.read_only()
        if not read_only:
            enabled = enabled + [BuiltinTools.CREATE_FILE, BuiltinTools.EDIT_FILE]

        repo_src_str = str(repo_src.resolve())

        def _inside_repo_src(tc) -> bool:
            path = tc.canonical_path or ""
            if not path:
                return False
            abs_path = os.path.abspath(path)
            return abs_path == repo_src_str or abs_path.startswith(repo_src_str + os.sep)

        policies = [
            policy.deny("create_file", when=_inside_repo_src, name="repo_src_read_only"),
            policy.deny("edit_file", when=_inside_repo_src, name="repo_src_read_only"),
        ]

        config = LocalAgentConfig(
            model=MODEL.value,
            system_instructions=system_prompt,
            capabilities=CapabilitiesConfig(enabled_tools=enabled),
            workspaces=[str(workspace), repo_src_str],
            policies=policies,
        )

        # (monotonic, ts_hms, kind, body) — monotonic orders events emitted by
        # the three concurrent collectors below; ts_hms is for display only.
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

        log_io.write("## timeline\n\n")
        if not merged:
            log_io.write("_(no events)_\n")
        else:
            for ts, kind, body in merged:
                body = body.rstrip("\n") if kind in ("chat", "thinking") else body
                log_io.write(f"### {ts} {kind}\n\n{body}\n\n")

        return usage
