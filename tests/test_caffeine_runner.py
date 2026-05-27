"""Tests for the caffeine_runner CLI + invocation builder."""

from pathlib import Path

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
        output_dir=Path("/tmp/x"),
        num_of_normal_buckets=DEFAULT_NUM_OF_NORMAL_BUCKETS,
        num_of_stash_buckets=DEFAULT_NUM_OF_STASH_BUCKETS,
        bucket_size=DEFAULT_BUCKET_SIZE,
    )
    defaults.update(overrides)
    return Invocation(**defaults)


def test_gradle_cmd_emits_outputdir():
    cmd = _invocation(output_dir=Path("/tmp/x")).gradle_cmd()
    assert "--outputDir=/tmp/x" in cmd


def test_cli_accepts_output_dir_flag():
    args = caffeine_runner._parse_args([
        "run",
        "--library-path", "/lib",
        "--trace", "lirs:multi1.trace.gz",
        "--policies", "classiclru",
        "--size", "512",
        "--output-dir", "/tmp/x",
    ])
    assert args.output_dir == Path("/tmp/x")
