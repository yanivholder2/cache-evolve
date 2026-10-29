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
