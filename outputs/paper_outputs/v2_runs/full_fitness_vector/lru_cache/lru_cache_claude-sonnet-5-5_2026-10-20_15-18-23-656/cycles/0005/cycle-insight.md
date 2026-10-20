# Cycle 0005 Insight

## Cohort outcome

Three small refinements of the cycle-0004 baseline were tried: the fold-free hasher combined with the reference-bit get path (v001), a capped second-chance eviction walk (v002), and a larger protected share of the segmented cache (v003). All three were Winners, but the gains were small. Only v002 clearly changed the trade-off, and it paid in hit-rate.

## Patterns

- Combining the fold-free hasher with the reference-bit path (v001) preserved eviction order and left hit-rate and size unchanged. Speed moved only marginally and may be noise, so the two cycle-0004 gains compose weakly. The hash is now a small share of per-op cost.
- Capping the eviction walk (v002) cut get and put latency, including put p99, which was the cycle-0004 target. It did so by coarsening recency, which lowered hit-rate. Latency and hit-rate now trade off directly through the cap.
- Raising the protected share (v003) slightly lowered hit-rate on the scan-heavy trace. A smaller protected segment and a larger probationary one look at least as good, so the split is a weak lever.
- v003 edited only a construction-time constant, yet its latency shifted. This points to layout or measurement noise at the nanosecond scale, so small per-op-time deltas across the cohort should not be credited to the changes.
- Variant authors could not read the code, so all attribution is inferred from hypotheses and fitness vectors.

## Open questions

- Can the put p99 gain be kept with a larger or adaptive cap, or with a cheaper rule for choosing which referenced nodes to spare, so hit-rate recovers?
- What is the noise floor of per-op-time, and which recent "speed wins" fall below it?
- Does a smaller protected share improve hit-rate beyond the baseline split?
- Does hit-rate behave the same on workloads other than the scan-heavy trace? That check has been pending for several cycles.

## Continuation

Treat v001's combination (fold-free hasher plus reference bit) as the baseline, and explore the cap in v002 as a latency/hit-rate dial, looking for a setting or an adaptive rule that keeps put p99 low without losing hit-rate. Probe a smaller protected share rather than a larger one. Drop further hasher and constant-tuning ideas that sit within noise. Measure a noise floor and evaluate on non-scan workloads before accepting small speed claims.
