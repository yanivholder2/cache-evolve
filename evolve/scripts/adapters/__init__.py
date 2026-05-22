"""LLM adapters. `REGISTRY` maps `--ai-agent-tool` choice → adapter class."""

from .base_llm_adapter import LLMAdapter
from .antigravity_sdk import AntigravitySdkAdapter
from .claude_sdk import ClaudeSdkAdapter

REGISTRY: dict[str, type[LLMAdapter]] = {
    "antigravity-sdk": AntigravitySdkAdapter,
    "claude-sdk": ClaudeSdkAdapter,
}

__all__ = ["LLMAdapter", "REGISTRY"]
