"""UTF-8 file IO helpers. Use these everywhere instead of bare `read_text`/`write_text`/`open`."""

from __future__ import annotations

from pathlib import Path


def read_utf8(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def write_utf8(path: Path, text: str) -> None:
    path.write_text(text, encoding="utf-8")


def append_utf8(path: Path, text: str) -> None:
    with open(path, "a", encoding="utf-8") as f:
        f.write(text)
