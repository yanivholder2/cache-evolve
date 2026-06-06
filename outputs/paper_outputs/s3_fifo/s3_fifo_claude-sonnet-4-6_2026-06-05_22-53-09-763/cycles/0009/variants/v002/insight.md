# Variant v002 Insight

## Helped

- hit-rate: Replacing the in-place frequency increment for main-queue hits with a position-extract-then-push_front sequence (MTF) moved recently accessed hot items back to the safe end of the eviction queue, preventing them from being swept out by `evict_main()`'s FIFO sweep between accesses.

## Hurt

## Takeaway

Stacking MTF on main-queue hits onto the `capacity / 14` queue-sizing base yields a small but positive additive gain, confirming that the two mechanisms operate on different failure modes — queue sizing limits cold-item admission while MTF limits hot-item positional decay. The gain from MTF alone is narrower than the queue-sizing gain measured in the parent cycle, indicating that on this Zipfian workload at size 2048, cold-item admission pressure (addressed by the smaller small queue) is the larger drag, and position drift in main is a secondary inefficiency that MTF can incrementally correct.
