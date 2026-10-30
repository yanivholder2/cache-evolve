# plots

Aggregate run outputs into figures.

## Pipeline

Paper runs live under `outputs/paper_outputs/<batch>_runs/<group>/<subject>/<run>/`,
where the batch is any `*_runs` directory (`v1_runs/` for the original runs, `v2_runs/` for
the Claude 5.5 runs) and `<group>` is the fitness configuration the runs share
(`full_fitness_vector`, `only_hit_rate`). Each group is its own
dataset for one evaluation profile spanning every batch. Each run's setup
records its `batch` and `evaluation-profile`; batch alone does not identify a workload.

1. **`build_data.py`** — walks `outputs/paper_outputs/`, writes one
   `paper-plots/data_<group>.json` per group. Stdlib only, idempotent
   (rebuilds from scratch). One row per variant per cycle, plus a baseline row
   per run when its leaderboard (or `baseline-vs-winner-fitness-diff.md`)
   recorded one. Any fitness dimension absent from a variant's vector is
   written as `-1`. Pass group names to rebuild only those groups.

   ```sh
   .venv/bin/python plots/build_data.py            # every group, default Zipf profile
   .venv/bin/python plots/build_data.py only_hit_rate
   .venv/bin/python plots/build_data.py --evaluation-profile msr
   ```

2. **`plot.py`** — renders PNGs from both `paper-plots/data_<group>.json` files
   into `paper-plots/<group>/`. Needs matplotlib/pandas/numpy, which are not
   project deps; run them ephemerally from the repo root:

   ```sh
   uv run --no-project --with matplotlib --with pandas --with numpy python plots/plot.py
   uv run --no-project --with matplotlib --with pandas --with numpy python plots/plot.py --evaluation-profile msr
   ```

   Final-cycle bar charts get one bar per model per subject, ordered by family
   (Opus, Sonnet, Haiku, then any other family) and newest version first within
   a family; unparseable model ids sort last. The agent tool is added to the
   label only when runs in a group used different tools. The cycle-evolution
   line charts are one line per subject, averaged across that subject's runs.

## Workload comparisons

Both scripts select exactly one profile. With no flag, existing v1/v2 Zipf
filenames, numerical data, aggregation and figure titles remain unchanged.
Named profiles use `paper-plots/profiles/profile-<URL-encoded-name>/` for both
`data_<group>.json` and `<group>/*.png`; MSR titles also identify the profile.
There is no combined-profile mode. Filtering happens before both line and bar
aggregation, including when reading stale or manually combined JSON.

The JSON records `evaluation-profile` at dataset and run-setup level. The empty
string means the unnamed default grid (Zipf in these experiments), and `msr`
means the MSR grid; it is copied from `[fitness].evaluation_profile` in the
run snapshot. An absent key in an existing snapshot also means default, as in
the runner. Missing snapshots are inferred as default only for historical
`v1_runs`/`v2_runs`; other batches get `null` and are excluded from comparisons.
Restore their authoritative snapshots before including them. The builder
reports excluded run counts and overwrites even an empty selection, avoiding
stale data. Old JSON without profile fields is accepted only for known v1/v2
batches; rebuild older files without batch metadata.

Compare winners with the baseline recorded in the same run/profile. Raw hit
rates and baseline-relative weighted scores must not be pooled across Zipf
and MSR. Separate figures support within-workload comparisons; claims about
transfer require rescoring the same variants on a common trace/size grid.
Profile names assume fixed grids; use distinct names for different grids.

Regression checks (renderer check skips without optional plotting packages):

```sh
.venv/bin/python -m unittest discover -s tests -p test_plots_profiles.py -v
```

## Where setup comes from

Each run folder is self-describing: the runner copies `config.toml` and the
subject's `manifest.toml` into the run dir at start (see
`snapshot_run_inputs` in `src/evolve/run_snapshot.py`). CLI flags that won over
config.toml are overlaid onto the `config.toml` snapshot so it reflects the
values the run actually used. `build_data.py` reads the run setup from that
`config.toml` snapshot. Cycle count and cohort size come from
authoritative run artifacts (cycle dirs, ledger) and always win. The model
comes from the run dir name `<subject>_<model>_<YYYY-MM-DD>_<HH-MM-SS-mmm>`,
which is authoritative (some older snapshots have a wrong or missing
`[framework].model`); the snapshot's `model` is only a fallback for names that
do not parse. Runs predating
the snapshot fall back to those artifacts, with the rest left as `"unknown"` / `-1`.

## Files

| File                                   | Role                                                                       |
| -------------------------------------- | -------------------------------------------------------------------------- |
| `build_data.py`                        | Aggregator: all `*_runs/<group>/`, filtered by evaluation profile. |
| `paper-plots/data_<group>.json`        | Generated. Real per-variant fitness data; `-1` = dimension not calculated. |
| `plot.py`                              | Renders evolution lines (cycle winners + baseline) and final-cycle bars.   |
| `paper-plots/<group>/*.png`            | Generated figures (gitignored).                                            |
| `data.json`                            | Legacy output of the old single-dataset pipeline; no longer written.       |

## Adding a run

`build_data.py` picks up every complete run dir (one with `cycles-ledger.jsonl`)
under `outputs/paper_outputs/*_runs/<group>/<subject>/` automatically. New runs carry
their own `config.toml` snapshot, so the setup block fills in with no manual
step.

Canonical fitness dimensions: `hit-rate`, `metadata-size`, `per-op-time`.

