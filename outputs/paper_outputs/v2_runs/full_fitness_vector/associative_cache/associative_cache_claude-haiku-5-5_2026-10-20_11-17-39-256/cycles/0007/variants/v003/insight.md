# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Swapping the bucket index for a Fibonacci multiplicative hash left workload-a hit-rate effectively unchanged, which is the refutation outcome H03 defined: the remaining misses are not mainly caused by how keys spread across buckets under this policy. The per-op timing shifted in the favorable direction, but a single multiply and shift should not produce a speedup of this size, so that movement is more likely noise or a measurement artifact than an effect of the change. The overlay source could not be inspected in this run, so the code-level claim is inferred from the hypothesis and not confirmed against the diff.
