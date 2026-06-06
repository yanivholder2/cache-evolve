"""LLM invocation layer: backend-agnostic agent contract plus shared token accounting."""

import importlib.util

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

# Importing an adapter module is SDK-free — each adapter defers its SDK import
# into the call path — so the registry is always fully populated. A backend is
# only unusable if its optional SDK is not installed, which BACKEND_EXTRAS makes
# checkable: every backend's SDK ships as a pip extra, so the map gives each one
# the import name to probe and the extra that provides it.
ADAPTER_REGISTRY: dict[str, type[LLMAdapter]] = {
    "antigravity-sdk": AntigravitySdkAdapter,
    "claude-sdk": ClaudeSdkAdapter,
    "ollama-local": OllamaLocalAdapter,
}

BACKEND_EXTRAS: dict[str, tuple[str, str]] = {
    "antigravity-sdk": ("google.antigravity", "antigravity"),
    "claude-sdk": ("claude_agent_sdk", "claude"),
    "ollama-local": ("ollama", "ollama"),
}


def _is_importable(module_name: str) -> bool:
    try:
        return importlib.util.find_spec(module_name) is not None
    except ModuleNotFoundError:
        # find_spec imports parent packages of a dotted name; a missing parent
        # raises rather than returning None. Either way the module is absent.
        return False


def backend_unavailable_reason(name: str) -> str | None:
    """Return an instructive message if `name`'s optional SDK is not installed,
    else None. Backends with no extra (shipped in core) are always available."""
    probe = BACKEND_EXTRAS.get(name)
    if probe is None:
        return None
    module_name, extra = probe
    if _is_importable(module_name):
        return None
    return (
        f"ai_agent_tool {name!r} needs the {extra!r} optional dependency, which "
        f"is not installed. Install it with: uv sync --extra {extra} "
        f"(or pip install 'evolve[{extra}]')."
    )


def backend_choices_help() -> str:
    """One-line list of every backend with its install hint, for error messages."""
    parts = []
    for name in sorted(ADAPTER_REGISTRY):
        extra = BACKEND_EXTRAS.get(name)
        suffix = f" (extra: {extra[1]})" if extra else " (no extra, ships in core)"
        parts.append(f"{name}{suffix}")
    return ", ".join(parts)


__all__ = [
    "ADAPTER_REGISTRY",
    "BACKEND_EXTRAS",
    "LLMAdapter",
    "ZeroTokenUsageError",
    "backend_choices_help",
    "backend_unavailable_reason",
    "format_token_totals",
    "reset_token_totals",
    "snapshot_token_totals",
]
