# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0002

## Winners

- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0002/hypotheses/H03.md

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
