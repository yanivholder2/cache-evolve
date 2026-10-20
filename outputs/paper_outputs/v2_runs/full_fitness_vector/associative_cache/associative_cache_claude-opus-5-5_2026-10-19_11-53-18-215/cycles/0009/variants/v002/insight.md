# Variant v002 Insight

## Helped

None. metadata-size did not move because `Item` stays `{ key, value }`, the cache header was not touched, and the sketch lives inside each bucket rather than on each item.

## Hurt

- hit-rate: The sketch is far too small for its aging window, so it saturates and admits almost every key. It has `F ≈ max_size` counters and each miss bumps two of them. Halving happens only every `4 * max_size` misses, so each counter gets about 8 bumps per window. The estimate (the smaller of the key's two counters) therefore sits at or above `FREQ_ADMIT = 2` for nearly every tag. Both indices use only the low `log2(F)` bits of the tag (3 bits for an 8-slot bucket), so collisions are certain, not just likely. The result is that `seen` is true for most cold keys, which sends them to probation MRU again. That undoes v003's LRU entry for history-less keys. One-hit wonders once more outlive the just-demoted protected item and the real ghost-hit keys, which is exactly the protection v003 added.
- per-op-time (put): Every new-key SLRU put now does two more counter reads, a two-counter bump, and an O(F) halving pass every `4 * max_size` misses. More importantly, the saturated sketch moves most full-bucket cold puts off v003's cheap path (two `mem::replace` calls) and onto the MRU path. That path does `evict_front` (a `Vec::remove(0)` shift of `items` and `tags`) and then a `Vec::insert(len0)` shift. Tail latency rises more than the median, which fits shifts whose cost grows with bucket width.
- per-op-time (get): The get path's code is byte-for-byte unchanged, so this regression is indirect. Each bucket now holds an extra `Vec<u8>` header and a `u32`, plus a separate heap allocation per bucket. That spreads the bucket array over more cache lines. The churn also changes protected/probation occupancy, so the shape of hits changes too. Read this as a footprint/locality cost, not a change in what the get logic does.

## Takeaway

A count-min frequency filter used as an admission gate is only useful if it can tell keys apart. With roughly one counter per slot and an aging window several times the bucket size, nearly every tag clears a threshold of 2. The "frequency-qualified" branch then quietly turns back into "everyone enters at MRU", which is the behaviour v003 had just shown to be worse. Any retry needs counters ≥ the window size divided by the threshold. Either the window has to shrink, or there have to be many more counters per slot, or index bits have to be drawn from more of the tag. The threshold must also be checked against the saturation level before hit rate is measured. Otherwise the extra put-path cost buys nothing.
