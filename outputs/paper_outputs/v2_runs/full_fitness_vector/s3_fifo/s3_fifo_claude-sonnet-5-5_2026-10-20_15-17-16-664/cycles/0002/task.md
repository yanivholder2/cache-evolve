# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0002

## Winners

- v002
- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0002/hypotheses/H03.md

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
