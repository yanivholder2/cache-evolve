# Task: hypotheses-writer (cycle 0002)

Cycle: 0002
Number of hypotheses: 3
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0002/hypotheses

## Framing

You are proposing the next batch of 3 hypotheses. Each hypothesis MUST cite exactly one parent variant: either `baseline` or a past variant `cycle/<NNNN>/v<NNN>` from any prior cycle (1..1). Aim for distinct strategies — near-duplicates collapse the cohort's winners.

## Authoring scope

Any overlay file may be edited, except the contract-protected files which are writable by no scope: ['src/dash/jni.rs', 'src/factory.rs'].

## Incumbents (cross-cycle winners)

- cycle/0001/v001: hit-rate=0.678

At least one of the hypotheses MUST cite a cross-cycle winner above as its `## Parent`. Prefer the non-baseline incumbents — they are the proven best-so-far. The remaining hypotheses may explore from `baseline` or prior losers.

## Read for context

### Previous cycle (0001)

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0001/variants — every variant directory `v<NNN>/` under this path. For each variant read:
  - /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0001/variants/v<NNN>/attempts/<NN>/overlay — the variant's overlay (subject files it replaced); read its latest attempt.
  - /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0001/variants/v<NNN>/eval/fitness-vector.json — its measured fitness vector.
  - /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0001/variants/v<NNN>/insight.md — one-line fitness summary.
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0001/cycle-winners.json — Winners + per-variant fitness vectors for the cycle.
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0001/cycle-insight.md — cycle-level summary.

### Cross-cycle history

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/leaderboard.md — current winner variants and fitness summaries across all cycles.
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles-ledger.jsonl — one JSON row per completed cycle (cohort size, winners, insights).
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cross-cycle-insights.md — long-running narrative of what has been learned across cycles. Read if present.

### Baseline

- /home/user/cache-evolve/inputs/subjects/dash/baseline — the permanent baseline (always a valid parent variant).
