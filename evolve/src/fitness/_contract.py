"""Plugin JSON contract validator."""

import json
import math

REQUIRED_FIELDS = ("dim", "values", "summary", "higher_is_better")


class ContractError(ValueError):
    """Raised when a plugin record violates the contract."""


def validate_plugin_dict(data: dict, *, source: str) -> dict:
    """Validates a parsed plugin record and returns it. Required fields: dim, values, summary, higher_is_better."""

    if not isinstance(data, dict):
        raise ContractError(f"{source}: expected JSON object, got {type(data).__name__}")
    missing = [f for f in REQUIRED_FIELDS if f not in data]
    if missing:
        raise ContractError(f"{source}: missing required fields {missing}")
    if not isinstance(data["dim"], str) or not data["dim"]:
        raise ContractError(f"{source}: 'dim' must be a non-empty string")
    if not isinstance(data["higher_is_better"], bool):
        raise ContractError(f"{source}: 'higher_is_better' must be bool")
    summary = data["summary"]
    if not isinstance(summary, (int, float)) or isinstance(summary, bool):
        raise ContractError(f"{source}: 'summary' must be a number, got {type(summary).__name__}")
    if not math.isfinite(float(summary)):
        raise ContractError(f"{source}: 'summary' must be finite, got {summary!r}")
    if not isinstance(data["values"], list):
        raise ContractError(f"{source}: 'values' must be a list")
    return data


def parse_plugin_stdout(stdout: str, *, source: str) -> dict:
    """Parses stdout as JSON and validates against the plugin contract."""

    try:
        data = json.loads(stdout)
    except json.JSONDecodeError as e:
        raise ContractError(f"{source}: invalid JSON ({e}); stdout starts with: {stdout[:120]!r}") from e
    return validate_plugin_dict(data, source=source)
