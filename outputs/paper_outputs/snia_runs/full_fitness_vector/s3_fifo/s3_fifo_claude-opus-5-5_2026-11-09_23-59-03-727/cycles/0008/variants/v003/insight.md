# Variant v003 Insight

## Helped

(No dimension improved.)

- hit-rate: Not an improvement, but the outcome the hypothesis predicted. Every trace and size matches the parent exactly. The 32-bit fingerprints never caused a spurious ghost hit that changed a measured hit rate. The `u32` seq wraparound with `retain` compaction at `2 * main_size` kept the same liveness semantics as the `u64` seq. So swapping full keys for fingerprints is safe for policy at these cache sizes.

## Hurt

- per-op-time: `put` got slower, mostly in the tail (p99), with a smaller rise at p50. That is the opposite of what the hypothesis predicted. For `u64` keys the parent's `key.clone()` was a free register copy, so dropping it saved nothing. Meanwhile every `put` miss now calls `fingerprint()`, which runs a full Fx hash and a 64→32 fold, and then `ghost_index` hashes that `u32` again with `FxBuild`. Every ghost probe now pays two hashes instead of one. `evict_small` and the reused branch of `evict_main` add another `fingerprint()` call per ghosting, on top of the insert's own rehash. The halved ghost tuple size did not make up for this extra work. `get` is unchanged in code, so its slight drift is noise.

(metadata-size did not move. `Entry<u64,u64>` is still 24 bytes. The `S3Fifo<u64,u64>` header stays the same size because the `VecDeque`/`HashMap` headers are unchanged and `ghost_seq: u32` is padded away. The ghost heap footprint did get smaller, but this dimension does not measure it.)

## Takeaway

Fingerprinting the ghost structures does not change policy, but for `u64` keys it is a net loss on time. The key is already 8 bytes and `Copy`-cheap, so the savings are only a smaller ghost footprint. The cost is an extra hash on every miss, plus a second hash at each ghosting, and that cost lands in the `put` tail. If ghost fingerprints are tried again, the table should use the fingerprint directly as its hash, for example with an identity/pass-through `BuildHasher` on `HashMap<u32, u32>`. Otherwise the change only makes sense for large or expensive-to-clone `K`, which the current traces do not use.
