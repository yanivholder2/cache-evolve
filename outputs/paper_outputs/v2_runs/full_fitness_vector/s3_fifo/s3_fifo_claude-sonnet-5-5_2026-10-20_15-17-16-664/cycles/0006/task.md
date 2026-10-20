# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0006

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

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


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cycle tried hash-index membership lookups (v001), a main-eviction ghost that re-admits keys into main with a freq head start (v002), and a doubled ghost bound (v003). Only v001 has a real gain: lower typical `put` and `get` cost with hit rate unchanged. v002 won with no improved dimension. Its extra ghost scan and its lost hit rate point to noise as the reason it was picked.

## Patterns

- v001 is the only variant with a mechanism for its gain. The index removes linear scans and preserves the hit/miss sequence exactly. Its cost is a much larger whole-cache footprint that the fixed per-entry `metadata-size` figure does not show.
- v001's `put` tail barely improved. The remaining tail cost likely comes from eviction and ghost-trimming work, not membership checks.
- Both ghost-side changes (v002, v003) lowered hit rate on workload-a. A freq head start for main-evicted keys and a longer ghost history both let marginal objects into main. This is the third cycle in which anything that loosens main admission has hurt.
- v002 contradicts Cycle 0002's reading of the separate main ghost as a small gain. Its head-start variant regressed, and the extra linear scan undid scan savings. The earlier gain may have been noise or may depend on details of the design.
- Baseline ghost sizing looks near a sweet spot. Reuse beyond main capacity is rare on this trace.
- Latency shifts that have no mechanism, such as v003's tail, are noise again. Latency is still unreliable for selection unless the change has a mechanism.
- Everything still comes from one trace (workload-a) at one cache size.

## Open questions

- Does v001's index hold its gain across traces? What is its real memory cost?
- Can the `put` tail be reduced by making eviction and ghost trimming cheaper?
- Is any form of special handling for main-evicted keys worthwhile? If so, it needs an O(1) tagged-ghost design with evidence of faster reuse.
- Do the earlier small-ratio and ghost results survive repeated runs and other traces?

## Continuation

Keep v001's index-based membership as the new base. Next, target `put` tail cost in eviction and ghost trimming, and account for the index's memory footprint explicitly. Drop the main-ghost freq head start, the larger ghost bound, and further ghost-size tuning. Revisit main-evicted-key handling only as an O(1) tagged variant of the existing ghost. Validate on multiple traces and repeated runs, and judge on hit rate.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three independent experiments from different parents: an O(1) ring-buffer ghost (v001), a CLOCK-style pop/push rewrite of main eviction (v002), and a 5% small queue from baseline (v003). Only v001 has a mechanism for its gain (lower `put` cost at unchanged hit rate). v003 shows a modest hit-rate gain, but its large latency drop is implausible for a one-constant edit. v002 was neutral and probably won on noise.

## Patterns

- Both behavior-preserving rewrites (v001, v002) kept hit rate and eviction decisions identical. Only the one that targeted ghost maintenance (v001) moved `put`. The other, on main eviction (v002), did not.
- Together these confirm that the `put` tail comes from ghost maintenance, not from main-queue eviction or membership checks.
- The per-op-time headline tracks `get` p50, so a variant that improves `put` can still look worse on the headline (v001). `get` shifts appeared in variants that never touched `get` (v001, v002), so they are noise.
- The 5% small ratio now has a second hit-rate gain on workload-a, from a clean baseline. It is consistent in direction but still small, and it is single-trace.
- The latency drop in v003 has no mechanism and should not be credited to the ratio change.
- Whole-cache footprint grows slightly with each new structure (ring bookkeeping here), and the per-entry `metadata-size` metric does not show it.

## Open questions

- Does the 5% small-ratio gain survive repeated runs and other traces and sizes?
- Does v001's ring ghost combine with the index and scan work from earlier cycles without a `get` regression?
- What is left in the `put` tail now that ghost trimming is O(1)? Candidates are hash-map operations and the small-to-main move.
- Is the `get` p50 shift real or a layout effect?

## Continuation

Adopt v001's ring-buffer ghost as the base for `put` work, and stop reworking main-queue eviction (v002 shows nothing to gain). Treat 5% as the small ratio and stop sweeping it unless repeated runs on other traces disagree. Next, re-measure `get` p50 with repeated runs, and look for the remaining `put` tail cost outside ghost trimming and eviction. Judge on hit rate and on latency changes that have a clear mechanism.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
