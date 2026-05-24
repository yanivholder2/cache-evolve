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

# Top-level `scripts/` package at repo root (e.g. `from scripts import caffeine_runner`).
REPO_ROOT = EVOLVE_DIR.parent
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

if str(TESTS_DIR) not in sys.path:
    sys.path.insert(0, str(TESTS_DIR))


def pytest_addoption(parser):
    parser.addoption(
        "--slow",
        action="store_true",
        default=False,
        help="run tests marked `slow` (real cargo builds; adds ~2min)",
    )
    parser.addoption(
        "--caffeine-needed",
        action="store_true",
        default=False,
        help="run tests marked `caffeine_needed` (require ~/projects/caffeine)",
    )


def pytest_configure(config):
    config.addinivalue_line(
        "markers", "slow: end-to-end test that drives a real cargo build"
    )
    config.addinivalue_line(
        "markers",
        "caffeine_needed: end-to-end test that requires ~/projects/caffeine "
        "and a working JDK; invoke explicitly with --caffeine-needed, never in CI",
    )


def pytest_collection_modifyitems(config, items):
    if not config.getoption("--slow"):
        skip_slow = pytest.mark.skip(reason="needs --slow to run")
        for item in items:
            if "slow" in item.keywords:
                item.add_marker(skip_slow)
    if not config.getoption("--caffeine-needed"):
        skip_caffeine = pytest.mark.skip(reason="needs --caffeine-needed to run")
        for item in items:
            if "caffeine_needed" in item.keywords:
                item.add_marker(skip_caffeine)


@pytest.fixture(autouse=True, scope="session")
def mock_cargo_on_path():
    original = os.environ.get("PATH", "")
    os.environ["PATH"] = f"{MOCKS_DIR}{os.pathsep}{original}"
    yield MOCKS_DIR
    os.environ["PATH"] = original


@pytest.fixture(autouse=True)
def stub_caffeine_simulator(request, monkeypatch):
    """Mock the caffeine simulator by default — pipeline tests don't need
    a real JVM/gradle. Tests marked `caffeine_needed` opt out and exercise
    the real simulator."""
    if "caffeine_needed" in request.keywords:
        return
    from src.fitness.caffeine_simulator import CaffeineResult, CaffeineResults
    stub = CaffeineResults(results=[CaffeineResult(
        trace="lirs:multi1.trace.gz", size=512, policy="ClassicLRU",
        hit_rate=0.5, miss_rate=0.5, hits=500, misses=500,
        requests=1000, evictions=0,
    )])
    monkeypatch.setattr(
        "src.fitness.runner.simulate_variant",
        lambda *_a, **_kw: stub,
    )


