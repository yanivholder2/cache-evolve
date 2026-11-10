# Variant v001 Insight

## Helped

- hit-rate: Cutting the small queue at or below `SMALL_CACHE_THRESHOLD` from 1/5 to 1/6 gives main a bit more room at 16384, and both traces gained a little there. hm_0 gained more than web_0, and web_0 still did not lose, so the share optimum at 16384 is at or below 1/6, not at 1/5. Both 131072 cells are bit-identical to the parent, as the size gate guarantees.

## Hurt

- per-op-time: `get` p50 and `put` p50 each rose by about 1 ns, and `put` p99 rose a few ns. The diff only changes a divisor in `try_new`, which runs once at construction. A slightly smaller small queue at 16384 does not add work per operation, since every eviction, ghost and graduation path is unchanged. So this is most likely benchmark noise, not a cost of the change. It is worth re-measuring before blaming the split.

(metadata-size did not move: the split only changes queue capacities chosen at runtime, so the `Entry<u64,u64>` and `S3Fifo<u64,u64>` layouts are the same.)

## Takeaway

The 1/6 probe did not trigger the "converged" condition: the 16384 hit rate kept rising in the same direction for both traces. The sweet spot for small caches is somewhere at or below 1/6. Cycle 5 showed 1/10 hurting web_0, so it is likely between 1/6 and 1/10. Each step from 1/4 to 1/5 to 1/6 has returned only a tiny gain, which is concentrated in hm_0. Further narrowing is a low-yield lever: probe 1/7 or 1/8 once to find the peak, then stop tuning the split.
