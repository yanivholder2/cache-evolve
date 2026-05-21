"""LLM adapters. `REGISTRY` maps `--ai-agent-tool` choice → adapter class."""

from ._base import LLMAdapter
from .gemini import GeminiCliAdapter

REGISTRY: dict[str, type[LLMAdapter]] = {
    "gemini": GeminiCliAdapter,
}

__all__ = ["LLMAdapter", "REGISTRY"]
