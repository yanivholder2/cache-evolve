# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0009

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants built on cycle 0007's v001, and each changed one lever behind the 32_768 size gate. v001 gave the doorkeeper four times the bits at small size. v002 cut the protected share to 75% at large size. v003 damped the climber's step on each flip at large size. v002 is the Winner and the second strict gain in a row. Large-size web_0 improved, large-size hm_0 lost a little, and small size was unchanged.

## Patterns

- Size gates held every time. Each variant left the cells on the other side of its gate exactly unchanged. Gating levers by size is now a reliable tool. The open question is which lever to gate, not whether gating works.
- Large-size gains have become a trade between web_0 and hm_0. The step gate, the split gate and the damping rule all moved hit-rate along the same line, gaining on one large trace and losing on the other. Only the split gate came out net positive. Fixed rules that ignore the workload seem close to their limit at large size.
- The doorkeeper hypothesis for small-size web_0 is now tested and rejected. A saturated doorkeeper acts like a roughly uniform +1 bonus on first touches. Making it more accurate removed that bonus unevenly, hurt moderate-reuse keys on small-size hm_0 and did nothing for web_0. Small-size web_0 has now resisted climber, split, admission-loosening and doorkeeper changes.
- Small-size hm_0 is still the fragile cell. Any change that alters how moderate-reuse keys are scored against victims hurts it first.
- For the eighth cycle, only hit-rate was measured. `win_cap`, `metadata-size` and `per-op-time` are still not reported.

## Open questions

- Can a runtime signal tell scan-heavy traffic from medium-reuse traffic at large size? Candidates are the hit-rate response after a flip, how scan-heavy the window traffic is, or a climber or duel that also adjusts the protected split.
- What limits small-size web_0? The candidates left are the admission tie-break, the sketch's `reset_at` period, and the window/main split. None has been tested directly at small size only.

## Continuation

Use cycle 0008 v002 as the new parent, with the 75% large-size split. Keep the doorkeeper, the 80% small-size split and the climber settings as they are. Next, try a lever that adapts to the workload at large size, for example letting the protected split adapt at runtime. Separately, test the tie-break or `reset_at` at small size only. Drop doorkeeper resizing, flip damping and further static large-size gates on the step or split. Reading `win_cap` and reporting `metadata-size` and `per-op-time` is now eight cycles overdue.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
