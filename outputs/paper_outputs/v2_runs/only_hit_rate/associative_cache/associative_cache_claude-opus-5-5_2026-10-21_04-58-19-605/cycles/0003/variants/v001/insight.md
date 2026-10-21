# Variant v001 Insight

## Helped

- hit-rate: New keys now enter a full 16-slot bucket at index 1 instead of index 3. A key that is never reused is now evicted after about 2 misses in its bucket instead of about 4, so its slot goes back to reused keys sooner. The gain over the parent is clearly smaller than the parent's gain over the midpoint, so each step deeper is paying off less.

## Hurt

(None measured. This evaluation recorded only hit-rate. The diff changes only `INSERT_DEPTH_DEN` from 4 to 8, so metadata-size cannot have changed. Each miss's `Vec::insert` at index 1 shifts about 14 elements instead of about 12, so a small per-op-time cost on the miss path is expected. The hypothesis asked for per-op-time to be recorded, but it was not, so that cost is unconfirmed.)

## Takeaway

On workload-a, a shorter probation window still helps. Even with only about 2 misses to prove reuse, the cache gains more from evicting one-off keys quickly than it loses from evicting keys with short reuse distances. That answers H01's bracketing question: the best depth is at 1/8 or deeper, and pure LIP (index 0) is the next test. The smaller step-to-step gain suggests the curve is flattening near the eviction end, so index 0 may add little or even lose a bit. Note that `len * 1 / 8` is already 0 whenever a bucket holds fewer than 8 items, so this variant runs pure LIP while buckets are still filling. The difference from index 0 only matters once buckets are mostly full.
