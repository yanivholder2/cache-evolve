# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003

## Winners

(none)

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three changes to how Dash moves items between stash and the normal tier were proposed: demote to the stash MRU end (H01), require two stash hits before promotion (H02), and balance promotions between target and probing buckets (H03). Only the two-hit gate (v002) got evaluated. The other two variants (v001, v003) failed because the simulator build hit HTTP 429 rate limits from Maven/Gradle, so the failure was infrastructure, not the policy. There are no Winners.

## Patterns

- The cohort's working theory is that the small normal tier gets flushed by weakly re-referenced items. v002 is the only result, and it supports that theory: gating admission to the protected tier raised hit rate on every trace and size tested. The gain was largest at the larger size, on hm_0 most of all.
- The hit-rate gain cost latency. `get` got slower at both p50 and p99, which contradicts the hypothesis's guess that a first stash hit would be cheaper. `put` slowed too, possibly because stash pressure rises when items stay in stash longer. Benchmark noise has not been ruled out as the cause of the `put` slowdown.
- Reusing the unused `lfu_counter` field kept metadata size unchanged. That makes it a good pattern to follow for cheap per-item state.
- H01 and H03 are still untested. Their failures say nothing about the ideas themselves.

## Open questions

- Is the `put` slowdown real (from stash buckets filling and evicting more often) or benchmark noise?
- Can the two-hit gate keep its hit-rate gain without the extra MRU move on the first stash hit?
- Would H01 (MRU demotion) or H03 (balanced promotion) stack with the gate? H03 claims it lowers cascade cost, which could offset the gate's latency.

## Continuation

Re-run H01 and H03 unchanged, since their ideas were never measured. Build on v002's two-hit gate: try a variant that leaves the item in place on the first stash hit (just set the counter, no MRU move), and try combining the gate with H03's balanced promotion to win back per-op time. Keep the gate, and do not trade away its hit-rate gain for latency.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants were built on cycle 0001's two-hit gate (v002) and all were evaluated: leave the item in place on the first stash hit (v001), send twice-hit items into free probing space (v002), and demote normal-tier victims to the stash MRU end (v003). Each one improved median `get` slightly, and none improved tail latency. There are no Winners. v003 came closest, with higher hit rate everywhere and slightly lower `get` p50, but its tail got slightly worse.

## Patterns

- The cohort settled one question: the gate's per-op cost is not on the first-hit path. Removing the first-hit MRU move (v001) helped p50 only a little, left p99 unchanged, and cost some hit rate. The tail cost comes from the second-hit promote/demote cascade and from lookup length.
- Changes that keep proven items alive longer helped hit rate. v002 did this by using free probing space instead of evicting a target resident. v003 did it by giving demoted items a full stash residency. v003's gain was largest on hm_0 at the large size, the same configuration that gained most from the gate in cycle 0001 and lost most in v001. That configuration is the main place where re-reference gaps decide the result.
- Tail latency got slightly worse in every variant, each for its own reason: the cascade (v001), a three-bucket lookup for items placed in probing (v002), and stash occupancy pressure on `put` (v003). Stash pressure is now the leading explanation for the `put` regression, which v003 made worse. v002 lowered demotions into stash and barely changed `put`, so the cause is not settled.
- v002 and v003 work on different parts of the policy. v002 changes where promotions land, and v003 changes where demotions land. Neither has been tried together with the other.

## Open questions

- Does combining v003 (MRU demotion) with v002 (probing overflow) stack the hit-rate gains? Would v002's fewer demotions offset the stash pressure v003 adds?
- Can the probing-overflow lookup penalty be removed, for example by changing the search order or moving items back into target once it has space, without giving up its hit-rate gain?
- Is the remaining `put` p99 regression caused by stash fullness, or is it benchmark noise? None of the variants has separated the two.

## Continuation

Take v003 (MRU demotion on the gate) as the new base. It is the strongest hit-rate result, and its only cost is a small tail regression. Next, try v003 combined with v002's probing overflow, and look for a way to shorten the lookup for items placed in probing. Drop the first-hit no-reorder idea (v001): it costs hit rate and does not help the tail. Aim latency work at the second-hit cascade and at stash occupancy.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
