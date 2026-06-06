# Variant v002 Insight

## Helped

- (none) — hit-rate held flat because the side table only short-circuits the lookup path and leaves promotion, eviction, and geometry untouched, so resident-set behavior is identical to the parent.

## Hurt

- per-op-time: the `get` path got slower, not faster — every resolved get now clones the key into the round-robin table (`record_hot`) and each lookup linearly scans up to four `Option<K>` slots, which is strictly more work than the single xor+multiply `FastHasher` probe it was meant to bypass; `put` p50 also rose despite being byte-identical, consistent with the near-doubled `LruCache` struct hurting cache locality.
- metadata-size: the scored `Node<u64,u64>` stayed at 24 bytes as designed (the table lives on the cache struct), but the four `Option<K>` keys, four `u32` indices, and cursor nearly doubled the `LruCache` footprint — a real footprint cost the scored summary does not capture.

## Takeaway

When the baseline probe is already a one-multiply `FastHasher` lookup, there is essentially nothing left to save by bypassing it, so a hash-free side table that clones a key and scans four slots on every get is pure added cost. H01's one-entry-vs-four-entry framing missed the real constraint: the HashMap probe was never the get bottleneck, so any structure layered in front of it to skip it only makes the fast path heavier.
