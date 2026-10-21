# Variant v001 Insight

## Helped

- hit-rate: Shrinking the small queue from capacity/4 to capacity/10 gives main a larger share of the capacity. Items that were promoted on the `freq > 1` gate are less likely to be displaced, and the LIRS workload improved over parent v002.

## Hurt

- None observed. Only hit-rate was measured, so metadata-size and per-op-time effects are unknown. The smaller small queue also gives one-hit-wonders a shorter probation window, which could hurt on other workloads, but this evaluation didn't test that.

## Takeaway

On this LIRS trace, probation-queue size is a tunable lever: a smaller small queue is a modest but real improvement over v002's ghosting-only change. The gain comes from allocating more capacity to main, not from any new eviction logic. The result comes from one trace and one cache size, so it may not carry over to scan-heavy workloads, where a larger small queue filters scans better.
