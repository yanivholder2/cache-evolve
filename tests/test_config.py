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
    ai_agent_tool: str | None = "claude-sdk",
    model: str | None = "claude-opus-4-8",
    selection_body: str | None = '[selection]\nmethod = "pareto"\nwinners_count = 3\n',
    fitness_body: str | None = '[fitness]\ndimensions = ["hit-rate"]\n',
) -> Path:
    lines = [
        "[framework]",
        "cycles = 1",
        "cohort_size = 3",
        'log_level = "DEBUG"',
        "digest_cycles = 3",
        "max_workers = 0",
        "implementation_attempts = 3",
    ]
    if ai_agent_tool is not None:
        lines.append(f'ai_agent_tool = "{ai_agent_tool}"')
    if model is not None:
        lines.append(f'model = "{model}"')
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


def test_model_resolves_from_config(tmp_path: Path):
    _write_config(tmp_path, model="claude-sonnet-4-6")
    configs = load_configs(tmp_path)
    assert configs.framework.model == "claude-sonnet-4-6"


def test_model_flag_overrides_config(tmp_path: Path):
    _write_config(tmp_path, model="claude-opus-4-8")
    configs = load_configs(tmp_path, model="claude-haiku-4-5-20251001")
    assert configs.framework.model == "claude-haiku-4-5-20251001"


def test_model_absent_raises(tmp_path: Path):
    _write_config(tmp_path, model=None)
    with pytest.raises(ConfigError, match="framework.model"):
        load_configs(tmp_path)


def test_model_not_valid_for_adapter_raises(tmp_path: Path):
    # gemini model under the claude-sdk adapter: valid string, wrong namespace.
    _write_config(tmp_path, ai_agent_tool="claude-sdk", model="gemini-3-pro-preview")
    with pytest.raises(ConfigError, match="not valid for ai_agent_tool"):
        load_configs(tmp_path)


def test_model_validated_against_chosen_adapter(tmp_path: Path):
    _write_config(tmp_path, ai_agent_tool="ollama-local", model="gemma4:e4b")
    configs = load_configs(tmp_path)
    assert configs.framework.ai_agent_tool == "ollama-local"
    assert configs.framework.model == "gemma4:e4b"


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
    with pytest.raises(ConfigError, match="must be a positive finite number"):
        load_configs(tmp_path)


# --- CLI-flag overrides for framework attempts, fitness, and selection ---

_THREE_DIMS = '[fitness]\ndimensions = ["hit-rate", "metadata-size", "per-op-time"]\n'
_WEIGHTED_THREE = (
    '[selection]\nmethod = "weighted"\nwinners_count = 3\n'
    "[selection.weights]\nhit-rate = 0.6\nmetadata-size = 0.2\nper-op-time = 0.2\n"
)


@pytest.mark.parametrize(
    "field", ["digest_cycles", "implementation_attempts", "hypothesis_attempts"]
)
def test_framework_int_flags_override_config(tmp_path: Path, field: str):
    _write_config(tmp_path)
    configs = load_configs(tmp_path, **{field: 7})
    assert getattr(configs.framework, field) == 7


@pytest.mark.parametrize(
    ("field", "flag"),
    [
        ("digest_cycles", "--digest-cycles"),
        ("implementation_attempts", "--implementation-attempts"),
        ("hypothesis_attempts", "--hypothesis-attempts"),
    ],
)
def test_framework_int_flags_reject_non_positive_naming_the_flag(tmp_path: Path, field: str, flag: str):
    _write_config(tmp_path)
    with pytest.raises(ConfigError, match=flag):
        load_configs(tmp_path, **{field: 0})


def test_bad_config_value_error_names_config_key_not_flag(tmp_path: Path):
    _write_config(tmp_path, hypothesis_attempts=0)
    with pytest.raises(ConfigError, match=r"framework\.hypothesis_attempts must be a positive"):
        load_configs(tmp_path)


def test_dimensions_flag_overrides_config(tmp_path: Path):
    _write_config(tmp_path, fitness_body=_THREE_DIMS)
    configs = load_configs(tmp_path, dimensions=["hit-rate", "per-op-time"])
    assert configs.fitness.dimensions == ("hit-rate", "per-op-time")


def test_dimensions_flag_duplicate_raises(tmp_path: Path):
    _write_config(tmp_path)
    with pytest.raises(ConfigError):
        load_configs(tmp_path, dimensions=["hit-rate", "hit-rate"])


def test_winners_count_flag_overrides_config(tmp_path: Path):
    _write_config(tmp_path)
    configs = load_configs(tmp_path, winners_count=5)
    assert configs.selection.winners_count == 5


def test_winners_count_flag_non_positive_names_flag(tmp_path: Path):
    _write_config(tmp_path)
    with pytest.raises(ConfigError, match="--winners-count"):
        load_configs(tmp_path, winners_count=0)


def test_selection_method_flag_unknown_raises(tmp_path: Path):
    _write_config(tmp_path)
    with pytest.raises(ConfigError):
        load_configs(tmp_path, selection_method="lexicographic")


def test_pareto_flag_with_single_dimension_drops_config_weights(tmp_path: Path):
    _write_config(tmp_path, fitness_body=_THREE_DIMS, selection_body=_WEIGHTED_THREE)
    configs = load_configs(tmp_path, dimensions=["hit-rate"], selection_method="pareto")
    assert configs.fitness.dimensions == ("hit-rate",)
    assert configs.selection.method == "pareto"
    assert configs.selection.weights is None
    assert configs.selection.winners_count == 3


def test_pareto_flag_with_weights_flag_raises(tmp_path: Path):
    _write_config(tmp_path, fitness_body=_THREE_DIMS, selection_body=_WEIGHTED_THREE)
    with pytest.raises(ConfigError, match="weights must be absent"):
        load_configs(tmp_path, selection_method="pareto", weights={"hit-rate": 0.6, "metadata-size": 0.2, "per-op-time": 0.2})


def test_weighted_dimensions_flag_without_weights_flag_raises(tmp_path: Path):
    # Narrowing the dimensions leaves config.toml's 3-key weights table stale.
    _write_config(tmp_path, fitness_body=_THREE_DIMS, selection_body=_WEIGHTED_THREE)
    with pytest.raises(ConfigError, match=r"--dimensions hit-rate does not match.*--weights.*--selection-method pareto"):
        load_configs(tmp_path, dimensions=["hit-rate"])


def test_weights_flag_replaces_config_table(tmp_path: Path):
    _write_config(tmp_path, fitness_body=_THREE_DIMS, selection_body=_WEIGHTED_THREE)
    configs = load_configs(tmp_path, dimensions=["hit-rate", "per-op-time"], weights={"hit-rate": 0.5, "per-op-time": 0.5})
    assert configs.selection.method == "weighted"
    assert configs.selection.weights == {"hit-rate": 0.5, "per-op-time": 0.5}


def test_weights_flag_non_positive_raises(tmp_path: Path):
    _write_config(tmp_path, fitness_body=_THREE_DIMS, selection_body=_WEIGHTED_THREE)
    with pytest.raises(ConfigError, match="must be a positive finite number"):
        load_configs(tmp_path, weights={"hit-rate": 0.0, "metadata-size": 0.5, "per-op-time": 0.5})


def test_weighted_flag_against_pareto_config_with_weights_flag(tmp_path: Path):
    _write_config(tmp_path)  # pareto, hit-rate only, no weights
    configs = load_configs(tmp_path, selection_method="weighted", weights={"hit-rate": 1.0})
    assert configs.selection.method == "weighted"
    assert configs.selection.weights == {"hit-rate": 1.0}


def test_weighted_flag_against_pareto_config_without_weights_raises(tmp_path: Path):
    _write_config(tmp_path)
    with pytest.raises(ConfigError, match="weights is required"):
        load_configs(tmp_path, selection_method="weighted")


# --- model namespace (claude-sdk is open: any `claude-*` id) ---

def test_claude_sdk_accepts_new_claude_model(tmp_path: Path):
    _write_config(tmp_path, ai_agent_tool="claude-sdk", model="claude-opus-4-8")
    configs = load_configs(tmp_path, model="claude-opus-5-5")
    assert configs.framework.model == "claude-opus-5-5"


def test_claude_sdk_rejects_gemini_model_with_reason(tmp_path: Path):
    _write_config(tmp_path, ai_agent_tool="claude-sdk")
    with pytest.raises(ConfigError, match=r"not valid for ai_agent_tool 'claude-sdk'.*claude-"):
        load_configs(tmp_path, model="gemini-3-pro-preview")


def test_enum_adapter_still_rejects_unknown_model(tmp_path: Path):
    _write_config(tmp_path, ai_agent_tool="ollama-local", model="claude-opus-5-5")
    with pytest.raises(ConfigError, match="not valid for ai_agent_tool 'ollama-local'"):
        load_configs(tmp_path)


@pytest.mark.parametrize("bad", [float("nan"), float("inf")])
def test_weights_flag_non_finite_raises(tmp_path: Path, bad: float):
    _write_config(tmp_path, fitness_body=_THREE_DIMS, selection_body=_WEIGHTED_THREE)
    with pytest.raises(ConfigError, match="--weights hit-rate must be a positive finite number"):
        load_configs(tmp_path, weights={"hit-rate": bad, "metadata-size": 0.5, "per-op-time": 0.5})


def test_dimensions_flag_error_names_flag(tmp_path: Path):
    _write_config(tmp_path, fitness_body=_THREE_DIMS, selection_body=_WEIGHTED_THREE)
    with pytest.raises(ConfigError, match="^--dimensions must not repeat"):
        load_configs(tmp_path, dimensions=["hit-rate", "hit-rate"])


def test_config_weights_error_names_config_key(tmp_path: Path):
    _write_config(
        tmp_path,
        selection_body='[selection]\nmethod = "weighted"\nwinners_count = 2\n[selection.weights]\nhit-rate = nan\n',
    )
    with pytest.raises(ConfigError, match="^selection.weights.hit-rate must be a positive finite number"):
        load_configs(tmp_path)


def test_evaluation_profile_defaults_to_none(tmp_path: Path):
    cfg = load_configs(_write_config(tmp_path))
    assert cfg.fitness.evaluation_profile is None


def test_evaluation_profile_resolves_from_config(tmp_path: Path):
    root = _write_config(
        tmp_path, fitness_body='[fitness]\ndimensions = ["hit-rate"]\nevaluation_profile = "msr"\n'
    )
    assert load_configs(root).fitness.evaluation_profile == "msr"


def test_evaluation_profile_flag_overrides_config(tmp_path: Path):
    root = _write_config(
        tmp_path, fitness_body='[fitness]\ndimensions = ["hit-rate"]\nevaluation_profile = "msr"\n'
    )
    assert load_configs(root, evaluation_profile="other").fitness.evaluation_profile == "other"


def test_evaluation_profile_empty_raises(tmp_path: Path):
    with pytest.raises(ConfigError, match=r"--evaluation-profile.*non-empty"):
        load_configs(_write_config(tmp_path), evaluation_profile="")
