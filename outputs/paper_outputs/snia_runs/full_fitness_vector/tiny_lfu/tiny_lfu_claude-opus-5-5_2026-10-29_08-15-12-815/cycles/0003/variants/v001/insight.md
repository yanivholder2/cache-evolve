# Variant v001 Insight

## Helped

- hit-rate: Hit-rate rose on every trace/size pair, and the gain was largest at the 16384 size on both traces, as the hypothesis predicted. In the parent, u32 counters never decayed, so a probation victim that was hot long ago kept winning the `cand_freq > victim_freq` contest against new arrivals. Now `reset()` halves all counters every `capacity * 10` effective additions, and nibbles saturate at 15. Together these limit how long old frequency can block newcomers, which matters most when the probation segment is small. At 131072 the gain on hm_0 is close to zero. With a large main region, stale victims were not the bottleneck there.
- metadata-size: The sketch heap at cap=1024 dropped to one eighth of the parent's. This is the drop the hypothesis said to expect. It comes from storing 4-bit nibbles packed into 64-byte `Block`s instead of four rows of u32, with the block count rounded to a power of two. `Node<u64,u64>` and the cache header did not change, because residency code was not touched. The dimension's summary (node size) therefore doesn't show this gain.
- per-op-time (put p99): Tail put latency improved. Each eviction does two `estimate_hashed` calls plus an `increment_hashed`. Each of these used to touch four scattered rows, and now touches a single cache line selected by `h >> 32`. At the 16384 size the whole sketch also shrinks to one eighth, so it is far more likely to stay in cache.

## Hurt

- per-op-time (get p50): Median `get` got slightly slower, and `get` p50 is the dimension's summary. The parent did one blind `+= 1` per row. On every `get`, `increment_hashed` now reads each nibble, compares it against 15, does a conditional add, and updates the `additions`/`sample_size` counter. On a path this short, that extra work shows up more than the locality gain does. The hypothesis's "cut put latency" claim also held only at the tail: put p50 did not move.

## Takeaway

Swapping only the sketch, with residency left alone, combined as cleanly as the cycle-0002 merge did. Aging fixed the small-cache problem of stale probation victims blocking new arrivals, and a smaller sketch was a side benefit. The cost moved to the per-access increment. The cache-line layout helps tail latency, but the median `get` now pays for saturation checks and sample counting. That makes the increment path, not the sketch layout, the place to optimize next. One option is a branch-free saturating add, done once per word, so `get` stops paying for the 4-bit design.
