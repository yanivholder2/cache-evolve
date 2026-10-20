# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-06-05T21:58:57+00:00

## timeline

### 21:59:16 thinking



### 21:59:16 chat

I'll write the fresh projection from the ledger digest.

### 21:59:23 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Eviction *ordering*, not extra residency, is the core lever: gains came\n  from deciding which resident key dies first under a fixed capacity\n  bound. \"Protect on second access\" (SLRU/midpoint/CLOCK) beat plain LRU\n  decisively and remains the productive axis.\n- Enlarging the protected tier kept paying across four cycles\n  (80/20 → 90/10 → 95/5), each step a real but shrinking win — more room\n  for twice-touched hot keys converts directly to hits.\n- First-hit promotion plus a generous protected tier is confirmed\n  correct: reuse on this trace arrives fast, within the narrow\n  probationary window.\n\n## Recurring failures\n\n- Loosening the promotion signal always regresses: two-hit gates, wider\n  probationary dwell, and post-eviction straight-to-protected ghosts all\n  lost keys that churn out before a delayed second touch.\n- Non-recency signals lose inside the SLRU frame every time: LFU victim\n  selection, frequency at the demotion point, and front admission windows\n  (W-TinyLFU) are inert or harmful — one-hit-wonders do not dominate this\n  reuse-heavy trace.\n- Ghost / reuse-distance re-admission is structurally redundant against a\n  maximally-protected tier: the warm-but-evicted band left to rescue is\n  vanishingly thin, and it costs structure plus per-insert scans.\n- Drain-floor protection is exhausted in both skeletons: static window,\n  reprieve laps, adaptive boundary resizing, and reactive minimums all\n  fail — the binding constraint is the probationary floor's *size*, which\n  these erode.\n- SLRU tuning lessons do not transplant onto LIRS as parameter edits;\n  cross-policy transfer is disconfirmed.\n\n## Open questions\n\n- The persistent blind spot: every new skeleton (LIRS, ARC, 2Q) is\n  parented on baseline and reports a baseline-relative lift, but no clean\n  head-to-head against the 95/5 SLRU incumbent on its home trace has ever\n  been run. Does any of them actually clear the incumbent?\n- Is ARC's residual gap tunable via T2 internal ordering/admission (its\n  size lever is a confirmed no-op), or is the skeleton simply below the\n  incumbent?\n- Has this trace's recency/promotion structure been fully consumed, or\n  does a genuinely different workload signal remain in an alternative\n  skeleton?\n\n## Per-policy notes\n\n- **SLRU**: converged at 95/5. Geometry is closed — static tuning,\n  adaptive resizing, depth-stratified protected tiers, and every\n  ghost-placement variant have been tested and exhausted.\n- **LIRS**: reported a real lift on its home turf but never cleanly\n  verified against the incumbent; all tuning attempts (HIR window, ghost\n  diversion, widened horizon, reactive floor) regressed.\n- **ARC**: first skeleton to beat baseline structurally via\n  recency/frequency separation; `p`-clamp tuning is a no-op because `p`\n  never climbs on this trace. Untested directly against incumbent.\n- **2Q**: FIFO admission filters one-hit-wonders and beats baseline; like\n  LIRS and ARC, never raced against the 95/5 incumbent.\n"
}
```

### 21:59:34 thinking



### 21:59:37 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Eviction *ordering*, not extra residency, is the core lever: gains came\n  from deciding which resident key dies first under a fixed capacity\n  bound. \"Protect on second access\" (SLRU/midpoint/CLOCK) beat plain LRU\n  decisively and remains the productive axis.\n- Enlarging the protected tier kept paying across four cycles\n  (80/20 → 90/10 → 95/5), each step a real but shrinking win — more room\n  for twice-touched hot keys converts directly to hits.\n- First-hit promotion plus a generous protected tier is confirmed\n  correct: reuse on this trace arrives fast, within the narrow\n  probationary window.\n\n## Recurring failures\n\n- Loosening the promotion signal always regresses: two-hit gates, wider\n  probationary dwell, and post-eviction straight-to-protected ghosts all\n  lost keys that churn out before a delayed second touch.\n- Non-recency signals lose inside the SLRU frame every time: LFU victim\n  selection, frequency at the demotion point, and front admission windows\n  (W-TinyLFU) are inert or harmful — one-hit-wonders do not dominate this\n  reuse-heavy trace.\n- Ghost / reuse-distance re-admission is structurally redundant against a\n  maximally-protected tier: the warm-but-evicted band left to rescue is\n  vanishingly thin, and it costs structure plus per-insert scans.\n- Drain-floor protection is exhausted in both skeletons: static window,\n  reprieve laps, adaptive boundary resizing, and reactive minimums all\n  fail — the binding constraint is the probationary floor's *size*, which\n  these erode.\n- SLRU tuning lessons do not transplant onto LIRS as parameter edits;\n  cross-policy transfer is disconfirmed.\n\n## Open questions\n\n- The persistent blind spot: every new skeleton (LIRS, ARC, 2Q) is\n  parented on baseline and reports a baseline-relative lift, but no clean\n  head-to-head against the 95/5 SLRU incumbent on its home trace has ever\n  been run. Does any of them actually clear the incumbent?\n- Is ARC's residual gap tunable via T2 internal ordering/admission (its\n  size lever is a confirmed no-op), or is the skeleton simply below the\n  incumbent?\n- Has this trace's recency/promotion structure been fully consumed, or\n  does a genuinely different workload signal remain in an alternative\n  skeleton?\n\n## Per-policy notes\n\n- **SLRU**: converged at 95/5. Geometry is closed — static tuning,\n  adaptive resizing, depth-stratified protected tiers, and every\n  ghost-placement variant have been tested and exhausted.\n- **LIRS**: reported a real lift on its home turf but never cleanly\n  verified against the incumbent; all tuning attempts (HIR window, ghost\n  diversion, widened horizon, reactive floor) regressed.\n- **ARC**: first skeleton to beat baseline structurally via\n  recency/frequency separation; `p`-clamp tuning is a no-op because `p`\n  never climbs on this trace. Untested directly against incumbent.\n- **2Q**: FIFO admission filters one-hit-wonders and beats baseline; like\n  LIRS and ARC, never raced against the 95/5 incumbent.\n"
}
```

### 21:59:43 chat

Wrote `cross-cycle-insights.md` as a fresh projection of all 9 cycles. Summary of what the four sections capture:

- **Recurring wins**: eviction *ordering* over residency, "protect on second access," and the four-cycle run of protected-tier enlargement (80/20 → 95/5).
- **Recurring failures**: loosening promotion, non-recency signals inside SLRU, ghost re-admission, drain-floor protection, and the disconfirmed SLRU→LIRS transfer.
- **Open questions**: the persistent blind spot — no new skeleton (LIRS/ARC/2Q) has ever been raced head-to-head against the 95/5 incumbent — plus ARC's tunability and whether the trace's structure is fully consumed.
- **Per-policy notes**: grouped status for SLRU (converged/closed), LIRS, ARC, and 2Q.


---

- finished: 2026-06-05T21:59:44+00:00
- duration: 46.14s
- status: ok
- input_tokens: 5
- output_tokens: 3595
- cache_read_tokens: 72174
- cache_write_tokens: 15258
- cost_usd: 0.139702
