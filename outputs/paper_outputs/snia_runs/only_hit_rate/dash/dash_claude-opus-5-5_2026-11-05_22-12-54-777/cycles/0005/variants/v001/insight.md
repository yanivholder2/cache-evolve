# Variant v001 Insight

## Helped

- hit-rate (hm_0, size 16384): The ghost used to be a 64-entry ring per segment. It is now a direct-mapped table sized to the segment's resident capacity (`Ghost::new(normal_cap + stash_cap)`, rounded up to a power of two). Keys dropped from the stash are now remembered across reuse gaps of about one full segment's worth of evictions, not just the last 64 drops. At the small cache size, many of hm_0's returning keys have gaps in that range, so more of them are re-admitted with `lfu_counter = 1` and survive the 8-slot tail scan. This step is clearly larger than the gain the 64-entry ring gave in the parent.

## Hurt

- hit-rate (hm_0, size 131072): No regression, but no real gain either. The change is at noise level. With 8× more capacity, the reuse that is still lost at this size seems to sit either beyond one cache's worth of drops or in keys where a single counter step does not save them from `lru_tail_lfu_victim` once halve-on-cap ageing runs. So reach is not the binding limit at this size.
- hit-rate (web_0): Flat at both sizes. This supports the rationale that scan keys in this trace rarely come back. Even a cache-sized ghost seldom fires, and a direct-mapped collision only overwrites a fingerprint that would not have matched anyway.
- (Not measured: the hypothesis asked for metadata-size, about 2 bytes per resident slot, roughly 40 → 42 per entry. It also asked for per-op-time, where the 64-compare scan on `put` becomes one splitmix64 mix plus one compare. Neither appears in the fitness vector. The per-op cost claim follows from the code, but it is still unverified empirically.)

## Takeaway

Making ghost reach match the segment's capacity does turn the ghost into a useful mechanism, but only where the cache is small compared with hm_0's working set. At the larger size, a cache-sized ghost changes almost nothing. As the hypothesis predicted, the next limit there is the weight of the admission bonus (a single `lfu_counter = 1` step), not how far back the ghost remembers. The direct-mapped O(1) table is a safe base for that next step: it removed the per-`put` scan concern without costing hit-rate on any trace.
