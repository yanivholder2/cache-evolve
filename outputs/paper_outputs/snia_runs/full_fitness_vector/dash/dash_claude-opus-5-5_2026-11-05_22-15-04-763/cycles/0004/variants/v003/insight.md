# Variant v003 Insight

## Helped

- per-op-time (`put` p99): When a stash hit's target is full but its probing bucket has room, the item now goes to probing. That path skips the chain of evicting a target resident and pushing it into stash at MRU. Stash buckets stay less crowded, so the slow tail of `put` (which inserts into stash) shrinks as the hypothesis predicted.
- hit-rate: In the overflow case, no target resident is evicted to make room for the promoted item, so warm items in target stay cached. The gain is small but shows up in every trace/size cell, so overflow does not cost any retention.

## Hurt

- per-op-time (`get` p99): Overflowed items now sit in a neighbour's probing bucket. `promotion_get` scans stash, then the full target, and only then probing, so each later hit on such an item pays for two missed scans first. Overflowed items also fill the neighbour's probing bucket. When that neighbour later promotes its own hottest item, it evicts one of them. The evicted item then takes the foreign-resident retry into its full true target, and that target's LRU cascades into stash. This moves the cascade cost from `put` onto a later `get`, and `get` p99 regresses clearly.
- per-op-time (`put` p50): This barely moved even though `put` was not edited, so it is most likely noise between runs rather than an effect of the change.

## Takeaway

On the ungated line, probing overflow does not remove the stash-hit cascade. It defers it: `put` tail latency improves, but `get` tail latency degrades because overflowed items are found last and then become foreign residents that a neighbour's promotion evicts. This gives up v003's main advantage, its low `get` p99, for a smaller `put` p99 gain. So without a gate to slow the rate of promotions, overflow is not a better route to cheaper stash hits. Metadata size is unchanged because no per-item state was added.
