# Cross-Cycle Insights

## Recurring wins

- Removing hidden hot-path cost is the most repeatable latency win: O(n)
  front-shifts, `Instant::now()` syscalls, and trait-object dispatch each cut
  per-op time once excised. Structural devirtualization (inner policy) is large
  enough to clear the harness noise floor.
- Per-entry metadata slimming paid off early on both size and latency, because
  the positional eval policy never reads recency/frequency fields — every such
  field was dead weight padding `Item`.
- Relocating the promotion-heat signal out of `Item` into a segment-side map
  reached the long-sought Pareto corner: best-known hit-rate held bit-for-bit
  at the metadata floor a counter-carrying entry could never reach.
- On the side-table probe, trivializing the hasher (dropping SipHash's keyed
  permutation for a plain `u64` hash on integer keys) broke the get-p50 floor
  for free while preserving layout and gate semantics.
- Tail-variance reducers compose safely: capacity pre-reservation and a
  2×-sized direct-mapped array each cut p99s while holding hit-rate exactly.
- Early on, MRU placement plus a proportional top-quarter promotion gate lifted
  hit-rate without re-introducing probing-tier churn.

## Recurring failures

- Tuning hit-rate via gate width, gate shape, placement, associativity, or
  frequency/ghost admission: every class moved hit-rate within noise. Residency
  is capacity/compulsory-bound, not conflict-bound, on this trace.
- Micro-latency edits — branch reorders, removing "wasted" index arithmetic —
  sit below the measurement floor and often move the wrong op via codegen
  perturbation.
- Stacking the two banked corners (devirtualized latency + side-table size) on
  one hot path is substitutive, not additive: the side-table probe, not the
  vtable, bounds that get path.
- Monomorphizing the *outer* container layer regresses locality — a fat inline
  struct widens `Vec` stride and costs more than the dispatch it removes.
- Surgery on the demotion/stash cascade cannot be scored on latency: the events
  are too rare to register, and decouple-by-dropping a victim is strictly worse
  (concedes real residency for an unobservable tail saving).

## Open questions

- Hit-rate broke its long-standing cap only by deepening the cascade, at steep
  put-p99 cost. Is any residency above the cap reachable at a tolerable tail?
- Is there a conflict-resolution path that does *not* terminate in the stash,
  so the put-tail win and the get-median cost finally decouple?
- The put-p99 tail resists every probe-side lever and appears owned by
  eviction/cascade work — is that cascade reachable without disturbing hit-rate?
- With get-p50 at a hard hasher/probe floor and cascade paths noise-bound, is
  any op-latency lever left above run-to-run variance?

## Per-policy notes

- ClassicLRU (active eval policy) is purely positional: it ignores `timestamp`
  and the LFU counter, making per-entry metadata pure overhead, and it makes
  MRU placement and promotion-gate shape substitutes rather than additive
  levers — placement already captures the protectable warm mass.
- On the dash segment, the median get resolves in the **stash**, not the
  promoted tiers, so it is hasher/probe-bound; stash length and scan direction
  do not move it. The stash is a shared resource that couples put-side cascade
  work to get-side median latency — any cascade change must be scored on both
  ops at once. The demotion cascade is a genuine latency↔hit-rate dial, not
  pure overhead, so "hit-rate fully capped" is not exact in this machinery.
