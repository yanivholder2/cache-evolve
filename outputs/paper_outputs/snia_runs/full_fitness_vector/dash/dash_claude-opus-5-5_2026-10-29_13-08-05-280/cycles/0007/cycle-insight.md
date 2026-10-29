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
