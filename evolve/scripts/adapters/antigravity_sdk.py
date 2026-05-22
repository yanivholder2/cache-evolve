"""Antigravity SDK adapter — uses `google.antigravity.Agent` in-process.

A CLI-based counterpart (`AntigravityCliAdapter`) may be added later.
"""

from __future__ import annotations

import asyncio
import os
from datetime import datetime, timezone
from enum import StrEnum
from pathlib import Path
from typing import TextIO

from .base_llm_adapter import LLMAdapter


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
    ) -> None:
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

        text_chunks: list[str] = []
        tool_events: list[str] = []
        thought_chunks: list[str] = []

        async def collect_text(response) -> None:
            async for token in response:
                text_chunks.append(token)

        async def collect_tools(response) -> None:
            async for call in response.tool_calls:
                ts = datetime.now(timezone.utc).isoformat(timespec="seconds")
                name = getattr(call, "name", "?")
                args = getattr(call, "args", None)
                tool_events.append(f"- {ts} `{name}` args={args!r}")

        async def collect_thoughts(response) -> None:
            async for thought in response.thoughts:
                thought_chunks.append(str(thought))

        async with Agent(config) as agent:
            response = await agent.chat(user_prompt)
            await asyncio.gather(
                collect_text(response),
                collect_tools(response),
                collect_thoughts(response),
            )

        log_io.write("## chat\n\n")
        log_io.write("".join(text_chunks) or "_(no conversational text)_\n")
        if not text_chunks[-1:] == ["\n"]:
            log_io.write("\n")

        log_io.write("\n## tool calls\n\n")
        log_io.write("\n".join(tool_events) if tool_events else "_(none)_")
        log_io.write("\n")

        log_io.write("\n## thoughts\n\n")
        log_io.write("".join(thought_chunks) if thought_chunks else "_(none)_\n")
        if thought_chunks and not thought_chunks[-1].endswith("\n"):
            log_io.write("\n")
