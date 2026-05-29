"""Shared logger setup for evolve framework scripts.

Handlers are attached to the root logger so every `logging.getLogger(name)`
in the process inherits them — important when one script imports another.
"""

from __future__ import annotations

import logging
import sys
from pathlib import Path

FORMAT = "%(asctime)s %(levelname)-8s [%(threadName)s : %(name)s : %(funcName)s] %(message)s"
DATEFMT = "%H:%M:%S"

LOG_LEVELS = ("DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL")


class _ColumnFormatter(logging.Formatter):
    """Pad each variable-width field to a fixed column so terminal output

    aligns like a table; truncate (with a trailing "…") anything longer.
    Used for the stderr handler only — the file handler keeps full-width
    fields so on-disk logs never lose information.
    """

    WIDTHS = {"threadName": 12, "name": 30, "funcName": 18}

    def format(self, record: logging.LogRecord) -> str:
        # Copy: stderr and file handlers format the same record; mutating the
        # original would corrupt the file handler's full-width output.
        record = logging.makeLogRecord(record.__dict__)
        for attr, width in self.WIDTHS.items():
            value = str(getattr(record, attr, ""))
            if len(value) > width:
                value = value[: width - 1] + "…"
            setattr(record, attr, value.ljust(width))
        return super().format(record)


def setup(log_file: Path | None, level: str = "DEBUG") -> None:
    """Attach stderr handler (once) + file handler to the root logger.

    Repeated calls with a different `log_file` swap the FileHandler so each
    cycle can own its own file. `level` sets the verbosity of our own
    `evolve` loggers; third-party dependencies stay at WARNING so their
    DEBUG/INFO chatter never reaches our handlers.
    """

    lvl = logging.getLevelNamesMapping()[level.upper()]
    root = logging.getLogger()
    # Root is the floor for every logger without an explicit level — i.e. all
    # dependencies. Keep it at WARNING (or quieter, if our own level is) so deps
    # never out-talk us. Our code is opted up to `lvl` via the "evolve" logger.
    root.setLevel(max(lvl, logging.WARNING))
    logging.getLogger("evolve").setLevel(lvl)
    fmt = logging.Formatter(FORMAT, DATEFMT)
    if not any(
        isinstance(h, logging.StreamHandler) and getattr(h, "stream", None) is sys.stderr
        for h in root.handlers
    ):
        sh = logging.StreamHandler(sys.stderr)
        sh.setFormatter(_ColumnFormatter(FORMAT, DATEFMT))
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
