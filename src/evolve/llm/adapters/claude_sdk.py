"""Claude Agent SDK adapter — uses `claude_agent_sdk.ClaudeSDKClient` in-process."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from enum import StrEnum
from pathlib import Path
from typing import Any, TextIO

from evolve.llm.base_adapter import LLMAdapter, write_timeline
from evolve.llm.token_usage import Usage


class Model(StrEnum):
    OPUS_4_8 = "claude-opus-4-8"
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


def _format_args(args: Any) -> str:
    try:
        return "```json\n" + json.dumps(args, indent=2, ensure_ascii=False) + "\n```"
    except (TypeError, ValueError):
        return f"`{args!r}`"


def _now_hms() -> str:
    return datetime.now(timezone.utc).strftime("%H:%M:%S")


def _usage_from_result(message: Any) -> Usage | None:
    """Normalise a ResultMessage's usage/cost into the base adapter's keys."""
    raw = message.usage or {}
    key_map = {
        "input_tokens": "input_tokens",
        "output_tokens": "output_tokens",
        "cache_read_input_tokens": "cache_read_tokens",
        "cache_creation_input_tokens": "cache_write_tokens",
    }
    usage: Usage = {}
    for src, dst in key_map.items():
        value = raw.get(src)
        if value is not None:
            usage[dst] = value
    if message.total_cost_usd is not None:
        usage["cost_usd"] = round(message.total_cost_usd, 6)
    return usage or None




class ClaudeSdkAdapter(LLMAdapter):
    def identity(self) -> str:
        return f"{type(self).__name__}(model={MODEL.value})"

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
        from claude_agent_sdk import (
            ClaudeAgentOptions,
            ClaudeSDKClient,
            PermissionResultAllow,
            PermissionResultDeny,
            ToolPermissionContext,
        )
        from claude_agent_sdk.types import (
            AssistantMessage,
            ResultMessage,
            TextBlock,
            ThinkingBlock,
            ToolUseBlock,
        )

        allowed_set = set(READ_ONLY_TOOLS + WRITE_TOOLS if allow_edits else READ_ONLY_TOOLS)
        workspace_root = workspace.resolve()
        write_tools = set(WRITE_TOOLS)

        def _resolve(raw: str) -> Path:
            p = Path(raw)
            if not p.is_absolute():
                p = workspace_root / p
            return p.resolve()

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
                        f"(allow_edits={allow_edits}). "
                        f"allowed tools: {sorted(allowed_set)}"
                    ),
                )
            path_key = PATH_INPUT_KEYS.get(tool_name)
            if path_key is not None:
                raw = input_data.get(path_key)
                if raw is not None:
                    abs_path = _resolve(str(raw))
                    in_workspace = abs_path.is_relative_to(workspace_root)
                    in_readable = any(abs_path.is_relative_to(r) for r in readable_roots)
                    is_write = tool_name in write_tools
                    if is_write and not in_workspace:
                        return PermissionResultDeny(
                            behavior="deny",
                            message=(
                                f"path {raw!r} is outside the workspace. "
                                f"writes must stay under {workspace_root}."
                            ),
                        )
                    if not is_write and not (in_workspace or in_readable):
                        return PermissionResultDeny(
                            behavior="deny",
                            message=(
                                f"path {raw!r} is outside the allowed roots. "
                                f"reads must stay under {workspace_root} or one of {readable_roots}."
                            ),
                        )
            return PermissionResultAllow(behavior="allow")

        # `setting_sources=[]` prevents user/project `permissions.allow` rules
        # from pre-approving tools and bypassing `gate`.
        options = ClaudeAgentOptions(
            model=MODEL.value,
            system_prompt=system_prompt,
            cwd=str(workspace_root),
            can_use_tool=gate,
            setting_sources=[],
        )

        events: list[tuple[str, str, str]] = []
        usage: Usage | None = None

        async with ClaudeSDKClient(options=options) as client:
            await client.query(user_prompt)
            async for message in client.receive_response():
                if isinstance(message, AssistantMessage):
                    for block in message.content:
                        if isinstance(block, TextBlock):
                            events.append((_now_hms(), "chat", block.text.rstrip("\n")))
                        elif isinstance(block, ThinkingBlock):
                            events.append((_now_hms(), "thinking", block.thinking.rstrip("\n")))
                        elif isinstance(block, ToolUseBlock):
                            events.append((_now_hms(), f"tool: {block.name}", _format_args(block.input)))
                elif isinstance(message, ResultMessage):
                    usage = _usage_from_result(message)

        write_timeline(log_io, events)
        return usage
