# Cross-Cycle Insights

## Recurring wins

- Layering complementary mechanisms beats widening a single signal: the
  ghost ring + single-hop Cold→Hot promotion combination established in
  cycle 2 has survived a structural challenge (cycle 5's S4LRU regressed
  below it) and is the only durable hit-rate lift the loop has produced.
- Tuning a knob that is *already in the hot path* is the only refinement
  that has ever moved hit-rate after the initial CLOCK-Pro-lite win —
  ghost-ring size at cycle 3, marginally. Every other tuning attempt has
  tied.
- Recency-encoding state (Cold→Hot on second touch, evicted-key memory)
  is the variety of extra per-slot state that has paid off on this trace.

## Recurring failures

- SLRU / segmented-LRU rebuilds: three attempts across cycles 1, 2, and
  5. The first two were uninterpretable because the variant and baseline
  were sampled at disjoint capacities (matched-size discipline never
  enforced); the third (S4LRU, properly sampled) regressed. Do not retry
  without either matched-size sampling AND a ghost-ring graft or a
  top-heavy asymmetric split.
- Frequency-leaning bits on top of CLOCK (cycle 2 GCLOCK aging counter):
  on a recency-skewed trace, frequency state actively shields stale
  keys and is strictly worse than recency-only state.
- Pass-2 / Hot-side edits at cap=512 on lirs:multi1 (cycle 3 sticky
  demotion, Hot-ghosts, warm re-admit): three variants scored at exact
  parity because pass 2 barely fires at this capacity. Any pass-2-only
  knob is inert here.
- Pass-1 / Cold-side victim re-ranking (cycle 4 scan-taint pre-pass,
  intra-Cold LRU, Warm ladder): three orthogonal edits, all bit-identical
  to the champion. Re-ordering the existing Cold victim pool does not
  change net hit-rate on this trace/size — the spared slot dies a sweep
  later anyway.
- Ghost-ring positional weighting (cycle 5): ghost freshness is not a
  usable promotion-strength gradient on this workload.
- The cohort *uniformly competing on hit-rate* rather than addressing
  the fitness-vector narrowness: this has been flagged every cycle and
  acted on zero times.

## Open questions

- Is lirs:multi1@512 simply wrung out for the CLOCK-Pro-lite skeleton?
  Three consecutive plateau cohorts suggest the residual loss channel
  on this single workload/capacity is no longer policy-reachable.
- Has the loop's productive bottleneck shifted decisively from policy
  design to evaluation breadth (more capacities, more traces sampled at
  matched sizes) and/or to fitness-axis breadth (per-op time,
  metadata-size dimensions)? Cycles 1–5 say yes; the cohorts have not
  acted on it.
- Would the inert pass-2 / Cold-re-ranking / Warm-ladder knobs actually
  differentiate at smaller capacities or on workloads where the Cold
  pool churns faster and pass 2 is forced to fire?
- Would a *correctly instrumented* cold-admission gate (sketch marked
  on `put`, not `get`) lift hit-rate, or come back at parity too? The
  cycle-5 v001 attempt's precondition collapsed before the lever was
  exercised — this is the one genuinely unpulled single-trace lever.
- Can any non-CLOCK-Pro-lite skeleton keep up on this trace without
  borrowing the ghost ring and the single-hop promotion?

## Per-policy notes

- **Baseline LRU**: the arena-slot-reuse scaffolding has been permissive
  of large representational rewrites (cycle 1) without invariant breakage.
- **SLRU / S4LRU (segmented LRU without augmentation)**: 0/3 record.
  Cycles 1 and 2 attempts were uninterpretable due to mismatched
  capacities; cycle 5's properly-sampled S4LRU lost to the champion. A
  multi-hop promotion ladder shrinks effective top-level capacity on a
  trace whose re-reference distance does not survive intermediate hops.
- **CLOCK (v002 cycle 1)**: landed on Pareto front but its real wins
  (per-op time) live on unscored axes — visible only by accident.
- **GCLOCK (frequency aging counter)**: regressed. Frequency bits are a
  structural loss on this recency-skewed trace.
- **CLOCK-Pro-lite (cycle 2 v002, cycle 3 v001 = current champion)**:
  the working skeleton since cycle 2. The ghost ring and single-hop
  Cold→Hot promotion are load-bearing (cycle 5 gave structural credit,
  not just plateau credit). Ghost-ring sizing is in strong diminishing
  returns past full-cap; Hot-demotion stickiness, Hot-ghosts, ghost
  provenance, ghost positional weighting, intra-Cold LRU, scan-taint
  pre-pass, and Warm two-touch ladder are all confirmed inert on
  lirs:multi1@512.
- **u32+sentinel link representation (v003 cycle 1)**: metadata-size
  win is real but unscoreable; selection cannot see it. Defer
  representation-only repacks until a metadata-size dimension exists.
