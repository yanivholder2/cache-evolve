"""Test collection wiring for the evolve test suite."""

import os
import sys
from pathlib import Path

import pytest

TESTS_DIR = Path(__file__).parent
MOCKS_DIR = TESTS_DIR / "_mocks"

# Allow sibling helper modules (e.g. `_stubs`) to be imported without an
# explicit package — they live alongside the test files.
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
        help="run tests marked `caffeine_needed` (require a caffeine checkout: a `caffeine/` repo sibling by default, or $CAFFEINE_DIR)",
    )


def pytest_configure(config):
    config.addinivalue_line(
        "markers", "slow: end-to-end test that drives a real cargo build"
    )
    config.addinivalue_line(
        "markers",
        "caffeine_needed: end-to-end test that requires a caffeine checkout "
        "(a `caffeine/` repo sibling by default, or $CAFFEINE_DIR) and a working "
        "JDK; invoke explicitly with --caffeine-needed, never in CI",
    )
    if config.getoption("--slow") and config.getoption("--caffeine-needed"):
        raise pytest.UsageError(
            "--slow and --caffeine-needed cannot be combined: slow tests "
            "assume the caffeine stub is active, which --caffeine-needed disables. "
            "Run each marker in a separate invocation."
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


@pytest.fixture(autouse=True)
def _reset_adapter_token_totals():
    from evolve.llm.token_usage import reset_token_totals
    reset_token_totals()
    yield
    reset_token_totals()


@pytest.fixture(autouse=True, scope="session")
def mock_cargo_on_path():
    original = os.environ.get("PATH", "")
    os.environ["PATH"] = f"{MOCKS_DIR}{os.pathsep}{original}"
    yield MOCKS_DIR
    os.environ["PATH"] = original


@pytest.fixture(autouse=True, scope="session")
def stub_caffeine_simulator(request):
    """Mock the caffeine simulator session-wide — pipeline tests don't need
    a real JVM/gradle. Session scope so the stub is live when module-scoped
    fixtures (e.g. `shared_cohort_run`) build their setup. With
    `--caffeine-needed`, tests marked `caffeine_needed` are collected and
    everything runs against real caffeine."""
    if request.config.getoption("--caffeine-needed"):
        yield
        return
    from evolve.fitness.caffeine_simulator import CaffeineResult, CaffeineResults
    stub = CaffeineResults(results=[CaffeineResult(
        trace="lirs:multi1.trace.gz", size=512, policy="ClassicLRU",
        hit_rate=0.5, miss_rate=0.5, hits=500, misses=500,
        requests=1000, evictions=0,
    )])
    mp = pytest.MonkeyPatch()
    mp.setattr("evolve.fitness.runner.simulate_variant", lambda *_a, **_kw: stub)
    yield
    mp.undo()


