# plots

Aggregate run outputs into figures.

## Pipeline

1. **`build_data.py`** — walks `outputs/`, writes `data.json`. Stdlib only,
   idempotent (rebuilds from scratch). One row per variant per cycle, plus a
   baseline row per run when `champions.json` recorded one. Any fitness
   dimension absent from a variant's vector is written as `-1`.

   ```sh
   .venv/bin/python plots/build_data.py
   ```

2. **`plot.py`** — renders PNGs from `data.json`. Needs matplotlib/pandas/numpy,
   which are not project deps; run them ephemerally:

   ```sh
   uv run --no-project --with matplotlib --with pandas --with numpy python plots/plot.py
   ```

## Where setup comes from

Each run folder is self-describing: the runner copies `config.toml` and the
subject's `manifest.toml` into the run dir at start (see
`_snapshot_run_inputs` in `src/evolve/cli.py`). `build_data.py` reads the run
setup from that `config.toml` snapshot. Cycle count and cohort size come from
authoritative run artifacts (cycle dirs, ledger) and always win. Runs predating
the snapshot fall back to those artifacts, with the rest left as `"unknown"` / `-1`.

## Files

| File            | Role                                                                       |
| --------------- | -------------------------------------------------------------------------- |
| `build_data.py` | Aggregator: `outputs/` into `data.json`.                                   |
| `data.json`     | Generated. Real per-variant fitness data; `-1` = dimension not calculated. |
| `plot.py`       | Renders evolution lines (cycle winners + baseline) and final-cycle bars.   |
| `*.png`         | Generated figures (gitignored).                                            |

## Adding a run

`build_data.py` picks up every complete run dir (one with `cycles-ledger.jsonl`)
automatically. New runs carry their own `config.toml` snapshot, so the setup
block fills in with no manual step.

Canonical fitness dimensions: `hit-rate`, `metadata-size`, `per-op-time`.
