# Variant v001 Insight

## Helped

- hit-rate: Unchanged from the parent, as H01 required; the victim policy (lowest saturating access counter, same tie-breaking) was kept and the cached minimum index only changes how the victim is found, not which one is chosen.
- metadata-size: Unchanged from the parent; the per-bucket cached index did not show up in the reported type sizes, so this dimension neither confirms nor refutes the stated extra-bytes cost.

## Hurt

- per-op-time (put): Slower rather than faster, so the central claim of H01 did not hold. The cached-minimum shortcut is only useful while the cache stays valid, and the put path still pays for cache maintenance on every fill and replacement, which apparently costs more than the full-bucket scan it avoids on this workload.
- per-op-time (get): Regressed even though H01 asked for no added scan on the get path. Each get hit now also compares the hit slot against the cached minimum and invalidates it on a match, which adds a branch and a load to the hottest path.

## Takeaway

Caching the minimum-counter slot does not pay off for v003's victim policy: the counter increments on get hits keep invalidating the cache exactly when the cached slot is the minimum, so puts fall back to the full scan and still carry the bookkeeping cost. Any future attempt at a running minimum needs a way to keep the index valid across get hits (or a cheaper invalidation rule) before the put-side scan is worth removing.
