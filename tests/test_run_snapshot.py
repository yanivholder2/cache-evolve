"""Unit tests for src/evolve/run_snapshot.py."""

from __future__ import annotations

import argparse

from evolve.paths import Layout
from evolve.run_snapshot import flag_overrides, overlay_config, snapshot_run_inputs


def _args(**overrides) -> argparse.Namespace:
    flags = dict(
        subject=None, cycles=None, cohort_size=None, ai_agent_tool=None, model=None,
        log_level=None, max_workers=None, authoring_scope=None, simulator_timeout_s=None,
    )
    flags.update(overrides)
    return argparse.Namespace(**flags)


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
