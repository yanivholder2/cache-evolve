"""LLM adapters: backend-specific implementations of the framework's agent-invocation contract."""

from .base_llm_adapter import LLMAdapter
from .antigravity_sdk import AntigravitySdkAdapter
from .claude_sdk import ClaudeSdkAdapter

REGISTRY: dict[str, type[LLMAdapter]] = {
    "antigravity-sdk": AntigravitySdkAdapter,
    "claude-sdk": ClaudeSdkAdapter,
}

__all__ = ["LLMAdapter", "REGISTRY"]
