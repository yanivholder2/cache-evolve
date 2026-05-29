"""Resolved configuration for one evolve invocation.

Built once at the boundary (CLI + config.toml) and threaded through every
downstream callable. Sinks must read settings via `Configs` — never from
`config.toml` directly.
"""

from __future__ import annotations

import tomllib
from dataclasses import dataclass
from datetime import datetime
from enum import StrEnum
from functools import lru_cache
from pathlib import Path

from evolve._log import LOG_LEVELS
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
    outputs_root: Path
    log_level: str
    digest_cycles: int
    max_workers: int
    implementation_attempts: int
    authoring_scope: AuthoringScope
    subject: str


@dataclass(frozen=True)
class SimulatorConfig:
    timeout_s: int


@dataclass(frozen=True)
class Configs:
    framework: FrameworkConfig
    simulator: SimulatorConfig


def load_configs(
    repo_root: Path,
    *,
    cycles: int | None = None,
    cohort_size: int | None = None,
    ai_agent_tool: str | None = None,
    outputs_root: Path | None = None,
    log_level: str | None = None,
    simulator_timeout_s: int | None = None,
    max_workers: int | None = None,
    authoring_scope: AuthoringScope | str | None = None,
    subject: str | None = None,
) -> Configs:
    """Build `Configs` from `<repo_root>/config.toml`, with optional overrides
    taking precedence per field. Raises `ConfigError` for missing or invalid
    values.
    """

    cfg_path = SourcesLayout(root=repo_root).config_toml
    if not cfg_path.is_file():
        raise ConfigError(f"config.toml not found at {cfg_path}")
    with open(cfg_path, "rb") as f:
        data = tomllib.load(f)
    framework_defaults = data.get("framework", {})
    simulator_defaults = data.get("simulator", {})

    framework = FrameworkConfig(
        cycles=_resolve_positive_int(cycles, framework_defaults, "cycles", "--cycles", "framework"),
        cohort_size=_resolve_positive_int(cohort_size, framework_defaults, "cohort_size", "--cohort-size", "framework"),
        ai_agent_tool=_resolve_ai_agent_tool(ai_agent_tool, framework_defaults),
        outputs_root=_resolve_outputs_root(outputs_root, framework_defaults, repo_root),
        log_level=_resolve_log_level(log_level, framework_defaults),
        digest_cycles=_resolve_positive_int(None, framework_defaults, "digest_cycles", None, "framework"),
        max_workers=_resolve_non_negative_int(max_workers, framework_defaults, "max_workers", "--max-workers", "framework"),
        implementation_attempts=_resolve_positive_int(None, framework_defaults, "implementation_attempts", None, "framework"),
        authoring_scope=_resolve_authoring_scope(authoring_scope, framework_defaults),
        subject=_resolve_subject(subject, framework_defaults, repo_root),
    )
    simulator = SimulatorConfig(
        timeout_s=_resolve_positive_int(simulator_timeout_s, simulator_defaults, "timeout_s", "--simulator-timeout-s", "simulator"),
    )
    return Configs(framework=framework, simulator=simulator)


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
        source = flag if flag else f"{section}.{key}"
        raise ConfigError(f"{source} must be a positive integer, got: {value}")
    return value


def _resolve_non_negative_int(override: int | None, defaults: dict, key: str, flag: str | None, section: str) -> int:
    value = int(_resolve(override, defaults, key, flag, section))
    if value < 0:
        source = flag if flag else f"{section}.{key}"
        raise ConfigError(f"{source} must be a non-negative integer, got: {value}")
    return value


def _resolve_ai_agent_tool(override: str | None, defaults: dict) -> str:
    value = str(_resolve(override, defaults, "ai_agent_tool", "--ai-agent-tool", "framework"))
    if value not in ADAPTER_REGISTRY:
        raise ConfigError(
            f"--ai-agent-tool must be one of {sorted(ADAPTER_REGISTRY)}, got: {value!r}"
        )
    return value


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


def _resolve_outputs_root(override: Path | None, defaults: dict, repo_root: Path) -> Path:
    if override is not None:
        raw = str(override)
    else:
        raw = str(_resolve(None, defaults, "outputs_root", "--outputs-root", "framework"))
        raw = raw.format(timestamp=_timestamp())
    path = Path(raw)
    if not path.is_absolute():
        path = repo_root / path
    return path.resolve()


@lru_cache(maxsize=1)
def _timestamp() -> str:
    now = datetime.now()
    return now.strftime("%Y-%m-%d_%H-%M-%S-") + f"{now.microsecond // 1000:03d}"
