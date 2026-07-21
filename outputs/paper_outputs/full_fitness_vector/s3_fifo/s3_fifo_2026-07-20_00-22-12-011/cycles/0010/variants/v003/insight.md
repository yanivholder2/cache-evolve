# Variant v003 Insight

## Helped

- hit-rate: Raising `MAX_FREQ` from 3 to 4 gives hot entries in `evict_main()`'s second-chance loop one additional frequency credit to burn before reaching zero, reducing premature evictions of medium-frequency items and yielding a measurable hit-rate improvement over the parent.

## Hurt

## Takeaway

`MAX_FREQ=4` achieves strictly better fitness than the parent: hit rate rises while per-op-time (get p50=0 ns, put p50=100 ns) remains identical to `MAX_FREQ=3`. The deeper scan required to drain one extra frequency level in `evict_main()` is too small to register in any latency percentile, confirming that the quantisation floor absorbs the added work at this working-set size. The unexplored gap between 3 and 5 was worth probing — it lands closer to the higher-`MAX_FREQ` hit-rate without inheriting its latency cost.
