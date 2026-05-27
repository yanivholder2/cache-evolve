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
	TARGET,
	CaffeineSimulatorError,
	_parse_csv,
	_write_gradle_logs,
)
from evolve.identifiers import CycleId, VariantId
from evolve.paths import CycleVariant, Layout, SourcesLayout, VariantDir
from evolve.variant.build import build_cdylib

REPO_ROOT = Path(__file__).parent.parent.parent
FIXTURES_DIR = REPO_ROOT / "tests" / "fixtures"

# One policy per family: recency (with movement), frequency, insertion-order.
POLICIES = ["ClassicLRU", "Lfu", "Fifo"]


def _stage_variant(tmp_path: Path) -> tuple[Layout, CycleVariant]:
	variant = CycleVariant(CycleId(1), VariantId(1))
	layout = Layout(repo_root=tmp_path / "repo", outputs_root=tmp_path / "out")
	layout.repo_root.mkdir(parents=True, exist_ok=True)
	layout.sources.rust_scaffold_dir.parent.mkdir(parents=True, exist_ok=True)
	layout.sources.rust_scaffold_dir.symlink_to(SourcesLayout(root=REPO_ROOT).rust_scaffold_dir)
	shutil.copy(SourcesLayout(root=REPO_ROOT).config_toml, layout.sources.config_toml)
	variant_dir = layout.outputs.cycle(variant.cycle_id).variant(variant.variant_id)
	shutil.copytree(VariantDir(root=FIXTURES_DIR / "good").impl_dir, variant_dir.impl_dir)
	# Manifest policy is unused here (we pass policies directly to caffeine),
	# but the file must exist for downstream tooling that may read it.
	variant_dir.manifest_toml.write_text(
		f'[variant]\neviction_policy = "{POLICIES[0]}"\n'
	)
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
		workspace, cdylib_path = build_cdylib(variant, layout=layout, repo_root=REPO_ROOT)
	except (RuntimeError, subprocess.CalledProcessError) as exception:
		raise CaffeineSimulatorError(f"cdylib build failed: {exception}") from exception

	try:
		try:
			run_result = caffeine_runner.run(
				target=TARGET,
				library_path=cdylib_path.parent,
				trace=HARDCODED_TRACE,
				size=HARDCODED_SIZE,
				policies=[p.lower() for p in POLICIES],
				timeout_s=cfg.simulator.timeout_s,
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

	rust_logs = list(caffeine_out.rust_logs_dir.glob("dash_variant_*.log"))
	assert rust_logs, f"no rust JNI log under {caffeine_out.rust_logs_dir}"
