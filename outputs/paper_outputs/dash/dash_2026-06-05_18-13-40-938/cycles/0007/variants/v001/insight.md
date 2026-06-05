# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Target-first eviction priority (proactively evicting LRU items from the target tier when the probing tier is full) did not improve hit-rate. The stagnation persists despite both H02's eligibility-delay gate and H01's eviction burden shifting, indicating that the 0.684 ceiling is neither constrained by *when* items become promotion candidates nor by *where* evicted items flow, but by a more fundamental factor in tier positioning or working-set interaction. Shifting eviction pressure away from probing to target is a different lever than eligibility gating, yet both fail to break the plateau—suggesting the bottleneck lies upstream, in how items arrive and navigate the promotion pathway itself.
