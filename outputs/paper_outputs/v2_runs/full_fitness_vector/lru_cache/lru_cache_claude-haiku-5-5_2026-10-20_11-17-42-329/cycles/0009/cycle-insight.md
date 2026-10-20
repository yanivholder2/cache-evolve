# Cycle 0009 Insight

## Cohort outcome

The cohort tried three levers: a larger protected segment (v001), skipping the reference-bit write when it is already set (v002), and replacing the two-segment SLRU with a single CLOCK ring of 2-bit counters (v003). Only v001 records a dimension that improved (hit rate on workload-a at 2048), and it paid for that with `put` tail latency. v002 records no improvement, and v003 gained metadata size at a hit-rate and latency cost. The Winner labels for v002 and v003 are not supported by their insights' Helped sections.

## Patterns

- The protected share is a real hit-rate lever in this SLRU. Growing it moved reused keys out of reach of the probationary scan traffic, but the probationary segment got thin and the `put` miss path did more demotion work. Cycle 0006 found that shrinking the share hurt hit rate, so the current range looks like a trade-off rather than a clear optimum.
- Guarding the reference-bit write on hits (v002) cost more than the store it skipped. The unconditional OR on the packed word is cheaper than the extra load and branch, so the hit path should stay as it is.
- Replacing the segmented structure with CLOCK (v003) removed the protection that kept rescued keys safe, and it also moved the per-slot state into a separate allocation. The metadata saving is smaller once that allocation is counted, and the per-hit increment and the multi-slot sweep both show up as latency.
- The v003 insight's claim that the hit-rate cost was small was an underestimate. The segmented protection appears to matter more than the node-size argument suggested.
- Across the three variants, the `put` tail moved in the variants that touched capacity handling or the miss path (v001 and v003). This matches the recurring `put` tail pattern from earlier cycles and is still not separated from run-to-run noise.
- The v001 and v003 per-operation causes are inferred from the code and the hypotheses, not profiled. Their code-level claims should be checked against the diffs before they are credited.

## Open questions

- Does the protected-share gain on workload-a at 2048 hold at other capacities and traces, and can it be had without the `put` tail cost?
- Is the v001 `put` tail regression real, or is it the same run-to-run artifact seen in earlier cycles?
- Would a CLOCK variant that keeps some form of protection recover the hit rate while keeping the smaller node?
- Is the `refs` allocation cost in v003 large enough to offset the node-size saving across the full footprint?

## Continuation

Next cycle, build on v001's larger protected share as the hit-rate candidate, and test it at a second capacity or trace before stacking more on it. Measure its `put` tail against the baseline before accepting the change. Drop the reference-bit guard from v002 and the plain CLOCK rewrite from v003, since neither kept the protection or the hit path that the parent relied on.
