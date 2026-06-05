# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-06-05T20:59:29+00:00

## timeline

### 20:59:36 thinking



### 20:59:36 tool: Bash

```json
{
  "command": "pwd",
  "description": "Print working directory"
}
```

### 20:59:38 thinking



### 20:59:53 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Eviction *ordering*, not extra residency, is the lever that pays: every\n  gain has come from deciding which resident key dies first under a fixed\n  capacity bound, never from relaxing the bound.\n- \"Protect on second access\" is the durable winning signal — distinguishing\n  one-hit-wonders from genuine reuse (via segments, midpoint, or reference\n  bit) beat pure LRU in the first cohort and anchored every cycle since.\n- Enlarging the protected tier has paid repeatedly (80/20 → 90/10 → 95/5),\n  the most reliable monotone lever across cycles — though each step's return\n  shrank and the latest win was the thinnest yet.\n- First-hit (early, generous) promotion is confirmed correct: reuse on this\n  trace arrives fast, landing within a narrow probationary window.\n\n## Recurring failures\n\n- Loosening the parent's promotion discipline always regressed: two-hit\n  promotion gates and wider probationary segments both lose keys that churn\n  out before a delayed second touch.\n- Ghost / reuse-distance re-admission is persistently marginal-to-dead:\n  directionally right but structurally redundant against a heavily protected\n  tier, where the warm-but-evicted band left to rescue is vanishingly thin —\n  tested in isolation and layered on the incumbent, it never moved the needle.\n- Non-recency admission signals (front admission window, W-TinyLFU) are\n  actively harmful: their windows are far smaller than this trace's reuse\n  distances, evicting genuinely reused keys before their second touch.\n\n## Open questions\n\n- Is the protected ratio at its ceiling? Each push still pays a little, but\n  the probationary floor needed to drain one-hit-wonders sets a hard limit\n  somewhere near the current point.\n- Has the recency/promotion axis been fully consumed, leaving the incumbent\n  a converged optimum — or is there a genuinely different, untapped workload\n  signal that off-recency levers keep failing to capture?\n- Are the cheap protection signals (reference bit, midpoint) additive with\n  segmentation, or redundant? Never tested in combination.\n\n## Per-policy notes\n\n- Pure LRU: baseline only; evicts reused keys as readily as one-hit-wonders\n  and has been beaten in every cohort. Dropped as a probe.\n- SLRU (segmented): the consistent winner and incumbent; gains track\n  protected-tier size, with promotion timing already near-optimal.\n- Midpoint insertion / CLOCK: cheap single-list approximations that also beat\n  LRU in cohort 1, showing the gain doesn't strictly require two-segment\n  bookkeeping — but explicit segmentation has won every head-to-head.\n- Ghost-list and admission-window variants: confirmed marginal or harmful on\n  this reuse-heavy LIRS trace; retired as standalone and layered probes.\n"
}
```

### 20:59:58 chat

Wrote `cross-cycle-insights.md` as a fresh projection of all four ledger cycles, using the four fixed sections in order:

- **Recurring wins** — eviction-ordering over residency, "protect on second access," the repeatedly-paying protected-tier enlargement, and fast/early promotion.
- **Recurring failures** — loosening promotion discipline, marginal-to-dead ghost re-admission, and harmful non-recency admission windows.
- **Open questions** — protected-ratio ceiling, whether the recency axis is exhausted, and untested additivity of cheap protection signals.
- **Per-policy notes** — grouped by LRU, SLRU, midpoint/CLOCK, and ghost/admission-window variants.

Signals are combined across cycles rather than restating single-cycle observations, with dimensions kept qualitative.


---

- finished: 2026-06-05T20:59:59+00:00
- duration: 30.25s
- status: ok
- input_tokens: 5
- output_tokens: 1759
- cache_read_tokens: 59927
- cache_write_tokens: 8112
- cost_usd: 0.07785
