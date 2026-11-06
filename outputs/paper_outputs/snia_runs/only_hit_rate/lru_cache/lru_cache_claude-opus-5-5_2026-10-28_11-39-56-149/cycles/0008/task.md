# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0008

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Each variant changed one lever on the cycle 0004 v002 cache (v003 used its twin, cycle 0005 v003). v001 used a larger initial climber step at large capacity only. v002 changed the main area's split to shrink protected and grow probation. v003 randomly admitted warm candidates that lost the duel. v001 is the Winner and the first strict gain since cycle 0004. It is small: large-size web_0 improved, large-size hm_0 lost a little, and small size was unchanged.

## Patterns

- Gating a parameter by cache size worked as designed. The large-size step gain and the small-size cells were independent, and their effects added up as predicted. This supports the lesson from earlier cycles about scale-dependent parameters, but the step lever now looks used up.
- Small-size web_0 still has not moved. This cycle ruled out two more structural causes. A longer probation segment hurt it, and so did breaking lockout of recurring candidates. Neither the main-area split nor stale victim counts explain the gap.
- Large protected is load-bearing for hm_0 at both sizes, like strict one-shot filtering. Shrinking protected pushes hm_0's reuse set into the eviction path, and the duel cannot prevent that.
- Admission is not too strict. Even a loosening that is rare and limited to warm keys hurt every cell, and it hurt small-size hm_0 the most. The deterministic duel that admits on ties is lenient enough under the long reset period. This is the third cycle in which loosening admission lost.
- The one useful side signal was that a longer probation helped large-size web_0. Medium-range reuse at large size gains from more time in probation, but that gain did not outweigh the hm_0 losses.
- For the seventh cycle, only hit-rate was measured. v003's insight could not see the code, so its reading depends on the change matching the hypothesis.

## Open questions

- What does limit small-size web_0? The remaining candidate is the doorkeeper's cold first-touch estimates, which has been suggested since cycle 0004 and never tested directly.
- Could probation be lengthened at large size only, the same way v001 gated its step, to keep the large-size web_0 gain without touching hm_0's protected set?
- Could a separate step for shrinking the window, or a decay that depends on cache size, recover v001's small large-size hm_0 loss?

## Continuation

Use cycle 0007 v001 as the new parent. Keep the protected cap, the deterministic tie-admitting duel and the climber settings as they are. Next, test the doorkeeper directly on small-size web_0 (for example, disable it or weaken it at small capacity only). Separately, try a split change that applies at large capacity only. Drop random or probabilistic admission, global split changes and further increases to the initial step. Reading `win_cap` and reporting `metadata-size` and `per-op-time` is now seven cycles overdue.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
