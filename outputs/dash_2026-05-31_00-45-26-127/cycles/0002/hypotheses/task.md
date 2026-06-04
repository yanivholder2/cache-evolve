# Task: hypotheses-writer (cycle 0002)

Cycle: 0002
Number of hypotheses: 3
Output dir: /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-31_00-45-26-127/cycles/0002/hypotheses

## Framing

You are proposing the next batch of 3 hypotheses. Each hypothesis MUST cite exactly one parent variant: either `baseline` or a past variant `cycle/<NNNN>/v<NNN>` from any prior cycle (1..1). Aim for distinct strategies — near-duplicates collapse the cohort's Pareto front.

## Authoring scope

Hypotheses must confine their edits to the extension-point impl files below. Propose only ideas expressible within these files:
- src/dash/dash_bucket.rs
- src/shared/eviction_policy.rs
- src/shared/item.rs

The contract-protected files are writable by no scope: ['src/dash/jni.rs', 'src/factory.rs'].

## Incumbents (global Pareto front)

- baseline: hit-rate=0.4653

At least one of the hypotheses MUST cite a global-front variant above as its `## Parent`. The remaining hypotheses may explore from `baseline` or prior losers.

## Read for context

### Previous cycle (0001)

- /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-31_00-45-26-127/cycles/0001/variants — every variant directory `v<NNN>/` under this path. For each variant read:
  - /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-31_00-45-26-127/cycles/0001/variants/v<NNN>/attempts/<NN>/overlay — the variant's overlay (subject files it replaced); read its latest attempt.
  - /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-31_00-45-26-127/cycles/0001/variants/v<NNN>/eval/fitness-vector.json — its measured fitness vector.
  - /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-31_00-45-26-127/cycles/0001/variants/v<NNN>/insight.md — one-line fitness summary.
- /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-31_00-45-26-127/cycles/0001/winners.json — Pareto front + per-variant fitness vectors for the cycle.
- /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-31_00-45-26-127/cycles/0001/cycle-insight.md — cycle-level summary.

### Cross-cycle history

- /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-31_00-45-26-127/leaderboard.md — current Pareto-front variants and fitness summaries across all cycles.
- /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-31_00-45-26-127/cycles-ledger.jsonl — one JSON row per completed cycle (cohort size, Pareto front, insights).
- /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-31_00-45-26-127/cross-cycle-insights.md — long-running narrative of what has been learned across cycles. Read if present.

### Baseline

- /home/yaniv/projects/cache-evolve/inputs/subjects/dash/baseline — the permanent baseline (always a valid parent variant).

## Scope correction

The previous attempt named files outside the writable surface. Rewrite every hypothesis so its `## Changes` edits only these extension-point impl files:
- src/dash/dash_bucket.rs
- src/shared/eviction_policy.rs
- src/shared/item.rs

Out-of-scope files named last attempt:
- H02.md: ['bucket.rs', 'promotion_logic.rs']

Express the same idea through one of the in-scope files instead.
