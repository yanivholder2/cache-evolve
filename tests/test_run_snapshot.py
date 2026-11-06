"""Unit tests for src/evolve/run_snapshot.py."""

from __future__ import annotations

import argparse
import tomllib

import pytest

from evolve.cli import _build_arg_parser, _parse_weight
from evolve.config import load_configs
from evolve.paths import Layout
from evolve.run_snapshot import flag_overrides, overlay_config, snapshot_run_inputs


def _args(**overrides) -> argparse.Namespace:
    # Start from the real parser's defaults so new flags are covered automatically.
    args = _build_arg_parser().parse_args([])
    for key, value in overrides.items():
        setattr(args, key, value)
    return args


def test_flag_overrides_omits_unset_and_empty_sections():
    assert flag_overrides(_args()) == {}


def test_flag_overrides_groups_given_flags_by_section():
    args = _args(cycles=2, subject="dash", simulator_timeout_s=30)
    assert flag_overrides(args) == {
        "framework": {"subject": "dash", "cycles": 2},
        "simulator": {"timeout_s": 30},
    }


def test_snapshot_run_inputs_copies_config_and_manifest(tmp_path):
    layout = Layout(repo_root=tmp_path / "repo", outputs_root=tmp_path / "out", subject="lru_cache")
    layout.sources.config_toml.parent.mkdir(parents=True, exist_ok=True)
    layout.sources.config_toml.write_text("[framework]\ncycles = 7\n")
    layout.sources.subject_manifest.parent.mkdir(parents=True, exist_ok=True)
    layout.sources.subject_manifest.write_text("[caffeine]\n")

    snapshot_run_inputs(layout, {})

    assert layout.outputs.config_toml.read_text() == "[framework]\ncycles = 7\n"
    assert layout.outputs.manifest_toml.read_text() == "[caffeine]\n"


def test_snapshot_run_inputs_applies_flag_overrides_and_leaves_source(tmp_path):
    layout = Layout(repo_root=tmp_path / "repo", outputs_root=tmp_path / "out", subject="lru_cache")
    layout.sources.config_toml.parent.mkdir(parents=True, exist_ok=True)
    layout.sources.config_toml.write_text("[framework]\ncycles = 7\n")
    layout.sources.subject_manifest.parent.mkdir(parents=True, exist_ok=True)
    layout.sources.subject_manifest.write_text("[caffeine]\n")

    snapshot_run_inputs(layout, {"framework": {"cycles": 2}})

    assert layout.outputs.config_toml.read_text() == "[framework]\ncycles = 2\n"
    # The source config.toml is read-only: only the snapshot reflects the flag.
    assert layout.sources.config_toml.read_text() == "[framework]\ncycles = 7\n"


def test_snapshot_run_inputs_preserves_existing_snapshot(tmp_path):
    # A resumed run (--continue) already has both snapshots; they are the run's
    # identity and must not be overwritten with the current repo state.
    layout = Layout(repo_root=tmp_path / "repo", outputs_root=tmp_path / "out", subject="lru_cache")
    layout.sources.config_toml.parent.mkdir(parents=True, exist_ok=True)
    layout.sources.config_toml.write_text("[framework]\ncycles = 7\n")
    layout.sources.subject_manifest.parent.mkdir(parents=True, exist_ok=True)
    layout.sources.subject_manifest.write_text("[caffeine]\n")

    layout.outputs.root.mkdir(parents=True, exist_ok=True)
    layout.outputs.config_toml.write_text("[framework]\ncycles = 3\n")
    layout.outputs.manifest_toml.write_text("[caffeine]\nold = true\n")

    snapshot_run_inputs(layout, {"framework": {"cycles": 99}})

    assert layout.outputs.config_toml.read_text() == "[framework]\ncycles = 3\n"
    assert layout.outputs.manifest_toml.read_text() == "[caffeine]\nold = true\n"


def test_overlay_config_no_overrides_is_verbatim():
    source = "[framework]\n# keep me\ncycles = 7\n"
    assert overlay_config(source, {}) == source


def test_overlay_config_rewrites_in_place_keeping_comments():
    source = (
        "[framework]\n"
        "# subject under evolution\n"
        'subject = "lru_cache"\n'
        "cycles = 10\n"
        "[simulator]\n"
        "timeout_s = 600\n"
    )
    overrides = {
        "framework": {"subject": "dash", "cycles": 3},
        "simulator": {"timeout_s": 30},
    }
    assert overlay_config(source, overrides) == (
        "[framework]\n"
        "# subject under evolution\n"
        'subject = "dash"\n'
        "cycles = 3\n"
        "[simulator]\n"
        "timeout_s = 30\n"
    )


def test_overlay_config_ignores_commented_key():
    source = "[framework]\n# cycles = 10\n"
    assert overlay_config(source, {"framework": {"cycles": 3}}) == (
        "[framework]\n# cycles = 10\ncycles = 3\n"
    )


def test_overlay_config_appends_missing_key_to_section():
    source = "[framework]\ncycles = 10\n\n[simulator]\ntimeout_s = 600\n"
    out = overlay_config(source, {"framework": {"subject": "dash"}})
    assert out == (
        "[framework]\ncycles = 10\nsubject = \"dash\"\n\n[simulator]\ntimeout_s = 600\n"
    )


def test_overlay_config_rewrite_preserves_inline_comment():
    source = '[framework]\ncycles = 10  # how many\n'
    assert overlay_config(source, {"framework": {"cycles": 3}}) == (
        "[framework]\ncycles = 3  # how many\n"
    )


def test_overlay_config_append_to_section_without_trailing_newline():
    source = "[framework]\ncycles = 10"  # no final newline
    assert overlay_config(source, {"framework": {"subject": "dash"}}) == (
        '[framework]\ncycles = 10\nsubject = "dash"\n'
    )


def test_overlay_config_creates_missing_section():
    source = "[framework]\ncycles = 10\n"
    out = overlay_config(source, {"simulator": {"timeout_s": 30}})
    assert out == "[framework]\ncycles = 10\n[simulator]\ntimeout_s = 30\n"


# --- new flags: attempts, fitness.dimensions, selection (incl. whole-table weights) ---

_WEIGHTED_SOURCE = (
    "[framework]\n"
    'subject = "lru_cache"\n'
    "cycles = 10\n"
    "cohort_size = 3\n"
    'ai_agent_tool = "claude-sdk"\n'
    'model = "claude-opus-4-8"\n'
    'log_level = "DEBUG"\n'
    "digest_cycles = 3\n"
    "max_workers = 0\n"
    "implementation_attempts = 3\n"
    "hypothesis_attempts = 3\n"
    'authoring_scope = "full-autonomy"\n'
    "\n"
    "[simulator]\n"
    "timeout_s = 600\n"
    "\n"
    "[fitness]\n"
    "# active dimensions\n"
    'dimensions = ["hit-rate", "metadata-size", "per-op-time"]\n'
    "\n"
    "[selection]\n"
    'method = "weighted"\n'
    "winners_count = 3\n"
    "\n"
    "# weights for weighted\n"
    "[selection.weights]\n"
    "hit-rate = 0.6\n"
    "metadata-size = 0.2\n"
    "per-op-time = 0.2\n"
)


def _parse(argv: list[str]):
    return _build_arg_parser().parse_args(argv)


def _reload_snapshot(tmp_path, snapshot: str):
    """Write `snapshot` as a run dir's config.toml and resume-load it, the way
    `--continue` does. Subject manifests are stubbed under the repo root."""
    repo_root = tmp_path / "repo"
    (repo_root / "inputs" / "subjects" / "lru_cache").mkdir(parents=True)
    (repo_root / "inputs" / "subjects" / "lru_cache" / "manifest.toml").write_text("")
    run_dir = tmp_path / "run"
    run_dir.mkdir()
    (run_dir / "config.toml").write_text(snapshot)
    return load_configs(repo_root, continue_from=run_dir)


def test_flag_overrides_maps_new_flags_to_sections():
    args = _parse([
        "--digest-cycles", "5", "--implementation-attempts", "2", "--hypothesis-attempts", "4",
        "--dimensions", "hit-rate", "per-op-time",
        "--selection-method", "weighted", "--winners-count", "2",
        "--weights", "hit-rate=0.7", "per-op-time=0.3",
    ])
    assert flag_overrides(args) == {
        "framework": {"digest_cycles": 5, "implementation_attempts": 2, "hypothesis_attempts": 4},
        "fitness": {"dimensions": ["hit-rate", "per-op-time"]},
        "selection": {"method": "weighted", "winners_count": 2},
        "selection.weights": {"hit-rate": 0.7, "per-op-time": 0.3},
    }


def test_flag_overrides_pareto_without_weights_clears_weights_table():
    assert flag_overrides(_parse(["--selection-method", "pareto"])) == {
        "selection": {"method": "pareto"},
        "selection.weights": {},
    }


def test_flag_overrides_weighted_without_weights_leaves_table_alone():
    assert flag_overrides(_parse(["--selection-method", "weighted"])) == {
        "selection": {"method": "weighted"},
    }


def test_overlay_config_replaces_weights_table_wholesale(tmp_path):
    args = _parse([
        "--dimensions", "hit-rate", "per-op-time",
        "--weights", "hit-rate=0.5", "per-op-time=0.5",
    ])
    out = overlay_config(_WEIGHTED_SOURCE, flag_overrides(args))

    data = tomllib.loads(out)
    assert data["fitness"]["dimensions"] == ["hit-rate", "per-op-time"]
    assert data["selection"]["weights"] == {"hit-rate": 0.5, "per-op-time": 0.5}
    assert "# active dimensions" in out  # untouched comments survive

    configs = _reload_snapshot(tmp_path, out)
    assert configs.fitness.dimensions == ("hit-rate", "per-op-time")
    assert configs.selection.method == "weighted"
    assert configs.selection.weights == {"hit-rate": 0.5, "per-op-time": 0.5}


def test_overlay_config_pareto_removes_weights_table(tmp_path):
    args = _parse(["--dimensions", "hit-rate", "--selection-method", "pareto"])
    out = overlay_config(_WEIGHTED_SOURCE, flag_overrides(args))

    assert "[selection.weights]" not in out
    data = tomllib.loads(out)
    assert data["selection"] == {"method": "pareto", "winners_count": 3}
    assert data["fitness"]["dimensions"] == ["hit-rate"]

    configs = _reload_snapshot(tmp_path, out)
    assert configs.selection.method == "pareto"
    assert configs.selection.weights is None
    assert configs.fitness.dimensions == ("hit-rate",)


def test_overlay_config_weights_table_not_last_section(tmp_path):
    # The dropped table must stop at the next header, keeping later sections.
    source = _WEIGHTED_SOURCE.replace("[simulator]\ntimeout_s = 600\n\n", "") + "\n[simulator]\ntimeout_s = 600\n"
    args = _parse(["--dimensions", "hit-rate", "--selection-method", "pareto"])
    out = overlay_config(source, flag_overrides(args))
    data = tomllib.loads(out)
    assert data["simulator"] == {"timeout_s": 600}
    assert "weights" not in data["selection"]
    assert _reload_snapshot(tmp_path, out).simulator.timeout_s == 600


def test_overlay_config_framework_attempts_round_trip(tmp_path):
    args = _parse(["--digest-cycles", "1", "--implementation-attempts", "2", "--hypothesis-attempts", "5", "--winners-count", "4"])
    out = overlay_config(_WEIGHTED_SOURCE, flag_overrides(args))
    configs = _reload_snapshot(tmp_path, out)
    assert (configs.framework.digest_cycles, configs.framework.implementation_attempts,
            configs.framework.hypothesis_attempts) == (1, 2, 5)
    assert configs.selection.winners_count == 4
    assert configs.selection.weights == {"hit-rate": 0.6, "metadata-size": 0.2, "per-op-time": 0.2}


@pytest.mark.parametrize(
    ("raw", "expected"),
    [("hit-rate=0.6", ("hit-rate", 0.6)), ("per-op-time=2", ("per-op-time", 2.0)), ("a=1e-3", ("a", 0.001))],
)
def test_parse_weight_accepts_dim_equals_number(raw, expected):
    assert _parse_weight(raw) == expected


@pytest.mark.parametrize("raw", ["hit-rate", "=0.5", "hit-rate=", "hit-rate=abc", "hit-rate:0.5"])
def test_parse_weight_rejects_malformed(raw):
    with pytest.raises(argparse.ArgumentTypeError, match="DIM=WEIGHT"):
        _parse_weight(raw)


def test_weights_flag_malformed_exits_via_argparse():
    with pytest.raises(SystemExit):
        _parse(["--weights", "hit-rate"])


def test_overlay_config_rewrites_multiline_array(tmp_path):
    source = _WEIGHTED_SOURCE.replace(
        'dimensions = ["hit-rate", "metadata-size", "per-op-time"]',
        'dimensions = [\n  "hit-rate",  # [primary]\n  "metadata-size",\n  "per-op-time",\n]  # trailing',
    )
    assert source != _WEIGHTED_SOURCE
    args = _parse(["--dimensions", "hit-rate", "--selection-method", "pareto"])
    out = overlay_config(source, flag_overrides(args))
    assert 'dimensions = ["hit-rate"]  # trailing\n' in out
    assert tomllib.loads(out)["fitness"]["dimensions"] == ["hit-rate"]
    assert _reload_snapshot(tmp_path, out).fitness.dimensions == ("hit-rate",)


def test_overlay_config_skips_untouched_multiline_array():
    # A continuation line that looks like an assignment must not be rewritten.
    source = '[a]\nitems = [\n  "x",\n]\ncycles = 1\n'
    out = overlay_config(source, {"a": {"cycles": 2}})
    assert out == '[a]\nitems = [\n  "x",\n]\ncycles = 2\n'


_DOCUMENTED_WEIGHTS = (
    '[selection]\nmethod = "weighted"\nwinners_count = 3\n\n'
    "# Weights doc: keys MUST match.\n"
    "[selection.weights]\nhit-rate = 0.6\nper-op-time = 0.4\n\n"
    "# Simulator doc.\n"
    "[simulator]\ntimeout_s = 600\n"
)


def test_overlay_config_pareto_drops_weights_doc_keeps_next_section_doc():
    out = overlay_config(_DOCUMENTED_WEIGHTS, {"selection": {"method": "pareto"}, "selection.weights": {}})
    assert "Weights doc" not in out
    assert "[selection.weights]" not in out
    assert "# Simulator doc.\n[simulator]" in out
    assert tomllib.loads(out)["simulator"] == {"timeout_s": 600}


def test_overlay_config_weights_replaced_in_place_under_its_doc():
    out = overlay_config(_DOCUMENTED_WEIGHTS, {"selection.weights": {"hit-rate": 1.0}})
    assert "# Weights doc: keys MUST match.\n[selection.weights]\nhit-rate = 1.0\n\n# Simulator doc." in out
    assert tomllib.loads(out)["selection"]["weights"] == {"hit-rate": 1.0}


def test_evaluation_profile_flag_round_trips_through_snapshot(tmp_path):
    args = _parse(["--evaluation-profile", "msr"])
    overrides = flag_overrides(args)
    assert overrides == {"fitness": {"evaluation_profile": "msr"}}
    snapshot = overlay_config('[fitness]\ndimensions = ["hit-rate"]\n', overrides)
    assert 'evaluation_profile = "msr"' in snapshot
