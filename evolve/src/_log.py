"""Shared logger setup for evolve framework scripts.

Single format, single log file per cycle. The `[name:funcName]` prefix
(e.g. `[evolve:run_cycle]`, `[seal:main]`) disambiguates lines when multiple
scripts append to the same `evolve/logs/cycle-NNNN.log`.

Handlers are attached to the root logger so every `logging.getLogger(name)`
in the process inherits them — important when one script imports another.
"""

from __future__ import annotations

import logging
import sys
from pathlib import Path

FORMAT = "%(asctime)s %(levelname)s [%(name)s:%(funcName)s] %(message)s"
DATEFMT = "%H:%M:%S"

LEVELS = ("DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL")


def setup(log_file: Path | None, level: str = "DEBUG") -> None:
    """Attach stderr handler (once) + file handler to the root logger.

    When called repeatedly with different `log_file` values, the previous
    FileHandler is swapped — used by evolve.py's N-cycle loop so each cycle
    gets its own file. `level` (re)applies to the root logger and every
    attached handler.
    """

    lvl = logging.getLevelNamesMapping()[level.upper()]
    root = logging.getLogger()
    root.setLevel(lvl)
    fmt = logging.Formatter(FORMAT, DATEFMT)
    if not any(
        isinstance(h, logging.StreamHandler) and getattr(h, "stream", None) is sys.stderr
        for h in root.handlers
    ):
        sh = logging.StreamHandler(sys.stderr)
        sh.setFormatter(fmt)
        root.addHandler(sh)
    if log_file is not None:
        for h in list(root.handlers):
            if isinstance(h, logging.FileHandler):
                root.removeHandler(h)
                h.close()
        log_file.parent.mkdir(parents=True, exist_ok=True)
        fh = logging.FileHandler(log_file, encoding="utf-8")
        fh.setFormatter(fmt)
        root.addHandler(fh)
    for h in root.handlers:
        h.setLevel(lvl)
    logging.getLogger("claude_agent_sdk").setLevel(logging.WARNING)
