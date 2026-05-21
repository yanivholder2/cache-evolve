"""Shared fixtures for evolve tests.

Two autouse fixtures:
- session-scoped: prepend `tests/_mocks/` to PATH so the shim cargo runs
  instead of the real toolchain. The shim handles the variant-build
  invocations from build_variant / run_gates in milliseconds (by reading
  markers in the variant source) and transparently delegates any other
  cargo call to the real binary — so fitness plugins, dash-bench, etc.
  still see a working cargo.
- session-scoped: ensure `evolve/scripts/` is on sys.path so tests can
  `from evolve import run_cycle`, `from adapters import ...`, etc.
"""

import os
import sys
from pathlib import Path

import pytest

EVOLVE_DIR = Path(__file__).parent.parent
SCRIPTS_DIR = EVOLVE_DIR / "scripts"
TESTS_DIR = Path(__file__).parent
MOCKS_DIR = TESTS_DIR / "_mocks"

# Make evolve scripts + test helpers importable at collection time.
# (Fixtures run too late — pytest imports test modules before fixtures fire.)
for p in (str(SCRIPTS_DIR), str(TESTS_DIR)):
    if p not in sys.path:
        sys.path.insert(0, p)


@pytest.fixture(autouse=True, scope="session")
def mock_cargo_on_path():
    original = os.environ.get("PATH", "")
    os.environ["PATH"] = f"{MOCKS_DIR}{os.pathsep}{original}"
    yield MOCKS_DIR
    os.environ["PATH"] = original


