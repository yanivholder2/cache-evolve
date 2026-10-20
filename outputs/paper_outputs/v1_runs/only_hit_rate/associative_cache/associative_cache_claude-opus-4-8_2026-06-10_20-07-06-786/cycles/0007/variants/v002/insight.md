# Variant v002 Insight

## Helped

- hit-rate: No movement — and that is the intended result. Replacing the reverse scan in `eviction_candidate_index` with the `protected_count` arithmetic (`len - protected_count - 1`, or `0` when all-protected) preserves the exact victim choice, so eviction order and the segmented working set are untouched.

## Hurt

- hit-rate: No regression. The `protected_count` field is incremented only on promotion (`lfu_counter 0 → 1`) in `get_from_position` and decremented only on protected eviction in `evict_item`, exactly mirroring the prefix/suffix invariant the scan relied on, so no admission or promotion decision changed.

## Takeaway

This variant is a behaviour-preserving rewrite: the O(bucket_size) reverse scan became O(1) arithmetic backed by a maintained protected-item count, and identical hit-rate confirms the prefix/suffix invariant holds at every mutation site. The intended payoff — reclaiming the per-eviction loop cost that the parent's wider buckets made costlier — lives on a per-op-time axis this hit-rate-only fitness vector cannot observe, so the change is provably free here but not provably beneficial under the current measurement.
