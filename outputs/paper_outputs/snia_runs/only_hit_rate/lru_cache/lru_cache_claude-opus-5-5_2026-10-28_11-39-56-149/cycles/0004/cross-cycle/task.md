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
