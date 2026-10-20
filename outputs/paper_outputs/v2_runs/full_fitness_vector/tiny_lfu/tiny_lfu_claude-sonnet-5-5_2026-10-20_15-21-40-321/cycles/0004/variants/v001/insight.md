# Variant v001 Insight

Note: I could not read or diff the overlay source in this session. This insight comes from the hypothesis and the two fitness vectors, not from a code diff.

## Helped

- per-op-time: Both `get` and `put` p50 and p99 dropped substantially. The change swapped the default SipHash `HashMap` hasher for a cheap multiply-fold hasher. Every get and put does one or two map lookups, so the hashing cost was a large share of per-op latency. The MRU short-circuit in `get` (skip `unlink`+`push_back` when the node is already the tail) also removes list relinking on repeated hits.
- metadata-size: `TinyLfuCache` itself got smaller, even though the hypothesis expected no change. `Node` is unchanged. The likely cause is that `RandomState` (two `u64` keys) was replaced by a zero-sized `FoldBuildHasher`, which removes the per-map seed state from the struct. The sketch heap is untouched.

## Hurt

- Nothing moved for the worse. Hit-rate is unchanged, as intended, because the sketch, admission rule and LRU order were not touched. The MRU skip leaves recency order the same, since the node is already last.

## Takeaway

Hashing, not list layout, was the main latency cost in this cache. Replacing SipHash recovered more than the per-op time that the previous cycle's `u32` link layout had lost, and it kept that layout's size gain. The result also suggests the map hasher should be tuned before further link-layout tweaks.
