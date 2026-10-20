# Cycle 0003 Insight

## Cohort outcome

The cycle tried a behavior-preserving scan refactor (v001), a smaller 5% small queue (v002), and a cap on main-eviction rotation (v003). v001 is the only change with a credible latency gain. v002 gives a tiny hit-rate gain. v003 slightly lowered hit rate and has no measurable speed benefit, so it probably won through noisy latency numbers.

## Patterns

- The cycle moved away from the ghost-tuning direction that Cycle 0002 recommended. None of the three variants touched the main-eviction ghost.
- v001 is the only change with a mechanism for its latency gain: it removes redundant linear scans, and hit rate and metadata are unchanged. The cost is that it deleted several tests on promotion and eviction behavior.
- The small-queue ratio shows diminishing returns. Moving from 10% to 5% helped only marginally, so the curve is flattening.
- The rotation cap in v003 is unnecessary because the frequency counter saturates at a low value, so rotation is already effectively bounded. It trades hit rate for nothing.
- Latency noise continues. The `get` path moved even in variants that did not touch it. Only changes with a clear mechanism, like v001's, should be credited.
- All evidence still comes from one trace (workload-a) at one cache size.

## Open questions

- Is v002's gain real, or noise? Does 5% hold on other traces?
- Can membership checks use an index such as a hash map? Linear scans still dominate `put` cost.
- Can the tests v001 removed be restored without losing its gains?

## Continuation

Keep v001's scan consolidation, restore the deleted tests, and pursue index-based membership to cut `put` cost. Keep 5% as the small-queue ratio only if repeated runs and other traces confirm it, and stop sweeping that knob. Drop the main-eviction rotation cap. Go back to tuning the separate main-eviction ghost from Cycle 0002, and judge on hit rate over multiple traces rather than on latency alone.
