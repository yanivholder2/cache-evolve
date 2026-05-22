"""Gemini CLI adapter — shells out to `gemini -p <prompt>`."""

from __future__ import annotations

import logging
import subprocess

from ._base import LLMAdapter

log = logging.getLogger("llm-adapter.gemini")


class GeminiCliAdapter(LLMAdapter):
    def _call_llm(self, system_prompt: str, user_prompt: str, *, read_only: bool) -> None:
        prompt = f"{system_prompt}\n\n---\n\n{user_prompt}"
        log.debug("invoking gemini -p (prompt bytes=%d, read_only=%s)", len(prompt), read_only)
        subprocess.run(["gemini", "-p", prompt], check=True)
