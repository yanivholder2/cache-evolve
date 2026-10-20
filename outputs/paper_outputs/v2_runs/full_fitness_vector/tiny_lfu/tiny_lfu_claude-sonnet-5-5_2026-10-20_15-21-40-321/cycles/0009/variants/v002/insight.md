# Variant v002 Insight

## Helped

- hit-rate: No help, but no harm either. It stayed within the 0.001 budget, so skipping the relink on near-head hits and reusing the cached victim estimate did not change admission decisions.

## Hurt

- per-op-time: Put p99 got worse, not better. The victim-estimate cache needs a key compare and epoch check on every admission, and the epoch is bumped on every sketch increment. Almost every put follows a get or put that incremented the sketch, so the cache is nearly always invalidated and the estimate is recomputed anyway. Get p50 rose slightly, which fits the extra near-head check and epoch bump on the hit path.
- metadata-size: The `TinyLfuCache` struct grew because of the new `victim_freq_cache`, victim key and `u32` epoch fields. `Node` and the sketch did not change, so the per-entry cost and the headline summary are the same.

## Takeaway

Caching the victim's frequency estimate behind an epoch that every access invalidates removes almost no `estimate` calls, while the bookkeeping adds cost to both paths. Skipping the relink for near-head hits saved nothing measurable, because the relink was already cheap. Per-op latency here is not dominated by redundant estimates or pointer writes, so the next change should target something else, or first profile.
