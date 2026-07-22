# Task: hypotheses-writer (cycle 0004)

Cycle: 0004
Number of hypotheses: 3
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_13-12-55-552\cycles\0004\hypotheses

## Framing

You are proposing the next batch of 3 hypotheses. Each hypothesis MUST cite exactly one parent variant: either `baseline` or a past variant `cycle/<NNNN>/v<NNN>` from any prior cycle (1..3). Prefer refining recent winners or diversifying from recent losers; aim for distinct strategies — near-duplicates collapse the cohort's winners.

## Authoring scope

Any overlay file may be edited, except the gate-integrity surface, writable by no scope (src\associative_cache\associative_cache_jni.rs, src\factory.rs, Cargo.toml, src\lib.rs, tests/, src\test_utils/, src\bin/). Do not propose ideas that require editing it.

## Read for context

### Previous cycle (0003)

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_13-12-55-552\cycles\0003\variants — every variant directory `v<NNN>/` under this path. For each variant read:
  - C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_13-12-55-552\cycles\0003\variants\v<NNN>\attempts\<NN>\overlay — the variant's overlay (subject files it replaced); read its latest attempt.
  - C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_13-12-55-552\cycles\0003\variants\v<NNN>\eval\fitness-vector.json — its measured fitness vector.
  - C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_13-12-55-552\cycles\0003\variants\v<NNN>\insight.md — one-line fitness summary.
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_13-12-55-552\cycles\0003\cycle-winners.json — Winners + per-variant fitness vectors for the cycle.
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_13-12-55-552\cycles\0003\cycle-insight.md — cycle-level summary.

### Cross-cycle history

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_13-12-55-552\leaderboard.md — current winner variants and fitness summaries across all cycles.
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_13-12-55-552\cycles-ledger.jsonl — one JSON row per completed cycle (cohort size, winners, insights).
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_13-12-55-552\cross-cycle-insights.md — long-running narrative of what has been learned across cycles. Read if present.

### Baseline

- C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\associative_cache\baseline — the permanent baseline (always a valid parent variant).
