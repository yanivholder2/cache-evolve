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
    selection_body: str | None = '[selection]\nmethod = "pareto"\nwinners_count = 3\n',
    fitness_body: str | None = '[fitness]\ndimensions = ["hit-rate"]\n',
) -> Path:
    lines = [
        "[framework]",
        "cycles = 1",
        "cohort_size = 3",
        'ai_agent_tool = "claude-sdk"',
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
    if fitness_body is not None:
        lines += ["", fitness_body]
    if selection_body is not None:
        lines += ["", selection_body]
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


def test_outputs_root_is_subject_prefixed_under_repo_outputs(tmp_path: Path):
    _write_config(tmp_path, subject="lru_cache")
    configs = load_configs(tmp_path)
    outputs_root = configs.framework.outputs_root
    assert outputs_root.parent == (tmp_path / "outputs").resolve()
    assert outputs_root.name.startswith("lru_cache_")


def test_hypothesis_attempts_resolves_from_config(tmp_path: Path):
    _write_config(tmp_path, hypothesis_attempts=5)
    configs = load_configs(tmp_path)
    assert configs.framework.hypothesis_attempts == 5


def test_hypothesis_attempts_absent_raises(tmp_path: Path):
    _write_config(tmp_path, hypothesis_attempts=None)
    with pytest.raises(ConfigError, match="hypothesis_attempts"):
        load_configs(tmp_path)


def test_fitness_and_selection_resolve_from_config(tmp_path: Path):
    _write_config(tmp_path)
    configs = load_configs(tmp_path)
    assert configs.fitness.dimensions == ("hit-rate",)
    assert configs.selection.method == "pareto"
    assert configs.selection.winners_count == 3
    assert configs.selection.weights is None


def test_dimensions_absent_raises(tmp_path: Path):
    _write_config(tmp_path, fitness_body=None)
    with pytest.raises(ConfigError, match="fitness.dimensions"):
        load_configs(tmp_path)


def test_dimensions_empty_raises(tmp_path: Path):
    _write_config(tmp_path, fitness_body="[fitness]\ndimensions = []\n")
    with pytest.raises(ConfigError, match="fitness.dimensions"):
        load_configs(tmp_path)


def test_dimensions_duplicate_raises(tmp_path: Path):
    _write_config(tmp_path, fitness_body='[fitness]\ndimensions = ["hit-rate", "hit-rate"]\n')
    with pytest.raises(ConfigError, match="fitness.dimensions"):
        load_configs(tmp_path)


def test_selection_method_absent_raises(tmp_path: Path):
    _write_config(tmp_path, selection_body="[selection]\nwinners_count = 3\n")
    with pytest.raises(ConfigError, match="selection.method"):
        load_configs(tmp_path)


def test_selection_method_unknown_raises(tmp_path: Path):
    _write_config(tmp_path, selection_body='[selection]\nmethod = "sideways"\nwinners_count = 3\n')
    with pytest.raises(ConfigError, match="selection.method"):
        load_configs(tmp_path)


def test_winners_count_absent_raises(tmp_path: Path):
    _write_config(tmp_path, selection_body='[selection]\nmethod = "pareto"\n')
    with pytest.raises(ConfigError, match="winners_count"):
        load_configs(tmp_path)


def test_winners_count_non_positive_raises(tmp_path: Path):
    _write_config(tmp_path, selection_body='[selection]\nmethod = "pareto"\nwinners_count = 0\n')
    with pytest.raises(ConfigError, match="winners_count"):
        load_configs(tmp_path)


def test_pareto_with_weights_raises(tmp_path: Path):
    _write_config(
        tmp_path,
        selection_body='[selection]\nmethod = "pareto"\nwinners_count = 3\n[selection.weights]\nhit-rate = 1.0\n',
    )
    with pytest.raises(ConfigError, match="weights must be absent"):
        load_configs(tmp_path)


def test_weighted_resolves_with_matching_weights(tmp_path: Path):
    _write_config(
        tmp_path,
        selection_body='[selection]\nmethod = "weighted"\nwinners_count = 2\n[selection.weights]\nhit-rate = 1.0\n',
    )
    configs = load_configs(tmp_path)
    assert configs.selection.method == "weighted"
    assert configs.selection.weights == {"hit-rate": 1.0}


def test_weighted_without_weights_raises(tmp_path: Path):
    _write_config(tmp_path, selection_body='[selection]\nmethod = "weighted"\nwinners_count = 2\n')
    with pytest.raises(ConfigError, match="weights is required"):
        load_configs(tmp_path)


def test_weighted_weight_keys_must_match_dimensions(tmp_path: Path):
    _write_config(
        tmp_path,
        fitness_body='[fitness]\ndimensions = ["hit-rate"]\n',
        selection_body='[selection]\nmethod = "weighted"\nwinners_count = 2\n[selection.weights]\nper-op-time = 1.0\n',
    )
    with pytest.raises(ConfigError, match="must exactly match"):
        load_configs(tmp_path)


def test_weighted_non_positive_weight_raises(tmp_path: Path):
    _write_config(
        tmp_path,
        selection_body='[selection]\nmethod = "weighted"\nwinners_count = 2\n[selection.weights]\nhit-rate = 0.0\n',
    )
    with pytest.raises(ConfigError, match="must be a positive number"):
        load_configs(tmp_path)
