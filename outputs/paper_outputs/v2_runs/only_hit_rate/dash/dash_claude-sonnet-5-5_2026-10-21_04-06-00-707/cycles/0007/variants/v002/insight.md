# Variant v002 Insight

## Helped

- None. hit-rate is the only reported dimension, and it is identical to the parent's on the single trace (workload-a, size 2048).

## Hurt

- None measured. The extra stash pass and the second-chance flag clearing could cost CPU time on eviction, but no dimension captures that. I could not inspect the overlay sources, so this rests on the hypothesis text and the two fitness vectors.

## Takeaway

Changing the victim scan order (stash first, then home bucket, then second-chance clearing) had no effect on hit-rate. Either the eviction path rarely sees stash slots that differ in referenced state from the bucket slots, or all-referenced buckets are rare on this trace. Parent v001's reuse-bit gain therefore came from using the flag at all, not from where the scan looks first or how the all-referenced case is resolved. This ordering refinement adds complexity for no measured benefit on workload-a.
