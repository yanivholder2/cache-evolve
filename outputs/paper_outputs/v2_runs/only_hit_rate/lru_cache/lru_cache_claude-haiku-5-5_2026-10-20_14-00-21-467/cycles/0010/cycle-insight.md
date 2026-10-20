# Cycle 0010 Insight

## Cohort outcome

The cohort tried inserting new keys just before a probation midpoint marker instead of at the MRU tail (v001, the Winner), doubling the ghost-set retention bound (v002), and shrinking the protected segment from 80% to 50% of capacity (v003). Only the Winner reports a gain, and it is marginal; both other variants are losses.

## Patterns

- Insertion depth again shows the only positive movement. The midpoint placement (v001) gives a small gain on the segmented baseline, which matches the cycle 0009 signal from a different variant. Its insight suggests the protected/probation split already absorbs much of the one-hit pollution that midpoint insertion targets, so the gain is modest.
- The protected-share change (v003) lost hit-rate. Keeping the larger protected share appears to be doing useful work by holding re-referenced keys, and giving probation more room did not compensate. Protected-share tuning in the downward direction is not supported on this trace.
- The ghost-set retention change (v002) was a loss, but its insight shows that changing only the retention bound moved the outcome. That is the first evidence in the lineage that the ghost filter engages, which resolves part of the earlier "never confirmed" concern. Longer ghost memory admits more keys that never recur.
- Across the cohort, the losses point the same way: more admission of cold keys and more room for probation hurt, while a shallower-than-tail placement of new keys helped slightly.
- Metadata size and per-op time are again absent from the fitness vectors. The Winner's cost side is unknown, which is the same gap carried since cycle 0001.

## Open questions

- Does the midpoint-insertion gain (v001) hold on a second trace or another cache size, and how does it change at other marker depths?
- Does the ghost filter's engagement now warrant a retention sweep, or is the capacity-sized bound already near the best point?
- Does the protected-share result (80% better than 50%) hold on a second trace, or is the upper side worth probing (for example, a value above 80%)?
- What does the midpoint marker cost in metadata and per-op time?

## Continuation

Keep insertion depth as the lead lineage: build on v001's midpoint placement, test a small sweep of marker positions, and verify on a second trace before stacking anything else on it. Drop shrinking the protected segment below 80% and drop lengthening the ghost retention bound beyond capacity for this workload. Do not extend the ghost-set idea further until its retention behaviour is tested on another trace.
