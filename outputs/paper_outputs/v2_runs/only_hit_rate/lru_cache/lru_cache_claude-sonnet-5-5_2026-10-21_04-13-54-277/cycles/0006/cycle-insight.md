# Cycle 0006 Insight

## Cohort outcome

Three variants worked on the sketch admission gate or its CLOCK alternative: no longer counting rejected misses, a one-bit doorkeeper in front of the sketch, and a CLOCK + sketch-gate rewrite of the strict LRU. Only the doorkeeper (v002) won, with a very small hit-rate gain. Not counting rejected misses lost a little hit-rate.

## Patterns

- Admission-gate tuning is saturated. The segmented LRU at 95% protected share already filters most scan keys, so cleaner frequency estimates leave little to gain.
- Counting rejected misses helps. It lets recurring keys build frequency across retries and earn admission. Removing it hurt.
- The doorkeeper's benefit (less scan pollution in the counters) is mostly offset by its costs. A new hot key estimates only 1 and loses the gate more often, and each halving forgets recent first sightings.
- CLOCK plus a sketch gate reports a hit-rate gain over strict LRU, but it did not win. The comparison base is unclear. This is the same "CLOCK is incremental" pattern seen in cycle 0003.
- Metadata-size and per-op-time were again missing from the fitness vectors, now for six cycles. The cost of the sketch, doorkeeper and CLOCK hand sweep is unverified.
- All gains are marginal and rest on one trace at one cache size.

## Open questions

- Is the doorkeeper's tiny gain worth its added state and logic, given that costs cannot be measured?
- Would the CLOCK + sketch design beat the current best segmented line if compared head to head, and on what cost profile?
- Does admission filtering matter on traces with long-distance reuse or heavier scan pressure?
- Is the missing cost measurement a harness fault?

## Continuation

Stop tuning the admission gate: the sketch update rule, the doorkeeper and the threshold are all at a plateau on this trace. Before more hypotheses, fix metadata-size and per-op-time measurement and add a second trace or cache size, since the loop is selecting on hit-rate alone with only marginal differences. Then pursue a different axis, such as adaptive segment sizing, frequency that persists across segments, or a direct cost-aware comparison of the CLOCK + sketch design against the segmented winner.
