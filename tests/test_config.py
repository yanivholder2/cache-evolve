"""Resolution of `authoring_scope` from config.toml + CLI override."""

from __future__ import annotations

from pathlib import Path

import pytest

from evolve.config import ConfigError, load_configs


def _write_config(root: Path, *, authoring_scope: str | None = "extension-points") -> Path:
    lines = [
        "[framework]",
        "cycles = 1",
        "cohort_size = 3",
        'ai_agent_tool = "claude-sdk"',
        'outputs_root = "outputs/run"',
        'log_level = "DEBUG"',
        "digest_cycles = 3",
        "max_workers = 0",
    ]
    if authoring_scope is not None:
        lines.append(f'authoring_scope = "{authoring_scope}"')
    lines += ["", "[simulator]", "timeout_s = 600", ""]
    (root / "config.toml").write_text("\n".join(lines), encoding="utf-8")
    return root


def test_authoring_scope_resolves_from_config(tmp_path: Path):
    _write_config(tmp_path, authoring_scope="full-autonomy")
    configs = load_configs(tmp_path)
    assert configs.framework.authoring_scope == "full-autonomy"


def test_authoring_scope_flag_overrides_config(tmp_path: Path):
    _write_config(tmp_path, authoring_scope="extension-points")
    configs = load_configs(tmp_path, authoring_scope="full-autonomy")
    assert configs.framework.authoring_scope == "full-autonomy"


def test_authoring_scope_absent_raises(tmp_path: Path):
    _write_config(tmp_path, authoring_scope=None)
    with pytest.raises(ConfigError, match="authoring_scope"):
        load_configs(tmp_path)


def test_authoring_scope_invalid_value_raises(tmp_path: Path):
    _write_config(tmp_path, authoring_scope="sideways")
    with pytest.raises(ConfigError, match="authoring-scope"):
        load_configs(tmp_path)
