# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0010

## Winners

- v002
- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cohort tried to untangle cycle 0006's bundled changes. v001 dropped the size gate on top of 0006 v002's cascade. v002 replaced the guarded full-probing cascade with a single outright eviction. v003 put the `PROBE_WINDOW` age check on top of 0006 v001's swap. The Winners are v003 and v001, and both are mostly latency wins. v003 recovered much of the swap's get and put tail cost and gave up a little hm_0 hit-rate. v001 slightly improved get p99 and lost a little small-cache hit-rate. v002 is not a Winner, but on hit-rate it may be the most useful result.

## Patterns

- The two hypotheses that said "the gains will stack" were both refuted, and they fail in the same way. Gate removal only helps when a swap is present (v001). The recency window only helps when a cascade is present (v003). Each lever fixes the same underlying cost, which is a promotion that evicts a still-useful resident, so once one lever is in place the other adds nothing. Promotion levers on the same arm overlap; they do not stack.
- The size gate is redundant only on the swap lineage. Under the cascade it was holding up hm_0 in small caches. Cycle 0006's claim that the gate is redundant should be read with that condition attached.
- The probing evictee is disposable. With v002, dropping it outright cost no hit-rate in any cell. The cascade's re-homing was not keeping useful keys, so the cheap evict-and-free-a-slot path is safe on hit-rate. This also resolves cycle 0006's dilemma: you can free a slot without the cascade's multi-bucket work.
- A recency window is better used as a throttle on how often the swap fires than as a hit-rate filter. The swap stays on the get path but fires less often. This is the first lazy check that brought the tail down, not just moved cost into it.
- Noise is still the binding constraint. v002's regression was uniform across ops and percentiles, including paths its change never touches, so it has to be an artifact. That makes it the third cycle in a row with an unexplained uniform shift. Again, none of the variants could read its overlay source.

## Open questions

- Is v002's latency regression real? Re-run it next to its parent on the same host.
- Would v002's single eviction, put on the swap/window lineage in place of the swap, keep v003's hit-rate and also free slots? That would combine the hit-rate of the swap, the occupancy of the cascade, and a cost lower than either.
- Could a window sized to the cache (wider in small caches) win back v003's hm_0-small loss?

## Continuation

Take v003 as the baseline, and drop further "stack two promotion levers" hypotheses, because they overlap. Top candidate: replace v003's swap with v002's single outright eviction while keeping the window as the throttle. Re-run v002 against its parent on the same host before trusting any latency verdict. This is the sixth cycle asking for an identical-baseline noise run, and it should be done in the next cohort, not deferred again.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants branched from cycle 0007 v003, the window-throttled swap lineage. v001 replaced the swap with a single outright eviction of the probing resident. v002 doubled `PROBE_WINDOW`. v003 changed stash victim choice to prefer never-hit entries. All three are listed as Winners, but only v001 brought a useful result: a large-cache hm_0 hit-rate gain, with its latency verdict lost to another uniform shift. v002's hit-rate gain was negligible, and it was bought with a real get-latency cost. v003 lost hit-rate in every cell.

## Patterns

- The eviction-instead-of-swap result now holds on two lineages. Dropping the probing evictee cost nothing under the cascade in cycle 0007, and under the window throttle it cost nothing and slightly helped large-cache hm_0. Keeping the demoted resident is worthless. Freeing a slot is at least neutral and possibly helpful.
- `PROBE_WINDOW` is a latency throttle, not a hit-rate lever. Widening it let many more hottest hits into the costly swap, and the median rose as well as the tail, so the swap is not a rare event once admitted. The parent's window already sits near the useful end of that trade-off. The hm_0-small dip has some other cause.
- The stash's LRU-position eviction is already the right scan-resistant policy. Protecting one-hit stash entries filled the stash with keys that are rarely reused and starved fresh arrivals before the gate could judge them. Small caches were hurt most, and web_0-large still did not move. This was the first test of "who leaves the stash", and it shows the lever cuts the wrong way unless the protection ages out.
- Any added scan or check shows up exactly where it runs. v003's put cost appeared on the stash-full put path, and v002's cost appeared on the get path. This is consistent with the recurring "work on a path shows on that path" lesson.
- Noise is still unmeasured. v001's near-uniform slowdown, including the median of an untouched put, has the same signature as cycle 0007 v002. This is the fourth cycle running with an unexplained uniform shift, and still no identical-baseline re-run has been done. As before, no variant could read its overlay source. v002 also flags a possible implementation side effect, a per-get bound computation, that could inflate its median.

## Open questions

- Does v001 actually improve latency once it is timed against v003 on the same host in the same session?
- What is holding back hm_0-small on the swap lineage, if the window is not the cause?
- Would an aging bound on stash protection, where one old hit does not shield an entry for long, recover any of the web_0-large gain, or is the stash-exit lever dead?
- Does narrowing the window on v001's eviction lineage cut latency further without costing hit-rate?

## Continuation

Adopt v001 (window throttle plus single eviction) as the baseline on hit-rate grounds. Before trusting any latency verdict, run v001, v003 from cycle 0007, and an identical copy of the parent together on one host. This request has gone unmet for seven cycles, and it has to be a hypothesis of its own in the next cohort. Drop window widening. Drop unbounded stash-victim protection. Look for hm_0-small and web_0-large hit-rate in the arm's mechanics or in admission, not in throttle tuning.


### Cycle 0009

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
