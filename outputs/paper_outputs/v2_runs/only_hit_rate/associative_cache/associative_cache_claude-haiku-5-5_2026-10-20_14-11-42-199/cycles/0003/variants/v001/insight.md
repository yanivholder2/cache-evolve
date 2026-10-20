# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Capping the bucket victim scan at 8 sampled slots left hit-rate on workload-a exactly where the parent variant put it, so the change neither gained nor lost hit-rate here. That matches the parent insight's read that exact-minimum eviction buys little on this trace. The fitness vector has no per-op-time dimension, so the scan-cost reduction the hypothesis was written to achieve is still unmeasured. The overlay source could not be read in this session, so this insight does not confirm that the sampled-victim code actually ran; an identical hit-rate to three decimals is also what a silently unapplied overlay would produce.
