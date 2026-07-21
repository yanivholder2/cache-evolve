# Cycle 0002 Insight

## Cohort outcome

All three variants targeted the CountMinSketch or node memory layout — one refining the node-encoding fix from the prior cycle, one consolidating sketch allocation, and one trimming sketch depth — but none crossed the winner threshold; the closest (NonZeroU32 node encoding) merely returned to baseline parity without surpassing it.

## Patterns

- **The CountMinSketch resists structural simplification on this workload.** Both sketch modifications failed in opposite ways: flattening to a single allocation introduced cache-set conflicts that more than doubled latency, while halving depth degraded collision suppression enough to hurt hit rate and latency simultaneously. The baseline's four-row, multi-allocation design is well-balanced for the trace's capacity and access pattern.
- **v001 confirmed NonZeroU32 as the correct node encoding, but node compaction is now fitness-neutral.** Recovering v002's latency regression with idiomatic `Option<NonZeroU32>` match arms proved the discriminant-test codegen was the culprit — not node footprint itself. However, the 24-byte node yields no measurable winner-level improvement over baseline, exhausting the node-layout axis.
- **Flat allocation is not a reliable substitute for multiple independent heap segments.** The pointer-indirection cost of `Vec<Vec<u32>>` was lower than predicted; the real cost at runtime is the cache-set pressure created by the large row strides in the flat layout, which the separate-allocation baseline avoided by chance of heap placement.
- **Depth redundancy in the sketch is load-bearing, not padding.** The four rows provide meaningful collision suppression on this Zipfian trace — removing two rows at this sketch width causes enough estimation noise to cascade into eviction errors and overall latency increase.

## Open questions

- Is there a sketch modification that preserves four-row accuracy while reducing the per-call hashing cost — e.g., computing all four positions from a single seeded hash rather than four independent hash invocations?
- Are there fitness gains still available on the admission or eviction logic side (e.g., window/protected segment sizing, admission threshold tuning) that are orthogonal to the sketch and node axes now both largely exhausted?

## Continuation

Next cycle should abandon sketch-structure changes (both flat allocation and depth reduction are definitively closed) and node-layout changes (NonZeroU32 is the correct encoding but offers no further gains). The one remaining open thread from Cycle 0001 — adaptive or longer-interval sketch decay — has not been tested and should be prioritized: the stable working set may tolerate a much longer reset window that ages stale keys without disturbing the hot-set frequency signal. Alternatively, shifting focus to admission and eviction policy parameters (window size, protected/probationary ratio) offers an unexplored axis.
