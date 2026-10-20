"""Model parsing from paper-run dir names in plots/build_data.py."""

import importlib.util
from pathlib import Path

import pytest

BUILD_DATA_PATH = Path(__file__).resolve().parent.parent / "plots" / "build_data.py"


def load_build_data():
    spec = importlib.util.spec_from_file_location("plots_build_data", BUILD_DATA_PATH)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


build_data = load_build_data()


@pytest.mark.parametrize(
    ("run_name", "subject", "expected"),
    [
        ("dash_claude-opus-4-8_2026-06-06_02-45-01-395", "dash", "claude-opus-4-8"),
        (
            "lru_cache_claude-haiku-4-5-20251001_2026-06-05_15-05-10-920",
            "lru_cache",
            "claude-haiku-4-5-20251001",
        ),
        (
            "associative_cache_claude-sonnet-5-5_2026-10-19_01-02-03-004",
            "associative_cache",
            "claude-sonnet-5-5",
        ),
        # Pre-model run names carry no model segment.
        ("associative_cache_2026-06-04_01-27-45-376", "associative_cache", None),
        # Wrong subject prefix or missing timestamp do not parse.
        ("dash_claude-opus-4-8_2026-06-06_02-45-01-395", "lru_cache", None),
        ("dash_claude-opus-4-8", "dash", None),
    ],
)
def test_model_from_run_name(run_name, subject, expected):
    assert build_data.model_from_run_name(run_name, subject) == expected


def test_run_setup_prefers_dir_name_over_snapshot_model(tmp_path):
    run_dir = tmp_path / "dash_claude-opus-4-8_2026-06-05_18-13-17-700"
    run_dir.mkdir()
    (run_dir / "cycles-ledger.jsonl").write_text('{"cycle": 1, "cohort_size": 3}\n')
    (run_dir / "config.toml").write_text(
        '[framework]\nai_agent_tool = "claude-sdk"\nmodel = "claude-sonnet-4-6"\n'
    )
    setup = build_data.run_setup(run_dir, "dash", 1)
    assert setup["model"] == "claude-opus-4-8"


def test_run_setup_falls_back_to_snapshot_model(tmp_path):
    run_dir = tmp_path / "dash_2026-06-05_18-13-17-700"
    run_dir.mkdir()
    (run_dir / "cycles-ledger.jsonl").write_text('{"cycle": 1, "cohort_size": 3}\n')
    (run_dir / "config.toml").write_text('[framework]\nmodel = "claude-haiku-4-5"\n')
    setup = build_data.run_setup(run_dir, "dash", 1)
    assert setup["model"] == "claude-haiku-4-5"


def test_groups_span_batches(tmp_path, monkeypatch):
    # Every batch sits at paper_outputs/v<N>_runs/<group>/, ordered by N; a
    # batch's plan doc is ignored. Same-named groups merge across batches.
    for rel in (
        "v1_runs/full_fitness_vector/dash/dash_claude-opus-4-8_2026-06-06_02-45-01-395",
        "v1_runs/only_hit_rate/dash/dash_claude-opus-4-8_2026-06-05_18-13-17-700",
        "v2_runs/full_fitness_vector/dash/dash_claude-opus-5-5_2026-10-20_01-00-00-000",
        "v10_runs/full_fitness_vector/dash/dash_claude-opus-9-9_2027-01-01_01-00-00-000",
    ):
        (tmp_path / rel).mkdir(parents=True)
    (tmp_path / "v2_runs" / "paper-runs-plan.md").write_text("# plan\n")
    monkeypatch.setattr(build_data, "PAPER_OUTPUTS_DIR", tmp_path)

    groups = build_data.resolve_groups([])

    assert sorted(groups) == ["full_fitness_vector", "only_hit_rate"]
    assert [batch for batch, _ in groups["full_fitness_vector"]] == ["v1_runs", "v2_runs", "v10_runs"]
    assert groups["full_fitness_vector"][1][1] == tmp_path / "v2_runs" / "full_fitness_vector"
    assert [batch for batch, _ in groups["only_hit_rate"]] == ["v1_runs"]


def test_unknown_group_selector_exits(tmp_path, monkeypatch):
    (tmp_path / "v1_runs" / "only_hit_rate").mkdir(parents=True)
    monkeypatch.setattr(build_data, "PAPER_OUTPUTS_DIR", tmp_path)
    with pytest.raises(SystemExit, match="v2_runs"):
        build_data.resolve_groups(["v2_runs"])
