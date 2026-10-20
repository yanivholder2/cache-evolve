# Variant v002 Insight

## Helped

- per-op-time (`put`): Each miss that ghosts a small-queue victim no longer clones `K` into two structures or bumps a stamp. Eviction from the ghost is now one overwrite of the head slot, where v003 ran the `ghost_pop_oldest_live` loop over stale records. The O(`ghost_cap`) `retain` compaction is gone, which is why `put` improved at both p50 and the tail.
- per-op-time (`get`, the summary): The `get` body does the same work as v003's: one `index` probe, one slab access, a freq bump. Change 4 only reshaped it (`match` instead of `as_mut()?`). The gain must therefore come from the new `#[inline]` hints on `S3Fifo::get` and the `framework::Cache` wrapper, from the smaller ghost no longer competing for cache lines with the interleaved `put`s, or from both. This variant cannot tell those causes apart.
- hit-rate: The tiny rise is not noise from fingerprint collisions. The ring changes what the ghost remembers. In v003, a ghost hit freed a live slot and the ghost kept the last `ghost_cap` *live* records. In the ring, a `ghost_take` leaves a zero hole that still uses up a ring position, so the ghost only covers the last `ghost_cap` *pushes*. As a result, the ghost holds a subset of what v003 held. Ghost hits and the small→main shifts they trigger become slightly rarer, and that nudged the adaptive split in a favourable direction on this trace.

## Hurt

- metadata-size: The summary did not move because it measures only `Entry<K,V>`, which was left unchanged. The `S3Fifo` struct row shrank a little, mainly because `ghost_stamp` was dropped. The real gain is in the ghost's heap memory, which this dimension does not see. A ghost record used to be up to two `(K, u64)` deque records plus a `(K, u64)` map entry. It is now 8 bytes of ring plus a `(u64, u32)` map entry, allocated once up front. That saving grows with key size, but `u64` keys in the eval hide most of it.

## Takeaway

Replacing the key-cloning, lazily-deleted ghost with a fixed ring of fingerprints is a clear latency win, mostly on `put`, where the clones and the `retain` compaction lived. But it is not policy-neutral, contrary to what H02 claimed. Holes left by `ghost_take` shorten the ghost's effective memory to "last `ghost_cap` evictions", so later variants should treat ghost size and hole handling as a policy knob, not a pure implementation detail. Also, the hypothesis's suggested diagnosis (double-counted re-pushes) is not the cause: the code already handles that case.
