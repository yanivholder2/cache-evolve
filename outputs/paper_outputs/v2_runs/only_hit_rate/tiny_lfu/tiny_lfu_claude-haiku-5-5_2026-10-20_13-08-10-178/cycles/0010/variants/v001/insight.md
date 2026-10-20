# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Scoping the aging to the saturated row has no measurable effect on workload-a at size 2048: hit-rate matches the parent exactly. The new `SAT_THRESHOLD` is `u32::MAX / 2`, roughly 2.1 billion bumps on a single counter, so the per-row halving branch in `CountMinSketch::increment()` almost certainly never fires on this trace. The result therefore says nothing about whether per-row scoping beats table-wide halving; that question needs a much lower threshold or a trace that actually drives counters toward saturation.
