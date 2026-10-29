# Cycle 0009 Insight

## Cohort outcome

This cohort finally ran the noise control. v001 is an identical copy of cycle 0007 v003, and v002 and v003 are two changes on the 0008 v001 eviction lineage: v002 halved `PROBE_WINDOW`, and v003 added aged, bounded stash-victim protection. All three are Winners. v001 is valuable as evidence, not as code. v003 is the real result: the first stash-exit change to gain hit-rate (hm_0 at both sizes) with no loss in any cell. v002 is hit-rate neutral to slightly positive.

## Patterns

- Cross-session latency is not trustworthy. The unchanged copy slowed sharply and uniformly, put included. v002 and v003 sped up uniformly against their parents, including on paths they never touch. So the "uniform shift" seen in cycles 0005–0008 was environmental drift. Cycle 0008 v001's apparent get regression is probably drift too. From now on, latency can only be scored against a same-session control.
- Under drift, the variant writers fell back on comparing ops within one run, and that turned out to be a usable signal. In v003 the stash-full put path shows no tail inflation relative to get. That suggests bounding protection to two slots avoided cycle 0008 v003's put cost.
- The stash-exit lever is alive once protection ages out. Unbounded protection lost everywhere. Recency-gated protection with a single reprieve and no MRU refresh gained on hm_0 and lost nowhere. The gain depends on the trace, not on cache size: it shows up where stashed keys get short-range reuse (hm_0) and not on web_0. That is the opposite of what H03 predicted. web_0-large is still untouched by any lever tried so far.
- `PROBE_WINDOW` is now confirmed as hit-rate-inert in both directions on the eviction arm. Narrowing it cost nothing and may help marginally, which fits the earlier finding that the demoted resident is disposable. It is a pure latency knob, and its latency effect is still unmeasured.
- Again, no variant could read its overlay source. Mechanisms are inferred from the hypotheses.

## Open questions

- Did v002 actually lower get latency? This needs a direct same-session read of v002 against v001. None of the variant insights made that comparison. The control was also the swap lineage, not the eviction parent.
- In this session, is the eviction lineage (0008 v001) faster than the swap lineage (0007 v003)? The cross-session evidence hints that it is, but this is unconfirmed.
- Does v003's protection stack with v002's narrower window? They act on different paths (stash exit vs. probing arm), so unlike earlier promotion levers they may not overlap.
- What would move web_0-large? Stash exit, the window, and arm eviction have all left it flat.

## Continuation

Adopt v003 as the hit-rate baseline. From now on, every cohort must include an identical-copy control of its own parent, and latency verdicts may only be scored against that control. Next, try combining v003 with v002's narrower window, since they act on different paths. You could also tune `STASH_PROTECT_AGE`, judging it on hm_0. Drop further `PROBE_WINDOW` hit-rate hypotheses. For web_0-large, look at admission (the stash gate or the door guard), not at stash exit or the probing arm.
