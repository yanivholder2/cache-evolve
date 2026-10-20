# Variant v001 Insight

## Helped

(No dimension improved. hit-rate and metadata-size match the parent exactly, which is what H01 predicted. Only the order of checks inside `promotion_get` changed. By the uniqueness invariant, exactly one branch can match for any key, so the sequence of cache mutations is the same. No fields or types were touched, so `Item` stays at 40 bytes. The unchanged hit-rate is a correctness check that passed, not a gain.)

## Hurt

- per-op-time: `get` p50 went up instead of down. This meets H01's own falsification condition. Reordering to target → probing → stash with a lazy `stash_idx` saves one stash hash and one stash-bucket scan per normal-bucket hit. But every stash hit now pays for two failed normal-bucket scans (target, then probing) before the stash is reached. Misses scan all three buckets under either order, so they gain nothing. The median got worse, which means stash hits are common enough on this lineage that the order is a net loss. That fits v003's design: fresh puts land in the stash, and a key's first reuse is always found there. The small rise in `get` p99 points the same way. The small `put` movements have no cause in this diff (`put` was not touched), so they are most likely run-to-run noise.

## Takeaway

Lookup order is not a free win. The best order depends on where hits land, and that depends on the placement policy. On the v003 lineage, fresh arrivals enter the stash and the stash protects reused items, so many get hits are stash hits. On that hit mix, stash-first beats target-first, even though stash-first costs an eager stash hash for every lookup. The gain measured in cycle 1 does not carry over to this cache state. To lower `get` cost here, try changes that make each bucket scan cheaper (or skip scans, e.g. with a fingerprint/filter), not a different fixed order.
