"""Tests for the caffeine_runner CLI + invocation builder."""

from pathlib import Path

import pytest

from scripts import caffeine_runner
from scripts.caffeine_runner import (
    DEFAULT_BUCKET_SIZE,
    DEFAULT_NUM_OF_NORMAL_BUCKETS,
    DEFAULT_NUM_OF_STASH_BUCKETS,
    TARGETS,
    Invocation,
)


def _invocation(**overrides) -> Invocation:
    defaults = dict(
        target=TARGETS["dash"],
        library_path=Path("/lib"),
        trace="lirs:multi1.trace.gz",
        sizes=[512],
        policies=["classiclru"],
        num_of_normal_buckets=DEFAULT_NUM_OF_NORMAL_BUCKETS,
        num_of_stash_buckets=DEFAULT_NUM_OF_STASH_BUCKETS,
        bucket_size=DEFAULT_BUCKET_SIZE,
    )
    defaults.update(overrides)
    return Invocation(**defaults)


# Gradle's simulator:simulate task declares no @Option for outputDir; passing
# --outputDir on the gradle CLI dies with "Unknown command-line option".
def test_gradle_cmd_does_not_emit_outputdir():
    cmd = _invocation().gradle_cmd()
    assert not any("--outputDir" in arg or "--output-dir" in arg for arg in cmd)


def test_invocation_does_not_accept_output_dir_kwarg():
    with pytest.raises(TypeError):
        Invocation(
            target=TARGETS["dash"],
            library_path=Path("/lib"),
            trace="lirs:multi1.trace.gz",
            sizes=[512],
            policies=["classiclru"],
            output_dir=Path("/tmp/x"),
        )


def test_cli_rejects_output_dir_flag():
    with pytest.raises(SystemExit):
        caffeine_runner._parse_args([
            "run",
            "--library-path", "/lib",
            "--trace", "lirs:multi1.trace.gz",
            "--policies", "classiclru",
            "--size", "512",
            "--output-dir", "/tmp/x",
        ])


def test_run_does_not_accept_output_dir_kwarg():
    with pytest.raises(TypeError):
        caffeine_runner.run(
            target="dash",
            library_path=Path("/lib"),
            trace="lirs:multi1.trace.gz",
            size=512,
            policies=["classiclru"],
            timeout_s=60,
            output_dir=Path("/tmp/x"),
        )
