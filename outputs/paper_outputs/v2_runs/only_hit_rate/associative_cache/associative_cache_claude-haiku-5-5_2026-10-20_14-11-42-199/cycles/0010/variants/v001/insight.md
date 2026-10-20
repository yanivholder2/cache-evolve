# Variant v001 Insight

## Helped

- hit-rate: The probationary victim is now the lowest shared-frequency probationary entry, with the policy's own pick winning ties, so eviction removes the least-reused probationary item instead of whatever the policy ranks first. The gain is small and comes from one trace at one size.

## Hurt

## Takeaway

Choosing the probationary victim by shared frequency, and sharing one victim index between the admission check and the removal, gives a marginal hit-rate gain over the parent on workload-a. The bucket-side change is the part I could read; the admission comparison in `AssociativeCache::put()` was not available to review, so the claim that admitted keys always displace the compared entry is unverified here. The extra probationary scan on each miss is an unmeasured cost, since only hit-rate is reported.
