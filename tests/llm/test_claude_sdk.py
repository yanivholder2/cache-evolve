"""Model validation and CLI-path selection for the claude-sdk adapter."""

from __future__ import annotations

import time
from concurrent.futures import ThreadPoolExecutor

import pytest

from evolve.llm.adapters import claude_sdk
from evolve.llm.adapters.claude_sdk import CLI_PATH_ENV, ClaudeSdkAdapter, resolve_cli_path


@pytest.mark.parametrize("model", ["claude-opus-5-5", "claude-opus-4-8", "claude-haiku-4-5-20251001"])
def test_accepts_any_claude_model(model):
    ClaudeSdkAdapter.validate_model(model)
    assert ClaudeSdkAdapter(model=model).model == model


@pytest.mark.parametrize(
    "model",
    ["gemini-3-pro-preview", "gemma4:e4b", "opus-5-5", "", "claude-", "claude-opus-5.5", "claude-opus"],
)
def test_rejects_non_claude_model(model):
    with pytest.raises(ValueError, match="claude-"):
        ClaudeSdkAdapter.validate_model(model)
    with pytest.raises(ValueError):
        ClaudeSdkAdapter(model=model)


@pytest.fixture
def fresh_cli_cache(monkeypatch):
    monkeypatch.delenv(CLI_PATH_ENV, raising=False)
    monkeypatch.setattr(claude_sdk.os, "name", "posix")
    claude_sdk._resolve_cli_path.cache_clear()
    yield
    claude_sdk._resolve_cli_path.cache_clear()


def _versions(monkeypatch, *, bundled, system, table):
    monkeypatch.setattr(claude_sdk, "_bundled_cli", lambda: bundled)
    monkeypatch.setattr(claude_sdk.shutil, "which", lambda name: system if name == "claude" else None)
    monkeypatch.setattr(claude_sdk, "_cli_version", lambda path: table.get(path))


def test_picks_newer_system_cli(fresh_cli_cache, monkeypatch):
    _versions(monkeypatch, bundled="/sdk/claude", system="/usr/bin/claude",
              table={"/sdk/claude": (2, 1, 9), "/usr/bin/claude": (2, 1, 10)})
    assert resolve_cli_path() == "/usr/bin/claude"


def test_picks_newer_bundled_cli(fresh_cli_cache, monkeypatch):
    _versions(monkeypatch, bundled="/sdk/claude", system="/usr/bin/claude",
              table={"/sdk/claude": (3, 0, 0), "/usr/bin/claude": (2, 9, 99)})
    assert resolve_cli_path() == "/sdk/claude"


def test_skips_unversionable_candidate(fresh_cli_cache, monkeypatch):
    _versions(monkeypatch, bundled="/sdk/claude", system="/usr/bin/claude",
              table={"/sdk/claude": (1, 0, 0)})
    assert resolve_cli_path() == "/sdk/claude"


def test_none_when_no_candidates(fresh_cli_cache, monkeypatch):
    _versions(monkeypatch, bundled=None, system=None, table={})
    assert resolve_cli_path() is None


def test_env_var_wins(fresh_cli_cache, monkeypatch):
    def boom(*_args, **_kwargs):
        raise AssertionError("auto-selection must not run when the env var is set")

    monkeypatch.setenv(CLI_PATH_ENV, "/opt/claude")
    monkeypatch.setattr(claude_sdk, "_bundled_cli", boom)
    monkeypatch.setattr(claude_sdk, "_cli_version", boom)
    assert resolve_cli_path() == "/opt/claude"


def test_windows_defers_to_sdk(fresh_cli_cache, monkeypatch):
    monkeypatch.setattr(claude_sdk.os, "name", "nt")
    _versions(monkeypatch, bundled="/sdk/claude", system="/usr/bin/claude",
              table={"/sdk/claude": (1, 0, 0), "/usr/bin/claude": (2, 0, 0)})
    assert resolve_cli_path() is None


def test_result_is_cached(fresh_cli_cache, monkeypatch):
    calls = []

    def version(path):
        calls.append(path)
        return (1, 0, 0)

    monkeypatch.setattr(claude_sdk, "_bundled_cli", lambda: "/sdk/claude")
    monkeypatch.setattr(claude_sdk.shutil, "which", lambda name: None)
    monkeypatch.setattr(claude_sdk, "_cli_version", version)
    assert resolve_cli_path() == resolve_cli_path() == "/sdk/claude"
    assert calls == ["/sdk/claude"]


def test_concurrent_first_calls_probe_once(fresh_cli_cache, monkeypatch):
    # Per-variant worker threads start their first agent calls together; only
    # one of them may run the `--version` probes.
    calls = []

    def version(path):
        calls.append(path)
        time.sleep(0.05)
        return (1, 0, 0)

    monkeypatch.setattr(claude_sdk, "_bundled_cli", lambda: "/sdk/claude")
    monkeypatch.setattr(claude_sdk.shutil, "which", lambda name: None)
    monkeypatch.setattr(claude_sdk, "_cli_version", version)
    with ThreadPoolExecutor(max_workers=4) as pool:
        results = list(pool.map(lambda _: resolve_cli_path(), range(4)))
    assert results == ["/sdk/claude"] * 4
    assert calls == ["/sdk/claude"]


def test_cli_version_parses_semver(monkeypatch):
    class _Done:
        stdout = "2.1.10 (Claude Code)\n"

    monkeypatch.setattr(claude_sdk.subprocess, "run", lambda *a, **k: _Done())
    assert claude_sdk._cli_version("/x/claude") == (2, 1, 10)


def test_cli_version_none_on_failure(monkeypatch):
    def fail(*_args, **_kwargs):
        raise OSError("no such file")

    monkeypatch.setattr(claude_sdk.subprocess, "run", fail)
    assert claude_sdk._cli_version("/missing/claude") is None
