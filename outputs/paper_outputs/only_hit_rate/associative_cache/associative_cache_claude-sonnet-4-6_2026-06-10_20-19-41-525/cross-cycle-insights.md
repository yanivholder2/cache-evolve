# Cross-Cycle Insights

## Recurring wins

- **Wide buckets reduce premature eviction.** Repartitioning capacity into fewer, larger
  sets was the first win (cycle 1) and has been a necessary enabler for every downstream
  mechanism. Useful range ends around 64 slots; beyond that, returns are negligible.
- **Cold-first eviction (counter == 0) is the strongest single lever.** Preferring
  zero-access items as eviction candidates reliably shields the working set from scan
  pollution. The `counter == 0` boundary has been probed and confirmed optimal for
  workload-a — one access is a reliable future-reuse signal on this trace.
- **Mid-point insertion adds independent value as a cold-pool concentrator.** Landing
  new items at the bucket midpoint keeps them away from the LRU-front eviction target
  and can substitute partially for geometry widening. It was confirmed non-trivial by
  the ablation in cycle 8.
- **Episodic bulk decay is a well-targeted warm-lock patch.** Decrementing all counters
  by 2 only when no cold candidate exists fires rarely, restores cold supply without
  touching the normal eviction path, and has been stable across many cycles without
  counter-value oscillation.
- **Positional recency at index 0 is the correct warm-lock eviction target.** Under
  mid-point insertion, index 0 encodes insertion age; it consistently outperforms any
  counter-guided interior selection during warm-lock fallback. Every attempt to replace
  it has regressed.
- **The accumulated single-queue stack is a robust local optimum.** The combination of
  wide buckets + cold-first eviction + mid-point insertion + episodic decay-by-2 has
  held the top position across four-plus consecutive cycles of structural challenges,
  including SLRU, CLOCK, asymmetric decay, and ghost re-admission.

## Recurring failures

- **O(1) LRU swap-to-back (cycle 1):** Corrupts the recency gradient across intermediate
  slots; the O(n) cost of correct LRU ordering is worth paying.
- **Geometry widening beyond ~64 slots (cycles 2–4):** Diminishing returns confirmed
  twice; the useful range is now exhausted for both plain and mid-point paths.
- **Counter threshold tightening to counter ≤ 1 (cycle 3):** Shields too many items,
  causing genuine hot-set members to be evicted prematurely on workload-a.
- **LFU warm-fallback after bulk decay (cycle 5):** Uniform decay compresses counter
  differences to near-zero; min-counter selection then becomes noisier than the recency
  order already in bucket position. Confirmed counter-productive.
- **Mid-point insertion + LFU warm-fallback combined (cycle 5):** The two mechanisms are
  adversarial — cold-first is position-agnostic, so mid-point's benefit vanishes, while
  LFU disrupts the recency structure mid-point depends on.
- **CLOCK (cycle 6):** Steady per-eviction counter sweep erodes still-relevant items
  faster than the infrequent bulk decay; episodic decay dominates.
- **Asymmetric / split decay — positional (cycle 9) and value-based (cycle 10):** In
  both forms, the cold-first scan already shields hot items before decay fires, so
  preserving upper-tier counters changes no eviction decisions. Both axes closed.
- **ARC-style ghost re-admission (cycle 7):** Workload-a lacks post-eviction re-access
  patterns; the ghost overhead is real but the benefit is not. Structurally mismatched
  to this trace.
- **Counter ceiling reduction (cycles 5, 7):** Cap-4 is inert; cap-2 mildly regresses.
  The ceiling axis is exhausted.
- **Post-decay cold re-scan as warm-lock target (cycles 7, 10):** Selecting an interior
  counter-0 item over index 0 discards temporal locality — the interior item is typically
  fresher. Confirmed regressive twice.

## Open questions

- **Per-op latency is entirely unmeasured after ten cycles.** The accumulated stack's
  efficiency versus simpler alternatives (pure-LFU wide-geometry from cycle 6, best SLRU
  from cycle 9) cannot be evaluated without this data. This is the highest-priority
  unresolved dimension.
- **Do the gains generalise beyond workload-a?** Cold-first eviction, mid-point
  insertion, and episodic decay have only ever been measured on one scan-dominated trace.
  Their relative ranking could shift substantially on traces with different locality
  profiles.
- **Is the hit-rate ceiling workload-specific or policy-family-specific?** Ten cycles of
  tuning have not broken through the current ceiling. A qualitatively different policy
  family — ARC proper, LIRS-proper, or an adaptive two-queue with explicit recency and
  frequency queues — might reach a higher ceiling, but this has not been tested.
- **SLRU segment sizing is unexplored beyond 50/50 and 25/75.** A protected-heavy split
  with a larger probationary buffer might alter the trade-off, though the SLRU branch as
  a whole appears to be approaching its ceiling.

## Per-policy notes

- **Single-queue accumulated stack (ClassicLRU + cold-first + wide buckets + mid-point +
  decay-by-2):** Dominant across cycles 2–10. Counter==0 threshold and index-0 warm-lock
  target are both confirmed settled for workload-a. Decay magnitude of 2 (doubled in
  cycle 8) is the current parametric baseline. The counter cap at 4 is carried silently
  as defensive hardening.
- **SLRU:** Produced the only new hit-rate improvement after the cycle-5 plateau (cycle
  8). Explored across cycles 8–9; best result combined cold-first eviction, decay scoped
  to probationary, and wide buckets (cycle 9 v003). Approached but never crossed the
  single-queue incumbent. Segment ratio (50/50 → 25/75) is exhausted. Explicit
  bookkeeping appears to cost more than the implicit position encoding in the single-queue
  family saves, on this workload.
- **CLOCK:** Tested once (cycle 6); inferior to episodic bulk decay. Steady sweep harms
  active items; the infrequent, concentrated cost of bulk decay is categorically better
  on workload-a. Do not revisit.
- **Pure LFU / min-counter on wide geometry (cycle 6 v003):** Closed most of the gap
  from the unmodified baseline to the accumulated-stack best, confirming that a large
  candidate pool largely substitutes for cold-first bias. The residual deficit quantifies
  the combined contribution of cold-first ordering and decay — small but real. This is
  the simplest competitive alternative if a latency audit reveals an efficiency trade-off.
- **ARC-style ghost re-admission (cycle 7):** Ineffective on workload-a; requires
  post-eviction temporal locality the trace does not exhibit. May be worth revisiting
  only on traces with scan-then-reuse patterns.
