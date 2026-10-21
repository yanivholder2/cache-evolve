# Variant v001 Insight

## Helped

## Hurt

- hit-rate: The change is a no-op within noise, very slightly below v002. The ghost ring for each stash bucket holds only `GHOST_WAYS = 4` fingerprints and overwrites the oldest entry on every eviction from that bucket. A loop-like key with reuse distance just over cache size returns only after that bucket has seen many more than 4 evictions, so its fingerprint is almost always gone and few re-inserts get the pre-credit. The rare false-positive or stale matches that do pre-credit a key also let the second-chance scan protect it over a counter-0 neighbour. That neighbour may have been the better keeper, so whatever these matches add or cost is too small to measure.

## Takeaway

The ghost pre-credit (`lfu_counter = PROMOTE_THRESHOLD - 1` on a ring match in `put`) only changes what happens when the re-inserted key is hit while it is still in stash. The keys H01 targets fail for a different reason: they are evicted before any stash hit, because their reuse distance is longer than their stash residency. Lowering the hit bar to one does nothing for a key that gets zero hits. The second-chance scan adds one extra pass of protection, which is far shorter than a reuse distance of about cache size. For non-resident history to matter here, two things would have to change. The ghost would need to cover roughly a cache-size window of evictions, not 4 per bucket. A ghost hit would also need to change placement or residency, for example admitting the key directly into its target bucket, not just adjusting the promotion counter.
