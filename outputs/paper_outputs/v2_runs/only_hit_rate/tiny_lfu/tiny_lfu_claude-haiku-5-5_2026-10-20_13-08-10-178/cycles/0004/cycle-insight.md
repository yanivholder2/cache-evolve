# Cycle 0004 Insight

## Cohort outcome

The cohort tried a sketch-width change, a resizing of the protected segment within the main region, and a change to where demoted protected entries land in probation. The only Winner, v002, was the sketch-width change, the first sketch-side change in recent cycles to show a hit-rate gain; the two main-region variants both lost hit-rate against the parent.

## Patterns

- Widening the sketch reduced over-counting from colliding keys and produced the only hit-rate gain in the cohort. This contrasts with Cycle 0001, where sketch-side changes lost hit-rate; the difference is that this change addressed collision error rather than decay or key mapping.
- Both main-region variants moved away from the incumbent and both lost hit-rate. The current probation/protected split and the demotion placement at the back of probation both look like the better settings for this trace.
- The Winner's gain is small, and its predicted metadata cost (sketch size scaling with width) was not measured. Its advancement rests on hit-rate alone, the same unmeasured-cost pattern seen in Cycles 0002 and 0003.
- All three variants were evaluated on a single trace at a single cache size, so every result here is provisional.

## Open questions

- Does the width gain persist on other workloads and cache sizes, and is it worth its metadata cost once that cost is measured?
- Is there a point where further widening stops helping, i.e. where collision error is no longer the limiting factor in admission?
- Does the protected/probation ratio near 4/5 hold as the best setting on other traces, or is it specific to workload-a?

## Continuation

Next cycle, build on the sketch-width result by testing it on additional workloads and sizes and measuring its metadata footprint before stacking further sketch changes. Hold the current protected-segment ratio and demotion placement as-is, and drop the halved protected cap and the push-to-front demotion variants.
