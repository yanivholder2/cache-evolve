# Task: hypotheses-writer (cycle 0006)

Cycle: 0006
Number of hypotheses: 3
Output dir: /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_01-27-45-376/cycles/0006/hypotheses

## Framing

You are proposing the next batch of 3 hypotheses. Each hypothesis MUST cite exactly one parent variant: either `baseline` or a past variant `cycle/<NNNN>/v<NNN>` from any prior cycle (1..5). Aim for distinct strategies — near-duplicates collapse the cohort's Pareto front.

## Authoring scope

Hypotheses must confine their edits to the extension-point impl files below. Propose only ideas expressible within these files:
- src/associative_cache/associative_cache_bucket.rs
- src/shared/eviction_policy.rs

The contract-protected files are writable by no scope: ['src/associative_cache/associative_cache_jni.rs', 'src/factory.rs'].

## Incumbents (global Pareto front)

- cycle/0005/v001: hit-rate=0.7278, metadata-size=40

At least one of the hypotheses MUST cite a global-front variant above as its `## Parent`. Prefer the non-baseline incumbents — they are the proven best-so-far. The remaining hypotheses may explore from `baseline` or prior losers.

## Read for context

### Previous cycle (0005)

- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_01-27-45-376/cycles/0005/variants — every variant directory `v<NNN>/` under this path. For each variant read:
  - /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_01-27-45-376/cycles/0005/variants/v<NNN>/attempts/<NN>/overlay — the variant's overlay (subject files it replaced); read its latest attempt.
  - /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_01-27-45-376/cycles/0005/variants/v<NNN>/eval/fitness-vector.json — its measured fitness vector.
  - /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_01-27-45-376/cycles/0005/variants/v<NNN>/insight.md — one-line fitness summary.
- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_01-27-45-376/cycles/0005/winners.json — Pareto front + per-variant fitness vectors for the cycle.
- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_01-27-45-376/cycles/0005/cycle-insight.md — cycle-level summary.

### Cross-cycle history

- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_01-27-45-376/leaderboard.md — current Pareto-front variants and fitness summaries across all cycles.
- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_01-27-45-376/cycles-ledger.jsonl — one JSON row per completed cycle (cohort size, Pareto front, insights).
- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_01-27-45-376/cross-cycle-insights.md — long-running narrative of what has been learned across cycles. Read if present.

### Baseline

- /home/yaniv/projects/cache-evolve/inputs/subjects/associative_cache/baseline — the permanent baseline (always a valid parent variant).
