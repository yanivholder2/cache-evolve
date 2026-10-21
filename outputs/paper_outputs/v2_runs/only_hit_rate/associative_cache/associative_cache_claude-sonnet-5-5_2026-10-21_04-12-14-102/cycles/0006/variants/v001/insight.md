# Variant v001 Insight

## Helped

- hit-rate: No movement versus parent v003. Hit-rate is identical to the parent on the single LIRS workload-a trace at this cache size, so the demotion-placement change gave no measurable gain.

## Hurt

- hit-rate: No regression either. The change was neutral, not harmful.

## Takeaway

Where a demoted protected entry lands in the probation segment (LRU end instead of MRU end) did not change hit-rate on this trace. The probation segment is only slots/16 per bucket, so the demoted entry is evicted or re-hit almost immediately wherever it sits. A tie this exact may also mean the demotion path rarely fires, or that the edit did not alter behaviour on this workload. Demotion placement is therefore not a useful lever in this design. Only hit-rate was recorded, so any effect on per-op time or metadata is unmeasured. I could not read the overlay source, so I did not check whether the code diff is as the hypothesis describes.
