# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-06-06T05:56:43+00:00

## timeline

### 05:57:09 thinking



### 05:57:16 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Hot-path changes that *remove work* pay off; ones that add a branch or\n  conversion do not. The fast integer hasher, the gentle ghost admission\n  that cut protected churn, and above all the flat-CLOCK rewrite (which\n  deleted the recency list entirely) all won by doing less.\n- The mechanism-vs-policy split is the single most durable finding:\n  layout mechanisms (narrow `u32` sentinel links, packing the segment/ref\n  flag into a spare high bit, centralized cast helpers) reliably hit the\n  24-byte node class at zero latency tax. Outcomes are decided by the\n  policy riding on top, never by the representation.\n- Orthogonal axes stack cleanly when they touch disjoint structures —\n  SLRU policy + fast hasher, and later the metadata fold onto the SLRU\n  base both combined without one paying for the other.\n- A *modest* geometry step is free where a blunt one is not: nudging the\n  protected cap to 7/8 gained hit-rate with no latency cost.\n- The biggest frontier moves come from structural change (flat CLOCK),\n  not micro-optimization — it reset a long-standing \"champion at frontier\"\n  conclusion.\n\n## Recurring failures\n\n- Admission cleverness on this short-reuse trace: ghost/reuse-distance\n  re-admission, two-hit probation gates, repeated-hit warm filters. They\n  drain short-reuse keys before the gate fires and/or over-promote into\n  protected, costing both hit-rate and latency. Refuted from many angles.\n- Probe-skipping get caches (single last-key, multi-slot hot-key table):\n  the baseline hasher+arena probe is already cheaper than any guard layered\n  in front of it. The HashMap probe is not the get bottleneck.\n- Micro-opts that remove compiler-handled redundancy (bounds-check elision,\n  trailing NIL/dead-store removal): they attack non-costs and only perturb\n  codegen, regressing the hot path. Needs an algorithmic op-count reduction,\n  not source-level trimming.\n- Blunt geometry widening (80→90%, 15/16): starves the protected-MRU fast\n  path for a small hit-rate gain — a sharp starvation cliff above 7/8.\n- Surgery-skip / position-guard branches outside their segment-local home:\n  on flat LRU or as bare data-dependent branches they mispredict and add\n  cost.\n\n## Open questions\n\n- Does the SLRU protected/probation split add anything over flat CLOCK on\n  this trace, or was the whole segmented lineage carrying cost without\n  payoff?\n- Flat-CLOCK's own tuning axes — hand placement, sweep bound, fresh-key\n  admission — are entirely untouched.\n- Is CLOCK's slight hit-rate edge over exact LRU real, or noise?\n- Does growing the cache struct's footprint carry a real latency tax worth\n  treating as a constraint? Footprint→p50 coupling appeared repeatedly but\n  was later shown non-causal on the control struct.\n\n## Per-policy notes\n\n- **SLRU (long-time champion):** geometry now closed from both sides of\n  7/8; surgery/store-elimination axes mined out; admission gating dead on\n  this workload. Little remaining room. Superseded on latency and footprint\n  by flat CLOCK.\n- **Flat single-list CLOCK (current base):** wins by dropping the recency\n  list — lower get/put latency and 24-byte node at equal-or-better hit-rate.\n  Its tuning axes remain open and unexplored.\n- **CLOCK-on-SLRU:** backfires — the protected-MRU path is already\n  write-free, so a ref bit only adds a store on the hottest hits.\n- **Ghost / reuse-distance admission:** gated admission won once (early)\n  but did not reproduce; the trace's reuse is too short-distance for the\n  signal to hold. Treat as trace/size-contingent, not general.\n- **Exact LRU recency tracking:** buys no hit-rate here; the pointer\n  surgery it requires is pure overhead a single CLOCK bit avoids.\n"
}
```

### 05:57:21 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 10-cycle ledger.

Key synthesis across cycles:
- **Wins** cluster around work-removal and the mechanism-vs-policy split: layout mechanisms reliably hit the 24-byte node for free, while the flat-CLOCK structural rewrite (deleting the recency list) was the biggest frontier move.
- **Failures** are consistent: admission cleverness, probe-skipping caches, and micro-opts against compiler-handled redundancy all repeatedly fail to pay — the first because the trace is short-reuse, the others because they target non-bottlenecks.
- **Open threads** center on the new flat-CLOCK lineage's untouched tuning axes and whether segmentation adds anything over it.
- **Per-policy notes** track SLRU (mined out), flat CLOCK (current base, room to tune), and the refuted ghost/reuse-distance and exact-LRU-recency approaches.


---

- finished: 2026-06-06T05:57:22+00:00
- duration: 38.92s
- status: ok
- input_tokens: 3
- output_tokens: 2696
- cache_read_tokens: 43531
- cache_write_tokens: 14900
- cost_usd: 0.116869
