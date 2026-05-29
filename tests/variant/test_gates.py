"""Tests for src/evolve/variant/build.py and src/evolve/variant/gates/.

Cargo is mocked via tests/_mocks/cargo — the shim short-circuits build/test
calls based on markers in the variant source so gate logic runs end-to-end
without paying for real Rust compilation.

Each test stages a fixture into a Layout-shaped scratch dir so the gate code
exercises the same `Variant.resolve(layout)` path as production.
"""

import re
import shutil
import subprocess
from pathlib import Path

import pytest

from evolve._io import read_utf8
from evolve.variant.build import compile_for_gates
from evolve.variant.gates import build_gate_list, run_gates
from evolve.identifiers import CycleId, VariantId
from evolve.paths import CycleVariant, Layout
from dataclasses import replace

from evolve.subject import SubjectGate, SubjectManifestError, load_subject

from _support import configured_subject

REPO_ROOT = Path(__file__).parent.parent.parent
FIXTURES_DIR = Path(__file__).parent.parent / "fixtures"
SUBJECT = configured_subject(REPO_ROOT)

VARIANT = CycleVariant(CycleId(1), VariantId(1))
EXPECTED_VARIANT_TAG = "variant=cycle/0001/v001"


def _stage(tmp_path: Path, fixture_name: str) -> tuple[Layout, CycleVariant]:
    """Stage a fixture into a Layout-shaped scratch dir under tmp_path. The
    repo subtree is a thin symlink farm pointing at the real subject + framework
    crate so the build harness's copytree + overlay path resolves to the real
    sources without mutating them. The overlay lands under attempts/01/overlay,
    matching the first-attempt layout the implementer produces."""

    layout = Layout(repo_root=tmp_path / "repo", outputs_root=tmp_path / "out", subject=SUBJECT)
    layout.repo_root.mkdir(parents=True, exist_ok=True)
    real = REPO_ROOT
    layout.sources.subject_dir.parent.mkdir(parents=True, exist_ok=True)
    (layout.sources.subject_dir).symlink_to(real / "inputs" / "subjects" / SUBJECT)
    layout.sources.framework_crate_dir.symlink_to(real / "framework-rs")
    variant_dir = layout.outputs.cycle(VARIANT.cycle_id).variant(VARIANT.variant_id)
    overlay_dst = variant_dir.attempt(1).overlay_dir
    fixture_files = FIXTURES_DIR / fixture_name / "files"
    if fixture_files.is_dir():
        shutil.copytree(fixture_files, overlay_dst)
    else:
        overlay_dst.mkdir(parents=True, exist_ok=True)
    return layout, VARIANT


def test_build_gate_list_appends_subject_gates_after_framework():
    subject = load_subject(REPO_ROOT / "inputs" / "subjects" / SUBJECT)
    gate_names = [g.name for g in build_gate_list(subject)]
    assert gate_names[:3] == ["build gate", "contract gate", "universal invariant gate"]
    # The invariant gate is declared in the subject manifest, not framework code.
    assert "invariant gate" in gate_names[3:]
    assert [g.name for g in subject.gates] == gate_names[3:]


def test_build_gate_list_rejects_subject_gate_colliding_with_framework_name():
    subject = load_subject(REPO_ROOT / "inputs" / "subjects" / SUBJECT)
    colliding = SubjectGate(name="contract gate", test="contract_gate", marker=None)
    subject = replace(subject, gates=(colliding,))
    with pytest.raises(SubjectManifestError, match=r"reserved by framework gates"):
        build_gate_list(subject)


def test_build_good_fixture_returns_workspace(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    workspace = compile_for_gates(variant, layout=layout, repo_root=REPO_ROOT)
    assert workspace.root.is_dir()
    shutil.rmtree(workspace.root, ignore_errors=True)


def test_build_broken_fixture_raises(tmp_path):
    layout, variant = _stage(tmp_path, "broken")
    with pytest.raises(subprocess.CalledProcessError):
        compile_for_gates(variant, layout=layout, repo_root=REPO_ROOT)


def test_gate_check_good_fixture_passes(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    assert run_gates(variant, layout=layout, repo_root=REPO_ROOT)


def test_gate_check_good_fixture_produces_no_rejection_entry(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    run_gates(variant, layout=layout, repo_root=REPO_ROOT)
    rejection_md = variant.resolve(layout).rejection_md
    assert not rejection_md.is_file() or read_utf8(rejection_md).strip() == ""


def test_gate_check_broken_fixture_fails(tmp_path):
    layout, variant = _stage(tmp_path, "broken")
    assert not run_gates(variant, layout=layout, repo_root=REPO_ROOT)


def test_gate_check_broken_fixture_logs_structured_build_entry(tmp_path):
    layout, variant = _stage(tmp_path, "broken")
    result = run_gates(variant, layout=layout, repo_root=REPO_ROOT)
    assert result.first_failed_gate == "build gate"
    content = read_utf8(variant.resolve(layout).rejection_md)
    assert "build gate" in content
    assert EXPECTED_VARIANT_TAG in content
    assert "attempt=1" in content


def test_gate_check_bad_invariant_fixture_fails_universal_invariant_gate(tmp_path):
    layout, variant = _stage(tmp_path, "bad-invariant")
    result = run_gates(variant, layout=layout, repo_root=REPO_ROOT)
    assert result.outcomes["build gate"].passed is True
    assert result.outcomes["contract gate"].passed is True
    assert result.outcomes["universal invariant gate"].passed is False
    # The universal gate short-circuits the subject invariant gate, so it never
    # runs and is absent from the outcomes.
    assert "invariant gate" not in result.outcomes
    assert not result.all_passed
    assert result.first_failed_gate == "universal invariant gate"


def test_gate_check_bad_invariant_fixture_logs_failing_invariant(tmp_path):
    layout, variant = _stage(tmp_path, "bad-invariant")
    run_gates(variant, layout=layout, repo_root=REPO_ROOT)
    content = read_utf8(variant.resolve(layout).rejection_md)
    assert "universal invariant gate" in content
    assert EXPECTED_VARIANT_TAG in content
    # **Failing invariant** must name a U<n> law, not just appear in the
    # embedded panic block.
    assert re.search(r"\*\*Failing invariant\*\*: U[0-9]+ [a-z_]+", content)


def test_gate_check_subject_invariant_gate_fails(tmp_path):
    """A failure of the manifest-declared subject invariant gate is caught and
    named after the framework gates pass."""
    layout, variant = _stage(tmp_path, "bad-subject-invariant")
    result = run_gates(variant, layout=layout, repo_root=REPO_ROOT)
    assert result.outcomes["build gate"].passed is True
    assert result.outcomes["contract gate"].passed is True
    assert result.outcomes["universal invariant gate"].passed is True
    assert result.outcomes["invariant gate"].passed is False
    assert result.first_failed_gate == "invariant gate"
    content = read_utf8(variant.resolve(layout).rejection_md)
    assert re.search(r"\*\*Failing invariant\*\*: P[1-5] [a-z_]+", content)


def test_gate_check_good_fixture_passes_all_gates(tmp_path):
    layout, variant = _stage(tmp_path, "good")
    result = run_gates(variant, layout=layout, repo_root=REPO_ROOT)
    assert result.all_passed
    assert all(outcome.passed for outcome in result.outcomes.values())
    assert result.outcomes["invariant gate"].passed is True
    rejection_md = variant.resolve(layout).rejection_md
    assert not rejection_md.is_file() or read_utf8(rejection_md).strip() == ""
