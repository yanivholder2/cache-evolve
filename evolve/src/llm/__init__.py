"""LLM invocation layer: backend-agnostic agent contract plus shared token accounting."""

from src.llm.base_adapter import LLMAdapter
from src.llm.token_usage import (
    ZeroTokenUsageError,
    format_token_totals,
    reset_token_totals,
    snapshot_token_totals,
)
from src.llm.adapters.antigravity_sdk import AntigravitySdkAdapter
from src.llm.adapters.claude_sdk import ClaudeSdkAdapter

ADAPTER_REGISTRY: dict[str, type[LLMAdapter]] = {
    "antigravity-sdk": AntigravitySdkAdapter,
    "claude-sdk": ClaudeSdkAdapter,
}

__all__ = [
    "ADAPTER_REGISTRY",
    "LLMAdapter",
    "ZeroTokenUsageError",
    "format_token_totals",
    "reset_token_totals",
    "snapshot_token_totals",
]
