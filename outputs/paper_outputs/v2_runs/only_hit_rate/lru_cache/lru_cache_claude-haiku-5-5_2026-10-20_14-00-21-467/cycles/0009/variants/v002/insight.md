# Variant v002 Insight

## Helped

## Hurt

## Takeaway

The hit-rate on workload-a at size 2048 is identical to the parent v003, so the correlated-reference filter (count a hit only if at least K accesses have passed since the last counted hit) produced no measurable change on this trace. A K=16 gap is short relative to a 2048-entry cache, so most re-references may have passed the filter unchanged, which would explain a null result. This insight could not confirm the overlay's actual diff, so it is unverified whether the filter landed as the hypothesis describes; check the source before drawing a conclusion about the idea itself.
