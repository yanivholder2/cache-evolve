# Variant v003 Insight

## Helped

- metadata-size: the gate reuses the existing `lfu_counter` field on `Item` to distinguish first-hit from repeat-hit slots, so no new per-item state was added and `Item<u64,u64>` stayed at its baseline width.

## Hurt

- hit-rate: throttling the first hit to a single-slot swap leaves a freshly-touched item only one position above the eviction front, so any short burst of bucket inserts before its second hit pushes it out — the one-step buffer is not deep enough headroom on this trace, and baseline's full move-to-back was doing real work even on first hits.

## Takeaway

For ClassicLRU buckets at this trace/size, "protect the just-touched slot from the *very next* eviction" is a weaker invariant than it looked in the v001 post-mortem: many hot items see several intervening bucket inserts between their first and second hit, so reach-throttled promotion lets them die before they ever earn the full move-to-back. Promotion distance, not just promotion presence, drives ClassicLRU's hit rate on Dash-style fixed-capacity buckets, and any first-hit gating needs to look at the bucket's recent insert pressure — not the slot's hit history alone — to decide how far to move.
