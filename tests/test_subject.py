"""Tests for the subject-manifest loader."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from evolve.subject import (
    Docs,
    Evaluation,
    ExtensionPoint,
    Subject,
    SubjectManifestError,
    load_subject,
)

REPO_ROOT = Path(__file__).resolve().parents[1]
DASH_SUBJECT = REPO_ROOT / "inputs" / "subjects" / "dash"
ASSOCIATIVE_CACHE_SUBJECT = REPO_ROOT / "inputs" / "subjects" / "associative_cache"


def _write_baseline(root: Path, grid: list[tuple[str, int]]) -> None:
    """Write a baseline fitness vector whose hit-rate grid matches ``grid``."""
    (root / "eval" / "fitness.json").write_text(
        json.dumps(
            {
                "variant": "baseline",
                "dimensions": [
                    {
                        "dim": "hit-rate",
                        "values": [
                            {"trace": trace, "size": size, "hit_rate": 0.5}
                            for trace, size in grid
                        ],
                        "summary": 0.5,
                        "higher_is_better": True,
                    }
                ],
            }
        )
        + "\n"
    )


def _write_valid_subject(root: Path) -> Path:
    """Materialise a minimal valid subject tree at ``root`` and return it."""
    (root / "src" / "policy").mkdir(parents=True)
    (root / "src" / "policy" / "ep.rs").write_text("// stub\n")
    (root / "src" / "policy" / "jni.rs").write_text("// jni\n")
    (root / "docs").mkdir()
    (root / "docs" / "context.md").write_text("ctx\n")
    (root / "docs" / "insight.md").write_text("ins\n")
    (root / "eval").mkdir()
    _write_baseline(root, [("lirs:multi1.trace.gz", 512)])
    (root / "Cargo.toml").write_text("[package]\nname = \"stub\"\n")
    manifest = root / "manifest.toml"
    manifest.write_text(
        '[subject]\n'
        'name = "stub"\n'
        'crate-path = "."\n'
        'cdylib-name = "stub"\n'
        '\n'
        '[[extension-points]]\n'
        'name = "policy"\n'
        'contract = "stub::policy::Policy"\n'
        'impl-files = ["src/policy/ep.rs"]\n'
        'description = "policy seam"\n'
        '\n'
        '[contract]\n'
        'protected-files = ["src/policy/jni.rs"]\n'
        '\n'
        '[caffeine]\n'
        'config-prefix = "caffeine.simulator.stub"\n'
        'policy-spec-name = "stub.StubRust"\n'
        'supports-log-path = false\n'
        '\n'
        '[docs]\n'
        'context = "docs/context.md"\n'
        'insight = "docs/insight.md"\n'
        '\n'
        '[evaluation]\n'
        'fitness-vector = "eval/fitness.json"\n'
        'traces = ["lirs:multi1.trace.gz"]\n'
        'sizes = [512]\n'
    )
    return root


def test_loads_real_manifest():
    subject = load_subject(DASH_SUBJECT)

    assert subject.subject_dir == DASH_SUBJECT
    assert subject.name == "dash"
    assert subject.crate_path == Path(".")
    assert subject.cdylib_name == "dash"
    assert {ep.name for ep in subject.extension_points} == {
        "eviction-policy",
        "bucket",
        "item",
        "segment",
        "geometry",
    }
    assert subject.protected_files == (Path("src/dash/jni.rs"), Path("src/factory.rs"))
    assert subject.docs.context == Path("CONTEXT.md")
    assert subject.docs.insight == Path("baseline/insight.md")
    assert subject.evaluation.fitness_vector == Path("baseline/eval/fitness-vector.json")
    # traces and sizes are operational sweep knobs (tuned per run), so assert
    # their shape, not specific values — exact grid parsing is covered
    # hermetically by the fixture-based grid tests below.
    assert subject.evaluation.traces and all(
        ":" in trace for trace in subject.evaluation.traces
    )
    assert subject.evaluation.sizes and all(
        isinstance(size, int) and size > 0 for size in subject.evaluation.sizes
    )
    assert subject.caffeine.config_prefix == "caffeine.simulator.dash"
    assert subject.caffeine.policy_spec_name == "dash.DashRust"
    assert subject.caffeine.policy_key == "evictionPolicies"
    assert subject.caffeine.supports_log_path is True
    # geometry numbers are the swept baseline topology; assert the key shape the
    # loader must preserve, not the operational values.
    assert set(subject.caffeine.geometry) == {
        "numOfNormalBuckets",
        "numOfStashBuckets",
        "bucketSize",
    }
    assert all(isinstance(value, int) for value in subject.caffeine.geometry.values())
    # dash evolves for hit rate only; it declares neither [bench] nor [metadata].
    assert subject.bench is None
    assert subject.metadata is None


def test_associative_cache_geometry_is_list_valued():
    # numOfBuckets is read Java-side via getDoubleList, so the manifest declares
    # it as a list; the loader must preserve that shape. The values are an
    # operational sweep, so assert the list shape, not the swept numbers.
    subject = load_subject(ASSOCIATIVE_CACHE_SUBJECT)
    geometry = subject.caffeine.geometry
    assert set(geometry) == {"numOfBuckets"}
    assert isinstance(geometry["numOfBuckets"], list)
    assert geometry["numOfBuckets"]
    assert all(isinstance(count, int) for count in geometry["numOfBuckets"])


def test_list_geometry_must_be_non_empty_integers(tmp_path: Path):
    root = _write_valid_subject(tmp_path)
    text = (root / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text + '\n[caffeine.geometry]\nnumOfBuckets = []\n')
    with pytest.raises(SubjectManifestError, match=r"geometry.numOfBuckets.*non-empty integers"):
        load_subject(root)


def test_extension_points_carry_contract_and_impl_files():
    subject = load_subject(DASH_SUBJECT)
    by_name = {ep.name: ep for ep in subject.extension_points}

    eviction = by_name["eviction-policy"]
    assert eviction.contract == "dash::shared::traits::EvictionPolicy"
    assert eviction.impl_files == (Path("src/shared/eviction_policy.rs"),)
    assert eviction.description == "Algorithm choosing which item to evict from a bucket"

    bucket = by_name["bucket"]
    assert bucket.contract == "dash::shared::traits::Bucket"
    assert bucket.impl_files == (Path("src/dash/dash_bucket.rs"),)

    item = by_name["item"]
    assert item.contract == "dash::shared::item::Item"
    assert Path("src/shared/item.rs") in item.impl_files


def test_ep_scope_whitelist_is_union_of_impl_files():
    subject = load_subject(DASH_SUBJECT)

    assert subject.ep_scope_whitelist == frozenset(
        {
            Path("src/shared/eviction_policy.rs"),
            Path("src/dash/dash_bucket.rs"),
            Path("src/shared/item.rs"),
            Path("src/dash/segments/dash_segment.rs"),
            Path("src/shared/geometry.rs"),
        }
    )


def test_whitelist_dedupes_overlapping_impl_files(tmp_path: Path):
    _write_valid_subject(tmp_path)
    (tmp_path / "src" / "policy" / "ep2.rs").write_text("// stub\n")
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(
        tmp_path,
        text
        + '\n[[extension-points]]\n'
          'name = "policy2"\n'
          'contract = "stub::policy::Policy2"\n'
          'impl-files = ["src/policy/ep.rs", "src/policy/ep2.rs"]\n'
          'description = "overlaps with policy"\n',
    )
    subject = load_subject(tmp_path)

    assert subject.ep_scope_whitelist == frozenset(
        {Path("src/policy/ep.rs"), Path("src/policy/ep2.rs")}
    )


def test_synthesized_subject_loads(tmp_path: Path):
    _write_valid_subject(tmp_path)
    subject = load_subject(tmp_path)
    assert subject.name == "stub"
    assert subject.ep_scope_whitelist == frozenset({Path("src/policy/ep.rs")})


def _rewrite_manifest(root: Path, text: str) -> None:
    (root / "manifest.toml").write_text(text)


def test_missing_manifest_file_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    (tmp_path / "manifest.toml").unlink()
    with pytest.raises(SubjectManifestError, match="manifest.toml"):
        load_subject(tmp_path)


@pytest.mark.parametrize(
    "drop_line, expected_match",
    [
        ('name = "stub"', r"subject.*name"),
        ('crate-path = "."', r"subject.*crate-path"),
        ('cdylib-name = "stub"', r"subject.*cdylib-name"),
        ('name = "policy"', r"extension-points\[0\].*name"),
        ('contract = "stub::policy::Policy"', r"extension-points\[0\].*contract"),
        ('impl-files = ["src/policy/ep.rs"]', r"extension-points\[0\].*impl-files"),
        ('description = "policy seam"', r"extension-points\[0\].*description"),
        ('context = "docs/context.md"', r"docs.*context"),
        ('insight = "docs/insight.md"', r"docs.*insight"),
        ('fitness-vector = "eval/fitness.json"', r"evaluation.*fitness-vector"),
        ('traces = ["lirs:multi1.trace.gz"]', r"evaluation.*traces"),
        ('sizes = [512]', r"evaluation.*sizes"),
    ],
)
def test_missing_required_field_within_table_is_rejected(
    tmp_path: Path, drop_line: str, expected_match: str
):
    _write_valid_subject(tmp_path)
    lines = (tmp_path / "manifest.toml").read_text().splitlines()
    assert drop_line in lines, f"setup error: line {drop_line!r} not in manifest"
    lines.remove(drop_line)
    _rewrite_manifest(tmp_path, "\n".join(lines) + "\n")
    with pytest.raises(SubjectManifestError, match=expected_match):
        load_subject(tmp_path)


def test_nonexistent_impl_file_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text.replace("src/policy/ep.rs", "src/policy/missing.rs"))
    with pytest.raises(SubjectManifestError, match=r"extension-points\[0\].*impl-files.*missing\.rs"):
        load_subject(tmp_path)


def test_nonexistent_docs_context_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text.replace("docs/context.md", "docs/nope.md"))
    with pytest.raises(SubjectManifestError, match=r"docs.*context.*nope\.md"):
        load_subject(tmp_path)


def test_nonexistent_docs_insight_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text.replace("docs/insight.md", "docs/nope.md"))
    with pytest.raises(SubjectManifestError, match=r"docs.*insight.*nope\.md"):
        load_subject(tmp_path)


def test_nonexistent_fitness_vector_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text.replace("eval/fitness.json", "eval/nope.json"))
    with pytest.raises(SubjectManifestError, match=r"evaluation.*fitness-vector.*nope\.json"):
        load_subject(tmp_path)


def test_evaluation_grid_parsed():
    # Smoke-test that the real manifest's evaluation grid loads; traces and sizes
    # are operational sweep knobs, so assert shape, not values. Exact grid parsing
    # is covered hermetically by test_multi_point_grid_parsed.
    subject = load_subject(ASSOCIATIVE_CACHE_SUBJECT)
    assert subject.evaluation.traces and all(
        ":" in trace for trace in subject.evaluation.traces
    )
    assert subject.evaluation.sizes and all(
        isinstance(size, int) and size > 0 for size in subject.evaluation.sizes
    )


def test_multi_point_grid_parsed(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(
        tmp_path,
        text.replace('traces = ["lirs:multi1.trace.gz"]', 'traces = ["lirs:multi1.trace.gz", "arc:p1.trace"]')
            .replace("sizes = [512]", "sizes = [256, 512, 1024]"),
    )
    _write_baseline(
        tmp_path,
        [
            (trace, size)
            for trace in ("lirs:multi1.trace.gz", "arc:p1.trace")
            for size in (256, 512, 1024)
        ],
    )
    grid = load_subject(tmp_path).evaluation
    assert grid.traces == ("lirs:multi1.trace.gz", "arc:p1.trace")
    assert grid.sizes == (256, 512, 1024)


def test_empty_traces_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text.replace('traces = ["lirs:multi1.trace.gz"]', "traces = []"))
    with pytest.raises(SubjectManifestError, match=r"evaluation.traces.*non-empty"):
        load_subject(tmp_path)


def test_trace_without_format_prefix_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text.replace('"lirs:multi1.trace.gz"', '"multi1.trace.gz"'))
    with pytest.raises(SubjectManifestError, match=r"evaluation.traces.*format.*path"):
        load_subject(tmp_path)


def test_non_string_trace_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text.replace('traces = ["lirs:multi1.trace.gz"]', "traces = [1]"))
    with pytest.raises(SubjectManifestError, match=r"evaluation.traces"):
        load_subject(tmp_path)


def test_empty_sizes_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text.replace("sizes = [512]", "sizes = []"))
    with pytest.raises(SubjectManifestError, match=r"evaluation.sizes.*non-empty"):
        load_subject(tmp_path)


def test_non_positive_size_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text.replace("sizes = [512]", "sizes = [0]"))
    with pytest.raises(SubjectManifestError, match=r"evaluation.sizes.*positive"):
        load_subject(tmp_path)


def test_non_integer_size_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text.replace("sizes = [512]", "sizes = [512.5]"))
    with pytest.raises(SubjectManifestError, match=r"evaluation.sizes.*positive integer"):
        load_subject(tmp_path)


def test_duplicate_size_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text.replace("sizes = [512]", "sizes = [512, 512]"))
    with pytest.raises(SubjectManifestError, match=r"evaluation.sizes.*duplicate"):
        load_subject(tmp_path)


def test_baseline_grid_mismatch_is_rejected(tmp_path: Path):
    # A stale/stub anchor (wrong sizes, and keyed by `cache` not `trace`) must
    # be caught at load time before it poisons the global champions/leaderboard.
    _write_valid_subject(tmp_path)
    (tmp_path / "eval" / "fitness.json").write_text(
        json.dumps(
            {
                "variant": "baseline",
                "dimensions": [
                    {
                        "dim": "hit-rate",
                        "values": [{"cache": "lru", "size": 16, "hit_rate": 0.0}],
                        "summary": 0.0,
                        "higher_is_better": True,
                    }
                ],
            }
        )
    )
    with pytest.raises(SubjectManifestError, match=r"hit-rate.*trace.*size|grid.*does not match"):
        load_subject(tmp_path)


def test_baseline_missing_hit_rate_dimension_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    (tmp_path / "eval" / "fitness.json").write_text(json.dumps({"dimensions": []}))
    with pytest.raises(SubjectManifestError, match=r"missing the 'hit-rate' dimension"):
        load_subject(tmp_path)


def test_baseline_malformed_json_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    (tmp_path / "eval" / "fitness.json").write_text("not json {{{")
    with pytest.raises(SubjectManifestError, match=r"cannot read baseline fitness vector"):
        load_subject(tmp_path)


def test_crate_path_without_cargo_toml_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    (tmp_path / "Cargo.toml").unlink()
    with pytest.raises(SubjectManifestError, match=r"subject.*crate-path.*Cargo\.toml"):
        load_subject(tmp_path)


def test_duplicate_extension_point_names_are_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    (tmp_path / "src" / "policy" / "ep2.rs").write_text("// stub\n")
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(
        tmp_path,
        text
        + '\n[[extension-points]]\n'
          'name = "policy"\n'
          'contract = "stub::policy::Policy2"\n'
          'impl-files = ["src/policy/ep2.rs"]\n'
          'description = "dup"\n',
    )
    with pytest.raises(SubjectManifestError, match=r"duplicate.*policy"):
        load_subject(tmp_path)


def test_duplicate_gate_names_are_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(
        tmp_path,
        text
        + '\n[[gates]]\n'
          'name = "invariant gate"\n'
          'test = "invariant_gate"\n'
        + '\n[[gates]]\n'
          'name = "invariant gate"\n'
          'test = "invariant_gate_2"\n',
    )
    with pytest.raises(SubjectManifestError, match=r"gates: duplicate name.*invariant gate"):
        load_subject(tmp_path)


def test_empty_impl_files_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(
        tmp_path, text.replace('impl-files = ["src/policy/ep.rs"]', "impl-files = []")
    )
    with pytest.raises(SubjectManifestError, match=r"extension-points\[0\].*impl-files.*empty"):
        load_subject(tmp_path)


def test_absolute_path_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(
        tmp_path,
        text.replace('"src/policy/ep.rs"', '"/etc/passwd"'),
    )
    with pytest.raises(SubjectManifestError, match=r"absolute"):
        load_subject(tmp_path)


def test_unknown_top_level_table_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text + '\n[bogus]\nkey = "value"\n')
    with pytest.raises(SubjectManifestError, match=r"unknown.*bogus"):
        load_subject(tmp_path)


def test_unknown_field_in_subject_table_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text.replace('cdylib-name = "stub"', 'cdylib-name = "stub"\nbogus = 1'))
    with pytest.raises(SubjectManifestError, match=r"subject.*unknown.*bogus"):
        load_subject(tmp_path)


def test_missing_required_top_level_table_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    # Drop the [contract] table entirely; it is now a required top-level table.
    _rewrite_manifest(
        tmp_path,
        text.replace('[contract]\nprotected-files = ["src/policy/jni.rs"]\n\n', ""),
    )
    with pytest.raises(SubjectManifestError, match="contract"):
        load_subject(tmp_path)


def test_missing_caffeine_table_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(
        tmp_path,
        text.replace(
            '[caffeine]\n'
            'config-prefix = "caffeine.simulator.stub"\n'
            'policy-spec-name = "stub.StubRust"\n'
            'supports-log-path = false\n'
            '\n',
            "",
        ),
    )
    with pytest.raises(SubjectManifestError, match="caffeine"):
        load_subject(tmp_path)


def test_bench_is_optional_and_absent_by_default(tmp_path: Path):
    _write_valid_subject(tmp_path)
    assert load_subject(tmp_path).bench is None


def test_bench_table_parsed_when_present(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text + '\n[bench]\nbinary-name = "stub-bench"\n')
    bench = load_subject(tmp_path).bench
    assert bench is not None
    assert bench.binary_name == "stub-bench"


def test_parent_traversal_in_crate_path_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text.replace('crate-path = "."', 'crate-path = ".."'))
    with pytest.raises(SubjectManifestError, match=r"subject.crate-path.*escape"):
        load_subject(tmp_path)


def test_parent_traversal_in_impl_files_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(
        tmp_path, text.replace('"src/policy/ep.rs"', '"../outside.rs"')
    )
    with pytest.raises(SubjectManifestError, match=r"impl-files.*escape"):
        load_subject(tmp_path)


def test_parent_traversal_in_docs_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(
        tmp_path, text.replace('"docs/context.md"', '"../context.md"')
    )
    with pytest.raises(SubjectManifestError, match=r"docs.context.*escape"):
        load_subject(tmp_path)


def test_malformed_toml_raises_subject_manifest_error(tmp_path: Path):
    _write_valid_subject(tmp_path)
    _rewrite_manifest(tmp_path, '[subject\nname = "stub"\n')
    with pytest.raises(SubjectManifestError, match=r"invalid TOML"):
        load_subject(tmp_path)


def test_cdylib_name_mismatch_with_cargo_lib_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text.replace('cdylib-name = "stub"', 'cdylib-name = "wrong"'))
    with pytest.raises(SubjectManifestError, match=r"cdylib-name.*'wrong'.*'stub'"):
        load_subject(tmp_path)


def test_cdylib_name_resolves_lib_name_over_package_name(tmp_path: Path):
    _write_valid_subject(tmp_path)
    (tmp_path / "Cargo.toml").write_text(
        '[package]\nname = "pkg-name"\n[lib]\nname = "stub"\n'
    )
    load_subject(tmp_path)


def test_cdylib_name_resolves_package_name_when_no_lib_table(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text.replace('cdylib-name = "stub"', 'cdylib-name = "pkg-name"'))
    (tmp_path / "Cargo.toml").write_text('[package]\nname = "pkg-name"\n')
    load_subject(tmp_path)


def _write_zero_ep_manifest(root: Path) -> None:
    _rewrite_manifest(
        root,
        '[subject]\n'
        'name = "stub"\n'
        'crate-path = "."\n'
        'cdylib-name = "stub"\n'
        '\n'
        '[contract]\n'
        'protected-files = ["src/policy/jni.rs"]\n'
        '\n'
        '[caffeine]\n'
        'config-prefix = "caffeine.simulator.stub"\n'
        'policy-spec-name = "stub.StubRust"\n'
        'supports-log-path = false\n'
        '\n'
        '[docs]\n'
        'context = "docs/context.md"\n'
        'insight = "docs/insight.md"\n'
        '\n'
        '[evaluation]\n'
        'fitness-vector = "eval/fitness.json"\n'
        'traces = ["lirs:multi1.trace.gz"]\n'
        'sizes = [512]\n',
    )


def test_absent_extension_points_loads_with_empty_tuple(tmp_path: Path):
    _write_valid_subject(tmp_path)
    _write_zero_ep_manifest(tmp_path)
    subject = load_subject(tmp_path)
    assert subject.extension_points == ()
    assert subject.ep_scope_whitelist == frozenset()


def test_empty_extension_points_array_loads_with_empty_tuple(tmp_path: Path):
    _write_valid_subject(tmp_path)
    _write_zero_ep_manifest(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, "extension-points = []\n\n" + text)
    subject = load_subject(tmp_path)
    assert subject.extension_points == ()
    assert subject.ep_scope_whitelist == frozenset()


def test_contract_protected_files_parsed(tmp_path: Path):
    _write_valid_subject(tmp_path)
    subject = load_subject(tmp_path)
    assert subject.protected_files == (Path("src/policy/jni.rs"),)


def test_missing_contract_table_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(
        tmp_path,
        text.replace('[contract]\nprotected-files = ["src/policy/jni.rs"]\n\n', ""),
    )
    with pytest.raises(SubjectManifestError, match=r"contract"):
        load_subject(tmp_path)


def test_missing_protected_files_key_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(
        tmp_path,
        text.replace('protected-files = ["src/policy/jni.rs"]', ""),
    )
    with pytest.raises(SubjectManifestError, match=r"contract.*protected-files"):
        load_subject(tmp_path)


def test_empty_protected_files_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(
        tmp_path,
        text.replace('protected-files = ["src/policy/jni.rs"]', "protected-files = []"),
    )
    with pytest.raises(SubjectManifestError, match=r"protected-files.*non-empty"):
        load_subject(tmp_path)


def test_nonexistent_protected_file_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(
        tmp_path,
        text.replace('"src/policy/jni.rs"', '"src/policy/missing.rs"'),
    )
    with pytest.raises(SubjectManifestError, match=r"protected-files.*missing\.rs"):
        load_subject(tmp_path)


def test_absolute_protected_file_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(
        tmp_path,
        text.replace('"src/policy/jni.rs"', '"/etc/passwd"'),
    )
    with pytest.raises(SubjectManifestError, match=r"absolute"):
        load_subject(tmp_path)


def test_parent_traversal_in_protected_file_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(
        tmp_path,
        text.replace('"src/policy/jni.rs"', '"../outside.rs"'),
    )
    with pytest.raises(SubjectManifestError, match=r"protected-files.*escape"):
        load_subject(tmp_path)


def test_empty_description_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(tmp_path, text.replace('description = "policy seam"', 'description = ""'))
    with pytest.raises(SubjectManifestError, match=r"description.*non-empty"):
        load_subject(tmp_path)


def test_non_list_impl_files_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(
        tmp_path, text.replace('impl-files = ["src/policy/ep.rs"]', 'impl-files = "src/policy/ep.rs"')
    )
    with pytest.raises(SubjectManifestError, match=r"impl-files.*list"):
        load_subject(tmp_path)


def test_non_string_impl_files_entry_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    text = (tmp_path / "manifest.toml").read_text()
    _rewrite_manifest(
        tmp_path, text.replace('impl-files = ["src/policy/ep.rs"]', 'impl-files = [1]')
    )
    with pytest.raises(SubjectManifestError, match=r"impl-files.*string"):
        load_subject(tmp_path)


def test_axis_descriptions_map_name_to_description():
    subject = load_subject(DASH_SUBJECT)

    assert subject.axis_descriptions == {
        "eviction-policy": "Algorithm choosing which item to evict from a bucket",
        "bucket": "Container for items within a segment",
        "item": (
            "Storage unit a bucket holds; its layout and metadata co-evolve with the "
            "eviction policy that reads them. Keep Item::new(key, value) constructible "
            "and the key/value fields present."
        ),
        "segment": (
            "The segment's get-time strategy: where items land on a hit and how they "
            "migrate between the stash, target, and probing slots. Implements the "
            "Segment trait over the shared SegmentCore."
        ),
        "geometry": (
            "Factors the fixed total capacity into segment count, normal/stash "
            "bucket counts, and bucket size. Reshapes topology only; keeps the "
            "budget constant."
        ),
    }
