# plots

Aggregate run outputs into figures.

## Pipeline

Paper runs live under `outputs/paper_outputs/<group>/<subject>/<run>/`, where
`<group>` is the fitness configuration the runs share (`full_fitness_vector`,
`only_hit_rate`). Each group is its own dataset.

1. **`build_data.py`** — walks `outputs/paper_outputs/`, writes one
   `paper-plots/data_<group>.json` per group. Stdlib only, idempotent
   (rebuilds from scratch). One row per variant per cycle, plus a baseline row
   per run when its leaderboard (or `baseline-vs-winner-fitness-diff.md`)
   recorded one. Any fitness dimension absent from a variant's vector is
   written as `-1`. Pass group names to rebuild only those groups.

   ```sh
   .venv/bin/python plots/build_data.py            # every group
   .venv/bin/python plots/build_data.py only_hit_rate
   ```

2. **`plot.py`** — renders PNGs from both `paper-plots/data_<group>.json` files
   into `paper-plots/<group>/`. Needs matplotlib/pandas/numpy, which are not
   project deps; run them ephemerally from the repo root:

   ```sh
   uv run --no-project --with matplotlib --with pandas --with numpy python plots/plot.py
   ```

   Final-cycle bar charts get one bar per model per subject, ordered by family
   (Opus, Sonnet, Haiku, then any other family) and newest version first within
   a family; unparseable model ids sort last. The agent tool is added to the
   label only when runs in a group used different tools. The cycle-evolution
   line charts are one line per subject, averaged across that subject's runs.

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
| `build_data.py`                        | Aggregator: `outputs/paper_outputs/<group>/` into `paper-plots/data_<group>.json`. |
| `paper-plots/data_<group>.json`        | Generated. Real per-variant fitness data; `-1` = dimension not calculated. |
| `plot.py`                              | Renders evolution lines (cycle winners + baseline) and final-cycle bars.   |
| `paper-plots/<group>/*.png`            | Generated figures (gitignored).                                            |
| `data.json`                            | Legacy output of the old single-dataset pipeline; no longer written.       |

## Adding a run

`build_data.py` picks up every complete run dir (one with `cycles-ledger.jsonl`)
under `outputs/paper_outputs/<group>/<subject>/` automatically. New runs carry
their own `config.toml` snapshot, so the setup block fills in with no manual
step.

Canonical fitness dimensions: `hit-rate`, `metadata-size`, `per-op-time`.
