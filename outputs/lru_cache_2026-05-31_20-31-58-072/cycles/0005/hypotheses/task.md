# Task: hypotheses-writer (cycle 0005)

Cycle: 0005
Number of hypotheses: 3
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0005/hypotheses

## Framing

You are proposing the next batch of 3 hypotheses. Each hypothesis MUST cite exactly one parent variant: either `baseline` or a past variant `cycle/<NNNN>/v<NNN>` from any prior cycle (1..4). Aim for distinct strategies — near-duplicates collapse the cohort's Pareto front.

## Authoring scope

Any overlay file may be edited, except the contract-protected files which are writable by no scope: ['src/lru_cache/lru_cache_jni.rs'].

## Incumbents (global Pareto front)

- cycle/0003/v001: hit-rate=0.4946
- cycle/0004/v001: hit-rate=0.4946
- cycle/0004/v002: hit-rate=0.4946
- cycle/0004/v003: hit-rate=0.4946

At least one of the hypotheses MUST cite a global-front variant above as its `## Parent`. Prefer the non-baseline incumbents — they are the proven best-so-far. The remaining hypotheses may explore from `baseline` or prior losers.

## Read for context

### Previous cycle (0004)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0004/variants — every variant directory `v<NNN>/` under this path. For each variant read:
  - /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0004/variants/v<NNN>/attempts/<NN>/overlay — the variant's overlay (subject files it replaced); read its latest attempt.
  - /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0004/variants/v<NNN>/eval/fitness-vector.json — its measured fitness vector.
  - /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0004/variants/v<NNN>/insight.md — one-line fitness summary.
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0004/winners.json — Pareto front + per-variant fitness vectors for the cycle.
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0004/cycle-insight.md — cycle-level summary.

### Cross-cycle history

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/leaderboard.md — current Pareto-front variants and fitness summaries across all cycles.
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles-ledger.jsonl — one JSON row per completed cycle (cohort size, Pareto front, insights).
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cross-cycle-insights.md — long-running narrative of what has been learned across cycles. Read if present.

### Baseline

- /home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline — the permanent baseline (always a valid parent variant).
