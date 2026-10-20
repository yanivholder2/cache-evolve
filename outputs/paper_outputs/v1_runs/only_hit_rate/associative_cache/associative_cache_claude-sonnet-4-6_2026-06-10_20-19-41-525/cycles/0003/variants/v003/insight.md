# Variant v003 Insight

## Helped

- hit-rate: Adding cold-first eviction (`lfu_counter == 0` victim preference) on top of mid-point insertion compounded the scan-resistance already present in the parent: new arrivals land at the midpoint with a zero counter and remain explicitly evictable even after being pushed away from the LRU front, so scan items are doubly disadvantaged without narrowing the eviction window for warm entries.

## Hurt

_(no measured dimension declined)_

## Takeaway

Cold-first eviction does deliver a hit-rate gain on narrow 16-slot buckets when combined with mid-point insertion, which challenges the cycle/0002/v002 finding that bucket widening is a "necessary prerequisite." The likely explanation is that mid-point insertion acts as a substitute for wide buckets: it concentrates cold items away from the MRU tail, giving the `counter == 0` eviction selector enough cold candidates per bucket to work effectively even at 16 slots. The two mechanisms are thus complementary rather than redundant — mid-point insertion shapes *where* cold items sit, and cold-first eviction shapes *which* items are targeted at eviction time.
