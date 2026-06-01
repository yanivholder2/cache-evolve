"""Subject-manifest schema and loader.

A subject declares its identity, extension points, Caffeine wiring, and key
file locations in ``manifest.toml`` at its root. This module parses that
manifest into a frozen ``Subject`` dataclass that the rest of the evolve
framework consumes.
"""

from __future__ import annotations

import json
import tomllib
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path
from typing import Any


class SubjectManifestError(ValueError):
    """Raised when a subject manifest is missing, malformed, or invalid."""


@dataclass(frozen=True)
class ExtensionPoint:
    name: str
    contract: str
    impl_files: tuple[Path, ...]
    description: str


@dataclass(frozen=True)
class SubjectGate:
    """A subject-declared correctness gate, run after the framework gates.

    `test` names the `cargo test --test <test>` target under the subject's
    `tests/`. `marker` is an optional regex matched against the failing test's
    output to name the violated invariant in rejection.md."""

    name: str
    test: str
    marker: str | None


@dataclass(frozen=True)
class Docs:
    context: Path
    insight: Path


@dataclass(frozen=True)
class Evaluation:
    """How a variant's fitness is measured.

    `fitness_vector` is the committed baseline anchor. `traces` and `sizes`
    declare the evaluation grid: each trace is a `"<format>:<path>"` string and
    sizes are cache capacities. The grid is their Cartesian product, scored
    identically for the baseline anchor and every variant.
    """

    fitness_vector: Path
    traces: tuple[str, ...]
    sizes: tuple[int, ...]


@dataclass(frozen=True)
class CaffeineConfig:
    """How the Caffeine simulator drives this subject's policy.

    Declarative wiring read from the manifest `[caffeine]` table; the gradle
    command builder (`scripts.caffeine_runner`) consumes these fields without
    knowing any subject by name. `geometry` maps a Caffeine config-key suffix
    (under `config_prefix`) to its fixed value; subjects sized solely by
    `--maximumSize` (e.g. lru) leave it empty. A value may be an int (the Java
    settings read it via `getLong`/`getInt`) or a list of ints (read via a
    `get*List`, e.g. associative_cache's `numOfBuckets`); the command builder
    emits scalars as `key=value` and lists in indexed `key.0=…` form.
    `policy_key` is the config key holding the eviction-policy list, or None
    when the subject has none.
    """

    config_prefix: str
    policy_spec_name: str
    geometry: dict[str, int | list[int]]
    policy_key: str | None
    supports_log_path: bool

    def as_runner_kwargs(self) -> dict:
        """The caffeine-runner wiring kwargs (config_prefix, policy_spec_name, geometry, policy_key, supports_log_path)."""
        return {
            "config_prefix": self.config_prefix,
            "policy_spec_name": self.policy_spec_name,
            "geometry": self.geometry,
            "policy_key": self.policy_key,
            "supports_log_path": self.supports_log_path,
        }


@dataclass(frozen=True)
class Bench:
    """The subject's standalone microbench crate (per-op-time fitness)."""

    binary_name: str


@dataclass(frozen=True)
class Metadata:
    """The subject's metadata-size measurement binary (metadata-size fitness)."""

    binary_name: str


@dataclass(frozen=True)
class Subject:
    subject_dir: Path
    name: str
    crate_path: Path
    cdylib_name: str
    extension_points: tuple[ExtensionPoint, ...]
    gates: tuple[SubjectGate, ...]
    protected_files: tuple[Path, ...]
    caffeine: CaffeineConfig
    docs: Docs
    evaluation: Evaluation
    bench: Bench | None
    metadata: Metadata | None

    @property
    def ep_scope_whitelist(self) -> frozenset[Path]:
        return frozenset(path for ep in self.extension_points for path in ep.impl_files)

    @property
    def axis_descriptions(self) -> dict[str, str]:
        return {ep.name: ep.description for ep in self.extension_points}


_ALLOWED_TOP = {"subject", "extension-points", "gates", "contract", "caffeine", "docs", "evaluation", "bench", "metadata"}
_ALLOWED_SUBJECT = {"name", "crate-path", "cdylib-name"}
_ALLOWED_EP = {"name", "contract", "impl-files", "description"}
_ALLOWED_GATES = {"name", "test", "marker"}
_ALLOWED_CONTRACT = {"protected-files"}
_ALLOWED_CAFFEINE = {"config-prefix", "policy-spec-name", "policy-key", "supports-log-path", "geometry"}
_ALLOWED_DOCS = {"context", "insight"}
_ALLOWED_EVAL = {"fitness-vector", "traces", "sizes"}
_ALLOWED_BENCH = {"binary-name"}
_ALLOWED_METADATA = {"binary-name"}


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


# Cached: a subject dir is the immutable baseline source for the duration of a
# run, so its manifest never changes mid-process and reparsing on every hot-path
# call is wasted work.
@lru_cache(maxsize=None)
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

    for key in ("subject", "contract", "caffeine", "docs", "evaluation"):
        if key not in data:
            raise SubjectManifestError(f"manifest.toml: missing required table {key!r}")
    _reject_unknown(data, _ALLOWED_TOP, "manifest.toml")

    subject_table = data["subject"]
    ep_tables = data.get("extension-points", [])
    gate_tables = data.get("gates", [])
    contract_table = data["contract"]
    caffeine_table = data["caffeine"]
    docs_table = data["docs"]
    eval_table = data["evaluation"]
    bench_table = data.get("bench")
    metadata_table = data.get("metadata")

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

    if not isinstance(ep_tables, list):
        raise SubjectManifestError(
            "extension-points: must be an array of tables"
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

    if not isinstance(gate_tables, list):
        raise SubjectManifestError("gates: must be an array of tables")
    gates = tuple(_load_gate(gate, i) for i, gate in enumerate(gate_tables))
    gate_seen: dict[str, int] = {}
    for i, gate in enumerate(gates):
        if gate.name in gate_seen:
            raise SubjectManifestError(
                f"gates: duplicate name {gate.name!r} at indices {gate_seen[gate.name]} and {i}"
            )
        gate_seen[gate.name] = i

    _reject_unknown(contract_table, _ALLOWED_CONTRACT, "contract")
    protected_raw = _require(contract_table, "protected-files", "contract")
    if not isinstance(protected_raw, list):
        raise SubjectManifestError(
            f"contract.protected-files: must be a list of strings, got {type(protected_raw).__name__}"
        )
    if not protected_raw:
        raise SubjectManifestError("contract.protected-files: must be non-empty")
    protected_files: list[Path] = []
    for raw in protected_raw:
        path = _check_relative(raw, "contract.protected-files")
        _check_exists(subject_dir, path, "contract.protected-files")
        protected_files.append(path)

    _reject_unknown(docs_table, _ALLOWED_DOCS, "docs")
    context_raw = _require(docs_table, "context", "docs")
    insight_raw = _require(docs_table, "insight", "docs")
    context = _check_relative(context_raw, "docs.context")
    insight = _check_relative(insight_raw, "docs.insight")
    _check_exists(subject_dir, context, "docs.context")
    _check_exists(subject_dir, insight, "docs.insight")
    docs = Docs(context=context, insight=insight)

    caffeine = _load_caffeine(caffeine_table)

    evaluation = _load_evaluation(eval_table, subject_dir)

    bench = _load_bench(bench_table)
    metadata = _load_metadata(metadata_table)

    return Subject(
        subject_dir=subject_dir,
        name=name,
        crate_path=crate_path,
        cdylib_name=cdylib_name,
        extension_points=extension_points,
        gates=gates,
        protected_files=tuple(protected_files),
        caffeine=caffeine,
        docs=docs,
        evaluation=evaluation,
        bench=bench,
        metadata=metadata,
    )


def _load_caffeine(table: dict) -> CaffeineConfig:
    _reject_unknown(table, _ALLOWED_CAFFEINE, "caffeine")
    config_prefix = _require(table, "config-prefix", "caffeine")
    policy_spec_name = _require(table, "policy-spec-name", "caffeine")
    supports_log_path = _require(table, "supports-log-path", "caffeine")
    for field_name, value in (("config-prefix", config_prefix), ("policy-spec-name", policy_spec_name)):
        if not isinstance(value, str) or not value:
            raise SubjectManifestError(f"caffeine.{field_name}: must be a non-empty string")
    if not isinstance(supports_log_path, bool):
        raise SubjectManifestError("caffeine.supports-log-path: must be a boolean")
    policy_key = table.get("policy-key")
    if policy_key is not None and (not isinstance(policy_key, str) or not policy_key):
        raise SubjectManifestError("caffeine.policy-key: must be a non-empty string when present")
    geometry_raw = table.get("geometry", {})
    if not isinstance(geometry_raw, dict):
        raise SubjectManifestError(
            f"caffeine.geometry: must be a table, got {type(geometry_raw).__name__}"
        )
    geometry: dict[str, int | list[int]] = {}
    for key, value in geometry_raw.items():
        if isinstance(value, list):
            if not value or any(not isinstance(item, int) or isinstance(item, bool) for item in value):
                raise SubjectManifestError(f"caffeine.geometry.{key}: list must be non-empty integers")
            geometry[key] = list(value)
        elif isinstance(value, int) and not isinstance(value, bool):
            geometry[key] = value
        else:
            raise SubjectManifestError(f"caffeine.geometry.{key}: must be an integer or a list of integers")
    return CaffeineConfig(
        config_prefix=config_prefix,
        policy_spec_name=policy_spec_name,
        geometry=geometry,
        policy_key=policy_key,
        supports_log_path=supports_log_path,
    )


def _load_evaluation(table: dict, subject_dir: Path) -> Evaluation:
    _reject_unknown(table, _ALLOWED_EVAL, "evaluation")
    fv_raw = _require(table, "fitness-vector", "evaluation")
    fv = _check_relative(fv_raw, "evaluation.fitness-vector")
    _check_exists(subject_dir, fv, "evaluation.fitness-vector")

    traces_raw = _require(table, "traces", "evaluation")
    if not isinstance(traces_raw, list) or not traces_raw:
        raise SubjectManifestError("evaluation.traces: must be a non-empty list of strings")
    traces: list[str] = []
    for trace in traces_raw:
        fmt, sep, path = trace.partition(":") if isinstance(trace, str) else ("", "", "")
        if not isinstance(trace, str) or not sep or not fmt or not path:
            raise SubjectManifestError(
                f"evaluation.traces: each entry must be a '<format>:<path>' string, got {trace!r}"
            )
        traces.append(trace)

    sizes_raw = _require(table, "sizes", "evaluation")
    if not isinstance(sizes_raw, list) or not sizes_raw:
        raise SubjectManifestError("evaluation.sizes: must be a non-empty list of integers")
    sizes: list[int] = []
    for size in sizes_raw:
        if not isinstance(size, int) or isinstance(size, bool) or size <= 0:
            raise SubjectManifestError(
                f"evaluation.sizes: each entry must be a positive integer, got {size!r}"
            )
        if size in sizes:
            raise SubjectManifestError(f"evaluation.sizes: duplicate size {size}")
        sizes.append(size)

    _validate_baseline_grid(subject_dir, fv, traces, sizes)
    return Evaluation(fitness_vector=fv, traces=tuple(traces), sizes=tuple(sizes))


def _validate_baseline_grid(
    subject_dir: Path, fitness_vector: Path, traces: list[str], sizes: list[int]
) -> None:
    """The committed baseline anchor must be scored on the same (trace, size)
    grid as live variants, or the global champions and leaderboard silently
    compare incomparable numbers. Catch a stale or stub anchor at load time."""

    where = "evaluation.fitness-vector"
    try:
        data = json.loads((subject_dir / fitness_vector).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exception:
        raise SubjectManifestError(
            f"{where}: cannot read baseline fitness vector at {fitness_vector}: {exception}"
        ) from exception

    dimensions = data.get("dimensions") if isinstance(data, dict) else None
    if not isinstance(dimensions, list):
        raise SubjectManifestError(f"{where}: baseline fitness vector has no 'dimensions' list")
    hit_rate = next(
        (d for d in dimensions if isinstance(d, dict) and d.get("dim") == "hit-rate"), None
    )
    if hit_rate is None:
        raise SubjectManifestError(
            f"{where}: baseline fitness vector is missing the 'hit-rate' dimension"
        )
    values = hit_rate.get("values")
    if not isinstance(values, list):
        raise SubjectManifestError(f"{where}: baseline 'hit-rate' dimension has no 'values' list")

    measured: set[tuple[str, int]] = set()
    for entry in values:
        if not isinstance(entry, dict) or "trace" not in entry or "size" not in entry:
            raise SubjectManifestError(
                f"{where}: each baseline 'hit-rate' value must carry 'trace' and 'size' keys "
                f"matching the evaluation grid, got {entry!r}; re-measure with "
                f"scripts/measure_baseline.py"
            )
        measured.add((entry["trace"], entry["size"]))

    expected = {(trace, size) for trace in traces for size in sizes}
    if measured != expected:
        raise SubjectManifestError(
            f"{where}: baseline 'hit-rate' grid {sorted(measured)} does not match the declared "
            f"evaluation grid {sorted(expected)}; re-measure with scripts/measure_baseline.py"
        )


def _load_bench(table: dict | None) -> Bench | None:
    if table is None:
        return None
    _reject_unknown(table, _ALLOWED_BENCH, "bench")
    binary_name = _require(table, "binary-name", "bench")
    if not isinstance(binary_name, str) or not binary_name:
        raise SubjectManifestError("bench.binary-name: must be a non-empty string")
    return Bench(binary_name=binary_name)


def _load_metadata(table: dict | None) -> Metadata | None:
    if table is None:
        return None
    _reject_unknown(table, _ALLOWED_METADATA, "metadata")
    binary_name = _require(table, "binary-name", "metadata")
    if not isinstance(binary_name, str) or not binary_name:
        raise SubjectManifestError("metadata.binary-name: must be a non-empty string")
    return Metadata(binary_name=binary_name)


def _load_extension_point(ep: dict, index: int, subject_dir: Path) -> ExtensionPoint:
    where = f"extension-points[{index}]"
    _reject_unknown(ep, _ALLOWED_EP, where)
    name = _require(ep, "name", where)
    contract = _require(ep, "contract", where)
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
        name=name, contract=contract, impl_files=tuple(impl_files), description=description
    )


def _load_gate(gate: dict, index: int) -> SubjectGate:
    where = f"gates[{index}]"
    _reject_unknown(gate, _ALLOWED_GATES, where)
    name = _require(gate, "name", where)
    test = _require(gate, "test", where)
    for field_name, value in (("name", name), ("test", test)):
        if not isinstance(value, str) or not value:
            raise SubjectManifestError(f"{where}.{field_name}: must be a non-empty string")
    marker = gate.get("marker")
    if marker is not None and (not isinstance(marker, str) or not marker):
        raise SubjectManifestError(f"{where}.marker: must be a non-empty string when present")
    return SubjectGate(name=name, test=test, marker=marker)


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
