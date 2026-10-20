# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Raising `SMALL_DIV` from 10 to 4 shrinks the SIEVE main queue from about 90% to 75% of capacity. Keys that have already proven reuse, whether promoted from small or admitted on a ghost hit, now get swept out of main sooner. The longer probation window did not win back enough second touches on workload-a to cover that loss. Keys re-referenced at distances between 10% and 25% of capacity were evidently not common enough to pay for it.
- per-op-time: The `put` tail got slightly worse while p50 and `get` held. This fits a smaller main with a higher share of `visited` bits per slot, which makes SIEVE hand walks a little longer when main does evict. It also fits more small-head pops ending in a promotion followed by another loop iteration in `evict()`. The shift is small enough that it may be noise.

## Takeaway

On workload-a, main capacity is worth more than probation length. Giving a quarter of the cache to the FIFO filter takes space away from the reused working set and costs more than it saves. Together with cycle 2's finding that shortening the second-touch window also hurt, this suggests v003's 10% split is at or near the optimum. Any further split tuning should test points close to 10%, such as 1/8 or 1/12, rather than going wider. Turning to the ghost size instead is the other option.
