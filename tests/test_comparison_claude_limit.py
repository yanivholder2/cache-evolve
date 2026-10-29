"""Usage-limit handling of the claude_code backend (comparison/claude_code_llm.py)."""

import asyncio
import datetime

import pytest

pytest.importorskip("openevolve")

from comparison import claude_code_llm as llm  # noqa: E402

NOW = datetime.datetime(2026, 10, 29, 14, 0)


@pytest.mark.parametrize("text", [
    "You've hit your session limit · resets 3pm (Asia/Jerusalem)",
    "Claude AI usage limit reached|1793300000",
    "You've reached your weekly limit. Resets Mon 9am",
])
def test_limit_notices_are_detected(text):
    assert llm.detect_usage_limit(text)


def test_model_text_mentioning_limits_is_not_a_notice():
    code_reply = "Let me explain the rate limit logic.\n" + "fn evict() {}\n" * 100
    assert not llm.detect_usage_limit(code_reply)
    assert not llm.detect_usage_limit("<<<<<<< SEARCH\nlet limit = 3;\n=======\nlet limit = 4;\n>>>>>>> REPLACE")


def test_wait_runs_to_the_named_reset_time_plus_a_minute():
    assert llm.seconds_until_reset("resets 2:20pm", NOW) == 20 * 60 + 60


def test_wait_is_capped_so_a_misread_reset_costs_one_probe():
    assert llm.seconds_until_reset("resets 11pm", NOW) == llm.POLL_S
    assert llm.seconds_until_reset("limit hit, try later", NOW) == llm.POLL_S


def test_past_reset_time_means_tomorrow():
    assert llm.seconds_until_reset("resets 1pm", NOW) == llm.POLL_S


def test_call_waits_for_reset_then_returns(monkeypatch):
    calls = []

    async def fake_parent(self, system_message, messages, **kwargs):
        calls.append(1)
        if len(calls) < 3:
            raise llm.UsageLimitReached("You've hit your session limit · resets 3pm")
        return "diff"

    slept = []

    async def fake_sleep(seconds):
        slept.append(seconds)

    monkeypatch.setattr(llm.ClaudeCodeLLM, "generate_with_context", fake_parent)
    monkeypatch.setattr(llm.asyncio, "sleep", fake_sleep)
    client = llm.IsolatedClaudeCodeLLM(type("Cfg", (), {"name": "m"})())
    assert asyncio.run(client.generate_with_context("sys", [])) == "diff"
    assert len(calls) == 3 and len(slept) == 2
