# Variant v003 Insight

## Helped

## Hurt

## Takeaway

H03 changed `evict_one()` so that stash-tier victims are chosen from the earliest-landed entry in the target bucket instead of the current selection, with no new timestamps or markers. On workload-a at size 2048, hit-rate is exactly equal to the baseline, so this eviction-order change produced no measurable hit-rate movement on this trace. The fitness vector also omits the metadata-size and per-op-time dimensions that the hypothesis asked to be recorded, so the cost side of this change is unmeasured. I could not inspect the overlay source diff in this session (no shell or search tools were available), so this insight is based on the hypothesis text and the fitness vectors only.
