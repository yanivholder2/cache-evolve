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
