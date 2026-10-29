# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0003

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0003/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0003/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0003/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0003/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0003/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants changed promotion and placement inside the Dash segment and left Item layout alone. They tried MRU landing (v001), free-slot balanced placement (v002) and a stash reuse gate driven by `lfu_counter` (v003). The Winners were the gate (v003), which improved both hit-rate and get tail, and balanced placement (v002), which won on latency only.

## Patterns

- Hit-rate and per-op time move together through one shared cost: how many multi-bucket moves (promotions plus demotion cascades) each hit triggers. Changes that reduce those moves (v002, v003) were faster. The change that added moves (v001) was slower even though each insert was cheaper.
- Hit-rate improved only when a change decided what to keep based on reuse evidence (v001 protects items that were hit; v003 admits only items with proven reuse). Changes that decided placement by free space alone (v002) did nothing under pressure and kept cold items when there was slack.
- In this design, insert position and promotion rules are coupled. Where an item lands sets how quickly it climbs tiers on its next hit, not only its eviction order. The hottest-in-target → probing rule makes this coupling worse.
- The gains from reuse-aware changes were largest on hm_0 and at the large cache size, where there is room for a reused working set once churn stops.
- Fixed bookkeeping on every hit (counter writes, comparisons with the victim) costs a little on median get even when it cuts the tail a lot.

## Open questions

- Is v003's median get penalty caused by the per-hit counter increments in target/probing, or by the victim comparison at admission?
- Would v001's protective `demote_pos = Back` still raise hit-rate if `promote_pos` stays Front, or if it is combined with v003's gate, without the extra promotion churn?
- Is v002's latency gain real or partly noise? Could the cheap no-evict move into a free slot be kept for demotion only, without letting cold evictees fill probing?
- Should the gate threshold grow with cache size?

## Continuation

Build on v003's reuse gate as the main line. Then try adding v001's MRU landing for demotions only, and/or requiring reuse before target → probing, to gain hit-rate without extra churn. Re-measure v002's chain-shortening and keep it only if the latency gain holds and free slots go to hot items, not cold evictees. Drop pure capacity-based balanced placement as a hit-rate strategy.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Two variants built on cycle 0001's stash reuse gate. v001 removed the gate's extra parts (the per-hit counter writes and the victim comparison), and v002 added reuse-aware MRU landing for demotions. The third, v003, added a reuse flag to v002's free-slot placement. The Winners are v001, which got a faster get path for a small loss of large-cache hit-rate, and v003. v003's own insight reports no improvement on any dimension, so its place among the Winners should be read with caution (see Open questions).

## Patterns

- The cycle-0001 gate splits into separate parts. The two-hit stash threshold gives almost all of the hit-rate gain. The counter writes on normal-tier hits cause most of the median get cost. The victim comparison buys a small large-cache hit-rate gain, mainly on hm_0, for a small get-latency cost. Each part can now be priced on its own.
- Once the gate is in place, other "protect reused items" mechanisms add little. v002's MRU demotion landing helped only slightly, and only where the gate had not already saturated the benefit (web_0 large). v003's flag-limited free-slot placement left hit-rate unchanged in every cell. Extra reuse evidence mostly duplicates what the gate already captures.
- Any reuse check on the common target-hit path costs a little median get, whether it is a counter read, a compare or a conditional store. The summary latency statistic is sensitive to this, so the common hit path should stay free of bookkeeping.
- Large cache sizes are where the remaining hit-rate is won or lost. Both the victim comparison's benefit and the cost of dropping it showed up there, not in the small-cache cells.
- The per-op-time benchmark looks too noisy to separate small changes. v003 regressed by more than its code change can plausibly explain, and that casts doubt on v002's cycle-0001 latency gain.

## Open questions

- Is the per-op-time measurement stable? Re-running an identical variant would show how large the noise is before more latency conclusions are drawn.
- Can a cheaper victim guard win back v001's lost large-cache hits without per-hit counter writes? For example, a single-bit check, or a guard active only at large sizes.
- Why is put p99 higher in v001? Is it from the extra stash churn of unconditional promotion into a full target, and can it be reduced?
- If free-slot placement does not drive hit-rate, what limits web_0 at the large size?

## Continuation

Use v001 (threshold-only gate, lean hit path) as the new baseline. Next, try a cheap, write-free victim guard, possibly only at large cache sizes, to recover hit-rate there. Drop the free-slot placement line (from cycle-0001's v002) and MRU demotion landing, since both are redundant with the gate. Before trusting small latency differences, check measurement noise, for example with a re-run of the baseline.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
