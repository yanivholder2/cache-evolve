"""Durable, provider-reported OpenEvolve usage (never text-length estimates)."""

from __future__ import annotations

import json
import os
import threading
import time
from collections import Counter
from pathlib import Path

_LOCK = threading.Lock()
TOKEN_KEYS = ("input_tokens", "output_tokens", "cache_read_tokens", "cache_write_tokens")


def claude_usage(raw: dict | None) -> dict | None:
    if not raw or not any(key in raw for key in ("input_tokens", "output_tokens")):
        return None
    return {
        "input_tokens": raw.get("input_tokens", 0) or 0,
        "output_tokens": raw.get("output_tokens", 0) or 0,
        "cache_read_tokens": raw.get("cache_read_input_tokens", 0) or 0,
        "cache_write_tokens": raw.get("cache_creation_input_tokens", 0) or 0,
    }


def openai_usage(raw: dict | None) -> dict | None:
    if not raw or not any(key in raw for key in ("prompt_tokens", "completion_tokens")):
        return None
    # OpenAI's prompt count includes cached input; Claude's input count excludes it.
    cached = (raw.get("prompt_tokens_details") or {}).get("cached_tokens", 0) or 0
    return {
        "input_tokens": (raw.get("prompt_tokens", 0) or 0) - cached,
        "output_tokens": raw.get("completion_tokens", 0) or 0,
        "cache_read_tokens": cached,
        "cache_write_tokens": 0,
    }


def record_usage(*, provider: str, model: str, status: str, usage: dict | None = None,
                 raw_usage: dict | None = None, cost_usd: float | None = None) -> None:
    """One row per CLI invocation/API attempt, including failures and retries.

    Called inside the blocking transport, so a late response after an outer async
    timeout is still recorded. Do not store prompts, replies, or credentials.
    """
    directory = os.environ.get("COMPARISON_LOG_DIR")
    if not directory:
        return
    row = {"time": time.time(), "provider": provider, "model": model, "status": status,
           "usage": usage, "raw_usage": raw_usage, "cost_usd": cost_usd}
    path = Path(directory) / "token_usage.jsonl"
    path.parent.mkdir(parents=True, exist_ok=True)
    with _LOCK, path.open("a", encoding="utf-8") as f:
        f.write(json.dumps(row, ensure_ascii=False) + "\n")


def summarise_usage(directory: Path) -> dict:
    path = directory / "token_usage.jsonl"
    totals = Counter({key: 0 for key in TOKEN_KEYS})
    attempts = missing = cost_reports = 0
    cost = 0.0
    if path.is_file():
        for line in path.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            attempts += 1
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                # A process killed mid-write leaves an explicitly incomplete account.
                missing += 1
                continue
            if row["usage"] is None:
                missing += 1
            else:
                totals.update(row["usage"])
            if row.get("cost_usd") is not None:
                cost_reports += 1
                cost += row["cost_usd"]
    return {
        "attempts": attempts, "attempts_missing_usage": missing,
        "complete": path.is_file() and missing == 0,
        "totals": {**totals, "total_tokens": sum(totals[key] for key in TOKEN_KEYS)},
        "reported_cost_usd": round(cost, 8) if cost_reports else None,
        "attempts_with_cost": cost_reports,
    }


def write_usage_summary(directory: Path) -> dict:
    summary = summarise_usage(directory)
    (directory / "token_usage_summary.json").write_text(
        json.dumps(summary, indent=2) + "\n", encoding="utf-8",
    )
    return summary
