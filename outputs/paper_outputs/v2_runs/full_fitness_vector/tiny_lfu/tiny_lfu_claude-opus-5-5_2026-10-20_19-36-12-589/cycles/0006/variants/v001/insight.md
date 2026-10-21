# Variant v001 Insight

## Helped

- metadata-size: Taking `prev`/`next` out of `Node` leaves just `{ key, value }`, so the headline per-entry `Node<u64,u64>` shrinks by 8 bytes as the hypothesis predicted. Recency is now kept in a shared bitmap at one bit per slot (`refbits`), not in per-node pointers.
- hit-rate: Swapping strict LRU order for CLOCK's coarser second-chance order did not hurt admission quality. The hit rate is flat to marginally up, which supports the rationale that victims at the LRU end already had near-equal sketch estimates, so which one the duel faces barely matters.
- per-op-time: A `get` hit is now one bit-OR on a small bitmap with no writes to `nodes`, and that tightened the `get` tail. `put` p50 also dropped because admitting a newcomer overwrites the victim slot in place instead of unlinking it and pushing it back.

## Hurt

- per-op-time: The headline `get` p50 did not move. The `HashMap` lookup and the 4-row `increment_only` still dominate the typical hit, and the pointer relinking that was removed was never on that median path. `put` p99 got worse: a full-cache insert now has to run the CLOCK sweep, which can clear a long run of set ref bits before it finds a victim. This is an unbounded walk that the O(1) LRU head pop never needed.
- metadata-size: The inline `TinyLfuCache` struct got bigger because it now carries the `refbits` `Vec` header and the `hand` field. This is a fixed cost per cache, not per entry, so it doesn't show in the headline number.

## Takeaway

LRU→CLOCK is close to a free win on memory: 8 bytes less per entry with no hit-rate penalty, which shows that admission under this TinyLFU duel doesn't care about exact recency order. It did not move median `get` latency, though. Relinking on a hit was a tail cost, not the median bottleneck, which points to the map lookup plus sketch increment as what's left to attack. The new risk is the sweep's tail cost on `put`, which could be bounded by scanning a whole `u64` word of ref bits at once (e.g. with `trailing_ones`) instead of one bit at a time.
