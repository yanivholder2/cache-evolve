"""Workload profiles must never become interchangeable plot observations."""

import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

PLOTS = Path(__file__).resolve().parent.parent / "plots"
sys.path.insert(0, str(PLOTS))
import build_data


class ProfileTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.outputs = self.root / "outputs"
        self.plots = self.root / "plots"
        for name, value in (("PAPER_OUTPUTS_DIR", self.outputs), ("PAPER_PLOTS_DIR", self.plots)):
            patcher = patch.object(build_data, name, value)
            patcher.start()
            self.addCleanup(patcher.stop)

    def make_run(self, batch, name, profile=None, snapshot=True, baseline=0.8):
        run = self.outputs / batch / "only_hit_rate" / "dash" / name
        vector = run / "cycles/0001/variants/v001/eval"
        vector.mkdir(parents=True)
        (vector / "fitness-vector.json").write_text(json.dumps({
            "dimensions": [{"dim": "hit-rate", "summary": baseline + 0.01}],
        }))
        (run / "cycles-ledger.jsonl").write_text(
            '{"cycle": 1, "cohort_size": 1, "winners": ["v001"]}\n'
        )
        (run / "leaderboard.md").write_text(
            f"| Cycle | hit-rate |\n| --- | --- |\n| baseline | {baseline} |\n"
        )
        if snapshot:
            config = '[framework]\nmodel = "claude-opus-5-5"\n[fitness]\n'
            if profile is not None:
                config += f'evaluation_profile = {json.dumps(profile)}\n'
            (run / "config.toml").write_text(config)
        return run

    def build(self, profile=""):
        groups = build_data.resolve_groups([])
        path = build_data.build_group("only_hit_rate", groups["only_hit_rate"], profile)
        return path, json.loads(path.read_text())

    def test_mixed_profiles_keep_legacy_values_and_baselines(self):
        self.make_run("v1_runs", "old", snapshot=False)
        self.make_run("v2_runs", "zipf")
        before_path, before = self.build()
        before_bytes = before_path.read_bytes()
        # Profile, not batch, determines inclusion in a comparison.
        self.make_run("snia_runs", "msr", "msr", baseline=0.5)
        self.make_run("v2_runs", "msr_in_versioned_batch", "msr", baseline=0.4)
        self.make_run("future_runs", "unidentified", snapshot=False)
        path, after = self.build()
        self.assertEqual(path, self.plots / "data_only_hit_rate.json")
        self.assertEqual(path.read_bytes(), before_bytes)
        self.assertEqual(after, before)
        msr_path, msr = self.build("msr")
        self.assertEqual(msr_path, self.plots / "profiles/profile-msr/data_only_hit_rate.json")
        self.assertEqual(len(msr["runs"]), 2)
        self.assertEqual({r["setup"]["evaluation-profile"] for r in msr["runs"]}, {"msr"})
        self.assertEqual({r["variants"][0]["hit-rate"] for r in msr["runs"]}, {0.4, 0.5})
        self.assertTrue(all(r["variants"][1]["is-winner"] for r in msr["runs"]))
        self.assertEqual({r["setup"]["batch"] for r in msr["runs"]}, {"v2_runs", "snia_runs"})

    def test_missing_and_empty_profiles(self):
        for batch, snapshot, profile, expected in (
            ("v1_runs", False, None, ""), ("v2_runs", False, None, ""),
            ("snia_runs", False, None, None), ("future_runs", True, None, ""),
            ("future_runs", True, "", ""), ("v1_runs", True, "msr", "msr"),
        ):
            with self.subTest(batch=batch, snapshot=snapshot, profile=profile):
                run = self.make_run(batch, str(len(list(self.outputs.rglob('config.toml')))) if self.outputs.exists() else 'first', profile, snapshot)
                self.assertEqual(build_data.run_setup(run, "dash", 1, batch)["evaluation-profile"], expected)

    def test_empty_selection_overwrites_stale_data(self):
        run = self.make_run("snia_runs", "msr", "msr")
        path, data = self.build("msr")
        self.assertEqual(len(data["runs"]), 1)
        (run / "config.toml").write_text('[fitness]\nevaluation_profile = "other"\n')
        self.assertEqual(self.build("msr")[1]["runs"], [])
        self.assertEqual(json.loads(path.read_text())["runs"], [])

    def test_profile_paths_cannot_escape_or_collide_with_default(self):
        for profile in ("../msr", "/tmp/msr", "default", ".", "a/b"):
            path = build_data.profile_directory(self.plots, profile)
            self.assertEqual(path.parent, self.plots / "profiles")
            self.assertNotEqual(path, self.plots)

    @unittest.skipUnless(
        all(importlib.util.find_spec(name) for name in ("matplotlib", "pandas", "numpy")),
        "optional plotting dependencies are not installed",
    )
    def test_renderer_filters_before_line_and_bar_aggregation(self):
        import matplotlib
        matplotlib.use("Agg")
        import plot

        zipf = self.make_run("v2_runs", "zipf", baseline=0.8)
        msr = self.make_run("snia_runs", "msr", "msr", baseline=0.5)
        unknown = self.make_run("future_runs", "unknown", snapshot=False, baseline=0.1)
        runs = [build_data.build_run(run, "dash", batch) for run, batch in (
            (zipf, "v2_runs"), (msr, "snia_runs"), (unknown, "future_runs"),
        )]
        # Simulate pre-fix JSON: known historical batches may lack this field.
        del runs[0]["setup"]["evaluation-profile"]
        data_file = self.root / "mixed.json"
        data_file.write_text(json.dumps({"runs": runs}))
        for profile, baseline in (("", 0.8), ("msr", 0.5)):
            with self.subTest(profile=profile), \
                    patch.object(plot.plt, "plot", wraps=plot.plt.plot) as lines, \
                    patch.object(plot.plt, "bar", wraps=plot.plt.bar) as bars, \
                    patch.object(plot.plt, "savefig"), \
                    patch("matplotlib.figure.Figure.savefig"):
                plot.generate_plots(data_file, self.root / "figures", profile)
                self.assertEqual(list(lines.call_args_list[0].args[1]), [baseline, baseline + 0.01])
                self.assertEqual(list(bars.call_args_list[0].args[1]), [baseline + 0.01])

    def test_cli_profile_selection(self):
        self.make_run("snia_runs", "msr", "msr")
        with patch.object(sys, "argv", ["build_data.py", "only_hit_rate", "--evaluation-profile", "msr"]):
            build_data.main()
        path = self.plots / "profiles/profile-msr/data_only_hit_rate.json"
        self.assertEqual(json.loads(path.read_text())["evaluation-profile"], "msr")


if __name__ == "__main__":
    unittest.main()
