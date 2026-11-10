# Cycle 0009 Insight

## Cohort outcome

All three variants were single-change children of cycle-5 v002. v001 used a small share of 1/5 only at capacities up to 32768. v002 merged ghost-expiry probe pairs into `entry()` calls. v003 replaced the main queue's pop/push sweep with an equivalent CLOCK ring. v001 is the first variant since cycle 5 to beat the parent on summary hit rate: hm_0 and web_0 both rose slightly at 16384, and 131072 was unchanged by construction. v002 and v003 kept hit rate bit-identical but made the `put` tail worse.

## Patterns

- Gating the change on cache size worked. Because the split only changes below the threshold, it cannot lose at the saturated 131072 cells, so any small gain at 16384 comes through cleanly. Together with cycle 5's global 10% (hurt web_0) and cycle 6's larger share (hurt everywhere), the small share at 16384 looks like it has a sweet spot near 1/5, not a monotonic trend. The gain was small, so this lever is close to used up.
- Both timing variants missed the mark in instructive ways. Cutting a probe through std `entry()` added a hidden `reserve(1)` on the many stale-tuple misses, which can set off occasional rehashes, a classic p99 cost. The CLOCK ring showed that a sweep step's cost is the slab entry access and the freq decrement, not moving the queue index. Container shape is now ruled out for both the ghost (cycle 7) and main (this cycle).
- The `put` tail tracks the number of sweep steps and rare expensive map events. It does not track per-step bookkeeping or probe counts on the ghost-expiry path. Removing a nominal operation does not help if the replacement API hides extra work.
- Hit-rate-preserving refactors reproduced the parent exactly. That confirms the eviction-order and ghost-liveness semantics are well understood, so pure implementation variants can be trusted not to drift.
- Per-entry metadata stayed flat. Only v003 grew the top-level struct, slightly.

## Open questions

- Is 1/5 the optimum at 16384, or would a nearby value (for example 1/6) or a threshold at a different capacity add a bit more without costing web_0?
- Can the ghost-expiry path drop a probe using calls that never reserve (keep `get`, then call `remove` only on a match), or is that path just not a tail driver?
- Since sweep length sets the tail, can the number of sweep steps per miss be capped without changing eviction order in practice, for example by bounding the steps and falling back to the current hand position?

## Continuation

Promote cycle-9 v001 (the size-gated 1/5 split) as the new parent. On hit rate, at most one fine-tuning probe of the share or threshold at 16384 is worth running; after that, treat hit rate as converged. Drop container reshaping of main and ghost and any `entry()`-based probe merging. On timing, target the number of sweep steps per miss or the rare rehash/`retain` events, keep eviction work bounded, and give no timing credit to ~1 ns shifts from policy-only diffs.
