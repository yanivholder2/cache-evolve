# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-06-05T22:13:40+00:00

## timeline

### 22:13:54 thinking



### 22:13:54 chat

I'll write the fresh projection of the ledger digest.

### 22:14:07 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Eviction *ordering*, not extra residency, is where gains come from: every\n  win kept the capacity bound fixed and changed only which resident dies first.\n- \"Protect on second access\" / first-hit promotion is the load-bearing signal\n  on this trace — reuse arrives fast, so a single early touch is the right\n  promotion trigger.\n- Protected-tier *size* is the dominant lever: successive enlargements of the\n  protected segment (80/20 → 90/10 → 95/5) each won, with steadily shrinking\n  margins as the probationary drain floor became the binding constraint.\n- When a probe must regress, the variant anchored to the incumbent lineage\n  tends to take the Winners slot by regressing least — many recent \"wins\" are\n  confirmations of a negative.\n\n## Recurring failures\n\n- Loosening the promotion signal (two-hit gate, wider probationary dwell) keeps\n  losing keys that churn out before a second touch.\n- Non-recency signals inside the SLRU frame — LFU victim selection, frequency\n  at the demotion point, front admission windows (W-TinyLFU) — regress or go\n  inert; one-hit-wonders do not dominate this LIRS-derived trace.\n- Reuse-distance memory (ghost lists, re-admission, reactive/adaptive floors)\n  is directionally right but redundant against a near-fully-protected tier; the\n  warm-but-evicted band is too thin to monetize.\n- Adaptive geometry (floating the protected/probationary split) does worse than\n  the static optimum; the split cannot be discovered from reuse feedback alone.\n- Transplanting SLRU tuning lessons onto LIRS is disconfirmed — lessons inside\n  one skeleton are not portable as parameter edits to a different invariant\n  structure.\n- Every alternative skeleton (LIRS, ARC, 2Q) beats the plain-LRU baseline\n  structurally, yet none clears the tuned 95/5 SLRU incumbent when raced.\n\n## Open questions\n\n- Is there any genuinely untested *structural* axis left, or is the 95/5 SLRU\n  optimum demonstrably unbeatable on this trace?\n- ARC's residual gap was never decomposed to a single mechanism the way 2Q's\n  was — its T2 internal ordering/admission is the one unfinished thread.\n- A persistent process blind spot (now mostly closed): new skeletons were\n  parented on baseline and reported baseline-relative lifts; direct\n  head-to-heads against the incumbent were deferred for many cycles.\n\n## Per-policy notes\n\n- **SLRU**: The converged winner at 95/5. Geometry is exhausted both statically\n  (97/3 regresses below the drain floor) and adaptively; every drain-floor,\n  reprieve, ghost-placement, and depth-stratification tweak has regressed.\n- **LIRS**: Home-turf skeleton; reported a real lift early but a clean untuned\n  LIRS-vs-incumbent verification was repeatedly called for and never executed.\n  Its S-membership/stack pruning already does the filtering SLRU bolts on, so\n  SLRU-style tunings (wider HIR window, weak-tier ghost diversion, widened\n  non-resident horizon, reactive floor) all backfire.\n- **ARC**: First skeleton to beat baseline by separating recency (T1) from\n  frequency (T2). Gap to incumbent is *not* recency-tier size — clamping `p`\n  was a no-op since `p` never climbs on this reuse-heavy trace; the gap lives\n  in T2 ordering/admission, still undecomposed.\n- **2Q**: Beats baseline via FIFO admission filtering. Raced directly and\n  trailed; deficit pinned to neither geometry nor promotion timing — the\n  delayed A1out ghost gate is load-bearing and demands cross-lifetime reuse\n  proof before a main-list slot.\n"
}
```

### 22:14:12 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 10-cycle ledger digest, using the four fixed sections in order.

Key cross-cycle synthesis:
- **Wins**: eviction-ordering over residency; first-hit "protect on second access"; protected-tier *size* as the dominant lever (80/20→90/10→95/5 with diminishing returns); incumbent-anchored variants winning by confirming negatives.
- **Failures**: loosening promotion, non-recency signals inside SLRU, ghost/reuse-distance memory, adaptive geometry, SLRU→LIRS transfer, and three skeletons (LIRS/ARC/2Q) beating baseline but never the incumbent.
- **Open questions**: any untested structural axis; ARC's undecomposed T2 gap; the long-standing baseline-vs-incumbent comparison blind spot.
- **Per-policy notes** grouped across SLRU, LIRS, ARC, and 2Q.


---

- finished: 2026-06-05T22:14:12+00:00
- duration: 32.41s
- status: ok
- input_tokens: 3
- output_tokens: 2476
- cache_read_tokens: 43982
- cache_write_tokens: 15125
- cost_usd: 0.114823
