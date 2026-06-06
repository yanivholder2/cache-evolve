# Variant v002 Insight

## Helped

- per-op-time: Replacing SipHash with the zero-sized `FastHasher` (one xor + wrapping-multiply over the 8-byte key) strips the dominant hash cost from every lookup, and the new `idx == prot_tail` / `idx == seg_mru` short-circuits skip the `unlink`/`push_back` pair on protected-MRU gets and same-segment-MRU put-updates — together they pull both get and put p50 well below v001's segment-bookkeeping penalty.
- metadata-size: Swapping the default `RandomState` for the stateless `FastBuildHasher` removes SipHash's per-map seed words, trimming the `LruCache` struct footprint (the per-node `Node` size, the dimension summary, is unchanged).

## Hurt

- (none) — hit-rate stayed at v001's record because `prot_cap` geometry, probation-first eviction, and demote-on-overflow were left exactly as inherited.

## Takeaway

v001's latency regression was not intrinsic to segmentation — it was SipHash cost plus redundant list surgery on touches that don't change order. Swapping in a cheap integer hasher and fast-pathing already-MRU hits recovers v003-class latency while the untouched SLRU policy defends the 0.724 hit-rate, showing the hit-rate lever (policy) and the latency lever (hashing + no-op fast path) are orthogonal and can be optimized independently.
