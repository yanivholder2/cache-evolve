"""Subject-manifest schema and loader.

A subject (e.g. ``dash``) declares its identity, extension points, and key
file locations in ``manifest.toml`` at its root. This module parses that
manifest into a frozen ``Subject`` dataclass that the rest of the evolve
framework consumes.
"""

from __future__ import annotations

import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any


class SubjectManifestError(ValueError):
    """Raised when a subject manifest is missing, malformed, or invalid."""


@dataclass(frozen=True)
class ExtensionPoint:
    name: str
    trait: str
    impl_files: tuple[Path, ...]
    description: str


@dataclass(frozen=True)
class Docs:
    context: Path
    insight: Path


@dataclass(frozen=True)
class Evaluation:
    fitness_vector: Path


@dataclass(frozen=True)
class Subject:
    subject_dir: Path
    name: str
    crate_path: Path
    cdylib_name: str
    extension_points: tuple[ExtensionPoint, ...]
    docs: Docs
    evaluation: Evaluation

    @property
    def ep_scope_whitelist(self) -> frozenset[Path]:
        return frozenset(path for ep in self.extension_points for path in ep.impl_files)

    @property
    def axis_descriptions(self) -> dict[str, str]:
        return {ep.name: ep.description for ep in self.extension_points}


_ALLOWED_TOP = {"subject", "extension-points", "docs", "evaluation"}
_ALLOWED_SUBJECT = {"name", "crate-path", "cdylib-name"}
_ALLOWED_EP = {"name", "trait", "impl-files", "description"}
_ALLOWED_DOCS = {"context", "insight"}
_ALLOWED_EVAL = {"fitness-vector"}


def _require(table: dict, key: str, where: str) -> Any:
    if key not in table:
        raise SubjectManifestError(f"{where}: missing required field {key!r}")
    return table[key]


def _reject_unknown(table: dict, allowed: set[str], where: str) -> None:
    extras = sorted(k for k in table.keys() if k not in allowed)
    if extras:
        raise SubjectManifestError(f"{where}: unknown field {extras[0]!r}")


def _check_relative(path_str: str, where: str) -> Path:
    if not isinstance(path_str, str):
        raise SubjectManifestError(
            f"{where}: path must be a string, got {type(path_str).__name__}"
        )
    path = Path(path_str)
    if path.is_absolute():
        raise SubjectManifestError(f"{where}: path must be relative, got absolute {path_str!r}")
    if ".." in path.parts:
        raise SubjectManifestError(
            f"{where}: path must not escape subject directory, got {path_str!r}"
        )
    return path


def _check_exists(subject_dir: Path, rel_path: Path, where: str) -> None:
    if not (subject_dir / rel_path).is_file():
        raise SubjectManifestError(f"{where}: file not found at {rel_path}")


def load_subject(subject_dir: Path) -> Subject:
    manifest_path = subject_dir / "manifest.toml"
    if not manifest_path.is_file():
        raise SubjectManifestError(f"manifest.toml not found at {manifest_path}")
    try:
        with manifest_path.open("rb") as handle:
            data = tomllib.load(handle)
    except tomllib.TOMLDecodeError as exception:
        raise SubjectManifestError(
            f"manifest.toml: invalid TOML: {exception}"
        ) from exception

    for key in ("subject", "extension-points", "docs", "evaluation"):
        if key not in data:
            raise SubjectManifestError(f"manifest.toml: missing required table {key!r}")
    _reject_unknown(data, _ALLOWED_TOP, "manifest.toml")

    subject_table = data["subject"]
    ep_tables = data["extension-points"]
    docs_table = data["docs"]
    eval_table = data["evaluation"]

    _reject_unknown(subject_table, _ALLOWED_SUBJECT, "subject")
    name = _require(subject_table, "name", "subject")
    crate_path_raw = _require(subject_table, "crate-path", "subject")
    cdylib_name = _require(subject_table, "cdylib-name", "subject")
    crate_path = _check_relative(crate_path_raw, "subject.crate-path")
    cargo_toml_path = subject_dir / crate_path / "Cargo.toml"
    if not cargo_toml_path.is_file():
        raise SubjectManifestError(
            f"subject.crate-path: no Cargo.toml under {crate_path}"
        )
    cargo_lib_name = _read_cargo_lib_name(cargo_toml_path)
    if cargo_lib_name != cdylib_name:
        raise SubjectManifestError(
            f"subject.cdylib-name: {cdylib_name!r} does not match Cargo.toml lib name {cargo_lib_name!r}"
        )

    if not isinstance(ep_tables, list) or not ep_tables:
        raise SubjectManifestError(
            "extension-points: must be a non-empty array of tables"
        )
    extension_points = tuple(
        _load_extension_point(ep, i, subject_dir) for i, ep in enumerate(ep_tables)
    )
    seen: dict[str, int] = {}
    for i, ep in enumerate(extension_points):
        if ep.name in seen:
            raise SubjectManifestError(
                f"extension-points: duplicate name {ep.name!r} at indices {seen[ep.name]} and {i}"
            )
        seen[ep.name] = i

    _reject_unknown(docs_table, _ALLOWED_DOCS, "docs")
    context_raw = _require(docs_table, "context", "docs")
    insight_raw = _require(docs_table, "insight", "docs")
    context = _check_relative(context_raw, "docs.context")
    insight = _check_relative(insight_raw, "docs.insight")
    _check_exists(subject_dir, context, "docs.context")
    _check_exists(subject_dir, insight, "docs.insight")
    docs = Docs(context=context, insight=insight)

    _reject_unknown(eval_table, _ALLOWED_EVAL, "evaluation")
    fv_raw = _require(eval_table, "fitness-vector", "evaluation")
    fv = _check_relative(fv_raw, "evaluation.fitness-vector")
    _check_exists(subject_dir, fv, "evaluation.fitness-vector")
    evaluation = Evaluation(fitness_vector=fv)

    return Subject(
        subject_dir=subject_dir,
        name=name,
        crate_path=crate_path,
        cdylib_name=cdylib_name,
        extension_points=extension_points,
        docs=docs,
        evaluation=evaluation,
    )


def _load_extension_point(ep: dict, index: int, subject_dir: Path) -> ExtensionPoint:
    where = f"extension-points[{index}]"
    _reject_unknown(ep, _ALLOWED_EP, where)
    name = _require(ep, "name", where)
    trait = _require(ep, "trait", where)
    impl_files_raw = _require(ep, "impl-files", where)
    description = _require(ep, "description", where)
    if not isinstance(description, str) or not description:
        raise SubjectManifestError(f"{where}.description: must be a non-empty string")
    if not isinstance(impl_files_raw, list):
        raise SubjectManifestError(
            f"{where}.impl-files: must be a list of strings, got {type(impl_files_raw).__name__}"
        )
    if not impl_files_raw:
        raise SubjectManifestError(f"{where}.impl-files: must be non-empty")
    impl_files: list[Path] = []
    for raw in impl_files_raw:
        path = _check_relative(raw, f"{where}.impl-files")
        if not (subject_dir / path).is_file():
            raise SubjectManifestError(f"{where}.impl-files: file not found at {path}")
        impl_files.append(path)
    return ExtensionPoint(
        name=name, trait=trait, impl_files=tuple(impl_files), description=description
    )


def _read_cargo_lib_name(cargo_toml_path: Path) -> str:
    try:
        with cargo_toml_path.open("rb") as handle:
            cargo = tomllib.load(handle)
    except tomllib.TOMLDecodeError as exception:
        raise SubjectManifestError(
            f"subject.crate-path: invalid Cargo.toml at {cargo_toml_path}: {exception}"
        ) from exception
    lib = cargo.get("lib")
    if isinstance(lib, dict) and isinstance(lib.get("name"), str):
        return lib["name"]
    package = cargo.get("package")
    if isinstance(package, dict) and isinstance(package.get("name"), str):
        return package["name"]
    raise SubjectManifestError(
        f"subject.crate-path: Cargo.toml at {cargo_toml_path} has no [lib].name or [package].name"
    )
