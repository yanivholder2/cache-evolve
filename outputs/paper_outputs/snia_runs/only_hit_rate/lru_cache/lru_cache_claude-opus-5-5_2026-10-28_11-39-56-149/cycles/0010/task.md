# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0010

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants built on cycle 0008's v002, and each was size-gated. v001 added a slow ±2 hill climber on the protected share at large size. v002 doubled the sketch's `reset_at` to 32 × capacity at small size. v003 picked the colder of the probation LRU and the protected LRU as the main-area victim at large size. v002 is the Winner. It is the first gain on small-size web_0 in many cycles, and it also lifted small-size hm_0 at no cost. Both large-size adaptive levers made things worse.

## Patterns

- The small-size limit was the sketch's memory length. Small-size web_0 resisted climber, split, admission and doorkeeper changes, but it moved once frequency history survived across its multi-fill reuse gaps. This is the first lever that helped both small traces together and did not trade one against the other. It agrees with the earlier finding that faster aging hurts: at tight capacity, longer memory helps on both counts.
- Size gates held again in every variant. Cells across each gate were exactly unchanged, so each result can be attributed to its one lever.
- The large-size adaptive levers failed because their signals were weak or biased, not because adapting is a bad idea. The split climber could not tell a 2-point split change apart from web_0's phase changes and the window climber's moves. Shrinking applies at once while growing fills in lazily, which biased it toward a smaller protected share. The frequency-based victim rule compared decayed protected counts against probation counts that had just won the duel, so in effect it evicted protected first. That removed the long-reuse residency that large-size web_0 relies on.
- Protected residency is still load-bearing at large size for both traces, not only for hm_0. Any rule that drains protected faster hurts web_0 as well.
- For the ninth cycle, only hit-rate was measured.

## Open questions

- How far does the small-size `reset_at` gain extend (48–64 ×) before stale counts start to block new hot keys?
- Would a longer aging period also help at large size? It could help keep large-size web_0's long-reuse keys warm, and it is untested.
- Could ghost hits or a shadow comparison give a clean enough signal to adapt the split at large size?

## Continuation

Use cycle 0009 v002 as the new parent, with 32 × `reset_at` at small size. Next, push the small-size multiplier further (48 × and 64 ×), and separately try a longer `reset_at` at large size only. Both are low-risk and use one lever each. Drop per-eviction frequency victim selection and non-decaying hit-rate climbers on the protected split. Only revisit split adaptation with a ghost-hit or shadow signal. `win_cap`, `metadata-size` and `per-op-time` are now nine cycles overdue.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
