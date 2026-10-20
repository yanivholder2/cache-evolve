# Cycle 0005 Insight

## Cohort outcome

The cohort tried three changes: a ghost-history fast path for readmitting recently evicted keys, an equivalence-preserving occupancy counter for victim search, and moving probing-tier demotions to the LRU end of their normal-tier bucket. Only v001 is listed as a Winner, yet its own insight reports a slight hit-rate drop on the one measured trace, so the Winner status is not supported by this cohort's evidence.

## Patterns

- Early readmission looks harmful. The ghost-history path let returning keys skip the seen-in-stash wait, and the result was a slight hit-rate loss, which suggests recently evicted keys displaced residents that were still earning their place.
- The two no-change variants agree with each other. The occupancy-counter refactor and the demotion-placement change both matched the parent exactly, so this cohort gives no evidence that victim-search cost or demoted-item placement moves hit-rate on this workload.
- No variant measured per-op-time or metadata-size. The occupancy counter's claimed speedup and the ghost set's memory cost are both untested.
- Three of three insights rest partly on hypothesis text because the overlay diffs were not readable. The implementations are not confirmed as described.

## Open questions

- Why is v001 listed as a Winner when its measured hit-rate is below its parent? Is it winning against a different baseline?
- Does the occupancy counter produce any speedup once per-op-time is measured?
- Is demoted-item placement ever the deciding factor for a victim on any workload or capacity?
- Does the ghost-history cost in metadata justify any hit-rate it might gain on other traces?

## Continuation

Next cycle should not pursue the ghost-history fast path further unless a second workload shows a clear hit-rate gain, since the one measured result was negative. Drop the demotion-placement change, which showed no effect. Keep the occupancy counter only as a cost-side experiment: re-test it with per-op-time and metadata-size recorded, and confirm the overlay diffs before drawing conclusions.
