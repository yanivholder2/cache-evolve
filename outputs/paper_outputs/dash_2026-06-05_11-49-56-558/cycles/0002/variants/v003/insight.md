# Variant v003 Insight

## Helped

*(none)*

## Hurt

- hit-rate: The per-hit O(n) counter decay — every bucket access decrements all co-resident items' `lfu_counter` by 1 — is too aggressive for workload-a's access pattern, driving moderately-accessed items toward zero and evicting them before they can contribute hits that ClassicLRU's recency ordering would have preserved.

## Takeaway

Passive aging via symmetric decay collapses the counter spread too quickly on workload-a: items that are only occasionally hot get their counts drained by their more-popular bucket neighbours before re-access, so AgingLfu produces a worse eviction ordering than the baseline's plain recency rule. The result suggests workload-a rewards retaining mildly-warm items (which LRU does naturally) more than it rewards frequency discrimination with aging, and that the −1-per-hit-to-all-others decay rate is too coarse for this trace's access distribution.
