# Variant v001 Insight

## Helped

- (none) — the single-entry last-key cache was meant to drop get p50 below the parent by skipping the HashMap probe on repeated hot-key gets, but it never paid off; hit-rate held flat (no algorithmic change), and the targeted latency win did not materialize.

## Hurt

- per-op-time: the new fast path prepends a guard chain to every get — an `Option<K>` key compare, the `prot_tail == last_idx` check, a `nodes[last].key == *key` re-check, plus a `key.clone()` to re-prime `last_key` on each promoting/MRU return — all executing ahead of the map lookup; this overhead landed on the common path instead of saving it, and get p50 more than doubled rather than falling. put p50 regressed in lockstep even though `put` was untouched, consistent with the added fields and guard perturbing hot-path codegen rather than any change to put itself.

## Takeaway

Caching one key in front of the lookup is a net loss here because the thing it bypasses — a zero-sized `FastHasher` xor-multiply plus an arena index probe — is already cheaper than the Option-compare-plus-clone guard added to reach the shortcut. The HashMap probe was never the get bottleneck, so short-circuiting it adds branches and a key clone to the hot path without removing comparable work; the `Node` footprint stayed 24 bytes as intended (the fields live on `LruCache`, growing it 96→120 bytes), but that layout discipline bought nothing once the latency goal was refuted.
