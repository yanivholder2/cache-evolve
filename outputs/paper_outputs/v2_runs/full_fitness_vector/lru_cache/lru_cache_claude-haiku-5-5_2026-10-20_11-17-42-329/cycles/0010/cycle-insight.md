# Cycle 0010 Insight

## Cohort outcome

The cohort tried three levers: moving new keys to the eviction end of the probationary segment (v001), bounding the CLOCK victim search to two probes (v002), and adding a doorkeeper that refuses a new key's first sighting at capacity (v003). Only v003 records a dimension that improved (hit rate on workload-a at 2048). The Winner labels for v001 and v002 are not supported by their insights: v001 records only a hit-rate loss, and v002 records a flat `put` tail gain that is offset by a hit-rate loss.

## Patterns

- Admission filtering is the one lever here that helped. Refusing first sightings kept one-hit scan traffic from displacing residents, and the cost is a bounded hash on new-key misses at capacity. The `put` median also fell, which fits the skipped victim-selection and map work on refused keys.
- Insertion position matters for reuse. Placing new keys at the eviction end (v001) left a one-hit key only about one miss's worth of time to be re-referenced, and re-referenced keys that the old placement kept alive were evicted first. The loss is on hit rate, not on latency.
- Bounding the CLOCK sweep (v002) traded a tighter worst-case `put` cost for hit rate. The second probe evicts a key whose counter is still non-zero, so recently hit keys lose the extra passes the unbounded sweep gave them.
- The `get` and `put` median movements in v002 are not explained by the diff, since the `get` path is unchanged and the bounded sweep does less work per miss. These should be treated as measurement noise until re-measured. This matches the recurring unexplained tail and median movements from earlier cycles.
- v003's metadata summary omits a heap-allocated doorkeeper table and a new header on `LruCache`. Its size benefit, if any, is not captured by the measured node-size dimension.

## Open questions

- Does the doorkeeper's hit-rate gain hold on workloads other than workload-a at 2048, and does it survive a disabled aging reset or a larger table?
- How much of v003's gain comes from refusing one-hit keys, and how much from shared-slot collisions that admit a one-hit key early?
- Does the doorkeeper's table footprint offset any size advantage once it is counted in the full memory footprint?
- Would a bounded CLOCK sweep that respects the second probe's counter keep the `put` tail benefit without the hit-rate loss?
- Are the v002 median movements a run-to-run artifact, as the recurring pattern suggests?

## Continuation

Next cycle, build on v003's first-sighting admission filter as the hit-rate candidate, and ablate its aging interval and table size before stacking more on it. Drop the insertion-end placement from v001, since it removes the reuse window that the probationary segment depends on. Do not keep the forced second-probe eviction from v002; if the bounded sweep is retried, make the second probe respect its counter. Re-measure `put` and `get` tails before crediting any of these changes.
