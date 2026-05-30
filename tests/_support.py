"""Shared test helpers for keeping the suite subject-agnostic.

Two ways tests obtain a subject without hardcoding a subject name:

- `configured_subject(repo_root)` returns the subject the repo's own
  `config.toml` selects. Use it when a test points at the real sources and
  needs a real, compilable subject (cdylib builds, the Caffeine simulator).
- `write_stub_subject(subject_dir)` materialises a minimal valid subject tree
  (manifest + the files it references) at an arbitrary directory. Use it for
  plumbing-only tests that merely parse a manifest or build path objects.
"""

from __future__ import annotations

import tomllib
from dataclasses import dataclass
from pathlib import Path

# Name of the stub subject. Deliberately not any real subject name so a stray
# real-subject assumption fails loudly.
STUB_SUBJECT_NAME = "stub_subject"


def configured_subject(repo_root: Path) -> str:
    """Return the subject `config.toml` selects under `repo_root`."""
    config_toml = repo_root / "config.toml"
    with config_toml.open("rb") as handle:
        data = tomllib.load(handle)
    return data["framework"]["subject"]


@dataclass(frozen=True)
class StubSubject:
    """Meaningful paths within a materialised stub subject (all relative to the
    subject dir / a variant `files/` overlay)."""

    name: str
    extension_point_file: Path
    protected_files: tuple[Path, ...]
    free_file: Path  # neither extension-point nor protected — writable in full-autonomy


def write_stub_subject(subject_dir: Path) -> StubSubject:
    """Materialise a minimal valid subject tree at `subject_dir` and return its
    meaningful paths. The manifest declares one extension point, two protected
    files, the required docs + evaluation, and a `[bench]` table."""

    extension_point_file = Path("src/policy/ep.rs")
    protected_files = (Path("src/policy/jni.rs"), Path("src/factory.rs"))
    free_file = Path("src/policy/internals.rs")

    (subject_dir / "src" / "policy").mkdir(parents=True, exist_ok=True)
    for rel in (extension_point_file, *protected_files):
        target = subject_dir / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("// stub\n", encoding="utf-8")
    (subject_dir / "CONTEXT.md").write_text("ctx\n", encoding="utf-8")
    (subject_dir / "baseline").mkdir(parents=True, exist_ok=True)
    (subject_dir / "baseline" / "insight.md").write_text("ins\n", encoding="utf-8")
    (subject_dir / "baseline" / "eval").mkdir(parents=True, exist_ok=True)
    (subject_dir / "baseline" / "eval" / "fitness-vector.json").write_text("{}\n", encoding="utf-8")
    (subject_dir / "Cargo.toml").write_text(
        f'[package]\nname = "{STUB_SUBJECT_NAME}"\n', encoding="utf-8"
    )
    (subject_dir / "manifest.toml").write_text(
        "[subject]\n"
        f'name = "{STUB_SUBJECT_NAME}"\n'
        'crate-path = "."\n'
        f'cdylib-name = "{STUB_SUBJECT_NAME}"\n'
        "\n"
        "[[extension-points]]\n"
        'name = "policy"\n'
        'contract = "stub::policy::Policy"\n'
        f'impl-files = ["{extension_point_file.as_posix()}"]\n'
        'description = "policy seam"\n'
        "\n"
        "[contract]\n"
        "protected-files = ["
        + ", ".join(f'"{p.as_posix()}"' for p in protected_files)
        + "]\n"
        "\n"
        "[caffeine]\n"
        'config-prefix = "caffeine.simulator.stub"\n'
        'policy-spec-name = "stub.StubRust"\n'
        "supports-log-path = false\n"
        "\n"
        "[docs]\n"
        'context = "CONTEXT.md"\n'
        'insight = "baseline/insight.md"\n'
        "\n"
        "[evaluation]\n"
        'fitness-vector = "baseline/eval/fitness-vector.json"\n'
        'traces = ["lirs:multi1.trace.gz"]\n'
        "sizes = [512]\n"
        "\n"
        "[bench]\n"
        'binary-name = "stub-bench"\n',
        encoding="utf-8",
    )
    return StubSubject(
        name=STUB_SUBJECT_NAME,
        extension_point_file=extension_point_file,
        protected_files=protected_files,
        free_file=free_file,
    )
