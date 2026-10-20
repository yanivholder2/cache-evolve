# Variant v002 Insight

## Helped

- hit-rate: Pushing keys evicted from main into the ghost queue lets a prematurely evicted main resident be re-admitted straight to main on its next access. The gain over baseline is very small.

## Hurt

- metadata-size, per-op-time: These dimensions were not measured for v002, so no effect can be attributed. The extra ghost push and pop in `evict_main` adds some put-path work and may increase ghost occupancy, but that is unverified.

## Takeaway

Extending the ghost queue to cover main-queue victims is nearly neutral on this workload. Baseline S3-FIFO's main eviction (freq-based reinsertion) already rarely evicts items that are about to be reused. The hypothesis targets a rare case, so the gain is marginal. Judging any cost needs the metadata and timing dimensions, which this evaluation did not report.
