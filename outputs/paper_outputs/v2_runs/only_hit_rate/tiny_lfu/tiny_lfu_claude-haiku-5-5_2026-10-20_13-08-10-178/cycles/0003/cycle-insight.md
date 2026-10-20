# Cycle 0003 Insight

## Cohort outcome

The cohort tried two sketch-layout changes (narrower counters, and hoisting per-key row indices into a shared buffer) and one change to the main region's internal ordering, splitting its LRU into probation and protected segments. The only Winner, v002, was the main-region reordering, the one change that showed a hit-rate movement; the two layout-only variants preserved behaviour but advanced on no measured dimension.

## Patterns

- The only hit-rate gain came from changing recency ordering inside the main region, with the sketch and admission path left alone. That is a different lever from the sketch-side and window-side changes of Cycles 0001 and 0002.
- Layout-only changes to the sketch again preserved hit-rate by construction, and again their claimed benefits (footprint, per-op time) sit on dimensions the fitness vector does not measure. This is the same unmeasured-payoff pattern seen in Cycle 0002.
- The Winner's signal rests on a single trace at a single cache size, so its gain is provisional rather than established.

## Open questions

- Does the probation/protected gain hold on other workloads and cache sizes, or is it specific to this trace?
- Would a timing or memory dimension in the fitness vector confirm the hashing-reuse and counter-narrowing gains, and justify keeping them?
- Does the sketch's saturation point ever bind on very hot keys, or does the admission comparison stay insensitive across workloads?

## Continuation

Next cycle, build on the main-region recency structure from v002: test it on additional workloads and sizes before stacking further changes on it, and avoid re-proposing the window-in-front-of-LRU idea. Drop further sketch-layout-only changes until the fitness vector gains a metadata or timing signal that can confirm them.
