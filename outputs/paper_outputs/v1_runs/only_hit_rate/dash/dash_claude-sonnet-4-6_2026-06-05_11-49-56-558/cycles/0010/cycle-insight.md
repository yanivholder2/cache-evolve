# Cycle 0010 Insight

## Cohort outcome

This cycle tested three attempts to extract ranking value from `lfu_counter` beyond its existing binary zero-count guard: using it as a fallback eviction tiebreaker, using it as a promotion selector, and crediting it on the stash-overflow second-chance path. All three either regressed or were neutral; v003 (the overflow credit) won only by producing no harm while the other two hurt.

## Patterns

- **`lfu_counter` is a binary guard, not a ranking signal.** Both frequency-guided eviction (v001) and frequency-guided promotion selection (v002) regressed. On this Zipfian workload, LRU position already correlates strongly with coldness; treating a low counter as equivalent to "cold" introduces noise by confusing newly-inserted items (low counter because new) with genuinely cold ones. The counter is useful only to distinguish zero-access items from any-access items — once that threshold is crossed, recency dominates.
- **The stash-overflow second-chance path does not mirror the stash-hit path.** The Cycle 0009 stash-hit credit worked because that path selects items that just received a genuine access. The overflow path selects the coldest LRU victim in an overflowing stash bucket — an item with no recency evidence. Crediting its counter before target insertion confers protection it has not earned, but also fails to change outcomes because the protection gap on this path either fires rarely enough or involves sufficiently cold items that it is not a measurable loss site.
- **The `lfu_counter` optimization axis is now exhausted.** Prior cycles extracted value from the counter as a zero-count eviction guard and as a pre-insertion credit on confirmed-access paths. This cycle explored every remaining way to use the counter as a ranking signal and found none that help. The binary guard is the right abstraction for this mechanism.

## Open questions

- With geometry and `lfu_counter` mechanics both exhausted, what qualitatively different loss mechanism remains — long-lived item counter staleness (soft-cap), structural asymmetry in probing-bucket demotion handling, or something outside the normal/stash pipeline entirely?
- The soft-cap hypothesis (preventing `lfu_counter` from accumulating unboundedly high values that crowd out newly warm items) has been mentioned across multiple cycles but never tested — does it actually address a measurable loss site, or is staleness not the binding constraint at 7/8 geometry?

## Continuation

The `lfu_counter` axis is closed: it works as a binary threshold and as a pre-insertion credit on confirmed-access paths, and further attempts to use it as a ranking signal should not be revisited. The next hypotheses-writer should pivot to the one remaining untested thread carried forward across multiple prior cycles: soft-capping `lfu_counter` to prevent long-lived hot items from accumulating stale high counts. If that also proves neutral, the next layer of investigation is qualitatively different loss mechanisms unrelated to counter management — for example, demotion handling or insertion ordering in the probing bucket.
