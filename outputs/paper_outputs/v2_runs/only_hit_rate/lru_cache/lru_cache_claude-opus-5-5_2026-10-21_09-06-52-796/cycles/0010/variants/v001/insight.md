# Variant v001 Insight

## Helped

(No measured dimension moved. `hit-rate` matches the parent exactly, and H01 predicted this outcome; see Takeaway. The intended gain is in `metadata-size`, which the harness does not report yet, so this variant can't be scored on it. In structural terms, the ghost no longer owns any copies of `K`. `ghost_q: VecDeque<K>` and `ghost_set: HashMap<K, ()>` stored every ghosted key twice. They are replaced by a preallocated `Vec<u64>` ring of `ghost_cap` slots plus a `HashMap<u64, u32>` count map. `evict_one()` now fingerprints and removes keys through borrows, so the `node.key.clone()` is gone from both the `small` and `main` eviction paths. Ghost memory is now fixed per slot and no longer grows with key size.)

## Hurt

(No measured dimension regressed. The change does add hidden costs. Every `put()` miss now computes a `RandomState` hash for the ghost probe, on top of the hash done for the `map` lookup. Every small-queue eviction hashes the evicted key once more. A 64-bit fingerprint collision would turn an unrelated key into a false ghost hit. With roughly 332 live fingerprints at size 2048, that risk is negligible, but it is not zero, whereas the old key-based set had no collisions.)

## Takeaway

The port is policy-neutral, and the reason goes beyond the matching hit rate: the ring copies the parent's quirks one-for-one.
- In the parent, a ghost hit removed the key from `ghost_set` but left a stale entry in `ghost_q`. In the ring, the hit decrements `ghost_count` but leaves the fingerprint in its slot.
- So in both versions, stale entries still count against `ghost_cap`.
- In both versions, overwriting or popping a stale entry can expire a key that was ghosted again more recently.

So the only real change is how the ghost is stored, not how ghosting decides. That makes this variant a safe base to replace cycle 9 v001. The ghost-path credit experiment that cycle 9 recommended can now be built on it, and any later hit-rate movement can be attributed to policy rather than to the ghost representation.
