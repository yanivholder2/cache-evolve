# Variant v001 Insight

## Helped

- per-op-time: Replacing the `reuse: Vec<bool>` with a packed `reuse_mask: u64` removed the second heap-vector element shift on every hit and the linear `for … if !reused` eviction scan — the two operations that had pinned `get` p99 and both `put` percentiles a bucket above the floor — so those tails dropped to the 100ns timer floor.

## Hurt

- (none) — no dimension regressed.

## Takeaway

Porting v002's constant-time `mask_remove`/`mask_insert_set` splice and single `trailing_zeros` eviction lookup onto v001's FastHasher bucket held the 0.7272 hit-rate and 16-byte metadata bit-for-bit (the mask mirrors the exact index-aligned single-element relocation) while collapsing the heap-vector tails to the floor — confirming that on this cache the reuse-set *representation*, not the hash, was v001's only observable latency lever. The orthogonal compose delivered the predicted Pareto win: record hit-rate now sits at the latency floor.
