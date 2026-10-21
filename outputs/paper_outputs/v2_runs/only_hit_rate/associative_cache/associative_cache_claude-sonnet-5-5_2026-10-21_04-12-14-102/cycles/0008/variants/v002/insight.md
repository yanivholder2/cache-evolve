# Variant v002 Insight

## Helped

- hit-rate: Doubling ways per bucket again (4x baseline) with the bucket count halved lets more keys share a set, which removes a few more conflict-induced evictions. The gain over v001 is marginal.

## Hurt

- (none): hit-rate is the only measured dimension, and it did not regress. I could not read the overlay source in this session, so any cost in lookup scan length is unmeasured.

## Takeaway

Going from 2x to 4x associativity at fixed capacity (probation kept at slots/16) gave a much smaller increment than the first doubling did. Conflict misses on this trace are mostly already eliminated, so the remaining misses are capacity or recency-driven. Further associativity changes are unlikely to pay off. Gains would have to come from the promotion, demotion or eviction policy.
