"""UTF-8 file IO helpers. Use these everywhere instead of bare `read_text`/`write_text`/`open`."""

from __future__ import annotations

import json
from pathlib import Path
from typing import TextIO


def read_utf8(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def write_utf8(path: Path, text: str) -> None:
    path.write_text(text, encoding="utf-8")


def append_utf8(path: Path, text: str) -> None:
    with open_utf8_append(path) as f:
        f.write(text)


def open_utf8_write(path: Path) -> TextIO:
    """Open `path` for streaming UTF-8 writes (truncates). Caller closes."""
    return path.open("w", encoding="utf-8")


def open_utf8_append(path: Path) -> TextIO:
    """Open `path` for streaming UTF-8 appends. Caller closes."""
    return path.open("a", encoding="utf-8")


def read_utf8_tail(path: Path, max_bytes: int) -> str:
    """Return up to the trailing `max_bytes` of `path`, UTF-8 decoded (replacing bad bytes)."""

    size = path.stat().st_size
    with path.open("rb") as f:
        if size > max_bytes:
            f.seek(size - max_bytes)
        data = f.read()
    return data.decode("utf-8", errors="replace")


def write_json(path: Path, data: object) -> None:
    """Pretty-print JSON with indent=2 and a trailing newline. Use for all .json file writes."""

    write_utf8(path, json.dumps(data, indent=2) + "\n")
