# Baseline

Every subject carries a checked-in **baseline**: the subject as it stands today, with its measured fitness recorded alongside. The baseline is the anchor that cycle-1 hypotheses ground their expectations in, and the reference point the cross-cycle winners are compared against.

## What the baseline is

The baseline implementation *is* the subject crate's own source (an empty overlay — no variant edits). The baseline directory holds its measured grounding:

```
inputs/subjects/<name>/baseline/
  eval/fitness-vector.json   # the measured fitness anchor
  insight.md                 # short prose describing the baseline
```

- `fitness-vector.json` is produced by re-running the same scoring used for live variants, so the numbers are directly comparable to a cohort's.
- `insight.md` is checked-in prose (declared in the subject's manifest), not machine-generated.

## Measuring (and re-measuring) the baseline

`fitness-vector.json` is a committed artifact. Regenerate it — and commit the diff — whenever something that affects the measurement changes (for example, the benchmark configuration or the set of fitness plugins), so that cohorts are always compared against a current anchor:

```bash
uv run python scripts/measure_baseline.py
```

The script builds the subject with an empty overlay, runs the correctness [gates](Concepts#gates) (aborting if the baseline itself fails them), and then computes fitness exactly as a live variant would — writing `eval/fitness/*.json` and the aggregated `eval/fitness-vector.json`.

To measure a baseline for a specific subject, select it the usual way (via `--subject` / `config.toml`); the script measures whichever subject the configuration resolves to.

## Evaluation profiles

A manifest may declare named alternative grids, e.g. `[evaluation.profiles.msr]` with `traces` and/or `sizes` overriding `[evaluation]`. A run opts in with `--evaluation-profile msr` (or `fitness.evaluation_profile` in `config.toml`); without it the default grid and anchor are used. Each profile has its own anchor at `baseline/eval/profiles/<name>/fitness-vector.json`, created with:

```bash
.venv/bin/python scripts/measure_baseline.py <subject> --evaluation-profile <name>
```

## Going deeper

- [`scripts/measure_baseline.py`](https://github.com/yanivholder2/cache-evolve/blob/main/scripts/measure_baseline.py) — the measurement script.
- [Evaluation and Fitness](Evaluation-and-Fitness) — how the fitness vector is produced.
- A subject's `manifest.toml` `[evaluation]` and `[docs]` tables — where the baseline artifacts are declared.
