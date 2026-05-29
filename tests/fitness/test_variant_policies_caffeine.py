"""End-to-end caffeine smoke across multiple eviction policies.

Verifies that every Rust `EvictionPolicy` name flowing through the JNI
cdylib produces a valid hit_rate row. Catches regressions where the
Java-side policy list or the JNI signature drifts from Rust's canonical
names. Gated by `caffeine_needed`.

Runs a single caffeine invocation with all policies at once — caffeine
emits one CSV row per (size, policy), and the cdylib is policy-agnostic
(policy is selected at cache_init time via JNI).
"""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

import pytest

from scripts import caffeine_runner

from evolve.config import load_configs
from evolve.fitness.caffeine_simulator import (
	HARDCODED_SIZE,
	HARDCODED_TRACE,
	CaffeineSimulatorError,
	_parse_csv,
	_write_gradle_logs,
)
from evolve.identifiers import CycleId, VariantId
from evolve.paths import CycleVariant, Layout, SourcesLayout
from evolve.subject import load_subject
from evolve.variant.build import compile_cdylib

from _support import configured_subject

REPO_ROOT = Path(__file__).parent.parent.parent
FIXTURES_DIR = REPO_ROOT / "tests" / "fixtures"
SUBJECT = configured_subject(REPO_ROOT)

# One policy per family: recency (with movement), frequency, insertion-order.
POLICIES = ["ClassicLRU", "Lfu", "Fifo"]


def _stage_variant(tmp_path: Path) -> tuple[Layout, CycleVariant]:
	variant = CycleVariant(CycleId(1), VariantId(1))
	layout = Layout(repo_root=tmp_path / "repo", outputs_root=tmp_path / "out", subject=SUBJECT)
	layout.repo_root.mkdir(parents=True, exist_ok=True)
	real = SourcesLayout(root=REPO_ROOT, subject=SUBJECT)
	layout.sources.subject_dir.parent.mkdir(parents=True, exist_ok=True)
	layout.sources.subject_dir.symlink_to(real.subject_dir)
	layout.sources.framework_crate_dir.symlink_to(real.root / "framework-rs")
	shutil.copy(real.config_toml, layout.sources.config_toml)
	variant_dir = layout.outputs.cycle(variant.cycle_id).variant(variant.variant_id)
	# Empty overlay (the `good` fixture's shape) — baseline-equivalent build.
	variant_dir.files_dir.mkdir(parents=True, exist_ok=True)
	return layout, variant


@pytest.mark.caffeine_needed
def test_variant_policies_reach_caffeine_end_to_end(tmp_path):
	layout, variant = _stage_variant(tmp_path)
	cfg = load_configs(layout.repo_root)

	variant_dir = variant.resolve(layout)
	caffeine_out = variant_dir.caffeine_dir
	caffeine_out.root.mkdir(parents=True, exist_ok=True)
	caffeine_out.rust_logs_dir.mkdir(parents=True, exist_ok=True)

	try:
		workspace, cdylib_path = compile_cdylib(variant, layout=layout, repo_root=REPO_ROOT)
	except (RuntimeError, subprocess.CalledProcessError) as exception:
		raise CaffeineSimulatorError(f"cdylib build failed: {exception}") from exception

	try:
		caffeine = load_subject(layout.sources.subject_dir).caffeine
		try:
			run_result = caffeine_runner.run(
				library_path=cdylib_path.parent,
				trace=HARDCODED_TRACE,
				size=HARDCODED_SIZE,
				policies=[p.lower() for p in POLICIES],
				timeout_s=cfg.simulator.timeout_s,
				config_prefix=caffeine.config_prefix,
				policy_spec_name=caffeine.policy_spec_name,
				geometry=caffeine.geometry,
				policy_key=caffeine.policy_key,
				supports_log_path=caffeine.supports_log_path,
				debug_mode=0,
				log_path=caffeine_out.rust_logs_dir,
				output_dir=caffeine_out.root,
			)
		except caffeine_runner.CaffeineError as exception:
			_write_gradle_logs(caffeine_out, stdout="", stderr=str(exception))
			raise

		_write_gradle_logs(caffeine_out, stdout=run_result.stdout, stderr=run_result.stderr)
		csv_path = caffeine_out.hit_rate_csv(HARDCODED_SIZE)
		assert csv_path.is_file(), f"caffeine did not produce {csv_path}"
		results = _parse_csv(csv_path, trace=HARDCODED_TRACE, size=HARDCODED_SIZE)
	finally:
		shutil.rmtree(workspace.root, ignore_errors=True)

	assert len(results) == len(POLICIES), (
		f"expected one row per policy ({POLICIES}), got {[r.policy for r in results]}"
	)
	reported = [r.policy for r in results]
	for policy in POLICIES:
		matches = [r for r in results if policy.lower() in r.policy.lower()]
		assert len(matches) == 1, (
			f"policy={policy} not found exactly once in CSV rows {reported}"
		)
		row = matches[0]
		assert row.trace == HARDCODED_TRACE
		assert row.size == HARDCODED_SIZE
		assert row.requests > 0
		assert 0.0 < row.hit_rate <= 1.0, (
			f"policy={policy} hit_rate={row.hit_rate} outside (0, 1]"
		)

	rust_logs = list(caffeine_out.rust_logs_dir.glob(f"{load_subject(layout.sources.subject_dir).cdylib_name}_*.log"))
	assert rust_logs, f"no rust JNI log under {caffeine_out.rust_logs_dir}"
