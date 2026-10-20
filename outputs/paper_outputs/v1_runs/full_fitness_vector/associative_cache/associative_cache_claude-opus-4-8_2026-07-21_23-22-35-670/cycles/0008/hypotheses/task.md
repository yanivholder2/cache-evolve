# Task: hypotheses-writer (cycle 0008)

Cycle: 0008
Number of hypotheses: 3
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0008\hypotheses

## Framing

You are proposing the next batch of 3 hypotheses. Each hypothesis MUST cite exactly one parent variant: either `baseline` or a past variant `cycle/<NNNN>/v<NNN>` from any prior cycle (1..7). Aim for distinct strategies — near-duplicates collapse the cohort's winners.

## Authoring scope

Any overlay file may be edited, except the gate-integrity surface, writable by no scope (src\associative_cache\associative_cache_jni.rs, src\factory.rs, Cargo.toml, src\lib.rs, tests/, src\test_utils/, src\bin/). Do not propose ideas that require editing it.

## Incumbents (cross-cycle winners)

- cycle/0003/v001: hit-rate=0.7267, metadata-size=16, per-op-time=100
- cycle/0006/v001: hit-rate=0.7187, metadata-size=16, per-op-time=100
- cycle/0003/v003: hit-rate=0.7182, metadata-size=16, per-op-time=100

At least one of the hypotheses MUST cite a cross-cycle winner above as its `## Parent`. Prefer the non-baseline incumbents — they are the proven best-so-far. The remaining hypotheses may explore from `baseline` or prior losers.

## Read for context

### Previous cycle (0007)

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0007\variants — every variant directory `v<NNN>/` under this path. For each variant read:
  - C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0007\variants\v<NNN>\attempts\<NN>\overlay — the variant's overlay (subject files it replaced); read its latest attempt.
  - C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0007\variants\v<NNN>\eval\fitness-vector.json — its measured fitness vector.
  - C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0007\variants\v<NNN>\insight.md — one-line fitness summary.
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0007\cycle-winners.json — Winners + per-variant fitness vectors for the cycle.
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0007\cycle-insight.md — cycle-level summary.

### Cross-cycle history

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\leaderboard.md — current winner variants and fitness summaries across all cycles.
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles-ledger.jsonl — one JSON row per completed cycle (cohort size, winners, insights).
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cross-cycle-insights.md — long-running narrative of what has been learned across cycles. Read if present.

### Baseline

- C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\associative_cache\baseline — the permanent baseline (always a valid parent variant).
