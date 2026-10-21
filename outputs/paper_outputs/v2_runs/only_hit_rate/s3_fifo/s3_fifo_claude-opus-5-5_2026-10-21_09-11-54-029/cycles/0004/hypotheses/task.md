# Task: hypotheses-writer (cycle 0004)

Cycle: 0004
Number of hypotheses: 3
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0004/hypotheses

## Framing

You are proposing the next batch of 3 hypotheses. Each hypothesis MUST cite exactly one parent variant: either `baseline` or a past variant `cycle/<NNNN>/v<NNN>` from any prior cycle (1..3). Aim for distinct strategies — near-duplicates collapse the cohort's winners.

## Authoring scope

Any overlay file may be edited, except the gate-integrity surface, writable by no scope (src/s3_fifo/s3_fifo_jni.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/, src/bin/). Do not propose ideas that require editing it.

## Incumbents (cross-cycle winners)

- cycle/0003/v003: hit-rate=0.7289

At least one of the hypotheses MUST cite a cross-cycle winner above as its `## Parent`. Prefer the non-baseline incumbents — they are the proven best-so-far. The remaining hypotheses may explore from `baseline` or prior losers.

## Read for context

### Previous cycle (0003)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0003/variants — every variant directory `v<NNN>/` under this path. For each variant read:
  - /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0003/variants/v<NNN>/attempts/<NN>/overlay — the variant's overlay (subject files it replaced); read its latest attempt.
  - /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0003/variants/v<NNN>/eval/fitness-vector.json — its measured fitness vector.
  - /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0003/variants/v<NNN>/insight.md — one-line fitness summary.
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0003/cycle-winners.json — Winners + per-variant fitness vectors for the cycle.
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0003/cycle-insight.md — cycle-level summary.

### Cross-cycle history

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/leaderboard.md — current winner variants and fitness summaries across all cycles.
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles-ledger.jsonl — one JSON row per completed cycle (cohort size, winners, insights).
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cross-cycle-insights.md — long-running narrative of what has been learned across cycles. Read if present.

### Baseline

- /home/user/cache-evolve/inputs/subjects/s3_fifo/baseline — the permanent baseline (always a valid parent variant).
