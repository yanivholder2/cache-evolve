# Variant v003 Insight

## Helped

- per-op-time: replacing the baseline's per-hit unlink+push_back pointer surgery with a single `referenced = true` boolean write shrank both `get` and `put` latency, helped further by the zero-sized `FastHasher` removing SipHash from every map probe.
- metadata-size: dropping the doubly-linked recency list (two `usize` links per node) collapsed `Node<u64,u64>` from the 48-byte to the 24-byte class and shrank the `LruCache` struct, since recency now lives in a single in-node bit plus one `hand` pointer instead of `head`/`tail`.
- hit-rate: CLOCK's second-chance bit approximated LRU closely enough on this short-reuse trace that hit-rate did not regress and edged slightly above the exact-LRU baseline.

## Hurt

## Takeaway

On this trace, exact LRU recency tracking buys nothing: a single-bit CLOCK approximation with no recency list matches LRU's hit-rate while cutting both latency and per-node footprint, because every per-hit pointer mutation the baseline paid for was pure overhead.
