# Variant v003 Insight

## Helped

- hit-rate: Both hm_0 cells, small and large, rose by the same clear margin, and the large hm_0 gain meets the hypothesis's confirmation bar. Both web_0 cells moved only at noise level. No cell went down, so the refutation condition (a small-cache loss) is not met. The gain follows the trace, not the cache size. hm_0 has stash entries that get hit again soon after they are stashed. Sparing a recently hit `lru0` for one round, at the cost of a never-hit `lru1`, keeps those entries long enough for the next hit. web_0 seems to have too few such entries for the spare condition to fire often. This goes against the hypothesis's expectation that the benefit would show up mainly in large caches.
- per-op-time: All four latency figures improved by a large, near-uniform factor, including get p50 and get p99. The change cannot have caused the get improvement, because it never touches the get path. Most of the shift is therefore environmental: the parent's timings were already flagged as host-inflated. What the data does support is narrower. The stash-full put path now does an extra two-slot check, yet put p99 sits below get p99 and shows no tail inflation. That is evidence the bounded check avoided the put cost seen in cycle 0008 v003. A strict check of the "put p99 near the same-session H01 control" criterion needs that control's numbers, which this comparison does not have.

## Hurt

(none: no dimension regressed; metadata-size is unchanged at the 24-byte Item, as the constraint required)

## Takeaway

Stash victim protection works on this lineage once it is bounded. The rule is: a recent hit, only the two LRU-end slots, a single reprieve with no MRU refresh, and the spare taken only from a never-hit neighbour. With those limits, the change turned cycle 0008 v003's losses in every cell into a gain on hm_0 at both sizes and no loss anywhere. So the stash-exit lever is not dead. Its benefit depends on how much short-range reuse the workload has, not on cache size, so future tuning (for example `STASH_PROTECT_AGE`) should be judged on hm_0-like traces. Caveat: I could not list or open the overlay source in this session. The code-level reasoning assumes H03 was implemented as written.
