# Variant v001 Insight

## Helped

- hit-rate: Doubling ways per bucket (halving bucket count at constant capacity) reduces conflict misses, because hot keys that collided in a narrow bucket now compete over twice as many slots. The probation/protected split, which scales with bucket width, also gets more absolute room in each bucket.

## Hurt

- (none observed; hit-rate is the only measured dimension, so any cost in lookup scan length or per-bucket move-to-back work is not visible in the fitness vector)

## Takeaway

Widening associativity does stack on top of the segmented-LRU split, but the increment is very small, so the incumbent's remaining misses are mostly capacity or recency-ordering misses rather than bucket-conflict misses. Further gains probably need a change to the admission, promotion or eviction policy, not wider buckets. I could not read the overlay source in this session, so this interpretation rests on the hypothesis text and the fitness vectors.
