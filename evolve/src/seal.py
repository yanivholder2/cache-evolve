"""Mark a cycle directory immutable: drop write bits and write a SEALED marker.

The framework never writes into a sealed cycle by convention; chmod is the
safety net. Idempotent — re-running on an already-sealed dir is a no-op.
"""

from __future__ import annotations

import logging
import os
from datetime import datetime, timezone
from pathlib import Path

from src.paths import CycleLayout

log = logging.getLogger("seal")


def _strip_write_bits(root: Path) -> None:
    """Recursively clear write bits (a-w) on root and everything under it."""

    for dirpath, dirnames, filenames in os.walk(root):
        for name in filenames + dirnames:
            p = Path(dirpath) / name
            os.chmod(p, p.stat().st_mode & ~0o222)
    os.chmod(root, root.stat().st_mode & ~0o222)


def seal_cycle(cycle_dir: Path) -> None:
    """Seal a cycle directory. No-op if already sealed."""

    cycle_dir = cycle_dir.resolve()
    if not cycle_dir.is_dir():
        raise FileNotFoundError(f"not a directory: {cycle_dir}")

    marker = CycleLayout(root=cycle_dir).sealed_marker
    if marker.is_file():
        log.info("already sealed: %s", cycle_dir)
        return

    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    marker.write_text(f"{ts}\n", encoding="utf-8")
    _strip_write_bits(cycle_dir)
    log.info("sealed: %s", cycle_dir)
