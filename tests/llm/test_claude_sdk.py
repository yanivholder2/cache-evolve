"""Model validation for the claude-sdk adapter."""

from __future__ import annotations

import pytest

from evolve.llm.adapters.claude_sdk import ClaudeSdkAdapter


@pytest.mark.parametrize("model", ["claude-opus-5-5", "claude-opus-4-8", "claude-haiku-4-5-20251001"])
def test_accepts_any_claude_model(model):
    ClaudeSdkAdapter.validate_model(model)
    assert ClaudeSdkAdapter(model=model).model == model


@pytest.mark.parametrize("model", ["gemini-3-pro-preview", "gemma4:e4b", "opus-5-5", ""])
def test_rejects_non_claude_model(model):
    with pytest.raises(ValueError, match="claude-"):
        ClaudeSdkAdapter.validate_model(model)
    with pytest.raises(ValueError):
        ClaudeSdkAdapter(model=model)

