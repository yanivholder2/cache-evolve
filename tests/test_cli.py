"""Unit tests for src/evolve/cli.py helpers."""

from __future__ import annotations

import argparse
import dataclasses
import json
import types
from pathlib import Path

import pytest

from evolve._io import write_json
from evolve.cli import (
    _build_arg_parser,
    _completed_cycle_ids,
    _erase_incomplete_cycles,
    _prepare_resume,
    _reject_overrides_with_continue,
    _require_scope_supported,
    _validate_active_dimensions,
)
from evolve.config import (
    AuthoringScope,
    Configs,
    FitnessConfig,
    FrameworkConfig,
    SelectionConfig,
    SimulatorConfig,
)
from evolve.identifiers import CycleId
from evolve.paths import Layout


def _stub_subject(extension_points: tuple = ()) -> types.SimpleNamespace:
    return types.SimpleNamespace(name="lru_cache", extension_points=extension_points)


def _cfg(dimensions: tuple[str, ...]) -> Configs:
    return Configs(
        framework=FrameworkConfig(
            cycles=1, cohort_size=1, ai_agent_tool="claude-sdk", model="claude-opus-4-8",
            outputs_root=Path("/tmp"), log_level="DEBUG", digest_cycles=3,
            max_workers=0, implementation_attempts=3, hypothesis_attempts=3,
            authoring_scope="full-autonomy", subject="lru_cache",
        ),
        simulator=SimulatorConfig(timeout_s=600),
        fitness=FitnessConfig(dimensions=dimensions),
        selection=SelectionConfig(method="pareto", winners_count=3, weights=None),
    )


def _dim_subject(*, bench=None, metadata=None) -> types.SimpleNamespace:
    return types.SimpleNamespace(name="lru_cache", bench=bench, metadata=metadata)


def _layout_with_baseline(tmp_path: Path, baseline_dims: list[str]) -> Layout:
    layout = Layout(repo_root=tmp_path / "repo", outputs_root=tmp_path / "out", subject="lru_cache")
    layout.sources.baseline.fitness_vector_json.parent.mkdir(parents=True, exist_ok=True)
    write_json(
        layout.sources.baseline.fitness_vector_json,
        {"variant": "baseline", "dimensions": [{"dim": d, "summary": 0.5, "higher_is_better": True} for d in baseline_dims]},
    )
    return layout


def test_validate_dimensions_happy_path(tmp_path):
    layout = _layout_with_baseline(tmp_path, ["hit-rate"])
    _validate_active_dimensions(_cfg(("hit-rate",)), _dim_subject(), layout)


def test_validate_dimensions_unknown_dimension_exits(tmp_path):
    layout = _layout_with_baseline(tmp_path, ["hit-rate"])
    with pytest.raises(SystemExit):
        _validate_active_dimensions(_cfg(("made-up",)), _dim_subject(), layout)


def test_validate_dimensions_missing_capability_exits(tmp_path):
    layout = _layout_with_baseline(tmp_path, ["hit-rate", "per-op-time"])
    # subject declares no [bench], so per-op-time cannot be activated.
    with pytest.raises(SystemExit):
        _validate_active_dimensions(_cfg(("per-op-time",)), _dim_subject(bench=None), layout)


def test_validate_dimensions_not_in_baseline_exits(tmp_path):
    layout = _layout_with_baseline(tmp_path, ["hit-rate"])
    # metadata-size is subject-capable but absent from the baseline anchor.
    with pytest.raises(SystemExit):
        _validate_active_dimensions(
            _cfg(("metadata-size",)), _dim_subject(metadata=object()), layout
        )


def test_extension_points_scope_rejects_subject_without_extension_points():
    with pytest.raises(SystemExit):
        _require_scope_supported(_stub_subject(()), AuthoringScope.EXTENSION_POINTS)


def test_extension_points_scope_allows_subject_with_extension_points():
    _require_scope_supported(
        _stub_subject(("ep",)), AuthoringScope.EXTENSION_POINTS
    )


def test_full_autonomy_scope_allows_subject_without_extension_points():
    _require_scope_supported(_stub_subject(()), AuthoringScope.FULL_AUTONOMY)


# --- resume (--continue) helpers ---


def _run_layout(tmp_path: Path) -> Layout:
    return Layout(repo_root=tmp_path / "repo", outputs_root=tmp_path / "out", subject="lru_cache")


def _write_ledger(layout: Layout, cycle_values: list[int]) -> None:
    layout.outputs.ledger.parent.mkdir(parents=True, exist_ok=True)
    lines = [json.dumps({"cycle": str(CycleId(v))}) + "\n" for v in cycle_values]
    layout.outputs.ledger.write_text("".join(lines), encoding="utf-8")


def _make_cycle_dir(layout: Layout, value: int) -> None:
    layout.outputs.cycle(CycleId(value)).variants_dir.mkdir(parents=True, exist_ok=True)
    log = layout.outputs.cycle_log(CycleId(value))
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text("log", encoding="utf-8")


def test_completed_cycle_ids_reads_ledger_rows(tmp_path):
    layout = _run_layout(tmp_path)
    _write_ledger(layout, [1, 2, 3])
    assert _completed_cycle_ids(layout.outputs.ledger) == {CycleId(1), CycleId(2), CycleId(3)}


def test_completed_cycle_ids_no_ledger_is_empty(tmp_path):
    layout = _run_layout(tmp_path)
    assert _completed_cycle_ids(layout.outputs.ledger) == set()


def test_erase_incomplete_cycles_removes_uncommitted_dir_and_log(tmp_path):
    layout = _run_layout(tmp_path)
    _write_ledger(layout, [1, 2, 3])
    for value in (1, 2, 3, 4):
        _make_cycle_dir(layout, value)

    erased = _erase_incomplete_cycles(layout, _completed_cycle_ids(layout.outputs.ledger))

    assert erased == [CycleId(4)]
    assert not layout.outputs.cycle(CycleId(4)).root.exists()
    assert not layout.outputs.cycle_log(CycleId(4)).exists()
    for value in (1, 2, 3):
        assert layout.outputs.cycle(CycleId(value)).root.is_dir()


def test_erase_incomplete_cycles_keeps_all_when_committed(tmp_path):
    layout = _run_layout(tmp_path)
    _write_ledger(layout, [1, 2, 3])
    for value in (1, 2, 3):
        _make_cycle_dir(layout, value)

    assert _erase_incomplete_cycles(layout, _completed_cycle_ids(layout.outputs.ledger)) == []


def _cfg_cycles(cycles: int) -> Configs:
    cfg = _cfg(("hit-rate",))
    return dataclasses.replace(cfg, framework=dataclasses.replace(cfg.framework, cycles=cycles))


def test_prepare_resume_returns_completed_count_and_erases_partial(tmp_path):
    layout = _run_layout(tmp_path)
    _write_ledger(layout, [1, 2, 3])
    for value in (1, 2, 3, 4):  # cycle 4 crashed mid-run
        _make_cycle_dir(layout, value)

    assert _prepare_resume(layout, _cfg_cycles(5)) == 3
    assert not layout.outputs.cycle(CycleId(4)).root.exists()


def test_prepare_resume_on_finished_run_is_noop(tmp_path):
    layout = _run_layout(tmp_path)
    _write_ledger(layout, [1, 2, 3])
    for value in (1, 2, 3):
        _make_cycle_dir(layout, value)

    assert _prepare_resume(layout, _cfg_cycles(3)) == 3
    for value in (1, 2, 3):
        assert layout.outputs.cycle(CycleId(value)).root.is_dir()


def _override_args(**given) -> argparse.Namespace:
    # Start from the real parser's defaults so new flags are covered automatically.
    args = _build_arg_parser().parse_args([])
    for key, value in given.items():
        setattr(args, key, value)
    return args


def test_reject_overrides_with_continue_allows_bare_continue():
    _reject_overrides_with_continue(_override_args())  # no raise


@pytest.mark.parametrize(
    "given",
    [{"cycles": 3}, {"subject": "dash"}, {"simulator_timeout_s": 30}],
)
def test_reject_overrides_with_continue_rejects_any_setting_flag(given):
    with pytest.raises(SystemExit):
        _reject_overrides_with_continue(_override_args(**given))


@pytest.mark.parametrize(
    "argv",
    [
        ["--digest-cycles", "2"],
        ["--implementation-attempts", "2"],
        ["--hypothesis-attempts", "2"],
        ["--dimensions", "hit-rate"],
        ["--selection-method", "pareto"],
        ["--selection-method", "weighted"],
        ["--winners-count", "2"],
        ["--weights", "hit-rate=1.0"],
    ],
)
def test_reject_overrides_with_continue_rejects_new_flags(argv, capsys):
    args = _build_arg_parser().parse_args(["--continue", "/tmp/run", *argv])
    with pytest.raises(SystemExit):
        _reject_overrides_with_continue(args)
    assert "--continue" in capsys.readouterr().err


def test_main_rejects_continue_with_new_flag(tmp_path):
    from evolve.cli import main

    with pytest.raises(SystemExit):
        main(["--continue", str(tmp_path), "--weights", "hit-rate=1.0"])


@pytest.mark.parametrize(
    ("argv", "flag"),
    [
        (["--weights", "hit-rate=0.6"], "--weights"),
        (["--selection-method", "pareto"], "--selection-method"),
        (["--simulator-timeout-s", "30"], "--simulator-timeout-s"),
        (["--dimensions", "hit-rate"], "--dimensions"),
        (["--max-workers", "2"], "--max-workers"),
    ],
)
def test_reject_overrides_with_continue_names_the_flag(argv, flag, capsys):
    args = _build_arg_parser().parse_args(["--continue", "/tmp/run", *argv])
    with pytest.raises(SystemExit):
        _reject_overrides_with_continue(args)
    given = capsys.readouterr().err.rsplit("given: ", 1)[1]
    assert flag in given
    assert "hit-rate" not in given


def test_main_rejects_duplicate_weights(capsys):
    from evolve.cli import main

    with pytest.raises(SystemExit):
        main(["--weights", "hit-rate=0.6", "hit-rate=0.9"])
    assert "--weights repeats a dimension: hit-rate" in capsys.readouterr().err
