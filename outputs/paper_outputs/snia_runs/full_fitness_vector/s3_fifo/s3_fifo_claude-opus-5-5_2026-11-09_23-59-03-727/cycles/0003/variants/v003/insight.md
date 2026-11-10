# Variant v003 Insight

## Helped

- per-op-time: `get` is one `index.get` plus a slab access, so the hash function was most of its cost. Swapping SipHash-1-3 for a single rotate, xor and multiply on the `u64` key lowered both the `get` median and its tail.
- per-op-time: `put` gained proportionally more than `get` because a miss chains several hash operations: `index.get`, `ghost_index.remove`, `index.insert`, `index.remove` in `release`, and the `ghost_index.get`/`insert` calls in `push_ghost`, including the `retain` compaction pass. The `put` p99 improved most, consistent with the `retain` sweep (one `ghost_index.get` per ghost tuple) now running much faster.
- metadata-size: The fixed `S3Fifo<u64,u64>` struct shrank. Each `HashMap` previously stored a `RandomState` holding two `u64` SipHash keys, and `BuildHasherDefault<FxHasher>` is zero-sized. The summary metric, the per-entry `Entry<u64,u64>` footprint, did not change.

## Hurt

(none: hit-rate is identical to the parent on every trace and size)

## Takeaway

Changing only the `BuildHasher` on the two indexes left hit rates bit-identical, which confirms the policy was untouched. All of the timing gain can therefore be credited to hashing cost. This is the remaining per-op bottleneck the parent's slab-plus-index design exposed, and `put` benefits most because a miss performs several hash operations. With hashing now cheap, the remaining `put` cost lies in the `key.clone()` calls, the slab writes and the ghost-deque bookkeeping. A future variant could try to cut the number of hash probes per miss, for example by using the `entry` API instead of a separate `get` followed by `insert`/`remove`.
