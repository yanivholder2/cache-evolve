# Variant v003 Insight

## Helped

- hit-rate: `demote_from_normal_bucket` now puts normal-tier victims, both direct and cascaded, at the stash MRU end (`InsertPosition::Back`) instead of index 0. Under v002's two-hit gate, a demoted item with its mark cleared now has a full stash residency to collect the two hits it needs to get back into its target, instead of being the very next stash victim. Hit rate rose on every trace and size. The largest gain was on hm_0 at the large cache size, where many recently hot items fall out of the normal tier and come back soon after. This is the case the LRU-end demotion was throwing away.
- per-op-time: `get` p50 (the summary statistic) improved slightly. This fits the hypothesis's point that `Vec::push` at Back avoids the element shift of `insert(0)` on every demotion into stash.

## Hurt

- per-op-time: tail latency got a little worse, more for `put` p99 than for `get` p99. The most likely cause is that demoted items now stay in stash longer. Stash buckets are fuller with proven items, so a `put` that lands in a full stash bucket evicts more often. v002 showed the same stash-pressure pattern, and this change pushes it further. The shifts are small, so benchmark noise is still a possible explanation.

## Takeaway

Under a two-hit stash gate, where demoted items land in stash matters a lot. Putting them at the LRU end with the mark cleared nearly guarantees that a recently hot item is lost. MRU demotion makes stash work like an SLRU probationary segment and recovers hit rate for free, with no change to `Item` size and no extra logic on the hit path. The remaining cost is stash occupancy pressure, which shows up as `put` tail latency, not as per-operation work.
