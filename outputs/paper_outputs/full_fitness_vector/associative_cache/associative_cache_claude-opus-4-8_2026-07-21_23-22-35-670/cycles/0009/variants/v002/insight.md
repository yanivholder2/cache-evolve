# Variant v002 Insight

## Helped

- per-op-time: Folding the parallel `reuse: Vec<bool>` into a single `reuse_mask: u64` turned the per-hit `remove(position)`/`insert(index, bit)` heap shift and the linear `eviction_candidate_index` scan into a handful of register bit-ops (`mask_remove`/`mask_insert_set`, `trailing_zeros`), collapsing the `put` p99 tail — the only remaining heap-vector maintenance beyond the policy's own item move is gone.

## Hurt

## Takeaway

The parent's `put` p99 tail was the second heap-vector shift the parallel `Vec<bool>` forced on every hit; representing the same scan-resistance state as an index-aligned `u64` bitmask makes that maintenance branch-free and O(1) without touching observable behaviour — hit-rate (0.7267) and the 16-byte `Item` floor hold bit-for-bit because the mask splice mirrors the policy's single-element relocation exactly. Under `bucket_size <= 64`, the whole reuse set fits in a scalar the CPU manipulates in a few instructions, so scan-resistance is now free on latency as well as metadata.
