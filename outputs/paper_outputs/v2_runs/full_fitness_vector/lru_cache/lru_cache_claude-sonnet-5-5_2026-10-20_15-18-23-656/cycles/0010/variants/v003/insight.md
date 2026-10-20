# Variant v003 Insight

## Helped

- hit-rate: Second-chance eviction with a ghost-hit-driven `walk_cap` keeps recently re-referenced entries from being evicted, giving a small gain over plain LRU.

## Hurt

- metadata-size: Porting second-chance and ghost tracking into the baseline added per-node state (reference bit) and ghost/counter bookkeeping to the cache struct. Both `Node` and `LruCache` grew, despite the hypothesis expecting the node layout to stay unchanged.
- per-op-time: Put p99 rose well above the baseline, the opposite of what the cap was meant to do. The eviction walk, ghost lookups and the every-256-puts window check add work on the put path. Get p50 also rose, which suggests extra reference-bit or ghost bookkeeping on the hot path.

## Takeaway

Making the walk cap adaptive did not recover the put-tail gain that a fixed cap gave. The ghost-list and reference-bit machinery needed to drive adaptation costs more per put than the capped walk saves, and it grew the metadata. A cap that needs a feedback signal has to get that signal almost for free, or the adaptivity costs more than the walk it bounds.
