# Variant v003 Insight

## Helped

- per-op-time: Moving the target and probing scans ahead of the stash scan in `promotion_get()` means the common target and probing hits no longer pay for a stash bucket linear scan, which is where the get latency drop comes from (both p50 and p99 for get fell).

## Hurt

## Takeaway

Reordering the tier lookup in `promotion_get()` cut get latency on this workload while leaving hit rate and metadata size untouched, which supports the claim that the stash scan sat on the common path. The cost lands on stash hits, which now also pay for two earlier misses, but the fitness vector does not break out stash-hit latency, so that cost is unmeasured. The put numbers also moved even though this diff only touches the get path, so treat that movement as unexplained by this change until it is rerun.
