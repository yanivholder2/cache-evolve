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
