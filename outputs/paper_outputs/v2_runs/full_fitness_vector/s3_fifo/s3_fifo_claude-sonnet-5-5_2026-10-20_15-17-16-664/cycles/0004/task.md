# Task: cycle-insight-writer

Cycle: 0004
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0004

## Winners

- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0004/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0004/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0004/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0004/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0004/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0004/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three single-point tweaks to the baseline S3-FIFO were tried: a lower small-to-main promotion threshold, a smaller small-queue ratio, and ghost recycling of main evictions. The first was lost to an infrastructure failure. The two Winners were the small-queue ratio change and the ghost-recycling change.

## Patterns

- The v001 rejection came from a Maven 429 while downloading the Caffeine jar. It says nothing about the promotion-threshold idea, which is still untested.
- Both Winners report large per-op latency drops, but each variant's own analysis says the change has no mechanism for that. Treat the drops as run-to-run noise.
- Only the small-queue ratio change (v002) has a plausible hit-rate gain, and it is modest. It is a constant-only change that adds no work and leaves metadata unchanged.
- The ghost-recycling change (v003) lowered hit rate. Main evictions crowd out small-queue demotions in the shared ghost, which weakens its one-hit-wonder filter. It likely won through the noisy latency numbers, not on merit.
- All evidence comes from one trace (LIRS workload-a) at one cache size, so the gains are not shown to generalize.

## Open questions

- Is the latency improvement reproducible, or is the fitness signal noisy enough to promote regressions?
- Is the best small-queue ratio workload-dependent? Is 10% better than other values such as 5% or 15%?
- Does the lower promotion threshold (freq>=1) help, now that v001's infrastructure failure has hidden it?

## Continuation

Build on v002's 10% small queue and retry the promotion-threshold hypothesis from H01, since it never got evaluated. Drop ghost recycling of main evictions unless it is redesigned, for example with a separate ghost or without the freq=1 head start. Next cycle should sweep the small ratio and use more traces or repeated runs to separate real latency effects from noise.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Three variants were tried: a lower promotion threshold (freq>=1) on the 10% small queue (v001), a 15% small-queue ratio (v002), and a separate bounded main-eviction ghost (v003). Only v003 shows a real, modest hit-rate gain. v001 and v002 were slight hit-rate regressions, so they probably won through noisy latency numbers.

## Patterns

- Both constant-level tweaks to the small-queue path (looser promotion, larger small queue) lost hit rate against the 10% parent. This points to 10% with the baseline threshold being a local optimum on this trace.
- The small queue's filtering role appears to matter more than quick promotion or extra residence time. Anything that lets more one-hit-wonders into main, or takes capacity from main, hurts.
- v003 supports the diagnosis from Cycle 1. Main-eviction recycling failed because it diluted the shared demotion ghost, and a separate ghost keeps the filter intact and gives a small positive gain. The cost is a struct-level metadata increase and a slower typical put.
- Latency shifts appeared again in all three variants, including edits that add no hot-path work. Every variant insight attributes them to noise. Latency is still not a trustworthy selection signal.
- All evidence still comes from one trace (workload-a) at one cache size.

## Open questions

- Is v003's hit-rate gain larger than run-to-run noise, and does it hold on other traces and sizes?
- Is the 25% main-ghost size a good choice? Is it worth the metadata and put-latency cost?
- Is the small-ratio optimum below 10% (the 5% point was not tested this cycle)?

## Continuation

Treat v003's separate main-eviction ghost as the leading direction and tune it: ghost size, and whether it needs a freq head start on re-entry. Drop the freq>=1 promotion threshold and larger small-queue ratios (15% and up), since both regressed. If anything is swept further, test 5% and below. Evaluate on more traces and repeated runs, and judge on hit rate rather than latency.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
