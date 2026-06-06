# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Shrinking protected from 90% to 85% displaced ~102 already-promoted items that had earned their place, and the wider probationary buffer (~205 → ~307 slots) did not rescue enough borderline warming items to compensate for that eviction pressure on the hot set.

## Takeaway

The hypothesis that a larger probationary buffer would reduce premature eviction of warming items did not hold on workload-a: the k=4 frequency-sampled victim scan already provided sufficient shelter for nearly-warm items at the 10% probationary size, so the freed space offered little additional benefit. The cost side dominated — each slot taken from protected directly reduces residence time for items that had already crossed the PROMO_THRESHOLD=6 gate, and on this workload that trade is net-negative. Widening probationary beyond 10% is the wrong direction; if premature eviction of warming items remains a concern, a lower PROMO_THRESHOLD or a larger k scan window is a less destructive lever.
