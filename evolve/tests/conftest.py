"""Test collection wiring for the evolve test suite."""

import importlib.util
import os
import sys
from pathlib import Path

import pytest

EVOLVE_DIR = Path(__file__).parent.parent
SCRIPTS_DIR = EVOLVE_DIR / "scripts"
TESTS_DIR = Path(__file__).parent
MOCKS_DIR = TESTS_DIR / "_mocks"


def _register_module(name: str, file: Path, *, is_package: bool = False) -> None:
    """Load `file` as `name` without leaking its parent onto sys.path."""
    if name in sys.modules:
        return
    submodule_search = [str(file.parent)] if is_package else None
    spec = importlib.util.spec_from_file_location(
        name, file, submodule_search_locations=submodule_search
    )
    if spec is None or spec.loader is None:
        raise RuntimeError(f"could not build spec for {name} from {file}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)


# Pytest imports test modules before fixtures fire — must register at import time.
_register_module("src", EVOLVE_DIR / "src" / "__init__.py", is_package=True)
_register_module("evolve", SCRIPTS_DIR / "evolve.py")

if str(TESTS_DIR) not in sys.path:
    sys.path.insert(0, str(TESTS_DIR))


def pytest_addoption(parser):
    parser.addoption(
        "--slow",
        action="store_true",
        default=False,
        help="run tests marked `slow` (real cargo builds; adds ~2min)",
    )


def pytest_configure(config):
    config.addinivalue_line(
        "markers", "slow: end-to-end test that drives a real cargo build"
    )


def pytest_collection_modifyitems(config, items):
    if config.getoption("--slow"):
        return
    skip_slow = pytest.mark.skip(reason="needs --slow to run")
    for item in items:
        if "slow" in item.keywords:
            item.add_marker(skip_slow)


@pytest.fixture(autouse=True, scope="session")
def mock_cargo_on_path():
    original = os.environ.get("PATH", "")
    os.environ["PATH"] = f"{MOCKS_DIR}{os.pathsep}{original}"
    yield MOCKS_DIR
    os.environ["PATH"] = original


