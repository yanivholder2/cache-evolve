# Variant v001 Insight

## Helped

- hit-rate: Pushing `BaselineGeometry::layout` from v002's even 16/16 split to a normal-dominant `total_buckets * 3 / 4` (24 normal / 8 stash) further enlarged the promotion-protected tier, so more hot keys on the skewed LIRS trace survive in normal buckets instead of being evicted from the shrunken stash.

## Hurt

- (none): The split still satisfies `num_of_segments * (normal + stash) * bucket_size`, so capacity conservation held and no measured dimension regressed; the `clamp(2, total_buckets - 1)` keeps the minimal 2/1 proptest topology valid.

## Takeaway

The cycle-0001 axis is still climbing: moving the bucket budget further toward the normal tier (1:1 → 3:1) kept gaining hit-rate rather than starving admission, so the optimum for this skewed workload lies beyond v002's even point. The stash was over-provisioned more than v002 corrected, and reallocating its capacity to the protected tier remains a pure, budget-preserving win.
