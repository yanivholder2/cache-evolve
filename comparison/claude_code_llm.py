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
import datetime
import glob
import logging
import os
import re
import shutil
import subprocess
import tempfile
from pathlib import Path

from openevolve.llm.claude_code import ClaudeCodeLLM

logger = logging.getLogger(__name__)

_SCRATCH = str(Path(tempfile.gettempdir()) / "openevolve-claude-cwd")


def _scratch() -> str:
    """The empty working directory, recreated if a temp cleaner removed it mid-run
    (a seed-44 run lost it after 14 iterations and every later call failed)."""
    Path(_SCRATCH).mkdir(parents=True, exist_ok=True)
    return _SCRATCH

# Per-call spend cap passed to `claude --max-budget-usd`. OpenEvolve's model config
# carries max_budget_usd=None, which the CLI rejects; a 16k-token Opus reply costs
# about $1.2, so the cap only guards against a runaway call.
MAX_BUDGET_USD = 5.0


class UsageLimitReached(RuntimeError):
    """The subscription's session/usage limit is exhausted; retrying now cannot work."""


# `claude -p` prints e.g. "You've hit your session limit · resets 3pm" as its reply, not as an
# error, so a short reply matching this is a limit notice and not model output.
_LIMIT = re.compile(r"(hit|reached|exceeded)\s+(your|the)\s+[\w\s-]*limit|usage limit|limit\s+reached", re.I)
_RESET = re.compile(r"resets?\s+(?:at\s+)?(\d{1,2})(?::(\d{2}))?\s*([ap]m)?", re.I)
_NOTICE_MAX_CHARS = 400
POLL_S = 1800  # re-probe at least this often while limited
MIN_WAIT_S = 120


def detect_usage_limit(text: str) -> bool:
    return len(text) <= _NOTICE_MAX_CHARS and bool(_LIMIT.search(text))


def seconds_until_reset(text: str, now: datetime.datetime | None = None) -> float:
    """Seconds to sleep before probing again: until the reset time the notice names (local
    time, +1 min), capped at POLL_S so a misread time or timezone only costs one extra probe."""
    now = now or datetime.datetime.now()
    match = _RESET.search(text)
    if not match:
        return POLL_S
    hour, minute, meridiem = int(match[1]), int(match[2] or 0), (match[3] or "").lower()
    if meridiem == "pm" and hour < 12:
        hour += 12
    elif meridiem == "am" and hour == 12:
        hour = 0
    if hour > 23 or minute > 59:
        return POLL_S
    reset = now.replace(hour=hour, minute=minute, second=0, microsecond=0)
    if reset <= now:
        reset += datetime.timedelta(days=1)
    return min(max((reset - now).total_seconds() + 60, MIN_WAIT_S), POLL_S)


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
        if not self.max_budget_usd:
            self.max_budget_usd = MAX_BUDGET_USD

    async def generate_with_context(self, system_message, messages, **kwargs):
        """Like the parent, but a spent subscription limit is waited out instead of failing
        the iteration: the call is retried until the limit resets (cost: nothing, the
        probe is rejected). The caller's per-iteration timeout must allow for the wait."""
        while True:
            try:
                return await super().generate_with_context(system_message, messages, **kwargs)
            except UsageLimitReached as limit:
                wait = seconds_until_reset(str(limit))
                logger.warning("Claude usage limit reached (%s); waiting %.0f min", limit, wait / 60)
                await asyncio.sleep(wait)

    def _run_cli(self, cmd: list, timeout: int) -> str:
        # The parent builds [claude, -p, ..., --system-prompt, <sys>, ..., <prompt>].
        cmd = list(cmd)
        prompt = cmd.pop()
        cwd = _scratch()
        cmd[0] = claude_executable()
        sys_file = None
        if "--system-prompt" in cmd:
            at = cmd.index("--system-prompt")
            handle, sys_file = tempfile.mkstemp(prefix="openevolve-sys-", suffix=".md", dir=cwd)
            with os.fdopen(handle, "w", encoding="utf-8") as f:
                f.write(cmd[at + 1])
            cmd[at:at + 2] = ["--system-prompt-file", sys_file]
        try:
            result = subprocess.run(
                cmd, input=prompt, capture_output=True, text=True, encoding="utf-8",
                errors="replace", timeout=timeout, cwd=cwd,
            )
        except subprocess.TimeoutExpired:
            raise asyncio.TimeoutError("Claude CLI subprocess timed out")
        finally:
            if sys_file:
                Path(sys_file).unlink(missing_ok=True)
        if result.returncode != 0 and result.stderr.strip():
            self._warn(result.stderr.strip()[:500])
        output = result.stdout.strip()
        for text in (output, result.stderr.strip()):
            if detect_usage_limit(text):
                raise UsageLimitReached(text)
        if not output:
            raise RuntimeError(f"Empty response from Claude CLI. stderr: {result.stderr[:500]}")
        return output

    @staticmethod
    def _warn(message: str) -> None:
        import logging
        logging.getLogger(__name__).warning("Claude CLI stderr: %s", message)


def init_isolated_claude_code_client(model_cfg):
    return IsolatedClaudeCodeLLM(model_cfg)
