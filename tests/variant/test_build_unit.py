"""Fast unit tests for src/variant/build.py helpers (no cargo invocation)."""

from pathlib import Path

import pytest

from evolve._io import read_utf8, write_utf8
from evolve.paths import cdylib_filename
from evolve.variant.build import _apply_overlay, _rewrite_framework_path


def _never_pinned(_rel: Path) -> bool:
    return False


@pytest.mark.parametrize(
    "platform, name, expected",
    [
        ("linux", "dash", "libdash.so"),
        ("win32", "dash", "dash.dll"),
        ("darwin", "dash", "libdash.dylib"),
        ("linux", "lru_cache", "liblru_cache.so"),
        ("win32", "associative_cache", "associative_cache.dll"),
    ],
)
def test_cdylib_filename_supported(platform, name, expected):
    assert cdylib_filename(platform, name) == expected


def test_cdylib_filename_unsupported_raises():
    with pytest.raises(RuntimeError, match="unsupported platform"):
        cdylib_filename("plan9", "dash")


def test_rewrite_framework_path_substitutes_absolute(tmp_path):
    cargo_toml = tmp_path / "Cargo.toml"
    write_utf8(
        cargo_toml,
        '[dependencies]\nframework = { path = "../../../framework-rs" }\n',
    )
    absolute = tmp_path / "framework-rs"
    _rewrite_framework_path(cargo_toml, absolute)
    contents = read_utf8(cargo_toml)
    assert f'path = "{absolute.as_posix()}"' in contents


def test_rewrite_framework_path_preserves_other_dep_keys(tmp_path):
    cargo_toml = tmp_path / "Cargo.toml"
    write_utf8(
        cargo_toml,
        '[dependencies]\nframework = { path = "old", features = ["x"] }\n',
    )
    _rewrite_framework_path(cargo_toml, Path("/abs/framework-rs"))
    contents = read_utf8(cargo_toml)
    assert 'features = ["x"]' in contents
    assert 'path = "/abs/framework-rs"' in contents


def test_rewrite_framework_path_missing_dep_raises(tmp_path):
    cargo_toml = tmp_path / "Cargo.toml"
    write_utf8(cargo_toml, '[dependencies]\nother = "1.0"\n')
    with pytest.raises(RuntimeError, match="framework"):
        _rewrite_framework_path(cargo_toml, Path("/abs/framework-rs"))


def test_apply_overlay_copies_files_at_relative_paths(tmp_path):
    overlay_dir = tmp_path / "overlay"
    workspace = tmp_path / "workspace"
    (overlay_dir / "src" / "deep").mkdir(parents=True)
    (overlay_dir / "src" / "factory.rs").write_text("// overlay")
    (overlay_dir / "src" / "deep" / "nested.rs").write_text("// nested")
    workspace.mkdir()
    (workspace / "src").mkdir()
    (workspace / "src" / "factory.rs").write_text("// baseline")  # to be overwritten

    _apply_overlay(overlay_dir, workspace, _never_pinned)

    assert (workspace / "src" / "factory.rs").read_text() == "// overlay"
    assert (workspace / "src" / "deep" / "nested.rs").read_text() == "// nested"


def test_apply_overlay_noop_when_overlay_dir_missing(tmp_path):
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    _apply_overlay(tmp_path / "absent", workspace, _never_pinned)  # must not raise
    assert list(workspace.iterdir()) == []


def test_apply_overlay_keeps_pristine_for_pinned_paths(tmp_path):
    # The structural backstop: a pinned overlay file is dropped, so the pristine
    # crate copy of the gate harness survives even if the implementer's reject
    # is bypassed. Non-pinned files still apply.
    overlay_dir = tmp_path / "overlay"
    workspace = tmp_path / "workspace"
    (overlay_dir / "tests").mkdir(parents=True)
    (overlay_dir / "tests" / "invariant_gate.rs").write_text("// gutted gate")
    (overlay_dir / "src").mkdir(parents=True)
    (overlay_dir / "src" / "cache.rs").write_text("// overlay cache")
    workspace.mkdir()
    (workspace / "tests").mkdir()
    (workspace / "tests" / "invariant_gate.rs").write_text("// pristine gate")

    _apply_overlay(
        overlay_dir, workspace, lambda rel: rel == Path("tests/invariant_gate.rs")
    )

    assert (workspace / "tests" / "invariant_gate.rs").read_text() == "// pristine gate"
    assert (workspace / "src" / "cache.rs").read_text() == "// overlay cache"
