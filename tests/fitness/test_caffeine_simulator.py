"""Tests for the caffeine CSV parser."""

from pathlib import Path

import pytest

from evolve.fitness.caffeine_simulator import (
    CaffeineSimulatorError,
    _parse_csv,
    simulator_debug_mode,
)


HEADER = "Policy,Hit Rate,Miss Rate,Hits,Misses,Requests,Evictions"


def _write(tmp_path: Path, body: str) -> Path:
    csv_path = tmp_path / "hit_rate_512.csv"
    csv_path.write_text(f"{HEADER}\n{body}\n", encoding="utf-8")
    return csv_path


def test_parse_csv_populated_row(tmp_path):
    csv_path = _write(tmp_path, "dashvariant,42.0,58.0,420,580,1000,10")
    [result] = _parse_csv(csv_path, trace="lirs:multi1.trace.gz", size=512)
    assert result.policy == "dashvariant"
    assert result.hit_rate == pytest.approx(0.42)
    assert result.miss_rate == pytest.approx(0.58)
    assert result.hits == 420
    assert result.misses == 580
    assert result.requests == 1000
    assert result.evictions == 10


def test_parse_csv_empty_numeric_cells_treated_as_zero(tmp_path):
    # Caffeine's Metrics.java emits "" for any rate/long equal to zero.
    csv_path = _write(tmp_path, "dashvariant,,100.0,,1000,1000,")
    [result] = _parse_csv(csv_path, trace="lirs:multi1.trace.gz", size=512)
    assert result.hit_rate == 0.0
    assert result.hits == 0
    assert result.evictions == 0
    assert result.miss_rate == pytest.approx(1.0)
    assert result.misses == 1000
    assert result.requests == 1000


def test_parse_csv_empty_policy_rejected(tmp_path):
    csv_path = _write(tmp_path, ",42.0,58.0,420,580,1000,10")
    with pytest.raises(CaffeineSimulatorError, match="missing required fields.*policy"):
        _parse_csv(csv_path, trace="lirs:multi1.trace.gz", size=512)


def test_simulate_variant_surfaces_missing_csv_as_error(tmp_path, monkeypatch):
    """A Caffeine call that exits 0 but writes no CSV (a thrown policy
    constructor) must fail the evaluation, not yield an empty result."""

    import subprocess
    from types import SimpleNamespace

    from evolve.config import (
        Configs, FitnessConfig, FrameworkConfig, SelectionConfig, SimulatorConfig,
    )
    from evolve.fitness import caffeine_simulator
    from evolve.identifiers import CycleId, VariantId
    from evolve.paths import CycleVariant, Layout

    repo_root = Path(__file__).resolve().parents[2]
    layout = Layout(repo_root=repo_root, outputs_root=tmp_path / "out", subject="dash")
    variant = CycleVariant(CycleId(1), VariantId(1))
    variant.resolve(layout).overlay_dir.mkdir(parents=True)
    cfg = Configs(
        framework=FrameworkConfig(
            cycles=1, cohort_size=1, ai_agent_tool="claude-sdk", model="claude-opus-4-8",
            outputs_root=tmp_path, log_level="WARNING", digest_cycles=3,
            max_workers=0, implementation_attempts=3, hypothesis_attempts=3,
            authoring_scope="extension-points", subject="dash",
        ),
        simulator=SimulatorConfig(timeout_s=5),
        fitness=FitnessConfig(dimensions=("hit-rate",)),
        selection=SelectionConfig(method="pareto", winners_count=3, weights=None),
    )
    workspace = SimpleNamespace(root=tmp_path / "ws")
    workspace.root.mkdir()
    monkeypatch.setattr(
        caffeine_simulator, "compile_cdylib",
        lambda *_a, **_kw: (workspace, tmp_path / "lib" / "libdash.so"),
    )
    stderr = "java.lang.IllegalArgumentException: Dash topology must fill maximumSize exactly\n"
    monkeypatch.setattr(
        caffeine_simulator.caffeine_runner.subprocess, "run",
        lambda cmd, **_kw: subprocess.CompletedProcess(cmd, 0, stdout="", stderr=stderr),
    )
    with pytest.raises(CaffeineSimulatorError, match=r"no hit_rate CSV for sizes \[2048\].*\ncause: java.lang.IllegalArgumentException"):
        caffeine_simulator.simulate_variant(variant, layout=layout, repo_root=repo_root, cfg=cfg)
    gradle_stderr = variant.resolve(layout).caffeine_dir.gradle_stderr_log
    assert "IllegalArgumentException" in gradle_stderr.read_text(encoding="utf-8")


@pytest.mark.parametrize(
    ("log_level", "cap", "expected"),
    [
        ("DEBUG", None, 2),
        ("INFO", None, 1),
        ("WARNING", None, 0),
        ("DEBUG", "INFO", 1),
        ("DEBUG", "WARNING", 0),
        ("INFO", "DEBUG", 1),
        ("WARNING", "INFO", 0),
    ],
)
def test_simulator_debug_mode_caps_log_level(log_level, cap, expected):
    assert simulator_debug_mode(log_level, cap) == expected


@pytest.mark.parametrize(("profile", "expected"), [(None, 2), ("msr", 1)])
def test_simulate_variant_passes_capped_debug_mode(tmp_path, monkeypatch, profile, expected):
    """At --log-level DEBUG, dash's msr profile caps the Rust debugMode at INFO
    (1); the default grid keeps DEBUG (2)."""

    from types import SimpleNamespace

    from evolve.config import (
        Configs, FitnessConfig, FrameworkConfig, SelectionConfig, SimulatorConfig,
    )
    from evolve.fitness import caffeine_simulator
    from evolve.identifiers import CycleId, VariantId
    from evolve.paths import CycleVariant, Layout

    repo_root = Path(__file__).resolve().parents[2]
    layout = Layout(
        repo_root=repo_root, outputs_root=tmp_path / "out", subject="dash",
        evaluation_profile=profile,
    )
    variant = CycleVariant(CycleId(1), VariantId(1))
    variant.resolve(layout).overlay_dir.mkdir(parents=True)
    cfg = Configs(
        framework=FrameworkConfig(
            cycles=1, cohort_size=1, ai_agent_tool="claude-sdk", model="claude-opus-4-8",
            outputs_root=tmp_path, log_level="DEBUG", digest_cycles=3,
            max_workers=0, implementation_attempts=3, hypothesis_attempts=3,
            authoring_scope="extension-points", subject="dash",
        ),
        simulator=SimulatorConfig(timeout_s=5),
        fitness=FitnessConfig(dimensions=("hit-rate",)),
        selection=SelectionConfig(method="pareto", winners_count=3, weights=None),
    )
    workspace = SimpleNamespace(root=tmp_path / "ws")
    workspace.root.mkdir()
    monkeypatch.setattr(
        caffeine_simulator, "compile_cdylib",
        lambda *_a, **_kw: (workspace, tmp_path / "lib" / "libdash.so"),
    )
    seen: list[int] = []

    def fake_simulate(**kwargs):
        seen.append(kwargs["debug_mode"])
        raise caffeine_simulator.caffeine_runner.CaffeineError("stop")

    monkeypatch.setattr(caffeine_simulator.caffeine_runner, "simulate", fake_simulate)
    with pytest.raises(CaffeineSimulatorError, match="stop"):
        caffeine_simulator.simulate_variant(variant, layout=layout, repo_root=repo_root, cfg=cfg)
    assert seen == [expected]


@pytest.mark.parametrize(
    ("profile", "expected_calls", "expected_dirs"),
    [
        (None, [("lirs:inputs/traces/workload-a.trace.gz", [2048])], [""]),
        (
            "msr",
            [
                ("lirs:inputs/traces/msr/web_0.trace.gz", [16384, 131072]),
                ("lirs:inputs/traces/msr/hm_0.trace.gz", [16384, 131072]),
            ],
            ["web_0_lirs", "hm_0_lirs"],
        ),
    ],
)
def test_simulate_variant_scores_the_profile_grid(
    tmp_path, monkeypatch, profile, expected_calls, expected_dirs
):
    """A profile run must call Caffeine on the profile's traces and sizes (not
    the default grid), and a multi-trace grid keeps each trace's CSVs in its own
    subdirectory instead of overwriting the previous trace's."""

    from types import SimpleNamespace

    from evolve.config import (
        Configs, FitnessConfig, FrameworkConfig, SelectionConfig, SimulatorConfig,
    )
    from evolve.fitness import caffeine_simulator
    from evolve.identifiers import CycleId, VariantId
    from evolve.paths import CycleVariant, Layout

    repo_root = Path(__file__).resolve().parents[2]
    layout = Layout(
        repo_root=repo_root, outputs_root=tmp_path / "out", subject="dash",
        evaluation_profile=profile,
    )
    variant = CycleVariant(CycleId(1), VariantId(1))
    variant.resolve(layout).overlay_dir.mkdir(parents=True)
    cfg = Configs(
        framework=FrameworkConfig(
            cycles=1, cohort_size=1, ai_agent_tool="claude-sdk", model="claude-opus-4-8",
            outputs_root=tmp_path, log_level="INFO", digest_cycles=3,
            max_workers=0, implementation_attempts=3, hypothesis_attempts=3,
            authoring_scope="extension-points", subject="dash",
        ),
        simulator=SimulatorConfig(timeout_s=5),
        fitness=FitnessConfig(dimensions=("hit-rate",), evaluation_profile=profile),
        selection=SelectionConfig(method="pareto", winners_count=3, weights=None),
    )
    workspace = SimpleNamespace(root=tmp_path / "ws")
    workspace.root.mkdir()
    monkeypatch.setattr(
        caffeine_simulator, "compile_cdylib",
        lambda *_a, **_kw: (workspace, tmp_path / "lib" / "libdash.so"),
    )
    calls: list[tuple[str, list[int]]] = []
    out_dirs: list[Path] = []

    def fake_simulate(**kwargs):
        calls.append((kwargs["trace"], kwargs["sizes"]))
        out_dirs.append(kwargs["output_dir"])
        for size in kwargs["sizes"]:
            (kwargs["output_dir"] / f"hit_rate_{size}.csv").write_text(
                f"{HEADER}\ndash,{len(calls)}0.0,0,1,1,2,0\n", encoding="utf-8"
            )
        return SimpleNamespace(stdout="", stderr="")

    monkeypatch.setattr(caffeine_simulator.caffeine_runner, "simulate", fake_simulate)
    results = caffeine_simulator.simulate_variant(
        variant, layout=layout, repo_root=repo_root, cfg=cfg
    ).results

    assert calls == expected_calls
    caffeine_root = variant.resolve(layout).caffeine_dir.root
    assert [str(d.relative_to(caffeine_root)) if d != caffeine_root else "" for d in out_dirs] == expected_dirs
    # Each (trace, size) point parsed from its own trace's CSV.
    assert [(r.trace, r.size, r.hit_rate) for r in results] == [
        (trace, size, pytest.approx(0.1 * (i + 1)))
        for i, (trace, sizes) in enumerate(expected_calls) for size in sizes
    ]


def test_log_level_map_covers_manifest_simulator_log_levels():
    # simulator_debug_mode indexes the map with a manifest `simulator-log-level`
    # cap; every level the manifest loader accepts must have a debug_mode.
    from evolve.fitness import caffeine_simulator
    from evolve.subject import _SIMULATOR_LOG_LEVELS

    assert set(caffeine_simulator._LOG_LEVEL_TO_DEBUG_MODE) == set(_SIMULATOR_LOG_LEVELS)
