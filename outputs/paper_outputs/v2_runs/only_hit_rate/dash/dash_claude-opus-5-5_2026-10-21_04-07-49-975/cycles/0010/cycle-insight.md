# Cycle 0010 Insight

## Cohort outcome

All three variants were built on cycle-0009 v002 (halved recency-gated ghost). v001 shrank the ghost again, to a quarter. v002 sent untouched demoted normal-tier victims to stash probation instead of MRU. v003 sent very recent ghost hits straight to the target bucket. v001 won by holding hit-rate within noise (slightly up) at a smaller ghost size. v002 lost heavily. v003 was neutral.

## Patterns

- The ghost-size floor is lower than expected. A quarter-size direct-mapped ghost still loses no hits, so the age gate, not table size, is still the limit. Two shrinks in a row each came out slightly positive, which looks like noise or the cheap removal of stale ghosts, not lost evidence.
- The "filter protected admission" rule from 0004–0009 has a clear limit. Demotion to stash MRU is a second chance that pays off, not a leak. Keys that earned normal-tier residency carry reuse evidence even when they go untouched, likely because their reuse distance is longer than their stay in the normal tier. This is the first large hit-rate loss in several cycles, and it came from filtering a route that was already near its optimum.
- Routing between the stash and the target is close to zero-sum. Admitting a key straight to target demotes a resident back to stash MRU, so the stash slot it frees is filled again at once. Very recent ghost hits promote reliably anyway, so skipping probation changes only when they arrive, not whether they survive.
- Every admission route into protected space now appears tuned on this trace: ghost recency, BIP, fresh-key doorkeeper, tier ratio, demotion placement, and direct-to-target. The only gains left are in metadata, and those are still unmeasured. For the eighth cycle running, writers could not inspect source or instrumentation, and evaluation is one LIRS trace at one size, hit-rate only.

## Open questions

- Where is the actual ghost-size floor (an eighth, a sixteenth)? Is it set by collisions or by the epoch gate?
- Did v003's age ≤ 1 branch actually fire, or mostly fall back? Unchanged results look the same as a no-op.
- Did v002's "hit in normal tier" flag track exactly what it was meant to, or did reused state misclassify hot keys?

## Continuation

Build on v001 (quarter-size ghost). Drop demotion-placement filters and direct-to-target admission. Both are settled as no better than the stash-mediated design. Further ghost shrinking is worth trying only as a single step to an eighth. Hit-rate mechanisms on this trace and size are exhausted. The next cycle should treat reporting metadata-size, per-op-time and branch-fire counters, and a second trace or cache size, as prerequisites rather than proposing new admission or placement tweaks.
