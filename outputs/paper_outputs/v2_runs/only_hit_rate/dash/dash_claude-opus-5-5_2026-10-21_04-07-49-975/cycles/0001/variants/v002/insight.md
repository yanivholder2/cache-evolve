# Variant v002 Insight

## Helped

- hit-rate: Sending a stash hit to a free probing slot when its target bucket is full avoids evicting the target's LRU item into stash. That keeps a little more of the working set resident. The gain over baseline is only marginal, which suggests the "target full, probing free" case seldom fires on this trace.

## Hurt

(None measured. This variant's eval reported only `hit-rate`. `metadata-size` and `per-op-time` were not evaluated, so the cost of the two extra `is_full()` checks on the stash-hit path is unknown.)

## Takeaway

This change only behaves differently from baseline in one case: a stash hit whose target bucket is full while its neighbouring probing bucket has room. In every other case it does the same thing as `promote_to_target`, because a put into a non-full target never evicts anyway. The hit-rate barely moved, so at this cache size the normal buckets are mostly full together in steady state. Spreading promotions across neighbouring buckets adds little when there is almost never a free legal slot to use. Future hypotheses should change what gets evicted once both buckets are full, not where an item lands when a slot is free.
