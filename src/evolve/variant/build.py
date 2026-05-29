"""Variant build harness: materialise a self-contained cargo workspace for a
variant by overlaying its `overlay/` tree on a copy of the subject crate, so
`cargo build`/`cargo test` can run from any cwd."""

from __future__ import annotations

import logging
import re
import shutil
import subprocess
from pathlib import Path
from tempfile import mkdtemp

from evolve._io import read_utf8, write_utf8
from evolve.paths import Layout, Variant, CargoWorkspace
from evolve.subject import load_subject

log = logging.getLogger("build")


def _rewrite_framework_path(cargo_toml: Path, framework_crate_dir: Path) -> None:
    """Rewrite the `framework = { path = "..." }` dependency to an absolute path.

    The subject crate's checked-in `Cargo.toml` uses a relative path
    (`../../../framework-rs`) that only resolves inside the repo tree. After
    copytree into a tempdir, the relative path is broken, so we replace the
    `path = "<anything>"` segment of the framework dep line(s) with an absolute
    path pointing at the in-repo framework crate."""

    text = read_utf8(cargo_toml)
    abs_path = framework_crate_dir.as_posix()
    pattern = re.compile(
        r'(framework\s*=\s*\{[^}]*\bpath\s*=\s*")[^"]*(")', re.MULTILINE
    )
    new_text, count = pattern.subn(rf'\g<1>{abs_path}\g<2>', text)
    if count == 0:
        raise RuntimeError(
            f"could not locate `framework = {{ path = ... }}` in {cargo_toml}"
        )
    write_utf8(cargo_toml, new_text)


def _apply_overlay(overlay_dir: Path, workspace_root: Path) -> None:
    """Copy every file under `overlay_dir` onto `workspace_root` at the matching
    relative path. Missing/empty `overlay_dir` is a no-op, meaning "no overlay"
    (a baseline-equivalent build)."""

    if not overlay_dir.is_dir():
        return
    for src in overlay_dir.rglob("*"):
        if not src.is_file():
            continue
        rel = src.relative_to(overlay_dir)
        dst = workspace_root / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy(src, dst)


def prepare_variant_workspace(
    variant: Variant,
    *,
    layout: Layout,
    repo_root: Path,
) -> CargoWorkspace:
    """Materialise a cargo workspace for `variant`: subject copytree → overlay
    `overlay/` → rewrite the framework dep path to absolute. Returns a `CargoWorkspace`
    rooted at a fresh tempdir; the caller owns cleanup."""

    variant_dir = variant.resolve(layout)
    subject_dir = layout.sources.subject_dir
    subject = load_subject(subject_dir)
    framework_crate_dir = layout.sources.framework_crate_dir

    def _ignore_build_cache(directory: str, names: list[str]) -> set[str]:
        # Skip the subject's build cache: it's gitignored-but-present after any
        # local cargo build, and copying it is gigabytes of redundant I/O per
        # variant. Anchored to the crate root so a "target" segment deeper in
        # the tree (fixtures, test data) is never silently dropped.
        if Path(directory) == subject_dir and "target" in names:
            return {"target"}
        return set()

    root = Path(mkdtemp(prefix="variant-workspace-"))
    try:
        # mkdtemp already created root, so copytree must merge into it.
        shutil.copytree(subject_dir, root, dirs_exist_ok=True, ignore=_ignore_build_cache)
        _apply_overlay(variant_dir.overlay_dir, root)
        _rewrite_framework_path(root / "Cargo.toml", framework_crate_dir)
    except BaseException:
        shutil.rmtree(
            root,
            onexc=lambda fn, path, exception: log.warning(
                "workspace cleanup failed for %s: %s", path, exception
            ),
        )
        raise

    log.debug("workspace=%s subject=%s repo_root=%s", root, subject_dir, repo_root)
    return CargoWorkspace(root=root, cdylib_name=subject.cdylib_name)


def compile_for_gates(
    variant: Variant,
    *,
    layout: Layout,
    repo_root: Path,
) -> CargoWorkspace:
    """Prepare a workspace and run `cargo build --tests --features test-utils`
    against it. Returns the workspace (caller owns cleanup).

    Raises:
        subprocess.CalledProcessError: cargo build failed (build output on the
            exception's stdout/stderr, workspace removed). The build gate turns
            this into a rejection entry.
    """

    workspace = prepare_variant_workspace(variant, layout=layout, repo_root=repo_root)
    try:
        log.info("build gate: compiling variant %s ...", variant)
        proc = subprocess.run(
            [
                "cargo", "build", "--tests", "--features", "test-utils",
                "--manifest-path", str(workspace.cargo_toml),
            ],
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            log.info("build gate: FAILED")
            raise subprocess.CalledProcessError(
                proc.returncode, proc.args, proc.stdout, proc.stderr
            )
    except BaseException:
        shutil.rmtree(
            workspace.root,
            onexc=lambda fn, path, exception: log.warning(
                "workspace cleanup failed for %s: %s", path, exception
            ),
        )
        raise

    log.info("build gate: passed")
    return workspace


def compile_cdylib(
    variant: Variant, *, layout: Layout, repo_root: Path
) -> tuple[CargoWorkspace, Path]:
    """Prepare a workspace and run `cargo build --release` to produce a
    Caffeine-loadable cdylib.

    Returns `(workspace, cdylib_path)`. The caller owns the workspace's
    lifecycle and is responsible for cleaning it up once the artifact is no
    longer needed.

    Raises:
        RuntimeError: platform is unsupported.
        subprocess.CalledProcessError: cargo build failed (workspace removed).
    """

    workspace = prepare_variant_workspace(variant, layout=layout, repo_root=repo_root)
    try:
        log.info("building cdylib for variant %s ...", variant)
        proc = subprocess.run(
            ["cargo", "build", "--release", "--manifest-path", str(workspace.cargo_toml)],
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            log.error("cdylib build FAILED for %s\n%s", variant, proc.stdout + proc.stderr)
            raise subprocess.CalledProcessError(
                proc.returncode, proc.args, proc.stdout, proc.stderr
            )
    except BaseException:
        shutil.rmtree(
            workspace.root,
            onexc=lambda fn, path, exception: log.warning(
                "workspace cleanup failed for %s: %s", path, exception
            ),
        )
        raise

    cdylib_path = workspace.cdylib
    log.info("cdylib built at %s", cdylib_path)
    return workspace, cdylib_path
