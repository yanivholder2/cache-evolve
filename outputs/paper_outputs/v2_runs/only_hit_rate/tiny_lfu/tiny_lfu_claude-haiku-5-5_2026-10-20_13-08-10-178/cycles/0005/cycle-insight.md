# Cycle 0005 Insight

## Cohort outcome

The cohort tried a reduction in sketch row count, a conservative-update rule that raises only the counters at the row minimum, and periodic global aging of sketch counters. The only Winner, v002, was the conservative-update change, a modest hit-rate gain that came from tightening over-estimates rather than from any new memory.

## Patterns

- Both sketch-side changes that touched the estimator's error were the main levers this cycle. The Winner reduced collision over-counting by changing the update rule, which extends the collision-error lever first seen in Cycle 0004's width gain.
- Removing estimator depth hurt. The min-across-rows estimate depends on enough independent rows to suppress collisions, so the cohort's depth reduction undid the kind of gain that width provided.
- Aging the sketch on a fixed access-count trigger hurt. Discarding accumulated frequency history at this capacity weakened the admission comparison, so the sketch needs its history intact on this trace.
- The Winner's per-operation cost is unmeasured. Its extra minimum-row pass on every access is a plausible cost, and the fitness vector does not report timing to confirm it is acceptable.
- All results come from one trace at one cache size, so the Winner's gain is provisional.

## Open questions

- Does the conservative-update gain hold on other workloads and cache sizes, and what does its extra pass cost per operation?
- Is there an aging trigger, not tied only to access count, that keeps the admission comparison intact while still shedding stale frequency?
- Does conservative update stack cleanly with the Cycle 0004 width change, or do the two gains overlap?

## Continuation

Next cycle, build on the conservative-update rule from v002: test it on additional workloads and sizes, and pair it with a timing signal before stacking more estimator changes. Drop reductions in sketch depth, and avoid global halving on a fixed access-count trigger; any future aging idea should use a different trigger and be tested against the Winner.
