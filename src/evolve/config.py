"""Resolved configuration for one evolve invocation.

Built once at the boundary (CLI + config.toml) and threaded through every
downstream callable. Sinks must read settings via `Configs` — never from
`config.toml` directly.
"""

from __future__ import annotations

import math
import tomllib
from dataclasses import dataclass
from datetime import datetime
from enum import StrEnum
from pathlib import Path

from evolve._log import LOG_LEVELS
from evolve.aggregate.selectors import SelectorConfigError, build_selector
from evolve.llm import ADAPTER_REGISTRY
from evolve.paths import SourcesLayout


class ConfigError(ValueError):
    """Raised when config.toml + overrides cannot produce a valid `Configs`."""


class AuthoringScope(StrEnum):
    """Write surface the variant-implementer agent may author."""

    EXTENSION_POINTS = "extension-points"
    FULL_AUTONOMY = "full-autonomy"


@dataclass(frozen=True)
class FrameworkConfig:
    cycles: int
    cohort_size: int
    ai_agent_tool: str
    model: str
    outputs_root: Path
    log_level: str
    digest_cycles: int
    max_workers: int
    implementation_attempts: int
    hypothesis_attempts: int
    authoring_scope: AuthoringScope
    subject: str


@dataclass(frozen=True)
class SimulatorConfig:
    timeout_s: int


@dataclass(frozen=True)
class FitnessConfig:
    """Which Fitness dimensions are active this run.

    Activation only — capability (which dimensions a subject *can* measure) lives
    in the Subject manifest. `dimensions` must be a subset of the subject's
    capable dimensions; the cli boundary cross-checks that against the manifest.

    `evaluation_profile` names an `[evaluation.profiles.<name>]` grid in the
    subject manifest to score on instead of its default `[evaluation]` grid;
    None (the default) keeps the default grid and baseline. Existence is checked
    against the manifest at the cli boundary.
    """

    dimensions: tuple[str, ...]
    evaluation_profile: str | None = None


@dataclass(frozen=True)
class SelectionConfig:
    """How Winners are chosen from the active fitness dimensions.

    Pure data: `method` names a key in `SELECTOR_REGISTRY`, `winners_count` is
    the shared top-k both selection stages clamp to the population, and `weights`
    is the per-dimension weighting for `weighted` (absent under `pareto`). The
    selector itself is built from this at the boundary via `build_selector`.
    """

    method: str
    winners_count: int
    weights: dict[str, float] | None


@dataclass(frozen=True)
class Configs:
    framework: FrameworkConfig
    simulator: SimulatorConfig
    fitness: FitnessConfig
    selection: SelectionConfig


def load_configs(
    repo_root: Path,
    *,
    continue_from: Path | None = None,
    cycles: int | None = None,
    cohort_size: int | None = None,
    ai_agent_tool: str | None = None,
    model: str | None = None,
    log_level: str | None = None,
    simulator_timeout_s: int | None = None,
    max_workers: int | None = None,
    authoring_scope: AuthoringScope | str | None = None,
    subject: str | None = None,
    digest_cycles: int | None = None,
    implementation_attempts: int | None = None,
    hypothesis_attempts: int | None = None,
    dimensions: list[str] | tuple[str, ...] | None = None,
    evaluation_profile: str | None = None,
    selection_method: str | None = None,
    winners_count: int | None = None,
    weights: dict[str, float] | None = None,
) -> Configs:
    """Build `Configs` from `<repo_root>/config.toml`, with optional overrides
    taking precedence per field. Raises `ConfigError` for missing or invalid
    values.

    When `continue_from` is set (resuming a crashed run), config is read from
    that run's snapshot (`<continue_from>/config.toml`) instead of the repo
    config, and `outputs_root` is the run directory itself rather than a fresh
    timestamped one. The resumed run reuses the snapshot verbatim — the caller
    rejects per-field overrides so the continuation stays faithful to the
    original.
    """

    if continue_from is not None:
        cfg_path = continue_from / "config.toml"
        if not cfg_path.is_file():
            raise ConfigError(
                f"--continue: run config snapshot not found at {cfg_path}; "
                "expected an evolve outputs directory from a prior run"
            )
    else:
        cfg_path = SourcesLayout(root=repo_root).config_toml
        if not cfg_path.is_file():
            raise ConfigError(f"config.toml not found at {cfg_path}")
    with open(cfg_path, "rb") as f:
        data = tomllib.load(f)
    framework_defaults = data.get("framework", {})
    simulator_defaults = data.get("simulator", {})
    fitness_defaults = data.get("fitness", {})
    selection_defaults = data.get("selection", {})

    resolved_subject = _resolve_subject(subject, framework_defaults, repo_root)
    resolved_ai_agent_tool = _resolve_ai_agent_tool(ai_agent_tool, framework_defaults)
    outputs_root = (
        continue_from.resolve()
        if continue_from is not None
        else (repo_root / "outputs" / f"{resolved_subject}_{_timestamp()}").resolve()
    )
    framework = FrameworkConfig(
        cycles=_resolve_positive_int(cycles, framework_defaults, "cycles", "--cycles", "framework"),
        cohort_size=_resolve_positive_int(cohort_size, framework_defaults, "cohort_size", "--cohort-size", "framework"),
        ai_agent_tool=resolved_ai_agent_tool,
        model=_resolve_model(model, framework_defaults, resolved_ai_agent_tool),
        outputs_root=outputs_root,
        log_level=_resolve_log_level(log_level, framework_defaults),
        digest_cycles=_resolve_positive_int(digest_cycles, framework_defaults, "digest_cycles", "--digest-cycles", "framework"),
        max_workers=_resolve_non_negative_int(max_workers, framework_defaults, "max_workers", "--max-workers", "framework"),
        implementation_attempts=_resolve_positive_int(
            implementation_attempts, framework_defaults, "implementation_attempts", "--implementation-attempts", "framework"
        ),
        hypothesis_attempts=_resolve_positive_int(
            hypothesis_attempts, framework_defaults, "hypothesis_attempts", "--hypothesis-attempts", "framework"
        ),
        authoring_scope=_resolve_authoring_scope(authoring_scope, framework_defaults),
        subject=resolved_subject,
    )
    simulator = SimulatorConfig(
        timeout_s=_resolve_positive_int(simulator_timeout_s, simulator_defaults, "timeout_s", "--simulator-timeout-s", "simulator"),
    )
    fitness = FitnessConfig(
        dimensions=_resolve_dimensions(dimensions, fitness_defaults),
        evaluation_profile=_resolve_evaluation_profile(evaluation_profile, fitness_defaults),
    )
    selection = _resolve_selection(
        selection_defaults,
        fitness.dimensions,
        dimensions_from_flag=dimensions is not None,
        method=selection_method,
        winners_count=winners_count,
        weights=weights,
    )
    return Configs(framework=framework, simulator=simulator, fitness=fitness, selection=selection)


def _resolve(override, defaults: dict, key: str, flag: str | None, section: str):
    if override is not None:
        return override
    if key in defaults:
        return defaults[key]
    hint = (
        f"pass {flag} or set {section}.{key} in config.toml"
        if flag
        else f"set {section}.{key} in config.toml"
    )
    raise ConfigError(f"{section}.{key} not set ({hint})")


def _resolve_positive_int(override: int | None, defaults: dict, key: str, flag: str | None, section: str) -> int:
    value = int(_resolve(override, defaults, key, flag, section))
    if value < 1:
        source = flag if flag and override is not None else f"{section}.{key}"
        raise ConfigError(f"{source} must be a positive integer, got: {value}")
    return value


def _resolve_non_negative_int(override: int | None, defaults: dict, key: str, flag: str | None, section: str) -> int:
    value = int(_resolve(override, defaults, key, flag, section))
    if value < 0:
        source = flag if flag and override is not None else f"{section}.{key}"
        raise ConfigError(f"{source} must be a non-negative integer, got: {value}")
    return value


def _resolve_ai_agent_tool(override: str | None, defaults: dict) -> str:
    value = str(_resolve(override, defaults, "ai_agent_tool", "--ai-agent-tool", "framework"))
    if value not in ADAPTER_REGISTRY:
        raise ConfigError(
            f"--ai-agent-tool must be one of {sorted(ADAPTER_REGISTRY)}, got: {value!r}"
        )
    return value


def _resolve_model(override: str | None, defaults: dict, ai_agent_tool: str) -> str:
    """Resolve `framework.model` and reject any model the chosen adapter can't drive.

    The model namespace is per-adapter (`LLMAdapter.validate_model`), so what is
    valid depends on `ai_agent_tool`. Switching tools without switching model
    dies here rather than mid-run.
    """
    value = str(_resolve(override, defaults, "model", "--model", "framework"))
    try:
        ADAPTER_REGISTRY[ai_agent_tool].validate_model(value)
    except ValueError as exception:
        raise ConfigError(
            f"framework.model {value!r} is not valid for ai_agent_tool {ai_agent_tool!r}: {exception}"
        ) from exception
    return value


def _resolve_dimensions(override: list[str] | tuple[str, ...] | None, defaults: dict) -> tuple[str, ...]:
    value = _resolve(
        list(override) if override is not None else None, defaults, "dimensions", "--dimensions", "fitness"
    )
    source = "--dimensions" if override is not None else "fitness.dimensions"
    if not isinstance(value, list) or not value:
        raise ConfigError(f"{source} must be a non-empty list of strings")
    if any(not isinstance(dim, str) or not dim for dim in value):
        raise ConfigError(f"{source} must be a list of non-empty strings, got: {value!r}")
    if len(set(value)) != len(value):
        raise ConfigError(f"{source} must not repeat a dimension, got: {value!r}")
    return tuple(value)


def _resolve_evaluation_profile(override: str | None, defaults: dict) -> str | None:
    """Optional: absent from both flag and config.toml means the default grid."""
    value = override if override is not None else defaults.get("evaluation_profile")
    if value is None:
        return None
    if not isinstance(value, str) or not value:
        source = "--evaluation-profile" if override is not None else "fitness.evaluation_profile"
        raise ConfigError(f"{source} must be a non-empty string, got: {value!r}")
    return value


def _resolve_selection(
    defaults: dict,
    dimensions: tuple[str, ...],
    *,
    dimensions_from_flag: bool = False,
    method: str | None = None,
    winners_count: int | None = None,
    weights: dict[str, float] | None = None,
) -> SelectionConfig:
    resolved_method = str(_resolve(method, defaults, "method", "--selection-method", "selection"))
    resolved_winners_count = _resolve_positive_int(
        winners_count, defaults, "winners_count", "--winners-count", "selection"
    )
    if weights is not None:
        resolved_weights = _validate_weights(weights, from_flag=True)
    elif method == "pareto":
        # Pareto takes no weights. When the flag switches to it, drop config.toml's
        # table rather than reject it, so `--selection-method pareto` works against
        # a weighted config.toml. (A pareto config.toml with weights still errors.)
        resolved_weights = None
    else:
        resolved_weights = _resolve_weights(defaults)
        if (
            dimensions_from_flag
            and resolved_method == "weighted"
            and resolved_weights is not None
            and set(resolved_weights) != set(dimensions)
        ):
            # config.toml's weights are keyed to its own dimensions; a narrower
            # or different `--dimensions` set needs its own weights (or pareto).
            raise ConfigError(
                f"--dimensions {' '.join(dimensions)} does not match config.toml's "
                f"selection.weights keys {sorted(resolved_weights)}; pass matching "
                "--weights DIM=W ... or --selection-method pareto"
            )
    # Build the selector now so any method-specific inconsistency (weights present
    # under pareto, weight keys not matching the active dimensions, unknown method)
    # dies at config load rather than mid-run. The instance is discarded; the
    # boundary rebuilds it from this pure-data SelectionConfig.
    try:
        build_selector(
            method=resolved_method,
            winners_count=resolved_winners_count,
            dimensions=dimensions,
            weights=resolved_weights,
        )
    except SelectorConfigError as exception:
        raise ConfigError(str(exception)) from exception
    return SelectionConfig(method=resolved_method, winners_count=resolved_winners_count, weights=resolved_weights)


def _resolve_weights(defaults: dict) -> dict[str, float] | None:
    if "weights" not in defaults:
        return None
    return _validate_weights(defaults["weights"])


def _validate_weights(raw: object, *, from_flag: bool = False) -> dict[str, float]:
    table = "--weights" if from_flag else "selection.weights"
    if not isinstance(raw, dict) or not raw:
        raise ConfigError(f"{table} must be a non-empty table of dimension weights")
    weights: dict[str, float] = {}
    for key, value in raw.items():
        source = f"--weights {key}" if from_flag else f"selection.weights.{key}"
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            raise ConfigError(f"{source} must be a number, got: {value!r}")
        if not math.isfinite(value) or value <= 0:
            # Keys must match dimensions exactly, so a dimension can't be dropped by
            # omission; a zero weight would silently neutralize a still-measured
            # dimension and a negative one would invert it. NaN / inf would poison
            # every weighted score. Require positive and finite.
            raise ConfigError(f"{source} must be a positive finite number, got: {value!r}")
        weights[key] = float(value)
    return weights


def _resolve_authoring_scope(override: AuthoringScope | str | None, defaults: dict) -> AuthoringScope:
    value = str(_resolve(override, defaults, "authoring_scope", "--authoring-scope", "framework"))
    if value not in {scope.value for scope in AuthoringScope}:
        raise ConfigError(
            f"--authoring-scope must be one of {[scope.value for scope in AuthoringScope]}, got: {value!r}"
        )
    return AuthoringScope(value)


def _resolve_subject(override: str | None, defaults: dict, repo_root: Path) -> str:
    value = str(_resolve(override, defaults, "subject", "--subject", "framework"))
    if not value:
        raise ConfigError("--subject must be a non-empty string")
    subjects_dir = repo_root / "inputs" / "subjects"
    available = (
        sorted(p.name for p in subjects_dir.iterdir() if (p / "manifest.toml").is_file())
        if subjects_dir.is_dir()
        else []
    )
    if value not in available:
        raise ConfigError(
            f"--subject {value!r} not found under {subjects_dir}; available subjects: {available}"
        )
    return value


def _resolve_log_level(override: str | None, defaults: dict) -> str:
    value = str(_resolve(override, defaults, "log_level", "--log-level", "framework")).upper()
    if value not in LOG_LEVELS:
        raise ConfigError(f"--log-level must be one of {LOG_LEVELS}, got: {value!r}")
    return value


def _timestamp() -> str:
    now = datetime.now()
    return now.strftime("%Y-%m-%d_%H-%M-%S-") + f"{now.microsecond // 1000:03d}"
