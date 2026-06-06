## Recurring wins

- Hot-path changes that *remove work* pay off; ones that add a branch or
  conversion do not. The fast integer hasher, the gentle ghost admission
  that cut protected churn, and above all the flat-CLOCK rewrite (which
  deleted the recency list entirely) all won by doing less.
- The mechanism-vs-policy split is the single most durable finding:
  layout mechanisms (narrow `u32` sentinel links, packing the segment/ref
  flag into a spare high bit, centralized cast helpers) reliably hit the
  24-byte node class at zero latency tax. Outcomes are decided by the
  policy riding on top, never by the representation.
- Orthogonal axes stack cleanly when they touch disjoint structures —
  SLRU policy + fast hasher, and later the metadata fold onto the SLRU
  base both combined without one paying for the other.
- A *modest* geometry step is free where a blunt one is not: nudging the
  protected cap to 7/8 gained hit-rate with no latency cost.
- The biggest frontier moves come from structural change (flat CLOCK),
  not micro-optimization — it reset a long-standing "champion at frontier"
  conclusion.

## Recurring failures

- Admission cleverness on this short-reuse trace: ghost/reuse-distance
  re-admission, two-hit probation gates, repeated-hit warm filters. They
  drain short-reuse keys before the gate fires and/or over-promote into
  protected, costing both hit-rate and latency. Refuted from many angles.
- Probe-skipping get caches (single last-key, multi-slot hot-key table):
  the baseline hasher+arena probe is already cheaper than any guard layered
  in front of it. The HashMap probe is not the get bottleneck.
- Micro-opts that remove compiler-handled redundancy (bounds-check elision,
  trailing NIL/dead-store removal): they attack non-costs and only perturb
  codegen, regressing the hot path. Needs an algorithmic op-count reduction,
  not source-level trimming.
- Blunt geometry widening (80→90%, 15/16): starves the protected-MRU fast
  path for a small hit-rate gain — a sharp starvation cliff above 7/8.
- Surgery-skip / position-guard branches outside their segment-local home:
  on flat LRU or as bare data-dependent branches they mispredict and add
  cost.

## Open questions

- Does the SLRU protected/probation split add anything over flat CLOCK on
  this trace, or was the whole segmented lineage carrying cost without
  payoff?
- Flat-CLOCK's own tuning axes — hand placement, sweep bound, fresh-key
  admission — are entirely untouched.
- Is CLOCK's slight hit-rate edge over exact LRU real, or noise?
- Does growing the cache struct's footprint carry a real latency tax worth
  treating as a constraint? Footprint→p50 coupling appeared repeatedly but
  was later shown non-causal on the control struct.

## Per-policy notes

- **SLRU (long-time champion):** geometry now closed from both sides of
  7/8; surgery/store-elimination axes mined out; admission gating dead on
  this workload. Little remaining room. Superseded on latency and footprint
  by flat CLOCK.
- **Flat single-list CLOCK (current base):** wins by dropping the recency
  list — lower get/put latency and 24-byte node at equal-or-better hit-rate.
  Its tuning axes remain open and unexplored.
- **CLOCK-on-SLRU:** backfires — the protected-MRU path is already
  write-free, so a ref bit only adds a store on the hottest hits.
- **Ghost / reuse-distance admission:** gated admission won once (early)
  but did not reproduce; the trace's reuse is too short-distance for the
  signal to hold. Treat as trace/size-contingent, not general.
- **Exact LRU recency tracking:** buys no hit-rate here; the pointer
  surgery it requires is pure overhead a single CLOCK bit avoids.
