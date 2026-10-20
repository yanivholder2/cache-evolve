# Task: hypotheses-writer (cycle 0010)

Cycle: 0010
Number of hypotheses: 3
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0010\hypotheses

## Framing

You are proposing the next batch of 3 hypotheses. Each hypothesis MUST cite exactly one parent variant: either `baseline` or a past variant `cycle/<NNNN>/v<NNN>` from any prior cycle (1..9). Aim for distinct strategies — near-duplicates collapse the cohort's winners.

## Authoring scope

Any overlay file may be edited, except the gate-integrity surface, writable by no scope (src\dash\jni.rs, src\factory.rs, Cargo.toml, src\lib.rs, tests/, src\test_utils/, src\bin/). Do not propose ideas that require editing it.

## Incumbents (cross-cycle winners)

- cycle/0006/v001: hit-rate=0.692, metadata-size=24, per-op-time=100
- cycle/0008/v002: hit-rate=0.6919, metadata-size=24, per-op-time=100
- cycle/0009/v001: hit-rate=0.6918, metadata-size=24, per-op-time=100

At least one of the hypotheses MUST cite a cross-cycle winner above as its `## Parent`. Prefer the non-baseline incumbents — they are the proven best-so-far. The remaining hypotheses may explore from `baseline` or prior losers.

## Read for context

### Previous cycle (0009)

- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0009\variants — every variant directory `v<NNN>/` under this path. For each variant read:
  - C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0009\variants\v<NNN>\attempts\<NN>\overlay — the variant's overlay (subject files it replaced); read its latest attempt.
  - C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0009\variants\v<NNN>\eval\fitness-vector.json — its measured fitness vector.
  - C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0009\variants\v<NNN>\insight.md — one-line fitness summary.
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0009\cycle-winners.json — Winners + per-variant fitness vectors for the cycle.
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0009\cycle-insight.md — cycle-level summary.

### Cross-cycle history

- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\leaderboard.md — current winner variants and fitness summaries across all cycles.
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles-ledger.jsonl — one JSON row per completed cycle (cohort size, winners, insights).
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cross-cycle-insights.md — long-running narrative of what has been learned across cycles. Read if present.

### Baseline

- C:\Users\yaniv\Projects\cache-evolve\inputs\subjects\dash\baseline — the permanent baseline (always a valid parent variant).
