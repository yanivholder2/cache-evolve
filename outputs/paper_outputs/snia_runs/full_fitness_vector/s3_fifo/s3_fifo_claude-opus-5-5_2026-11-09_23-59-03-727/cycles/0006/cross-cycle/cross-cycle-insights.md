## Recurring wins

- Changing one thing per variant is what has produced the real gains. Each time changes were bundled (cycles 1–2), the result was a trade-off between hm_0 and web_0. Once changes were isolated (cycle 3 onward), wins appeared where neither trace regressed.
- Main should only credit reuse that happens inside main. Two filters do this: ghosting a main victim only if it was hit while resident, and resetting freq/reuse when a key graduates from small to main. They each won in cycle 3 and added up cleanly when merged in cycle 4. That merged design is still the core of the parent.
- Spend the ghost budget selectively, not by making the ghost bigger. Filters that leave stale entries out of the ghost lengthen the useful history at no cost.
- Structural changes that leave hit rate untouched can earn real timing credit: slab plus hash indexes (cycle 2) and the Fx hasher (cycle 3). These are the only per-op-time gains the diffs themselves explain.
- Small main-retention knobs pay off mainly on hm_0 at the small cache size. The MAX_FREQ 7 parent from cycle 5 was a slight gain, and nothing since has replaced it.

## Recurring failures

- Resizing the small queue in either direction does not help. Fixed ratios (cycle 1), shrinking to 10% (cycle 5) and growing to 1/3 (cycle 6) all lost. The 1/4 share is a local optimum.
- Adaptive ARC-style splitting regressed in cycle 1.
- A larger or re-split ghost does not help. Region splits (cycle 2) and a ghost sized to full capacity (cycle 4) let stale reuse back into main, where it pushes out live residents, mostly on web_0.
- Loosening graduation or admission keeps costing web_0. Examples are the `freq >= 1` threshold (cycle 2), an extra reinsertion inside small (cycle 6), and giving freq credit to keys re-admitted from the ghost (cycle 6).
- Index-layout work on probe count or load factor made tails worse. This covers merging the indexes into one map (cycle 4) and pre-sizing indexes to 2x occupancy (cycle 5). Memory footprint and locality matter more than the number of lookups.
- An eviction loop whose work per miss is unbounded (multi-pop, cycle 6) inflates the `put` p99.
- Timing shifts in policy-only diffs have been noise in every cycle. The per-entry metadata metric stayed flat throughout and never reflected ghost or index heap growth.

## Open questions

- web_0 at the large size is the last persistent gap. Probation length, small share and ghost bound have all been ruled out. The remaining lead is the rule for admitting keys from the ghost into main. A stricter rule (for example, re-admitting small-demotee ghost hits to small instead of main) is untested.
- hm_0 at the small size is stuck under changes that only act on ghost hits. Two untested levers act on every main eviction: the main-eviction sweep itself, and the graduation threshold (`freq > 1`).
- Could cycle-6 v003's large-size hm_0 gain be kept without its web_0 cost, for example by giving ghost-hit credit only when the ghost hit comes soon after eviction?
- Making the ghost index denser (fingerprints or compact keys) to cut the `put` tail is still untested. So is a compact unified slot (a `u32` index plus a flag bit).
- Should heap bytes per resident key be added alongside the metadata metric? And is a same-environment baseline needed before any timing credit?

## Per-policy notes

- S3-FIFO is the only policy in the ledger so far. The current parent is: Fx slab+hash, reuse-gated main-victim ghost with bound ≤ `main_size`, reset on graduation, `freq > 1` graduation threshold, small share 1/4, MAX_FREQ 7.
- On S3-FIFO, hm_0 responds to anything that strengthens main retention or main-victim history, and most of its remaining headroom is at the small cache size. Main retention looks nearly saturated at the large size.
- On S3-FIFO, web_0 is hurt by any loosening of graduation or ghost→main admission, and its large-size gap has resisted every structural change tried so far.
- On S3-FIFO, the ghost already serves as the second chance for demoted keys, so extra rounds inside small duplicate it and push out newcomers.
