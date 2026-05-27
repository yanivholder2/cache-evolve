"""Validation tests for the LLMAdapter base class."""

from __future__ import annotations

from pathlib import Path

import pytest

from evolve._io import read_utf8, write_utf8
from evolve.llm.base_adapter import LLMAdapter
from evolve.llm.token_usage import ZeroTokenUsageError

# Adapter raises on zero token usage, so happy-path tests must return non-zero.
_OK_USAGE = {"input_tokens": 1, "output_tokens": 1}


class _NoopAdapter(LLMAdapter):
    async def call_llm_and_log(self, *args, **kwargs):  # noqa: ANN001
        return _OK_USAGE


def _prep_workspace(tmp_path: Path) -> Path:
    workspace = tmp_path / "ws"
    workspace.mkdir()
    write_utf8(workspace / "task.md", "stub")
    return workspace


def test_run_rejects_empty_readable_roots(tmp_path):
    workspace = _prep_workspace(tmp_path)
    with pytest.raises(ValueError, match="readable_roots must be non-empty"):
        _NoopAdapter().run(
            agent_name="hypotheses-writer",
            workspace=workspace,
            readable_roots=[],
            allow_edits=False,
        )


def test_run_rejects_nonexistent_readable_root(tmp_path):
    workspace = _prep_workspace(tmp_path)
    ghost = tmp_path / "does-not-exist"
    with pytest.raises(FileNotFoundError, match="readable_root does not exist"):
        _NoopAdapter().run(
            agent_name="hypotheses-writer",
            workspace=workspace,
            readable_roots=[ghost],
            allow_edits=False,
        )


def test_run_resolves_relative_readable_roots(tmp_path, monkeypatch):
    workspace = _prep_workspace(tmp_path)
    relative_root = tmp_path / "ro"
    relative_root.mkdir()
    monkeypatch.chdir(tmp_path)

    captured: dict = {}

    class _CaptureAdapter(LLMAdapter):
        async def call_llm_and_log(self, *args, **kwargs):  # noqa: ANN001
            captured["readable_roots"] = kwargs["readable_roots"]
            return _OK_USAGE

    _CaptureAdapter().run(
        agent_name="hypotheses-writer",
        workspace=workspace,
        readable_roots=[Path("ro")],
        allow_edits=False,
    )
    assert captured["readable_roots"] == [relative_root.resolve()]


def test_run_requires_task_md_in_workspace(tmp_path):
    workspace = tmp_path / "ws"
    workspace.mkdir()
    with pytest.raises(FileNotFoundError, match="task.md not found"):
        _NoopAdapter().run(
            agent_name="hypotheses-writer",
            workspace=workspace,
            readable_roots=[tmp_path],
            allow_edits=False,
        )


def test_run_creates_log_file_in_workspace(tmp_path):
    workspace = _prep_workspace(tmp_path)
    _NoopAdapter().run(
        agent_name="hypotheses-writer",
        workspace=workspace,
        readable_roots=[tmp_path],
        allow_edits=False,
    )
    assert (workspace / "hypotheses-writer.log.md").is_file()


def test_run_writes_header_fields_to_log(tmp_path):
    workspace = _prep_workspace(tmp_path)
    _NoopAdapter().run(
        agent_name="hypotheses-writer",
        workspace=workspace,
        readable_roots=[tmp_path],
        allow_edits=True,
    )
    text = read_utf8(workspace / "hypotheses-writer.log.md")
    header, _, _ = text.partition("\n---\n")
    assert "# Agent log: hypotheses-writer" in header
    assert "- adapter: _NoopAdapter" in header
    assert f"- workspace: {workspace}" in header
    assert "- allow_edits: True" in header
    assert "- started: " in header


def test_run_writes_footer_with_status_ok_and_usage(tmp_path):
    workspace = _prep_workspace(tmp_path)

    class _UsageAdapter(LLMAdapter):
        async def call_llm_and_log(self, *args, **kwargs):  # noqa: ANN001
            return {"input_tokens": 42, "output_tokens": 7}

    _UsageAdapter().run(
        agent_name="hypotheses-writer",
        workspace=workspace,
        readable_roots=[tmp_path],
        allow_edits=False,
    )
    text = read_utf8(workspace / "hypotheses-writer.log.md")
    _, sep, footer = text.partition("\n---\n")
    assert sep, "footer separator missing"
    assert "- status: ok" in footer
    assert "- finished: " in footer
    assert "- duration: " in footer
    assert "- input_tokens: 42" in footer
    assert "- output_tokens: 7" in footer


def test_run_records_error_status_and_reraises(tmp_path):
    workspace = _prep_workspace(tmp_path)

    class _BoomAdapter(LLMAdapter):
        async def call_llm_and_log(self, *args, **kwargs):  # noqa: ANN001
            raise RuntimeError("backend exploded")

    with pytest.raises(RuntimeError, match="backend exploded"):
        _BoomAdapter().run(
            agent_name="hypotheses-writer",
            workspace=workspace,
            readable_roots=[tmp_path],
            allow_edits=False,
        )
    text = read_utf8(workspace / "hypotheses-writer.log.md")
    _, sep, footer = text.partition("\n---\n")
    assert sep, "footer separator missing"
    assert "- status: error: RuntimeError: backend exploded" in footer


def test_run_requires_agent_prompt_file(tmp_path):
    workspace = _prep_workspace(tmp_path)
    with pytest.raises(FileNotFoundError, match="agent prompt not found"):
        _NoopAdapter().run(
            agent_name="not-a-real-agent",
            workspace=workspace,
            readable_roots=[tmp_path],
            allow_edits=False,
        )


def test_run_raises_when_usage_is_none(tmp_path):
    workspace = _prep_workspace(tmp_path)

    class _NoUsageAdapter(LLMAdapter):
        async def call_llm_and_log(self, *args, **kwargs):  # noqa: ANN001
            return None

    with pytest.raises(ZeroTokenUsageError):
        _NoUsageAdapter().run(
            agent_name="hypotheses-writer",
            workspace=workspace,
            readable_roots=[tmp_path],
            allow_edits=False,
        )


def test_run_raises_when_both_input_and_output_tokens_are_zero(tmp_path):
    workspace = _prep_workspace(tmp_path)

    class _ZeroTokenAdapter(LLMAdapter):
        async def call_llm_and_log(self, *args, **kwargs):  # noqa: ANN001
            return {"input_tokens": 0, "output_tokens": 0, "cache_read_tokens": 1234}

    with pytest.raises(ZeroTokenUsageError):
        _ZeroTokenAdapter().run(
            agent_name="hypotheses-writer",
            workspace=workspace,
            readable_roots=[tmp_path],
            allow_edits=False,
        )


def test_run_accepts_nonzero_input_with_zero_output(tmp_path):
    workspace = _prep_workspace(tmp_path)

    class _OnlyInputAdapter(LLMAdapter):
        async def call_llm_and_log(self, *args, **kwargs):  # noqa: ANN001
            return {"input_tokens": 42, "output_tokens": 0}

    _OnlyInputAdapter().run(
        agent_name="hypotheses-writer",
        workspace=workspace,
        readable_roots=[tmp_path],
        allow_edits=False,
    )
