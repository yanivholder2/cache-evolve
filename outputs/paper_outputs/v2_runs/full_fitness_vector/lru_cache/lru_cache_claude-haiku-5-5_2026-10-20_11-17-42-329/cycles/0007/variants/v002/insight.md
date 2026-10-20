# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Admitting keys that return from the ghost set straight into the protected segment lowered hit-rate on workload-a at size 2048, so on this trace the ghost promotions appear to push out protected entries that were already hitting rather than adding new hits.
- metadata-size: The `Node` and `LruCache` structs both grew, even though the hypothesis kept the ghost on the heap; the added fields in the cache struct for the set, ring, and sequence counter are the likely cause, and the `Node` growth is not explained by the hypothesis text and needs checking against the diff.
- per-op-time: Both `get` and `put` medians and tails rose, and `get` regressed even though the hypothesis left the hit path untouched, which points to the larger `Node` (more cache-line pressure per hit) rather than new work on the hit path; the `put` regression matches the set insert and lookup the hypothesis added on the miss path.

## Takeaway

Protected-segment admission for ghost hits is the wrong lever for this workload: the variant's own hit-rate loss shows that recency-based re-reference of evicted keys did not identify keys worth protecting on workload-a. The ghost idea from cycle 0006 is not supported by this hashed-set version, so the next attempt should either keep ghost hits probationary or measure the ghost's hit rate before changing admission.
