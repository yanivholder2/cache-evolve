"""Ollama adapter — drives a local/remote Ollama model through a hand-rolled agent loop.

Unlike the Claude and Antigravity SDK adapters, Ollama exposes only a chat +
tool-calling primitive: it does not execute tools, loop, or sandbox file access.
This adapter therefore implements the whole agent harness itself — the file
tools (`Read`/`Glob`/`Grep`/`Write`/`Edit`), the workspace/readable-root sandbox,
and the call → tool-call → execute → repeat loop — to match the contract the SDK
adapters get for free.

Backend host follows the `OLLAMA_HOST` env var (default `http://localhost:11434`),
so the same code talks to a local Ollama or a remote GPU box with no code change.
Swap the served model by editing `MODEL` below.
"""

from __future__ import annotations

import json
import logging
import re
from datetime import datetime, timezone
from enum import StrEnum
from pathlib import Path
from typing import Any, Callable, TextIO

from evolve.llm.base_adapter import LLMAdapter, write_timeline
from evolve.llm.token_usage import Usage

log = logging.getLogger(__name__)


class Model(StrEnum):
    GEMMA4_E2B = "gemma4:e2b"
    GEMMA4_E4B = "gemma4:e4b"
    GEMMA4_26B = "gemma4:26b"
    GEMMA4_31B = "gemma4:31b"


# The one knob for "which Gemma": edit this constant. The agent loop is
# model-agnostic, so any tool-capable Ollama model id works here too.
MODEL = Model.GEMMA4_E4B

# Ollama defaults the context window to a few thousand tokens regardless of what
# the model supports; below ~16K tool calling degrades badly on long prompts.
NUM_CTX = 16384

# Keep the model resident in (V)RAM between the loop's many sequential chat calls
# and across agents in a cycle, instead of paying the reload cost each turn. The
# 9.6 GB Gemma weights take real time to reload on CPU; a generous window pays off.
KEEP_ALIVE = "1h"

# Gemma 4 advertises a `thinking` capability. Enabling it makes the model reason
# before emitting tool calls (more reliable tool use) and lets us log the reasoning
# in the timeline, matching the SDK adapters. All `Model` entries support it; flip
# to False if you point `MODEL` at a non-thinking model.
THINK = True

# Hard ceiling on agent turns so a model that never stops calling tools (or loops
# on a failing edit) cannot run forever. Generous: real agents finish well under this.
MAX_TURNS = 50

# Bound tool output fed back into the context. A weak local model drowns in a
# 200 KB file dump; these caps keep each tool result digestible.
MAX_READ_BYTES = 64_000
MAX_GREP_MATCHES = 200
MAX_GLOB_MATCHES = 500

READ_ONLY_TOOLS = ["Read", "Glob", "Grep"]
WRITE_TOOLS = ["Write", "Edit"]


def _now_hms() -> str:
    return datetime.now(timezone.utc).strftime("%H:%M:%S")


def _format_args(args: Any) -> str:
    try:
        return "```json\n" + json.dumps(args, indent=2, ensure_ascii=False) + "\n```"
    except (TypeError, ValueError):
        return f"`{args!r}`"


class _SandboxError(Exception):
    """A tool call asked to read or write outside its allowed roots."""


def _tool_schemas(allow_edits: bool) -> list[dict[str, Any]]:
    """OpenAI-style function schemas for the tools, gated by `allow_edits`."""

    def fn(name: str, description: str, properties: dict[str, Any], required: list[str]) -> dict[str, Any]:
        return {
            "type": "function",
            "function": {
                "name": name,
                "description": description,
                "parameters": {"type": "object", "properties": properties, "required": required},
            },
        }

    schemas = [
        fn(
            "Read",
            "Read a UTF-8 text file and return its contents (no line-number prefixes).",
            {"file_path": {"type": "string", "description": "Absolute path, or relative to the workspace."}},
            ["file_path"],
        ),
        fn(
            "Glob",
            "List files matching a glob pattern. Returns newline-separated paths.",
            {
                "pattern": {"type": "string", "description": "Glob pattern, e.g. '**/*.rs'."},
                "path": {"type": "string", "description": "Directory to search in. Defaults to the workspace."},
            },
            ["pattern"],
        ),
        fn(
            "Grep",
            "Search file contents with a regular expression. Returns 'path:line:text' matches.",
            {
                "pattern": {"type": "string", "description": "Python regular expression."},
                "path": {"type": "string", "description": "File or directory to search. Defaults to the workspace."},
            },
            ["pattern"],
        ),
    ]
    if allow_edits:
        schemas += [
            fn(
                "Write",
                "Create or overwrite a file with the given UTF-8 contents.",
                {
                    "file_path": {"type": "string", "description": "Path inside the workspace."},
                    "content": {"type": "string", "description": "Full file contents to write."},
                },
                ["file_path", "content"],
            ),
            fn(
                "Edit",
                "Replace an exact string in a file. old_string must occur exactly once.",
                {
                    "file_path": {"type": "string", "description": "Path inside the workspace."},
                    "old_string": {"type": "string", "description": "Exact text to replace (must be unique)."},
                    "new_string": {"type": "string", "description": "Replacement text."},
                },
                ["file_path", "old_string", "new_string"],
            ),
        ]
    return schemas


class _Tools:
    """File tools plus the sandbox that keeps writes in `workspace` and reads in the readable roots."""

    def __init__(self, workspace: Path, readable_roots: list[Path]):
        self.workspace = workspace.resolve()
        self.readable_roots = [r.resolve() for r in readable_roots]

    def _resolve(self, raw: str) -> Path:
        path = Path(raw)
        if not path.is_absolute():
            path = self.workspace / path
        return path.resolve()

    def _check_read(self, path: Path, raw: str) -> None:
        allowed = path.is_relative_to(self.workspace) or any(
            path.is_relative_to(r) for r in self.readable_roots
        )
        if not allowed:
            raise _SandboxError(
                f"path {raw!r} is outside the allowed roots; reads must stay under "
                f"{self.workspace} or one of {self.readable_roots}."
            )

    def _check_write(self, path: Path, raw: str) -> None:
        if not path.is_relative_to(self.workspace):
            raise _SandboxError(
                f"path {raw!r} is outside the workspace; writes must stay under {self.workspace}."
            )

    def dispatch(self, name: str, args: dict[str, Any]) -> str:
        handler: Callable[[dict[str, Any]], str] | None = {
            "Read": self._read,
            "Glob": self._glob,
            "Grep": self._grep,
            "Write": self._write,
            "Edit": self._edit,
        }.get(name)
        if handler is None:
            raise _SandboxError(f"unknown tool {name!r}")
        return handler(args)

    def _read(self, args: dict[str, Any]) -> str:
        raw = str(args["file_path"])
        path = self._resolve(raw)
        self._check_read(path, raw)
        data = path.read_text(encoding="utf-8")
        if len(data.encode("utf-8")) > MAX_READ_BYTES:
            data = data[:MAX_READ_BYTES] + f"\n... (truncated at {MAX_READ_BYTES} bytes)"
        return data

    def _glob(self, args: dict[str, Any]) -> str:
        raw = str(args.get("path") or self.workspace)
        base = self._resolve(raw)
        self._check_read(base, raw)
        matches = sorted(str(p) for p in base.glob(str(args["pattern"])) if p.is_file())
        if not matches:
            return "(no matches)"
        if len(matches) > MAX_GLOB_MATCHES:
            matches = matches[:MAX_GLOB_MATCHES] + [f"... (truncated at {MAX_GLOB_MATCHES} matches)"]
        return "\n".join(matches)

    def _grep(self, args: dict[str, Any]) -> str:
        raw = str(args.get("path") or self.workspace)
        base = self._resolve(raw)
        self._check_read(base, raw)
        regex = re.compile(str(args["pattern"]))
        files = [base] if base.is_file() else [p for p in base.rglob("*") if p.is_file()]
        out: list[str] = []
        for file in files:
            try:
                text = file.read_text(encoding="utf-8")
            except (OSError, UnicodeDecodeError):
                continue
            for lineno, line in enumerate(text.splitlines(), start=1):
                if regex.search(line):
                    out.append(f"{file}:{lineno}:{line}")
                    if len(out) >= MAX_GREP_MATCHES:
                        out.append(f"... (truncated at {MAX_GREP_MATCHES} matches)")
                        return "\n".join(out)
        return "\n".join(out) if out else "(no matches)"

    def _write(self, args: dict[str, Any]) -> str:
        raw = str(args["file_path"])
        path = self._resolve(raw)
        self._check_write(path, raw)
        content = str(args["content"])
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return f"wrote {len(content)} chars to {path}"

    def _edit(self, args: dict[str, Any]) -> str:
        raw = str(args["file_path"])
        path = self._resolve(raw)
        self._check_write(path, raw)
        old = str(args["old_string"])
        new = str(args["new_string"])
        text = path.read_text(encoding="utf-8")
        count = text.count(old)
        if count == 0:
            raise _SandboxError(f"old_string not found in {path}")
        if count > 1:
            raise _SandboxError(f"old_string occurs {count} times in {path}; it must be unique")
        path.write_text(text.replace(old, new, 1), encoding="utf-8")
        return f"edited {path}"


def _accumulate_turn_usage(usage: Usage, response: Any) -> None:
    """Fold one chat response's token counts into the running per-call usage."""
    prompt = int(getattr(response, "prompt_eval_count", 0) or 0)
    output = int(getattr(response, "eval_count", 0) or 0)
    usage["input_tokens"] = int(usage.get("input_tokens", 0)) + prompt
    usage["output_tokens"] = int(usage.get("output_tokens", 0)) + output


class OllamaLocalAdapter(LLMAdapter):
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
        from ollama import AsyncClient

        tools = _Tools(workspace, readable_roots)
        schemas = _tool_schemas(allow_edits)
        allowed = set(READ_ONLY_TOOLS + (WRITE_TOOLS if allow_edits else []))

        client = AsyncClient()  # honours OLLAMA_HOST
        messages: list[dict[str, Any]] = [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt},
        ]
        events: list[tuple[str, str, str]] = []
        usage: Usage = {}

        for _ in range(MAX_TURNS):
            response = await client.chat(
                model=MODEL.value,
                messages=messages,
                tools=schemas,
                think=THINK,
                keep_alive=KEEP_ALIVE,
                options={"num_ctx": NUM_CTX},
            )
            _accumulate_turn_usage(usage, response)
            message = response.message

            if message.thinking:
                events.append((_now_hms(), "thinking", message.thinking.rstrip("\n")))
            if message.content:
                events.append((_now_hms(), "chat", message.content.rstrip("\n")))

            tool_calls = message.tool_calls or []
            # Echo the assistant turn (thinking + text + tool calls) back so the model
            # sees its own request alongside the tool results we append below.
            messages.append(
                {
                    "role": "assistant",
                    "content": message.content or "",
                    "thinking": message.thinking or "",
                    "tool_calls": [tc.model_dump() for tc in tool_calls],
                }
            )

            if not tool_calls:
                break

            for call in tool_calls:
                name = call.function.name
                args = dict(call.function.arguments or {})
                events.append((_now_hms(), f"tool: {name}", _format_args(args)))
                if name not in allowed:
                    result = f"ERROR: tool {name!r} is not available (allow_edits={allow_edits})."
                else:
                    try:
                        result = tools.dispatch(name, args)
                    except (
                        _SandboxError,
                        KeyError,
                        OSError,
                        UnicodeDecodeError,
                        ValueError,
                        NotImplementedError,
                        re.error,
                    ) as exception:
                        # A weak model routinely emits an invalid regex (re.error),
                        # an absolute glob pattern (NotImplementedError), or a missing
                        # arg (KeyError). Feed the failure back as a tool result so it
                        # can retry, instead of crashing the whole agent run.
                        result = f"ERROR: {type(exception).__name__}: {exception}"
                events.append((_now_hms(), f"tool-result: {name}", result))
                messages.append({"role": "tool", "tool_name": name, "content": result})
        else:
            events.append((_now_hms(), "diagnostic", f"stopped: reached MAX_TURNS={MAX_TURNS}"))
            log.warning("ollama agent hit MAX_TURNS=%d without finishing", MAX_TURNS)

        write_timeline(log_io, events)
        return usage or None
