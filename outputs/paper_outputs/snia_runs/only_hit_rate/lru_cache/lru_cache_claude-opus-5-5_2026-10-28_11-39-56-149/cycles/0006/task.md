# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0006

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants targeted the small-size web_0 gap. Two tuned cycle 0002's W-TinyLFU: a hill-climbed adaptive window (v001), and a tighter sketch with dedup on miss-then-put, 4x width, a doorkeeper and a longer reset (v002). The third widened LIRS's HIR share and history bound (v003). v002 was the only Winner. It won through hm_0, mostly at small capacity, and slipped further on small-size web_0, the cell it was aimed at.

## Patterns

- None of the three levers closed the small-size web_0 gap while keeping the other cells. Each one moved hit-rate between traces or sizes. The sketch fix helped hm_0 and hurt web_0. The adaptive window helped both traces at small size but gave back large-size web_0. The wider HIR share swapped hm_0 for web_0 in about equal amounts.
- Diagnosis: small-size web_0 is limited by recency and window size, not by sketch noise. Only the levers that enlarge the front segment (v001's window, v003's HIR queue) moved it up. Longer frequency memory (v002's reset period) and colder first-touch estimates (doorkeeper) moved it down.
- The one-shot filter is now strong. Dedup plus doorkeeper is the clearest hm_0 gain so far, and it reinforces the cohort-wide lesson that scan-heavy traffic rewards filtering.
- Both v001 and v003 bundled scale-dependent parameters (climber step and sample period tied to capacity, HIR share and history bound changed together), and both lost at one end of the size range. This repeats cycle 0002's lesson that fixed capacity-proportional settings fail at one extreme. v002 also changed four things at once, so its web_0 drop cannot be pinned on one of them.
- For the third cycle, only hit-rate was reported. Two variant insights could not inspect their landed code, so their mechanism explanations are inferred from the specs.

## Open questions

- Which v002 component causes the small-size web_0 loss: the longer reset, the doorkeeper, or both? Does dedup plus width alone keep the hm_0 gain?
- Can v001's climber keep its small-size gains at large size with a smaller step or a capped sample period, and does it stack with v002's sketch?
- Is there a front-segment share that works for both traces, or does it have to adapt?

## Continuation

Build on v002. First, isolate its sketch changes: try reverting the reset period to the shorter value and/or dropping the doorkeeper while keeping dedup and the wider sketch. Then layer v001's hill-climbed window on top, with its step and sample period capped at large capacity rather than scaled linearly. Drop LIRS: tuning HIR share is a trace-dependent trade, not a gain. Change one lever per variant so causes can be attributed, and get `metadata-size` and `per-op-time` measured.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants built on cycle 0003's v002 (W-TinyLFU with the tightened sketch), and each changed one lever aimed at the small-size web_0 gap. v001 shortened the sketch reset period. v002 added a hill-climbed window with step and sample period capped at absolute values, and kept the admission duel in force while the window grows. v003 let candidates re-referenced in the window skip the duel. v002 was the only Winner and the first variant in the run to improve every trace/size cell, with by far its largest gain on small-size web_0.

## Patterns

- The cycle 0003 diagnosis held: small-size web_0 is limited by window size. Only enlarging the window recovered it in a large way. Sketch aging (v001) and an admission bypass (v003) each moved it only a little.
- Small-size hm_0 depends on strict one-shot filtering. Both levers that loosened admission gave back much of v002's earlier hm_0 gain at tight capacity. v001 did it through faster count decay and doorkeeper clears. v003 did it by letting burst-then-dead blocks through on a single in-window hit. The long reset period and the doorkeeper should be treated as load-bearing.
- Fixing one lever per variant worked. Each result maps to a clear cause, unlike in earlier cycles. The cause of cycle 0003's small-size web_0 drop is now narrowed. The reset period explains little of it, so the doorkeeper's colder first-touch estimates, or simply the window being too small, remain the explanation.
- The lesson about scale-dependent parameters was confirmed from the other direction. Capping the climber's knobs instead of scaling them with capacity removed the large-size loss of cycle 0003's climber. The floor at the old window size protected hm_0.
- The cohort repeated an earlier lesson: hard overrides lose to soft, relative signals. The in-window reuse signal carries real information on web_0, but as an unconditional bypass it costs more than it gains.
- For the fourth cycle, only hit-rate was measured.

## Open questions

- Which cap does the work in v002: the sample-period cap or the step cap? Both test sizes hit the same caps, so their effects can't be separated. How far does the window actually move in each cell?
- With a larger adaptive window in place, does the doorkeeper still cost anything on web_0, or has the climber absorbed that loss?
- Could in-window reuse help as a bounded bonus or tie-breaker inside the duel rather than as a bypass?
- Does the climber's per-access sampling add measurable per-op cost?

## Continuation

Build on cycle 0004's v002 and keep its sketch (long reset, doorkeeper, dedup, wide width), its capped climber, and its window floor. Next, vary the period cap and the step cap one at a time, and log the final window size in each cell. Separately, test in-window reuse as a small duel bonus only. Drop shorter reset periods and any unconditional admission bypass. Getting `metadata-size` and `per-op-time` reported is now four cycles overdue and should come before more complexity is added.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants built on cycle 0004's v002, and each changed one lever. v001 shortened the climber's sample-period cap. v002 gave in-window reuse a +1 bonus in the duel. v003 raised the window ceiling from 40% to 75%. v003 is the nominal Winner, but it matched its parent exactly in every cell. It won only because it changed nothing; it did not improve anything. v001 and v002 lost in most cells, and they lost most on small-size web_0, the cell both were aimed at.

## Patterns

- The cycle produced diagnosis, not gains. Each one-lever test ruled out a cause of cycle 0004's small-size web_0 gain or limit. The window ceiling never binds, because the window never reaches 40%. A shorter period cap hurts. Duel bonuses for in-window reuse hurt.
- The climber is limited by how fast and how steadily it climbs, not by how high it may go. Its moves land lazily, as misses fill the window. A round shorter than the time a resize needs to fill measures a half-applied move, and flip-on-drop then reverses growth that would have paid off. The sample period should be tied to how long a resize takes to fill, not shortened for speed.
- In-window reuse carries no signal beyond the sketch. Every window hit is already counted, so a bonus counts the same evidence twice and tilts close duels toward recency. That holds for both the cycle 0004 bypass and this cycle's bounded bonus. The general lesson that soft signals beat hard ones still holds, but only if the signal is independent of the sketch.
- Strict one-shot filtering on small-size hm_0 is still fragile. Anything that adds climber noise or loosens close duels leaks a little of the gain back.
- For the fifth cycle, only hit-rate was measured. The window size the climber settles at, which three hypotheses asked for, was not logged or not read, so the main open question is still answered only by inference.

## Open questions

- Where does `win_cap` actually settle in each cell, and does it oscillate at small size?
- Does a larger step cap, with the period held at its current cap, let the window grow further on small-size web_0 without hurting hm_0?
- Would a sample period set as a multiple of capacity (fill time) beat a fixed access count at both sizes?

## Continuation

Treat cycle 0004's v002 as the effective parent; v003 is the same cache plus harmless headroom. Next, vary the step cap with the period held at its current value. Separately, try a period tied to capacity or fill time instead of a fixed count, or make flip-on-drop less sensitive (for example, require two drops in a row). Drop shorter fixed periods, ceiling increases, and any in-window-reuse bonus. Before the next cycle, read the climber's window-size log for each cell. Getting `metadata-size` and `per-op-time` reported is now five cycles overdue.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
