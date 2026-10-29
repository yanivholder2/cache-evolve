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
