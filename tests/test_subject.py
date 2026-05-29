"""Tests for the subject-manifest loader."""

from __future__ import annotations

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


def _write_valid_subject(root: Path) -> Path:
    """Materialise a minimal valid subject tree at ``root`` and return it."""
    (root / "src" / "policy").mkdir(parents=True)
    (root / "src" / "policy" / "ep.rs").write_text("// stub\n")
    (root / "docs").mkdir()
    (root / "docs" / "context.md").write_text("ctx\n")
    (root / "docs" / "insight.md").write_text("ins\n")
    (root / "eval").mkdir()
    (root / "eval" / "fitness.json").write_text("{}\n")
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
        'trait = "stub::policy::Policy"\n'
        'impl-files = ["src/policy/ep.rs"]\n'
        'description = "policy seam"\n'
        '\n'
        '[docs]\n'
        'context = "docs/context.md"\n'
        'insight = "docs/insight.md"\n'
        '\n'
        '[evaluation]\n'
        'fitness-vector = "eval/fitness.json"\n'
    )
    return root


def test_loads_real_manifest():
    subject = load_subject(DASH_SUBJECT)

    assert subject.subject_dir == DASH_SUBJECT
    assert subject.name == "dash"
    assert subject.crate_path == Path(".")
    assert subject.cdylib_name == "dash"
    assert len(subject.extension_points) == 2
    assert subject.docs.context == Path("CONTEXT.md")
    assert subject.docs.insight == Path("baseline/insight.md")
    assert subject.evaluation.fitness_vector == Path("baseline/eval/fitness-vector.json")


def test_extension_points_carry_trait_and_impl_files():
    subject = load_subject(DASH_SUBJECT)
    by_name = {ep.name: ep for ep in subject.extension_points}

    eviction = by_name["eviction-policy"]
    assert eviction.trait == "dash::shared::traits::EvictionPolicy"
    assert eviction.impl_files == (Path("src/shared/eviction_policy.rs"),)
    assert eviction.description == "Algorithm choosing which item to evict from a bucket"

    bucket = by_name["bucket"]
    assert bucket.trait == "dash::shared::traits::Bucket"
    assert bucket.impl_files == (Path("src/dash/dash_bucket.rs"),)


def test_ep_scope_whitelist_is_union_of_impl_files():
    subject = load_subject(DASH_SUBJECT)

    assert subject.ep_scope_whitelist == frozenset(
        {
            Path("src/shared/eviction_policy.rs"),
            Path("src/dash/dash_bucket.rs"),
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
          'trait = "stub::policy::Policy2"\n'
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
        ('trait = "stub::policy::Policy"', r"extension-points\[0\].*trait"),
        ('impl-files = ["src/policy/ep.rs"]', r"extension-points\[0\].*impl-files"),
        ('description = "policy seam"', r"extension-points\[0\].*description"),
        ('context = "docs/context.md"', r"docs.*context"),
        ('insight = "docs/insight.md"', r"docs.*insight"),
        ('fitness-vector = "eval/fitness.json"', r"evaluation.*fitness-vector"),
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
          'trait = "stub::policy::Policy2"\n'
          'impl-files = ["src/policy/ep2.rs"]\n'
          'description = "dup"\n',
    )
    with pytest.raises(SubjectManifestError, match=r"duplicate.*policy"):
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
    _rewrite_manifest(
        tmp_path,
        '[subject]\n'
        'name = "stub"\n'
        'crate-path = "."\n'
        'cdylib-name = "stub"\n'
        # no [[extension-points]], no [docs], no [evaluation]
    )
    with pytest.raises(SubjectManifestError, match="extension-points"):
        load_subject(tmp_path)


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


def test_empty_extension_points_array_is_rejected(tmp_path: Path):
    _write_valid_subject(tmp_path)
    _rewrite_manifest(
        tmp_path,
        'extension-points = []\n'
        '\n'
        '[subject]\n'
        'name = "stub"\n'
        'crate-path = "."\n'
        'cdylib-name = "stub"\n'
        '\n'
        '[docs]\n'
        'context = "docs/context.md"\n'
        'insight = "docs/insight.md"\n'
        '\n'
        '[evaluation]\n'
        'fitness-vector = "eval/fitness.json"\n',
    )
    with pytest.raises(SubjectManifestError, match=r"extension-points.*non-empty"):
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
    }
