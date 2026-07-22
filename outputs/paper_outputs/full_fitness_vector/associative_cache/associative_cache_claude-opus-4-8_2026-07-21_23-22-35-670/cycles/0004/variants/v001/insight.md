# Variant v001 Insight

## Helped

- per-op-time: Replacing the O(n²) key-matched `realign_reuse` rebuild (plus its `before` snapshot allocation) with a single `Vec::remove(position)` + `Vec::insert(index, bit)` that mirrors the policy's single-element relocation collapsed both p99 tails and the put p50 — the quadratic per-hit tax the parent paid on every `get_from_position` is gone, exactly as the hypothesis predicted.
- hit-rate: Held at the parent's level because the scan-resistant `eviction_candidate_index` (first LRU-front slot with `reuse == false`, else 0) and the `reuse.push(false)` / bit-set semantics were preserved unchanged; the faster realignment reproduces the identical `reuse[i] ↔ items[i]` correspondence, so eviction decisions are bit-for-bit the same.

## Hurt

- metadata-size: Regressed off the 16-byte floor, but not because of this diff — the only changed file is the bucket, `Item` is still key+value, and the relocated `reuse: Vec<bool>` lives in the unmeasured bucket struct. The measured `Item<u64,u64>` growth is not produced by the reuse-bit permutation this variant made.

## Takeaway

The index-preserving swap the parent's insight called for works: mirroring each policy's single-element move onto the parallel `reuse` vector recovers clean p99 tails while holding hit-rate, proving the parent's only real tax was the rebuild algorithm, not the scan-resistance mechanism. The metadata jump is the anomaly to chase — since `Item` is untouched at key+value, that dimension moved for reasons outside this hypothesis's edit rather than from the reuse-bit relocation.
