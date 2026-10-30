"""Usage is measured at the transport, including retries and rejected responses."""

import asyncio
import json
import subprocess
from concurrent.futures import ThreadPoolExecutor
from types import SimpleNamespace

import pytest

from comparison.token_usage import (
    claude_usage, openai_usage, record_usage, summarise_usage, write_usage_summary,
)


@pytest.fixture
def usage_dir(tmp_path, monkeypatch):
    monkeypatch.setenv("COMPARISON_LOG_DIR", str(tmp_path))
    return tmp_path


def test_normalisation_counts_cached_input_once():
    claude = claude_usage({"input_tokens": 10, "output_tokens": 20,
                          "cache_read_input_tokens": 30, "cache_creation_input_tokens": 40})
    api = openai_usage({"prompt_tokens": 40, "completion_tokens": 20,
                        "prompt_tokens_details": {"cached_tokens": 30},
                        "completion_tokens_details": {"reasoning_tokens": 15}})
    assert claude == {"input_tokens": 10, "output_tokens": 20,
                      "cache_read_tokens": 30, "cache_write_tokens": 40}
    assert api == {**claude, "cache_write_tokens": 0}
    assert claude_usage(None) is None
    assert openai_usage({}) is None


def test_summary_includes_concurrent_attempts_and_missing_usage(usage_dir):
    def record(_):
        record_usage(provider="test", model="m", status="response",
                     usage=claude_usage({"input_tokens": 2, "output_tokens": 3}), cost_usd=0.01)

    with ThreadPoolExecutor(max_workers=4) as pool:
        list(pool.map(record, range(30)))
    record_usage(provider="test", model="m", status="timeout")
    summary = write_usage_summary(usage_dir)
    assert summary["attempts"] == 31
    assert summary["attempts_missing_usage"] == 1
    assert summary["complete"] is False
    assert summary["totals"]["total_tokens"] == 150
    assert summary["reported_cost_usd"] == 0.3
    assert json.loads((usage_dir / "token_usage_summary.json").read_text()) == summary


def test_legacy_run_and_truncated_ledger_are_incomplete(usage_dir):
    assert summarise_usage(usage_dir)["complete"] is False
    (usage_dir / "token_usage.jsonl").write_text('{"usage":')
    summary = summarise_usage(usage_dir)
    assert summary["attempts_missing_usage"] == 1
    assert summary["complete"] is False


def cli_client(monkeypatch, usage_dir):
    pytest.importorskip("openevolve")
    from comparison import claude_code_llm as llm

    monkeypatch.setattr(llm, "claude_executable", lambda: "claude")
    monkeypatch.setattr(llm, "_scratch", lambda: str(usage_dir))
    return llm, llm.IsolatedClaudeCodeLLM(SimpleNamespace(name="m", retries=1, retry_delay=0))


def test_cli_json_reply_and_usage_survive_timeout_retry(monkeypatch, usage_dir):
    llm, client = cli_client(monkeypatch, usage_dir)
    commands = []
    raw = {"input_tokens": 10, "output_tokens": 20, "cache_read_input_tokens": 30}

    def run(cmd, **kwargs):
        commands.append(cmd)
        assert kwargs["input"] == "prompt"
        if len(commands) == 1:
            raise subprocess.TimeoutExpired(cmd, 1)
        return SimpleNamespace(returncode=0, stderr="", stdout=json.dumps({
            "result": "diff", "usage": raw, "total_cost_usd": 0.2,
        }))

    monkeypatch.setattr(llm.subprocess, "run", run)
    assert asyncio.run(client.generate_with_context("system", [{"role": "user", "content": "prompt"}])) == "diff"
    assert all(cmd[cmd.index("--output-format") + 1] == "json" for cmd in commands)
    assert not list(usage_dir.glob("openevolve-sys-*"))
    summary = summarise_usage(usage_dir)
    assert summary["attempts"] == 2
    assert summary["attempts_missing_usage"] == 1
    assert summary["totals"]["total_tokens"] == 60
    rows = [json.loads(line) for line in (usage_dir / "token_usage.jsonl").read_text().splitlines()]
    assert rows[1]["raw_usage"] == raw
    assert "prompt" not in rows[1]


@pytest.mark.parametrize("payload,returncode,exception", [
    ({"result": "You've hit your session limit", "is_error": True}, 1, "UsageLimitReached"),
    ({"is_error": True, "errors": ["You've hit your session limit"]}, 1, "UsageLimitReached"),
    ({"is_error": True, "usage": {"input_tokens": 1, "output_tokens": 2}}, 1, "RuntimeError"),
    ("plain text", 0, "RuntimeError"),
])
def test_cli_errors_still_record_usage(monkeypatch, usage_dir, payload, returncode, exception):
    llm, client = cli_client(monkeypatch, usage_dir)
    output = json.dumps(payload) if isinstance(payload, dict) else payload
    monkeypatch.setattr(llm.subprocess, "run", lambda *a, **kw: SimpleNamespace(
        returncode=returncode, stderr="", stdout=output,
    ))
    with pytest.raises(getattr(llm, exception, RuntimeError)):
        client._run_cli(["claude", "-p", "--output-format", "text", "prompt"], 1)
    assert summarise_usage(usage_dir)["attempts"] == 1


def test_api_transport_logs_retries_and_raw_usage(monkeypatch, usage_dir):
    pytest.importorskip("openevolve")
    import openai
    from openevolve.config import LLMModelConfig
    from comparison.openai_llm import UsageLoggingOpenAILLM

    calls = []
    raw = {"prompt_tokens": 40, "completion_tokens": 20,
           "prompt_tokens_details": {"cached_tokens": 30}}

    def create(**kwargs):
        calls.append(kwargs)
        if len(calls) == 1:
            raise RuntimeError("transient failure")
        return SimpleNamespace(usage=SimpleNamespace(model_dump=lambda: raw), choices=[
            SimpleNamespace(message=SimpleNamespace(content="diff")),
        ])

    completions = SimpleNamespace(create=create)
    sdk = SimpleNamespace(chat=SimpleNamespace(completions=completions), max_retries=99)
    monkeypatch.setattr(openai, "OpenAI", lambda **kwargs: sdk)
    model = LLMModelConfig(name="m", api_key="test", retries=1, retry_delay=0, timeout=10)
    client = UsageLoggingOpenAILLM(model)
    assert asyncio.run(client.generate("prompt")) == "diff"
    assert sdk.max_retries == 0
    assert len(calls) == 2
    summary = summarise_usage(usage_dir)
    assert summary["attempts"] == 2
    assert summary["attempts_missing_usage"] == 1
    assert summary["totals"]["input_tokens"] == 10
    assert summary["totals"]["total_tokens"] == 60


@pytest.mark.parametrize("raw", [None, {"prompt_tokens": 10, "completion_tokens": 4}])
def test_api_wrapper_with_real_sdk_and_mock_http(monkeypatch, usage_dir, raw):
    pytest.importorskip("openevolve")
    import httpx
    import openai
    from openevolve.config import LLMModelConfig
    from comparison.openai_llm import UsageLoggingOpenAILLM

    def respond(request):
        return httpx.Response(200, json={
            "id": "test", "object": "chat.completion", "created": 0, "model": "m",
            "choices": [{"index": 0, "finish_reason": "stop",
                         "message": {"role": "assistant", "content": "diff"}}],
            "usage": raw,
        })

    with openai.OpenAI(api_key="test", http_client=httpx.Client(
        transport=httpx.MockTransport(respond),
    )) as sdk:
        monkeypatch.setattr(openai, "OpenAI", lambda **kwargs: sdk)
        client = UsageLoggingOpenAILLM(LLMModelConfig(name="m", retries=0, timeout=10))
        assert asyncio.run(client.generate("prompt")) == "diff"
    summary = summarise_usage(usage_dir)
    assert summary["complete"] is (raw is not None)
    assert summary["totals"]["total_tokens"] == (14 if raw else 0)


def test_run_writes_summary_when_evolution_fails(monkeypatch, usage_dir):
    pytest.importorskip("openevolve")
    from comparison.run_openevolve import main

    def fail(**kwargs):
        record_usage(provider="test", model="m", status="error")
        raise RuntimeError("evolution failed")

    monkeypatch.setattr("openevolve.api.run_evolution", fail)
    with pytest.raises(RuntimeError, match="evolution failed"):
        main(["--subject", "lru_cache", "--iterations", "1", "--model", "m",
              "--out", str(usage_dir)])
    summary = json.loads((usage_dir / "token_usage_summary.json").read_text())
    assert summary["attempts"] == 1
    assert summary["complete"] is False


def test_collection_embeds_usage_totals(tmp_path, monkeypatch):
    from comparison import collect

    directory = tmp_path / "openevolve" / "rep0"
    monkeypatch.setenv("COMPARISON_LOG_DIR", str(directory))
    record_usage(provider="test", model="m", status="response",
                 usage=claude_usage({"input_tokens": 10, "output_tokens": 20}))
    monkeypatch.setattr(collect, "_plot", lambda *a, **kw: None)
    summary = collect.write_outputs({("openevolve", 0): []}, {}, tmp_path, rescored=False)
    usage = summary["openevolve/rep0"]["token_usage"]
    assert usage["complete"] is True
    assert usage["totals"]["total_tokens"] == 30
