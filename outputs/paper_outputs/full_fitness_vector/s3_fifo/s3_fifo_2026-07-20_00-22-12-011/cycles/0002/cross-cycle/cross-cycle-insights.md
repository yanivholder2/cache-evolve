## Recurring wins

- **Eliminating linear scans pays immediately and consistently**: Every O(n)→O(1) replacement (ghost-membership check in cycle 0001, `ghost.retain()` in cycle 0002) produced the single largest latency drop in its cohort. Scan elimination is the highest-confidence bet in each new cycle.
- **Amortizing cleanup to future evictions beats eager cleanup**: Deferring deque maintenance (lazy tombstoning) rather than doing it on the hot path (per ghost-hit retain) cut put tail latency substantially in cycle 0002 and did not degrade hit rate — the eviction capacity guard absorbs the deferred work cleanly.
- **Orthogonal structural changes compose without penalty**: Queue-size tuning, parallel frequency deques, and O(1) ghost operations all touched disjoint code paths across both cycles, and their combination in the cycle 0002 composite variant stacked additively on all three fitness axes without observed interaction loss.
- **Reducing the small-queue fraction improves hit rate and read latency**: Shrinking the small queue from 1/4 to 1/10 consistently boosted main-queue residency and shortened the admission-filter scan; the improvement held across both cycles, including inside the composite.

## Recurring failures

- *(No approach has been tried and clearly failed across multiple cycles yet — both cohorts have been uniformly winning. This section will populate once a hypothesis is falsified.)*

## Open questions

- **Optimal small-queue fraction and workload sensitivity**: 1/10 wins over 1/4, but the lower bound is unknown. Does further shrinking hurt cold-miss handling or accelerate tombstone backlog accumulation? Does the optimal fraction shift under low-skew or adversarial workloads?
- **`evict_main()` second-chance scan is the new primary cost**: All known O(n) hotspots on the put/get path are gone; the main-queue second-chance eviction loop is now the dominant remaining scan. Sub-linear victim-selection strategies (approximate frequency indexing, segmented clock, two-random-choices) have not yet been probed.
- **Tombstone backlog behavior under ghost-heavy workloads**: Lazy deletion allows deque slack to grow between capacity-guard sweeps. The maximum transient overshoot and its impact on eviction timing under dense ghost-hit workloads is uncharacterized.
- **Put latency pressure from a larger main queue**: Enlarging the main queue shifts `evict_main()` traversal further; this cost was visible in cycle 0002 v003 but may be masked by the O(1) ghost path at moderate ghost-hit rates. The break-even ghost-hit rate has not been measured.

## Per-policy notes

- **S3-FIFO (baseline and descendants)**: The baseline left headroom on at least three independent axes simultaneously (queue sizing, ghost-check complexity, entry layout), all of which were exploited in cycle 0001. By end of cycle 0002, the cumulative composite (1/10 small queue + parallel freq deques + O(1) ghost ops + lazy deletion) is the dominant variant on every measured dimension. The remaining optimization surface is concentrated in `evict_main()`.
