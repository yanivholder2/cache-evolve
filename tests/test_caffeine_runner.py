"""Tests for the caffeine_runner CLI + invocation builder."""

from pathlib import Path

from scripts import caffeine_runner
from scripts.caffeine_runner import Invocation

# Per-subject Caffeine wiring, mirroring each manifest's [caffeine] table. The
# runner is subject-agnostic; these dicts stand in for what the manifest loader
# feeds it.
DASH_WIRING = dict(
    config_prefix="caffeine.simulator.dash",
    policy_spec_name="dash.DashRust",
    geometry={"numOfNormalBuckets": 2, "numOfStashBuckets": 6, "bucketSize": 64},
    policy_key="evictionPolicies",
    supports_log_path=True,
)
ASSOCIATIVE_CACHE_WIRING = dict(
    config_prefix="caffeine.simulator.associativeCache",
    policy_spec_name="associative.AssociativeCacheRust",
    geometry={"numOfBuckets": [2]},
    policy_key="policy",
    supports_log_path=False,
)
LRU_CACHE_WIRING = dict(
    config_prefix="caffeine.simulator.lruRust",
    policy_spec_name="lrurust.LruRust",
    geometry={},
    policy_key=None,
    supports_log_path=False,
)


def _invocation(*, wiring=DASH_WIRING, **overrides) -> Invocation:
    defaults = dict(
        library_path=Path("/lib"),
        trace="lirs:multi1.trace.gz",
        sizes=[512],
        policies=["classiclru"],
        output_dir=Path("/tmp/x"),
        **wiring,
    )
    defaults.update(overrides)
    return Invocation(**defaults)


def test_gradle_cmd_emits_outputdir():
    cmd = _invocation(output_dir=Path("/tmp/x")).gradle_cmd()
    assert "--outputDir=/tmp/x" in cmd


def test_dash_gradle_cmd_byte_for_byte():
    cmd = _invocation(
        library_path=Path("/lib"),
        output_dir=Path("/tmp/x"),
        log_path=Path("/logs"),
        debug_mode=2,
    ).gradle_cmd()
    # Dash-specific tail must remain exactly as historically emitted.
    assert "-Dcaffeine.simulator.policies.0=dash.DashRust" in cmd
    assert "-Dcaffeine.simulator.dash.numOfNormalBuckets=2" in cmd
    assert "-Dcaffeine.simulator.dash.numOfStashBuckets=6" in cmd
    assert "-Dcaffeine.simulator.dash.bucketSize=64" in cmd
    assert "-Dcaffeine.simulator.dash.debugMode=2" in cmd
    assert "-Dcaffeine.simulator.dash.evictionPolicies.0=classiclru" in cmd
    assert "-Dcaffeine.simulator.dash.logPath=/logs" in cmd
    # Order preserved: geometry, then debugMode, then policy list, then logPath.
    idx = cmd.index
    assert (
        idx("-Dcaffeine.simulator.dash.numOfNormalBuckets=2")
        < idx("-Dcaffeine.simulator.dash.numOfStashBuckets=6")
        < idx("-Dcaffeine.simulator.dash.bucketSize=64")
        < idx("-Dcaffeine.simulator.dash.debugMode=2")
        < idx("-Dcaffeine.simulator.dash.evictionPolicies.0=classiclru")
        < idx("-Dcaffeine.simulator.dash.logPath=/logs")
    )


def test_associative_cache_gradle_cmd_keys():
    cmd = _invocation(
        wiring={**ASSOCIATIVE_CACHE_WIRING, "geometry": {"numOfBuckets": [8]}},
    ).gradle_cmd()
    assert "-Dcaffeine.simulator.policies.0=associative.AssociativeCacheRust" in cmd
    # numOfBuckets is list-typed Java-side (getDoubleList), so it must be
    # emitted in indexed form, not as a scalar.
    assert "-Dcaffeine.simulator.associativeCache.numOfBuckets.0=8" in cmd
    assert "-Dcaffeine.simulator.associativeCache.numOfBuckets=8" not in cmd
    assert "-Dcaffeine.simulator.associativeCache.debugMode=0" in cmd
    assert "-Dcaffeine.simulator.associativeCache.policy.0=classiclru" in cmd
    # Dash-only keys must be absent.
    joined = " ".join(cmd)
    assert "numOfNormalBuckets" not in joined
    assert "numOfStashBuckets" not in joined
    assert "bucketSize" not in joined
    assert "evictionPolicies" not in joined
    assert "logPath" not in joined


def test_lru_cache_gradle_cmd_keys():
    cmd = _invocation(wiring=LRU_CACHE_WIRING).gradle_cmd()
    assert "-Dcaffeine.simulator.policies.0=lrurust.LruRust" in cmd
    assert "-Dcaffeine.simulator.lruRust.debugMode=0" in cmd
    joined = " ".join(cmd)
    # No geometry, no policy list, no logPath for lru_cache.
    assert "numOfBuckets" not in joined
    assert "numOfNormalBuckets" not in joined
    assert "bucketSize" not in joined
    assert "lruRust.policy" not in joined
    assert "evictionPolicies" not in joined
    assert "logPath" not in joined


def test_cli_accepts_output_dir_flag():
    args = caffeine_runner._parse_args([
        "run",
        "--subject-dir", "inputs/subjects/dash",
        "--library-path", "/lib",
        "--trace", "lirs:multi1.trace.gz",
        "--policies", "classiclru",
        "--size", "512",
        "--output-dir", "/tmp/x",
    ])
    assert args.output_dir == Path("/tmp/x")


# --- loud failure ------------------------------------------------------------

import subprocess  # noqa: E402

import pytest  # noqa: E402

# What Caffeine/picocli print when a policy constructor throws: an ANSI-coloured
# throwable header plus stack frames. The JVM can still exit 0.
_JAVA_THROW_STDERR = (
    "Picked up JAVA_TOOL_OPTIONS: -Dfoo\n"
    "\x1b[31m\x1b[1mjava.lang.IllegalArgumentException: Dash topology must fill maximumSize "
    "exactly: ... = 2048 != maximumSize(4096)\x1b[21m\x1b[39m\x1b[0m\n"
    "\x1b[3m\tat com.github.benmanes.caffeine.cache.simulator.policy.dash.DashRustPolicy.<init>"
    "(DashRustPolicy.java:32)\x1b[23m\x1b[0m\n"
)


def _fake_run(returncode: int, stderr: str, *, write_sizes=(), calls=None):
    def run(cmd, **_kwargs):
        if calls is not None:
            calls.append(cmd)
        out_dir = Path(next(a for a in cmd if a.startswith("--outputDir=")).split("=", 1)[1])
        for size in write_sizes:
            out_dir.mkdir(parents=True, exist_ok=True)
            (out_dir / f"hit_rate_{size}.csv").write_text("Policy,Hit Rate\n", encoding="utf-8")
        return subprocess.CompletedProcess(cmd, returncode, stdout="", stderr=stderr)
    return run


def _simulate(tmp_path, sizes, **overrides):
    kwargs = dict(
        library_path=Path("/lib"), trace="lirs:multi1.trace.gz", sizes=sizes,
        policies=["classiclru"], timeout_s=5, output_dir=tmp_path, **DASH_WIRING,
    )
    kwargs.update(overrides)
    return caffeine_runner.simulate(**kwargs)


def test_zero_exit_without_csv_raises_naming_size_and_cause(tmp_path, monkeypatch):
    monkeypatch.setattr(caffeine_runner.subprocess, "run", _fake_run(0, _JAVA_THROW_STDERR))
    with pytest.raises(caffeine_runner.CaffeineError) as excinfo:
        _simulate(tmp_path, [4096])
    message = str(excinfo.value)
    assert "sizes [4096]" in message
    assert "dash.DashRust" in message
    assert "cause: java.lang.IllegalArgumentException: Dash topology must fill" in message
    assert "\x1b[" not in message


def test_partial_csvs_name_only_the_missing_size(tmp_path, monkeypatch):
    monkeypatch.setattr(
        caffeine_runner.subprocess, "run", _fake_run(0, "", write_sizes=(2048,))
    )
    with pytest.raises(caffeine_runner.CaffeineError, match=r"sizes \[4096\].*\ncause: unknown"):
        _simulate(tmp_path, [2048, 4096])


def test_non_zero_exit_names_root_cause(tmp_path, monkeypatch):
    monkeypatch.setattr(caffeine_runner.subprocess, "run", _fake_run(1, _JAVA_THROW_STDERR))
    with pytest.raises(caffeine_runner.CaffeineError, match=r"exit 1.*\ncause: java.lang.IllegalArgumentException"):
        _simulate(tmp_path, [4096])


def test_cli_exits_non_zero_on_missing_csv(tmp_path, monkeypatch, capsys):
    monkeypatch.setattr(caffeine_runner.subprocess, "run", _fake_run(0, _JAVA_THROW_STDERR))
    rc = caffeine_runner._main([
        "simulate", "--subject-dir", "inputs/subjects/dash", "--library-path", "/lib",
        "--trace", "lirs:multi1.trace.gz", "--policies", "classiclru",
        "--sizes", "2048", "--output-dir", str(tmp_path),
    ])
    assert rc == 1
    err = capsys.readouterr().err
    assert "subject dash" in err and "sizes [2048]" in err


# --- size-derived keys -------------------------------------------------------

def test_size_divisor_emits_derived_key():
    cmd = _invocation(sizes=[6144], size_divisors={"numOfSegments": 2048}).gradle_cmd()
    assert "-Dcaffeine.simulator.dash.numOfSegments=3" in cmd
    assert cmd.index("-Dcaffeine.simulator.dash.bucketSize=64") < cmd.index(
        "-Dcaffeine.simulator.dash.numOfSegments=3"
    ) < cmd.index("-Dcaffeine.simulator.dash.debugMode=0")


def test_no_size_divisors_emits_no_derived_key():
    joined = " ".join(_invocation(sizes=[2048, 4096]).gradle_cmd())
    assert "numOfSegments" not in joined
    assert "--maximumSize=2_048,4_096" in joined


def test_size_divisor_rejects_non_multiple():
    with pytest.raises(caffeine_runner.CaffeineError, match="size 6100 is not a positive multiple"):
        _invocation(sizes=[6100], size_divisors={"numOfSegments": 2048}).gradle_cmd()


def test_size_divisors_split_one_call_per_size(tmp_path, monkeypatch):
    calls: list[list[str]] = []
    monkeypatch.setattr(
        caffeine_runner.subprocess, "run",
        _fake_run(0, "", write_sizes=(16384, 131072), calls=calls),
    )
    _simulate(tmp_path, [16384, 131072], size_divisors={"numOfSegments": 2048})
    assert len(calls) == 2
    assert "--maximumSize=16_384" in calls[0]
    assert "-Dcaffeine.simulator.dash.numOfSegments=8" in calls[0]
    assert "--maximumSize=131_072" in calls[1]
    assert "-Dcaffeine.simulator.dash.numOfSegments=64" in calls[1]


def test_size_divisors_validate_all_sizes_before_any_call(tmp_path, monkeypatch):
    calls: list[list[str]] = []
    monkeypatch.setattr(caffeine_runner.subprocess, "run", _fake_run(0, "", calls=calls))
    with pytest.raises(caffeine_runner.CaffeineError, match="size 3000"):
        _simulate(tmp_path, [2048, 3000], size_divisors={"numOfSegments": 2048})
    assert calls == []


def test_stale_csv_does_not_mask_failure(tmp_path, monkeypatch):
    (tmp_path / "hit_rate_4096.csv").write_text("stale\n", encoding="utf-8")
    monkeypatch.setattr(caffeine_runner.subprocess, "run", _fake_run(0, _JAVA_THROW_STDERR))
    with pytest.raises(caffeine_runner.CaffeineError, match=r"sizes \[4096\]"):
        _simulate(tmp_path, [4096])
    assert not (tmp_path / "hit_rate_4096.csv").exists()
