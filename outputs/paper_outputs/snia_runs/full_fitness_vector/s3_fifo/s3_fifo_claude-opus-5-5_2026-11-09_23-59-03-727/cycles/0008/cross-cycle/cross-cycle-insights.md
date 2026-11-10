## Recurring wins

- One change per variant is what made progress possible (cycles 3–5). Every bundled change in cycles 1–2 just moved the trade-off between hm_0 and web_0 around. Isolated changes pinned down mechanisms and let gains stack.
- Credit should come only from reuse inside main. Reuse-gated main-victim ghosting, resetting freq/reused at graduation, and keeping the `freq > 1` graduation threshold each helped alone (cycle 3) and added up when merged (cycle 4). This is still the core of the parent.
- Spending the ghost budget selectively beats making it bigger. Filters that free ghost slots lengthen useful history at no cost.
- Structural rewrites that leave hit rate bit-identical are the only trustworthy source of timing gains: slab plus hash indexes (cycle 2), then the Fx hasher (cycle 3).
- Current parent: cycle-5 v002 (Fx slab, both filters, `MAX_FREQ` 7 with decrement-by-one aging, ghost bound = `main_size`, small share 1/4). It has survived three cycles of challengers.

## Recurring failures

- Resizing the small queue fails both ways: fixed ratios (cycle 1), shrinking (cycle 5), growing (cycle 6). The 1/4 share is a local optimum.
- Changing the ghost bound or container hurts or does nothing. A larger bound lets stale reuse back into main (cycle 4). Separate ghost regions didn't help (cycle 2). A ring container (cycle 7) and fingerprint keys (cycle 8) both raised `put` cost.
- Changing ghost→main admission fails both ways: looser credit (cycle 6) and stricter re-probation (cycle 7) both regressed web_0 at the large size.
- Main-tenure knobs have hit their limit from both sides. Aging: halving hurt (cycle 7) and a cap of 15 hurt (cycle 8). Graduation credit: full carry-over and a credit of 1 both hurt. Tenure has to be earned in main.
- Extra rounds inside small (cycle 6) duplicate what the ghost already does and push out newcomers.
- Index layout tweaks (merged index, over-allocation) made tails worse. The `put` tail tracks hash computations and sweep steps per miss, not footprint or load factor.
- Policy-only diffs keep showing timing shifts their diffs can't explain. These are environmental noise and never get timing credit. Eviction work per miss must stay bounded, because the multi-pop loops and cascades tried so far inflated the tail.

## Open questions

- Is a small/main split that depends on cache size, aimed at 16384, the last untested hit-rate lever? If it fails, treat hit rate in this policy family as converged.
- Is web_0's large-size gap out of reach for this policy family? Probation length, small share, ghost bound and both admission directions have all been ruled out.
- Would fingerprint ghosts with a pass-through hasher (one hash per miss) finally beat the parent on `put`? More broadly, can a hash probe or a sweep step be removed from the miss path?
- Should the metadata metric add heap bytes per resident key? Ghost memory and index memory keep growing without the metric showing it.
- The benchmark environment still isn't fixed, so small per-op-time differences can't be trusted.

## Per-policy notes

- S3-FIFO (the only policy explored so far):
  - hm_0 responds to how main retains reused keys. web_0 is sensitive to probation and admission, and every change aimed at its large-size cell has failed.
  - At 131072, hit rate has been flat under every policy change since cycle 5, so main retention there is saturated. All remaining hit-rate room is at 16384, mostly on hm_0.
  - Per-entry metadata has stayed flat at 24 bytes across all cycles. New bits fit in padding or tag space.
