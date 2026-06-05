# Variant v002 Insight

## Helped

- hit-rate: Rebalancing the bucket split from 8 normal / 24 stash to an even 16 / 16 in `BaselineGeometry::layout` enlarged the promotion-protected normal tier and spread keys across more target buckets, cutting forced evictions of hot items on the skewed trace.

## Hurt

- (none): The even split conserves `num_of_segments * (normal + stash) * bucket_size`, so capacity conservation was untouched and no measured dimension regressed.

## Takeaway

For this skewed workload the baseline over-provisioned the cold stash admission area at the expense of the protected tier; reallocating fixed capacity toward normal buckets is a pure win on hit-rate without breaking budget conservation. The clamp keeping normal >= 2 / stash >= 1 means even the split shift stays a safe, validity-preserving reshape rather than a capacity change.
