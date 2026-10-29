# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0007

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0007/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants changed one knob of the climber in cycle 0004's v002 lineage. v001 raised the step cap 4x. v002 made a reversal wait for two drops in a row. v003 tied the sample period to capacity at the large size only. v003 is the nominal Winner because it lost the least. It matched its parent exactly at small size and was slightly worse in both large-size cells, so it did not improve anything. For the second cycle in a row there was no real gain.

## Patterns

- The climber's knobs are now used up. Over cycles 0005 and 0006, the ceiling, a shorter period, a longer period, a larger step and a slower flip rule have all been tried. Each one was neutral or worse. The cycle 0004 settings sit at a local optimum.
- Small-size web_0 did not move under any climber change. Faster steps and slower reversals both failed to lift it, and slower reversals hurt it the most. Its remaining gap is probably structural, coming from admission, the duel or how the main area is split, and not from where the window settles.
- Single-sample drops are real signals, not noise from a half-applied resize. Delaying reactions cost the most on the cell the change was aimed at. v003 showed that a capped step settles quickly, so the cycle 0005 "lazy-resize transient" diagnosis is withdrawn.
- The step size relative to capacity is what matters. A large step amplifies noise on small-size hm_0, where one-shot filtering is still the fragile, load-bearing property. At the large size, the capped step limits how many rounds the climber gets, so a longer period only slows it down.
- For the sixth cycle, only hit-rate was measured. `win_cap` was again not read, although v001 may have produced a debug log of it.

## Open questions

- Does `win_cap` at small size oscillate or settle? Read the v001 `climb` log if it exists.
- If small-size web_0 is limited by admission or the duel, which part is it: the doorkeeper's cold first-touch estimates, or how the main area is split between probation and protected?
- Would a step proportional to capacity at the large size only (about 1% or less), with the period scaled to match, help large-size web_0 without touching small size?

## Continuation

Stop tuning the climber. Treat cycle 0004's v002 as the parent and leave its climber settings alone. Next, aim at admission and the main area: test whether the doorkeeper still costs web_0 now that the window adapts, and try varying the main area's probation/protected split. A large-size-only step scaled to capacity is the one climber idea still worth testing. Drop hysteresis, larger fixed step caps and period-only changes. Read `win_cap` logs, and get `metadata-size` and `per-op-time` reported, which is now six cycles overdue.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
