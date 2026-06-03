"""LLM invocation layer: backend-agnostic agent contract plus shared token accounting."""

from evolve.llm.base_adapter import LLMAdapter
from evolve.llm.token_usage import (
    ZeroTokenUsageError,
    format_token_totals,
    reset_token_totals,
    snapshot_token_totals,
)
from evolve.llm.adapters.antigravity_sdk import AntigravitySdkAdapter
from evolve.llm.adapters.claude_sdk import ClaudeSdkAdapter
from evolve.llm.adapters.ollama_local import OllamaLocalAdapter

ADAPTER_REGISTRY: dict[str, type[LLMAdapter]] = {
    "antigravity-sdk": AntigravitySdkAdapter,
    "claude-sdk": ClaudeSdkAdapter,
    "ollama-local": OllamaLocalAdapter,
}

__all__ = [
    "ADAPTER_REGISTRY",
    "LLMAdapter",
    "ZeroTokenUsageError",
    "format_token_totals",
    "reset_token_totals",
    "snapshot_token_totals",
]
