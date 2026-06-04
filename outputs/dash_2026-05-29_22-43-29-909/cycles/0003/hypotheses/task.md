# Task: hypotheses-writer (cycle 0003)

Cycle: 0003
Number of hypotheses: 3
Output dir: /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-29_22-43-29-909/cycles/0003/hypotheses

## Framing

You are proposing the next batch of 3 hypotheses. Each hypothesis MUST cite exactly one parent variant: either `baseline` or a past variant `cycle/<NNNN>/v<NNN>` from any prior cycle (1..2). Prefer refining recent winners or diversifying from recent losers; aim for distinct strategies — near-duplicates collapse the cohort's Pareto front.

## Read for context

### Previous cycle (0002)

- /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-29_22-43-29-909/cycles/0002/variants — every variant directory `v<NNN>/` under this path. For each variant read:
  - /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-29_22-43-29-909/cycles/0002/variants/v<NNN>/attempts/<NN>/overlay — the variant's overlay (subject files it replaced); read its latest attempt.
  - /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-29_22-43-29-909/cycles/0002/variants/v<NNN>/eval/fitness-vector.json — its measured fitness vector.
  - /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-29_22-43-29-909/cycles/0002/variants/v<NNN>/insight.md — one-line fitness summary.
- /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-29_22-43-29-909/cycles/0002/winners.json — Pareto front + per-variant fitness vectors for the cycle.
- /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-29_22-43-29-909/cycles/0002/cycle-insight.md — cycle-level summary.

### Cross-cycle history

- /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-29_22-43-29-909/leaderboard.md — current Pareto-front variants and fitness summaries across all cycles.
- /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-29_22-43-29-909/cycles-ledger.jsonl — one JSON row per completed cycle (cohort size, Pareto front, insights).
- /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-29_22-43-29-909/cross-cycle-insights.md — long-running narrative of what has been learned across cycles. Read if present.

### Baseline

- /home/yaniv/projects/cache-evolve/inputs/subjects/dash/baseline — the permanent baseline (always a valid parent variant).
