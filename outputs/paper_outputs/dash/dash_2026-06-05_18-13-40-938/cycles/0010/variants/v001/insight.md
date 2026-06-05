# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Changing the eviction policy from LRU to FIFO (with configurable `EvictionPolicyKind`) did not improve hit-rate—it remained flat at 0.684. This suggests that the choice of bucket-level eviction algorithm (whether to evict least-recently-used or first-inserted) is not the bottleneck. Despite the occupancy-aware demotion routing already in place (bypassing congested target when >= 70% full), replacing recency-biased eviction with insertion-order eviction yielded no gain, indicating the 0.684 ceiling is constrained by higher-order tier structure or promotion eligibility gating, not by the eviction selection criterion.

