# Variant v001 Insight

## Helped

*(no dimensions moved positively)*

## Hurt

- **hit-rate**: Bypassing a saturated target to place stash-promoted items directly into probing displaces hot probing residents via cascade eviction, producing a net loss on this workload — the cascade from a forced probing insert is costlier than the cascade that would have occurred on a full-target insert.
- **per-op-time**: The bypass path runs `promote_to_probing` instead of `promote_to_target`, which triggers a longer demotion chain (probing evictee → one-more-chance retry into true target → potential second eviction into stash), inflating p99 latency even though p50 held steady.

## Takeaway

The saturation-aware bypass does not compose additively with MRU-end insertion: because `InsertPosition::Back` causes stash-promoted items to land immediately at the MRU end, target saturation is more common, so the bypass fires frequently and repeatedly pushes stash items into the already-contested probing tier. The benefit seen in the LRU-end baseline (cycle/0002/v003) likely came from the bypass being rare; on the MRU-end base it fires often enough to make probing a spillover sink for stash traffic, which hurts both hit rate and tail latency.
