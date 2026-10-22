"""OpenEvolve's Claude Code CLI backend, run from an empty directory.

`claude -p` is a full agent session that loads CLAUDE.md files and can read the
working directory. Launched from the repo it would see cache-evolve's sources
and past results, which OpenEvolve's text-only protocol does not give it, so the
shipped backend is wrapped to start in a scratch directory."""

from __future__ import annotations

import tempfile

from openevolve.llm.claude_code import ClaudeCodeLLM

_SCRATCH = tempfile.mkdtemp(prefix="openevolve-claude-cwd-")


class IsolatedClaudeCodeLLM(ClaudeCodeLLM):
    def __init__(self, model_cfg=None):
        super().__init__(model_cfg)
        self.cwd = _SCRATCH


def init_isolated_claude_code_client(model_cfg):
    return IsolatedClaudeCodeLLM(model_cfg)
