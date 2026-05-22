"""Claude Agent SDK adapter — uses `claude_agent_sdk.ClaudeSDKClient` in-process.

A CLI-based counterpart (`ClaudeCliAdapter`) may be added later.
"""

from __future__ import annotations

from datetime import datetime, timezone
from enum import StrEnum
from pathlib import Path
from typing import Any, TextIO

from .base_llm_adapter import LLMAdapter


class Model(StrEnum):
    OPUS_4_7 = "claude-opus-4-7"
    SONNET_4_6 = "claude-sonnet-4-6"
    HAIKU_4_5 = "claude-haiku-4-5-20251001"


MODEL = Model.HAIKU_4_5

READ_ONLY_TOOLS = ["Read", "Glob", "Grep"]
WRITE_TOOLS = ["Write", "Edit"]

# Tools whose input carries a filesystem path that must stay inside `workspace`.
# Map: tool name -> input key holding the path.
PATH_INPUT_KEYS = {
    "Read": "file_path",
    "Write": "file_path",
    "Edit": "file_path",
    "Glob": "path",
    "Grep": "path",
}


class ClaudeSdkAdapter(LLMAdapter):
    async def call_llm_and_log(
        self,
        system_prompt: str,
        user_prompt: str,
        *,
        workspace: Path,
        read_only: bool,
        log_io: TextIO,
    ) -> None:
        from claude_agent_sdk import (
            ClaudeAgentOptions,
            ClaudeSDKClient,
            PermissionResultAllow,
            PermissionResultDeny,
            ToolPermissionContext,
        )
        from claude_agent_sdk.types import (
            AssistantMessage,
            TextBlock,
            ThinkingBlock,
            ToolUseBlock,
        )

        allowed_set = set(READ_ONLY_TOOLS if read_only else READ_ONLY_TOOLS + WRITE_TOOLS)
        workspace_root = workspace.resolve()

        def _in_workspace(raw: str) -> bool:
            p = Path(raw)
            if not p.is_absolute():
                p = workspace_root / p
            try:
                p.resolve().relative_to(workspace_root)
            except ValueError:
                return False
            return True

        async def gate(
            tool_name: str,
            input_data: dict[str, Any],
            context: ToolPermissionContext,
        ) -> PermissionResultAllow | PermissionResultDeny:
            if tool_name not in allowed_set:
                return PermissionResultDeny(
                    behavior="deny",
                    message=(
                        f"tool {tool_name!r} is not available "
                        f"(read_only={read_only}). "
                        f"allowed tools: {sorted(allowed_set)}"
                    ),
                )
            path_key = PATH_INPUT_KEYS.get(tool_name)
            if path_key is not None:
                raw = input_data.get(path_key)
                if raw is not None and not _in_workspace(str(raw)):
                    return PermissionResultDeny(
                        behavior="deny",
                        message=(
                            f"path {raw!r} is outside the workspace. "
                            f"all file operations must stay under {workspace_root}."
                        ),
                    )
            return PermissionResultAllow(behavior="allow")

        # `allowed_tools` pre-approves and bypasses the callback, so leave it
        # empty: every call must go through `gate` for both tool- and
        # path-level checks.
        options = ClaudeAgentOptions(
            model=MODEL.value,
            system_prompt=system_prompt,
            cwd=str(workspace),
            can_use_tool=gate,
        )

        text_chunks: list[str] = []
        tool_events: list[str] = []
        thought_chunks: list[str] = []

        async with ClaudeSDKClient(options=options) as client:
            await client.query(user_prompt)
            async for message in client.receive_response():
                if not isinstance(message, AssistantMessage):
                    continue
                for block in message.content:
                    if isinstance(block, TextBlock):
                        text_chunks.append(block.text)
                    elif isinstance(block, ThinkingBlock):
                        thought_chunks.append(block.thinking)
                    elif isinstance(block, ToolUseBlock):
                        ts = datetime.now(timezone.utc).isoformat(timespec="seconds")
                        tool_events.append(f"- {ts} `{block.name}` args={block.input!r}")

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
