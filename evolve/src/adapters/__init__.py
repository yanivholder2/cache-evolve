"""LLM adapters: backend-specific implementations of the framework's agent-invocation contract."""

from .base_llm_adapter import (
    LLMAdapter,
    format_token_totals,
    snapshot_token_totals,
)
from .antigravity_sdk import AntigravitySdkAdapter
from .claude_sdk import ClaudeSdkAdapter

ADAPTER_REGISTRY: dict[str, type[LLMAdapter]] = {
    "antigravity-sdk": AntigravitySdkAdapter,
    "claude-sdk": ClaudeSdkAdapter,
}

__all__ = [
    "LLMAdapter",
    "ADAPTER_REGISTRY",
    "format_token_totals",
    "snapshot_token_totals",
]
