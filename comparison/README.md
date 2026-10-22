# cache-evolve vs OpenEvolve

Runs both frameworks on the same subject with the same budget, then scores every
candidate with the same code.

```bash
uv pip install -r comparison/requirements.txt        # openevolve + matplotlib, once
python -m comparison.run_comparison --subject lru_cache --budget 30 --cohort-size 3 \
    --model claude-opus-4-8 --repeats 3
```

Output lands in `comparison_runs/<subject>-<timestamp>/` (gitignored):
`summary.json`, `candidates.csv`, `trajectory.png`, plus each arm's raw run
under `evolve/rep<k>/` and `openevolve/rep<k>/`. Useful flags: `--dry-run` prints
the commands, `--collect-only --out DIR` re-summarises a finished run,
`--arms openevolve` runs one arm, `--skip-rescore` reports in-loop scores only.

Prerequisites are the same as a normal `evolve` run (Caffeine checkout, JDK 25,
`cargo`; see the main README) and a logged-in `claude` CLI.
A full lru_cache run of 30 candidates per arm takes a few hours; the first
simulator call also builds Caffeine (~6 min).

## What is held equal

| | cache-evolve | OpenEvolve |
|---|---|---|
| Subject, seed program | `inputs/subjects/<subject>` | the same crate's `src/<subject>/mod.rs` |
| LLM | `--model` via `claude-sdk` | the same `--model` via OpenEvolve's `claude_code` backend |
| Budget | `budget/cohort` cycles of `cohort` variants | `budget` iterations, one child each |
| Gates | build, contract, universal and subject invariants | the same functions (`run_gates`) |
| Fitness | hit rate, metadata size, per-op time | the same functions (`compute_fitness`) |
| Score | `weighted` selector, `config.toml` weights, checked-in baseline | the same `WeightedSelector` |
| Task context | subject `CONTEXT.md` and baseline insight, objective weights | the same text in OpenEvolve's system prompt |

The score is the weighted fractional improvement over the baseline; 0 is the
baseline and only a positive score is a win. A failed candidate scores -1.

## What differs, by design

Each framework is run as published, so these differences are the thing under test:
cache-evolve's hypothesis step, self-repair loop, insight ledger, Pareto/weighted
selection and agent tools (Read/Glob/Grep/Write/Edit) versus OpenEvolve's
single-shot SEARCH/REPLACE mutation, island/MAP-Elites database and artifact
feedback. Failures reach OpenEvolve as artifacts (the gate's rejection text), the
counterpart of cache-evolve's self-repair prompt.

## Fairness caveats you should state when reporting

- **Budget unit.** Parity is on candidate programs evaluated, not on LLM calls.
  cache-evolve spends more calls per candidate (hypothesis, up to 3 implementation
  attempts, insights); OpenEvolve spends one. Wall-clock is plotted for that
  reason, but token cost is only logged for cache-evolve (OpenEvolve's CLI backend
  does not report it). Compare cost from your Anthropic usage page if it matters.
- **One file.** OpenEvolve evolves a single file, so the default subjects are the
  ones whose logic lives in `src/<subject>/mod.rs` (`lru_cache`, `s3_fifo`,
  `tiny_lfu`). cache-evolve can write any overlay file; `summary.json` counts
  `multi_file_candidates`, which cannot be re-scored and drop out of the
  re-scored comparison. For `dash` / `associative_cache` pass `--evolve-file`
  and treat the result as a restricted-surface comparison.
- **Timing noise.** Per-op time is measured in whole nanoseconds, so an unchanged
  program varies by about 1 ns (about 0.005 in score). Hence the **serial re-score**:
  every gate-passing candidate from both arms is re-evaluated one at a time with
  the same evaluator, and `summary.json` reports those numbers. cache-evolve runs
  with `--max-workers 1` for the same reason. Report differences smaller than
  ~0.01 as ties, and use `--repeats` (3+) because LLM runs are not reproducible;
  `--seed` only fixes OpenEvolve's sampler.
- **Isolation.** OpenEvolve's `claude -p` backend is a full Claude Code session, so
  it is launched from an empty scratch directory (`claude_code_llm.py`); otherwise
  it could read this repo and earlier results.
- **Baseline anchor.** Both arms score against the checked-in
  `baseline/eval/fitness-vector.json`. Re-measure it on the machine you run on
  (`scripts/measure_baseline.py`) if its per-op time is from different hardware.
