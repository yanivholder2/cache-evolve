# Cycle 0006 Insight

## Cohort outcome

All three variants built on the cycle-0005 SLRU winner. v001 added a 1% W-TinyLFU admission window. v002 folded in the narrow u16 conservative-update sketch. v003 picked the protected entry to demote by frequency instead of by LRU. v003 won only because it held the parent's hit-rate unchanged. v001 and v002 both landed slightly below the parent, so this cycle produced no real gain.

## Patterns

- The SLRU design is over-defended. Probation's second-chance MRU placement and the frequency-aware victim scan already protect hot keys. Changing which entry gets demoted, or adding a small window in front, rearranges entries without changing which keys get evicted.
- The window lever is now tested on clean estimates and is negative. Few keys on workload-a come back within a short recency trial, and the slots the window takes from the main region cost more hits than the window recovers. Losing ties as a first-time newcomer is not where the remaining misses are.
- In SLRU, the narrow sketch is no longer free. Probation-only sampling compares the newcomer against a few low-count entries, so small amounts of leftover collision error now flip some decisions. Width 8× looks like the floor for this design. In the non-SLRU parent it was exactly neutral.
- This is the sixth cycle with per-op-time and metadata-size unscored. H02 assumed metadata-size was on the leaderboard, but it wasn't. The cohort's only memory-saving variant therefore counted as a small loss. v003 also adds unpriced per-promotion estimate cost.
- Hit-rate on workload-a has now held flat for two cycles across policy, structure and sketch levers.

## Open questions

- Could a recency tie-break between equal estimates let the sketch go below 8× in SLRU without flipping decisions?
- Is anything left on workload-a beyond changing what enters probation? Is the plateau simply this trace's ceiling for TinyLFU-family policies?

## Continuation

Stop spending variants on SLRU-internal hit-rate tweaks. Drop small windows, demotion policy, victim sampling and aging. Before proposing any memory or latency variant, confirm that metadata-size and per-op-time are actually being scored. If they are not, such variants can only tie or lose. If they are scored, re-run the narrow sketch on SLRU with a recency tie-break between equal estimates, and price v003's extra estimates against the plain-LRU-demotion parent.
