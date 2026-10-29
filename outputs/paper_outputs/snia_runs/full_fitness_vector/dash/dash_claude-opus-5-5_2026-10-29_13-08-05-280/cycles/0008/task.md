# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0008

## Winners

- v003
- v002
- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants branched separately from cycle 0004 v003, and all three are Winners. v001 merged in the 24-byte Item with a per-segment tick and added the small-cache size gate. v002 replaced the full-probing cascade with a direct two-bucket swap. v003 made the probing guard strict, so any non-hottest hit resets the count. Two variants gave clean gains at no hit-rate cost: v001 improved metadata-size and latency, and v002 cut the get tail. v003 bought a small hit-rate gain with clear get-latency costs.

## Patterns

- Removing work beats deferring it, and this held again. v002's swap removed the cascade outright and gave back v003's get-tail cost. Hit-rate was identical to the last digit in every cell. *Where* the cascade sent displaced entries never mattered. Only *when* promotion fires does.
- Contention, not layout, caused cycle 0004's 24-byte regression. With a per-segment tick, the smaller Item made puts and the get tail faster, probably because the slow paths touch fewer cache lines.
- Any check on the common hit path shows up in median get. This is the third cycle with this lesson. v003's reset sits on the most common hit path, so median get rose even though the reset almost never fires.
- The stash-side size gate is now mostly redundant. Because the target-side probing guard already fixed the small-cache slip, the gate adds only a negligible small-cache gain.
- The guarded full-probing arm seems to fire rarely on these traces. v002's identical hit-rate suggests this, so the remaining tails likely come from other paths: stash→target moves or put eviction. This sits uneasily with v003's hit-rate gain on that same arm, and the two results need reconciling.
- Noise is still unmeasured. v002's and v003's put tails moved with `put` untouched. None of the three variant insights could inspect the overlay source, so their mechanisms are inferred from the hypotheses.

## Open questions

- Do v001 and v002 combine cleanly? They touch separate paths, so the gains should add.
- Can v003's strict reset be moved off the common hit path while keeping its web_0-large gain? One option is to check staleness lazily when the hottest hit happens, for example by comparing ticks.
- How often does the full-probing arm actually fire? Which paths now dominate get and put p99?
- How large is the per-op-time noise? A re-run of an identical baseline is still missing after four cycles of asking.

## Continuation

Merge v001 (24-byte Item, per-segment tick) with v002 (direct swap) as the new baseline. Consider dropping the size gate as redundant complexity. Keep v003's strict-guard idea only if the reset can leave the common hit path, for example as a lazy tick-based staleness check on the rare hottest arm. Otherwise drop it. Next, target the tail paths that remain, stash→target and put eviction. Re-run the baseline before acting on any latency-only change.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
