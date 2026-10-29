"""OpenEvolve's Claude Code CLI backend, run from an empty directory.

`claude -p` is a full agent session that loads CLAUDE.md files and can read the
working directory. Launched from the repo it would see cache-evolve's sources
and past results, which OpenEvolve's text-only protocol does not give it, so the
shipped backend is wrapped to start in a scratch directory.

The shipped backend also passes the whole prompt as a command-line argument, which
Windows caps at about 32k characters (OpenEvolve prompts grow with every parent and
inspiration program), and decodes output with the locale codec. Here the prompt goes
over stdin, the system prompt through a file, and both streams are UTF-8."""

from __future__ import annotations

import asyncio
import glob
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

from openevolve.llm.claude_code import ClaudeCodeLLM

_SCRATCH = tempfile.mkdtemp(prefix="openevolve-claude-cwd-")

# Per-call spend cap passed to `claude --max-budget-usd`. OpenEvolve's model config
# carries max_budget_usd=None, which the CLI rejects; a 16k-token Opus reply costs
# about $1.2, so the cap only guards against a runaway call.
MAX_BUDGET_USD = 5.0


def claude_executable() -> str:
    """The `claude` CLI: $CLAUDE_CLI, else PATH, else the newest VS Code extension's
    bundled binary (which shares the user's login)."""
    explicit = os.environ.get("CLAUDE_CLI")
    if explicit:
        return explicit
    on_path = shutil.which("claude")
    if on_path:
        return on_path
    pattern = str(Path.home() / ".vscode" / "extensions" / "anthropic.claude-code-*" /
                  "resources" / "native-binary" / ("claude.exe" if os.name == "nt" else "claude"))
    bundled = sorted(glob.glob(pattern), key=lambda p: [int(x) for x in _version(p)])
    if bundled:
        return bundled[-1]
    raise FileNotFoundError("no `claude` CLI found; install it or set CLAUDE_CLI")


def _version(path: str) -> list[str]:
    name = Path(path).parents[2].name  # anthropic.claude-code-2.1.312-win32-x64
    digits = name.removeprefix("anthropic.claude-code-").split("-")[0].split(".")
    return [d if d.isdigit() else "0" for d in digits]


class IsolatedClaudeCodeLLM(ClaudeCodeLLM):
    def __init__(self, model_cfg=None):
        super().__init__(model_cfg)
        self.cwd = _SCRATCH
        if not self.max_budget_usd:
            self.max_budget_usd = MAX_BUDGET_USD

    def _run_cli(self, cmd: list, timeout: int) -> str:
        # The parent builds [claude, -p, ..., --system-prompt, <sys>, ..., <prompt>].
        cmd = list(cmd)
        prompt = cmd.pop()
        cmd[0] = claude_executable()
        sys_file = None
        if "--system-prompt" in cmd:
            at = cmd.index("--system-prompt")
            handle, sys_file = tempfile.mkstemp(prefix="openevolve-sys-", suffix=".md", dir=_SCRATCH)
            with os.fdopen(handle, "w", encoding="utf-8") as f:
                f.write(cmd[at + 1])
            cmd[at:at + 2] = ["--system-prompt-file", sys_file]
        try:
            result = subprocess.run(
                cmd, input=prompt, capture_output=True, text=True, encoding="utf-8",
                errors="replace", timeout=timeout, cwd=self.cwd,
            )
        except subprocess.TimeoutExpired:
            raise asyncio.TimeoutError("Claude CLI subprocess timed out")
        finally:
            if sys_file:
                Path(sys_file).unlink(missing_ok=True)
        if result.returncode != 0 and result.stderr.strip():
            self._warn(result.stderr.strip()[:500])
        output = result.stdout.strip()
        if not output:
            raise RuntimeError(f"Empty response from Claude CLI. stderr: {result.stderr[:500]}")
        return output

    @staticmethod
    def _warn(message: str) -> None:
        import logging
        logging.getLogger(__name__).warning("Claude CLI stderr: %s", message)


def init_isolated_claude_code_client(model_cfg):
    return IsolatedClaudeCodeLLM(model_cfg)
