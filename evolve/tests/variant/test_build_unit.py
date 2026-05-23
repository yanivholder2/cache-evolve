"""Fast unit tests for src/variant/build.py helpers (no cargo invocation)."""

from pathlib import Path

import pytest

from src.variant.build import LIB_RS_CDYLIB, _cargo_toml_cdylib, _cargo_toml_gates
from src.paths import cdylib_filename


REPO_ROOT = Path("/some/repo/root")


def test_cargo_toml_cdylib_shape():
    toml = _cargo_toml_cdylib(REPO_ROOT)
    assert 'crate-type = ["cdylib", "rlib"]' in toml
    assert 'name = "dash_variant"' in toml
    assert 'jni = "0.20.0"' in toml
    assert 'log = "0.4"' in toml
    # Production cdylib must NOT pull test-utils.
    assert "test-utils" not in toml


def test_cargo_toml_gates_shape():
    toml = _cargo_toml_gates(REPO_ROOT)
    assert "[lib]" not in toml
    assert "jni =" not in toml
    assert "test-utils" in toml


def test_cargo_toml_uses_posix_path_separators():
    # Cargo accepts forward slashes on Windows; this avoids TOML treating
    # backslashes as escape sequences.
    toml = _cargo_toml_cdylib(Path("/a/b/c"))
    assert 'path = "/a/b/c"' in toml


def test_lib_rs_cdylib_shape():
    assert "pub mod jni;" in LIB_RS_CDYLIB
    assert "pub use variant::VariantSegment;" in LIB_RS_CDYLIB


@pytest.mark.parametrize(
    "platform, expected",
    [
        ("linux", "libdash_variant.so"),
        ("win32", "dash_variant.dll"),
        ("darwin", "libdash_variant.dylib"),
    ],
)
def test_cdylib_filename_supported(platform, expected):
    assert cdylib_filename(platform) == expected


def test_cdylib_filename_unsupported_raises():
    with pytest.raises(RuntimeError, match="unsupported platform"):
        cdylib_filename("plan9")
