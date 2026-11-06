# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three independent rewrites of the baseline LRU were tried: SLRU segmentation (v001), a CLOCK second-chance ring (v002), and a TinyLFU admission gate in front of LRU (v003). The SLRU split (v001) was the only Winner. It raised hit-rate on every trace and size, and the gain grew with capacity.

## Patterns

- All three variants share one idea: a block seen only once should not displace blocks that were reused. Results depended on how firmly each enforced it. A soft filter (SLRU probation, CLOCK's unset bit) helped. A hard gate (TinyLFU's strict-greater reject) hurt the recency-driven trace.
- The two traces pull in different directions. hm_0 rewards frequency filtering at both sizes. web_0 relies on short-range recency reuse, mostly at the small size. Any mechanism that delays or blocks a block's second access, such as a short probation list or a reject on a count-of-1 tie, loses there.
- Capacity matters. With more room, SLRU's gains grew. With a large ring, CLOCK's bits stayed set and it decayed toward FIFO on web_0. At the small size, TinyLFU's web_0 loss was worst.
- No variant measured `metadata-size` or `per-op-time`; every fitness vector held only hit-rate. CLOCK's main claims (a smaller node, a cheaper `get`) and SLRU's and TinyLFU's possible costs are all still unverified.

## Open questions

- Does SLRU keep its edge on the cost axes once they are measured, and does CLOCK's expected cost advantage make up for its slight large-size web_0 loss?
- Would a larger probation share at small capacity, or an adaptive split, recover blocks whose reuse distance is longer than probation?
- Can admission filtering help web_0 if it is put behind a recency window or allowed to admit on low-count ties?

## Continuation

Build on v001's SLRU. Next, tune the probation/protected split, especially at small capacity, or try a W-TinyLFU design: a small admission-free window in front of SLRU, gated by the sketch with ties admitted at low counts. Do not try plain TinyLFU over LRU again in its v003 form. Keep CLOCK as an option for the cost axes, and test whether CLOCK-based segmentation keeps its cheap `get`. Make sure the next cycle evaluates `metadata-size` and `per-op-time`, so the trade-offs can actually be compared.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Two variants extended cycle 0001's SLRU: a capacity-sized ghost list that sends ghost hits straight into protected (v001), and W-TinyLFU, a 1% admission-free window plus a tie-admitting count-min gate in front of SLRU (v002). A third replaced it with LIRS (v003). W-TinyLFU (v002) was the only Winner. It gained on hm_0 at both sizes and most on web_0 at the large size. Its one slight loss was web_0 at the small size.

## Patterns

- Every variant tried to reach further back for reuse evidence than SLRU's probation list can: ghost keys, a frequency sketch, or LIRS's non-resident history. What decided the outcome was how much weight each gave that evidence. Giving a returning key full protected status (v001) hurt. A relative comparison against the current victim, with ties admitted (v002), helped. This repeats cycle 0001's lesson that soft filtering beats hard filtering, with ties now on the lenient side.
- Ghost or non-resident history detects hm_0's long reuse distances well. On web_0, though, one return from outside the cache is a weak signal. It should not displace keys that were re-referenced while still resident.
- The same weak spot appeared again: web_0 at the small size. Both v002 and v003 use a fixed 1% front segment (window or HIR queue), which is too narrow there for web_0's short-range reuse. v001's ghost window, set equal to capacity, caused the opposite problem at the large size. Fixed fractions tied to capacity keep failing at one end of the size range.
- LIRS beat LRU in most cells. Its weak cell and its likely fix (a larger HIR share) match v002's, but it did not beat v002.
- For the second cycle in a row, `metadata-size` and `per-op-time` were not measured. Ghost keys, sketch counters and LIRS's three link pairs all add costs no one has checked.

## Open questions

- Does a larger or adaptive window (for example, hill-climbing the window/main split) close v002's small-size web_0 gap without giving back hm_0 gains?
- How much of the small-size slip comes from sketch tuning rather than window size: a sketch sized near capacity, frequent halving, and the double increment on miss-then-put?
- Are v002's sketch and window cheap enough on the cost axes compared with plain SLRU?

## Continuation

Build on v002 (W-TinyLFU over SLRU) and keep its tie-admitting rule. Tune the window fraction next, preferably adaptively, especially at small capacity. Separately, test a wider sketch, a longer reset period, and removing the double increment. Drop ghost-to-protected promotion. If ghost history is tried again, it should only feed the admission comparison. Put LIRS aside unless a tuned HIR share is tested directly against v002. Making the evaluator report `metadata-size` and `per-op-time` is now overdue.


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


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

All three variants built on cycle 0009's v002 and changed the frequency sketch. v001 raised the small-size `reset_at` to 64 × capacity. v002 raised the large-size `reset_at` to 32 × capacity. v003 switched to conservative update at every size, with no size gate. v003 is the Winner, but its gain is small and uneven: web_0 @ 16384 rose, and the other three cells slipped slightly.

## Patterns

- Sketch memory length has peaked. At small size, going from 32 × to 64 × produced equal and opposite moves at noise level. At large size, 32 × hurt both traces. The useful aging period seems bounded in absolute access count, not proportional to capacity. Small size is already near its best, and large size should stay at 16 × or shorter.
- Two variants independently named doorkeeper fill as the likely cause. The doorkeeper is cleared only on reset, so a longer period lets it saturate. One-shot keys then get a +1 from false positives. That links the c8 doorkeeper finding to the aging period: the two cannot be tuned separately.
- Admitting on ties is the hidden hinge. Conservative update did not fix inflated one-shot keys, because the doorkeeper already holds them at the floor. It lowered victim scores instead. That helped long-reuse web_0 at small size and loosened hm_0's scan filtering at both sizes. Any change that lowers victim estimates is really a change to admission strictness.
- For the first time in several cycles, the Winner is not size-gated. It is a trade (web_0 gains, hm_0 pays everywhere), not a clean single-axis gain. Its large-size cells moved for reasons unrelated to the stated rationale.
- Large size has still found no lever that helps both traces together. Longer memory joins the step, split, damping, split climber and victim selection on the list of failed levers.
- For the tenth cycle, only hit-rate was measured.

## Open questions

- Would a stricter tie rule recover hm_0's loss while keeping v003's web_0 gain? For example, require the candidate to strictly beat the victim when the victim's count is above 1, or apply strict-beat only to the doorkeeper-only candidate.
- Should conservative update be gated to small size only? Large-size web_0 got nothing from it and large-size hm_0 lost.
- Would clearing the doorkeeper on its own shorter schedule, separate from `reset_at`, let a long sketch memory stop leaking one-shot +1s?
- Is a shorter large-size `reset_at` (for example 8 ×) the direction that actually helps?

## Continuation

Use cycle 0010 v003 as the parent, but treat its gain as fragile. Next, try one of two single-lever changes: tighten the duel's tie rule (strict-beat for warm victims), or gate conservative update to small size. Separately, try clearing the doorkeeper on its own schedule, or a shorter large-size `reset_at`. Drop any further increase to `reset_at` at either size. `win_cap`, `metadata-size` and `per-op-time` are now ten cycles overdue.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
