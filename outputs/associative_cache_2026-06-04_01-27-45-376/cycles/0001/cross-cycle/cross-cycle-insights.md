## Recurring wins

- Eviction-side discrimination beats access-side gating: leaving LRU
  promotion intact and only biasing *which* slot gets picked as victim
  has produced the cohort's Pareto leader so far.
- Reusing the existing `Item.lfu_counter` as a cheap recency/touched
  signal lets variants add discrimination without growing per-item
  metadata or changing the bucket trait's `&[Item]` eviction signature.

## Recurring failures

- Withholding promotion on first access (scan-resistance via delayed
  promote) regressed hit rate — the bucket is too narrow (~16 slots)
  for a hot newcomer to survive its vulnerable window unpromoted.
- Replacing exact LRU ordering with a coarser scheme (e.g., CLOCK's
  two-level recency) lost information the existing `remove`+`push`
  path was preserving, hurting hit rate without a compensating gain.

## Open questions

- Does extending the touched-bit to a multi-bit aging counter (with
  decay on the eviction sweep) keep paying off, or does the bucket's
  small size make further gradations noise?
- Would loosening the bucket trait's `&[Item]` eviction signature
  (so a reference bit can live on `Item` directly) unlock the
  per-op-time wins that side-channel CLOCK designs had to forgo?
- Is there a workload regime where a "must survive two hits" victim
  filter helps, or does it reintroduce the failure mode of killing
  genuinely hot newcomers?
- Can eviction-side filters be hybridised with insertion-placement
  tweaks without re-triggering the promotion-side regressions?

## Per-policy notes

- **ClassicLRU**: blanket move-to-back on every hit is the obvious
  attack surface, but the productive edit is on the eviction side, not
  the promotion side. Preserve the existing ordering; add a cheap
  filter that sacrifices never-touched slots before re-used ones.
- **CLOCK-style replacements for LRU ordering**: not competitive at
  this bucket width under the current trait signature; revisit only if
  the trait changes to let the reference bit live on `Item`.
