# Variant v003 Insight

## Helped

- hit-rate: Widening the protected soft cap from 80% to 90% (`capacity * 9 / 10`) holds more of the re-referenced LIRS working set in the protected segment before `demote_protected_lru` pushes it back to probation, so hot keys survive longer and beat the 0.724 champion — confirming the hypothesis that the 80% split was demoting hot keys too aggressively.

## Hurt

- per-op-time: get and put p50/p99 both rose, even though the change is a single constant in `try_new` that touches no hot-path code; with the protected set 10% larger, a bigger share of gets land on protected-but-not-MRU nodes that must run the full `unlink`/`push_back` pair instead of the protected-MRU fast-path early return — though at these nanosecond scales run-to-run variance plausibly accounts for much of the move.

## Takeaway

The protected/probation ratio is a live hit-rate lever on this LIRS workload: shifting from an 80/10 to a 90/10 split is what carried the cache past the 0.724 record, validating that the prior champion under-sized its protected segment for a working set this hot. The cost is that fewer hot accesses qualify for the MRU fast path, so pushing the cap higher trades hot-path latency for retention rather than coming for free.
