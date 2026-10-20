# Task: hypotheses-writer (cycle 0010)

Cycle: 0010
Number of hypotheses: 3
Output dir: /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0010/hypotheses

## Framing

You are proposing the next batch of 3 hypotheses. Each hypothesis MUST cite exactly one parent variant: either `baseline` or a past variant `cycle/<NNNN>/v<NNN>` from any prior cycle (1..9). Aim for distinct strategies — near-duplicates collapse the cohort's winners.

## Authoring scope

Any overlay file may be edited, except the gate-integrity surface, writable by no scope (src/lru_cache/lru_cache_jni.rs, Cargo.toml, src/lib.rs, tests/, src/test_utils/). Do not propose ideas that require editing it.

## Incumbents (cross-cycle winners)

- cycle/0006/v001: hit-rate=0.7343

At least one of the hypotheses MUST cite a cross-cycle winner above as its `## Parent`. Prefer the non-baseline incumbents — they are the proven best-so-far. The remaining hypotheses may explore from `baseline` or prior losers.

## Read for context

### Previous cycle (0009)

- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0009/variants — every variant directory `v<NNN>/` under this path. For each variant read:
  - /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0009/variants/v<NNN>/attempts/<NN>/overlay — the variant's overlay (subject files it replaced); read its latest attempt.
  - /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0009/variants/v<NNN>/eval/fitness-vector.json — its measured fitness vector.
  - /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0009/variants/v<NNN>/insight.md — one-line fitness summary.
- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0009/cycle-winners.json — Winners + per-variant fitness vectors for the cycle.
- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0009/cycle-insight.md — cycle-level summary.

### Cross-cycle history

- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/leaderboard.md — current winner variants and fitness summaries across all cycles.
- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles-ledger.jsonl — one JSON row per completed cycle (cohort size, winners, insights).
- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cross-cycle-insights.md — long-running narrative of what has been learned across cycles. Read if present.

### Baseline

- /home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline — the permanent baseline (always a valid parent variant).
