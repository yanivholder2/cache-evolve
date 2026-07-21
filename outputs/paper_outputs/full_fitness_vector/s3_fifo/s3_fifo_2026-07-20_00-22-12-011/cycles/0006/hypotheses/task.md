# Task: hypotheses-writer (cycle 0006)

Cycle: 0006
Number of hypotheses: 3
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0006\hypotheses

## Framing

You are proposing the next batch of 3 hypotheses. Each hypothesis MUST cite exactly one parent variant: either `baseline` or a past variant `cycle/<NNNN>/v<NNN>` from any prior cycle (1..5). Aim for distinct strategies — near-duplicates collapse the cohort's winners.

## Authoring scope

Any overlay file may be edited, except the gate-integrity surface, writable by no scope (src\s3_fifo\s3_fifo_jni.rs, Cargo.toml, src\lib.rs, tests/, src\test_utils/, src\bin/). Do not propose ideas that require editing it.

## Incumbents (cross-cycle winners)

- cycle/0005/v001: hit-rate=0.7292, metadata-size=8, per-op-time=0
- cycle/0005/v002: hit-rate=0.7292, metadata-size=16, per-op-time=0
- cycle/0005/v003: hit-rate=0.7279, metadata-size=8, per-op-time=100

At least one of the hypotheses MUST cite a cross-cycle winner above as its `## Parent`. Prefer the non-baseline incumbents — they are the proven best-so-far. The remaining hypotheses may explore from `baseline` or prior losers.

## Read for context

### Previous cycle (0005)

- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0005\variants — every variant directory `v<NNN>/` under this path. For each variant read:
  - C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0005\variants\v<NNN>\attempts\<NN>\overlay — the variant's overlay (subject files it replaced); read its latest attempt.
  - C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0005\variants\v<NNN>\eval\fitness-vector.json — its measured fitness vector.
  - C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0005\variants\v<NNN>\insight.md — one-line fitness summary.
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0005\cycle-winners.json — Winners + per-variant fitness vectors for the cycle.
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0005\cycle-insight.md — cycle-level summary.

### Cross-cycle history

- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\leaderboard.md — current winner variants and fitness summaries across all cycles.
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles-ledger.jsonl — one JSON row per completed cycle (cohort size, winners, insights).
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cross-cycle-insights.md — long-running narrative of what has been learned across cycles. Read if present.

### Baseline

- C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\s3_fifo\baseline — the permanent baseline (always a valid parent variant).
