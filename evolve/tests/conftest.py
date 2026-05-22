"""Shared fixtures for evolve tests.

Two autouse fixtures:
- session-scoped: prepend `tests/_mocks/` to PATH so the shim cargo runs
  instead of the real toolchain. The shim handles the variant-build
  invocations from build_variant / run_gates in milliseconds (by reading
  markers in the variant source) and transparently delegates any other
  cargo call to the real binary — so fitness plugins, dash-bench, etc.
  still see a working cargo.

Import setup: tests import `src.*` and the `evolve` script via importlib
registration in `sys.modules` rather than dropping `evolve/` itself on
`sys.path` — that would also expose sibling dirs (`bench`, `subject`,
`harness`, `traces`) as top-level imports, with shadow risk against
installed packages of the same name.

Slow-test gating: tests marked `@pytest.mark.slow` (full run_cycle paths
that drive real cargo builds of fitness binaries) are skipped unless
`--slow` is passed.
"""

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
    """Load `file` as `name` and stash it in `sys.modules`.

    Avoids adding `evolve/` to `sys.path`, which would expose every
    sibling dir as a top-level import.
    """
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


# `src` and `evolve` must be importable at collection time — fixtures
# fire too late, pytest imports test modules first.
_register_module("src", EVOLVE_DIR / "src" / "__init__.py", is_package=True)
_register_module("evolve", SCRIPTS_DIR / "evolve.py")

# Test helpers (`_stubs`) live alongside the tests; tests import them
# bare. Putting TESTS_DIR on sys.path is local and doesn't leak.
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


