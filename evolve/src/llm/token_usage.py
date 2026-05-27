"""Token/usage accounting shared across LLM adapters."""

from __future__ import annotations

import threading
from collections import Counter
from pathlib import Path
from typing import Any

from src._io import read_utf8_tail

Usage = dict[str, Any]


# Convention: adapters normalise their backend-specific usage fields so that
# token-count entries end in `_tokens`. Counting tokens (not USD) keeps the
# signal portable across providers, each of which prices differently; non-token
# entries like `cost_usd` are ignored by this filter.
def _is_token_key(key: str) -> bool:
    return key.endswith("_tokens")


_token_totals: Counter[str] = Counter()
_token_lock = threading.Lock()


def accumulate_usage(usage: Usage | None) -> None:
    """Add `usage` into the process-wide token totals. No-op for None/empty."""

    if not usage:
        return
    with _token_lock:
        for key, value in usage.items():
            if not _is_token_key(key) or not value:
                continue
            # bool is an int subclass — exclude it explicitly to catch contract violations.
            if not isinstance(value, int) or isinstance(value, bool):
                raise TypeError(
                    f"usage[{key!r}] must be int, got {type(value).__name__}: {value!r}"
                )
            _token_totals[key] += value


def snapshot_token_totals() -> dict[str, int]:
    """Return a snapshot of cumulative token totals across all adapter calls."""

    with _token_lock:
        return dict(_token_totals)


def reset_token_totals() -> None:
    """Clear the process-wide token totals. Intended for test isolation."""

    with _token_lock:
        _token_totals.clear()


def format_token_totals(totals: dict[str, int]) -> str:
    """Render `totals` as a flat `k=v` string for log lines."""

    if not totals:
        return "none"
    return " ".join(f"{key}={totals[key]}" for key in sorted(totals))


def format_usage(usage: Usage | None) -> str:
    if not usage:
        return "none"
    return ", ".join(f"{key}={value}" for key, value in usage.items())


class ZeroTokenUsageError(RuntimeError):
    """Backend returned a 'successful' result with no token usage.

    Almost always a silent backend failure (session limit, quota, dropped
    connection). Raising here points at the agent log's chat tail instead
    of letting a downstream `adapter wrote 0 files` ValueError swallow it.
    """


# Tail of the agent log inspected for the chat message that explains a
# zero-token "success". 4 KiB is enough to surface the session-limit/quota
# banner the backend usually prints in the final chat block.
_CHAT_TAIL_BYTES = 4096


def _chat_tail(log_path: Path) -> str:
    """Return the trailing bytes of the agent log, or an explanatory marker."""

    try:
        return read_utf8_tail(log_path, _CHAT_TAIL_BYTES)
    except OSError as exception:
        return f"<could not read {log_path}: {exception}>"


def raise_if_zero_token(agent_name: str, usage: Usage | None, log_path: Path) -> None:
    """Convert a no-token "success" into a loud failure with the chat tail.

    A backend that hits its session limit returns status=ok with all-zero
    usage; without this, the next validator step fails seconds later with
    no breadcrumb tying it to the silent backend failure.
    """

    if usage is None:
        raise ZeroTokenUsageError(
            f"agent {agent_name!r} returned no usage info — likely silent backend "
            f"failure. tail of {log_path}:\n{_chat_tail(log_path)}"
        )
    # input_tokens / output_tokens are the universal proof-of-work signal:
    # every adapter normalises its backend's prompt/completion counts to these
    # two keys. Cache/thinking/total counts don't prove the model produced
    # output (a cache hit can be non-zero while output is zero).
    input_tokens = int(usage.get("input_tokens") or 0)
    output_tokens = int(usage.get("output_tokens") or 0)
    if input_tokens == 0 and output_tokens == 0:
        raise ZeroTokenUsageError(
            f"agent {agent_name!r} returned status=ok but recorded 0 input AND 0 output "
            f"tokens (usage={format_usage(usage)}) — almost always a silent backend "
            f"failure (session limit, quota, dropped connection). tail of {log_path}:\n"
            f"{_chat_tail(log_path)}"
        )
