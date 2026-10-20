# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Halving the `lfu_counter` cap from 4 to 2 causes frequently-accessed items to cycle back to cold eligibility after fewer decay events, allowing some hot-set members to be evicted slightly sooner and producing a marginal increase in misses.

## Takeaway

Reducing the `lfu_counter` ceiling from 4 to 2 did not accelerate useful cold-pool replenishment — it produced a negligible hit-rate decline rather than any gain. Combined with the parent's finding that cap=4 was already inert versus an unbounded counter, this confirms that the counter ceiling is not a sensitive tuning axis for workload-a at size 2048: the workload's access pattern either keeps natural counter pressure low enough that the cap rarely binds, or the decay cadence resolves warm-lock episodes before the ceiling becomes the bottleneck. Further exploration of `lfu_counter` magnitude is unlikely to yield returns on this trace.
