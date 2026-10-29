# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0009

## Winners

- v003
- v002
- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants branched from cycle 0005 v001. v001 merged in the direct full-probing swap and dropped the stash-side size gate. v002 replaced the strict reset with a lazy tick-age check on the hottest/full arm. v003 applied the swap idea one tier down, to stash→target. All three are Winners, but none is a clean win. v001 and v002 bought hit-rate with worse get tails. v003 left hit-rate identical and slowed every latency percentile.

## Patterns

- The "swap instead of cascade" lesson did not carry over. In cycle 0005 the probing swap left hit-rate unchanged and cut the tail. On the merged parent it raised hit-rate and hurt both the get and put tails. The likely reason is that a swap never frees a slot, so buckets stay saturated and more gets and puts take the slow full-bucket paths. Keeping items cached helps hit-rate, but the full buckets that result cost latency. The cascade's evictions were doing useful "make room" work.
- The size gate is confirmed redundant. Small-cache cells held or improved without it in v001.
- Lazy checks move cost from the median to the tail, but don't remove it. v002 kept get p50 flat by checking only on the rare arm, but get p99 rose. The recency filter accounts for only part of cycle 0005 v003's large-cache gain. The rest needs the non-hottest-hit signal, which you can't observe without touching the common path.
- Stash→target swapping leaves hit-rate exactly unchanged, the same signature cycle 0005 saw at the probing tier. Either the neighbour retry never mattered, or that branch rarely fires. Firing frequency is still unknown at both tiers.
- Noise is now the binding constraint. v003's slowdown was uniform and hit put, which its diff never touches. That points to environment drift or a shared-helper change, not the mechanism. Put moves in v002 also had no cause in the code. No variant could read its overlay source, so every mechanism here is inferred from the hypotheses.

## Open questions

- Is v003's uniform slowdown real or environmental? A same-session re-run against the parent would answer this.
- Can a swap be combined with occasional eviction (for example, swap only when stash or the bucket has slack) so that hit-rate is kept without saturating buckets?
- How often do the full-probing and full-target guarded arms actually fire?
- Does widening or narrowing `PROBE_WINDOW` change how much of the gain v002 recovers?

## Continuation

Stop adding latency-motivated variants until an identical-baseline re-run measures per-op-time noise. That request has now been ignored for five cycles, and it makes this cohort's latency verdicts unreadable. Keep the size-gate removal. Treat swaps as a hit-rate lever that costs occupancy, not as a tail fix. If you pursue them, test a hybrid that still frees slots sometimes. Drop further recency-window tuning unless it is cheap to test alongside the noise baseline.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
