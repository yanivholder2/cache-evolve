# Variant v003 Insight

## Helped

- per-op-time: get p50 and p99 both dropped sharply. Hashing a u64 key with an inline multiply-rotate instead of SipHash removes most of the fixed cost of a lookup, which is nearly all of a get on the tagged index. The tail also shrank, so the parent's get-tail penalty came from hashing and probing cost, not just index size.
- per-op-time: put p50 improved for the same reason. Each put does a tagged-index lookup and update, and the cheaper hash is paid on both.
- metadata-size: the S3Fifo struct got slightly smaller. The Fx hasher is a zero-sized or tiny state, while the default hasher carries random keys in the map. The per-entry size did not change.
- hit-rate: unchanged, as intended. The hasher, pre-sizing and `#[inline]` hints touch no eviction, promotion or ghost logic.

## Hurt

- per-op-time: put p99 got much worse even though put p50 improved. Pre-sizing to about 1.25x capacity makes the map large and sparse, which probably costs cache and TLB misses on cold inserts. A multiplicative hash also clusters sequential or structured keys, which lengthens probe runs on some inserts. Both would show up in the tail rather than the median. This is a hypothesis; I could not isolate the cause.

## Takeaway

Hash cost dominated the hot path of the tagged index, so swapping the hasher cut get latency at no cost to hit rate. The put tail shows that a weak hash plus an oversized pre-allocation can trade a better median for a worse worst case. The next step is to separate the two changes: keep the hasher, and test a tighter pre-size or a better-mixing finalizer.
