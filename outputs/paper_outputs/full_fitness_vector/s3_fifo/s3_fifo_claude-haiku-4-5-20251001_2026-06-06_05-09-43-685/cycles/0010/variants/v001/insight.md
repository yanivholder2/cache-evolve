# Variant v001 Insight

## Helped

(None)

## Hurt

- **hit-rate**: Linear occupancy scaling (1.5 + 2.0 * occupancy_ratio) provides weaker re-admission protection than the quadratic formula, causing re-admitted ghost-hit entries to be evicted prematurely and disrupting the balance between small-queue working-set development and main-queue occupancy.
- **per-op-time (get)**: Loss of v002's latency advantage suggests the linear formula alters queue dynamics in ways that increase cache traversal cost or eviction frequency, raising get p50 from 106ns to 142ns.
- **per-op-time (put)**: Weakened re-admission protection forces more aggressive eviction cascades in the main queue, raising put p50 from 544ns to 600ns and p99 from 1637ns to 2086ns.

## Takeaway

Switching from quadratic to linear occupancy-responsive re-admission scaling failed on both fronts: hit-rate did not recover and v002's latency gains were lost entirely. The hypothesis assumed that a gentler boost would improve cache dynamics, but the linear formula (1.5–3.5 range) underprotects ghost-hit items in this workload. The variant demonstrates that correcting v002's miscalibration requires more than reducing boost magnitude—either the occupancy-responsiveness approach itself, the specific coefficients, or the re-admission criteria must be reconsidered.
