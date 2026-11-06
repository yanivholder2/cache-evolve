# Variant v003 Insight

## Helped

- hit-rate: Swapping ClassicLRU's exact recency order for CLOCK's single reference bit (reusing `Item::lfu_counter`) did not cost hit-rate; it went up slightly on three of the four trace/size cells, at both 16384-slot configs and hm_0 at 131072. On these MSR traces, second chance seems to protect re-referenced items about as well as strict LRU, and a little better when the cache is small.

## Hurt

- hit-rate: web_0 at 131072 slipped by a negligible amount. This fits CLOCK's coarser approximation: once a large bucket has many bits set, the sweep's victim is close to FIFO order rather than the true LRU item.

## Takeaway

H03's main claim is unverified. The goal was lower get/put per-op-time from removing the `Vec::remove`+`push` memmove on hits and the `remove(0)` shift on evictions. But v003's fitness-vector.json reports only hit-rate, with no per-op-time and no metadata-size. What this variant does show is the cost side of the trade: moving from LRU to an in-place CLOCK, with the reference bit stored in an existing `Item` field, leaves hit-rate unchanged or slightly better. That makes v003 a good candidate to re-benchmark for latency. Note one structural change: the bucket struct gains a `hand: usize` field, though `Item` stays the same size.
