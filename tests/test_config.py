"""Resolution of `authoring_scope` from config.toml + CLI override."""

from __future__ import annotations

from pathlib import Path

import pytest

from evolve.config import ConfigError, load_configs


def _write_config(
    root: Path,
    *,
    authoring_scope: str | None = "extension-points",
    subject: str | None = "dash",
    hypothesis_attempts: int | None = 3,
) -> Path:
    lines = [
        "[framework]",
        "cycles = 1",
        "cohort_size = 3",
        'ai_agent_tool = "claude-sdk"',
        'outputs_root = "outputs/run"',
        'log_level = "DEBUG"',
        "digest_cycles = 3",
        "max_workers = 0",
        "implementation_attempts = 3",
    ]
    if hypothesis_attempts is not None:
        lines.append(f"hypothesis_attempts = {hypothesis_attempts}")
    if authoring_scope is not None:
        lines.append(f'authoring_scope = "{authoring_scope}"')
    if subject is not None:
        lines.append(f'subject = "{subject}"')
    lines += ["", "[simulator]", "timeout_s = 600", ""]
    (root / "config.toml").write_text("\n".join(lines), encoding="utf-8")
    # Subject resolution validates against discoverable subject manifests, so
    # stub out the subjects this suite references.
    for name in ("dash", "associative_cache", "lru_cache"):
        subject_dir = root / "inputs" / "subjects" / name
        subject_dir.mkdir(parents=True, exist_ok=True)
        (subject_dir / "manifest.toml").write_text("", encoding="utf-8")
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


def test_subject_resolves_from_config(tmp_path: Path):
    _write_config(tmp_path, subject="associative_cache")
    configs = load_configs(tmp_path)
    assert configs.framework.subject == "associative_cache"


def test_subject_flag_overrides_config(tmp_path: Path):
    _write_config(tmp_path, subject="dash")
    configs = load_configs(tmp_path, subject="lru_cache")
    assert configs.framework.subject == "lru_cache"


def test_subject_absent_raises(tmp_path: Path):
    _write_config(tmp_path, subject=None)
    with pytest.raises(ConfigError, match="subject"):
        load_configs(tmp_path)


def test_hypothesis_attempts_resolves_from_config(tmp_path: Path):
    _write_config(tmp_path, hypothesis_attempts=5)
    configs = load_configs(tmp_path)
    assert configs.framework.hypothesis_attempts == 5


def test_hypothesis_attempts_absent_raises(tmp_path: Path):
    _write_config(tmp_path, hypothesis_attempts=None)
    with pytest.raises(ConfigError, match="hypothesis_attempts"):
        load_configs(tmp_path)
