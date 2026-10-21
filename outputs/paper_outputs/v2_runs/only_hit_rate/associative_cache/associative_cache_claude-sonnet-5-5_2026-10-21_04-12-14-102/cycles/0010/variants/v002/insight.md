# Variant v002 Insight

## Helped

- hit-rate: Doubling ways per bucket (half as many buckets, same total capacity) lets each key hash into a bucket with more eviction candidates, which removes a few conflict misses. The gain is very small, so the parent's associativity was already close to what this workload can use.

## Hurt

- (none observed; hit-rate is the only measured dimension, and the eval shows no regression.)

## Takeaway

Going from the parent's width to 8x baseline associativity, with probation scaled proportionally and single-bucket placement kept, gave only a marginal hit-rate gain on workload-a at size 2048. Conflict misses were not the main source of misses here, so more associativity has sharply diminishing returns. Further gains probably need changes to the insertion, promotion or eviction policy rather than more width. Note that I could only read the hypothesis and the fitness vectors in this session, not the overlay source diffs, so this reflects the intended change rather than a verified code comparison.
