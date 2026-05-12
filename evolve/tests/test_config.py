"""AC #2: config.toml parses and contains all required v1 knobs."""

import tomllib
from pathlib import Path

CONFIG = Path(__file__).parent.parent / "config.toml"


def test_config_has_required_v1_knobs():
    with open(CONFIG, "rb") as f:
        cfg = tomllib.load(f)
    framework = cfg["framework"]
    for key in ("action_space", "profile", "cohort_size", "retry_budget"):
        assert key in framework, f"missing key: {key}"
