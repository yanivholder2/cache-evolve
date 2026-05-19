"""Shared fixtures for evolve tests.

Autouse + session-scoped: prepend `tests/_mocks/` to PATH so the shim cargo
runs instead of the real toolchain. The shim handles the variant-build
invocations from scripts/build.sh and scripts/gate-check.sh in milliseconds
(by reading markers in the variant source) and transparently delegates any
other cargo call to the real binary — so fitness plugins, dash-bench, etc.
still see a working cargo.
"""

import os
from pathlib import Path

import pytest

MOCKS_DIR = Path(__file__).parent / "_mocks"


@pytest.fixture(autouse=True, scope="session")
def mock_cargo_on_path():
    original = os.environ.get("PATH", "")
    os.environ["PATH"] = f"{MOCKS_DIR}{os.pathsep}{original}"
    yield MOCKS_DIR
    os.environ["PATH"] = original
