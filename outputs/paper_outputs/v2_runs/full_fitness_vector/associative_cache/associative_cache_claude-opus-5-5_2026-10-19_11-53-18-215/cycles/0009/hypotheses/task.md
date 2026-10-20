# Task: hypotheses-writer (cycle 0009)

Cycle: 0009
Number of hypotheses: 3
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/hypotheses

## Framing

You are proposing the next batch of 3 hypotheses. Each hypothesis MUST cite exactly one parent variant: either `baseline` or a past variant `cycle/<NNNN>/v<NNN>` from any prior cycle (1..8). Aim for distinct strategies — near-duplicates collapse the cohort's winners.

## Authoring scope

Any overlay file may be edited, except the gate-integrity surface, writable by no scope (src/associative_cache/associative_cache_jni.rs, src/factory.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/, src/bin/). Do not propose ideas that require editing it.

## Incumbents (cross-cycle winners)

- cycle/0008/v003: hit-rate=0.7333, metadata-size=16, per-op-time=45
- cycle/0008/v001: hit-rate=0.7303, metadata-size=16, per-op-time=45
- cycle/0007/v001: hit-rate=0.7295, metadata-size=16, per-op-time=45

At least one of the hypotheses MUST cite a cross-cycle winner above as its `## Parent`. Prefer the non-baseline incumbents — they are the proven best-so-far. The remaining hypotheses may explore from `baseline` or prior losers.

## Read for context

### Previous cycle (0008)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/variants — every variant directory `v<NNN>/` under this path. For each variant read:
  - /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/variants/v<NNN>/attempts/<NN>/overlay — the variant's overlay (subject files it replaced); read its latest attempt.
  - /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/variants/v<NNN>/eval/fitness-vector.json — its measured fitness vector.
  - /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/variants/v<NNN>/insight.md — one-line fitness summary.
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/cycle-winners.json — Winners + per-variant fitness vectors for the cycle.
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/cycle-insight.md — cycle-level summary.

### Cross-cycle history

- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/leaderboard.md — current winner variants and fitness summaries across all cycles.
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles-ledger.jsonl — one JSON row per completed cycle (cohort size, winners, insights).
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cross-cycle-insights.md — long-running narrative of what has been learned across cycles. Read if present.

### Baseline

- /home/user/cache-evolve/inputs/subjects/associative_cache/baseline — the permanent baseline (always a valid parent variant).
